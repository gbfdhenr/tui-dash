use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Chinese,
}

impl Language {
    pub fn detect() -> Self {
        // 检测系统语言环境
        // 优先检查LANG环境变量，然后检查LC_ALL和LC_MESSAGES
        let lang_env = env::var("LANG")
            .or_else(|_| env::var("LC_ALL"))
            .or_else(|_| env::var("LC_MESSAGES"))
            .unwrap_or_else(|_| "".to_string());

        // 检查语言代码是否包含中文
        // 支持各种中文环境：zh_CN, zh_TW, zh_HK, zh_SG, zh_MO
        // 也支持现代语言标记：zh-Hans, zh-Hant
        let lang_lower = lang_env.to_lowercase();
        if lang_lower.contains("zh_cn") || 
           lang_lower.contains("zh_tw") || 
           lang_lower.contains("zh_hk") ||
           lang_lower.contains("zh_sg") ||
           lang_lower.contains("zh_mo") ||
           lang_lower.contains("zh-hans") ||
           lang_lower.contains("zh-hant") {
            Language::Chinese
        } else if lang_lower.starts_with("en") || 
                  lang_lower.contains("en_us") || 
                  lang_lower.contains("en_gb") ||
                  lang_lower.contains("en_au") ||
                  lang_lower.contains("en_ca") {
            Language::English
        } else {
            // 未知语言默认使用中文（根据用户要求）
            Language::Chinese
        }
    }
}

pub struct I18n {
    language: Language,
}

impl I18n {
    pub fn new() -> Self {
        Self {
            language: Language::detect(),
        }
    }

