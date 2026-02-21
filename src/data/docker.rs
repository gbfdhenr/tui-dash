use anyhow::Result;
use bollard::{Docker, query_parameters::ListContainersOptions};
use tokio::time::{timeout, Duration};
use crate::i18n;

/// Docker 容器信息结构体（简化版本）
#[derive(Debug, Clone)]
pub struct DockerContainer {
    /// 容器名称
    pub name: String,
    /// 容器状态（如 running/exited）
    pub status: String,
    /// CPU 使用率（百分比）- 简化版本设为0
    pub cpu_usage: f32,
    /// 内存使用率（百分比）- 简化版本设为0
    pub memory_usage: f32,
    /// 暴露的端口（如 8080/tcp）
    pub ports: String,
}

/// Docker 数据管理结构体
#[derive(Debug)]
pub struct DockerData {
    /// 容器列表
    pub containers: Vec<DockerContainer>,
    /// Docker 客户端实例
    client: Option<Docker>,
    /// 错误信息（如 Docker 未运行）
    pub error: Option<String>,
    /// Tokio 运行时实例（缓存以避免重复创建）
    runtime: tokio::runtime::Runtime,
}

impl DockerData {
    /// 初始化 Docker 客户端
    pub fn new() -> Result<Self> {
        // 创建Tokio运行时
        let runtime = tokio::runtime::Runtime::new()?;
        
        // 尝试连接本地 Docker 守护进程
        let client = match Docker::connect_with_local_defaults() {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("{}", i18n::t("docker_connect_failed").replace("{}", &e.to_string()));
                None
            }
        };

        Ok(Self {
            containers: Vec::new(),
            client,
            error: None,
            runtime,
        })
    }

    /// 更新容器列表（简化版本）
    pub fn update(&mut self) -> Result<()> {
        if let Some(client) = &self.client {
            let options = ListContainersOptions {
                all: true,
                ..Default::default()
            };
            
            match self.runtime.block_on(async {
                timeout(Duration::from_secs(1), client.list_containers(Some(options))).await
            }) {
                Ok(Ok(containers)) => {
                    self.containers = containers
                        .iter()
                        .map(|c| {
                            let name = c.names.as_ref()
                                .and_then(|names| names.first())
                                .map(|n| n.trim_start_matches('/').to_string())
                                .unwrap_or_else(|| i18n::t("unknown").to_string());
                            
                            let status = c.status.as_ref()
                                .cloned()
                                .unwrap_or_else(|| i18n::t("unknown").to_string());
                            
                            let ports = c.ports.as_ref()
                                .map(|ports| {
                                    ports.iter()
                                        .filter_map(|p| {
                                            if let Some(public_port) = p.public_port {
                                                let port_type = match &p.typ {
                                                    Some(typ) => typ.to_string(),
                                                    None => i18n::t("tcp").to_string(),
                                                };
                                                Some(format!("{}/{}", public_port, port_type))
                                            } else {
                                                None
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                })
                                .unwrap_or_else(String::new);
                            
                            DockerContainer {
                                name,
                                status,
                                cpu_usage: 0.0, // 简化：不计算CPU使用率
                                memory_usage: 0.0, // 简化：不计算内存使用率
                                ports,
                            }
                        })
                        .collect();
                    self.error = None;
                }
                Ok(Err(e)) => {
                    self.error = Some(format!("{} {}", i18n::t("failed_to_list_containers"), e));
                    self.containers.clear();
                }
                Err(_) => {
                    // 超时：保持原有数据，不清除
                    if self.containers.is_empty() {
                        self.error = Some("Docker 响应超时，请检查 Docker 守护进程状态".to_string());
                    }
                }
            }
        } else {
            self.error = Some(i18n::t("docker_not_available").to_string());
            self.containers.clear();
        }
        
        Ok(())
    }
}