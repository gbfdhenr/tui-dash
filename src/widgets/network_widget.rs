use ratatui::{
    layout::Rect,
    widgets::{Block, Row, Table},
    Frame,
};
use crate::data::{NetworkData, bytes_to_mb};
use crate::i18n;
use super::{BLOCK_BORDERS, block_style, default_style, highlight_style};

pub fn render(f: &mut Frame, area: Rect, data: &NetworkData) {
    // 为每个网络接口创建一行
    let net_rows: Vec<Row> = data.interfaces
        .iter()
        .map(|(name, received, transmitted, rx_speed, tx_speed)| {
            // 计算MB/s速度：字节/秒 ÷ 1024 ÷ 1024 = MB/s
            let rx_speed_mb = *rx_speed as f64 / 1024.0 / 1024.0;
            let tx_speed_mb = *tx_speed as f64 / 1024.0 / 1024.0;
            
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

    let net_table = Table::new(net_rows, [
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(20),
        ])
        .block(Block::default()
            .title(i18n::t("network_interfaces"))
            .borders(BLOCK_BORDERS)
            .style(block_style()))
        .header(Row::new(vec![
            i18n::t("interface"), 
            i18n::t("received"), 
            i18n::t("sent"),
            i18n::t("receive_speed"),
            i18n::t("transmit_speed"),
        ])
            .style(highlight_style()));

    f.render_widget(net_table, area);
}
