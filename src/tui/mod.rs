pub mod app;
pub mod event;
pub mod ui;

use std::io;
use std::time::Duration;
use crossterm::{
    event::{DisableBracketedPaste, EnableBracketedPaste, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use reqwest::Client;
use tui_textarea::Input;

use crate::db::Database;
use crate::error::Result;
use app::{App, FocusedPane};
use event::{AppEvent, EventHandler};

pub async fn run_tui(client: Client, db: Database) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(client, db);
    let mut events = EventHandler::new(Duration::from_millis(150));

    loop {
        terminal.draw(|f| ui::render(f, &mut app))?;

        if let Some(event) = events.next().await {
            match event {
                AppEvent::Tick => {}
                AppEvent::Paste(pasted) => {
                    if app.focused_pane == FocusedPane::Input && !app.show_history_drawer && !app.show_help {
                        app.paste_text(&pasted);
                    }
                }
                AppEvent::Key(key) => {
                    // 全局强制退出
                    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                        app.should_quit = true;
                    }

                    // 帮助弹窗
                    if app.show_help {
                        app.show_help = false;
                        continue;
                    }

                    // 历史与生词抽屉
                    if app.show_history_drawer {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('h') => {
                                app.show_history_drawer = false;
                                app.focused_pane = FocusedPane::Input;
                            }
                            KeyCode::Up => {
                                if app.history_selected_index > 0 {
                                    app.history_selected_index -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if !app.history_items.is_empty()
                                    && app.history_selected_index + 1 < app.history_items.len()
                                {
                                    app.history_selected_index += 1;
                                }
                            }
                            KeyCode::PageUp => {
                                app.history_selected_index = app.history_selected_index.saturating_sub(10);
                            }
                            KeyCode::PageDown => {
                                if !app.history_items.is_empty() {
                                    app.history_selected_index = (app.history_selected_index + 10).min(app.history_items.len() - 1);
                                }
                            }
                            KeyCode::Enter => {
                                let query_opt = app
                                    .history_items
                                    .get(app.history_selected_index)
                                    .map(|i| i.query.clone());
                                if let Some(query) = query_opt {
                                    app.set_input_string(&query);
                                    app.show_history_drawer = false;
                                    app.focused_pane = FocusedPane::Result;
                                    app.trigger_search().await;
                                }
                            }
                            KeyCode::Char('f') => {
                                app.toggle_favorite_current_selected();
                            }
                            KeyCode::Char('d') => {
                                app.delete_current_selected();
                            }
                            KeyCode::Char('c') => {
                                app.toggle_history_filter();
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // 全局帮助
                    if key.code == KeyCode::Char('?') && app.focused_pane != FocusedPane::Input {
                        app.show_help = true;
                        continue;
                    }

                    // Tab 切换双栏焦点
                    if key.code == KeyCode::Tab {
                        app.focused_pane = match app.focused_pane {
                            FocusedPane::Input => FocusedPane::Result,
                            FocusedPane::Result => FocusedPane::Input,
                            FocusedPane::HistoryModal => FocusedPane::Input,
                        };
                        continue;
                    }

                    let term_size = terminal.size().unwrap_or_default();
                    let inner_height = term_size.height.saturating_sub(3);

                    match app.focused_pane {
                        FocusedPane::Input => match key.code {
                            // Enter 触发翻译
                            KeyCode::Enter if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                                app.trigger_search().await;
                            }
                            // Shift+Enter 换行
                            KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
                                app.textarea.insert_newline();
                            }
                            KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.textarea.insert_newline();
                            }
                            KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.show_history_drawer = true;
                                app.reload_history();
                            }
                            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) => {
                                let max_width = (term_size.width / 2).saturating_sub(4);
                                app.insert_char_with_wrap(c, max_width);
                            }
                            _ => {
                                // 委托给专业的 TextArea 处理：精确中英文光标、左右上下移动、删除、退格、行首行尾
                                app.textarea.input(Input::from(key));
                            }
                        },
                        FocusedPane::Result => match key.code {
                            KeyCode::Up => {
                                if app.result_scroll_offset > 0 {
                                    app.result_scroll_offset -= 1;
                                }
                            }
                            KeyCode::Down => {
                                app.result_scroll_offset = app.result_scroll_offset.saturating_add(1);
                            }
                            KeyCode::PageUp => {
                                app.result_scroll_offset = app.result_scroll_offset.saturating_sub(inner_height.min(15));
                            }
                            KeyCode::PageDown => {
                                app.result_scroll_offset = app.result_scroll_offset.saturating_add(inner_height.min(15));
                            }
                            KeyCode::Home => {
                                app.result_scroll_offset = 0;
                            }
                            KeyCode::Char('h') => {
                                app.show_history_drawer = true;
                                app.reload_history();
                            }
                            KeyCode::Esc => {
                                app.focused_pane = FocusedPane::Input;
                            }
                            KeyCode::Char('q') => {
                                app.should_quit = true;
                            }
                            _ => {}
                        },
                        FocusedPane::HistoryModal => {}
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), DisableBracketedPaste, LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
