use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap},
    Frame,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::api::dict::QueryOutput;
use crate::tui::app::{App, FocusedPane, HistoryFilter, InputMode};
use crate::views::theme::*;

const WIDE_LAYOUT_MIN_WIDTH: u16 = 100;
const COMPACT_LAYOUT_MIN_HEIGHT: u16 = 16;

/// Width used by the input auto-wrap logic in the event loop. It mirrors the
/// responsive layout below so pasted text wraps at the same point at which it
/// is actually displayed.
pub fn input_content_width(terminal_width: u16) -> u16 {
    let pane_width = if terminal_width < WIDE_LAYOUT_MIN_WIDTH {
        terminal_width
    } else {
        terminal_width.saturating_mul(46) / 100
    };
    pane_width.saturating_sub(4).max(1)
}

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let compact = area.width < WIDE_LAYOUT_MIN_WIDTH || area.height < COMPACT_LAYOUT_MIN_HEIGHT;

    // Paint a stable background so panel and text contrast does not depend on
    // whether the host terminal has a dark or light default background.
    f.render_widget(Block::default().style(Style::default().bg(BG)), area);

    if compact {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(2)])
            .split(area);
        render_dual_pane(f, app, chunks[0], true);
        render_status_bar(f, app, chunks[1], true);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(7),
                Constraint::Length(2),
            ])
            .split(area);
        render_header(f, app, chunks[0]);
        render_dual_pane(f, app, chunks[1], false);
        render_status_bar(f, app, chunks[2], false);
    }

    if app.show_history_drawer {
        render_history_drawer(f, app);
    }

    if app.show_help {
        render_help_popup(f);
    }
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let state = if app.is_searching {
        "查询中"
    } else if app.error_message.is_some() {
        "查询失败"
    } else if app.current_result.is_some() {
        "已就绪"
    } else {
        "等待输入"
    };
    let focus = match app.focused_pane {
        FocusedPane::Input => "原文",
        FocusedPane::Result => "译文",
    };

    let line = Line::from(vec![
        Span::styled(
            " tran ",
            Style::default()
                .fg(Color::Black)
                .bg(CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " 终端翻译工作台 ",
            Style::default().fg(FG).add_modifier(Modifier::BOLD),
        ),
        Span::styled("·", Style::default().fg(DARK_BORDER)),
        Span::styled(
            format!(" {} · {} ", focus, state),
            Style::default().fg(FG_SUB),
        ),
    ]);
    f.render_widget(Paragraph::new(line).style(Style::default().bg(BG)), area);
}

fn render_dual_pane(f: &mut Frame, app: &mut App, area: Rect, compact: bool) {
    if compact {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(38),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(area);
        render_input_editor(f, app, rows[0]);
        render_separator(f, rows[1], false);
        render_result_view(f, app, rows[2]);
    } else {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(46),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(area);
        render_input_editor(f, app, columns[0]);
        render_separator(f, columns[1], true);
        render_result_view(f, app, columns[2]);
    }
}

fn render_separator(f: &mut Frame, area: Rect, vertical: bool) {
    if vertical {
        let lines = (0..area.height)
            .map(|_| Line::from(Span::styled("│", Style::default().fg(DARK_BORDER))))
            .collect::<Vec<_>>();
        f.render_widget(Paragraph::new(lines).style(Style::default().bg(BG)), area);
    } else {
        let line = "─".repeat(area.width as usize);
        f.render_widget(
            Paragraph::new(line).style(Style::default().fg(DARK_BORDER).bg(BG)),
            area,
        );
    }
}

fn pane_block<'a>(title: &'a str, border_color: Color) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(BG))
        .padding(Padding::horizontal(1))
        .title(Span::styled(
            title,
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        ))
}

