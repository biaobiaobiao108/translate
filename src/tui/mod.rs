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
use tokio::sync::mpsc;
use tui_textarea::Input;

use crate::api::dict::smart_query;
use crate::db::Database;
use crate::error::Result;
use app::{App, FocusedPane, InputMode, SearchRequest};
use event::{AppEvent, EventHandler};

struct TerminalSession;

impl TerminalSession {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen, EnableBracketedPaste) {
            let _ = disable_raw_mode();
            return Err(error);
        }
        Ok(Self)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, DisableBracketedPaste, LeaveAlternateScreen);
    }
}

fn spawn_search(app: &mut App<'_>, sender: &mpsc::Sender<AppEvent>) {
    if let Some((query, result)) = app.cached_result() {
        app.focused_pane = FocusedPane::Result;
        app.apply_cached_result(query, result);
        return;
    }

    let Some(SearchRequest {
        request_id,
        query,
        client,
        cancel,
    }) = app.begin_search()
    else {
        return;
    };

    app.focused_pane = FocusedPane::Result;
    let event_sender = sender.clone();
    tokio::spawn(async move {
        let result = tokio::select! {
            _ = cancel => return,
            result = smart_query(&client, &query, false) => result.map_err(|error| error.to_string()),
        };

        let _ = event_sender
            .send(AppEvent::SearchFinished {
                request_id,
                query,
                result,
            })
            .await;
    });
}

pub async fn run_tui(client: Client, db: Database) -> Result<()> {
    let _session = TerminalSession::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(client, db);
    let mut events = EventHandler::new(Duration::from_millis(150));
    let event_sender = events.sender();

    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        let Some(event) = events.next().await else {
            break;
        };

        match event {
            AppEvent::Tick => {}
            AppEvent::SearchFinished {
                request_id,
                query,
                result,
            } => app.apply_search_result(request_id, query, result),
            AppEvent::EventError(error) => {
                app.error_message = Some(format!("终端事件流失败: {}", error));
                app.should_quit = true;
            }
            AppEvent::Paste(pasted) => {
                if app.mode == InputMode::Insert
                    && app.focused_pane == FocusedPane::Input
                    && !app.show_history_drawer
                    && !app.show_help
                {
                    let term_size = terminal.size()?;
                    let max_width = (term_size.width / 2).saturating_sub(4);
                    app.paste_text(&pasted, max_width);
                }
            }
            AppEvent::Key(key) => {
                // 全局强制退出，并取消尚未完成的网络请求。
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    app.cancel_active_search();
                    app.should_quit = true;
                    continue;
                }

                // 帮助弹窗激活时，按任意键关闭。
                if app.show_help {
                    app.show_help = false;
                    continue;
                }

                // 历史与生词抽屉激活时。
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
                            app.history_selected_index =
                                app.history_selected_index.saturating_sub(10);
                        }
                        KeyCode::PageDown => {
                            if !app.history_items.is_empty() {
                                app.history_selected_index = (app.history_selected_index + 10)
                                    .min(app.history_items.len() - 1);
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
                                .map(|item| item.query.clone());
                            if let Some(query) = query_opt {
                                app.set_input_string(&query);
                                app.show_history_drawer = false;
                                spawn_search(&mut app, &event_sender);
                            }
                        }
                        KeyCode::Char('f') => app.toggle_favorite_current_selected(),
                        KeyCode::Char('d') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.delete_current_selected();
                        }
                        KeyCode::Char('c') => app.toggle_history_filter(),
                        _ => {}
                    }
                    continue;
                }

                let term_size = terminal.size()?;
                let inner_height = term_size.height.saturating_sub(3);

                match app.mode {
                    InputMode::Insert => match key.code {
                        KeyCode::Esc => app.mode = InputMode::Normal,
                        KeyCode::Enter if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                            spawn_search(&mut app, &event_sender);
                        }
                        KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
                            app.textarea.insert_newline();
                        }
                        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.textarea.insert_newline();
                        }
                        KeyCode::Tab => {
                            app.mode = InputMode::Normal;
                            app.focused_pane = FocusedPane::Result;
                        }
                        KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.show_history_drawer = true;
                            app.reload_history();
                        }
                        KeyCode::Char(c)
                            if !key.modifiers.contains(KeyModifiers::CONTROL)
                                && !key.modifiers.contains(KeyModifiers::ALT) =>
                        {
                            let max_width = (term_size.width / 2).saturating_sub(4);
                            app.insert_char_with_wrap(c, max_width);
                        }
                        _ => {
                            app.textarea.input(Input::from(key));
                        }
                    },
                    InputMode::Normal => {
                        match key.code {
                            KeyCode::Char('q') => {
                                app.cancel_active_search();
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
                                    FocusedPane::Result => FocusedPane::Input,
                                };
                                continue;
                            }
                            _ => {}
                        }

                        match app.focused_pane {
                            FocusedPane::Input => match key.code {
                                KeyCode::Char('i') | KeyCode::Char('a') => {
                                    app.mode = InputMode::Insert;
                                }
                                KeyCode::Char('c') => {
                                    app.clear_input();
                                    app.mode = InputMode::Insert;
                                }
                                KeyCode::Char('x') => {
                                    app.clear_input();
                                    app.set_toast("已清空输入框");
                                }
                                KeyCode::Enter => spawn_search(&mut app, &event_sender),
                                KeyCode::Right | KeyCode::Char('l') => {
                                    app.focused_pane = FocusedPane::Result;
                                }
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
                                KeyCode::Char('i') => {
                                    app.focused_pane = FocusedPane::Input;
                                    app.mode = InputMode::Insert;
                                }
                                KeyCode::Esc | KeyCode::Left => {
                                    app.focused_pane = FocusedPane::Input;
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    app.result_scroll_offset =
                                        app.result_scroll_offset.saturating_sub(1);
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    app.result_scroll_offset =
                                        app.result_scroll_offset.saturating_add(1);
                                }
                                KeyCode::PageUp | KeyCode::Char('u') => {
                                    app.result_scroll_offset = app
                                        .result_scroll_offset
                                        .saturating_sub(inner_height.min(15));
                                }
                                KeyCode::PageDown | KeyCode::Char('d') => {
                                    app.result_scroll_offset = app
                                        .result_scroll_offset
                                        .saturating_add(inner_height.min(15));
                                }
                                KeyCode::Home | KeyCode::Char('g') => {
                                    app.result_scroll_offset = 0;
                                }
                                KeyCode::End | KeyCode::Char('G') => {
                                    app.result_scroll_offset = app
                                        .result_scroll_offset
                                        .saturating_add(inner_height.saturating_mul(3));
                                }
                                _ => {}
                            },
                        }
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    terminal.show_cursor()?;
    Ok(())
}
