use anyhow::Result;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct BatteryData {
    pub present: bool,
    pub status: BatteryStatus,
    pub capacity: u32,
    pub energy_full: u64,
    pub energy_now: u64,
    pub power_now: Option<u64>,
    pub voltage_now: Option<u64>,
    pub time_to_empty: Option<u64>,
    pub time_to_full: Option<u64>,
}

impl BatteryData {
    pub fn new() -> Result<Self> {
        Ok(Self {
            present: false,
            status: BatteryStatus::Unknown,
            capacity: 0,
            energy_full: 0,
            energy_now: 0,
            power_now: None,
            voltage_now: None,
            time_to_empty: None,
            time_to_full: None,
        })
    }

    #[cfg(target_os = "linux")]
    pub fn update(&mut self) -> Result<()> {
        let power_supply_path = Path::new("/sys/class/power_supply");

        if !power_supply_path.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(power_supply_path)? {
            let entry = entry?;
            let device_path = entry.path();

            let type_path = device_path.join("type");
            if type_path.exists() {
                if let Ok(device_type) = fs::read_to_string(&type_path) {
                    if device_type.trim() == "Battery" {
                        self.read_battery_info(&device_path)?;
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn read_battery_info(&mut self, path: &Path) -> Result<()> {
        self.present = true;

        self.status = fs::read_to_string(path.join("status"))
            .ok()
            .and_then(|s| match s.trim() {
                "Charging" => Some(BatteryStatus::Charging),
                "Discharging" => Some(BatteryStatus::Discharging),
                "Full" => Some(BatteryStatus::Full),
                _ => Some(BatteryStatus::Unknown),
            })
            .unwrap_or(BatteryStatus::Unknown);

        self.capacity = fs::read_to_string(path.join("capacity"))
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);

        self.energy_full = fs::read_to_string(path.join("energy_full"))
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);

        self.energy_now = fs::read_to_string(path.join("energy_now"))
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);

        self.power_now = fs::read_to_string(path.join("power_now"))
            .ok()
            .and_then(|s| s.trim().parse().ok());

        self.voltage_now = fs::read_to_string(path.join("voltage_now"))
            .ok()
            .and_then(|s| s.trim().parse().ok());

        if let Some(power) = self.power_now {
            if power > 0 {
                match self.status {
                    BatteryStatus::Discharging => {
                        let time = self.energy_now / power;
                        self.time_to_empty = Some(time);
                        self.time_to_full = None;
                    }
                    BatteryStatus::Charging => {
                        let energy_needed = self.energy_full.saturating_sub(self.energy_now);
                        let time = energy_needed / power;
                        self.time_to_empty = None;
                        self.time_to_full = Some(time);
                    }
                    _ => {
                        self.time_to_empty = None;
                        self.time_to_full = None;
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn update(&mut self) -> Result<()> {
        use windows::core::*;
        use wmi::{COMLibrary, WMIConnection};

        let com_con = COMLibrary::new()?;
        let wmi_con = WMIConnection::new(com_con)?;

        if let Ok(batteries) = wmi_con.query::<Win32_Battery>() {
            if let Some(battery) = batteries.first() {
                self.present = true;

                self.status = match battery.BatteryStatus {
                    Some(1) => BatteryStatus::Discharging,
                    Some(2) => BatteryStatus::Charging,
                    Some(3) => BatteryStatus::Full,
                    _ => BatteryStatus::Unknown,
                };

                self.capacity = battery.EstimatedChargeRemaining.unwrap_or(0) as u32;

                let design_capacity = battery.DesignCapacity.unwrap_or(0) as u64;
                let estimated_charge = (design_capacity * self.capacity as u64) / 100;
                self.energy_full = design_capacity;
                self.energy_now = estimated_charge;

                self.power_now = None;
                self.voltage_now = battery.DesignVoltage.map(|v| v as u64);

                if let Some(_rate) = battery.EstimatedChargeRemaining {
                    if let Some(run_time) = battery.EstimatedRunTime {
                        if run_time > 0 {
                            match self.status {
                                BatteryStatus::Discharging => {
                                    self.time_to_empty = Some(run_time as u64 * 60);
                                    self.time_to_full = None;
                                }
                                BatteryStatus::Charging => {
                                    let energy_needed =
                                        self.energy_full.saturating_sub(self.energy_now);
                                    let rate = if energy_needed > 0 {
                                        energy_needed / (run_time as u64 * 60).max(1)
                                    } else {
                                        10000000
                                    };
                                    let time = energy_needed / rate.max(1);
                                    self.time_to_empty = None;
                                    self.time_to_full = Some(time);
                                }
                                _ => {
                                    self.time_to_empty = None;
                                    self.time_to_full = None;
                                }
                            }
                        }
                    }
                }
            } else {
                self.present = false;
                self.status = BatteryStatus::Unknown;
                self.capacity = 0;
                self.energy_full = 0;
                self.energy_now = 0;
                self.power_now = None;
                self.voltage_now = None;
                self.time_to_empty = None;
                self.time_to_full = None;
            }
        } else {
            self.present = false;
        }

        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    pub fn update(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn format_time_remaining(&self) -> String {
        if let Some(seconds) = self.time_to_empty.or(self.time_to_full) {
            let hours = seconds / 3600;
            let minutes = (seconds % 3600) / 60;
            format!("{}h {}m", hours, minutes)
        } else {
            "-".to_string()
        }
    }

    pub fn format_power(&self) -> String {
        if let Some(power) = self.power_now {
            let watts = power as f64 / 1_000_000.0;
            format!("{:.2}W", watts)
        } else {
            "-".to_string()
        }
    }

    pub fn format_voltage(&self) -> String {
        if let Some(voltage) = self.voltage_now {
            let volts = voltage as f64 / 1_000_000.0;
            format!("{:.2}V", volts)
        } else {
            "-".to_string()
        }
    }
}

impl Default for BatteryData {
    fn default() -> Self {
        Self::new().expect("Failed to create default BatteryData")
    }
}

#[cfg(target_os = "windows")]
#[derive(serde::Deserialize)]
struct Win32_Battery {
    BatteryStatus: Option<u16>,
    EstimatedChargeRemaining: Option<u16>,
    DesignCapacity: Option<u32>,
    DesignVoltage: Option<u32>,
    EstimatedRunTime: Option<u32>,
}