fn render_input_editor(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focused_pane == FocusedPane::Input;
    let (border_color, title_text) = if is_focused {
        match app.mode {
            InputMode::Insert => (GREEN, " 原文输入 · INSERT "),
            InputMode::Normal => (CYAN, " 原文输入 · NORMAL "),
        }
    } else {
        (DARK_BORDER, " 原文输入 ")
    };

    let input_block = pane_block(title_text, border_color);
    let inner_area = input_block.inner(area);

    app.textarea.set_cursor_line_style(Style::default());
    app.textarea.set_block(input_block);
    app.textarea.set_style(Style::default().fg(FG).bg(BG));
    app.textarea
        .set_cursor_style(if is_focused && app.mode == InputMode::Insert {
            Style::default()
                .fg(Color::Black)
                .bg(CYAN)
                .add_modifier(Modifier::BOLD)
        } else if is_focused && app.mode == InputMode::Normal {
            Style::default().fg(CYAN).add_modifier(Modifier::UNDERLINED)
        } else {
            Style::default()
        });
    app.textarea
        .set_placeholder_style(Style::default().fg(COMMENT).bg(BG));

    f.render_widget(&app.textarea, area);

    // Keep the hardware cursor visible for IME positioning. The text area
    // itself handles its viewport; clamping here prevents an invalid cursor
    // position when the terminal is temporarily resized to a tiny window.
    if is_focused && app.mode == InputMode::Insert && !app.show_history_drawer && !app.show_help {
        let (cursor_row, cursor_col) = app.textarea.cursor();
        if inner_area.width > 0 && inner_area.height > 0 {
            let lines = app.textarea.lines();
            let current_line = lines.get(cursor_row).map(String::as_str).unwrap_or("");
            let width_before_cursor = current_line
                .chars()
                .take(cursor_col)
                .map(|ch| UnicodeWidthChar::width(ch).unwrap_or(0) as u16)
                .sum::<u16>();
            let screen_x =
                inner_area.x + width_before_cursor.min(inner_area.width.saturating_sub(1));
            let screen_y =
                inner_area.y + (cursor_row as u16).min(inner_area.height.saturating_sub(1));
            f.set_cursor_position((screen_x, screen_y));
        }
    }
}

