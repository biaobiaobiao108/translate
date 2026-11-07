use std::io::{self, Write};

use colored::*;
use terminal_size::{terminal_size, Width};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::api::dict::{QueryOutput, WordDetail};
use crate::text::sanitize_terminal_text;
use crate::views::theme::{CliTheme, Rgb, ThemeMode};

pub fn render_cli_output(output: &QueryOutput, mode: ThemeMode) -> io::Result<()> {
    let stdout = io::stdout();
    let mut output_writer = stdout.lock();
    let theme = mode.cli();
    match output {
        QueryOutput::Dict(detail) => render_word_card(detail, &theme, &mut output_writer),
        QueryOutput::Sentence {
            original,
            translated,
            detected_lang,
            target_lang,
        } => render_sentence_card(
            original,
            translated,
            detected_lang,
            target_lang,
            &theme,
            &mut output_writer,
        ),
    }
}

fn render_word_card<W: Write>(
    detail: &WordDetail,
    theme: &CliTheme,
    output: &mut W,
) -> io::Result<()> {
    writeln!(output)?;
    render_card_header(&detail.word, theme.blue, theme, output)?;

    // Phonetics & Tags row
    let mut plain_phonetics = Vec::new();
    let mut styled_phonetics = Vec::new();
    if let Some(ref us) = detail.phonetic_us {
        let safe = sanitize_terminal_text(us);
        let clean = safe.trim_matches(|c| c == '/' || c == '[' || c == ']' || c == ' ');
        plain_phonetics.push(format!("美 [{}]", clean));
        styled_phonetics.push(format!(
            "{} {}",
            color("美", theme.muted, theme),
            bold_color(&format!("[{}]", clean), theme.cyan, theme)
        ));
    }
    if let Some(ref uk) = detail.phonetic_uk {
        let safe = sanitize_terminal_text(uk);
        let clean = safe.trim_matches(|c| c == '/' || c == '[' || c == ']' || c == ' ');
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
        let safe_tag = sanitize_terminal_text(tag);
        plain_tags.push(format!("[{}]", safe_tag));
        styled_tags.push(color(&format!("[{}]", safe_tag), theme.muted, theme));
    }

    let phonetics_width = str_width(plain_phonetics.join("  ").as_str());
    let tags_width = str_width(plain_tags.join(" ").as_str());
    let gap = 4;

    if !plain_phonetics.is_empty()
        && !plain_tags.is_empty()
        && phonetics_width + gap + tags_width <= inner_width()
    {
        let plain = format!("{}    {}", plain_phonetics.join("  "), plain_tags.join(" "));
        let styled = format!(
            "{}    {}",
            styled_phonetics.join("  "),
            styled_tags.join(" ")
        );
        render_content_line(&styled, str_width(&plain), theme, output)?;
    } else {
        if !plain_phonetics.is_empty() {
            let plain = plain_phonetics.join("  ");
            if str_width(&plain) <= inner_width() {
                let styled = styled_phonetics.join("  ");
                render_content_line(&styled, str_width(&plain), theme, output)?;
            } else {
                for (plain_phonetic, styled_phonetic) in
                    plain_phonetics.iter().zip(styled_phonetics.iter())
                {
                    if str_width(plain_phonetic) <= inner_width() {
                        render_content_line(
                            styled_phonetic,
                            str_width(plain_phonetic),
                            theme,
                            output,
                        )?;
                    } else {
                        for line in wrap_display(plain_phonetic, inner_width()) {
                            render_content_line(
                                &color(&line, theme.cyan, theme),
                                str_width(&line),
                                theme,
                                output,
                            )?;
                        }
                    }
                }
            }
        }
        if !plain_tags.is_empty() {
            for line in wrap_display(&plain_tags.join(" "), inner_width()) {
                let styled = color(&line, theme.muted, theme);
                render_content_line(&styled, str_width(&line), theme, output)?;
            }
        }
    }

    // Definitions section
    if !detail.definitions.is_empty() {
        render_card_section("词典释义", theme.green, theme, output)?;
        let max_pos_display_width = 8;
        let compact_layout = inner_width() < max_pos_display_width + 12;
        let meaning_width = inner_width()
            .saturating_sub(max_pos_display_width + 2)
            .max(1);

        for definition in &detail.definitions {
            let safe_pos = sanitize_terminal_text(&definition.pos);
            let pos_raw = if safe_pos.is_empty() {
                String::new()
            } else if safe_pos.ends_with('.') {
                safe_pos.clone()
            } else {
                format!("{}.", safe_pos)
            };
            let pos_display = if pos_raw.is_empty() {
                String::new()
            } else {
                format!("[{}]", pos_raw)
            };
            let pos_display = truncate_display(&pos_display, max_pos_display_width);
            let pos_color_val = pos_color(&safe_pos, theme);
            let meaning = definition.meanings.join("；");

            if compact_layout {
                if !pos_display.is_empty() {
                    render_content_line(
                        &bold_color(&pos_display, pos_color_val, theme),
                        str_width(&pos_display),
                        theme,
                        output,
                    )?;
                }
                for line in wrap_display(&meaning, inner_width()) {
                    render_content_line(
                        &color(&line, theme.foreground, theme),
                        str_width(&line),
                        theme,
                        output,
                    )?;
                }
                continue;
            }

            let meaning_lines = wrap_display(&meaning, meaning_width);

            for (line_index, line) in meaning_lines.iter().enumerate() {
                if line_index == 0 {
                    let pos_padding = max_pos_display_width.saturating_sub(str_width(&pos_display));
                    let plain_pos = format!("{}{}", pos_display, " ".repeat(pos_padding));
                    let styled_pos = bold_color(&plain_pos, pos_color_val, theme);
                    let plain_line = format!("{}  {}", plain_pos, line);
                    let styled_line =
                        format!("{}  {}", styled_pos, color(line, theme.foreground, theme));
                    render_content_line(&styled_line, str_width(&plain_line), theme, output)?;
                } else {
                    let plain_indent = " ".repeat(max_pos_display_width + 2);
                    let plain_line = format!("{}{}", plain_indent, line);
                    let styled_line =
                        format!("{}{}", plain_indent, color(line, theme.foreground, theme));
                    render_content_line(&styled_line, str_width(&plain_line), theme, output)?;
                }
            }
        }
    }

    // Examples section
    if !detail.examples.is_empty() {
        render_card_section("双语例句", theme.magenta, theme, output)?;

        for (index, example) in detail.examples.iter().enumerate() {
            let number_prefix = format!("{}. ", index + 1);
            let prefix_width = str_width(number_prefix.as_str());
            let example_width = inner_width().saturating_sub(prefix_width).max(1);

            for (line_index, line) in wrap_display(&example.orig, example_width)
                .iter()
                .enumerate()
            {
                if line_index == 0 {
                    let plain_line = format!("{}{}", number_prefix, line);
                    let styled_line = format!(
                        "{}{}",
                        bold_color(&number_prefix, theme.blue, theme),
                        bold_color(line, theme.foreground, theme)
                    );
                    render_content_line(&styled_line, str_width(&plain_line), theme, output)?;
                } else {
                    let indent = " ".repeat(prefix_width);
                    let plain_line = format!("{}{}", indent, line);
                    let styled_line =
                        format!("{}{}", indent, bold_color(line, theme.foreground, theme));
                    render_content_line(&styled_line, str_width(&plain_line), theme, output)?;
                }
            }

            let arrow = "↳ ";
            let arrow_width = str_width(arrow);
            let trans_indent = " ".repeat(prefix_width);
            let trans_width = inner_width()
                .saturating_sub(prefix_width + arrow_width)
                .max(1);

            for (line_index, line) in wrap_display(&example.trans, trans_width).iter().enumerate() {
                if line_index == 0 {
                    let plain_line = format!("{}{}{}", trans_indent, arrow, line);
                    let styled_line = format!(
                        "{}{}{}",
                        trans_indent,
                        color(arrow, theme.cyan, theme),
                        color(line, theme.translation, theme)
                    );
                    render_content_line(&styled_line, str_width(&plain_line), theme, output)?;
                } else {
                    let indent = " ".repeat(prefix_width + arrow_width);
                    let plain_line = format!("{}{}", indent, line);
                    let styled_line =
                        format!("{}{}", indent, color(line, theme.translation, theme));
                    render_content_line(&styled_line, str_width(&plain_line), theme, output)?;
                }
            }
        }
    }

    render_card_footer(theme, output)?;
    writeln!(output)?;
    Ok(())
}

