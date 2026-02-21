use anyhow::Result;
use std::collections::HashMap;
use std::process::Command;
use std::time::{Duration, Instant};
use crate::i18n;
use crate::app::LogCategory;

const MAX_LOG_LINES: usize = 500; // 增加到500行，显示更多日志
const MAX_LINE_LENGTH: usize = 4096; // 单行最大长度

#[derive(Debug, Clone)]
pub struct LogsData {
    pub lines: Vec<String>,
    cached_logs: HashMap<LogCategory, Vec<String>>,  // 按类别缓存日志
    cache_valid: bool,         // 缓存是否有效
    last_update: Instant,
    update_interval: Duration,
}

impl LogsData {
    pub fn new() -> Self {
        let mut data = Self {
            lines: vec![],
            cached_logs: HashMap::new(),
            cache_valid: false,
            last_update: Instant::now(),
            update_interval: Duration::from_secs(2),
        };

        // 忽略初始化错误，使用空日志
        let _ = data.update();
        data
    }
    
    pub fn update(&mut self) -> Result<()> {
        if self.last_update.elapsed() < self.update_interval {
            return Ok(());
        }

        // 获取所有类别的日志
        self.get_all_logs()?;

        self.last_update = Instant::now();
        Ok(())
    }

    /// 获取指定类别的日志
    pub fn get_logs_by_category(&self, category: &LogCategory) -> Vec<String> {
        match category {
            LogCategory::All => {
                // 合并所有类别的日志
                let mut all_logs = Vec::new();
                for cat in [
                    LogCategory::System,
                    LogCategory::Kernel,
                    LogCategory::Error,
                    LogCategory::Docker,
                    LogCategory::Boot,
                ] {
                    if let Some(logs) = self.cached_logs.get(&cat) {
                        all_logs.extend(logs.clone());
                    }
                }
                all_logs
            }
            _ => {
                self.cached_logs.get(category).cloned().unwrap_or_default()
            }
        }
    }
    
    /// 清理资源（已简化，无需清理子进程）
    pub fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// 截断过长的日志行
    fn truncate_line(line: &str) -> String {
        if line.len() > MAX_LINE_LENGTH {
            format!("{}...[截断]", &line[..MAX_LINE_LENGTH])
        } else {
            line.to_string()
        }
    }
    
    #[cfg(target_os = "linux")]
    fn get_journalctl_logs_by_category(&self) -> Result<HashMap<LogCategory, Vec<String>>> {
        let mut logs_map = HashMap::new();

        // 1. 获取系统日志（最近1000条）
        if let Ok(output) = Command::new("journalctl")
            .args(["--no-pager", "-n", "1000", "--output", "short", "--system"])
            .output() {
            if output.status.success() {
                let stdout = String::from_utf8(output.stdout)?;
                let logs: Vec<String> = stdout.lines().map(|l| l.to_string()).collect();
                logs_map.insert(LogCategory::System, logs);
            }
        }

        // 2. 获取内核日志（更多行）
        if let Ok(output) = Command::new("journalctl")
            .args(["--no-pager", "-n", "500", "--output", "short", "-k"])
            .output() {
            if output.status.success() {
                let stdout = String::from_utf8(output.stdout)?;
                let logs: Vec<String> = stdout.lines().map(|l| l.to_string()).collect();
                logs_map.insert(LogCategory::Kernel, logs);
            }
        }

        // 3. 获取优先级较高的日志（错误和警告）
        if let Ok(output) = Command::new("journalctl")
            .args(["--no-pager", "-n", "200", "--output", "short", "-p", "err..warning"])
            .output() {
            if output.status.success() {
                let stdout = String::from_utf8(output.stdout)?;
                let logs: Vec<String> = stdout.lines().map(|l| l.to_string()).collect();
                logs_map.insert(LogCategory::Error, logs);
            }
        }

        // 4. 获取 Docker 容器日志
        if let Ok(docker_logs) = self.get_docker_container_logs() {
            logs_map.insert(LogCategory::Docker, docker_logs);
        }

        // 5. 获取引导日志
        if let Ok(output) = Command::new("journalctl")
            .args(["--no-pager", "-b", "--output", "short", "-n", "300"])
            .output() {
            if output.status.success() {
                let stdout = String::from_utf8(output.stdout)?;
                let logs: Vec<String> = stdout.lines().map(|l| l.to_string()).collect();
                logs_map.insert(LogCategory::Boot, logs);
            }
        }

        Ok(logs_map)
    }

