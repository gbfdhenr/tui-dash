use super::{block_style, default_style, highlight_style, BLOCK_BORDERS};
use crate::data::history::SystemHistory;
use crate::data::CpuData;
use crate::i18n;
use ratatui::{
    layout::Rect,
    style,
    widgets::{Block, Gauge, Row, Sparkline, Table},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, data: &CpuData, history: &SystemHistory) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(6),
            ratatui::layout::Constraint::Length(8),
            ratatui::layout::Constraint::Min(0),
        ])
        .split(area);

    let cpu_percent = data.global_cpu_usage.clamp(0.0, 100.0);

    let cpu_info = if !data.cpu_brand.is_empty() {
        format!(
            "{} | {} MHz | {} Cores",
            data.cpu_brand, data.cpu_frequency, data.cpu_cores
        )
    } else {
        format!("{} MHz | {} Cores", data.cpu_frequency, data.cpu_cores)
    };

    let global_gauge = Gauge::default()
        .block(
            Block::default()
                .title(i18n::t("global_cpu_usage"))
                .borders(BLOCK_BORDERS)
                .style(block_style()),
        )
        .gauge_style(default_style().fg(style::Color::Cyan))
        .percent(cpu_percent as u16)
        .label(format!("{:.1}% | {}", cpu_percent, cpu_info));
    f.render_widget(global_gauge, chunks[0]);

    let history_data: Vec<u64> = history
        .cpu
        .global_usage
        .get_all()
        .iter()
        .map(|&v| v as u64)
        .collect();
    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(i18n::t("cpu_history"))
                .borders(BLOCK_BORDERS)
                .style(block_style()),
        )
        .data(&history_data)
        .style(default_style().fg(style::Color::Green))
        .max(100);
    f.render_widget(sparkline, chunks[1]);

    let core_rows: Vec<Row> = data
        .core_usages
        .iter()
        .enumerate()
        .map(|(i, &usage)| {
            Row::new(vec![format!("Core {}", i), format!("{:.1}%", usage)]).style(default_style())
        })
        .collect();

    let core_table = Table::new(
        core_rows,
        [
            ratatui::layout::Constraint::Percentage(50),
            ratatui::layout::Constraint::Percentage(50),
        ],
    )
    .block(
        Block::default()
            .title(i18n::t("cpu_cores"))
            .borders(BLOCK_BORDERS)
            .style(block_style()),
    )
    .header(Row::new(vec![i18n::t("core"), i18n::t("usage")]).style(highlight_style()));

    f.render_widget(core_table, chunks[1]);
}