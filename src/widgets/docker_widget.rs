use super::{block_style, default_style, highlight_style, warning_style, BLOCK_BORDERS};
use crate::data::docker::{ContainerInfo, ContainerState, DockerData};
use crate::i18n;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Row, Table},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, data: &DockerData) {
    if data.containers.is_empty() {
        let block = Block::default()
            .title(i18n::t("docker"))
            .borders(BLOCK_BORDERS)
            .style(block_style());
        let paragraph = ratatui::widgets::Paragraph::new("No running containers")
            .alignment(ratatui::layout::Alignment::Center)
            .block(block)
            .style(default_style());
        f.render_widget(paragraph, area);
        return;
    }

    let container_rows: Vec<Row> = data
        .containers
        .iter()
        .map(|container| {
            let state_style = match container.state {
                ContainerState::Running => Style::default().fg(Color::Green),
                ContainerState::Paused => Style::default().fg(Color::Yellow),
                ContainerState::Restarting => Style::default().fg(Color::Yellow),
                ContainerState::Exited => Style::default().fg(Color::Gray),
                ContainerState::Dead => Style::default().fg(Color::Red),
                ContainerState::Unknown => Style::default().fg(Color::Gray),
            };

            let cpu_style = if container.cpu_percent > 80.0 {
                warning_style()
            } else if container.cpu_percent > 50.0 {
                Style::default().fg(Color::Yellow)
            } else {
                default_style()
            };

            let mem_style = if container.memory_percent > 80.0 {
                warning_style()
            } else if container.memory_percent > 50.0 {
                Style::default().fg(Color::Yellow)
            } else {
                default_style()
            };

            Row::new(vec![
                truncate_string(&container.name, 20),
                container.image.clone(),
                container.status.clone(),
                format!("{:.1}%", container.cpu_percent),
                format!("{:.1}MB", container.memory_usage_mb),
                format!("{:.1}%", container.memory_percent),
            ])
            .style(state_style)
        })
        .collect();

    let container_table = Table::new(
        container_rows,
        [
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(15),
            ratatui::layout::Constraint::Percentage(10),
            ratatui::layout::Constraint::Percentage(15),
            ratatui::layout::Constraint::Percentage(20),
        ],
    )
    .block(
        Block::default()
            .title(format!(" {} ", i18n::t("docker")))
            .borders(BLOCK_BORDERS)
            .style(block_style()),
    )
    .header(
        Row::new(vec![
            i18n::t("container_name"),
            i18n::t("image"),
            i18n::t("status"),
            i18n::t("cpu_percent"),
            i18n::t("memory_mb"),
            i18n::t("memory_percent"),
        ])
        .style(highlight_style()),
    );

    f.render_widget(container_table, area);
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if max_len == 0 {
        return String::new();
    }

    if s.chars().count() <= max_len {
        return s.to_string();
    }

    let mut result = String::new();
    let mut char_count = 0;

    for c in s.chars() {
        let char_width = if (c as u32) > 127 { 2 } else { 1 };

        if char_count + char_width > max_len.saturating_sub(1) {
            break;
        }

        result.push(c);
        char_count += char_width;
    }

    if !result.is_empty() {
        result.push('…');
    }
    result
}