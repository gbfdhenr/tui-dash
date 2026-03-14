use anyhow::Result;
use sysinfo::System;

use super::{BYTES_PER_GB, BYTES_PER_MB};

#[derive(Debug)]
pub struct MemoryData {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub has_warning: bool,
    sys: System,
}

impl MemoryData {
    pub fn new() -> Result<Self> {
        let sys = System::new_all();
        let mut data = Self {
            total_memory: 0,
            used_memory: 0,
            total_swap: 0,
            used_swap: 0,
            has_warning: false,
            sys,
        };
        data.update()?;
        Ok(data)
    }

    pub fn update(&mut self) -> Result<()> {
        self.sys.refresh_memory();
        self.total_memory = self.sys.total_memory();
        self.used_memory = self.sys.used_memory();
        self.total_swap = self.sys.total_swap();
        self.used_swap = self.sys.used_swap();

        self.has_warning = false;
        if self.total_memory > 0 {
            let memory_percent = (self.used_memory as f64 / self.total_memory as f64) * 100.0;
            if memory_percent > 90.0 {
                self.has_warning = true;
            }
        }

        if self.total_swap > 0 {
            let swap_percent = (self.used_swap as f64 / self.total_swap as f64) * 100.0;
            if swap_percent > 80.0 {
                self.has_warning = true;
            }
        }

        Ok(())
    }
}

pub fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / BYTES_PER_GB as f64
}

pub fn bytes_to_mb(bytes: u64) -> f64 {
    bytes as f64 / BYTES_PER_MB as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_gb() {
        assert_eq!(bytes_to_gb(1073741824), 1.0);
        assert_eq!(bytes_to_gb(2147483648), 2.0);
        assert_eq!(bytes_to_gb(536870912), 0.5);
        assert_eq!(bytes_to_gb(0), 0.0);
    }

    #[test]
    fn test_bytes_to_mb() {
        assert_eq!(bytes_to_mb(1048576), 1.0);
        assert_eq!(bytes_to_mb(2097152), 2.0);
        assert_eq!(bytes_to_mb(524288), 0.5);
        assert_eq!(bytes_to_mb(0), 0.0);
    }

    #[test]
    fn test_bytes_to_gb_to_mb_consistency() {
        let gb_value = bytes_to_gb(1073741824);
        let mb_value = bytes_to_mb(1073741824);
        assert_eq!(mb_value, gb_value * 1024.0);
    }
}