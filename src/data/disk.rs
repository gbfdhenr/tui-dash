use crate::i18n;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use sysinfo::{DiskKind, Disks};

#[derive(Debug)]
pub struct DiskData {
    pub disks: Vec<(String, u64, u64, u64, u64)>,
    disks_info: Disks,
    last_read_sectors: HashMap<String, u64>,
    last_write_sectors: HashMap<String, u64>,
    last_update_time: std::time::Instant,
    pub has_warning: bool,

    #[cfg(target_os = "linux")]
    device_to_mount: HashMap<String, String>,
}

impl DiskData {
    pub fn new() -> Result<Self> {
        let disks_info = Disks::new_with_refreshed_list();
        let mut disks = Vec::new();
        let mut last_read_sectors = HashMap::new();
        let mut last_write_sectors = HashMap::new();

        #[cfg(target_os = "linux")]
        let mut device_to_mount = Self::read_device_mount_mapping();

        for disk in disks_info.list() {
            if disk.kind() == DiskKind::HDD || disk.kind() == DiskKind::SSD {
                let mount_point = disk
                    .mount_point()
                    .to_str()
                    .unwrap_or(i18n::t("unknown"))
                    .to_string();
                let device_name = disk.name().to_string_lossy().to_string();

                #[cfg(target_os = "linux")]
                {
                    device_to_mount
                        .entry(device_name)
                        .or_insert_with(|| mount_point.clone());
                }

                last_read_sectors.insert(mount_point.clone(), 0);
                last_write_sectors.insert(mount_point.clone(), 0);
            }
        }

        #[cfg(target_os = "linux")]
        Self::update_disks(
            &disks_info,
            &mut disks,
            &last_read_sectors,
            &last_write_sectors,
            &device_to_mount,
            0.0,
        )?;

        #[cfg(target_os = "windows")]
        Self::update_disks_windows(&disks_info, &mut disks)?;

        Ok(Self {
            disks_info,
            disks,
            last_read_sectors,
            last_write_sectors,
            last_update_time: std::time::Instant::now(),
            has_warning: false,

            #[cfg(target_os = "linux")]
            device_to_mount,
        })
    }

