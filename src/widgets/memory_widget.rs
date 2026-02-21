use ratatui::{
    layout::Rect,
    style::Color,
    widgets::{Block, Gauge},
    Frame,
};
use crate::data::{MemoryData, bytes_to_gb};
use crate::i18n;
use super::{BLOCK_BORDERS, block_style, default_style, GAUGE_HEIGHT};

pub fn render(f: &mut Frame, area: Rect, data: &MemoryData) {
    // 布局：内存 + 交换分区
    let constraints: Vec<ratatui::layout::Constraint> = if data.total_swap > 0 {
        vec![
            ratatui::layout::Constraint::Length(GAUGE_HEIGHT),
            ratatui::layout::Constraint::Length(GAUGE_HEIGHT),
        ]
    } else {
        vec![ratatui::layout::Constraint::Length(GAUGE_HEIGHT)]
    };

    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints(constraints) // 直接传 Vec，无需 as_ref()
        .split(area);

    // 渲染内存使用率（转换为GB）
    let mem_used_gb = bytes_to_gb(data.used_memory);
    let mem_total_gb = bytes_to_gb(data.total_memory);
    let mem_percent = if mem_total_gb > 0.0 {
        (mem_used_gb / mem_total_gb) * 100.0
    } else {
        0.0
    };

    // 修复内存使用率溢出bug：确保内存使用率在0-100范围内
    let mem_percent_clamped = mem_percent.clamp(0.0, 100.0);
    let mem_gauge = Gauge::default()
        .block(Block::default()
            .title(i18n::t("memory_title"))
            .borders(BLOCK_BORDERS)
            .style(block_style()))
        .gauge_style(default_style().fg(Color::Cyan))
        .percent(mem_percent_clamped as u16)
        .label(format!("{:.1}GB / {:.1}GB ({:.1}%)", mem_used_gb, mem_total_gb, mem_percent_clamped));
    f.render_widget(mem_gauge, chunks[0]);

    // 渲染交换分区（如果有）
    if data.total_swap > 0 && chunks.len() > 1 {
        let swap_used_gb = bytes_to_gb(data.used_swap);
        let swap_total_gb = bytes_to_gb(data.total_swap);
        let swap_percent = if swap_total_gb > 0.0 {
            (swap_used_gb / swap_total_gb) * 100.0
        } else {
            0.0
        };

        // 修复交换分区使用率溢出bug：确保交换分区使用率在0-100范围内
        let swap_percent_clamped = swap_percent.clamp(0.0, 100.0);
        let swap_gauge = Gauge::default()
            .block(Block::default()
                .title(i18n::t("swap_title"))
                .borders(BLOCK_BORDERS)
                .style(block_style()))
            .gauge_style(default_style().fg(Color::Magenta))
            .percent(swap_percent_clamped as u16)
            .label(format!("{:.1}GB / {:.1}GB ({:.1}%)", swap_used_gb, swap_total_gb, swap_percent_clamped));
        f.render_widget(swap_gauge, chunks[1]);
    }
}
