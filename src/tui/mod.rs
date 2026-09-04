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
use app::{App, FocusedPane, InputMode};
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
                    if app.mode == InputMode::Insert
                        && app.focused_pane == FocusedPane::Input
                        && !app.show_history_drawer
                        && !app.show_help
                    {
                        app.paste_text(&pasted);
                    }
                }
                AppEvent::Key(key) => {
                    // 全局强制退出
                    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                        app.should_quit = true;
                        continue;
                    }

                    // 帮助弹窗激活时，按任意键关闭
                    if app.show_help {
                        app.show_help = false;
                        continue;
                    }

                    // 历史与生词抽屉激活时
                    if app.show_history_drawer {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('h') | KeyCode::Char('q') => {
                                app.show_history_drawer = false;
                                app.focused_pane = FocusedPane::Input;
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if app.history_selected_index > 0 {
                                    app.history_selected_index -= 1;
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
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
                            KeyCode::Home | KeyCode::Char('g') => {
                                app.history_selected_index = 0;
                            }
                            KeyCode::End | KeyCode::Char('G') => {
                                if !app.history_items.is_empty() {
                                    app.history_selected_index = app.history_items.len() - 1;
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
                            KeyCode::Char('d') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.delete_current_selected();
                            }
                            KeyCode::Char('c') => {
                                app.toggle_history_filter();
                            }
                            _ => {}
                        }
                        continue;
                    }

                    let term_size = terminal.size().unwrap_or_default();
                    let inner_height = term_size.height.saturating_sub(3);

                    match app.mode {
                        InputMode::Insert => {
                            match key.code {
                                // Esc 退出编辑，进入 Normal 模式
                                KeyCode::Esc => {
                                    app.mode = InputMode::Normal;
                                }
                                // Enter 触发翻译（翻译完成后自动切回 Normal 模式）
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
                                // Tab 切换至译文区并转入 Normal 模式
                                KeyCode::Tab => {
                                    app.mode = InputMode::Normal;
                                    app.focused_pane = FocusedPane::Result;
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
                                    app.textarea.input(Input::from(key));
                                }
                            }
                        }
                        InputMode::Normal => {
                            // 全局 Normal 快捷键
                            match key.code {
                                KeyCode::Char('q') => {
                                    app.should_quit = true;
                                    continue;
                                }
                                KeyCode::Char('?') => {
                                    app.show_help = true;
                                    continue;
                                }
                                KeyCode::Char('h') => {
                                    app.show_history_drawer = true;
                                    app.reload_history();
                                    continue;
                                }
                                KeyCode::Char('y') => {
                                    app.copy_result_to_clipboard();
                                    continue;
                                }
                                KeyCode::Tab => {
                                    app.focused_pane = match app.focused_pane {
                                        FocusedPane::Input => FocusedPane::Result,
                                        _ => FocusedPane::Input,
                                    };
                                    continue;
                                }
                                _ => {}
                            }

                            // 栏位专属 Normal 快捷键
                            match app.focused_pane {
                                FocusedPane::Input => match key.code {
                                    // i 或 a 进入 Insert 模式
                                    KeyCode::Char('i') | KeyCode::Char('a') => {
                                        app.mode = InputMode::Insert;
                                    }
                                    // c: 清空并进入 Insert 模式
                                    KeyCode::Char('c') => {
                                        app.clear_input();
                                        app.mode = InputMode::Insert;
                                    }
                                    // x: 仅清空文本，保持 Normal 模式
                                    KeyCode::Char('x') => {
                                        app.clear_input();
                                        app.set_toast("已清空输入框");
                                    }
                                    // Enter 重新查询
                                    KeyCode::Enter => {
                                        app.trigger_search().await;
                                    }
                                    // 右箭头或 l 切换到译文栏
                                    KeyCode::Right | KeyCode::Char('l') => {
                                        app.focused_pane = FocusedPane::Result;
                                    }
                                    // Vim 光标移动与方向键
                                    KeyCode::Up | KeyCode::Char('k') => {
                                        app.textarea.move_cursor(tui_textarea::CursorMove::Up);
                                    }
                                    KeyCode::Down | KeyCode::Char('j') => {
                                        app.textarea.move_cursor(tui_textarea::CursorMove::Down);
                                    }
                                    KeyCode::Left => {
                                        app.textarea.move_cursor(tui_textarea::CursorMove::Back);
                                    }
                                    KeyCode::Home | KeyCode::Char('0') => {
                                        app.textarea.move_cursor(tui_textarea::CursorMove::Head);
                                    }
                                    KeyCode::End | KeyCode::Char('$') => {
                                        app.textarea.move_cursor(tui_textarea::CursorMove::End);
                                    }
                                    _ => {}
                                },
                                FocusedPane::Result => match key.code {
                                    // i 直接切回输入区并进入 Insert 模式
                                    KeyCode::Char('i') => {
                                        app.focused_pane = FocusedPane::Input;
                                        app.mode = InputMode::Insert;
                                    }
                                    // Esc 或左箭头切回输入区
                                    KeyCode::Esc | KeyCode::Left => {
                                        app.focused_pane = FocusedPane::Input;
                                    }
                                    KeyCode::Up | KeyCode::Char('k') => {
                                        if app.result_scroll_offset > 0 {
                                            app.result_scroll_offset -= 1;
                                        }
                                    }
                                    KeyCode::Down | KeyCode::Char('j') => {
                                        app.result_scroll_offset = app.result_scroll_offset.saturating_add(1);
                                    }
                                    KeyCode::PageUp | KeyCode::Char('u') => {
                                        app.result_scroll_offset = app.result_scroll_offset.saturating_sub(inner_height.min(15));
                                    }
                                    KeyCode::PageDown | KeyCode::Char('d') => {
                                        app.result_scroll_offset = app.result_scroll_offset.saturating_add(inner_height.min(15));
                                    }
                                    KeyCode::Home | KeyCode::Char('g') => {
                                        app.result_scroll_offset = 0;
                                    }
                                    KeyCode::End | KeyCode::Char('G') => {
                                        app.result_scroll_offset = app.result_scroll_offset.saturating_add(inner_height * 3);
                                    }
                                    _ => {}
                                },
                            }
                        }
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
