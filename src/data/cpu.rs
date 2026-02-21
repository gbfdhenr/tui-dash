use anyhow::Result;
use sysinfo::System;

#[derive(Debug)]
pub struct CpuData {
    pub global_cpu_usage: f32,
    pub core_usages: Vec<f32>,
    sys: System,
    is_first_update: bool,
}

impl CpuData {
    pub fn new() -> Result<Self> {
        let sys = System::new_all();
        Ok(Self {
            global_cpu_usage: 0.0,
            core_usages: Vec::new(),
            sys,
            is_first_update: true,
        })
    }

    pub fn update(&mut self) -> Result<()> {
        if self.is_first_update {
            // 首次更新：刷新两次以获取准确的CPU使用率
            self.sys.refresh_cpu();
            self.sys.refresh_cpu();
            self.is_first_update = false;
        } else {
            self.sys.refresh_cpu();
        }
        self.global_cpu_usage = self.sys.global_cpu_info().cpu_usage();
        self.core_usages = self.sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
        Ok(())
    }
}
