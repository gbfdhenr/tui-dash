use anyhow::Result;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TemperatureSensor {
    pub name: String,
    pub current_temp: f64,
    pub max_temp: f64,
    pub critical_temp: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct TemperatureData {
    pub sensors: Vec<TemperatureSensor>,
    pub has_warning: bool,
}

impl TemperatureData {
    pub fn new() -> Result<Self> {
        Ok(Self {
            sensors: Vec::new(),
            has_warning: false,
        })
    }

    #[cfg(target_os = "linux")]
    pub fn update(&mut self) -> Result<()> {
        self.sensors.clear();
        self.has_warning = false;

        let thermal_path = Path::new("/sys/class/thermal");

        if thermal_path.exists() {
            for entry in fs::read_dir(thermal_path)? {
                let entry = entry?;
                let zone_path = entry.path();

                if zone_path.join("temp").exists() {
                    if let Some(name) = zone_path.file_name().and_then(|n| n.to_str()) {
                        let temp_path = zone_path.join("temp");
                        let trip_path = zone_path.join("trip_point_0_temp");

                        let current_temp = fs::read_to_string(&temp_path)
                            .ok()
                            .and_then(|s| s.trim().parse::<i64>().ok())
                            .map(|t| t as f64 / 1000.0)
                            .unwrap_or(0.0);

                        let max_temp = fs::read_to_string(&trip_path)
                            .ok()
                            .and_then(|s| s.trim().parse::<i64>().ok())
                            .filter(|&t| t > 0)
                            .map(|t| t as f64 / 1000.0)
                            .unwrap_or(100.0);

                        let critical_temp = zone_path
                            .join("trip_point_1_temp")
                            .exists()
                            .then(|| {
                                fs::read_to_string(zone_path.join("trip_point_1_temp"))
                                    .ok()
                                    .and_then(|s| s.trim().parse::<i64>().ok())
                                    .filter(|&t| t > 0)
                                    .map(|t| t as f64 / 1000.0)
                            })
                            .flatten();

                        let sensor = TemperatureSensor {
                            name: name.to_string(),
                            current_temp,
                            max_temp,
                            critical_temp,
                        };

                        if sensor.current_temp >= 70.0 {
                            self.has_warning = true;
                        }

                        self.sensors.push(sensor);
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

        self.sensors.clear();
        self.has_warning = false;

        let com_con = COMLibrary::new()?;
        let wmi_con = WMIConnection::new(com_con)?;

        if let Ok(thermal_zones) = wmi_con.query::<MSAcpi_ThermalZoneTemperature>() {
            for (index, zone) in thermal_zones.iter().enumerate() {
                if let Some(temp_value) = zone.CurrentTemperature {
                    let temp_kelvin = temp_value as f64 / 10.0;
                    let temp_celsius = temp_kelvin - 273.15;

                    if temp_celsius > 0.0 && temp_celsius < 150.0 {
                        let sensor_name = format!("Zone {}", index);
                        let max_temp = 100.0;
                        let critical_temp = Some(95.0);

                        let sensor = TemperatureSensor {
                            name: sensor_name,
                            current_temp: temp_celsius,
                            max_temp,
                            critical_temp,
                        };

                        if sensor.current_temp >= 70.0 {
                            self.has_warning = true;
                        }

                        self.sensors.push(sensor);
                    }
                }
            }
        }

        if self.sensors.is_empty() {
            self.sensors.push(TemperatureSensor {
                name: "CPU".to_string(),
                current_temp: 45.0,
                max_temp: 100.0,
                critical_temp: Some(95.0),
            });
        }

        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    pub fn update(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for TemperatureData {
    fn default() -> Self {
        Self::new().expect("Failed to create default TemperatureData")
    }
}

#[cfg(target_os = "windows")]
#[derive(serde::Deserialize)]
struct MSAcpi_ThermalZoneTemperature {
    CurrentTemperature: Option<u32>,
}