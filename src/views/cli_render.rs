use colored::*;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::api::dict::{QueryOutput, WordDetail};
use crate::db::HistoryItem;
use crate::views::theme::{CliTheme, Rgb, ThemeMode};

pub fn render_cli_output(output: &QueryOutput, mode: ThemeMode) {
    let theme = mode.cli();
    match output {
        QueryOutput::Dict(detail) => render_word_card(detail, &theme),
        QueryOutput::Sentence {
            original,
            translated,
            detected_lang,
            target_lang,
        } => render_sentence_card(original, translated, detected_lang, target_lang, &theme),
    }
}

pub fn render_history_items(items: &[HistoryItem], only_favorites: bool, mode: ThemeMode) {
    let theme = mode.cli();
    let title = if only_favorites {
        "收藏生词本"
    } else {
        "历史查询记录"
    };
    let total_count = format!("共 {} 条", items.len());
    let title_text = format!("{} · {}", title, total_count);

    println!();
    render_card_header(&title_text, theme.cyan, &theme);

    if items.is_empty() {
        let msg = "暂无记录。查询结果会自动保存在这里。";
        let styled = color(msg, theme.muted, &theme);
        render_content_line(&styled, str_width(msg), &theme);
        render_card_footer(&theme);
        println!();
        return;
    }

    let timestamp_width = items
        .iter()
        .map(|item| str_width(item.created_at.as_str()))
        .max()
        .unwrap_or(1);
    let iw = inner_width();
    let query_width = if iw >= 60 {
        24.min(iw.saturating_sub(timestamp_width + 8).max(1))
    } else {
        iw.saturating_mul(2)
            .checked_div(5)
            .unwrap_or(1)
            .clamp(1, iw.max(1))
    };
    let summary_width = iw
        .saturating_sub(query_width + timestamp_width + 6)
        .max(1);

    for item in items {
        let query = truncate_display_width(&single_line(&item.query), query_width);
        let query_padding =
            " ".repeat(query_width.saturating_sub(str_width(query.as_str())));
        let icon = if item.is_favorite { "★" } else { "·" };
        let icon_color = if item.is_favorite {
            theme.yellow
        } else {
            theme.muted
        };
        let summary_lines = wrap_display(&single_line(&item.result_summary), summary_width);

        for (line_index, line) in summary_lines.iter().enumerate() {
            if line_index == 0 {
                let plain_line = format!(
                    "{} {}{}  {}  {}",
                    icon, query, query_padding, line, item.created_at
                );
                let styled_line = format!(
                    "{} {}{}  {}  {}",
                    bold_color(icon, icon_color, &theme),
                    bold_color(&query, theme.foreground, &theme),
                    query_padding,
                    color(line, theme.translation, &theme),
                    color(&item.created_at, theme.muted, &theme)
                );
                render_content_line(&styled_line, str_width(&plain_line), &theme);
            } else {
                let indent = " ".repeat(query_width + 4);
                let plain_line = format!("{}{}", indent, line);
                let styled_line = format!("{}{}", indent, color(line, theme.translation, &theme));
                render_content_line(&styled_line, str_width(&plain_line), &theme);
            }
        }
    }

    render_card_footer(&theme);
    println!();
}

