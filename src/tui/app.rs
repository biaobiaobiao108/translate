use reqwest::Client;
use tui_textarea::TextArea;
use unicode_width::UnicodeWidthChar;
use crate::api::dict::{smart_query, QueryOutput};
use crate::db::{Database, HistoryItem};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FocusedPane {
    Input,
    Result,
    HistoryModal,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HistoryFilter {
    All,
    Favorites,
}

pub struct App<'a> {
    pub should_quit: bool,
    pub focused_pane: FocusedPane,
    pub textarea: TextArea<'a>,
    pub is_searching: bool,
    pub current_result: Option<QueryOutput>,
    pub error_message: Option<String>,
    pub result_scroll_offset: u16,

    // 历史与生词抽屉
    pub show_history_drawer: bool,
    pub history_items: Vec<HistoryItem>,
    pub history_selected_index: usize,
    pub history_filter: HistoryFilter,

    pub show_help: bool,
    pub db: Database,
    pub client: Client,
}

impl<'a> App<'a> {
    pub fn new(client: Client, db: Database) -> Self {
        let history = db.list_history(false, 100).unwrap_or_default();
        let mut textarea = TextArea::default();
        textarea.set_placeholder_text("在此输入要翻译的内容，按 Enter 即刻翻译...");

        Self {
            should_quit: false,
            focused_pane: FocusedPane::Input,
            textarea,
            is_searching: false,
            current_result: None,
            error_message: None,
            result_scroll_offset: 0,
            show_history_drawer: false,
            history_items: history,
            history_selected_index: 0,
            history_filter: HistoryFilter::All,
            show_help: false,
            db,
            client,
        }
    }

    pub fn get_input_string(&self) -> String {
        self.textarea.lines().join("\n")
    }

    pub fn set_input_string(&mut self, s: &str) {
        let lines: Vec<String> = s.lines().map(|l| l.to_string()).collect();
        let lines = if lines.is_empty() { vec![String::new()] } else { lines };
        self.textarea = TextArea::new(lines);
        self.textarea.move_cursor(tui_textarea::CursorMove::Bottom);
        self.textarea.move_cursor(tui_textarea::CursorMove::End);
    }

    /// 插入字符并检查是否达到行末宽度，实现平滑自动换行
    pub fn insert_char_with_wrap(&mut self, c: char, max_width: u16) {
        if max_width > 4 {
            let (cursor_row, cursor_col) = self.textarea.cursor();
            if let Some(line) = self.textarea.lines().get(cursor_row) {
                let mut current_line_width = 0u16;
                for ch in line.chars().take(cursor_col) {
                    current_line_width += UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
                }
                let char_width = UnicodeWidthChar::width(c).unwrap_or(0) as u16;
                if current_line_width + char_width >= max_width {
                    self.textarea.insert_newline();
                }
            }
        }
        self.textarea.insert_char(c);
    }

    pub fn paste_text(&mut self, text: &str) {
        let mut lines = text.split('\n').peekable();
        while let Some(line) = lines.next() {
            let clean = line.trim_end_matches('\r');
            self.textarea.insert_str(clean);
            if lines.peek().is_some() {
                self.textarea.insert_newline();
            }
        }
    }

    pub fn reload_history(&mut self) {
        let only_fav = self.history_filter == HistoryFilter::Favorites;
        if let Ok(items) = self.db.list_history(only_fav, 100) {
            self.history_items = items;
            if self.history_selected_index >= self.history_items.len() {
                self.history_selected_index = self.history_items.len().saturating_sub(1);
            }
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
        if let Some(item) = self.history_items.get(self.history_selected_index) {
            let id = item.id;
            let _ = self.db.toggle_favorite(id);
            self.reload_history();
        }
    }

    pub fn delete_current_selected(&mut self) {
        if let Some(item) = self.history_items.get(self.history_selected_index) {
            let id = item.id;
            let _ = self.db.delete_record(id);
            self.reload_history();
        }
    }

    pub async fn trigger_search(&mut self) {
        let text = self.get_input_string().trim().to_string();
        if text.is_empty() {
            return;
        }

        self.is_searching = true;
        self.error_message = None;
        self.result_scroll_offset = 0;

        match smart_query(&self.client, &text, false).await {
            Ok(output) => {
                let summary = match &output {
                    QueryOutput::Dict(d) => {
                        let mut s = String::new();
                        for def in &d.definitions {
                            s.push_str(&def.pos);
                            s.push_str(&def.meanings.join(" "));
                            s.push(' ');
                        }
                        s
                    }
                    QueryOutput::Sentence { translated, .. } => translated.clone(),
                };

                let _ = self.db.add_record(&text, &summary);
                self.current_result = Some(output);
                self.reload_history();
            }
            Err(e) => {
                self.error_message = Some(e.to_string());
                self.current_result = None;
            }
        }

        self.is_searching = false;
    }
}
