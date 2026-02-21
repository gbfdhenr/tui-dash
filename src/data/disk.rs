use crate::i18n;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use sysinfo::{Disks, DiskKind};
#[derive(Debug)]
pub struct DiskData {
    pub disks: Vec<(String, u64, u64, u64, u64)>, // (mount_point, used, total, read_speed, write_speed)
    disks_info: Disks,
    last_read_sectors: HashMap<String, u64>,
    last_write_sectors: HashMap<String, u64>,
    last_update_time: std::time::Instant,
    device_to_mount: HashMap<String, String>, // 设备名到挂载点的映射
}

impl DiskData {
    pub fn new() -> Result<Self> {
        let disks_info = Disks::new_with_refreshed_list();
        let mut disks = Vec::new();
        let mut last_read_sectors = HashMap::new();
        let mut last_write_sectors = HashMap::new();
        
        // 从 /proc/mounts 读取完整的设备-挂载点映射
        let mut device_to_mount = Self::read_device_mount_mapping();
        
        // 初始化设备到挂载点的映射
        for disk in disks_info.list() {
            if disk.kind() == DiskKind::HDD || disk.kind() == DiskKind::SSD {
                let mount_point = disk.mount_point()
                    .to_str()
                    .unwrap_or(i18n::t("unknown"))
                    .to_string();
                let device_name = disk.name().to_string_lossy().to_string();
                // 如果映射中没有，则添加
                device_to_mount.entry(device_name).or_insert_with(|| mount_point.clone());
                last_read_sectors.insert(mount_point.clone(), 0);
                last_write_sectors.insert(mount_point.clone(), 0);
            }
        }
        
        // 初始更新
        Self::update_disks(&disks_info, &mut disks, &last_read_sectors, &last_write_sectors, &device_to_mount, 0.0)?;
        
        Ok(Self { 
            disks_info, 
            disks,
            last_read_sectors,
            last_write_sectors,
            last_update_time: std::time::Instant::now(),
            device_to_mount,
        })
    }

    pub fn update(&mut self) -> Result<()> {
        self.disks.clear();
        self.disks_info.refresh();
        
        let now = std::time::Instant::now();
        let elapsed_secs = self.last_update_time.elapsed().as_secs_f64();
        
        // 从 /proc/mounts 读取最新的设备-挂载点映射
        self.device_to_mount = Self::read_device_mount_mapping();
        
        // 更新设备到挂载点的映射
        for disk in self.disks_info.list() {
            if disk.kind() == DiskKind::HDD || disk.kind() == DiskKind::SSD {
                let mount_point = disk.mount_point()
                    .to_str()
                    .unwrap_or(i18n::t("unknown"))
                    .to_string();
                let device_name = disk.name().to_string_lossy().to_string();
                // 如果映射中没有，则添加
                self.device_to_mount.entry(device_name).or_insert_with(|| mount_point.clone());
            }
        }
        
        Self::update_disks(&self.disks_info, &mut self.disks, &self.last_read_sectors, &self.last_write_sectors, &self.device_to_mount, elapsed_secs)?;
        
        // 更新最后一次的IO统计
        if let Ok(disk_stats) = Self::read_disk_stats() {
            for (device_name, (read_sectors, write_sectors)) in disk_stats {
                // 尝试通过设备名找到挂载点
                if let Some(mount_point) = self.device_to_mount.get(&device_name).cloned() {
                    self.last_read_sectors.insert(mount_point.clone(), read_sectors);
                    self.last_write_sectors.insert(mount_point.clone(), write_sectors);
                }
                // 如果找不到，使用设备名作为挂载点
                else {
                    self.last_read_sectors.insert(device_name.clone(), read_sectors);
                    self.last_write_sectors.insert(device_name.clone(), write_sectors);
                }
            }
        }
        
        self.last_update_time = now;
        Ok(())
    }

    /// 从 /proc/mounts 读取设备到挂载点的映射
    fn read_device_mount_mapping() -> HashMap<String, String> {
        let mut mapping = HashMap::new();
        
        if let Ok(content) = fs::read_to_string("/proc/mounts") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let device = parts[0].to_string();
                    let mount_point = parts[1].to_string();
                    // 只添加物理设备
                    if device.starts_with("/dev/") {
                        // 提取设备名（去掉 /dev/ 前缀）
                        let device_name = device.strip_prefix("/dev/").unwrap_or(&device).to_string();
                        mapping.insert(device_name, mount_point);
                    }
                }
            }
        }
        
        mapping
    }

    fn update_disks(
        disks_info: &Disks, 
        disks: &mut Vec<(String, u64, u64, u64, u64)>, 
        last_read_sectors: &HashMap<String, u64>,
        last_write_sectors: &HashMap<String, u64>,
        _device_to_mount: &HashMap<String, String>,
        elapsed_secs: f64,
    ) -> Result<()> {
        // 读取当前磁盘统计
        let current_stats = Self::read_disk_stats()?;
        
        for disk in disks_info.list() {
            if disk.kind() == DiskKind::HDD || disk.kind() == DiskKind::SSD {
                let mount_point = disk.mount_point()
                    .to_str()
                    .unwrap_or(i18n::t("unknown"))
                    .to_string();
                let total = disk.total_space();
                let used = total.saturating_sub(disk.available_space());
                
                // 获取设备名
                let device_name = disk.name().to_string_lossy().to_string();
                
                // 计算读写速度
                // 使用最小时间间隔避免除零
                let min_elapsed_secs = elapsed_secs.max(0.001);
                
                let (read_speed, write_speed) = if let Some((current_read_sectors, current_write_sectors)) = current_stats.get(&device_name) {
                    let last_read = last_read_sectors.get(&mount_point).unwrap_or(&0);
                    let last_write = last_write_sectors.get(&mount_point).unwrap_or(&0);
                    
                    // 扇区大小通常是512字节
                    let sector_size = 512;
                    
                    let read_speed = if *current_read_sectors >= *last_read {
                        (((*current_read_sectors - *last_read) * sector_size) as f64 / min_elapsed_secs) as u64
                    } else {
                        0
                    };
                    
                    let write_speed = if *current_write_sectors >= *last_write {
                        (((*current_write_sectors - *last_write) * sector_size) as f64 / min_elapsed_secs) as u64
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
    
    fn read_disk_stats() -> Result<HashMap<String, (u64, u64)>> {
        let content = fs::read_to_string("/proc/diskstats")?;
        let mut stats = HashMap::new();
        
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 14 {
                let device_name = parts[2].to_string();
                // 第6列：读取的扇区数（从0开始计数，所以是parts[5]）
                let read_sectors = parts[5].parse::<u64>().unwrap_or(0);
                // 第10列：写入的扇区数（从0开始计数，所以是parts[9]）
                let write_sectors = parts[9].parse::<u64>().unwrap_or(0);
                
                stats.insert(device_name, (read_sectors, write_sectors));
            }
        }
        
        Ok(stats)
    }
}
