use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use crate::data::settings::Settings;
use super::{BLOCK_BORDERS, block_style, default_style, highlight_style};

/// 渲染设置widget，支持鼠标点击和选项选择
pub fn render(f: &mut Frame, area: Rect, settings: &Settings, selected_option: usize) {
    // 分割区域：标题 + 设置选项
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // 标题区域
            Constraint::Min(0),     // 设置选项区域
        ])
        .split(area);
    
    // 渲染标题
    render_title(f, chunks[0]);
    
    // 渲染设置选项
    render_settings_options(f, chunks[1], settings, selected_option);
}

/// 渲染标题
fn render_title(f: &mut Frame, area: Rect) {
    let title = Paragraph::new("系统设置")
        .block(Block::default()
            .borders(Borders::ALL)
            .style(block_style()))
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center);
    
    f.render_widget(title, area);
}

/// 渲染设置选项
fn render_settings_options(f: &mut Frame, area: Rect, settings: &Settings, selected_option: usize) {
    // 创建设置选项列表
    let options = create_settings_options(settings);
    
    // 创建列表项
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, option)| {
            let style = if i == selected_option {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                default_style()
            };
            ListItem::new(option.clone()).style(style)
        })
        .collect();
    
    // 创建列表
    let list = List::new(items)
        .block(Block::default()
            .title("设置选项")
            .borders(BLOCK_BORDERS)
            .style(block_style()))
        .highlight_style(highlight_style());
    
    // 创建列表状态来管理选中项
    let mut list_state = ListState::default();
    list_state.select(Some(selected_option));
    
    f.render_stateful_widget(list, area, &mut list_state);
}

/// 创建设置选项列表
fn create_settings_options(settings: &Settings) -> Vec<Line<'_>> {
    let mut options = Vec::new();
    
    // 更新频率（毫秒）
    options.push(Line::from(vec![
        Span::raw("更新频率: "),
        Span::styled(format!("{} ms", settings.update_interval_ms), Style::default().fg(Color::Cyan)),
    ]));
    
    // 日志行数限制
    options.push(Line::from(vec![
        Span::raw("日志行数限制: "),
        Span::styled(format!("{}行", settings.log_lines_limit), Style::default().fg(Color::Cyan)),
    ]));
    
    // 鼠标支持
    let mouse_status = if settings.enable_mouse { "启用" } else { "禁用" };
    options.push(Line::from(vec![
        Span::raw("鼠标支持: "),
        Span::styled(mouse_status, Style::default().fg(Color::Cyan)),
    ]));
    
    // 鼠标滚轮滚动
    let wheel_status = if settings.enable_mouse_wheel { "启用" } else { "禁用" };
    options.push(Line::from(vec![
        Span::raw("鼠标滚轮滚动: "),
        Span::styled(wheel_status, Style::default().fg(Color::Cyan)),
    ]));
    
    // 点击切换页签
    let click_tabs_status = if settings.enable_click_tabs { "启用" } else { "禁用" };
    options.push(Line::from(vec![
        Span::raw("点击切换页签: "),
        Span::styled(click_tabs_status, Style::default().fg(Color::Cyan)),
    ]));
    
    // 显示滚动条
    let scrollbars_status = if settings.show_scrollbars { "显示" } else { "隐藏" };
    options.push(Line::from(vec![
        Span::raw("滚动条: "),
        Span::styled(scrollbars_status, Style::default().fg(Color::Cyan)),
    ]));
    
    // 语言设置
    options.push(Line::from(vec![
        Span::raw("语言: "),
        Span::styled(&settings.language, Style::default().fg(Color::Cyan)),
    ]));
    
    // 保存设置
    options.push(Line::from(Span::styled("保存设置", Style::default().fg(Color::Green))));
    
    // 恢复默认设置
    options.push(Line::from(Span::styled("恢复默认设置", Style::default().fg(Color::Red))));
    
    options
}

/// 处理鼠标点击事件
pub fn handle_mouse_click(area: Rect, mouse_x: u16, mouse_y: u16, selected_option: &mut usize, settings: &mut Settings) -> bool {
    // 计算设置选项区域
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // 标题区域
            Constraint::Min(0),     // 设置选项区域
        ])
        .split(area);
    
    // 检查是否点击在设置选项区域内
    let options_area = chunks[1];
    // 需要考虑边框：实际内容区域缩小1行和1列
    let content_area = Rect::new(
        options_area.x + 1,        // 左边框
        options_area.y + 1,        // 上边框
        options_area.width.saturating_sub(2),  // 减去左右边框
        options_area.height.saturating_sub(2)  // 减去上下边框
    );
    
    // 检查鼠标是否在内容区域内
    if mouse_x < content_area.x || mouse_x >= content_area.x + content_area.width ||
       mouse_y < content_area.y || mouse_y >= content_area.y + content_area.height {
        return false;
    }
    
    // 计算点击的选项索引（每个选项占1行）
    let click_y = mouse_y - content_area.y;
    let option_index = click_y as usize;
    
    // 获取选项总数（9个选项）
    let total_options = 9;
    
    if option_index < total_options {
        *selected_option = option_index;
        
        // 处理选项点击（切换设置）
        match option_index {
            0 => settings.increase_update_interval(),  // 增加更新间隔（毫秒）
            1 => settings.increase_log_lines(),  // 增加日志行数限制
            2 => settings.toggle_setting("enable_mouse"),  // 切换鼠标支持
            3 => settings.toggle_setting("enable_mouse_wheel"),  // 切换鼠标滚轮滚动
            4 => settings.toggle_setting("enable_click_tabs"),  // 切换点击切换页签
            5 => settings.toggle_setting("show_scrollbars"),  // 切换滚动条显示
            6 => {  // 语言设置（暂时不支持切换）
                // 可以在这里添加语言切换逻辑
            }
            7 => {  // 保存设置
                if let Err(e) = settings.save() {
                    eprintln!("保存设置失败: {}", e);
                }
            }
            8 => {  // 恢复默认设置
                *settings = Settings::default();
            }
            _ => {}
        }
        
        return true;
    }
    
    false
}