fn render_sentence_card<W: Write>(
    original: &str,
    translated: &str,
    detected_lang: &str,
    target_lang: &str,
    theme: &CliTheme,
    output: &mut W,
) -> io::Result<()> {
    writeln!(output)?;
    let header_title = format!(
        "微软翻译 · {} ➔ {}",
        detected_lang.to_uppercase(),
        target_lang.to_uppercase()
    );
    render_card_header(&header_title, theme.magenta, theme, output)?;

    // Original section
    render_card_section("原文", theme.blue, theme, output)?;
    let text_width = inner_width().saturating_sub(2).max(1);
    for line in wrap_display(original, text_width) {
        let plain_line = format!("  {}", line);
        let styled_line = format!("  {}", color(&line, theme.foreground, theme));
        render_content_line(&styled_line, str_width(&plain_line), theme, output)?;
    }

    // Translated section
    render_card_section("译文", theme.translation, theme, output)?;
    for line in wrap_display(translated, text_width) {
        let plain_line = format!("  {}", line);
        let styled_line = format!("  {}", bold_color(&line, theme.translation, theme));
        render_content_line(&styled_line, str_width(&plain_line), theme, output)?;
    }

    render_card_footer(theme, output)?;
    writeln!(output)
}

fn render_card_header<W: Write>(
    title: &str,
    title_color: Rgb,
    theme: &CliTheme,
    output: &mut W,
) -> io::Result<()> {
    let total_width = card_width();
    let safe_title = sanitize_terminal_text(title);
    let display_title = truncate_display(&safe_title, total_width.saturating_sub(6).max(1));
    let title_width = str_width(&display_title);
    let remaining = total_width.saturating_sub(title_width + 5).max(1);

    writeln!(
        output,
        "  {}{}{}{}{}",
        color("╭─ ", theme.border, theme),
        bold_color(&display_title, title_color, theme),
        color(" ", theme.border, theme),
        color(&"─".repeat(remaining), theme.border, theme),
        color("╮", theme.border, theme)
    )
}

