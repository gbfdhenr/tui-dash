use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Chinese,
}

impl Language {
    pub fn detect() -> Self {
        let lang_env = env::var("LANG")
            .or_else(|_| env::var("LC_ALL"))
            .or_else(|_| env::var("LC_MESSAGES"))
            .unwrap_or_else(|_| "".to_string());

        let lang_lower = lang_env.to_lowercase();
        if lang_lower.contains("zh_cn")
            || lang_lower.contains("zh_tw")
            || lang_lower.contains("zh_hk")
            || lang_lower.contains("zh_sg")
            || lang_lower.contains("zh_mo")
            || lang_lower.contains("zh-hans")
            || lang_lower.contains("zh-hant")
        {
            Language::Chinese
        } else if lang_lower.starts_with("en")
            || lang_lower.contains("en_us")
            || lang_lower.contains("en_gb")
            || lang_lower.contains("en_au")
            || lang_lower.contains("en_ca")
        {
            Language::English
        } else {
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
            "system_monitor" => "System Monitor",
            "cpu" => "CPU",
            "memory" => "Memory",
            "disk" => "Disk",
            "network" => "Network",
            "process" => "Process",
            "docker" => "Docker",
            "logs" => "Logs",
            "temperature" => "Temperature",

            "temperature_status" => "Temperature Status",
            "temperature_sensors" => "Temperature Sensors",
            "sensor" => "Sensor",
            "current_temp" => "Current",
            "max_temp" => "Max",
            "critical_temp" => "Critical",

            "battery_status" => "Battery Status",
            "no_battery" => "No battery detected",

            "global_cpu_usage" => "Global CPU Usage",
            "cpu_cores" => "CPU Cores",
            "core" => "Core",
            "usage" => "Usage",

            "memory_title" => "Memory",
            "swap_title" => "Swap",

            "disk_usage" => "Disk Usage",
            "filesystem" => "Filesystem",
            "mount_point" => "Mount Point",
            "total" => "Total",
            "used" => "Used",
            "free" => "Free",
            "percent" => "Percent",
            "read_speed" => "Read Speed",
            "write_speed" => "Write Speed",

            "network_interfaces" => "Network Interfaces",
            "interface" => "Interface",
            "received" => "Received",
            "sent" => "Sent",
            "receive_speed" => "Receive Speed",
            "transmit_speed" => "Transmit Speed",

            "processes" => "Processes",
            "no_processes" => "No processes found",
            "no_matching_processes" => "No matching processes",
            "command" => "Command",
            "memory_mb" => "Memory (MB)",
            "name" => "Name",
            "cpu_percent" => "CPU%",
            "memory_percent" => "Memory%",
            "status" => "Status",
            "pid" => "PID",
            "status_sleep" => "Sleep",
            "status_run" => "Run",
            "status_zombie" => "Zombie",
            "status_stopped" => "Stopped",
            "status_idle" => "Idle",

            "container_name" => "Container",
            "image" => "Image",
            "no_containers" => "No running containers",

            "system_logs" => "System Logs (Last 20 Lines)",
            "time" => "Time",
            "message" => "Message",
            "log_category_system" => "System Logs",
            "log_category_kernel" => "Kernel Logs",
            "log_category_error" => "Error Logs",
            "log_category_boot" => "Boot Logs",
            "log_category_all" => "All Logs",

            "gb" => "GB",
            "mb" => "MB",
            "kb" => "KB",
            "b" => "B",

            "url_request_about" => "Make HTTP requests to URLs",
            "url_help" => "The URL to request",
            "method_help" => "HTTP method (GET, POST, PUT, DELETE)",
            "data_help" => "Request data (JSON or plain text)",
            "unsupported_method" => "Unsupported HTTP method: {}",

            "data_update_failed" => "Data update failed: {}",
            "process_update_failed" => "Process data update failed: {}",
            "log_read_failed" => "Failed to read logs: {}",
            "log_update_failed" => "Log update failed: {}",
            "linux_only_logs" => "Log viewing is only supported on Linux systems",
            "windows_log_unavailable" => "Windows event logs are not available",
            "journalctl_failed" => "journalctl execution failed (status code: {})",
            "syslog_read_failed" => "Failed to read syslog (status code: {})",

            "unknown" => "unknown",
            "tcp" => "tcp",

            _ => key,
        }
    }