    pub fn get<'a>(&self, key: &'a str) -> &'a str {
        match self.language {
            Language::English => self.get_english(key),
            Language::Chinese => self.get_chinese(key),
        }
    }

    fn get_english<'a>(&self, key: &'a str) -> &'a str {
        match key {
            // Main titles
            "system_monitor" => "System Monitor",
            "cpu" => "CPU",
            "memory" => "Memory",
            "disk" => "Disk",
            "network" => "Network",
            "docker" => "Docker",
            "logs" => "Logs",
            
            // CPU widget
            "global_cpu_usage" => "Global CPU Usage",
            "cpu_cores" => "CPU Cores",
            "core" => "Core",
            "usage" => "Usage",
            
            // Memory widget
            "memory_title" => "Memory",
            "swap_title" => "Swap",
            
            // Disk widget
            "disk_usage" => "Disk Usage",
            "filesystem" => "Filesystem",
            "mount_point" => "Mount Point",
            "total" => "Total",
            "used" => "Used",
            "free" => "Free",
            "percent" => "Percent",
            "read_speed" => "Read Speed",
            "write_speed" => "Write Speed",
            
            // Network widget
            "network_interfaces" => "Network Interfaces",
            "interface" => "Interface",
            "received" => "Received",
            "sent" => "Sent",
            "receive_speed" => "Receive Speed",
            "transmit_speed" => "Transmit Speed",
            
            // Docker widget
            "docker_no_containers" => "Docker (No containers running)",
            "docker_error" => "Docker (Error)",
            "docker_containers" => "Docker Containers",
            "container_id" => "Container ID",
            "image" => "Image",
            "status" => "Status",
            "name" => "Name",
            "running_status" => "running",
            "cpu_percent" => "CPU %",
            "memory_percent" => "Memory %",
            "ports" => "Ports",
            "no_docker_containers_message" => "No Docker containers are currently running.",
            "docker_connection_error" => "Failed to connect to Docker: {}",
            "docker_not_available" => "Docker not available",
            "failed_to_list_containers" => "Failed to list containers: {}",
            
            // Logs widget
            "system_logs" => "System Logs (Last 20 Lines)",
            "time" => "Time",
            "message" => "Message",
            "log_category_system" => "System Logs",
            "log_category_kernel" => "Kernel Logs",
            "log_category_error" => "Error Logs",
            "log_category_docker" => "Docker Logs",
            "log_category_boot" => "Boot Logs",
            "log_category_all" => "All Logs",
            
            // Units
            "gb" => "GB",
            "mb" => "MB",
            "kb" => "KB",
            "b" => "B",
            
            // Command line
            "url_request_about" => "Make HTTP requests to URLs",
            "url_help" => "The URL to request",
            "method_help" => "HTTP method (GET, POST, PUT, DELETE)",
            "data_help" => "Request data (JSON or plain text)",
            "unsupported_method" => "Unsupported HTTP method: {}",
            
            // Error messages
            "data_update_failed" => "⚠️  Data update failed: {}",
            "docker_update_failed" => "Warning: Failed to update Docker data: {}",
            "docker_connect_failed" => "Failed to connect to Docker: {}",
            "log_read_failed" => "⚠️  Failed to read logs: {}",
            "log_update_failed" => "日志更新失败: {}",
            "linux_only_logs" => "⚠️  Log viewing is only supported on Linux systems",
            "journalctl_failed" => "journalctl execution failed (status code: {})",
            "syslog_read_failed" => "Failed to read syslog (status code: {})",
            
            // Common terms
            "unknown" => "unknown",
            "tcp" => "tcp",
            
            // Default fallback
            _ => key,
        }
    }

    fn get_chinese<'a>(&self, key: &'a str) -> &'a str {
        match key {
            // Main titles
            "system_monitor" => "系统监控",
            "cpu" => "CPU",
            "memory" => "内存",
            "disk" => "磁盘",
            "network" => "网络",
            "docker" => "Docker",
            "logs" => "日志",
            
            // CPU widget
            "global_cpu_usage" => "全局CPU使用率",
            "cpu_cores" => "CPU核心",
            "core" => "核心",
            "usage" => "使用率",
            
            // Memory widget
            "memory_title" => "内存",
            "swap_title" => "交换分区",
            
            // Disk widget
            "disk_usage" => "磁盘使用情况",
            "filesystem" => "文件系统",
            "mount_point" => "挂载点",
            "total" => "总量",
            "used" => "已用",
            "free" => "可用",
            "percent" => "百分比",
            "read_speed" => "读取速度",
            "write_speed" => "写入速度",
            
            // Network widget
            "network_interfaces" => "网络接口",
            "interface" => "接口",
            "received" => "接收",
            "sent" => "发送",
            "receive_speed" => "接收速度",
            "transmit_speed" => "发送速度",
            
            // Docker widget
            "docker_no_containers" => "Docker (无运行中的容器)",
            "docker_error" => "Docker (错误)",
            "docker_containers" => "Docker容器",
            "container_id" => "容器ID",
            "image" => "镜像",
            "status" => "状态",
            "name" => "名称",
            "running_status" => "running",
            "cpu_percent" => "CPU %",
            "memory_percent" => "内存 %",
            "ports" => "端口",
            "no_docker_containers_message" => "当前没有运行中的Docker容器。",
            "docker_connection_error" => "连接Docker失败: {}",
            "docker_not_available" => "Docker不可用",
            "failed_to_list_containers" => "获取容器列表失败: {}",
            
            // Logs widget
            "system_logs" => "系统日志 (最近20行)",
            "time" => "时间",
            "message" => "消息",
            "log_category_system" => "系统日志",
            "log_category_kernel" => "内核日志",
            "log_category_error" => "错误日志",
            "log_category_docker" => "Docker日志",
            "log_category_boot" => "引导日志",
            "log_category_all" => "全部日志",
            
            // Units
            "gb" => "GB",
            "mb" => "MB",
            "kb" => "KB",
            "b" => "B",
            
            // Command line
            "url_request_about" => "向URL发送HTTP请求",
            "url_help" => "要请求的URL",
            "method_help" => "HTTP方法 (GET, POST, PUT, DELETE)",
            "data_help" => "请求数据 (JSON或纯文本)",
            "unsupported_method" => "不支持的HTTP方法: {}",
            
            // Error messages
            "data_update_failed" => "⚠️  数据更新失败: {}",
            "docker_update_failed" => "警告: Docker数据更新失败: {}",
            "docker_connect_failed" => "连接Docker失败: {}",
            "log_read_failed" => "⚠️  读取日志失败: {}",
            "log_update_failed" => "日志更新失败: {}",
            "linux_only_logs" => "⚠️  仅支持Linux系统查看日志",
            "journalctl_failed" => "journalctl执行失败 (状态码: {})",
            "syslog_read_failed" => "读取syslog失败 (状态码: {})",
            
            // Common terms
            "unknown" => "未知",
            "tcp" => "tcp",
            
            // Default fallback
            _ => key,
        }
    }
}

// 全局I18n实例
use once_cell::sync::Lazy;

static I18N: Lazy<I18n> = Lazy::new(I18n::new);

pub fn t(key: &str) -> &str {
    I18N.get(key)
}