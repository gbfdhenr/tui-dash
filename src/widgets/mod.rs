pub mod cpu_widget;
pub mod memory_widget;
pub mod disk_widget;
pub mod network_widget;
pub mod docker_widget;
pub mod logs_widget;
use ratatui::style::{Color, Style};
pub fn default_style() -> Style { Style::default().fg(Color::White).bg(Color::Black) }
pub fn block_style() -> Style { Style::default().fg(Color::Cyan) }
pub fn highlight_style() -> Style { Style::default().fg(Color::Black).bg(Color::Cyan) }
pub fn warning_style() -> Style { Style::default().fg(Color::Red) }
pub const GAUGE_HEIGHT: u16 = 3;
pub const BLOCK_BORDERS: ratatui::widgets::Borders = ratatui::widgets::Borders::ALL;
