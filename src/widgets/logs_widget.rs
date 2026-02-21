use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use crate::data::logs::LogsData;
use crate::app::LogCategory;
use super::{BLOCK_BORDERS, block_style, default_style, highlight_style};
use crate::i18n;
use textwrap::wrap;

/// 处理日志滚动条点击
/// 返回新的滚动偏移量
pub fn handle_scrollbar_click(area: Rect, column: u16, row: u16, data: &LogsData, _current_offset: u16, category: &LogCategory) -> Option<u16> {
    handle_scrollbar_drag(area, column, row, data, category)
}

/// 处理日志滚动条拖动
/// 返回新的滚动偏移量
pub fn handle_scrollbar_drag(area: Rect, _column: u16, row: u16, data: &LogsData, category: &LogCategory) -> Option<u16> {
    // 获取当前类别的日志
    let logs = data.get_logs_by_category(category);

    // 计算可滚动区域高度
    let content_height = area.height.saturating_sub(2);
    let total_items = logs.len();

    if total_items == 0 || content_height == 0 {
        return None;
    }

    // 计算拖动位置相对于滚动条的比例（允许在滚动条区域外拖动）
    let click_row = row.saturating_sub(area.y + 1);
    let ratio = (click_row as f64) / (content_height as f64);

    // 计算对应的滚动位置
    let max_offset = total_items.saturating_sub(content_height as usize);
    let new_offset = (ratio * max_offset as f64) as usize;

    Some(new_offset as u16)
}

/// 处理日志内容区域点击
/// 返回新的滚动偏移量（用于实现点击定位）
pub fn handle_content_click(area: Rect, column: u16, row: u16, data: &LogsData, current_offset: u16, category: &LogCategory) -> Option<u16> {
    // 检查是否在内容区域内（排除滚动条）
    if column < area.x + 1 || column >= area.x + area.width - 1 {
        return None;
    }
    
    if row < area.y + 1 || row >= area.y + area.height - 1 {
        return None;
    }
    
    // 获取当前类别的日志
    let logs = data.get_logs_by_category(category);
    
    // 计算点击的是哪一行日志
    let _content_height = area.height.saturating_sub(2);
    let click_row = row - area.y - 1;
    
    // 点击的位置对应的日志行号
    let target_log_index = current_offset as usize + click_row as usize;
    
    // 如果点击了当前视图外的区域，可以跳转（这里暂时不做处理）
    if target_log_index >= logs.len() {
        return None;
    }
    
    None // 内容区域点击暂时不处理，可以用于选择日志行等扩展功能
}

/// 渲染日志widget
pub fn render(f: &mut Frame, area: Rect, data: &LogsData, scroll_offset: u16, category: &LogCategory) -> u16 {
    render_log_content(f, area, data, scroll_offset, category)
}

/// 将单行日志根据终端宽度自动换行，保持缩进
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

/// 渲染日志内容
fn render_log_content(f: &mut Frame, area: Rect, data: &LogsData, scroll_offset: u16, category: &LogCategory) -> u16 {
    // 获取当前类别的日志
    let logs = data.get_logs_by_category(category);

    // 分割区域：子页签栏 + 日志内容
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    // 渲染子页签
    render_log_tabs(f, chunks[0], category);

    // 为每行日志添加4空格缩进并根据宽度自动换行
    let log_items: Vec<ListItem> = logs
        .iter()
        .flat_map(|line| {
            let wrapped_lines = wrap_log_line(line, chunks[1].width as usize);
            wrapped_lines.into_iter().map(|s| ListItem::new(s).style(default_style()))
        })
        .collect();

    // 创建标题，显示日志数量
    let title = format!("{} ({} 条)", get_category_name(category), logs.len());

    // 创建可滚动列表
    let log_list = List::new(log_items)
        .block(Block::default()
            .title(title)
            .borders(BLOCK_BORDERS)
            .style(block_style()))
        .highlight_style(highlight_style());

    // 计算滚动位置
    let total_items = logs.len();
    let visible_items = (chunks[1].height as usize).saturating_sub(2); // 减去边框
    let max_scroll_offset = total_items.saturating_sub(visible_items);
    let scroll_offset = scroll_offset as usize;
    let scroll_position = scroll_offset.min(max_scroll_offset);

    // 创建列表状态来管理滚动
    let mut list_state = ListState::default();
    list_state.select(Some(scroll_position));

    let mut scrollbar_state = ScrollbarState::new(total_items).position(scroll_position);

    // 分割日志内容区域：列表 + 滚动条
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(chunks[1]);

    // 渲染列表和滚动条
    f.render_stateful_widget(log_list, content_chunks[0], &mut list_state);
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None),
        content_chunks[1],
        &mut scrollbar_state,
    );

    // 返回实际的滚动偏移量
    scroll_position as u16
}

