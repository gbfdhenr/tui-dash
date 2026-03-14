use anyhow::Result;
use sysinfo::System;

#[derive(Debug)]
pub struct CpuData {
    pub global_cpu_usage: f32,
    pub core_usages: Vec<f32>,
    pub cpu_frequency: u64,
    pub cpu_brand: String,
    pub cpu_cores: usize,
    sys: System,
    is_first_update: bool,
}

impl CpuData {
    pub fn new() -> Result<Self> {
        let sys = System::new_all();
        let cpu_count = sys.cpus().len();
        Ok(Self {
            global_cpu_usage: 0.0,
            core_usages: Vec::new(),
            cpu_frequency: 0,
            cpu_brand: "Unknown".to_string(),
            cpu_cores: cpu_count,
            sys,
            is_first_update: true,
        })
    }

    pub fn update(&mut self) -> Result<()> {
        if self.is_first_update {
            self.sys.refresh_cpu();
            self.sys.refresh_cpu();
            self.is_first_update = false;
        } else {
            self.sys.refresh_cpu();
        }
        self.global_cpu_usage = self.sys.global_cpu_info().cpu_usage();
        self.core_usages = self.sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

        if let Some(cpu) = self.sys.cpus().first() {
            self.cpu_frequency = cpu.frequency();
            self.cpu_brand = cpu.brand().to_string();
        }

        Ok(())
    }
}