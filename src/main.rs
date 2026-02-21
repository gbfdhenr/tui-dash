mod app;
mod data;
mod widgets;
mod i18n;

use anyhow::Result;
use app::{App, ActiveTab, LogCategory};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEvent, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use std::panic;

// 常量定义
const MAX_SCROLL_OFFSET: u16 = 9999;

// 修复点1：定义终端恢复结构体，实现Drop自动恢复
struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // 无论程序正常退出还是panic，都恢复终端
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

fn main() -> Result<()> {
    // 修复点2：设置panic钩子，确保终端恢复
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // 恢复终端
        let _ = disable_raw_mode();
        let _ = execute!(
            std::io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        // 调用原始panic钩子
        original_hook(panic_info);
    }));

    // 初始化终端
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    // 封装终端到Guard，自动恢复
    let mut terminal_guard = TerminalGuard { terminal };

    // 初始化应用
    let mut app = App::new()?;
    let mut last_update = std::time::Instant::now();
    const MIN_UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(1000);

    // 主循环
    loop {
        // 渲染UI
        terminal_guard.terminal.draw(|f| ui(f, &mut app))?;

        // 处理事件
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    match key.code {
                        // 退出程序
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => break,
                        // Tab切换标签页
                        KeyCode::Tab => app.next_tab(),
                        // 数字键直接跳转到标签页
                        KeyCode::Char('1') => app.active_tab = ActiveTab::Cpu,
                        KeyCode::Char('2') => app.active_tab = ActiveTab::Memory,
                        KeyCode::Char('3') => app.active_tab = ActiveTab::Disk,
                        KeyCode::Char('4') => app.active_tab = ActiveTab::Network,
                        KeyCode::Char('5') => app.active_tab = ActiveTab::Docker,
                        KeyCode::Char('6') => app.active_tab = ActiveTab::Logs,
                        // 上下箭头键滚动日志（仅在日志标签页时有效）
                        KeyCode::Up => {
                            if app.active_tab == ActiveTab::Logs && app.logs_scroll_offset > 0 {
                                app.logs_scroll_offset -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if app.active_tab == ActiveTab::Logs {
                                app.logs_scroll_offset += 1;
                            }
                        }
                        // PageUp/PageDown快速滚动
                        KeyCode::PageUp => {
                            if app.active_tab == ActiveTab::Logs && app.logs_scroll_offset >= 10 {
                                app.logs_scroll_offset -= 10;
                            } else if app.active_tab == ActiveTab::Logs {
                                app.logs_scroll_offset = 0;
                            }
                        }
                        KeyCode::PageDown => {
                            if app.active_tab == ActiveTab::Logs {
                                app.logs_scroll_offset += 10;
                            }
                        }
                        // Home/End跳转到开始/结束
                        KeyCode::Home => {
                            if app.active_tab == ActiveTab::Logs {
                                app.logs_scroll_offset = 0;
                            }
                        }
                        KeyCode::End => {
                            if app.active_tab == ActiveTab::Logs {
                                // 设置一个大的偏移量，widget会限制在最大值
                                app.logs_scroll_offset = MAX_SCROLL_OFFSET;
                            }
                        }
                        // 左右箭头切换标签页（在日志页签内切换日志类别）
                        KeyCode::Right => {
                            if app.active_tab == ActiveTab::Logs {
                                app.active_log_category = app.active_log_category.next();
                                app.logs_scroll_offset = 0;
                            } else {
                                app.next_tab();
                            }
                        }
                        KeyCode::Left => {
                            if app.active_tab == ActiveTab::Logs {
                                app.active_log_category = app.active_log_category.previous();
                                app.logs_scroll_offset = 0;
                            } else {
                                app.previous_tab();
                            }
                        }
                        _ => {}
                    }
                }
                Event::Mouse(mouse_event) => {
                    handle_mouse_event(mouse_event, &mut app, terminal_guard.terminal.size()?);
                }
                _ => {}
            }
        }

        // 数据更新节流
        if last_update.elapsed() >= MIN_UPDATE_INTERVAL {
            // 修复点3：捕获update错误，打印警告而非退出（兼容Docker等临时不可用）
            if let Err(e) = app.update_data() {
                eprintln!("{}", i18n::t("data_update_failed").replace("{}", &e.to_string()));
            }
            last_update = std::time::Instant::now();
        }
    }

    // 程序退出前清理日志监视进程和临时文件
    let _ = app.cleanup();
    
    Ok(())
}

