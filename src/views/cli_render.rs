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
    let width = content_width();

    println!();
    render_card_header(&title_text, theme.cyan, &theme);

    if items.is_empty() {
        println!(
            "  {}  {}",
            border_bar(&theme),
            color("暂无记录。查询结果会自动保存在这里。", theme.muted, &theme)
        );
        render_card_footer(&theme);
        println!();
        return;
    }

    let timestamp_width = items
        .iter()
        .map(|item| UnicodeWidthStr::width(item.created_at.as_str()))
        .max()
        .unwrap_or(1);
    let query_width = if width >= 60 {
        24.min(width.saturating_sub(timestamp_width + 10).max(1))
    } else {
        width
            .saturating_mul(2)
            .checked_div(5)
            .unwrap_or(1)
            .clamp(1, width.max(1))
    };
    let summary_width = width
        .saturating_sub(query_width + timestamp_width + 10)
        .max(1);

    for item in items {
        let query = truncate_display_width(&single_line(&item.query), query_width);
        let query_padding =
            " ".repeat(query_width.saturating_sub(UnicodeWidthStr::width(query.as_str())));
        let icon = if item.is_favorite { "★" } else { "·" };
        let icon_color = if item.is_favorite {
            theme.yellow
        } else {
            theme.muted
        };
        let summary_lines = wrap_display(&single_line(&item.result_summary), summary_width);

        for (line_index, line) in summary_lines.iter().enumerate() {
            if line_index == 0 {
                println!(
                    "  {}  {} {}{}  {}  {}",
                    border_bar(&theme),
                    bold_color(icon, icon_color, &theme),
                    bold_color(&query, theme.foreground, &theme),
                    query_padding,
                    color(line, theme.translation, &theme),
                    color(&item.created_at, theme.muted, &theme)
                );
            } else {
                println!(
                    "  {}  {}",
                    border_bar(&theme),
                    format!(
                        "  {}{}",
                        " ".repeat(query_width + 2),
                        color(line, theme.translation, &theme)
                    )
                );
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
    let mut phonetic_spans = Vec::new();
    if let Some(ref us) = detail.phonetic_us {
        phonetic_spans.push(format_phonetic("美", us, theme));
    }
    if let Some(ref uk) = detail.phonetic_uk {
        phonetic_spans.push(format_phonetic("英", uk, theme));
    }

    let mut header_info = Vec::new();
    if !phonetic_spans.is_empty() {
        header_info.push(phonetic_spans.join("  "));
    }
    if !detail.tags.is_empty() {
        let tags_str = detail
            .tags
            .iter()
            .map(|tag| color(&format!("[{}]", tag), theme.muted, theme))
            .collect::<Vec<_>>()
            .join(" ");
        header_info.push(tags_str);
    }

    if !header_info.is_empty() {
        println!("  {}  {}", border_bar(theme), header_info.join("    "));
    }

    // Definitions section
    if !detail.definitions.is_empty() {
        render_card_section("词典释义", theme.green, theme);
        let max_pos_display_width = 8;
        let meaning_width = content_width().saturating_sub(max_pos_display_width + 4).max(1);

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
                    let formatted_pos = format!("{:<width$}", pos_display, width = max_pos_display_width);
                    println!(
                        "  {}  {}  {}",
                        border_bar(theme),
                        bold_color(&formatted_pos, pos_color_val, theme),
                        color(line, theme.foreground, theme)
                    );
                } else {
                    println!(
                        "  {}  {}  {}",
                        border_bar(theme),
                        " ".repeat(max_pos_display_width),
                        color(line, theme.foreground, theme)
                    );
                }
            }
        }
    }

    // Examples section
    if !detail.examples.is_empty() {
        render_card_section("双语例句", theme.magenta, theme);

        for (index, example) in detail.examples.iter().enumerate() {
            let number_prefix = format!("{}. ", index + 1);
            let prefix_width = UnicodeWidthStr::width(number_prefix.as_str());
            let example_width = content_width().saturating_sub(prefix_width + 2).max(1);

            for (line_index, line) in wrap_display(&example.orig, example_width).iter().enumerate() {
                if line_index == 0 {
                    println!(
                        "  {}  {} {}",
                        border_bar(theme),
                        bold_color(&number_prefix, theme.blue, theme),
                        bold_color(line, theme.foreground, theme)
                    );
                } else {
                    println!(
                        "  {}  {} {}",
                        border_bar(theme),
                        " ".repeat(prefix_width),
                        bold_color(line, theme.foreground, theme)
                    );
                }
            }

            let arrow = "↳ ";
            let arrow_width = UnicodeWidthStr::width(arrow);
            let translation_width = content_width().saturating_sub(prefix_width + arrow_width + 2).max(1);

            for (line_index, line) in wrap_display(&example.trans, translation_width).iter().enumerate() {
                if line_index == 0 {
                    println!(
                        "  {}  {} {}{}",
                        border_bar(theme),
                        " ".repeat(prefix_width.saturating_sub(1)),
                        color(arrow, theme.cyan, theme),
                        color(line, theme.translation, theme)
                    );
                } else {
                    println!(
                        "  {}  {} {}",
                        border_bar(theme),
                        " ".repeat(prefix_width + arrow_width),
                        color(line, theme.translation, theme)
                    );
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
        "Google 翻译 · {} ➔ {}",
        detected_lang.to_uppercase(),
        target_lang.to_uppercase()
    );
    render_card_header(&header_title, theme.magenta, theme);

    // Original section
    println!(
        "  {}  {}",
        border_bar(theme),
        bold_color("原文", theme.blue, theme)
    );
    let text_width = content_width().saturating_sub(4).max(1);
    for line in wrap_display(original, text_width) {
        println!(
            "  {}    {}",
            border_bar(theme),
            color(&line, theme.foreground, theme)
        );
    }

    // Translated section
    println!("  {}", border_bar(theme));
    println!(
        "  {}  {}",
        border_bar(theme),
        bold_color("译文", theme.green, theme)
    );
    for line in wrap_display(translated, text_width) {
        println!(
            "  {}    {}",
            border_bar(theme),
            bold_color(&line, theme.green, theme)
        );
    }

    render_card_footer(theme);
    println!();
}

fn render_card_header(title: &str, title_color: Rgb, theme: &CliTheme) {
    let title_width = UnicodeWidthStr::width(title);
    let total_width = card_width();
    let remaining = total_width.saturating_sub(title_width + 5).max(2);

    println!(
        "  {}{}{}",
        color("╭─ ", theme.border, theme),
        bold_color(title, title_color, theme),
        color(&format!(" {}", "─".repeat(remaining)), theme.border, theme)
    );
}

fn render_card_section(title: &str, section_color: Rgb, theme: &CliTheme) {
    let title_width = UnicodeWidthStr::width(title);
    let total_width = card_width();
    let remaining = total_width.saturating_sub(title_width + 5).max(2);

    println!(
        "  {}{}{}",
        color("├─ ", theme.border, theme),
        bold_color(title, section_color, theme),
        color(&format!(" {}", "─".repeat(remaining)), theme.border, theme)
    );
}

fn render_card_footer(theme: &CliTheme) {
    let total_width = card_width();
    let line_count = total_width.saturating_sub(1).max(2);
    println!(
        "  {}",
        color(&format!("╰{}", "─".repeat(line_count)), theme.border, theme)
    );
}

fn border_bar(theme: &CliTheme) -> String {
    color("│", theme.border, theme)
}

fn format_phonetic(label: &str, raw: &str, theme: &CliTheme) -> String {
    let clean = raw.trim_matches(|c| c == '/' || c == '[' || c == ']' || c == ' ');
    format!(
        "{} {}",
        color(label, theme.muted, theme),
        bold_color(&format!("[{}]", clean), theme.cyan, theme)
    )
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
    terminal_width().saturating_sub(4).clamp(30, 88)
}

fn content_width() -> usize {
    card_width().saturating_sub(6).max(20)
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
            .all(|line| UnicodeWidthStr::width(line.as_str()) <= 6));
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
}