    pub fn update(&mut self) -> Result<()> {
        self.disks.clear();
        self.disks_info.refresh();
        self.has_warning = false;

        let now = std::time::Instant::now();

        #[cfg(target_os = "linux")]
        {
            let elapsed_secs = self.last_update_time.elapsed().as_secs_f64();

            self.device_to_mount = Self::read_device_mount_mapping();

            for disk in self.disks_info.list() {
                if disk.kind() == DiskKind::HDD || disk.kind() == DiskKind::SSD {
                    let mount_point = disk
                        .mount_point()
                        .to_str()
                        .unwrap_or(i18n::t("unknown"))
                        .to_string();
                    let device_name = disk.name().to_string_lossy().to_string();
                    self.device_to_mount
                        .entry(device_name)
                        .or_insert_with(|| mount_point.clone());
                }
            }

            Self::update_disks(
                &self.disks_info,
                &mut self.disks,
                &self.last_read_sectors,
                &self.last_write_sectors,
                &self.device_to_mount,
                elapsed_secs,
            )?;

            if let Ok(disk_stats) = Self::read_disk_stats() {
                for (device_name, (read_sectors, write_sectors)) in disk_stats {
                    if let Some(mount_point) = self.device_to_mount.get(&device_name).cloned() {
                        self.last_read_sectors
                            .insert(mount_point.clone(), read_sectors);
                        self.last_write_sectors
                            .insert(mount_point.clone(), write_sectors);
                    } else {
                        self.last_read_sectors
                            .insert(device_name.clone(), read_sectors);
                        self.last_write_sectors
                            .insert(device_name.clone(), write_sectors);
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            self.update_windows()?;
        }

        for (_mount_point, used, total, _, _) in &self.disks {
            if *total > 0 {
                let usage_percent = (*used as f64 / *total as f64) * 100.0;
                if usage_percent > 90.0 {
                    self.has_warning = true;
                    break;
                }
            }
        }

        self.last_update_time = now;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn read_device_mount_mapping() -> HashMap<String, String> {
        let mut mapping = HashMap::new();

        if let Ok(content) = fs::read_to_string("/proc/mounts") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let device = parts.get(0).unwrap_or(&"").to_string();
                    let mount_point = parts.get(1).unwrap_or(&"").to_string();
                    if device.starts_with("/dev/") {
                        let device_name =
                            device.strip_prefix("/dev/").unwrap_or(&device).to_string();
                        mapping.insert(device_name, mount_point);
                    }
                }
            }
        }

        mapping
    }

    #[cfg(target_os = "linux")]
    fn update_disks(
        disks_info: &Disks,
        disks: &mut Vec<(String, u64, u64, u64, u64)>,
        last_read_sectors: &HashMap<String, u64>,
        last_write_sectors: &HashMap<String, u64>,
        _device_to_mount: &HashMap<String, String>,
        elapsed_secs: f64,
    ) -> Result<()> {
        let current_stats = Self::read_disk_stats()?;

        const MAX_DISK_SPEED: u64 = 10 * 1024 * 1024 * 1024;

        for disk in disks_info.list() {
            if disk.kind() == DiskKind::HDD || disk.kind() == DiskKind::SSD {
                let mount_point = disk
                    .mount_point()
                    .to_str()
                    .unwrap_or(i18n::t("unknown"))
                    .to_string();
                let total = disk.total_space();
                let used = total.saturating_sub(disk.available_space());

                let device_name = disk.name().to_string_lossy().to_string();

                const MIN_ELAPSED_SECS: f64 = 0.1;
                let min_elapsed_secs = elapsed_secs.max(MIN_ELAPSED_SECS);

                let (read_speed, write_speed) =
                    if let Some((current_read_sectors, current_write_sectors)) =
                        current_stats.get(&device_name)
                    {
                        let last_read = last_read_sectors.get(&mount_point).unwrap_or(&0);
                        let last_write = last_write_sectors.get(&mount_point).unwrap_or(&0);

                        const SECTOR_SIZE: u64 = 512;

                        let read_speed = if *current_read_sectors >= *last_read {
                            let bytes_read =
                                (*current_read_sectors - *last_read).saturating_mul(SECTOR_SIZE);
                            ((bytes_read as f64) / min_elapsed_secs).min(MAX_DISK_SPEED as f64)
                                as u64
                        } else {
                            0
                        };

                        let write_speed = if *current_write_sectors >= *last_write {
                            let bytes_written =
                                (*current_write_sectors - *last_write).saturating_mul(SECTOR_SIZE);
                            ((bytes_written as f64) / min_elapsed_secs).min(MAX_DISK_SPEED as f64)
                                as u64
                        } else {
                            0
                        };

                        (read_speed, write_speed)
                    } else {
                        (0, 0)
                    };

                disks.push((mount_point, used, total, read_speed, write_speed));
            }
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn read_disk_stats() -> Result<HashMap<String, (u64, u64)>> {
        let content = fs::read_to_string("/proc/diskstats")?;
        let mut stats = HashMap::new();

        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 14 {
                let device_name = parts.get(2).unwrap_or(&"").to_string();
                let read_sectors = parts
                    .get(5)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
                let write_sectors = parts
                    .get(9)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);

                stats.insert(device_name, (read_sectors, write_sectors));
            }
        }

        Ok(stats)
    }

    #[cfg(target_os = "windows")]
    fn update_disks_windows(
        disks_info: &Disks,
        disks: &mut Vec<(String, u64, u64, u64, u64)>,
    ) -> Result<()> {
        use std::collections::HashMap;
        use windows::core::*;
        use wmi::{COMLibrary, WMIConnection};

        let com_con = COMLibrary::new()?;
        let wmi_con = WMIConnection::new(com_con)?;

        let logical_disks: Vec<Win32_LogicalDisk> = wmi_con.query()?;
        let mut disk_map: HashMap<String, (u64, u64)> = HashMap::new();
        for disk in logical_disks {
            if let Some(device_id) = disk.DeviceID {
                let mount_point = format!("{}:", device_id);
                let total = disk.Size.unwrap_or(0);
                let free = disk.FreeSpace.unwrap_or(0);
                let used = total - free;
                disk_map.insert(mount_point.clone(), (total, used));
            }
        }

        for (mount_point, (total, used)) in disk_map {
            disks.push((mount_point, used, total, 0, 0));
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn update_windows(&mut self) -> Result<()> {
        Self::update_disks_windows(&self.disks_info, &mut self.disks)
    }
}

#[cfg(target_os = "windows")]
#[derive(serde::Deserialize)]
struct Win32_LogicalDisk {
    DeviceID: Option<String>,
    Size: Option<u64>,
    FreeSpace: Option<u64>,
}