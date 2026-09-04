use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap,
    },
    Frame,
};
use unicode_width::UnicodeWidthChar;

use crate::api::dict::QueryOutput;
use crate::tui::app::{App, FocusedPane, HistoryFilter, InputMode};
use crate::views::theme::*;

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // 主体左右双栏对照区 (各 50%)
            Constraint::Length(1), // 底部状态提示栏
        ])
        .split(f.area());

    render_dual_pane(f, app, chunks[0]);
    render_status_bar(f, app, chunks[1]);

    if app.show_history_drawer {
        render_history_drawer(f, app);
    }

    if app.show_help {
        render_help_popup(f);
    }
}

fn render_dual_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // 左侧：原文输入区
            Constraint::Percentage(50), // 右侧：译文与详细释义区
        ])
        .split(area);

    render_input_editor(f, app, columns[0]);
    render_result_view(f, app, columns[1]);
}

fn render_input_editor(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focused_pane == FocusedPane::Input;
    let (border_color, title_text) = if is_focused {
        match app.mode {
            InputMode::Insert => (GREEN, " 📝 原文输入 [INSERT] "),
            InputMode::Normal => (CYAN, " 📝 原文输入 [NORMAL] "),
        }
    } else {
        (DARK_BORDER, " 📝 原文输入 ")
    };

    let title = Span::styled(
        title_text,
        Style::default().fg(border_color).add_modifier(Modifier::BOLD),
    );

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(title);

    let inner_area = input_block.inner(area);

    // 禁用整行下划线高亮
    app.textarea.set_cursor_line_style(Style::default());
    app.textarea.set_block(input_block);
    app.textarea.set_style(Style::default().fg(FG));
    app.textarea.set_cursor_style(if is_focused && app.mode == InputMode::Insert {
        Style::default().add_modifier(Modifier::REVERSED)
    } else if is_focused && app.mode == InputMode::Normal {
        Style::default().fg(CYAN).add_modifier(Modifier::UNDERLINED)
    } else {
        Style::default()
    });
    app.textarea.set_placeholder_style(Style::default().fg(COMMENT));

    f.render_widget(&app.textarea, area);

    // 同步物理硬件终端光标：仅在 Insert 模式下显示，让输入法 (IME) 候选框能跟随光标实时定位在正确字符右侧
    if is_focused && app.mode == InputMode::Insert && !app.show_history_drawer && !app.show_help {
        let (cursor_row, cursor_col) = app.textarea.cursor();

        if (cursor_row as u16) < inner_area.height {
            let screen_y = inner_area.y + (cursor_row as u16);

            // 计算当前行前 cursor_col 个字符的真实显示宽度（中文占2，英文占1）
            let lines = app.textarea.lines();
            let current_line = lines.get(cursor_row).map(|s| s.as_str()).unwrap_or("");
            let mut width_before_cursor = 0u16;
            for c in current_line.chars().take(cursor_col) {
                width_before_cursor += UnicodeWidthChar::width(c).unwrap_or(0) as u16;
            }

            if width_before_cursor < inner_area.width {
                let screen_x = inner_area.x + width_before_cursor;
                f.set_cursor_position((screen_x, screen_y));
            }
        }
    }
}