fn render_card_section<W: Write>(
    title: &str,
    section_color: Rgb,
    theme: &CliTheme,
    output: &mut W,
) -> io::Result<()> {
    let total_width = card_width();
    let safe_title = sanitize_terminal_text(title);
    let display_title = truncate_display(&safe_title, total_width.saturating_sub(6).max(1));
    let title_width = str_width(&display_title);
    let remaining = total_width.saturating_sub(title_width + 5).max(1);

    writeln!(
        output,
        "  {}{}{}{}{}",
        color("├─ ", theme.border, theme),
        bold_color(&display_title, section_color, theme),
        color(" ", theme.border, theme),
        color(&"─".repeat(remaining), theme.border, theme),
        color("┤", theme.border, theme)
    )
}

fn render_card_footer<W: Write>(theme: &CliTheme, output: &mut W) -> io::Result<()> {
    let total_width = card_width();
    let line_count = total_width.saturating_sub(2).max(1);
    writeln!(
        output,
        "  {}{}{}",
        color("╰", theme.border, theme),
        color(&"─".repeat(line_count), theme.border, theme),
        color("╯", theme.border, theme)
    )
}

fn render_content_line<W: Write>(
    styled_text: &str,
    plain_width: usize,
    theme: &CliTheme,
    output: &mut W,
) -> io::Result<()> {
    let iw = inner_width();
    let padding = iw.saturating_sub(plain_width);
    writeln!(
        output,
        "  {} {}{} {}",
        color("│", theme.border, theme),
        styled_text,
        " ".repeat(padding),
        color("│", theme.border, theme)
    )
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

fn terminal_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .or_else(|| terminal_size().map(|(Width(width), _)| usize::from(width)))
        .filter(|width| *width > 0)
        .unwrap_or(80)
}