fn render_word_card(detail: &WordDetail, theme: &CliTheme) {
    println!();
    render_card_header(&detail.word, theme.blue, theme);

    // Phonetics & Tags row
    let mut plain_phonetics = Vec::new();
    let mut styled_phonetics = Vec::new();
    if let Some(ref us) = detail.phonetic_us {
        let clean = us.trim_matches(|c| c == '/' || c == '[' || c == ']' || c == ' ');
        plain_phonetics.push(format!("美 [{}]", clean));
        styled_phonetics.push(format!(
            "{} {}",
            color("美", theme.muted, theme),
            bold_color(&format!("[{}]", clean), theme.cyan, theme)
        ));
    }
    if let Some(ref uk) = detail.phonetic_uk {
        let clean = uk.trim_matches(|c| c == '/' || c == '[' || c == ']' || c == ' ');
        plain_phonetics.push(format!("英 [{}]", clean));
        styled_phonetics.push(format!(
            "{} {}",
            color("英", theme.muted, theme),
            bold_color(&format!("[{}]", clean), theme.cyan, theme)
        ));
    }

    let mut plain_tags = Vec::new();
    let mut styled_tags = Vec::new();
    for tag in &detail.tags {
        plain_tags.push(format!("[{}]", tag));
        styled_tags.push(color(&format!("[{}]", tag), theme.muted, theme));
    }

    let phonetics_width = str_width(plain_phonetics.join("  ").as_str());
    let tags_width = str_width(plain_tags.join(" ").as_str());
    let gap = 4;

    if phonetics_width + gap + tags_width <= inner_width() && !plain_tags.is_empty() {
        let plain = format!("{}    {}", plain_phonetics.join("  "), plain_tags.join(" "));
        let styled = format!("{}    {}", styled_phonetics.join("  "), styled_tags.join(" "));
        render_content_line(&styled, str_width(&plain), theme);
    } else {
        if !plain_phonetics.is_empty() {
            let plain = plain_phonetics.join("  ");
            let styled = styled_phonetics.join("  ");
            render_content_line(&styled, str_width(&plain), theme);
        }
        if !plain_tags.is_empty() {
            for line in wrap_display(&plain_tags.join(" "), inner_width()) {
                let styled = color(&line, theme.muted, theme);
                render_content_line(&styled, str_width(&line), theme);
            }
        }
    }

    // Definitions section
    if !detail.definitions.is_empty() {
        render_card_section("词典释义", theme.green, theme);
        let max_pos_display_width = 8;
        let meaning_width = inner_width().saturating_sub(max_pos_display_width + 2).max(1);

        for definition in &detail.definitions {
            let pos_raw = if definition.pos.is_empty() {
                String::new()
            } else if definition.pos.ends_with('.') {
                definition.pos.clone()
            } else {
                format!("{}.", definition.pos)
            };
            let pos_display = if pos_raw.is_empty() {
                String::new()
            } else {
                format!("[{}]", pos_raw)
            };
            let pos_color_val = pos_color(&definition.pos, theme);
            let meaning = definition.meanings.join("；");
            let meaning_lines = wrap_display(&meaning, meaning_width);

            for (line_index, line) in meaning_lines.iter().enumerate() {
                if line_index == 0 {
                    let plain_pos = format!("{:<width$}", pos_display, width = max_pos_display_width);
                    let styled_pos = bold_color(&plain_pos, pos_color_val, theme);
                    let plain_line = format!("{}  {}", plain_pos, line);
                    let styled_line = format!("{}  {}", styled_pos, color(line, theme.foreground, theme));
                    render_content_line(&styled_line, str_width(&plain_line), theme);
                } else {
                    let plain_indent = " ".repeat(max_pos_display_width + 2);
                    let plain_line = format!("{}{}", plain_indent, line);
                    let styled_line = format!("{}{}", plain_indent, color(line, theme.foreground, theme));
                    render_content_line(&styled_line, str_width(&plain_line), theme);
                }
            }
        }
    }

    // Examples section
    if !detail.examples.is_empty() {
        render_card_section("双语例句", theme.magenta, theme);

        for (index, example) in detail.examples.iter().enumerate() {
            let number_prefix = format!("{}. ", index + 1);
            let prefix_width = str_width(number_prefix.as_str());
            let example_width = inner_width().saturating_sub(prefix_width).max(1);

            for (line_index, line) in wrap_display(&example.orig, example_width).iter().enumerate() {
                if line_index == 0 {
                    let plain_line = format!("{}{}", number_prefix, line);
                    let styled_line = format!(
                        "{}{}",
                        bold_color(&number_prefix, theme.blue, theme),
                        bold_color(line, theme.foreground, theme)
                    );
                    render_content_line(&styled_line, str_width(&plain_line), theme);
                } else {
                    let indent = " ".repeat(prefix_width);
                    let plain_line = format!("{}{}", indent, line);
                    let styled_line = format!("{}{}", indent, bold_color(line, theme.foreground, theme));
                    render_content_line(&styled_line, str_width(&plain_line), theme);
                }
            }

            let arrow = "↳ ";
            let arrow_width = str_width(arrow);
            let trans_indent = " ".repeat(prefix_width);
            let trans_width = inner_width().saturating_sub(prefix_width + arrow_width).max(1);

            for (line_index, line) in wrap_display(&example.trans, trans_width).iter().enumerate() {
                if line_index == 0 {
                    let plain_line = format!("{}{}{}", trans_indent, arrow, line);
                    let styled_line = format!(
                        "{}{}{}",
                        trans_indent,
                        color(arrow, theme.cyan, theme),
                        color(line, theme.translation, theme)
                    );
                    render_content_line(&styled_line, str_width(&plain_line), theme);
                } else {
                    let indent = " ".repeat(prefix_width + arrow_width);
                    let plain_line = format!("{}{}", indent, line);
                    let styled_line = format!("{}{}", indent, color(line, theme.translation, theme));
                    render_content_line(&styled_line, str_width(&plain_line), theme);
                }
            }
        }
    }

    render_card_footer(theme);
    println!();
}

