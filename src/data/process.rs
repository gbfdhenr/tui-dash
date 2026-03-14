use anyhow::Result;
use sysinfo::System;


fn mask_sensitive_info(command: &str) -> String {
    let password_patterns = [
        "--password=",
        "-p=",
        "--pass=",
        "--passwd=",
        "--token=",
        "--api-key=",
        "--apikey=",
        "--secret=",
        "--key=",
        "-P ",
        "--db-password=",
        "--dbpassword=",
    ];

    let mut masked = command.to_string();

    for pattern in &password_patterns {
        if let Some(idx) = masked.find(pattern) {
            let start = idx + pattern.len();
            let end = masked[start..]
                .find(' ')
                .map(|i| start + i)
                .unwrap_or(masked.len());

            let replacement = format!("{}***{}", &masked[..start + pattern.len()], &masked[end..]);
            masked = replacement;
        }
    }

    masked
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_mb: f32,
    pub memory_percent: f32,
    pub virtual_memory_mb: f64,
    pub status: String,
    pub command: String,
    pub run_time: u64,
}

impl ProcessInfo {
    pub fn display_command(&self) -> String {
        mask_sensitive_info(&self.command)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSortField {
    Pid,
    Name,
    Cpu,
    Memory,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSortOrder {
    Ascending,
    Descending,
}

#[derive(Debug)]
pub struct ProcessData {
    pub processes: Vec<ProcessInfo>,
    pub filter: String,
    pub sort_field: ProcessSortField,
    pub sort_order: ProcessSortOrder,
    pub max_processes: usize,
    system: System,
}

impl ProcessData {
    pub fn new() -> Result<Self> {
        let mut system = System::new_all();
        system.refresh_processes();

        let mut data = Self {
            processes: Vec::new(),
            filter: String::new(),
            sort_field: ProcessSortField::Cpu,
            sort_order: ProcessSortOrder::Descending,
            max_processes: 100,
            system,
        };

        data.update()?;
        Ok(data)
    }

    pub fn update(&mut self) -> Result<()> {
        self.system.refresh_processes();

        self.processes.clear();

        let total_memory = self.system.total_memory();

        for (pid, process) in self.system.processes() {
            let memory_mb = process.memory() as f32 / super::BYTES_PER_MB as f32;
            let memory_percent = if total_memory > 0 {
                (process.memory() as f32 / total_memory as f32) * super::PERCENTAGE_MULTIPLIER
            } else {
                0.0
            };

            let virtual_memory_mb = process.virtual_memory() as f64 / super::BYTES_PER_MB as f64;

            let process_info = ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_mb,
                memory_percent,
                virtual_memory_mb,
                status: format!("{:?}", process.status()),
                command: process.cmd().join(" "),
                run_time: process.run_time(),
            };

            self.processes.push(process_info);
        }

        if !self.filter.is_empty() {
            let filter_lower = self.filter.to_lowercase();
            self.processes.retain(|p| {
                p.name.to_lowercase().contains(&filter_lower)
                    || p.command.to_lowercase().contains(&filter_lower)
                    || p.pid.to_string().contains(&filter_lower)
            });
        }

        if self.processes.len() > self.max_processes {
            self.processes.truncate(self.max_processes);
        }

        self.sort_processes();

        Ok(())
    }

    fn sort_processes(&mut self) {
        match self.sort_field {
            ProcessSortField::Pid => {
                if self.sort_order == ProcessSortOrder::Ascending {
                    self.processes.sort_by(|a, b| a.pid.cmp(&b.pid));
                } else {
                    self.processes.sort_by(|a, b| b.pid.cmp(&a.pid));
                }
            }
            ProcessSortField::Name => {
                if self.sort_order == ProcessSortOrder::Ascending {
                    self.processes.sort_by(|a, b| a.name.cmp(&b.name));
                } else {
                    self.processes.sort_by(|a, b| b.name.cmp(&a.name));
                }
            }
            ProcessSortField::Cpu => {
                if self.sort_order == ProcessSortOrder::Ascending {
                    self.processes.sort_by(|a, b| {
                        a.cpu_usage
                            .partial_cmp(&b.cpu_usage)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                } else {
                    self.processes.sort_by(|a, b| {
                        b.cpu_usage
                            .partial_cmp(&a.cpu_usage)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                }
            }
            ProcessSortField::Memory => {
                if self.sort_order == ProcessSortOrder::Ascending {
                    self.processes.sort_by(|a, b| {
                        a.memory_mb
                            .partial_cmp(&b.memory_mb)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                } else {
                    self.processes.sort_by(|a, b| {
                        b.memory_mb
                            .partial_cmp(&a.memory_mb)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                }
            }
            ProcessSortField::Status => {
                if self.sort_order == ProcessSortOrder::Ascending {
                    self.processes.sort_by(|a, b| a.status.cmp(&b.status));
                } else {
                    self.processes.sort_by(|a, b| b.status.cmp(&a.status));
                }
            }
        }
    }

    pub fn set_filter(&mut self, filter: String) {
        self.filter = filter;
    }

    pub fn toggle_sort(&mut self, field: ProcessSortField) {
        if self.sort_field == field {
            self.sort_order = match self.sort_order {
                ProcessSortOrder::Ascending => ProcessSortOrder::Descending,
                ProcessSortOrder::Descending => ProcessSortOrder::Ascending,
            };
        } else {
            self.sort_field = field;
            self.sort_order = ProcessSortOrder::Descending;
        }
    }

    pub fn get_sort_indicator(&self, field: ProcessSortField) -> &'static str {
        if self.sort_field != field {
            return "";
        }
        match self.sort_order {
            ProcessSortOrder::Ascending => " ↑",
            ProcessSortOrder::Descending => " ↓",
        }
    }
}