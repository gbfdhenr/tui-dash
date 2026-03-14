use super::{block_style, default_style, highlight_style, BLOCK_BORDERS};
use crate::data::history::SystemHistory;
use crate::data::{bytes_to_mb, NetworkData, BYTES_PER_MB};
use crate::i18n;
use ratatui::{
    layout::Rect,
    style::Color,
    widgets::{Block, Row, Sparkline, Table},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, data: &NetworkData, history: &SystemHistory) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(8),
            ratatui::layout::Constraint::Length(8),
            ratatui::layout::Constraint::Min(0),
        ])
        .split(area);

    let available_width = chunks[0].width.saturating_sub(2) as usize;

    let rx_history_data: Vec<u64> = history
        .network
        .receive_speed
        .get_all()
        .iter()
        .map(|&v| (v * 100.0) as u64)
        .collect();

    let rx_data_limited: Vec<u64> = rx_history_data.into_iter().take(available_width).collect();

    let rx_sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(i18n::t("network_receive_history"))
                .borders(BLOCK_BORDERS)
                .style(block_style()),
        )
        .data(&rx_data_limited)
        .style(default_style().fg(Color::Cyan))
        .max(1000);
    f.render_widget(rx_sparkline, chunks[0]);

    let tx_history_data: Vec<u64> = history
        .network
        .transmit_speed
        .get_all()
        .iter()
        .map(|&v| (v * 100.0) as u64)
        .collect();

    let tx_data_limited: Vec<u64> = tx_history_data.into_iter().take(available_width).collect();

    let tx_sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(i18n::t("network_transmit_history"))
                .borders(BLOCK_BORDERS)
                .style(block_style()),
        )
        .data(&tx_data_limited)
        .style(default_style().fg(Color::Green))
        .max(1000);
    f.render_widget(tx_sparkline, chunks[1]);

    let net_rows: Vec<Row> = data
        .interfaces
        .iter()
        .map(|(name, received, transmitted, rx_speed, tx_speed)| {
            let rx_speed_mb = *rx_speed as f64 / BYTES_PER_MB as f64;
            let tx_speed_mb = *tx_speed as f64 / BYTES_PER_MB as f64;

            Row::new(vec![
                name.clone(),
                format!("{:.1}MB", bytes_to_mb(*received)),
                format!("{:.1}MB", bytes_to_mb(*transmitted)),
                format!("{:.1}MB/s", rx_speed_mb),
                format!("{:.1}MB/s", tx_speed_mb),
            ])
            .style(default_style())
        })
        .collect();

    let net_table = Table::new(
        net_rows,
        [
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(20),
        ],
    )
    .block(
        Block::default()
            .title(i18n::t("network_interfaces"))
            .borders(BLOCK_BORDERS)
            .style(block_style()),
    )
    .header(
        Row::new(vec![
            i18n::t("interface"),
            i18n::t("received"),
            i18n::t("sent"),
            i18n::t("receive_speed"),
            i18n::t("transmit_speed"),
        ])
        .style(highlight_style()),
    );

    f.render_widget(net_table, chunks[2]);
}