fn render_sentence_card(
    original: &str,
    translated: &str,
    detected_lang: &str,
    target_lang: &str,
    theme: &CliTheme,
) {
    println!();
    let header_title = format!(
        "微软翻译 · {} ➔ {}",
        detected_lang.to_uppercase(),
        target_lang.to_uppercase()
    );
    render_card_header(&header_title, theme.magenta, theme);

    // Original section
    render_card_section("原文", theme.blue, theme);
    let text_width = inner_width().saturating_sub(2).max(1);
    for line in wrap_display(original, text_width) {
        let plain_line = format!("  {}", line);
        let styled_line = format!("  {}", color(&line, theme.foreground, theme));
        render_content_line(&styled_line, str_width(&plain_line), theme);
    }

    // Translated section
    render_card_section("译文", theme.translation, theme);
    for line in wrap_display(translated, text_width) {
        let plain_line = format!("  {}", line);
        let styled_line = format!("  {}", bold_color(&line, theme.translation, theme));
        render_content_line(&styled_line, str_width(&plain_line), theme);
    }

    render_card_footer(theme);
    println!();
}

fn render_card_header(title: &str, title_color: Rgb, theme: &CliTheme) {
    let title_width = str_width(title);
    let total_width = card_width();
    let remaining = total_width.saturating_sub(title_width + 5).max(1);

    println!(
        "  {}{}{}{}{}",
        color("╭─ ", theme.border, theme),
        bold_color(title, title_color, theme),
        color(" ", theme.border, theme),
        color(&"─".repeat(remaining), theme.border, theme),
        color("╮", theme.border, theme)
    );
}

fn render_card_section(title: &str, section_color: Rgb, theme: &CliTheme) {
    let title_width = str_width(title);
    let total_width = card_width();
    let remaining = total_width.saturating_sub(title_width + 5).max(1);

    println!(
        "  {}{}{}{}{}",
        color("├─ ", theme.border, theme),
        bold_color(title, section_color, theme),
        color(" ", theme.border, theme),
        color(&"─".repeat(remaining), theme.border, theme),
        color("┤", theme.border, theme)
    );
}

fn render_card_footer(theme: &CliTheme) {
    let total_width = card_width();
    let line_count = total_width.saturating_sub(2).max(1);
    println!(
        "  {}{}{}",
        color("╰", theme.border, theme),
        color(&"─".repeat(line_count), theme.border, theme),
        color("╯", theme.border, theme)
    );
}

