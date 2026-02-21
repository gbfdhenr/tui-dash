use anyhow::Result;
use sysinfo::System;  // 添加 SystemExt

#[derive(Debug)]
pub struct MemoryData {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
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
            sys,
        };
        data.update()?;
        Ok(data)
    }

    pub fn update(&mut self) -> Result<()> {
        self.sys.refresh_memory();  // 需要 SystemExt
        self.total_memory = self.sys.total_memory();
        self.used_memory = self.sys.used_memory();
        self.total_swap = self.sys.total_swap();
        self.used_swap = self.sys.used_swap();
        Ok(())
    }
}

pub fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / 1024.0 / 1024.0 / 1024.0
}

pub fn bytes_to_mb(bytes: u64) -> f64 {
    bytes as f64 / 1024.0 / 1024.0
}