fn render_result_view(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focused_pane == FocusedPane::Result;
    let border_color = if is_focused { BLUE } else { DARK_BORDER };
    let title_text = if is_focused {
        " 译文 / 词典 · NORMAL "
    } else {
        " 译文 / 词典 "
    };
    let block = pane_block(title_text, border_color);

    if app.is_searching {
        let loading = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "正在检索翻译与词典数据，请稍候…",
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "网络请求在后台执行，输入区仍可安全浏览。",
                Style::default().fg(FG_SUB),
            )),
        ])
        .style(Style::default().fg(FG).bg(BG))
        .wrap(Wrap { trim: false })
        .block(block);
        f.render_widget(loading, area);
        return;
    }

    if let Some(ref error) = app.error_message {
        let error_widget = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "查询失败",
                Style::default().fg(RED).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(error.as_str(), Style::default().fg(FG))),
            Line::from(""),
            Line::from(Span::styled(
                "修改左侧内容后按 Enter 重试。",
                Style::default().fg(COMMENT),
            )),
        ])
        .style(Style::default().fg(FG).bg(BG))
        .wrap(Wrap { trim: false })
        .block(block);
        f.render_widget(error_widget, area);
        return;
    }

    match &app.current_result {
        Some(QueryOutput::Dict(detail)) => {
            let mut lines = Vec::new();
            let mut word_spans = vec![Span::styled(
                format!(" {} ", detail.word),
                Style::default()
                    .fg(BLUE)
                    .bg(SELECTION)
                    .add_modifier(Modifier::BOLD),
            )];
            if let Some(ref us) = detail.phonetic_us {
                let clean = us.trim_matches(|c| c == '/' || c == '[' || c == ']' || c == ' ');
                word_spans.push(Span::styled(
                    format!("  美 [{}]", clean),
                    Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
                ));
            }
            if let Some(ref uk) = detail.phonetic_uk {
                let clean = uk.trim_matches(|c| c == '/' || c == '[' || c == ']' || c == ' ');
                word_spans.push(Span::styled(
                    format!("  英 [{}]", clean),
                    Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
                ));
            }
            lines.push(Line::from(word_spans));
            if !detail.tags.is_empty() {
                lines.push(Line::from(Span::styled(
                    format!("标签：{}", detail.tags.join(" · ")),
                    Style::default().fg(COMMENT),
                )));
            }
            lines.push(Line::from(""));

            if !detail.definitions.is_empty() {
                lines.push(section_line("词典释义", GREEN));
                for definition in &detail.definitions {
                    let formatted_pos = if definition.pos.is_empty() {
                        String::new()
                    } else if definition.pos.ends_with('.') {
                        definition.pos.clone()
                    } else {
                        format!("{}.", definition.pos)
                    };
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  {:>6}  ", formatted_pos),
                            Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(definition.meanings.join("；"), Style::default().fg(FG)),
                    ]));
                }
            }

            if !detail.examples.is_empty() {
                if !detail.definitions.is_empty() {
                    lines.push(Line::from(""));
                }
                lines.push(section_line("双语例句", MAGENTA));
                for (index, example) in detail.examples.iter().enumerate() {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  {}. ", index + 1),
                            Style::default().fg(BLUE).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(&example.orig, Style::default().fg(BLUE)),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled("     -> ", Style::default().fg(COMMENT)),
                        Span::styled(&example.trans, Style::default().fg(FG_SUB)),
                    ]));
                }
            }

            let scroll_offset = app.result_scroll_offset;
            app.result_scroll_offset =
                render_scrolled_paragraph(f, area, block, lines, scroll_offset);
        }
        Some(QueryOutput::Sentence {
            original,
            translated,
            detected_lang,
            target_lang,
        }) => {
            let mut lines = vec![Line::from(Span::styled(
                format!(
                    " {} -> {} · Google 翻译 ",
                    detected_lang.to_uppercase(),
                    target_lang.to_uppercase()
                ),
                Style::default()
                    .fg(MAGENTA)
                    .bg(SELECTION)
                    .add_modifier(Modifier::BOLD),
            ))];
            lines.push(Line::from(""));
            lines.push(section_line("原文", BLUE));
            for line in original.lines() {
                lines.push(Line::from(Span::styled(
                    format!("  {}", line),
                    Style::default().fg(FG),
                )));
            }
            lines.push(Line::from(""));
            lines.push(section_line("译文", GREEN));
            for line in translated.lines() {
                lines.push(Line::from(Span::styled(
                    format!("  {}", line),
                    Style::default().fg(GREEN),
                )));
            }

            let scroll_offset = app.result_scroll_offset;
            app.result_scroll_offset =
                render_scrolled_paragraph(f, area, block, lines, scroll_offset);
        }
        None => {
            let lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "在左侧输入内容，按 Enter 开始查询。",
                    Style::default().fg(FG).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "单词  ",
                        Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("音标、词性释义和双语例句", Style::default().fg(FG_SUB)),
                ]),
                Line::from(vec![
                    Span::styled(
                        "长句  ",
                        Style::default().fg(BLUE).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("自动检测语种并生成对照译文", Style::default().fg(FG_SUB)),
                ]),
                Line::from(vec![
                    Span::styled(
                        "历史  ",
                        Style::default().fg(PURPLE).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("按 h 打开历史记录与生词本", Style::default().fg(FG_SUB)),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Tab 切换面板 · ? 查看快捷键 · q 退出",
                    Style::default().fg(COMMENT),
                )),
            ];
            let paragraph = Paragraph::new(lines)
                .style(Style::default().fg(FG).bg(BG))
                .wrap(Wrap { trim: false })
                .block(block);
            f.render_widget(paragraph, area);
        }
    }
}

fn section_line(label: &str, color: Color) -> Line<'static> {
    Line::from(Span::styled(
        format!("  {}  ", label),
        Style::default()
            .fg(color)
            .bg(SELECTION)
            .add_modifier(Modifier::BOLD),
    ))
}

fn render_scrolled_paragraph<'a>(
    f: &mut Frame,
    area: Rect,
    block: Block<'a>,
    lines: Vec<Line<'a>>,
    scroll_offset: u16,
) -> u16 {
    let viewport = block.inner(area);
    let paragraph = Paragraph::new(lines)
        .style(Style::default().fg(FG).bg(BG))
        .wrap(Wrap { trim: false });
    let max_scroll = paragraph
        .line_count(viewport.width.max(1))
        .saturating_sub(viewport.height as usize)
        .min(u16::MAX as usize) as u16;
    let scroll_offset = scroll_offset.min(max_scroll);

    f.render_widget(paragraph.block(block).scroll((scroll_offset, 0)), area);
    scroll_offset
}