fn card_width() -> usize {
    card_width_for(terminal_width())
}

fn card_width_for(terminal_width: usize) -> usize {
    terminal_width.saturating_sub(2).clamp(18, 92)
}

fn inner_width() -> usize {
    card_width().saturating_sub(4)
}

/// Wrap by terminal display width rather than byte count. This keeps CJK
/// text, phonetics, and long URLs from pushing the rest of a card off-screen.
fn wrap_display(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![String::new()];
    }

    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let sanitized = sanitize_terminal_text(&normalized);
    let mut wrapped = Vec::new();
    for source_line in sanitized.split('\n') {
        let source_line = source_line.trim();
        if source_line.is_empty() {
            wrapped.push(String::new());
            continue;
        }

        let mut line = String::new();
        let mut width: usize = 0;
        for ch in source_line.chars() {
            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            if !line.is_empty() && width.saturating_add(char_width) > max_width {
                if let Some(split_index) = last_wrap_break(&line) {
                    let head = line[..split_index].trim_end();
                    if !head.is_empty() {
                        wrapped.push(head.to_string());
                    }
                    line = line[split_index..].trim_start().to_string();
                } else {
                    wrapped.push(line.trim_end().to_string());
                    line.clear();
                }
                width = str_width(&line);
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

fn last_wrap_break(line: &str) -> Option<usize> {
    line.char_indices()
        .filter(|(_, character)| character.is_whitespace() || matches!(character, '-' | '/'))
        .map(|(index, character)| index + character.len_utf8())
        .next_back()
}

fn truncate_display(text: &str, max_width: usize) -> String {
    if str_width(text) <= max_width {
        return text.to_string();
    }
    if max_width == 0 {
        return String::new();
    }
    if max_width == 1 {
        return "…".to_string();
    }

    let content_width = max_width - 1;
    let mut truncated = String::new();
    let mut width: usize = 0;
    for character in text.chars() {
        let character_width = UnicodeWidthChar::width(character).unwrap_or(0);
        if width.saturating_add(character_width) > content_width {
            break;
        }
        truncated.push(character);
        width = width.saturating_add(character_width);
    }
    truncated.push('…');
    truncated
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
        assert!(lines.iter().all(|line| str_width(line.as_str()) <= 6));
        assert!(lines.iter().any(|line| line == "hello"));
    }

    #[test]
    fn wraps_english_at_word_boundaries() {
        let lines = wrap_display("Iron reacts with water and air", 20);
        assert_eq!(lines, vec!["Iron reacts with", "water and air"]);
        assert!(!lines.iter().any(|line| line.ends_with("ai")));
    }

    #[test]
    fn wraps_long_words_only_when_necessary() {
        let lines = wrap_display("extraordinary", 6);
        assert_eq!(lines, vec!["extrao", "rdinar", "y"]);
    }

    #[test]
    fn strips_controls_before_rendering() {
        let lines = wrap_display("safe\u{1b}[31m text", 20);
        assert_eq!(lines, vec!["safe[31m text"]);
    }

    #[test]
    fn truncates_display_width_with_ellipsis() {
        assert_eq!(truncate_display("你好世界", 5), "你好…");
        assert_eq!(truncate_display("short", 10), "short");
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

    #[test]
    fn card_width_respects_small_terminals() {
        assert_eq!(card_width_for(20), 18);
        assert_eq!(card_width_for(120), 92);
    }

    #[test]
    fn writes_cards_to_any_output() {
        let detail = WordDetail {
            word: "rust".to_string(),
            phonetic_us: None,
            phonetic_uk: None,
            tags: Vec::new(),
            definitions: Vec::new(),
            examples: Vec::new(),
        };
        let mut output = Vec::new();
        render_word_card(&detail, &ThemeMode::Dark.cli(), &mut output).unwrap();
        assert!(String::from_utf8(output).unwrap().contains("rust"));
    }
}
