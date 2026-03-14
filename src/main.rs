mod app;
mod data;
mod i18n;
mod widgets;

use anyhow::Result;
use app::{ActiveTab, App};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEvent,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use std::panic;

const MAX_SCROLL_OFFSET: u16 = u16::MAX;
const MAX_PROCESS_SCROLL_OFFSET: usize = usize::MAX;
const TAB_BAR_HEIGHT: u16 = 3;
const SEARCH_BAR_HEIGHT: u16 = 6;
const SCROLL_STEP: usize = 10;

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if let Err(_e) = disable_raw_mode() {}
        if let Err(_e) = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ) {}
        if let Err(_e) = self.terminal.show_cursor() {}
    }
}

fn main() -> Result<()> {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    let mut terminal_guard = TerminalGuard { terminal };

    let mut app = App::new()?;
    let mut last_update = std::time::Instant::now();
    let min_update_interval =
        std::time::Duration::from_millis(crate::data::DEFAULT_UPDATE_INTERVAL_MS);

    loop {
        terminal_guard.terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(
            crate::data::EVENT_POLL_INTERVAL_MS,
        ))? {
            match event::read()? {
                Event::Key(key) => {
                    if app.search_mode {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                                app.exit_search_mode();
                            }
                            KeyCode::Enter => {
                                app.apply_search();
                            }
                            KeyCode::Backspace => {
                                app.remove_from_search_query();
                            }
                            KeyCode::Char(c) => {
                                app.add_to_search_query(c);
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => break,
                            KeyCode::Char('/') => {
                                if app.active_tab == ActiveTab::Process {
                                    app.enter_search_mode();
                                }
                            }
                            KeyCode::Tab => app.next_tab(),
                            KeyCode::Char('1') => app.active_tab = ActiveTab::Cpu,
                            KeyCode::Char('2') => app.active_tab = ActiveTab::Memory,
                            KeyCode::Char('3') => app.active_tab = ActiveTab::Disk,
                            KeyCode::Char('4') => app.active_tab = ActiveTab::Network,
                            KeyCode::Char('5') => app.active_tab = ActiveTab::Process,
                            #[cfg(not(target_os = "windows"))]
                            KeyCode::Char('6') => app.active_tab = ActiveTab::Docker,
                            #[cfg(target_os = "windows")]
                            KeyCode::Char('6') => app.active_tab = ActiveTab::Logs,
                            #[cfg(not(target_os = "windows"))]
                            KeyCode::Char('7') => app.active_tab = ActiveTab::Logs,
                            #[cfg(target_os = "windows")]
                            KeyCode::Char('7') => app.active_tab = ActiveTab::Temperature,
                            #[cfg(not(target_os = "windows"))]
                            KeyCode::Char('8') => app.active_tab = ActiveTab::Temperature,
                            KeyCode::Char('R') => {
                                if let Err(_e) = app.update_data() {}
                            }
                            KeyCode::Char('P') => {
                                app.paused = !app.paused;
                            }
                            KeyCode::Char('L') => {
                                if app.active_tab == ActiveTab::Logs {
                                    app.logs_data.toggle_log_level();
                                    app.logs_scroll_offset = 0;
                                }
                            }
                            KeyCode::Up => {
                                if app.active_tab == ActiveTab::Logs && app.logs_scroll_offset > 0 {
                                    app.logs_scroll_offset -= 1;
                                } else if app.active_tab == ActiveTab::Process
                                    && app.process_scroll_offset > 0
                                {
                                    app.process_scroll_offset -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if app.active_tab == ActiveTab::Logs {
                                    app.logs_scroll_offset += 1;
                                } else if app.active_tab == ActiveTab::Process {
                                    app.process_scroll_offset += 1;
                                }
                            }
                            KeyCode::PageUp => {
                                if app.active_tab == ActiveTab::Logs
                                    && app.logs_scroll_offset >= SCROLL_STEP as u16
                                {
                                    app.logs_scroll_offset -= SCROLL_STEP as u16;
                                } else if app.active_tab == ActiveTab::Logs {
                                    app.logs_scroll_offset = 0;
                                } else if app.active_tab == ActiveTab::Process
                                    && app.process_scroll_offset >= SCROLL_STEP
                                {
                                    app.process_scroll_offset -= SCROLL_STEP;
                                } else if app.active_tab == ActiveTab::Process {
                                    app.process_scroll_offset = 0;
                                }
                            }
                            KeyCode::PageDown => {
                                if app.active_tab == ActiveTab::Logs {
                                    app.logs_scroll_offset += SCROLL_STEP as u16;
                                } else if app.active_tab == ActiveTab::Process {
                                    app.process_scroll_offset += SCROLL_STEP;
                                }
                            }
                            KeyCode::Home => {
                                if app.active_tab == ActiveTab::Logs {
                                    app.logs_scroll_offset = 0;
                                } else if app.active_tab == ActiveTab::Process {
                                    app.process_scroll_offset = 0;
                                }
                            }
                            KeyCode::End => {
                                if app.active_tab == ActiveTab::Logs {
                                    app.logs_scroll_offset = MAX_SCROLL_OFFSET;
                                } else if app.active_tab == ActiveTab::Process {
                                    app.process_scroll_offset = MAX_PROCESS_SCROLL_OFFSET;
                                }
                            }
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
                }
                Event::Mouse(mouse_event) => {
                    handle_mouse_event(mouse_event, &mut app, terminal_guard.terminal.size()?);
                }
                _ => {}
            }
        }

        if last_update.elapsed() >= min_update_interval {
            let _ = app.update_data();
            last_update = std::time::Instant::now();
        }
    }

    if let Err(_e) = app.cleanup() {}

    Ok(())
}

/// 渲染UI
fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    let top_height = if app.search_mode {
        SEARCH_BAR_HEIGHT
    } else {
        TAB_BAR_HEIGHT
    };

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(top_height), Constraint::Min(0)])
        .split(size);

    if app.search_mode {
        render_search_box(f, main_layout[0], &app.search_query);
    } else {
        render_tab_bar(f, main_layout[0], app);
    }

    let content_area = if app.search_mode {
        Rect {
            x: main_layout[0].x,
            y: main_layout[0].y + top_height,
            width: main_layout[0].width,
            height: main_layout[0].height - top_height,
        }
    } else {
        main_layout[1]
    };

    match app.active_tab {
        ActiveTab::Cpu => widgets::cpu_widget::render(f, content_area, &app.cpu_data, &app.history),
        ActiveTab::Memory => {
            widgets::memory_widget::render(f, content_area, &app.memory_data, &app.history)
        }
        ActiveTab::Disk => widgets::disk_widget::render(f, content_area, &app.disk_data),
        ActiveTab::Network => {
            widgets::network_widget::render(f, content_area, &app.network_data, &app.history)
        }
        ActiveTab::Process => {
            let actual_offset = widgets::process_widget::render(
                f,
                content_area,
                &app.process_data,
                app.process_scroll_offset,
                app.mouse_x,
                app.mouse_y,
            );
            app.process_scroll_offset = actual_offset;
        }
        ActiveTab::Docker => {
            widgets::docker_widget::render(f, content_area, &app.docker_data);
        }
        ActiveTab::Logs => {
            let actual_offset = widgets::logs_widget::render(
                f,
                content_area,
                &app.logs_data,
                app.logs_scroll_offset,
                &app.active_log_category,
                app.mouse_x,
                app.mouse_y,
            );
            app.logs_scroll_offset = actual_offset;
        }
        ActiveTab::Temperature => {
            widgets::temperature_widget::render(
                f,
                content_area,
                &app.temperature_data,
                &app.battery_data,
            );
        }
    }
}