fn render_history_drawer(f: &mut Frame, app: &App) {
    let area = centered_rect(84, 78, f.area());
    f.render_widget(Clear, area);

    let filter_title = match app.history_filter {
        HistoryFilter::All => " 历史记录 · c 切换生词本 · Esc 关闭 ",
        HistoryFilter::Favorites => " 生词本 · c 切换全部历史 · Esc 关闭 ",
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(PURPLE))
        .style(Style::default().bg(BG))
        .padding(Padding::horizontal(1))
        .title(Span::styled(
            filter_title,
            Style::default().fg(PURPLE).add_modifier(Modifier::BOLD),
        ));

    let inner_width = block.inner(area).width as usize;
    let query_width = if inner_width >= 70 {
        28.min(inner_width.saturating_sub(10).max(1))
    } else {
        inner_width
            .saturating_mul(2)
            .checked_div(5)
            .unwrap_or(1)
            .clamp(1, inner_width.max(1))
    };
    let summary_width = inner_width.saturating_sub(query_width + 10).max(1);

    let items = if app.history_items.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "暂无记录。完成一次查询后会自动出现在这里。",
            Style::default().fg(COMMENT),
        )))]
    } else {
        app.history_items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let selected = index == app.history_selected_index;
                let prefix = if selected { "▶ " } else { "  " };
                let favorite = if item.is_favorite { "★ " } else { "  " };
                let query = truncate_display_width(&single_line(&item.query), query_width);
                let query_padding =
                    " ".repeat(query_width.saturating_sub(UnicodeWidthStr::width(query.as_str())));
                let summary =
                    truncate_display_width(&single_line(&item.result_summary), summary_width);
                let row = Line::from(vec![
                    Span::styled(
                        prefix,
                        Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(favorite, Style::default().fg(YELLOW)),
                    Span::styled(
                        format!("{}{}", query, query_padding),
                        Style::default().fg(if selected { FG } else { FG_SUB }),
                    ),
                    Span::styled(format!("  {}", summary), Style::default().fg(COMMENT)),
                ]);
                let row_style = if selected {
                    Style::default().bg(SELECTION)
                } else {
                    Style::default()
                };
                ListItem::new(row).style(row_style)
            })
            .collect::<Vec<_>>()
    };

    let list = List::new(items)
        .style(Style::default().fg(FG).bg(BG))
        .block(block);
    f.render_widget(list, area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect, compact: bool) {
    let mode = match app.mode {
        InputMode::Insert => "INSERT",
        InputMode::Normal => "NORMAL",
    };
    let focus = match app.focused_pane {
        FocusedPane::Input => "原文",
        FocusedPane::Result => "译文",
    };
    let state = if app.is_searching {
        "查询中"
    } else if app.error_message.is_some() {
        "查询失败"
    } else {
        "就绪"
    };

    let mut first_text = format!("[{}] {} · {}", mode, focus, state);
    if let Some(toast) = app.get_active_toast() {
        first_text.push_str("  ");
        first_text.push_str(toast);
    }
    let first_color = if app.get_active_toast().is_some() {
        YELLOW
    } else if app.error_message.is_some() {
        RED
    } else {
        FG
    };

    let hints = if app.show_history_drawer {
        "j/k 浏览 · Enter 查询 · f 收藏 · d 删除 · c 过滤 · Esc 关闭"
    } else if compact {
        match app.mode {
            InputMode::Insert => "Esc 导航 · Enter 查询 · Shift+Enter 换行 · Tab 切换",
            InputMode::Normal => "Tab 切换 · j/k 滚动 · h 历史 · ? 帮助 · q 退出",
        }
    } else {
        match app.mode {
            InputMode::Insert => {
                "Esc 导航 · Enter 查询 · Shift+Enter 换行 · Tab 切换 · Ctrl+C 退出"
            }
            InputMode::Normal => match app.focused_pane {
                FocusedPane::Input => {
                    "i/a 编辑 · c 清空并输入 · Enter 查询 · Tab 切换 · h 历史 · ? 帮助"
                }
                FocusedPane::Result => {
                    "j/k 滚动 · g/G 顶/底 · i 编辑 · Tab 切换 · y 复制 · h 历史 · ? 帮助"
                }
            },
        }
    };

    let width = area.width as usize;
    let lines = vec![
        Line::from(Span::styled(
            truncate_display_width(&first_text, width),
            Style::default()
                .fg(first_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            truncate_display_width(hints, width),
            Style::default().fg(FG_SUB),
        )),
    ];
    f.render_widget(Paragraph::new(lines).style(Style::default().bg(BG)), area);
}

fn render_help_popup(f: &mut Frame) {
    let area = centered_rect(94, 94, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" HELP · 快捷键（按任意键关闭） ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(YELLOW))
        .style(Style::default().bg(BG))
        .padding(Padding::horizontal(2));

    let help_text = vec![
        section_line("INSERT 输入模式", GREEN),
        shortcut_line("键盘输入", "编辑原文内容"),
        shortcut_line("Enter", "执行翻译 / 查词"),
        shortcut_line("Shift+Enter", "插入换行（Ctrl+J 同效）"),
        shortcut_line("Esc", "返回 NORMAL 导航模式"),
        shortcut_line("Tab", "切换到译文区"),
        Line::from(""),
        section_line("NORMAL 导航模式", BLUE),
        shortcut_line("i / a", "进入输入模式"),
        shortcut_line("Tab / ← / →", "切换输入区与译文区"),
        shortcut_line("j / k", "移动光标或滚动译文"),
        shortcut_line("g / G", "译文区跳到顶部 / 底部"),
        shortcut_line("d / u", "译文区向下 / 向上翻页"),
        shortcut_line("y", "复制当前结果"),
        shortcut_line("h", "打开历史记录与生词本"),
        shortcut_line("? / q", "帮助 / 退出"),
        Line::from(""),
        section_line("历史记录抽屉", PURPLE),
        shortcut_line("j / k", "选择条目"),
        shortcut_line("Enter", "载入并重新查询"),
        shortcut_line("f / d", "收藏 / 删除"),
        shortcut_line("c", "切换全部历史与生词本"),
        shortcut_line("Esc / h / q", "关闭抽屉"),
    ];

    let paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(FG).bg(BG))
        .wrap(Wrap { trim: false })
        .block(block);
    f.render_widget(paragraph, area);
}

fn shortcut_line(key: &str, description: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {:<16}", key),
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        ),
        Span::styled(description.to_string(), Style::default().fg(FG)),
    ])
}

