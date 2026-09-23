use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use reqwest::Client;
use tokio::sync::oneshot;
use tui_textarea::TextArea;
use unicode_width::UnicodeWidthChar;

use crate::api::dict::QueryOutput;
use crate::db::{Database, HistoryItem};
use crate::views::theme::ThemeMode;

const QUERY_CACHE_CAPACITY: usize = 32;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FocusedPane {
    Input,
    Result,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputMode {
    Normal,
    Insert,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HistoryFilter {
    All,
    Favorites,
}

pub struct SearchRequest {
    pub request_id: u64,
    pub query: String,
    pub client: Client,
    pub cancel: oneshot::Receiver<()>,
}

pub struct App<'a> {
    pub should_quit: bool,
    pub mode: InputMode,
    pub focused_pane: FocusedPane,
    pub textarea: TextArea<'a>,
    pub is_searching: bool,
    pub current_result: Option<QueryOutput>,
    pub error_message: Option<String>,
    pub result_scroll_offset: u16,
    pub toast_message: Option<(String, Instant)>,

    // 历史与生词抽屉
    pub show_history_drawer: bool,
    pub history_items: Vec<HistoryItem>,
    pub history_selected_index: usize,
    pub history_filter: HistoryFilter,

    pub show_help: bool,
    pub theme_mode: ThemeMode,
    pub db: Database,
    pub client: Client,
    next_search_id: u64,
    active_search_id: Option<u64>,
    active_search_cancel: Option<oneshot::Sender<()>>,
    query_cache: HashMap<String, QueryOutput>,
    query_cache_order: VecDeque<String>,
}

impl<'a> App<'a> {
    pub fn new(client: Client, db: Database, theme_mode: ThemeMode) -> Self {
        let (history, history_error) = match db.list_history(false, 100) {
            Ok(items) => (items, None),
            Err(error) => (Vec::new(), Some(format!("历史记录加载失败: {}", error))),
        };
        let mut textarea = TextArea::default();
        set_placeholder(&mut textarea);

        Self {
            should_quit: false,
            mode: InputMode::Insert,
            focused_pane: FocusedPane::Input,
            textarea,
            is_searching: false,
            current_result: None,
            error_message: None,
            result_scroll_offset: 0,
            toast_message: history_error.map(|error| (error, Instant::now())),
            show_history_drawer: false,
            history_items: history,
            history_selected_index: 0,
            history_filter: HistoryFilter::All,
            show_help: false,
            theme_mode,
            db,
            client,
            next_search_id: 0,
            active_search_id: None,
            active_search_cancel: None,
            query_cache: HashMap::new(),
            query_cache_order: VecDeque::new(),
        }
    }

    pub fn toggle_theme(&mut self) {
        let current_resolved = self.theme_mode.resolved();
        let next_mode = match current_resolved {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Auto => ThemeMode::Light,
        };
        self.theme_mode = next_mode;
        let mode_desc = match next_mode {
            ThemeMode::Dark => "深色 (Dark)",
            ThemeMode::Light => "浅色 (Light)",
            ThemeMode::Auto => "自动 (Auto)",
        };
        if let Err(e) = self.db.set_config("theme", &next_mode.to_string()) {
            self.set_toast(format!("已切换至 {}（保存失败: {}）", mode_desc, e));
        } else {
            self.set_toast(format!("已切换至 {} 主题并保存偏好", mode_desc));
        }
    }

    pub fn get_input_string(&self) -> String {
        self.textarea.lines().join("\n")
    }

    pub fn set_input_string(&mut self, text: &str) {
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        let lines = normalized
            .split('\n')
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        self.textarea = TextArea::new(lines);
        self.textarea.move_cursor(tui_textarea::CursorMove::Bottom);
        self.textarea.move_cursor(tui_textarea::CursorMove::End);
    }

    /// 插入字符并检查是否达到行末宽度，实现平滑自动换行。
    pub fn insert_char_with_wrap(&mut self, c: char, max_width: u16) {
        if max_width > 4 {
            let (cursor_row, cursor_col) = self.textarea.cursor();
            if let Some(line) = self.textarea.lines().get(cursor_row) {
                let current_line_width = line
                    .chars()
                    .take(cursor_col)
                    .map(|ch| UnicodeWidthChar::width(ch).unwrap_or(0) as u16)
                    .sum::<u16>();
                let char_width = UnicodeWidthChar::width(c).unwrap_or(0) as u16;
                if current_line_width > 0
                    && current_line_width.saturating_add(char_width) > max_width
                {
                    self.textarea.insert_newline();
                }
            }
        }
        self.textarea.insert_char(c);
    }

    pub fn paste_text(&mut self, text: &str, max_width: u16) {
        for c in text.chars() {
            match c {
                '\r' => {}
                '\n' => self.textarea.insert_newline(),
                _ => self.insert_char_with_wrap(c, max_width),
            }
        }
    }

    pub fn reload_history(&mut self) {
        let only_favorites = self.history_filter == HistoryFilter::Favorites;
        match self.db.list_history(only_favorites, 100) {
            Ok(items) => {
                self.history_items = items;
                if self.history_selected_index >= self.history_items.len() {
                    self.history_selected_index = self.history_items.len().saturating_sub(1);
                }
            }
            Err(error) => self.set_toast(format!("[错误] 历史记录刷新失败: {}", error)),
        }
    }

    pub fn toggle_history_filter(&mut self) {
        self.history_filter = match self.history_filter {
            HistoryFilter::All => HistoryFilter::Favorites,
            HistoryFilter::Favorites => HistoryFilter::All,
        };
        self.history_selected_index = 0;
        self.reload_history();
    }

    pub fn toggle_favorite_current_selected(&mut self) {
        let Some(id) = self
            .history_items
            .get(self.history_selected_index)
            .map(|item| item.id)
        else {
            return;
        };

        match self.db.toggle_favorite(id) {
            Ok(is_favorite) => {
                self.set_toast(if is_favorite {
                    "[完成] 已加入生词本"
                } else {
                    "已从生词本移除"
                });
                self.reload_history();
            }
            Err(error) => self.set_toast(format!("[错误] 收藏操作失败: {}", error)),
        }
    }

    pub fn delete_current_selected(&mut self) {
        let Some(id) = self
            .history_items
            .get(self.history_selected_index)
            .map(|item| item.id)
        else {
            return;
        };

        match self.db.delete_record(id) {
            Ok(()) => {
                self.set_toast("已删除历史记录");
                self.reload_history();
            }
            Err(error) => self.set_toast(format!("[错误] 删除失败: {}", error)),
        }
    }

    pub fn set_toast(&mut self, msg: impl Into<String>) {
        self.toast_message = Some((msg.into(), Instant::now()));
    }

    pub fn get_active_toast(&self) -> Option<&str> {
        if let Some((msg, time)) = &self.toast_message {
            if time.elapsed() < Duration::from_secs(3) {
                return Some(msg.as_str());
            }
        }
        None
    }

    pub fn clear_input(&mut self) {
        self.textarea = TextArea::default();
        set_placeholder(&mut self.textarea);
    }

    pub fn copy_result_to_clipboard(&mut self) {
        let Some(result) = self.current_result.as_ref() else {
            self.set_toast("[提示] 暂无翻译结果可复制");
            return;
        };

        let content_to_copy = match result {
            QueryOutput::Dict(detail) => {
                let mut lines = vec![detail.word.clone()];
                if let Some(phonetic) = detail.phonetic_us.as_ref().or(detail.phonetic_uk.as_ref())
                {
                    lines.push(phonetic.clone());
                }
                for definition in &detail.definitions {
                    lines.push(format!(
                        "{}: {}",
                        definition.pos,
                        definition.meanings.join("；")
                    ));
                }
                lines.join("\n")
            }
            QueryOutput::Sentence { translated, .. } => translated.clone(),
        };

        match arboard::Clipboard::new() {
            Ok(mut clipboard) => match clipboard.set_text(&content_to_copy) {
                Ok(()) => self.set_toast("[完成] 译文已复制到剪贴板"),
                Err(error) => self.set_toast(format!("[错误] 复制失败: {}", error)),
            },
            Err(error) => self.set_toast(format!("[错误] 无法访问剪贴板: {}", error)),
        }
    }

    pub fn cached_result(&self) -> Option<(String, QueryOutput)> {
        if self.is_searching {
            return None;
        }
        let query = self.get_input_string().trim().to_string();
        self.query_cache
            .get(&query)
            .cloned()
            .map(|result| (query, result))
    }

    pub fn apply_cached_result(&mut self, query: String, output: QueryOutput) {
        self.error_message = None;
        self.result_scroll_offset = 0;
        self.mode = InputMode::Normal;
        self.record_success(query, output);
    }

    pub fn begin_search(&mut self) -> Option<SearchRequest> {
        if self.is_searching {
            return None;
        }

        let query = self.get_input_string().trim().to_string();
        if query.is_empty() {
            return None;
        }

        self.next_search_id = self.next_search_id.wrapping_add(1);
        let request_id = self.next_search_id;
        let (cancel_tx, cancel_rx) = oneshot::channel();

        self.is_searching = true;
        self.error_message = None;
        self.result_scroll_offset = 0;
        self.mode = InputMode::Normal;
        self.active_search_id = Some(request_id);
        self.active_search_cancel = Some(cancel_tx);

        Some(SearchRequest {
            request_id,
            query,
            client: self.client.clone(),
            cancel: cancel_rx,
        })
    }

    pub fn apply_search_result(
        &mut self,
        request_id: u64,
        query: String,
        result: std::result::Result<QueryOutput, String>,
    ) {
        if self.active_search_id != Some(request_id) {
            return;
        }

        self.active_search_id = None;
        self.active_search_cancel = None;
        self.is_searching = false;

        match result {
            Ok(output) => self.record_success(query, output),
            Err(error) => {
                self.error_message = Some(error);
                self.current_result = None;
            }
        }
    }

    pub fn cancel_active_search(&mut self) {
        if let Some(cancel) = self.active_search_cancel.take() {
            let _ = cancel.send(());
        }
        self.active_search_id = None;
        self.is_searching = false;
    }

    fn record_success(&mut self, query: String, output: QueryOutput) {
        self.cache_result(&query, output.clone());
        let summary = output.summary();
        self.current_result = Some(output);
        match self.db.add_record(&query, &summary) {
            Ok(_) => self.reload_history(),
            Err(error) => {
                self.set_toast(format!("[警告] 翻译成功，但历史记录保存失败: {}", error));
            }
        }
    }

    fn cache_result(&mut self, query: &str, result: QueryOutput) {
        if self.query_cache.contains_key(query) {
            if let Some(index) = self.query_cache_order.iter().position(|item| item == query) {
                self.query_cache_order.remove(index);
            }
        }
        self.query_cache.insert(query.to_string(), result);
        self.query_cache_order.push_back(query.to_string());

        while self.query_cache_order.len() > QUERY_CACHE_CAPACITY {
            if let Some(oldest) = self.query_cache_order.pop_front() {
                self.query_cache.remove(&oldest);
            }
        }
    }
}

fn set_placeholder(textarea: &mut TextArea<'_>) {
    textarea.set_placeholder_text("在此输入要翻译的内容，按 Enter 即刻翻译...");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_mode_and_toast() {
        let db = Database::open(":memory:").unwrap();
        let client = Client::new();
        let mut app = App::new(client, db, ThemeMode::Dark);

        assert_eq!(app.mode, InputMode::Insert);
        assert_eq!(app.focused_pane, FocusedPane::Input);
        assert_eq!(app.get_active_toast(), None);

        app.set_toast("测试提示");
        assert_eq!(app.get_active_toast(), Some("测试提示"));

        app.set_input_string("hello world");
        assert_eq!(app.get_input_string(), "hello world");
        app.clear_input();
        assert_eq!(app.get_input_string(), "");
    }

    #[test]
    fn preserves_trailing_newline_when_loading_history() {
        let db = Database::open(":memory:").unwrap();
        let mut app = App::new(Client::new(), db, ThemeMode::Dark);

        app.set_input_string("a\n");
        assert_eq!(app.get_input_string(), "a\n");
    }

    #[test]
    fn toggles_theme_and_persists() {
        let db = Database::open(":memory:").unwrap();
        let mut app = App::new(Client::new(), db, ThemeMode::Dark);
        assert_eq!(app.theme_mode, ThemeMode::Dark);

        app.toggle_theme();
        assert_eq!(app.theme_mode, ThemeMode::Light);
        assert_eq!(
            app.db.get_config("theme").unwrap().as_deref(),
            Some("light")
        );

        app.toggle_theme();
        assert_eq!(app.theme_mode, ThemeMode::Dark);
        assert_eq!(app.db.get_config("theme").unwrap().as_deref(), Some("dark"));
    }
}