fn render_result_view(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_pane == FocusedPane::Result;
    let border_color = if is_focused {
        BLUE
    } else {
        DARK_BORDER
    };

    let title_text = if is_focused {
        " 📖 译文与词典对照 [NORMAL] "
    } else {
        " 📖 译文与词典对照 "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(title_text, Style::default().fg(border_color).add_modifier(Modifier::BOLD)));

    if app.is_searching {
        let loading = Paragraph::new("\n\n  ⏳ 正在检索翻译与词典数据，请稍候...")
            .style(Style::default().fg(YELLOW))
            .block(block);
        f.render_widget(loading, area);
        return;
    }

    if let Some(ref err) = app.error_message {
        let err_widget = Paragraph::new(format!("\n\n  ❌ 查询出错: {}", err))
            .style(Style::default().fg(RED))
            .block(block);
        f.render_widget(err_widget, area);
        return;
    }

    match &app.current_result {
        Some(QueryOutput::Dict(detail)) => {
            let mut lines = Vec::new();

            // 词条胶囊徽章与音标 (对齐 HTML: 背景 #283449, 文字 #7aa2f7 加粗)
            let mut word_spans = vec![
                Span::styled(
                    format!("  {}  ", detail.word),
                    Style::default().fg(BLUE).bg(SELECTION).add_modifier(Modifier::BOLD),
                ),
                Span::raw("   "),
            ];
            if let Some(ref us) = detail.phonetic_us {
                let clean = us.trim_matches(|c| c == '/' || c == '[' || c == ']' || c == ' ');
                word_spans.push(Span::styled(format!("美 [{}]   ", clean), Style::default().fg(CYAN).add_modifier(Modifier::BOLD)));
            }
            if let Some(ref uk) = detail.phonetic_uk {
                let clean = uk.trim_matches(|c| c == '/' || c == '[' || c == ']' || c == ' ');
                word_spans.push(Span::styled(format!("英 [{}]", clean), Style::default().fg(CYAN).add_modifier(Modifier::BOLD)));
            }
            lines.push(Line::from(word_spans));
            lines.push(Line::from(""));

            // 词典释义
            if !detail.definitions.is_empty() {
                lines.push(Line::from(Span::styled(" 【词典释义】", Style::default().fg(GREEN).add_modifier(Modifier::BOLD))));
                for def in &detail.definitions {
                    let formatted_pos = if def.pos.ends_with('.') {
                        def.pos.clone()
                    } else {
                        format!("{}.", def.pos)
                    };
                    let pos_span = if !def.pos.is_empty() {
                        Span::styled(format!("    {:>5}  ", formatted_pos), Style::default().fg(ORANGE).add_modifier(Modifier::BOLD))
                    } else {
                        Span::styled("           ", Style::default())
                    };
                    let meanings_span = Span::styled(def.meanings.join("；"), Style::default().fg(FG));
                    lines.push(Line::from(vec![pos_span, meanings_span]));
                }
            }

            // 双语例句 (对齐 HTML: 英文 #7aa2f7 蓝色, 中文 #787c99 优雅灰)
            if !detail.examples.is_empty() {
                if !detail.definitions.is_empty() {
                    lines.push(Line::from(""));
                }
                lines.push(Line::from(Span::styled(" 【双语例句】", Style::default().fg(MAGENTA).add_modifier(Modifier::BOLD))));
                for (i, eg) in detail.examples.iter().enumerate() {
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {}. ", i + 1), Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
                        Span::styled(&eg.orig, Style::default().fg(BLUE)),
                    ]));
                    lines.push(Line::from(vec![
                        Span::raw("     "),
                        Span::styled(&eg.trans, Style::default().fg(COMMENT)),
                    ]));
                }
            }

            let paragraph = Paragraph::new(lines)
                .block(block)
                .wrap(Wrap { trim: false })
                .scroll((app.result_scroll_offset, 0));
            f.render_widget(paragraph, area);
        }
        Some(QueryOutput::Sentence { original: _, translated, detected_lang, target_lang }) => {
            let mut lines = Vec::new();
            // 对齐 HTML: [EN -> ZH] Google 翻译 胶囊标签 (背景 #283449, 文字 #bb9af7)
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  [{} -> {}] Google 翻译  ", detected_lang.to_uppercase(), target_lang.to_uppercase()),
                    Style::default().fg(MAGENTA).bg(SELECTION).add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(""));

            // 对齐 HTML: 纯粹高亮绿色输出译文，去除冗余标签
            for line in translated.lines() {
                lines.push(Line::from(Span::styled(format!("  {}", line), Style::default().fg(GREEN).add_modifier(Modifier::BOLD))));
            }

            let paragraph = Paragraph::new(lines)
                .block(block)
                .wrap(Wrap { trim: false })
                .scroll((app.result_scroll_offset, 0));
            f.render_widget(paragraph, area);
        }
        None => {
            // 对齐 HTML 设计语言的精致 Tokyo Night 欢迎指南卡片
            let mut lines = Vec::new();
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  🚀 Tokyo Night 双引擎极速翻译  ", Style::default().fg(CYAN).bg(SELECTION).add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("  在左栏输入要查询的内容，按 Enter 即刻在此呈现：", Style::default().fg(FG))));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  • ⚡ 单词智能精查: ", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
                Span::styled("英美权威双音标、词性精解、双语例句", Style::default().fg(COMMENT)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  • 🌐 长句流畅互译: ", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("Google 翻译中英精准互译，支持大段长文", Style::default().fg(COMMENT)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  • ⌨️ 极致双模式:   ", Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
                Span::styled("Normal / Insert 无缝切换，Vim 键位平滑滚动", Style::default().fg(COMMENT)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  • ⭐ 生词与历史:   ", Style::default().fg(MAGENTA).add_modifier(Modifier::BOLD)),
                Span::styled("随时按 h 呼出生词抽屉，一键收藏与重查", Style::default().fg(COMMENT)),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  💡 常用提示: ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
                Span::styled("i 开始输入 | Esc 导航 | Tab 切栏 | y 复制译文 | q 退出", Style::default().fg(COMMENT)),
            ]));

            let paragraph = Paragraph::new(lines).block(block);
            f.render_widget(paragraph, area);
        }
    }
}

