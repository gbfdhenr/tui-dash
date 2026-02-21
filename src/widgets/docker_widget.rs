use ratatui::{
    prelude::{Alignment, Constraint, Rect, Style},
    style::{Color, Modifier},
    text::{Line, Span},
    widgets::{
        Block, Cell, Padding, Paragraph, Row, Table,
    },
};

use crate::data::docker::DockerData;
use crate::i18n;
use super::{BLOCK_BORDERS, block_style, default_style, highlight_style, warning_style};

pub fn render(f: &mut ratatui::Frame, area: Rect, data: &DockerData) {
    // 修复：替换错误的 is_available 字段为 error.is_none()
    if data.error.is_none() && data.containers.is_empty() {
        let block = Block::default()
            .title(format!(" {} ", i18n::t("docker_no_containers")))
            .borders(BLOCK_BORDERS)
            .padding(Padding::new(1, 1, 1, 1))
            .style(block_style());
        
        let text = Paragraph::new(i18n::t("no_docker_containers_message"))
            .alignment(Alignment::Center)
            .block(block)
            .style(default_style());
        
        f.render_widget(text, area);
        return;
    }

    // 显示 Docker 连接错误
    if let Some(error) = &data.error {
        let block = Block::default()
            .borders(BLOCK_BORDERS)
            .title(format!(" {} ", i18n::t("docker_error")))
            .title_alignment(Alignment::Center)
            .style(warning_style());
        
        let text = Paragraph::new(format!("{} {}", i18n::t("docker_connection_error"), error))
            .block(block)
            .alignment(Alignment::Center)
            .style(default_style());
        
        f.render_widget(text, area);
        return;
    }

    // 正常显示容器列表
    let block = Block::default()
        .title(format!(" {} ", i18n::t("docker_containers")))
        .borders(BLOCK_BORDERS)
        .style(block_style());

    let header_cells = [
        i18n::t("name"),
        i18n::t("status"),
        i18n::t("cpu_percent"),
        i18n::t("memory_percent"),
        i18n::t("ports"),
    ];
    
    let header_cells_iter = header_cells.iter()
        .map(|h| {
            Cell::from(Line::from(Span::styled(
                *h,
                highlight_style().add_modifier(Modifier::BOLD),
            )))
        });
    let header = Row::new(header_cells_iter)
        .style(Style::default().fg(Color::White))
        .height(1)
        .bottom_margin(1);

    let rows = data.containers.iter().map(|container| {
        // 修复Docker状态识别bug：使用精确的状态字符串比较
        let status_span = if container.status == "running" {
            Span::styled(container.status.clone(), Style::default().fg(Color::Green))
        } else {
            Span::styled(container.status.clone(), Style::default().fg(Color::Yellow))
        };

        // 修复：内存使用率是百分比，无需转MB，直接显示百分比
        let cpu_usage = format!("{:.1}%", container.cpu_usage);
        let memory_usage = format!("{:.1}%", container.memory_usage);  // 修复单位错误

        let cells = [
            Cell::from(container.name.clone()),
            Cell::from(Line::from(status_span)),
            Cell::from(cpu_usage),
            Cell::from(memory_usage),
            Cell::from(container.ports.clone()),
        ];
        Row::new(cells).height(1).bottom_margin(0)
    });

    let table = Table::new(rows, &[
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(35),
        ])
        .header(header)
        .block(block)
        .column_spacing(1);

    f.render_widget(table, area);
}
