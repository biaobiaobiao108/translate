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

/// Render the history view with the same width-aware rules as query output.
/// Keeping this here avoids a second, less readable renderer in `main.rs`.
pub fn render_history_items(items: &[HistoryItem], only_favorites: bool, mode: ThemeMode) {
    let theme = mode.cli();
    let title = if only_favorites {
        "收藏生词本"
    } else {
        "历史查询记录"
    };
    let width = content_width();

    println!();
    println!(
        "  {}  {}",
        bold_color(title, theme.cyan, &theme),
        color(&format!("共 {} 条", items.len()), theme.muted, &theme)
    );
    println!("{}", divider(&theme));

    if items.is_empty() {
        println!(
            "  {}",
            color("暂无记录。查询结果会自动保存在这里。", theme.muted, &theme)
        );
        println!();
        return;
    }

    let timestamp_width = items
        .iter()
        .map(|item| UnicodeWidthStr::width(item.created_at.as_str()))
        .max()
        .unwrap_or(1);
    let query_width = if width >= 72 {
        28.min(width.saturating_sub(timestamp_width + 9).max(1))
    } else {
        width
            .saturating_mul(2)
            .checked_div(5)
            .unwrap_or(1)
            .clamp(1, width.max(1))
    };
    let summary_width = width
        .saturating_sub(query_width + timestamp_width + 8)
        .max(1);

    for item in items {
        let query = truncate_display_width(&single_line(&item.query), query_width);
        let query_padding =
            " ".repeat(query_width.saturating_sub(UnicodeWidthStr::width(query.as_str())));
        let icon = if item.is_favorite { "★" } else { "·" };
        let summary_lines = wrap_display(&single_line(&item.result_summary), summary_width);

        for (line_index, line) in summary_lines.iter().enumerate() {
            if line_index == 0 {
                println!(
                    "  {} {}{}  {}  {}",
                    color(icon, theme.yellow, &theme),
                    color(&query, theme.foreground, &theme),
                    query_padding,
                    color(line, theme.translation, &theme),
                    color(&item.created_at, theme.muted, &theme)
                );
            } else {
                println!(
                    "  {} {}",
                    " ".repeat(query_width + 5),
                    color(line, theme.translation, &theme)
                );
            }
        }
    }
    println!();
}

