use ratatui::{
    layout::Rect,
    style,
    widgets::{Block, Gauge, Row, Table},
    Frame,
};
use crate::data::CpuData;
use crate::i18n;
// 移除未使用的 GAUGE_HEIGHT 导入
use super::{BLOCK_BORDERS, block_style, default_style, highlight_style};

pub fn render(f: &mut Frame, area: Rect, data: &CpuData) {
    // 布局：顶部全局CPU + 下方核心列表
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(5),
            ratatui::layout::Constraint::Min(0),
        ])
        .split(area);

    // 渲染全局CPU使用率
    // 修复CPU使用率溢出bug：确保CPU使用率在0-100范围内
    let cpu_percent = data.global_cpu_usage.clamp(0.0, 100.0);
    let global_gauge = Gauge::default()
        .block(Block::default()
            .title(i18n::t("global_cpu_usage"))
            .borders(BLOCK_BORDERS)
            .style(block_style()))
        .gauge_style(default_style().fg(style::Color::Cyan))
        .percent(cpu_percent as u16)
        .label(format!("{:.1}%", cpu_percent));
    f.render_widget(global_gauge, chunks[0]);

    // 渲染核心列表
    let core_rows: Vec<Row> = data.core_usages
        .iter()
        .enumerate()
        .map(|(i, &usage)| {
            Row::new(vec![
                format!("Core {}", i),
                format!("{:.1}%", usage),
            ])
            .style(default_style())
        })
        .collect();

    let core_table = Table::new(core_rows, [
            ratatui::layout::Constraint::Percentage(50),
            ratatui::layout::Constraint::Percentage(50),
        ])
        .block(Block::default()
            .title(i18n::t("cpu_cores"))
            .borders(BLOCK_BORDERS)
            .style(block_style()))
        .header(Row::new(vec![i18n::t("core"), i18n::t("usage")]).style(highlight_style()));

    f.render_widget(core_table, chunks[1]);
}
