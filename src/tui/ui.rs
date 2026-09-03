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
use crate::tui::app::{App, FocusedPane, HistoryFilter};
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
    let border_color = if is_focused {
        CYAN
    } else {
        DARK_BORDER
    };

    let title = Span::styled(
        " 📝 原文输入 ",
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
    app.textarea.set_cursor_style(if is_focused {
        Style::default().bg(CYAN).fg(SELECTION)
    } else {
        Style::default()
    });
    app.textarea.set_placeholder_style(Style::default().fg(COMMENT));

    f.render_widget(&app.textarea, area);

    // 同步物理硬件终端光标，让输入法 (IME) 候选框能跟随光标实时定位在正确字符右侧
    if is_focused && !app.show_history_drawer && !app.show_help {
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

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(" 📖 译文与词典对照 ", Style::default().fg(border_color).add_modifier(Modifier::BOLD)));

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

            // 单词徽章与音标
            let mut word_spans = vec![
                Span::styled(format!(" {} ", detail.word), Style::default().fg(BLUE).bg(SELECTION).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
            ];
            if let Some(ref us) = detail.phonetic_us {
                word_spans.push(Span::styled(format!("美 {}  ", us), Style::default().fg(CYAN)));
            }
            if let Some(ref uk) = detail.phonetic_uk {
                word_spans.push(Span::styled(format!("英 {}", uk), Style::default().fg(CYAN)));
            }
            lines.push(Line::from(word_spans));
            lines.push(Line::from(""));

            // 词典释义
            if !detail.definitions.is_empty() {
                lines.push(Line::from(Span::styled("【 词性释义 】", Style::default().fg(GREEN).add_modifier(Modifier::BOLD))));
                for def in &detail.definitions {
                    let pos_span = if !def.pos.is_empty() {
                        Span::styled(format!("  {:>5} ", def.pos), Style::default().fg(ORANGE).add_modifier(Modifier::BOLD))
                    } else {
                        Span::styled("        ", Style::default())
                    };
                    let meanings_span = Span::styled(def.meanings.join("； "), Style::default().fg(FG));
                    lines.push(Line::from(vec![pos_span, meanings_span]));
                }
                lines.push(Line::from(""));
            }

            // 双语例句
            if !detail.examples.is_empty() {
                lines.push(Line::from(Span::styled("【 双语权威例句 】", Style::default().fg(MAGENTA).add_modifier(Modifier::BOLD))));
                for (i, eg) in detail.examples.iter().enumerate() {
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {}. ", i + 1), Style::default().fg(CYAN)),
                        Span::styled(&eg.orig, Style::default().fg(FG)),
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
            lines.push(Line::from(vec![
                Span::styled(format!(" [{} -> {}] ", detected_lang.to_uppercase(), target_lang.to_uppercase()), Style::default().fg(MAGENTA).bg(SELECTION).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled("Google 翻译", Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled("译文内容：", Style::default().fg(GREEN).add_modifier(Modifier::BOLD))));
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
            let empty_text = Paragraph::new("\n\n  💡 在左栏输入内容并按下 Enter 即在此显示译文。\n\n  • 左右等宽双栏对照，视野开阔不受单行限制\n  • 单词优先输出详细词典、音标和权威例句\n  • 句子智能提供 Google 翻译\n  • 按 Tab 切换栏目，按 h 键查看生词与历史")
                .style(Style::default().fg(COMMENT))
                .block(block);
            f.render_widget(empty_text, area);
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

            let mut spans = vec![
                Span::styled(fav_icon, Style::default().fg(YELLOW)),
                Span::styled(
                    format!("{:<20}", item.query),
                    if is_selected {
                        Style::default().fg(CYAN).bg(SELECTION).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(FG).add_modifier(Modifier::BOLD)
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
    let (focus_name, focus_tip) = if app.show_history_drawer {
        ("历史/生词抽屉", "↑/↓/PgUp/PgDn: 浏览 | Enter: 重新查询 | f: 收藏/取消 | d: 删除 | c: 过滤 | h/Esc: 关闭")
    } else {
        match app.focused_pane {
            FocusedPane::Input => ("原文输入", "Enter: 翻译 | Shift+Enter: 换行 | Tab: 切换 | h: 生词本 | Esc: 退出"),
            FocusedPane::Result => ("译文对照", "↑/↓: 滚动 | PgUp/PgDn: 翻页 | Tab: 切回输入 | h: 生词本"),
            FocusedPane::HistoryModal => ("历史抽屉", "h/Esc: 关闭"),
        }
    };

    let status_line = Line::from(vec![
        Span::styled(format!(" [{}] ", focus_name), Style::default().fg(BLUE).bg(SELECTION).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(focus_tip, Style::default().fg(FG)),
        Span::raw("  |  "),
        Span::styled("?: 帮助  q: 退出", Style::default().fg(YELLOW)),
    ]);

    let p = Paragraph::new(status_line).alignment(Alignment::Left);
    f.render_widget(p, area);
}

fn render_help_popup(f: &mut Frame) {
    let area = centered_rect(65, 65, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" 💡 快捷键指南 (按任意键关闭) ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(YELLOW));

    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled("  Enter          ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 执行翻译当前左栏输入的内容", Style::default().fg(FG))]),
        Line::from(""),
        Line::from(vec![Span::styled("  Shift+Enter    ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : (或 Ctrl+J) 在输入框中正常换行", Style::default().fg(FG))]),
        Line::from(""),
        Line::from(vec![Span::styled("  ↑ / ↓ / ← / →  ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 终端原生全方向光标自由游走与滚动", Style::default().fg(FG))]),
        Line::from(""),
        Line::from(vec![Span::styled("  PageUp / PageDn", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 快速跨页翻页", Style::default().fg(FG))]),
        Line::from(""),
        Line::from(vec![Span::styled("  Home / End     ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 快速跳转至当前行首 / 行尾", Style::default().fg(FG))]),
        Line::from(""),
        Line::from(vec![Span::styled("  Tab / Shift+Tab", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 在 [原文输入区] 与 [译文对照区] 之间切换焦点", Style::default().fg(FG))]),
        Line::from(""),
        Line::from(vec![Span::styled("  h / Ctrl+H     ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 打开 / 关闭 [历史记录与生词本抽屉]", Style::default().fg(FG))]),
        Line::from(""),
        Line::from(vec![Span::styled("  f              ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : (历史抽屉中) 收藏 / 取消收藏当前条目", Style::default().fg(FG))]),
        Line::from(""),
        Line::from(vec![Span::styled("  c              ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : (历史抽屉中) 切换 [全部历史] 与 [⭐ 生词本]", Style::default().fg(FG))]),
        Line::from(""),
        Line::from(vec![Span::styled("  Esc            ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)), Span::styled(" : 关闭弹窗/抽屉，或退出应用", Style::default().fg(FG))]),
        Line::from(""),
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