/// 渲染UI
fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    // 主布局：顶部标签栏 + 内容区域
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(size);

    // 标签名称
    let tab_titles = [
        i18n::t("cpu"),
        i18n::t("memory"),
        i18n::t("disk"),
        i18n::t("network"),
        i18n::t("docker"),
        i18n::t("logs"),
    ];
    
    // 计算每个标签的平均宽度（标签栏宽度减去左右边框和分隔符后除以标签数量）
    let tab_bar_width = main_layout[0].width - 2;  // 减去左右边框
    let tab_width = (tab_bar_width - 5) / 6;  // 减去5个分隔符宽度，除以6个标签
    
    // 手动渲染标签栏，确保平均分配宽度
    let block = Block::default().borders(Borders::ALL).title(i18n::t("system_monitor"));
    let inner = block.inner(main_layout[0]);
    f.render_widget(block, main_layout[0]);
    
    let mut current_x = inner.x;
    for (i, title) in tab_titles.iter().enumerate() {
        // 计算文本居中位置
        let title_len = title.chars().map(|c| if c as u32 > 127 { 2 } else { 1 }).sum::<u16>();
        let padding = (tab_width - title_len) / 2;
        
        // 计算实际文本位置
        let text_x = current_x + padding;
        
        // 渲染标签
        let style = if i == app.active_tab as usize {
            Style::default().fg(Color::Cyan).bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        
        // 如果是激活标签，绘制背景
        if i == app.active_tab as usize {
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

    // 根据当前标签渲染对应内容
    match app.active_tab {
        ActiveTab::Cpu => widgets::cpu_widget::render(f, main_layout[1], &app.cpu_data),
        ActiveTab::Memory => widgets::memory_widget::render(f, main_layout[1], &app.memory_data),
        ActiveTab::Disk => widgets::disk_widget::render(f, main_layout[1], &app.disk_data),
        ActiveTab::Network => widgets::network_widget::render(f, main_layout[1], &app.network_data),
        ActiveTab::Docker => widgets::docker_widget::render(f, main_layout[1], &app.docker_data),
        ActiveTab::Logs => {
            // 获取实际的滚动偏移量并更新
            let actual_offset = widgets::logs_widget::render(f, main_layout[1], &app.logs_data, app.logs_scroll_offset, &app.active_log_category);
            app.logs_scroll_offset = actual_offset;
        }
    }
}

/// 处理鼠标事件
fn handle_mouse_event(mouse_event: MouseEvent, app: &mut App, terminal_size: Rect) {
    match mouse_event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // 处理标签栏点击
            if handle_tab_click(mouse_event.column, mouse_event.row, app, terminal_size) {
                return;
            }

            // 计算内容区域（除去顶部标签栏）
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(terminal_size);
            let content_area = main_layout[1];

            // 处理日志页面点击
            if app.active_tab == ActiveTab::Logs {
                // 处理日志子页签点击
                if handle_log_tab_click(mouse_event.column, mouse_event.row, app, content_area) {
                    return;
                }

                // 尝试处理滚动条点击
                if let Some(new_offset) = widgets::logs_widget::handle_scrollbar_click(
                    content_area,
                    mouse_event.column,
                    mouse_event.row,
                    &app.logs_data,
                    app.logs_scroll_offset,
                    &app.active_log_category,
                ) {
                    app.logs_scroll_offset = new_offset;
                    // 开始拖动
                    app.is_dragging_scrollbar = true;
                    return;
                }

                // 处理内容区域点击（用于未来扩展）
                if widgets::logs_widget::handle_content_click(
                    content_area,
                    mouse_event.column,
                    mouse_event.row,
                    &app.logs_data,
                    app.logs_scroll_offset,
                    &app.active_log_category,
                ).is_some() {
                    // 可以在这里添加内容区域的交互逻辑
                }
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            // 处理鼠标拖动
            if app.active_tab == ActiveTab::Logs && app.is_dragging_scrollbar {
                let main_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(terminal_size);
                let content_area = main_layout[1];

                // 根据拖动位置更新滚动偏移
                if let Some(new_offset) = widgets::logs_widget::handle_scrollbar_drag(
                    content_area,
                    mouse_event.column,
                    mouse_event.row,
                    &app.logs_data,
                    &app.active_log_category,
                ) {
                    app.logs_scroll_offset = new_offset;
                }
            }
        }
        MouseEventKind::Up(MouseButton::Left) => {
            // 停止拖动
            app.is_dragging_scrollbar = false;
        }
        MouseEventKind::ScrollUp => {
            // 鼠标滚轮向上滚动
            if app.active_tab == ActiveTab::Logs && app.logs_scroll_offset > 0 {
                app.logs_scroll_offset -= 1;
            }
        }
        MouseEventKind::ScrollDown => {
            // 鼠标滚轮向下滚动
            if app.active_tab == ActiveTab::Logs {
                app.logs_scroll_offset += 1;
            }
        }
        _ => {}
    }
}

/// 处理标签栏点击
fn handle_tab_click(column: u16, row: u16, app: &mut App, terminal_size: Rect) -> bool {
    // 标签栏在顶部，实际标签内容在行 1（行0是边框，行1是标签文本）
    if row != 1 {
        return false;
    }
    
    // 标签数量
    const TAB_COUNT: usize = 6;
    
    // 检查是否在标签区域内（减去左右边框）
    if column < terminal_size.x + 1 || column >= terminal_size.x + terminal_size.width - 1 {
        return false;
    }
    
    // 计算标签栏有效宽度（总宽度减去左右边框）
    let tab_bar_width = terminal_size.width - 2;
    
    // 计算每个标签的平均宽度
    let tab_width = tab_bar_width / TAB_COUNT as u16;
    
    // 计算点击位置相对于标签栏起点的偏移
    let offset = column - (terminal_size.x + 1);
    
    // 计算点击位置对应的标签索引
    let tab_index = offset / tab_width;
    
    // 确保索引在有效范围内
    if tab_index < TAB_COUNT as u16 {
        // 设置对应的标签页
        match tab_index {
            0 => app.active_tab = ActiveTab::Cpu,
            1 => app.active_tab = ActiveTab::Memory,
            2 => app.active_tab = ActiveTab::Disk,
            3 => app.active_tab = ActiveTab::Network,
            4 => app.active_tab = ActiveTab::Docker,
            5 => app.active_tab = ActiveTab::Logs,
            _ => return false,
        }
        return true;
    }

    false
}

/// 处理日志子页签点击（与主页签使用相同的映射逻辑）
fn handle_log_tab_click(column: u16, row: u16, app: &mut App, content_area: Rect) -> bool {
    // 日志子页签栏高度为3，实际标签内容在行 1（相对于content_area）
    if row < content_area.y || row >= content_area.y + 3 {
        return false;
    }
    if row != content_area.y + 1 {
        return false;
    }

    // 日志类别数量
    const LOG_TAB_COUNT: usize = 6;

    // 检查是否在子页签区域内（减去左右边框）
    if column < content_area.x + 1 || column >= content_area.x + content_area.width - 1 {
        return false;
    }

    // 计算子页签栏有效宽度（总宽度减去左右边框）
    let tab_bar_width = content_area.width - 2;

    // 计算每个子页签的平均宽度
    let tab_width = tab_bar_width / LOG_TAB_COUNT as u16;

    // 计算点击位置相对于子页签栏起点的偏移
    let offset = column - (content_area.x + 1);

    // 计算点击位置对应的子页签索引
    let tab_index = offset / tab_width;

    // 确保索引在有效范围内
    if tab_index < LOG_TAB_COUNT as u16 {
        // 设置对应的日志类别
        match tab_index {
            0 => {
                app.active_log_category = LogCategory::System;
                app.logs_scroll_offset = 0;
            }
            1 => {
                app.active_log_category = LogCategory::Kernel;
                app.logs_scroll_offset = 0;
            }
            2 => {
                app.active_log_category = LogCategory::Error;
                app.logs_scroll_offset = 0;
            }
            3 => {
                app.active_log_category = LogCategory::Docker;
                app.logs_scroll_offset = 0;
            }
            4 => {
                app.active_log_category = LogCategory::Boot;
                app.logs_scroll_offset = 0;
            }
            5 => {
                app.active_log_category = LogCategory::All;
                app.logs_scroll_offset = 0;
            }
            _ => return false,
        }
        return true;
    }

    false
}
