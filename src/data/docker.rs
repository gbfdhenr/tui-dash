use anyhow::Result;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: ContainerState,
    pub cpu_percent: f32,
    pub memory_usage_mb: f64,
    pub memory_limit_mb: f64,
    pub memory_percent: f32,
    pub net_rx_mb: f64,
    pub net_tx_mb: f64,
    pub created: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerState {
    Running,
    Paused,
    Restarting,
    Exited,
    Dead,
    Unknown,
}

#[derive(Debug)]
pub struct DockerData {
    pub containers: Vec<ContainerInfo>,
}

impl DockerData {
    pub fn new() -> Result<Self> {
        let mut data = Self {
            containers: Vec::new(),
        };
        let _ = data.update();
        Ok(data)
    }

    pub fn update(&mut self) -> Result<()> {
        self.containers.clear();

        let output = Command::new("docker")
            .args(["ps", "--format", "{{.ID}}\t{{.Names}}\t{{.Image}}\t{{.Status}}\t{{.CreatedAt}}"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8(output.stdout)?;

                for line in stdout.lines() {
                    if let Some(container) = self.parse_container_line(line) {
                        self.containers.push(container);
                    }
                }
            }
        }

        Ok(())
    }

    fn parse_container_line(&self, line: &str) -> Option<ContainerInfo> {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 5 {
            return None;
        }

        let id = parts[0].to_string();
        let name = parts[1].to_string();
        let image = parts[2].to_string();
        let status = parts[3].to_string();
        let created = parts[4].to_string();
        let state = self.parse_state(&status);

        let (cpu_percent, memory_usage_mb, memory_limit_mb, memory_percent, net_rx_mb, net_tx_mb) =
            self.get_container_stats(&id);

        Some(ContainerInfo {
            id,
            name,
            image,
            status,
            state,
            cpu_percent,
            memory_usage_mb,
            memory_limit_mb,
            memory_percent,
            net_rx_mb,
            net_tx_mb,
            created,
        })
    }

    fn get_container_stats(&self, container_id: &str) -> (f32, f64, f64, f32, f64, f64) {
        let output = Command::new("docker")
            .args(["stats", container_id, "--no-stream", "--format", "{{.CPUPerc}}\t{{.MemUsage}}"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8(output.stdout).unwrap_or_default();
                let parts: Vec<&str> = stdout.split('\t').collect();

                if parts.len() >= 2 {
                    let cpu_percent = parts[0]
                        .trim_end_matches('%')
                        .parse::<f32>()
                        .unwrap_or(0.0);

                    let mem_usage_str = parts[1];
                    let mem_parts: Vec<&str> = mem_usage_str.split('/').collect();
                    if mem_parts.len() >= 2 {
                        let usage_mb = parse_size_mb(mem_parts[0].trim());
                        let limit_mb = parse_size_mb(mem_parts[1].trim());
                        let memory_percent = if limit_mb > 0.0 {
                            (usage_mb / limit_mb) * 100.0
                        } else {
                            0.0
                        };

                        return (cpu_percent, usage_mb, limit_mb, memory_percent as f32, 0.0, 0.0);
                    }
                }
            }
        }

        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
    }

    fn parse_state(&self, status: &str) -> ContainerState {
        let status_lower = status.to_lowercase();
        if status_lower.contains("running") {
            ContainerState::Running
        } else if status_lower.contains("paused") {
            ContainerState::Paused
        } else if status_lower.contains("restarting") {
            ContainerState::Restarting
        } else if status_lower.contains("exited") {
            ContainerState::Exited
        } else if status_lower.contains("dead") {
            ContainerState::Dead
        } else {
            ContainerState::Unknown
        }
    }
}

fn parse_size_mb(size_str: &str) -> f64 {
    let size_str = size_str.trim();
    let (num_str, unit) = size_str.split_at(
        size_str.chars().position(|c| c.is_alphabetic()).unwrap_or(size_str.len()),
    );

    if let Ok(num) = num_str.trim().parse::<f64>() {
        match unit.to_lowercase().as_str() {
            "b" => num / 1024.0 / 1024.0,
            "kb" => num / 1024.0,
            "mb" => num,
            "gb" => num * 1024.0,
            "tb" => num * 1024.0 * 1024.0,
            _ => num,
        }
    } else {
        0.0
    }
}

impl Default for DockerData {
    fn default() -> Self {
        Self::new().expect("Failed to create default DockerData")
    }
}