    fn get_chinese<'a>(&self, key: &'a str) -> &'a str {
        match key {
            "system_monitor" => "系统监控",
            "cpu" => "CPU",
            "memory" => "内存",
            "disk" => "磁盘",
            "network" => "网络",
            "process" => "进程",
            "docker" => "Docker",
            "logs" => "日志",
            "temperature" => "温度",

            "temperature_status" => "温度状态",
            "temperature_sensors" => "温度传感器",
            "sensor" => "传感器",
            "current_temp" => "当前温度",
            "max_temp" => "最高温度",
            "critical_temp" => "临界温度",

            "battery_status" => "电池状态",
            "no_battery" => "未检测到电池",

            "global_cpu_usage" => "全局CPU使用率",
            "cpu_cores" => "CPU核心",
            "core" => "核心",
            "usage" => "使用率",

            "memory_title" => "内存",
            "swap_title" => "交换分区",

            "disk_usage" => "磁盘使用情况",
            "filesystem" => "文件系统",
            "mount_point" => "挂载点",
            "total" => "总量",
            "used" => "已用",
            "free" => "可用",
            "percent" => "百分比",
            "read_speed" => "读取速度",
            "write_speed" => "写入速度",

            "network_interfaces" => "网络接口",
            "interface" => "接口",
            "received" => "接收",
            "sent" => "发送",
            "receive_speed" => "接收速度",
            "transmit_speed" => "发送速度",

            "processes" => "进程列表",
            "no_processes" => "未找到进程",
            "no_matching_processes" => "没有匹配的进程",
            "command" => "命令",
            "memory_mb" => "内存 (MB)",
            "name" => "进程名",
            "cpu_percent" => "CPU%",
            "memory_percent" => "内存%",
            "status" => "状态",
            "pid" => "进程ID",
            "status_sleep" => "休眠",
            "status_run" => "运行",
            "status_zombie" => "僵尸",
            "status_stopped" => "停止",
            "status_idle" => "空闲",

            "container_name" => "容器",
            "image" => "镜像",
            "no_containers" => "没有运行中的容器",

            "system_logs" => "系统日志 (最近20行)",
            "time" => "时间",
            "message" => "消息",
            "log_category_system" => "系统日志",
            "log_category_kernel" => "内核日志",
            "log_category_error" => "错误日志",
            "log_category_boot" => "引导日志",
            "log_category_all" => "全部日志",

            "gb" => "GB",
            "mb" => "MB",
            "kb" => "KB",
            "b" => "B",

            "url_request_about" => "向URL发送HTTP请求",
            "url_help" => "要请求的URL",
            "method_help" => "HTTP方法 (GET, POST, PUT, DELETE)",
            "data_help" => "请求数据 (JSON或纯文本)",
            "unsupported_method" => "不支持的HTTP方法: {}",

            "data_update_failed" => "数据更新失败: {}",
            "process_update_failed" => "进程数据更新失败: {}",
            "log_read_failed" => "读取日志失败: {}",
            "log_update_failed" => "日志更新失败: {}",
            "linux_only_logs" => "仅支持Linux系统查看日志",
            "windows_log_unavailable" => "无法获取Windows事件日志",
            "journalctl_failed" => "journalctl执行失败 (状态码: {})",
            "syslog_read_failed" => "读取syslog失败 (状态码: {})",

            "unknown" => "未知",
            "tcp" => "tcp",

            _ => key,
        }
    }
}

use once_cell::sync::Lazy;

static I18N: Lazy<I18n> = Lazy::new(I18n::new);

pub fn t(key: &str) -> &str {
    I18N.get(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_language_detect_chinese() {
        let original_lang = env::var("LANG").ok();
        let original_lc_all = env::var("LC_ALL").ok();
        let original_lc_messages = env::var("LC_MESSAGES").ok();

        env::set_var("LANG", "zh_CN.UTF-8");
        env::remove_var("LC_ALL");
        env::remove_var("LC_MESSAGES");
        assert_eq!(Language::detect(), Language::Chinese);

        env::set_var("LANG", "zh_TW.UTF-8");
        assert_eq!(Language::detect(), Language::Chinese);

        env::set_var("LANG", "zh-hans");
        assert_eq!(Language::detect(), Language::Chinese);

        if let Some(val) = original_lang {
            env::set_var("LANG", val);
        } else {
            env::remove_var("LANG");
        }
        if let Some(val) = original_lc_all {
            env::set_var("LC_ALL", val);
        } else {
            env::remove_var("LC_ALL");
        }
        if let Some(val) = original_lc_messages {
            env::set_var("LC_MESSAGES", val);
        } else {
            env::remove_var("LC_MESSAGES");
        }
    }

    #[test]
    fn test_language_detect_english() {
        let original_lang = env::var("LANG").ok();
        let original_lc_all = env::var("LC_ALL").ok();
        let original_lc_messages = env::var("LC_MESSAGES").ok();

        env::set_var("LANG", "en_US.UTF-8");
        env::remove_var("LC_ALL");
        env::remove_var("LC_MESSAGES");
        assert_eq!(Language::detect(), Language::English);

        env::set_var("LANG", "en_GB.UTF-8");
        assert_eq!(Language::detect(), Language::English);

        if let Some(val) = original_lang {
            env::set_var("LANG", val);
        } else {
            env::remove_var("LANG");
        }
        if let Some(val) = original_lc_all {
            env::set_var("LC_ALL", val);
        } else {
            env::remove_var("LC_ALL");
        }
        if let Some(val) = original_lc_messages {
            env::set_var("LC_MESSAGES", val);
        } else {
            env::remove_var("LC_MESSAGES");
        }
    }

    #[test]
    fn test_i18n_english_translation() {
        let i18n = I18n {
            language: Language::English,
        };
        assert_eq!(i18n.get("system_monitor"), "System Monitor");
        assert_eq!(i18n.get("cpu"), "CPU");
        assert_eq!(i18n.get("memory"), "Memory");
        assert_eq!(i18n.get("disk"), "Disk");
        assert_eq!(i18n.get("network"), "Network");
        assert_eq!(i18n.get("docker"), "Docker");
        assert_eq!(i18n.get("logs"), "Logs");
    }

    #[test]
    fn test_i18n_chinese_translation() {
        let i18n = I18n {
            language: Language::Chinese,
        };
        assert_eq!(i18n.get("system_monitor"), "系统监控");
        assert_eq!(i18n.get("cpu"), "CPU");
        assert_eq!(i18n.get("memory"), "内存");
        assert_eq!(i18n.get("disk"), "磁盘");
        assert_eq!(i18n.get("network"), "网络");
        assert_eq!(i18n.get("docker"), "Docker");
        assert_eq!(i18n.get("logs"), "日志");
    }

    #[test]
    fn test_i18n_unknown_key_fallback() {
        let i18n_en = I18n {
            language: Language::English,
        };
        let i18n_cn = I18n {
            language: Language::Chinese,
        };

        assert_eq!(i18n_en.get("unknown_key"), "unknown_key");
        assert_eq!(i18n_cn.get("unknown_key"), "unknown_key");
    }
}
