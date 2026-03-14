use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
    },
    Frame,
};

use super::{block_style, default_style, highlight_style, warning_style, BLOCK_BORDERS};
use crate::data::process::{ProcessData, ProcessSortField};
use crate::i18n;

pub fn render(
    f: &mut Frame,
    area: Rect,
    data: &ProcessData,
    scroll_offset: usize,
    mouse_x: u16,
    mouse_y: u16,
) -> usize {
    let block = Block::default()
        .title(format!(" {} ", i18n::t("processes")))
        .borders(BLOCK_BORDERS)
        .style(block_style());

    let col_widths = [
        Constraint::Length(8),
        Constraint::Length(20),
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Min(0),
    ];

    let content_width = area.width.saturating_sub(2);
    let fixed_width: u16 = 8 + 20 + 10 + 12 + 8 + 10;
    let variable_width = content_width.saturating_sub(fixed_width + 6);

    let col_actual_widths = [
        8u16,
        20,
        10,
        12,
        8,
        10,
        variable_width,
    ];

    let header_cells = vec![
        format!("PID{}", data.get_sort_indicator(ProcessSortField::Pid)),
        format!(
            "{}{}",
            i18n::t("name"),
            data.get_sort_indicator(ProcessSortField::Name)
        ),
        format!(
            "{}{}",
            i18n::t("cpu_percent"),
            data.get_sort_indicator(ProcessSortField::Cpu)
        ),
        format!(
            "{}{}",
            i18n::t("memory_percent"),
            data.get_sort_indicator(ProcessSortField::Memory)
        ),
        i18n::t("memory_mb").to_string(),
        format!(
            "{}{}",
            i18n::t("status"),
            data.get_sort_indicator(ProcessSortField::Status)
        ),
        i18n::t("command").to_string(),
    ];

    let mut col_positions = vec![];
    let mut current_x = area.x + 1;
    for (_i, width) in col_actual_widths.iter().enumerate() {
        col_positions.push(current_x);
        current_x += width + 1;
    }

    let is_mouse_on_header = mouse_y == area.y + 1;

    let header_cells_iter = header_cells.iter().enumerate().map(|(i, h)| {
        let is_mouse_over_col = is_mouse_on_header
            && i < col_actual_widths.len()
            && mouse_x >= col_positions[i]
            && mouse_x < col_positions[i] + col_actual_widths[i];

        let style = if is_mouse_over_col {
            highlight_style().add_modifier(Modifier::BOLD)
        } else {
            highlight_style()
        };

        Cell::from(Line::from(Span::styled(h.as_str(), style)))
    });

    let header = Row::new(header_cells_iter)
        .style(Style::default().fg(Color::White))
        .height(1)
        .bottom_margin(1);

    let visible_rows = area.height.saturating_sub(3) as usize;

    let total_processes = data.processes.len();
    let max_scroll_offset = total_processes.saturating_sub(visible_rows);
    let scroll_offset = scroll_offset.min(max_scroll_offset);

    let visible_processes: Vec<_> = data
        .processes
        .iter()
        .skip(scroll_offset)
        .take(visible_rows)
        .collect();

    if visible_processes.is_empty() {
        let no_data_text = if data.filter.is_empty() {
            i18n::t("no_processes")
        } else {
            i18n::t("no_matching_processes")
        };

        let text = Paragraph::new(no_data_text)
            .alignment(ratatui::layout::Alignment::Center)
            .block(block)
            .style(default_style());

        f.render_widget(text, area);
        return 0;
    }

    let rows = visible_processes.iter().map(|p| {
        let cpu_style = if p.cpu_usage > 80.0 {
            warning_style()
        } else if p.cpu_usage > 50.0 {
            Style::default().fg(Color::Yellow)
        } else {
            default_style()
        };

        let mem_style = if p.memory_percent > 80.0 {
            warning_style()
        } else if p.memory_percent > 50.0 {
            Style::default().fg(Color::Yellow)
        } else {
            default_style()
        };

        let status_style = match p.status.as_str() {
            "Sleep" => Style::default().fg(Color::Cyan),
            "Run" => Style::default().fg(Color::Green),
            "Zombie" => Style::default().fg(Color::Red),
            "Stopped" => Style::default().fg(Color::Red),
            _ => default_style(),
        };

        let cells = [
            Cell::from(format!("{}", p.pid)),
            Cell::from(truncate_string(&p.name, 18)),
            Cell::from(Line::from(Span::styled(
                format!("{:.1}%", p.cpu_usage),
                cpu_style,
            ))),
            Cell::from(Line::from(Span::styled(
                format!("{:.1}%", p.memory_percent),
                mem_style,
            ))),
            Cell::from(format!("{:.1}", p.memory_mb)),
            Cell::from(Line::from(Span::styled(
                format!("{}", translate_status(&p.status)),
                status_style,
            ))),
            Cell::from(truncate_string(&p.command, 50)),
        ];

        Row::new(cells).height(1).bottom_margin(0)
    });

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let table = Table::new(rows, col_widths)
        .header(header)
        .block(block)
        .column_spacing(1);

    f.render_widget(table, content_chunks[0]);

    if total_processes > visible_rows {
        let scrollbar_area = Rect {
            x: content_chunks[1].x,
            y: content_chunks[1].y + 2,
            width: content_chunks[1].width,
            height: content_chunks[1].height.saturating_sub(2),
        };

        let mut scrollbar_state = ScrollbarState::new(total_processes).position(scroll_offset);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            scrollbar_area,
            &mut scrollbar_state,
        );
    }

    scroll_offset
}

fn translate_status(status: &str) -> String {
    match status {
        "Sleep" => i18n::t("status_sleep").to_string(),
        "Run" => i18n::t("status_run").to_string(),
        "Zombie" => i18n::t("status_zombie").to_string(),
        "Stopped" => i18n::t("status_stopped").to_string(),
        "Idle" => i18n::t("status_idle").to_string(),
        _ => status.to_string(),
    }
}

pub fn handle_scrollbar_click(
    area: Rect,
    column: u16,
    row: u16,
    data: &ProcessData,
    current_offset: usize,
) -> Option<usize> {
    handle_scrollbar_drag(area, column, row, data, current_offset)
}

pub fn handle_scrollbar_drag(
    area: Rect,
    _column: u16,
    row: u16,
    data: &ProcessData,
    _current_offset: usize,
) -> Option<usize> {
    let total_processes = data.processes.len();

    let content_height = area.height.saturating_sub(3);
    let total_items = total_processes;

    if total_items == 0 || content_height == 0 {
        return None;
    }

    let click_row = row.saturating_sub(area.y + 2);
    let ratio = if content_height > 1 {
        let ratio = (click_row as f64) / ((content_height - 1) as f64);
        ratio.min(1.0)
    } else {
        0.0
    };

    let max_offset = total_items.saturating_sub(content_height as usize);
    let new_offset = if max_offset > 0 {
        let offset = (ratio * max_offset as f64).round() as usize;
        offset.min(max_offset)
    } else {
        0
    };

    Some(new_offset)
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