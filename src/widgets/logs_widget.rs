use super::{block_style, default_style, highlight_style, BLOCK_BORDERS};
use crate::app::{LogCategory, LogLevel};
use crate::data::logs::LogsData;
use crate::i18n;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Frame,
};
use textwrap::wrap;

pub fn handle_scrollbar_click(
    area: Rect,
    column: u16,
    row: u16,
    data: &LogsData,
    _current_offset: u16,
    category: &LogCategory,
) -> Option<u16> {
    handle_scrollbar_drag(area, column, row, data, category)
}

pub fn handle_scrollbar_drag(
    area: Rect,
    _column: u16,
    row: u16,
    data: &LogsData,
    category: &LogCategory,
) -> Option<u16> {
    let logs = data.get_logs_by_category(category);

    let content_height = area.height.saturating_sub(2);
    let total_items = logs.len();

    if total_items == 0 || content_height == 0 {
        return None;
    }

    let click_row = row.saturating_sub(area.y + 1);
    let ratio = (click_row as f64) / (content_height as f64);

    let max_offset = total_items.saturating_sub(content_height as usize);
    let new_offset = (ratio * max_offset as f64) as usize;

    Some(new_offset as u16)
}

pub fn handle_content_click(
    area: Rect,
    column: u16,
    row: u16,
    data: &LogsData,
    current_offset: u16,
    category: &LogCategory,
) -> Option<u16> {
    if column < area.x + 1 || column >= area.x + area.width - 1 {
        return None;
    }

    if row < area.y + 1 || row >= area.y + area.height - 1 {
        return None;
    }

    let logs = data.get_logs_by_category(category);

    let _content_height = area.height.saturating_sub(2);
    let click_row = row - area.y - 1;

    let target_log_index = current_offset as usize + click_row as usize;

    if target_log_index >= logs.len() {
        return None;
    }

    None
}

pub fn render(
    f: &mut Frame,
    area: Rect,
    data: &LogsData,
    scroll_offset: u16,
    category: &LogCategory,
    mouse_x: u16,
    mouse_y: u16,
) -> u16 {
    render_log_content(f, area, data, scroll_offset, category, mouse_x, mouse_y)
}

fn wrap_log_line(line: &str, width: usize) -> Vec<String> {
    if width <= 8 {
        return vec![format!("    {}", line)];
    }

    let base_indent = "    ";
    let available_width = width.saturating_sub(4);

    wrap(line, available_width)
        .into_iter()
        .map(|s| format!("{}{}", base_indent, s))
        .collect()
}

fn render_log_content(
    f: &mut Frame,
    area: Rect,
    data: &LogsData,
    scroll_offset: u16,
    category: &LogCategory,
    mouse_x: u16,
    mouse_y: u16,
) -> u16 {
    let logs = data.get_logs_by_category(category);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    render_log_tabs(f, chunks[0], category, mouse_x, mouse_y);

    let log_items: Vec<ListItem> = logs
        .iter()
        .flat_map(|line| {
            let wrapped_lines = wrap_log_line(line, chunks[1].width as usize);
            wrapped_lines
                .into_iter()
                .map(|s| ListItem::new(s).style(default_style()))
        })
        .collect();

    let title = format!(
        "{} ({} 条) [L: {}]",
        get_category_name(category),
        logs.len(),
        get_log_level_name(data.get_log_level())
    );

    let log_list = List::new(log_items)
        .block(
            Block::default()
                .title(title)
                .borders(BLOCK_BORDERS)
                .style(block_style()),
        )
        .highlight_style(highlight_style());

    let total_items = logs.len();
    let visible_items = (chunks[1].height as usize).saturating_sub(2);
    let max_scroll_offset = total_items.saturating_sub(visible_items);
    let scroll_offset = scroll_offset as usize;
    let scroll_position = scroll_offset.min(max_scroll_offset);

    let mut list_state = ListState::default();
    list_state.select(Some(scroll_position));

    let mut scrollbar_state = ScrollbarState::new(total_items).position(scroll_position);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(chunks[1]);

    f.render_stateful_widget(log_list, content_chunks[0], &mut list_state);
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None),
        content_chunks[1],
        &mut scrollbar_state,
    );

    scroll_position as u16
}

fn render_log_tabs(
    f: &mut Frame,
    area: Rect,
    active_category: &LogCategory,
    mouse_x: u16,
    mouse_y: u16,
) {
    let categories = [
        LogCategory::System,
        LogCategory::Kernel,
        LogCategory::Error,
        LogCategory::Boot,
        LogCategory::All,
    ];

    let tab_titles = [
        i18n::t("log_category_system"),
        i18n::t("log_category_kernel"),
        i18n::t("log_category_error"),
        i18n::t("log_category_boot"),
        i18n::t("log_category_all"),
    ];

    let tab_bar_width = area.width - 2;
    let tab_width = (tab_bar_width - 4) / 5;

    let block = Block::default().borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut current_x = inner.x;
    for (i, title) in tab_titles.iter().enumerate() {
        let title_len = title
            .chars()
            .map(|c| if c as u32 > 127 { 2 } else { 1 })
            .sum::<u16>();
        let padding = (tab_width - title_len) / 2;

        let text_x = current_x + padding;

        let is_mouse_over =
            mouse_y == inner.y && mouse_x >= current_x && mouse_x < current_x + tab_width;

        let style = if categories[i] == *active_category {
            Style::default()
                .fg(Color::Cyan)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else if is_mouse_over {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        if categories[i] == *active_category {
            let bg_rect = Rect {
                x: current_x,
                y: inner.y,
                width: tab_width,
                height: inner.height,
            };
            f.render_widget(
                Paragraph::new("").style(Style::default().fg(Color::Black).bg(Color::DarkGray)),
                bg_rect,
            );
        }

        let text = Span::styled(*title, style);
        let text_line = Line::from(vec![text]);
        let text_area = Rect {
            x: text_x,
            y: inner.y,
            width: tab_width - padding,
            height: inner.height,
        };
        f.render_widget(Paragraph::new(text_line), text_area);

        if i < 4 {
            let separator_x = current_x + tab_width;
            let separator_line = Line::from(vec![Span::styled(
                "|",
                Style::default().fg(Color::DarkGray),
            )]);
            let separator_area = Rect {
                x: separator_x,
                y: inner.y,
                width: 1,
                height: inner.height,
            };
            f.render_widget(Paragraph::new(separator_line), separator_area);
        }

        current_x += tab_width + 1;
    }
}

fn get_category_name(category: &LogCategory) -> &'static str {
    match category {
        LogCategory::System => i18n::t("log_category_system"),
        LogCategory::Kernel => i18n::t("log_category_kernel"),
        LogCategory::Error => i18n::t("log_category_error"),
        LogCategory::Boot => i18n::t("log_category_boot"),
        LogCategory::All => i18n::t("log_category_all"),
    }
}

fn get_log_level_name(level: LogLevel) -> &'static str {
    match level {
        LogLevel::All => "All",
        LogLevel::Emerg => "Emerg",
        LogLevel::Alert => "Alert",
        LogLevel::Crit => "Crit",
        LogLevel::Err => "Err",
        LogLevel::Warning => "Warn",
        LogLevel::Notice => "Notice",
        LogLevel::Info => "Info",
        LogLevel::Debug => "Debug",
    }
}