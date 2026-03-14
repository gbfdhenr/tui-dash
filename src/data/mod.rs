pub mod battery;
pub mod cpu;
pub mod disk;
pub mod docker;
pub mod history;
pub mod logs;
pub mod memory;
pub mod network;
pub mod process;
pub mod temperature;

pub use battery::BatteryData;
pub use cpu::CpuData;
pub use disk::DiskData;
pub use docker::DockerData;
pub use history::SystemHistory;
pub use memory::bytes_to_gb;
pub use memory::bytes_to_mb;
pub use memory::MemoryData;
pub use network::NetworkData;
pub use process::ProcessData;
pub use temperature::TemperatureData;

pub const BYTES_PER_MB: u64 = 1024 * 1024;
pub const BYTES_PER_GB: u64 = 1024 * 1024 * 1024;

pub const DEFAULT_HISTORY_POINTS: usize = 60;

#[allow(dead_code)]
pub const DEFAULT_MAX_PROCESSES: usize = 100;

#[allow(dead_code)]
pub const DEFAULT_MAX_LOG_LINES: usize = 500;

#[allow(dead_code)]
pub const DEFAULT_MAX_LINE_LENGTH: usize = 4096;

pub const DEFAULT_UPDATE_INTERVAL_MS: u64 = 1000;
pub const EVENT_POLL_INTERVAL_MS: u64 = 100;

pub const PERCENTAGE_MULTIPLIER: f32 = 100.0;

pub const MAX_SEARCH_QUERY_LENGTH: usize = 100;
pub const MAX_NETWORK_INTERFACES: usize = 50;
pub const MAX_LOG_LINES: usize = 500;