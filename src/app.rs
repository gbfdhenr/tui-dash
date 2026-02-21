use anyhow::Result;
use sysinfo::System;

use crate::data::{
    cpu::CpuData, disk::DiskData, docker::DockerData, logs::LogsData, memory::MemoryData,
    network::NetworkData,
};
use crate::i18n;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Cpu,
    Memory,
    Disk,
    Network,
    Docker,
    Logs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogCategory {
    System,      // 系统日志
    Kernel,      // 内核日志
    Error,       // 警告/错误日志
    Docker,      // Docker日志
    Boot,        // 引导日志
    All,         // 全部日志
}

impl LogCategory {
    pub fn next(&self) -> Self {
        match self {
            LogCategory::System => LogCategory::Kernel,
            LogCategory::Kernel => LogCategory::Error,
            LogCategory::Error => LogCategory::Docker,
            LogCategory::Docker => LogCategory::Boot,
            LogCategory::Boot => LogCategory::All,
            LogCategory::All => LogCategory::System,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            LogCategory::System => LogCategory::All,
            LogCategory::Kernel => LogCategory::System,
            LogCategory::Error => LogCategory::Kernel,
            LogCategory::Docker => LogCategory::Error,
            LogCategory::Boot => LogCategory::Docker,
            LogCategory::All => LogCategory::Boot,
        }
    }
}

impl ActiveTab {
    pub fn next(&self) -> Self {
        match self {
            ActiveTab::Cpu => ActiveTab::Memory,
            ActiveTab::Memory => ActiveTab::Disk,
            ActiveTab::Disk => ActiveTab::Network,
            ActiveTab::Network => ActiveTab::Docker,
            ActiveTab::Docker => ActiveTab::Logs,
            ActiveTab::Logs => ActiveTab::Cpu,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            ActiveTab::Cpu => ActiveTab::Logs,
            ActiveTab::Memory => ActiveTab::Cpu,
            ActiveTab::Disk => ActiveTab::Memory,
            ActiveTab::Network => ActiveTab::Disk,
            ActiveTab::Docker => ActiveTab::Network,
            ActiveTab::Logs => ActiveTab::Docker,
        }
    }
}

pub struct App {
    pub system: System,
    pub cpu_data: CpuData,
    pub memory_data: MemoryData,
    pub disk_data: DiskData,
    pub network_data: NetworkData,
    pub docker_data: DockerData,
    pub logs_data: LogsData,
    pub active_tab: ActiveTab,
    pub active_log_category: LogCategory,
    pub logs_scroll_offset: u16,
    // 拖动状态：true 表示正在拖动滚动条
    pub is_dragging_scrollbar: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut system = System::new_all();
        system.refresh_all();

        Ok(Self {
            system,
            cpu_data: CpuData::new()?,
            memory_data: MemoryData::new()?,
            disk_data: DiskData::new()?,
            network_data: NetworkData::new()?,
            docker_data: DockerData::new()?,
            logs_data: LogsData::new(),
            active_tab: ActiveTab::Cpu,
            active_log_category: LogCategory::System,
            logs_scroll_offset: 0,
            is_dragging_scrollbar: false,
        })
    }

    pub fn update_data(&mut self) -> Result<()> {
        self.system.refresh_all();
        
        self.cpu_data.update()?;
        self.memory_data.update()?;
        self.disk_data.update()?;
        self.network_data.update()?;
        
        // Docker 数据更新失败不影响整体程序
        if let Err(e) = self.docker_data.update() {
            eprintln!("{}", i18n::t("docker_update_failed").replace("{}", &e.to_string()));
        }
        
        // 日志数据按需更新（避免高频读取）
        if self.active_tab == ActiveTab::Logs {
            if let Err(e) = self.logs_data.update() {
                // 记录日志错误但不中断应用
                eprintln!("{}", i18n::t("log_update_failed").replace("{}", &e.to_string()));
            }
        }

        Ok(())
    }

    pub fn next_tab(&mut self) {
        self.active_tab = self.active_tab.next();
    }

    pub fn previous_tab(&mut self) {
        self.active_tab = self.active_tab.previous();
    }
    
    /// 清理资源
    pub fn cleanup(&mut self) -> Result<()> {
        self.logs_data.cleanup()?;
        Ok(())
    }
}