fn render_content_line(styled_text: &str, plain_width: usize, theme: &CliTheme) {
    let iw = inner_width();
    let padding = iw.saturating_sub(plain_width);
    println!(
        "  {} {}{} {}",
        color("│", theme.border, theme),
        styled_text,
        " ".repeat(padding),
        color("│", theme.border, theme)
    );
}

fn str_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

fn pos_color(pos: &str, theme: &CliTheme) -> Rgb {
    let lower = pos.to_lowercase();
    if lower.starts_with('n') {
        theme.orange
    } else if lower.starts_with('v') {
        theme.blue
    } else if lower.starts_with("adj") || (lower.starts_with('a') && !lower.starts_with("adv")) {
        theme.green
    } else if lower.starts_with("adv") {
        theme.magenta
    } else if lower.starts_with("prep") || lower.starts_with("conj") || lower.starts_with("pron") {
        theme.cyan
    } else {
        theme.yellow
    }
}

fn color(text: &str, color_value: Rgb, _theme: &CliTheme) -> String {
    text.truecolor(color_value.0, color_value.1, color_value.2)
        .to_string()
}

fn bold_color(text: &str, color_value: Rgb, _theme: &CliTheme) -> String {
    text.bold()
        .truecolor(color_value.0, color_value.1, color_value.2)
        .to_string()
}

fn single_line(text: &str) -> String {
    text.replace(['\r', '\n'], " ")
}

fn terminal_width() -> usize {
    crossterm::terminal::size()
        .map(|(width, _)| usize::from(width))
        .unwrap_or(80)
}

fn card_width() -> usize {
    terminal_width().saturating_sub(4).clamp(36, 92)
}

fn inner_width() -> usize {
    card_width().saturating_sub(4).max(20)
}

fn truncate_display_width(text: &str, max_width: usize) -> String {
    if str_width(text) <= max_width {
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

/// Wrap by terminal display width rather than byte count. This keeps CJK
/// text, phonetics, and long URLs from pushing the rest of a card off-screen.
fn wrap_display(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![String::new()];
    }

    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut wrapped = Vec::new();
    for source_line in normalized.split('\n') {
        if source_line.trim().is_empty() {
            wrapped.push(String::new());
            continue;
        }

        let mut line = String::new();
        let mut width: usize = 0;
        for ch in source_line.trim().chars() {
            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            if !line.is_empty() && width.saturating_add(char_width) > max_width {
                wrapped.push(line.trim_end().to_string());
                line.clear();
                width = 0;
            }
            if ch.is_whitespace() && line.is_empty() {
                continue;
            }
            line.push(ch);
            width = width.saturating_add(char_width);
        }
        if !line.is_empty() {
            wrapped.push(line.trim_end().to_string());
        }
    }

    if wrapped.is_empty() {
        wrapped.push(String::new());
    }
    wrapped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_themes_have_distinct_foreground_palettes() {
        let dark = ThemeMode::Dark.cli();
        let light = ThemeMode::Light.cli();
        assert_ne!(dark.foreground, light.foreground);
    }

    #[test]
    fn wraps_wide_characters_without_exceeding_width() {
        let lines = wrap_display("你好世界 hello", 6);
        assert!(lines
            .iter()
            .all(|line| str_width(line.as_str()) <= 6));
        assert_eq!(lines.join(""), "你好世界 hello");
    }

    #[test]
    fn truncates_by_display_width() {
        assert_eq!(truncate_display_width("你好世界", 5), "你好…");
    }

    #[test]
    fn pos_color_selection() {
        let theme = ThemeMode::Dark.cli();
        assert_eq!(pos_color("n.", &theme), theme.orange);
        assert_eq!(pos_color("v.", &theme), theme.blue);
        assert_eq!(pos_color("adj.", &theme), theme.green);
        assert_eq!(pos_color("adv.", &theme), theme.magenta);
        assert_eq!(pos_color("prep.", &theme), theme.cyan);
    }

    #[test]
    fn inner_width_and_card_width_relation() {
        let cw = card_width();
        let iw = inner_width();
        assert_eq!(cw - iw, 4);
    }
}
