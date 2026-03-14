use crate::app::{LogCategory, LogLevel};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use super::{DEFAULT_MAX_LINE_LENGTH, MAX_LOG_LINES};

const MAX_LINE_LENGTH: usize = DEFAULT_MAX_LINE_LENGTH;

const ALLOWED_JOURNALCTL_ARGS: &[&str] =
    &["--no-pager", "-n", "--output", "--system", "-k", "-p", "-b"];

const ALLOWED_JOURNALCTL_LEVELS: &[&str] =
    &["emerg", "alert", "crit", "err", "error", "warning", "warn", "notice", "info", "debug"];

/// 验证 journalctl 参数是否安全
fn validate_journalctl_args(args: &[&str]) -> Result<()> {
    let mut i = 0;
    while i < args.len() {
        let arg = args[i];

        if ALLOWED_JOURNALCTL_ARGS.contains(&arg) {
            if arg == "-p" && i + 1 < args.len() {
                let level = args[i + 1];
                if !ALLOWED_JOURNALCTL_LEVELS.contains(&level) {
                    return Err(anyhow::anyhow!(
                        "Invalid journalctl argument: {} (invalid log level: {})",
                        arg,
                        level
                    ));
                }
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        if let Ok(num) = arg.parse::<u32>() {
            if num >= 1 && num <= 10000 {
                i += 1;
                continue;
            } else {
                return Err(anyhow::anyhow!(
                    "Invalid journalctl argument: {} (must be between 1 and 10000)",
                    arg
                ));
            }
        }

        return Err(anyhow::anyhow!("Invalid journalctl argument: {}", arg));
    }
    Ok(())
}

#[derive(Debug)]
pub struct LogsData {
    cached_logs: HashMap<LogCategory, Vec<String>>,
    cache_valid: bool,
    last_update: Instant,
    update_interval: Duration,
    pub log_level: LogLevel,
    #[allow(dead_code)]
    temp_files: HashMap<LogCategory, PathBuf>,
    #[allow(dead_code)]
    background_processes: Vec<std::process::Child>,
    #[allow(dead_code)]
    app_pid: u32,
    initial_load: bool,
}

impl LogsData {
    pub fn new() -> Self {
        let mut data = Self {
            cached_logs: HashMap::new(),
            cache_valid: false,
            last_update: Instant::now(),
            update_interval: Duration::from_secs(3),
            log_level: LogLevel::All,
            temp_files: HashMap::new(),
            background_processes: Vec::new(),
            app_pid: std::process::id(),
            initial_load: true,
        };

        let _ = data.read_logs_initial();
        data
    }

    pub fn toggle_log_level(&mut self) {
        self.log_level = self.log_level.next();
        self.cache_valid = false;
        self.initial_load = false;
    }

    pub fn get_log_level(&self) -> LogLevel {
        self.log_level
    }

    fn read_logs_initial(&mut self) -> Result<()> {
        self.cached_logs.clear();

        #[cfg(target_os = "linux")]
        {
            let logs_map = self.get_journalctl_logs_by_category(true)?;

            for (category, logs) in logs_map {
                self.cached_logs.insert(category, logs);
            }
        }

        #[cfg(target_os = "windows")]
        {
            let logs_map = self.get_windows_event_logs(100)?;

            for (category, logs) in logs_map {
                self.cached_logs.insert(category, logs);
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            let categories = [LogCategory::System, LogCategory::Kernel, LogCategory::Error, LogCategory::Boot];
            for category in categories {
                self.cached_logs.insert(category, Vec::new());
            }
        }

        self.cache_valid = true;
        self.last_update = Instant::now();
        Ok(())
    }

    fn read_logs_from_files(&mut self) -> Result<()> {
        self.cached_logs.clear();

        #[cfg(target_os = "linux")]
        {
            let logs_map = self.get_journalctl_logs_by_category(false)?;

            for (category, logs) in logs_map {
                self.cached_logs.insert(category, logs);
            }
        }

        #[cfg(target_os = "windows")]
        {
            let logs_map = self.get_windows_event_logs(1000)?;

            for (category, logs) in logs_map {
                self.cached_logs.insert(category, logs);
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            let categories = [LogCategory::System, LogCategory::Kernel, LogCategory::Error, LogCategory::Boot];
            for category in categories {
                self.cached_logs.insert(category, Vec::new());
            }
        }

        self.cache_valid = true;
        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        if self.last_update.elapsed() < self.update_interval {
            return Ok(());
        }

        self.read_logs_from_files()?;

        self.last_update = Instant::now();
        Ok(())
    }

    pub fn get_logs_by_category(&self, category: &LogCategory) -> Vec<String> {
        match category {
            LogCategory::All => {
                let mut all_logs = Vec::new();
                for cat in [
                    LogCategory::System,
                    LogCategory::Kernel,
                    LogCategory::Error,
                    LogCategory::Boot,
                ] {
                    if let Some(logs) = self.cached_logs.get(&cat) {
                        all_logs.extend(logs.iter().cloned());
                    }
                }
                all_logs
            }
            _ => self.cached_logs.get(category).cloned().unwrap_or_default(),
        }
    }

    pub fn cleanup(&mut self) -> Result<()> {
        self.cached_logs.clear();
        self.cache_valid = false;
        Ok(())
    }

    fn truncate_line(line: &str) -> String {
        if line.len() > MAX_LINE_LENGTH {
            format!("{}...[截断]", &line[..MAX_LINE_LENGTH])
        } else {
            line.to_string()
        }
    }

    #[cfg(target_os = "linux")]
    fn get_journalctl_logs_by_category(&self, initial: bool) -> Result<HashMap<LogCategory, Vec<String>>> {
        let mut logs_map = HashMap::new();
        let count = if initial { "100" } else { "1000" };

        logs_map.insert(
            LogCategory::System,
            self.execute_journalctl(&["--system", "-n", count]),
        );

        logs_map.insert(
            LogCategory::Kernel,
            self.execute_journalctl(&["-k", "-n", count]),
        );

        logs_map.insert(
            LogCategory::Error,
            self.execute_journalctl(&["-p", "err", "-n", count]),
        );

        logs_map.insert(
            LogCategory::Boot,
            self.execute_journalctl(&["-b", "-n", count]),
        );

        Ok(logs_map)
    }

    #[cfg(target_os = "linux")]
    fn execute_journalctl(&self, args: &[&str]) -> Vec<String> {
        if let Err(_e) = validate_journalctl_args(args) {
            return Vec::new();
        }

        let mut logs = Vec::new();
        let mut cmd_args = vec!["5", "journalctl", "--no-pager", "--output", "short"];

        let level_filter = self.log_level.as_journalctl_filter();
        if !level_filter.is_empty() {
            if let Err(_e) = validate_journalctl_args(&["-p", level_filter]) {
                return Vec::new();
            }
            cmd_args.push("-p");
            cmd_args.push(level_filter);
        }

        cmd_args.extend(args);

        if let Ok(mut child) = Command::new("timeout")
            .args(&cmd_args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            if let Some(stdout) = child.stdout.take() {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(stdout);
                for log_line in reader.lines().map_while(Result::ok) {
                    logs.push(log_line);
                    if logs.len() >= MAX_LOG_LINES {
                        break;
                    }
                }
            }
            let _ = child.wait();
        }
        logs
    }

    #[cfg(target_os = "windows")]
    fn get_windows_event_logs(&self, limit: u32) -> Result<HashMap<LogCategory, Vec<String>>> {
        use windows::core::*;
        use wmi::{COMLibrary, WMIConnection};
        let mut logs_map = HashMap::new();

        let com_con = COMLibrary::new()?;
        let wmi_con = WMIConnection::new(com_con)?;

        let limit_str = limit.to_string();

        let mut system_logs = Vec::new();
        if let Ok(mut event_iter) = wmi_con.raw_query::<Win32_NTLogEvent>(
            &format!("SELECT TOP {} * FROM Win32_NTLogEvent WHERE Logfile='System' ORDER BY TimeGenerated DESC", limit_str),
        ) {
            for event in event_iter {
                let time = event
                    .TimeGenerated
                    .unwrap_or_else(|| "未知时间".to_string());
                let event_type = event.EventType.unwrap_or(0);
                let source = event.SourceName.unwrap_or_else(|| "未知".to_string());
                let message = event.Message.unwrap_or_else(|| "".to_string());

                let type_str = match event_type {
                    1 => "错误",
                    2 => "警告",
                    3 => "信息",
                    4 => "安全审核成功",
                    5 => "安全审核失败",
                    _ => "其他",
                };

                let log_line = format!("[{}] [{}] [{}] {}", time, type_str, source, message);
                system_logs.push(Self::truncate_line(&log_line));
            }
        }
        logs_map.insert(LogCategory::System, system_logs);

        let mut app_logs = Vec::new();
        if let Ok(mut event_iter) = wmi_con.raw_query::<Win32_NTLogEvent>(
            &format!("SELECT TOP {} * FROM Win32_NTLogEvent WHERE Logfile='Application' ORDER BY TimeGenerated DESC", limit_str),
        ) {
            for event in event_iter {
                let time = event.TimeGenerated.unwrap_or_else(|| "未知时间".to_string());
                let event_type = event.EventType.unwrap_or(0);
                let source = event.SourceName.unwrap_or_else(|| "未知".to_string());
                let message = event.Message.unwrap_or_else(|| "".to_string());

                let type_str = match event_type {
                    1 => "错误",
                    2 => "警告",
                    3 => "信息",
                    4 => "安全审核成功",
                    5 => "安全审核失败",
                    _ => "其他",
                };

                let log_line = format!("[{}] [{}] [{}] {}", time, type_str, source, message);
                app_logs.push(Self::truncate_line(&log_line));
            }
        }
        logs_map.insert(LogCategory::Error, app_logs);

        let mut security_logs = Vec::new();
        if let Ok(mut event_iter) = wmi_con.raw_query::<Win32_NTLogEvent>(
            &format!("SELECT TOP {} * FROM Win32_NTLogEvent WHERE Logfile='Security' ORDER BY TimeGenerated DESC", limit_str),
        ) {
            for event in event_iter {
                let time = event
                    .TimeGenerated
                    .unwrap_or_else(|| "未知时间".to_string());
                let event_type = event.EventType.unwrap_or(0);
                let source = event.SourceName.unwrap_or_else(|| "未知".to_string());
                let message = event.Message.unwrap_or_else(|| "".to_string());

                let type_str = match event_type {
                    1 => "错误",
                    2 => "警告",
                    3 => "信息",
                    4 => "安全审核成功",
                    5 => "安全审核失败",
                    _ => "其他",
                };

                let log_line = format!("[{}] [{}] [{}] {}", time, type_str, source, message);
                security_logs.push(Self::truncate_line(&log_line));
            }
        }
        logs_map.insert(LogCategory::Boot, security_logs);

        Ok(logs_map)
    }
}

#[cfg(target_os = "windows")]
#[derive(serde::Deserialize)]
struct Win32_NTLogEvent {
    TimeGenerated: Option<String>,
    EventType: Option<i32>,
    SourceName: Option<String>,
    Message: Option<String>,
}

impl Drop for LogsData {
    fn drop(&mut self) {
        for child in &mut self.background_processes {
            let _ = child.kill();
            let _ = child.wait();
        }

        for (_, file_path) in &self.temp_files {
            if file_path.exists() {
                let _ = fs::remove_file(file_path);
            }
        }
    }
}