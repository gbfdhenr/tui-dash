use super::{block_style, default_style, highlight_style, warning_style, BLOCK_BORDERS};
use crate::data::{BatteryData, TemperatureData};
use crate::i18n;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Gauge, Row, Table},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, temp_data: &TemperatureData, battery_data: &BatteryData) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Length(5),
            ratatui::layout::Constraint::Min(0),
        ])
        .split(area);

    let warning_text = if temp_data.has_warning {
        Span::styled("⚠ 温度过高！", Style::default().fg(Color::Red))
    } else {
        Span::styled("✓ 温度正常", Style::default().fg(Color::Green))
    };

    let status_block = Block::default()
        .title(i18n::t("temperature_status"))
        .borders(BLOCK_BORDERS)
        .style(block_style());

    let status_paragraph =
        ratatui::widgets::Paragraph::new(Line::from(vec![warning_text])).block(status_block);

    f.render_widget(status_paragraph, chunks[0]);

    if battery_data.present {
        let battery_status_text = match battery_data.status {
            crate::data::battery::BatteryStatus::Charging => "充电中",
            crate::data::battery::BatteryStatus::Discharging => "放电中",
            crate::data::battery::BatteryStatus::Full => "已充满",
            crate::data::battery::BatteryStatus::Unknown => "未知",
        };

        let battery_color = if battery_data.capacity < 20 {
            Color::Red
        } else if battery_data.capacity < 50 {
            Color::Yellow
        } else {
            Color::Green
        };

        let battery_gauge = Gauge::default()
            .block(
                Block::default()
                    .title(i18n::t("battery_status"))
                    .borders(BLOCK_BORDERS)
                    .style(block_style()),
            )
            .gauge_style(Style::default().fg(battery_color))
            .percent(battery_data.capacity as u16)
            .label(format!(
                "{} {}% | {} | {} | {}",
                battery_status_text,
                battery_data.capacity,
                battery_data.format_time_remaining(),
                battery_data.format_power(),
                battery_data.format_voltage()
            ));

        f.render_widget(battery_gauge, chunks[1]);
    } else {
        let no_battery_block = Block::default()
            .title(i18n::t("battery_status"))
            .borders(BLOCK_BORDERS)
            .style(block_style());

        let no_battery_text =
            ratatui::widgets::Paragraph::new(i18n::t("no_battery")).block(no_battery_block);

        f.render_widget(no_battery_text, chunks[1]);
    }

    let temp_rows: Vec<Row> = temp_data
        .sensors
        .iter()
        .map(|sensor| {
            let temp_percent = if sensor.max_temp > 0.0 {
                (sensor.current_temp / sensor.max_temp) * 100.0
            } else {
                0.0
            };

            let style = if sensor.current_temp > 85.0 {
                warning_style()
            } else if sensor.current_temp > 70.0 {
                Style::default().fg(Color::Yellow)
            } else {
                default_style()
            };

            Row::new(vec![
                sensor.name.clone(),
                format!("{:.1}°C", sensor.current_temp),
                format!("{:.1}°C", sensor.max_temp),
                sensor
                    .critical_temp
                    .map(|t| format!("{:.1}°C", t))
                    .unwrap_or_else(|| "-".to_string()),
                format!("{:.1}%", temp_percent),
            ])
            .style(style)
        })
        .collect();

    let temp_table = Table::new(
        temp_rows,
        [
            ratatui::layout::Constraint::Percentage(30),
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(15),
            ratatui::layout::Constraint::Percentage(15),
        ],
    )
    .block(
        Block::default()
            .title(i18n::t("temperature_sensors"))
            .borders(BLOCK_BORDERS)
            .style(block_style()),
    )
    .header(
        Row::new(vec![
            i18n::t("sensor"),
            i18n::t("current_temp"),
            i18n::t("max_temp"),
            i18n::t("critical_temp"),
            i18n::t("usage"),
        ])
        .style(highlight_style()),
    );

    f.render_widget(temp_table, chunks[2]);
}