fn render_history_drawer(f: &mut Frame, app: &App) {
    let area = centered_rect(75, 75, f.area());
    f.render_widget(Clear, area);

    let filter_title = match app.history_filter {
        HistoryFilter::All => " 📜 全部历史 (按 c 切换到生词本 | Esc/h 关闭) ",
        HistoryFilter::Favorites => " ⭐ 我的生词本 (按 c 切换全部历史 | Esc/h 关闭) ",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(PURPLE))
        .title(Span::styled(filter_title, Style::default().fg(PURPLE).add_modifier(Modifier::BOLD)));

    let items: Vec<ListItem> = app
        .history_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == app.history_selected_index;
            let fav_icon = if item.is_favorite { "★ " } else { "  " };
            let prefix = if is_selected { "▶ " } else { "  " };

            let mut spans = vec![
                Span::styled(prefix, Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
                Span::styled(fav_icon, Style::default().fg(YELLOW)),
                Span::styled(
                    format!("{:<20}", item.query),
                    if is_selected {
                        Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(FG)
                    },
                ),
            ];

            if !item.result_summary.is_empty() {
                let short_summary = if item.result_summary.chars().count() > 36 {
                    format!("  {}...", item.result_summary.chars().take(36).collect::<String>())
                } else {
                    format!("  {}", item.result_summary)
                };
                spans.push(Span::styled(short_summary, Style::default().fg(COMMENT)));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = Vec::new();

    // 1. Toast 临时消息优先展示（如复制反馈）
    if let Some(toast) = app.get_active_toast() {
        spans.push(Span::styled(
            format!(" {} ", toast),
            Style::default().fg(ratatui::style::Color::Black).bg(YELLOW).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" "));
    }

    // 2. 模式与按键提示
    if app.show_history_drawer {
        spans.push(Span::styled(" [历史抽屉] ", Style::default().fg(ratatui::style::Color::Black).bg(PURPLE).add_modifier(Modifier::BOLD)));
        spans.push(Span::raw(" "));
        spans.push(Span::styled("j/k: 浏览 | Enter: 重新查询 | f: 收藏 | d: 删除 | c: 过滤 | Esc/q/h: 关闭", Style::default().fg(FG)));
    } else {
        match app.mode {
            InputMode::Insert => {
                spans.push(Span::styled(" [INSERT] ", Style::default().fg(ratatui::style::Color::Black).bg(GREEN).add_modifier(Modifier::BOLD)));
                spans.push(Span::raw(" "));
                spans.push(Span::styled("Esc: 退出编辑(Normal) | Enter: 翻译 | Shift+Enter: 换行 | Tab: 切换至译文", Style::default().fg(FG)));
                spans.push(Span::raw("  |  "));
                spans.push(Span::styled("Ctrl+C: 强制退出", Style::default().fg(COMMENT)));
            }
            InputMode::Normal => {
                spans.push(Span::styled(" [NORMAL] ", Style::default().fg(ratatui::style::Color::Black).bg(BLUE).add_modifier(Modifier::BOLD)));
                spans.push(Span::raw(" "));
                match app.focused_pane {
                    FocusedPane::Input => {
                        spans.push(Span::styled("i/a: 编辑 | c: 清空并输入 | x: 清空 | Tab: 译文区 | y: 复制 | Enter: 翻译", Style::default().fg(FG)));
                    }
                    FocusedPane::Result => {
                        spans.push(Span::styled("j/k: 滚动 | g/G: 顶/底 | i: 编辑原文 | Tab: 切回输入 | y: 复制", Style::default().fg(FG)));
                    }
                }
                spans.push(Span::raw("  |  "));
                spans.push(Span::styled("h: 历史  ?: 帮助  q: 退出", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)));
            }
        }
    }

    let p = Paragraph::new(Line::from(spans)).alignment(Alignment::Left);
    f.render_widget(p, area);
}

fn render_help_popup(f: &mut Frame) {
    let area = centered_rect(72, 75, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" 💡 快捷键指南 (按任意键关闭) ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(YELLOW));

    let help_text = vec![
        Line::from(Span::styled("【 INSERT 输入模式 】", Style::default().fg(GREEN).add_modifier(Modifier::BOLD))),
        Line::from(vec![Span::styled("  键盘打字       ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 直接输入原文，终端硬件光标精准跟随输入法 (IME)", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  Esc            ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 退出编辑，切换至 [NORMAL] 导航模式", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  Enter          ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 执行翻译当前内容（翻译完成后自动切回 Normal 模式）", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  Shift+Enter    ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : (或 Ctrl+J) 在输入框中正常换行", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  Tab            ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 切换焦点至译文区并转入 Normal 模式", Style::default().fg(FG))]),
        Line::from(""),
        Line::from(Span::styled("【 NORMAL 导航模式 】", Style::default().fg(BLUE).add_modifier(Modifier::BOLD))),
        Line::from(vec![Span::styled("  i / a          ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 进入 [INSERT] 模式开始编辑输入", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  c              ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 一键清空输入框并自动进入 [INSERT] 模式", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  x              ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 清空输入框内容（保持 Normal 模式）", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  Tab / ← / →    ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 在 [原文输入区] 与 [译文对照区] 之间切换焦点", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  j / k          ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 原文区移动光标行 / 译文区平滑单行滚动", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  g / G          ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : (译文区) 直达顶部 / 底部", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  d / u          ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : (译文区) 向下翻页 / 向上翻页 (PageUp/PageDn 同效)", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  y              ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 一键复制译文内容到系统剪贴板", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  h              ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 打开 / 关闭 [历史记录与生词本抽屉]", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  ?              ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 打开快捷键指南弹窗", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  q              ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 安全退出应用", Style::default().fg(FG))]),
        Line::from(""),
        Line::from(Span::styled("【 历史与生词抽屉 】", Style::default().fg(PURPLE).add_modifier(Modifier::BOLD))),
        Line::from(vec![Span::styled("  j / k / ↑ / ↓  ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 上下浏览历史条目", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  Enter          ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 将选中条目填入输入框并即刻查询", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  f              ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 收藏 / 取消收藏当前条目", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  d              ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 从数据库中删除当前条目", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  c              ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 切换 [全部历史] 与 [⭐ 生词本]", Style::default().fg(FG))]),
        Line::from(vec![Span::styled("  Esc / q / h    ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 关闭抽屉返回主界面", Style::default().fg(FG))]),
    ];

    let p = Paragraph::new(help_text).block(block);
    f.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
