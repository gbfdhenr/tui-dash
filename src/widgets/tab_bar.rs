use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::app::{ActiveTab, LogCategory};
use crate::i18n;

/// 渲染标签栏
pub fn render_tab_bar(
    f: &mut Frame,
    area: Rect,
    active_tab: ActiveTab,
    mouse_x: u16,
    mouse_y: u16,
    has_alert: bool,
) {
    #[cfg(target_os = "windows")]
    let tab_titles = [
        i18n::t("cpu"),
        i18n::t("memory"),
        i18n::t("disk"),
        i18n::t("network"),
        i18n::t("process"),
        i18n::t("logs"),
        i18n::t("temperature"),
    ];

    #[cfg(not(target_os = "windows"))]
    let tab_titles = [
        i18n::t("cpu"),
        i18n::t("memory"),
        i18n::t("disk"),
        i18n::t("network"),
        i18n::t("process"),
        i18n::t("docker"),
        i18n::t("logs"),
        i18n::t("temperature"),
    ];

    let title = if has_alert {
        format!("{} [⚠]", i18n::t("system_monitor"))
    } else {
        i18n::t("system_monitor").to_string()
    };

    render_bar(
        f,
        area,
        &tab_titles,
        Some(active_tab as usize),
        mouse_x,
        mouse_y,
        &title,
    );
}

/// 渲染简化版标签栏（搜索模式下使用）
pub fn render_tab_bar_simple(f: &mut Frame, area: Rect) {
    #[cfg(target_os = "windows")]
    let tab_titles = [
        i18n::t("cpu"),
        i18n::t("memory"),
        i18n::t("disk"),
        i18n::t("network"),
        i18n::t("process"),
        i18n::t("logs"),
        i18n::t("temperature"),
    ];

    #[cfg(not(target_os = "windows"))]
    let tab_titles = [
        i18n::t("cpu"),
        i18n::t("memory"),
        i18n::t("disk"),
        i18n::t("network"),
        i18n::t("process"),
        i18n::t("docker"),
        i18n::t("logs"),
        i18n::t("temperature"),
    ];

    render_bar(f, area, &tab_titles, None, 0, 0, i18n::t("system_monitor"));
}

/// 通用的标签栏渲染函数
fn render_bar(
    f: &mut Frame,
    area: Rect,
    tab_titles: &[&str],
    active_index: Option<usize>,
    mouse_x: u16,
    mouse_y: u16,
    title: &str,
) {
    let tab_count = tab_titles.len();
    let tab_bar_width = area.width.saturating_sub(2);
    let tab_width = tab_bar_width / tab_count as u16;

    let block = Block::default()
        .borders(ratatui::widgets::Borders::ALL)
        .title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut current_x = inner.x;
    for (i, tab_title) in tab_titles.iter().enumerate() {
        let title_len = tab_title
            .chars()
            .map(|c| if c as u32 > 127 { 2 } else { 1 })
            .sum::<u16>();
        let padding = (tab_width - title_len) / 2;
        let text_x = current_x + padding;

        let is_mouse_over =
            mouse_y == inner.y && mouse_x >= current_x && mouse_x < current_x + tab_width;

        let style = if active_index == Some(i) {
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

        if active_index == Some(i) {
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

        let text = Span::styled(*tab_title, style);
        let text_line = Line::from(vec![text]);
        let text_area = Rect {
            x: text_x,
            y: inner.y,
            width: tab_width.saturating_sub(padding),
            height: inner.height,
        };
        f.render_widget(Paragraph::new(text_line), text_area);

        if i < tab_count - 1 {
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

/// 处理标签栏点击
pub fn handle_tab_click(column: u16, row: u16, terminal_size: Rect) -> Option<ActiveTab> {
    if row != 1 {
        return None;
    }

    #[cfg(target_os = "windows")]
    const TAB_COUNT: usize = 7;
    #[cfg(not(target_os = "windows"))]
    const TAB_COUNT: usize = 8;

    if column < terminal_size.x + 1 || column >= terminal_size.x + terminal_size.width - 1 {
        return None;
    }

    let tab_bar_width = terminal_size.width.saturating_sub(2);
    let tab_width = tab_bar_width / TAB_COUNT as u16;
    let offset = column.saturating_sub(terminal_size.x + 1);
    let tab_index = offset / tab_width;

    #[cfg(target_os = "windows")]
    match tab_index {
        0 => Some(ActiveTab::Cpu),
        1 => Some(ActiveTab::Memory),
        2 => Some(ActiveTab::Disk),
        3 => Some(ActiveTab::Network),
        4 => Some(ActiveTab::Process),
        5 => Some(ActiveTab::Logs),
        6 => Some(ActiveTab::Temperature),
        _ => None,
    }

    #[cfg(not(target_os = "windows"))]
    match tab_index {
        0 => Some(ActiveTab::Cpu),
        1 => Some(ActiveTab::Memory),
        2 => Some(ActiveTab::Disk),
        3 => Some(ActiveTab::Network),
        4 => Some(ActiveTab::Process),
        5 => Some(ActiveTab::Docker),
        6 => Some(ActiveTab::Logs),
        7 => Some(ActiveTab::Temperature),
        _ => None,
    }
}

/// 处理日志子页签点击
pub fn handle_log_tab_click(column: u16, row: u16, content_area: Rect) -> Option<LogCategory> {
    if row < content_area.y || row >= content_area.y + 3 {
        return None;
    }
    if row != content_area.y + 1 {
        return None;
    }

    const LOG_TAB_COUNT: usize = 5;

    if column < content_area.x + 1 || column >= content_area.x + content_area.width - 1 {
        return None;
    }

    let tab_bar_width = content_area.width.saturating_sub(2);
    let tab_width = tab_bar_width / LOG_TAB_COUNT as u16;
    let offset = column.saturating_sub(content_area.x + 1);
    let tab_index = offset / tab_width;

    match tab_index {
        0 => Some(LogCategory::System),
        1 => Some(LogCategory::Kernel),
        2 => Some(LogCategory::Error),
        3 => Some(LogCategory::Boot),
        4 => Some(LogCategory::All),
        _ => None,
    }
}