/// 渲染搜索框
fn render_search_box(f: &mut Frame, area: Rect, query: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)])
        .split(area);

    render_tab_bar_simple(f, chunks[0]);

    let search_text = format!("/{}", query);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Search ")
        .style(widgets::block_style());

    let paragraph = Paragraph::new(search_text)
        .block(block)
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(paragraph, chunks[1]);
}

/// 渲染标签栏
fn render_tab_bar(f: &mut Frame, area: Rect, app: &App) {
    widgets::tab_bar::render_tab_bar(
        f,
        area,
        app.active_tab,
        app.mouse_x,
        app.mouse_y,
        app.has_alert,
    );
}

/// 渲染简化版标签栏（搜索模式下使用）
fn render_tab_bar_simple(f: &mut Frame, area: Rect) {
    widgets::tab_bar::render_tab_bar_simple(f, area);
}

/// 处理鼠标事件
fn handle_mouse_event(mouse_event: MouseEvent, app: &mut App, terminal_size: Rect) {
    app.mouse_x = mouse_event.column;
    app.mouse_y = mouse_event.row;

    match mouse_event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if handle_tab_click(mouse_event.column, mouse_event.row, app, terminal_size) {
                return;
            }

            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(terminal_size);
            let content_area = main_layout[1];

            if app.active_tab == ActiveTab::Logs {
                if handle_log_tab_click(mouse_event.column, mouse_event.row, app, content_area) {
                    return;
                }

                if let Some(new_offset) = widgets::logs_widget::handle_scrollbar_click(
                    content_area,
                    mouse_event.column,
                    mouse_event.row,
                    &app.logs_data,
                    app.logs_scroll_offset,
                    &app.active_log_category,
                ) {
                    app.logs_scroll_offset = new_offset;
                    app.is_dragging_scrollbar = true;
                    return;
                }

                if widgets::logs_widget::handle_content_click(
                    content_area,
                    mouse_event.column,
                    mouse_event.row,
                    &app.logs_data,
                    app.logs_scroll_offset,
                    &app.active_log_category,
                )
                .is_some()
                {}
            } else if app.active_tab == ActiveTab::Process {
                if let Some(new_offset) = widgets::process_widget::handle_scrollbar_click(
                    content_area,
                    mouse_event.column,
                    mouse_event.row,
                    &app.process_data,
                    app.process_scroll_offset,
                ) {
                    app.process_scroll_offset = new_offset;
                    app.is_dragging_scrollbar = true;
                    return;
                }
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if app.is_dragging_scrollbar {
                let main_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(terminal_size);
                let content_area = main_layout[1];

                if app.active_tab == ActiveTab::Logs {
                    if let Some(new_offset) = widgets::logs_widget::handle_scrollbar_drag(
                        content_area,
                        mouse_event.column,
                        mouse_event.row,
                        &app.logs_data,
                        &app.active_log_category,
                    ) {
                        app.logs_scroll_offset = new_offset;
                    }
                } else if app.active_tab == ActiveTab::Process {
                    if let Some(new_offset) = widgets::process_widget::handle_scrollbar_drag(
                        content_area,
                        mouse_event.column,
                        mouse_event.row,
                        &app.process_data,
                        app.process_scroll_offset,
                    ) {
                        app.process_scroll_offset = new_offset;
                    }
                }
            }
        }
        MouseEventKind::Up(MouseButton::Left) => {
            app.is_dragging_scrollbar = false;
        }
        MouseEventKind::ScrollUp => {
            if app.active_tab == ActiveTab::Logs && app.logs_scroll_offset > 0 {
                app.logs_scroll_offset -= 1;
            } else if app.active_tab == ActiveTab::Process && app.process_scroll_offset > 0 {
                app.process_scroll_offset -= 1;
            }
        }
        MouseEventKind::ScrollDown => {
            if app.active_tab == ActiveTab::Logs {
                app.logs_scroll_offset += 1;
            } else if app.active_tab == ActiveTab::Process {
                app.process_scroll_offset += 1;
            }
        }
        _ => {}
    }
}

/// 处理标签栏点击
fn handle_tab_click(column: u16, row: u16, app: &mut App, terminal_size: Rect) -> bool {
    if let Some(active_tab) = widgets::tab_bar::handle_tab_click(column, row, terminal_size) {
        app.active_tab = active_tab;
        true
    } else {
        false
    }
}

/// 处理日志子页签点击
fn handle_log_tab_click(column: u16, row: u16, app: &mut App, content_area: Rect) -> bool {
    if let Some(category) = widgets::tab_bar::handle_log_tab_click(column, row, content_area) {
        app.active_log_category = category;
        app.logs_scroll_offset = 0;
        true
    } else {
        false
    }
}