fn render_word_card(detail: &WordDetail, theme: &CliTheme) {
    println!();

    println!(
        "  {}",
        badge(&format!("  {}  ", detail.word), theme.blue, theme)
    );

    let mut phonetics = Vec::new();
    if let Some(ref us) = detail.phonetic_us {
        phonetics.push(format_phonetic("美", us, theme));
    }
    if let Some(ref uk) = detail.phonetic_uk {
        phonetics.push(format_phonetic("英", uk, theme));
    }
    if !phonetics.is_empty() {
        println!("  {}", phonetics.join("  "));
    }
    if !detail.tags.is_empty() {
        let tags = detail
            .tags
            .iter()
            .map(|tag| format!("[{}]", tag))
            .collect::<Vec<_>>()
            .join(" ");
        for line in wrap_display(&tags, content_width()) {
            println!("  {}", color(&line, theme.muted, theme));
        }
    }

    println!("{}", divider(theme));

    if !detail.definitions.is_empty() {
        println!("{}", format_section_title("词典释义", theme.green, theme));
        let pos_width = 7;
        let meaning_width = content_width().saturating_sub(pos_width + 4).max(1);

        for definition in &detail.definitions {
            let pos = if definition.pos.is_empty() {
                String::new()
            } else if definition.pos.ends_with('.') {
                definition.pos.clone()
            } else {
                format!("{}.", definition.pos)
            };
            let pos = truncate_display_width(&pos, pos_width);
            let meaning = definition.meanings.join("；");
            let meaning_lines = wrap_display(&meaning, meaning_width);

            for (line_index, line) in meaning_lines.iter().enumerate() {
                let pos_text = if line_index == 0 {
                    format!("{:>width$}", pos, width = pos_width)
                } else {
                    " ".repeat(pos_width)
                };
                println!(
                    "  {}  {}",
                    bold_color(&pos_text, theme.orange, theme),
                    color(line, theme.foreground, theme)
                );
            }
        }
    }

    if !detail.examples.is_empty() {
        if !detail.definitions.is_empty() {
            println!();
        }
        println!("{}", format_section_title("双语例句", theme.magenta, theme));

        for (index, example) in detail.examples.iter().enumerate() {
            let number_prefix = format!("  {}. ", index + 1);
            let continuation_prefix = " ".repeat(UnicodeWidthStr::width(number_prefix.as_str()));
            let example_width = content_width()
                .saturating_sub(UnicodeWidthStr::width(number_prefix.as_str()))
                .max(1);
            for (line_index, line) in wrap_display(&example.orig, example_width)
                .iter()
                .enumerate()
            {
                println!(
                    "{}{}",
                    if line_index == 0 {
                        number_prefix.as_str()
                    } else {
                        continuation_prefix.as_str()
                    },
                    color(line, theme.blue, theme)
                );
            }

            let translation_prefix = "     -> ";
            let translation_width = content_width()
                .saturating_sub(UnicodeWidthStr::width(translation_prefix))
                .max(1);
            for line in wrap_display(&example.trans, translation_width) {
                println!(
                    "{}{}",
                    translation_prefix,
                    color(&line, theme.translation, theme)
                );
            }
        }
    }

    println!("{}", divider(theme));
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
    let badge_text = format!(
        "  {} -> {} · Google 翻译  ",
        detected_lang.to_uppercase(),
        target_lang.to_uppercase()
    );
    println!("  {}", badge(&badge_text, theme.magenta, theme));
    println!("{}", divider(theme));

    println!("{}", format_section_title("原文", theme.blue, theme));
    let text_width = content_width().saturating_sub(2).max(1);
    for line in wrap_display(original, text_width) {
        println!("  {}", color(&line, theme.foreground, theme));
    }

    println!();
    println!("{}", format_section_title("译文", theme.green, theme));
    for line in wrap_display(translated, text_width) {
        println!("  {}", color(&line, theme.green, theme));
    }

    println!("{}", divider(theme));
    println!();
}

fn format_section_title(label: &str, color_value: Rgb, theme: &CliTheme) -> String {
    badge(&format!("  {}  ", label), color_value, theme)
}

fn format_phonetic(label: &str, raw: &str, theme: &CliTheme) -> String {
    let clean = raw.trim_matches(|c| c == '/' || c == '[' || c == ']' || c == ' ');
    bold_color(&format!("{} [{}]", label, clean), theme.cyan, theme)
}

fn badge(text: &str, color_value: Rgb, theme: &CliTheme) -> String {
    let padded = text.to_string();
    if theme.mode == ThemeMode::Auto {
        padded.bold().to_string()
    } else {
        padded
            .bold()
            .truecolor(color_value.0, color_value.1, color_value.2)
            .on_truecolor(theme.selection.0, theme.selection.1, theme.selection.2)
            .to_string()
    }
}

fn color(text: &str, color_value: Rgb, theme: &CliTheme) -> String {
    if theme.mode == ThemeMode::Auto {
        text.to_string()
    } else {
        text.truecolor(color_value.0, color_value.1, color_value.2)
            .to_string()
    }
}

fn bold_color(text: &str, color_value: Rgb, theme: &CliTheme) -> String {
    if theme.mode == ThemeMode::Auto {
        text.bold().to_string()
    } else {
        text.bold()
            .truecolor(color_value.0, color_value.1, color_value.2)
            .to_string()
    }
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
    terminal_width().saturating_sub(4).clamp(1, 100)
}

fn content_width() -> usize {
    card_width().saturating_sub(4).max(1)
}

fn divider(theme: &CliTheme) -> String {
    format!(
        "  {}",
        color(&"─".repeat(card_width()), theme.border, theme)
    )
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
    fn auto_theme_keeps_cli_body_uncolored() {
        let theme = ThemeMode::Auto.cli();
        assert_eq!(color("正文", theme.foreground, &theme), "正文");
        assert_eq!(bold_color("标题", theme.cyan, &theme), "标题");
    }

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
}