fn truncate_display_width(text: &str, max_width: usize) -> String {
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }
    if max_width <= 1 {
        return "…".to_string();
    }

    let mut result = String::new();
    let mut width = 0;
    for ch in text.chars() {
        let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + char_width > max_width - 1 {
            break;
        }
        result.push(ch);
        width += char_width;
    }
    result.push('…');
    result
}

fn single_line(text: &str) -> String {
    text.replace(['\r', '\n'], " ")
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let width = if area.width < 60 {
        area.width.saturating_sub(2)
    } else {
        area.width
            .saturating_mul(percent_x)
            .checked_div(100)
            .unwrap_or(area.width)
            .max(1)
    }
    .min(area.width);
    let height = if area.height < 18 {
        area.height.saturating_sub(2)
    } else {
        area.height
            .saturating_mul(percent_y)
            .checked_div(100)
            .unwrap_or(area.height)
            .max(1)
    }
    .min(area.height);

    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{backend::TestBackend, Terminal};
    use reqwest::Client;

    use super::*;
    use crate::db::Database;

    #[test]
    fn input_width_matches_responsive_breakpoint() {
        assert_eq!(input_content_width(80), 76);
        assert_eq!(input_content_width(120), 51);
    }

    #[test]
    fn renders_compact_and_wide_layouts() {
        for (width, height) in [(80, 24), (120, 30), (40, 12)] {
            let backend = TestBackend::new(width, height);
            let mut terminal = Terminal::new(backend).unwrap();
            let db = Database::open(":memory:").unwrap();
            let mut app = App::new(Client::new(), db);
            terminal
                .draw(|frame| render(frame, &mut app))
                .expect("responsive layout should render");
        }
    }
}