/// 渲染日志类别子页签（与主页签使用相同的布局和渲染逻辑）
fn render_log_tabs(f: &mut Frame, area: Rect, active_category: &LogCategory) {
    let categories = [
        LogCategory::System,
        LogCategory::Kernel,
        LogCategory::Error,
        LogCategory::Docker,
        LogCategory::Boot,
        LogCategory::All,
    ];

    // 标签名称
    let tab_titles = [
        i18n::t("log_category_system"),
        i18n::t("log_category_kernel"),
        i18n::t("log_category_error"),
        i18n::t("log_category_docker"),
        i18n::t("log_category_boot"),
        i18n::t("log_category_all"),
    ];

    // 计算每个标签的平均宽度（标签栏宽度减去左右边框和分隔符后除以标签数量）
    let tab_bar_width = area.width - 2;  // 减去左右边框
    let tab_width = (tab_bar_width - 5) / 6;  // 减去5个分隔符宽度，除以6个标签

    // 手动渲染标签栏，确保平均分配宽度
    let block = Block::default().borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut current_x = inner.x;
    for (i, title) in tab_titles.iter().enumerate() {
        // 计算文本居中位置
        let title_len = title.chars().map(|c| if c as u32 > 127 { 2 } else { 1 }).sum::<u16>();
        let padding = (tab_width - title_len) / 2;

        // 计算实际文本位置
        let text_x = current_x + padding;

        // 渲染标签
        let style = if categories[i] == *active_category {
            Style::default().fg(Color::Cyan).bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        // 如果是激活标签，绘制背景
        if categories[i] == *active_category {
            let bg_rect = Rect {
                x: current_x,
                y: inner.y,
                width: tab_width,
                height: inner.height,
            };
            f.render_widget(Paragraph::new("").style(Style::default().fg(Color::Black).bg(Color::DarkGray)), bg_rect);
        }

        // 渲染文本
        let text = Span::styled(*title, style);
        let text_line = Line::from(vec![text]);
        let text_area = Rect {
            x: text_x,
            y: inner.y,
            width: tab_width - padding,
            height: inner.height,
        };
        f.render_widget(Paragraph::new(text_line), text_area);

        // 渲染分隔符（除了最后一个标签）
        if i < 5 {
            let separator_x = current_x + tab_width;
            let separator_line = Line::from(vec![Span::styled("|", Style::default().fg(Color::DarkGray))]);
            let separator_area = Rect {
                x: separator_x,
                y: inner.y,
                width: 1,
                height: inner.height,
            };
            f.render_widget(Paragraph::new(separator_line), separator_area);
        }

        current_x += tab_width + 1;  // 移动到下一个标签位置（+1 是分隔符）
    }
}

/// 获取类别名称
fn get_category_name(category: &LogCategory) -> &'static str {
    match category {
        LogCategory::System => i18n::t("log_category_system"),
        LogCategory::Kernel => i18n::t("log_category_kernel"),
        LogCategory::Error => i18n::t("log_category_error"),
        LogCategory::Docker => i18n::t("log_category_docker"),
        LogCategory::Boot => i18n::t("log_category_boot"),
        LogCategory::All => i18n::t("log_category_all"),
    }
}
