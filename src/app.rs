use anyhow::Result;
use sysinfo::System;

use crate::data::{
    cpu::CpuData, disk::DiskData, logs::LogsData, memory::MemoryData, network::NetworkData,
    BatteryData, DockerData, ProcessData, SystemHistory, TemperatureData, BYTES_PER_GB, BYTES_PER_MB,
    PERCENTAGE_MULTIPLIER,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Cpu,
    Memory,
    Disk,
    Network,
    Process,
    Docker,
    Logs,
    Temperature,
}

impl ActiveTab {
    pub fn next(&self) -> Self {
        match self {
            ActiveTab::Cpu => ActiveTab::Memory,
            ActiveTab::Memory => ActiveTab::Disk,
            ActiveTab::Disk => ActiveTab::Network,
            ActiveTab::Network => ActiveTab::Process,
            #[cfg(not(target_os = "windows"))]
            ActiveTab::Process => ActiveTab::Docker,
            #[cfg(target_os = "windows")]
            ActiveTab::Process => ActiveTab::Logs,
            #[cfg(not(target_os = "windows"))]
            ActiveTab::Docker => ActiveTab::Logs,
            #[cfg(not(target_os = "windows"))]
            ActiveTab::Logs => ActiveTab::Temperature,
            #[cfg(target_os = "windows")]
            ActiveTab::Logs => ActiveTab::Temperature,
            ActiveTab::Temperature => ActiveTab::Cpu,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            ActiveTab::Cpu => ActiveTab::Temperature,
            ActiveTab::Memory => ActiveTab::Cpu,
            ActiveTab::Disk => ActiveTab::Memory,
            ActiveTab::Network => ActiveTab::Disk,
            ActiveTab::Process => ActiveTab::Network,
            #[cfg(not(target_os = "windows"))]
            ActiveTab::Docker => ActiveTab::Process,
            #[cfg(not(target_os = "windows"))]
            ActiveTab::Logs => ActiveTab::Docker,
            #[cfg(target_os = "windows")]
            ActiveTab::Logs => ActiveTab::Process,
            ActiveTab::Temperature => ActiveTab::Logs,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogCategory {
    System,
    Kernel,
    Error,
    Boot,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogLevel {
    All,
    Emerg,
    Alert,
    Crit,
    Err,
    Warning,
    Notice,
    Info,
    Debug,
}

impl LogLevel {
    pub fn next(&self) -> Self {
        match self {
            LogLevel::All => LogLevel::Emerg,
            LogLevel::Emerg => LogLevel::Alert,
            LogLevel::Alert => LogLevel::Crit,
            LogLevel::Crit => LogLevel::Err,
            LogLevel::Err => LogLevel::Warning,
            LogLevel::Warning => LogLevel::Notice,
            LogLevel::Notice => LogLevel::Info,
            LogLevel::Info => LogLevel::Debug,
            LogLevel::Debug => LogLevel::All,
        }
    }

    pub fn as_journalctl_filter(&self) -> &str {
        match self {
            LogLevel::All => "",
            LogLevel::Emerg => "emerg",
            LogLevel::Alert => "alert",
            LogLevel::Crit => "crit",
            LogLevel::Err => "err",
            LogLevel::Warning => "warning",
            LogLevel::Notice => "notice",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
        }
    }
}

impl LogCategory {
    pub fn next(&self) -> Self {
        match self {
            LogCategory::System => LogCategory::Kernel,
            LogCategory::Kernel => LogCategory::Error,
            LogCategory::Error => LogCategory::Boot,
            LogCategory::Boot => LogCategory::All,
            LogCategory::All => LogCategory::System,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            LogCategory::System => LogCategory::All,
            LogCategory::Kernel => LogCategory::System,
            LogCategory::Error => LogCategory::Kernel,
            LogCategory::Boot => LogCategory::Error,
            LogCategory::All => LogCategory::Boot,
        }
    }
}

/// 应用程序主状态结构体
///
/// 管理所有系统数据、UI状态和用户交互状态
pub struct App {
    pub system: System,
    pub cpu_data: CpuData,
    pub memory_data: MemoryData,
    pub disk_data: DiskData,
    pub network_data: NetworkData,
    pub process_data: ProcessData,
    pub logs_data: LogsData,
    pub temperature_data: TemperatureData,
    pub battery_data: BatteryData,
    pub docker_data: DockerData,
    pub history: SystemHistory,
    pub active_tab: ActiveTab,
    pub active_log_category: LogCategory,
    pub logs_scroll_offset: u16,
    pub process_scroll_offset: usize,
    pub is_dragging_scrollbar: bool,
    pub search_mode: bool,
    pub search_query: String,
    pub mouse_x: u16,
    pub mouse_y: u16,
    pub paused: bool,
    pub has_alert: bool,
}

impl App {
    /// 创建新的应用程序实例
    ///
    /// 初始化系统信息采集器、所有数据模块、历史记录和配置
    ///
    /// # Returns
    ///
    /// 返回初始化好的 App 实例
    ///
    /// # Errors
    ///
    /// 如果数据模块初始化失败，返回错误
    pub fn new() -> Result<Self> {
        let mut system = System::new_all();
        system.refresh_all();

        let core_count = system.cpus().len();

        Ok(Self {
            system,
            cpu_data: CpuData::new()?,
            memory_data: MemoryData::new()?,
            disk_data: DiskData::new()?,
            network_data: NetworkData::new()?,
            process_data: ProcessData::new()?,
            logs_data: LogsData::new(),
            temperature_data: TemperatureData::new()?,
            battery_data: BatteryData::new()?,
            docker_data: DockerData::new()?,
            history: SystemHistory::new(core_count),
            active_tab: ActiveTab::Cpu,
            active_log_category: LogCategory::System,
            logs_scroll_offset: 0,
            process_scroll_offset: 0,
            is_dragging_scrollbar: false,
            search_mode: false,
            search_query: String::new(),
            mouse_x: 0,
            mouse_y: 0,
            paused: false,
            has_alert: false,
        })
    }

    /// 更新所有系统数据
    ///
    /// 刷新系统信息并更新所有数据模块，包括 CPU、内存、磁盘、网络、进程和日志数据
    /// 同时更新历史记录
    ///
    /// # Returns
    ///
    /// 如果数据更新成功，返回 Ok(())
    ///
    /// # Errors
    ///
    /// 如果数据更新失败，返回错误
    pub fn update_data(&mut self) -> Result<()> {
        if self.paused {
            return Ok(());
        }

        self.system.refresh_all();

        self.cpu_data.update()?;
        self.memory_data.update()?;
        self.disk_data.update()?;
        self.network_data.update()?;
        self.temperature_data.update()?;
        self.battery_data.update()?;

        if self.active_tab == ActiveTab::Process {
            if let Err(_e) = self.process_data.update() {}
        }

        #[cfg(target_os = "linux")]
        if self.active_tab == ActiveTab::Docker {
            if let Err(_e) = self.docker_data.update() {}
        }

        self.history
            .cpu
            .update(self.cpu_data.global_cpu_usage, &self.cpu_data.core_usages);

        let memory_percent = if self.memory_data.total_memory > 0 {
            (self.memory_data.used_memory as f32 / self.memory_data.total_memory as f32)
                * PERCENTAGE_MULTIPLIER
        } else {
            0.0
        };
        let used_memory_gb = self.memory_data.used_memory as f32 / BYTES_PER_GB as f32;
        let swap_percent = if self.memory_data.total_swap > 0 {
            (self.memory_data.used_swap as f32 / self.memory_data.total_swap as f32)
                * PERCENTAGE_MULTIPLIER
        } else {
            0.0
        };
        self.history
            .memory
            .update(memory_percent, used_memory_gb, swap_percent);

        let (total_rx_speed, total_tx_speed): (f32, f32) = self
            .network_data
            .interfaces
            .iter()
            .fold((0.0, 0.0), |(rx, tx), (_, _, _, r, t)| {
                (
                    rx + *r as f32 / BYTES_PER_MB as f32,
                    tx + *t as f32 / BYTES_PER_MB as f32,
                )
            });
        self.history.network.update(total_rx_speed, total_tx_speed);

        if let Err(_e) = self.logs_data.update() {}

        self.has_alert = self.temperature_data.has_warning
            || self.memory_data.has_warning
            || self.disk_data.has_warning;

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

    /// 进入搜索模式
    pub fn enter_search_mode(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
    }

    /// 退出搜索模式
    pub fn exit_search_mode(&mut self) {
        self.search_mode = false;
        self.process_data.set_filter(String::new());
    }

    /// 应用搜索
    pub fn apply_search(&mut self) {
        if self.search_mode {
            self.search_mode = false;
            self.process_data.set_filter(self.search_query.clone());
        }
    }

    /// 添加字符到搜索查询
    pub fn add_to_search_query(&mut self, c: char) {
        if self.search_mode && self.search_query.len() < crate::data::MAX_SEARCH_QUERY_LENGTH {
            self.search_query.push(c);
        }
    }

    /// 从搜索查询中删除最后一个字符
    pub fn remove_from_search_query(&mut self) {
        if self.search_mode && !self.search_query.is_empty() {
            self.search_query.pop();
        }
    }
}