    /// 获取 Docker 容器日志
    #[cfg(target_os = "linux")]
    fn get_docker_container_logs(&self) -> Result<Vec<String>> {
        use bollard::{Docker, container::LogOutput, query_parameters::LogsOptions};
        use tokio::runtime::Runtime;

        let mut all_logs = Vec::new();

        // 创建运行时
        let rt = Runtime::new()?;

        // 尝试连接 Docker
        if let Ok(docker) = Docker::connect_with_local_defaults() {
            // 获取所有容器（包括停止的）
            let containers = rt.block_on(async {
                docker.list_containers(None).await
            });

            if let Ok(containers) = containers {
                for container in containers {
                    if let Some(container_id) = container.id {
                        if let Some(names) = container.names {
                            let container_name = names.first()
                                .map(|n| n.trim_start_matches('/').to_string())
                                .unwrap_or_else(|| container_id.clone());

                            // 获取容器日志配置
                            let options = LogsOptions {
                                tail: "50".to_string(), // 获取最后50行
                                timestamps: true,
                                follow: false,
                                stdout: true,
                                stderr: true,
                                ..Default::default()
                            };

                            match rt.block_on(async {
                                let log_stream = docker.logs(&container_id, Some(options));
                                futures_util::pin_mut!(log_stream);
                                let mut logs = Vec::new();
                                while let Some(result) = futures_util::stream::StreamExt::next(&mut log_stream).await {
                                    if let Ok(log_output) = result {
                                        logs.push(log_output);
                                    }
                                }
                                Ok::<_, anyhow::Error>(logs)
                            }) {
                                Ok(log_outputs) => {
                                    for log_output in log_outputs {
                                        match log_output {
                                            LogOutput::StdOut { message } => {
                                                let log_str = String::from_utf8_lossy(&message);
                                                if !log_str.trim().is_empty() {
                                                    all_logs.push(format!("[{}][OUT] {}", container_name, log_str));
                                                }
                                            }
                                            LogOutput::StdErr { message } => {
                                                let log_str = String::from_utf8_lossy(&message);
                                                if !log_str.trim().is_empty() {
                                                    all_logs.push(format!("[{}][ERR] {}", container_name, log_str));
                                                }
                                            }
                                            LogOutput::Console { message } => {
                                                let log_str = String::from_utf8_lossy(&message);
                                                if !log_str.trim().is_empty() {
                                                    all_logs.push(format!("[{}][CON] {}", container_name, log_str));
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                Err(_) => {
                                    // 如果获取日志失败，跳过该容器
                                }
                            }
                        }
                    }
                }
            }
        }

        // 限制日志数量
        if all_logs.len() > MAX_LOG_LINES {
            all_logs.truncate(MAX_LOG_LINES);
        }

        Ok(all_logs)
    }
    
    /// 获取所有日志（混合）
    fn get_all_logs(&mut self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            // 按类别获取日志
            if let Ok(logs) = self.get_journalctl_logs_by_category() {
                self.cached_logs = logs;
                self.cache_valid = true;
            } else {
                // 如果 journalctl 失败，回退到 syslog
                if let Ok(logs) = self.get_syslog_logs() {
                    self.cached_logs.insert(LogCategory::All, logs);
                    self.cache_valid = true;
                } else {
                    let error_msg = i18n::t("log_read_failed").to_string();
                    self.cached_logs.insert(LogCategory::All, vec![error_msg]);
                }
            }

            // 默认显示系统日志
            self.lines = self.get_logs_by_category(&LogCategory::System);
        }
        #[cfg(not(target_os = "linux"))]
        {
            let msg = i18n::t("linux_only_logs").to_string();
            self.cached_logs.insert(LogCategory::All, vec![msg]);
            self.lines = vec![msg];
        }
        Ok(())
    }
    
    #[cfg(target_os = "linux")]
    fn get_syslog_logs(&self) -> Result<Vec<String>> {
        let mut all_logs = Vec::new();
        
        // 优先使用dmesg获取内核日志（用户要求）
        if let Ok(output) = Command::new("dmesg")
            .args(["-T", "-n", "200"])  // 获取200行，带时间戳
            .output() {
            if output.status.success() {
                let stdout = String::from_utf8(output.stdout)?;
                all_logs.extend(stdout.lines().map(|l| Self::truncate_line(&format!("[内核] {}", l))));
            }
        }
        
        // 尝试多个系统日志文件
        let log_files = [
            "/var/log/syslog",
            "/var/log/messages",
            "/var/log/kern.log",
            "/var/log/dmesg",
            "/var/log/auth.log",
            "/var/log/boot.log",
        ];
        
        for log_file in &log_files {
            if let Ok(output) = Command::new("tail")
                .args(["-n", "100", log_file])  // 每个文件获取100行
                .output() {
                if output.status.success() {
                    let stdout = String::from_utf8(output.stdout)?;
                    all_logs.extend(stdout.lines().map(|l| Self::truncate_line(&format!("[{}] {}", log_file, l))));
                }
            }
        }
        
        // 如果日志太少，尝试使用journalctl获取更多
        if all_logs.len() < 100 {
            if let Ok(output) = Command::new("journalctl")
                .args(["--no-pager", "-n", "300", "--output", "short"])
                .output() {
                if output.status.success() {
                    let stdout = String::from_utf8(output.stdout)?;
                    all_logs.extend(stdout.lines().map(|l| Self::truncate_line(&format!("[系统] {}", l))));
                }
            }
        }
        
        // 去重并限制行数
        all_logs.sort();
        all_logs.dedup();
        if all_logs.len() > MAX_LOG_LINES {
            all_logs.truncate(MAX_LOG_LINES);
        }
        
        Ok(all_logs)
    }
}