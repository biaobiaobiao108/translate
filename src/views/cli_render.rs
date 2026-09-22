use colored::*;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::api::dict::{QueryOutput, WordDetail};
use crate::db::HistoryItem;

// A deliberately high-contrast CLI palette. The CLI does not control the
// terminal background, so body text stays close to white and accent colors are
// reserved for labels, headings, and metadata.
const TN_FG: (u8, u8, u8) = (230, 237, 243); // #e6edf3 Main text
const TN_TRANSLATION: (u8, u8, u8) = (192, 202, 245); // #c0caf5 Secondary text
const TN_BLUE: (u8, u8, u8) = (122, 162, 247); // #7aa2f7 English / accent
const TN_CYAN: (u8, u8, u8) = (125, 207, 255); // #7dcfff Phonetics / labels
const TN_GREEN: (u8, u8, u8) = (158, 206, 106); // #9ece6a Translation / success
const TN_MAGENTA: (u8, u8, u8) = (187, 154, 247); // #bb9af7 Section labels
const TN_ORANGE: (u8, u8, u8) = (255, 158, 100); // #ff9e64 POS tags
const TN_MUTED: (u8, u8, u8) = (169, 177, 214); // #a9b1d6 Metadata
const TN_SELECTION: (u8, u8, u8) = (51, 65, 90); // #33415a Badge background
const TN_BORDER: (u8, u8, u8) = (86, 95, 137); // #565f89 Divider line

pub fn render_cli_output(output: &QueryOutput) {
    match output {
        QueryOutput::Dict(detail) => render_word_card(detail),
        QueryOutput::Sentence {
            original,
            translated,
            detected_lang,
            target_lang,
        } => render_sentence_card(original, translated, detected_lang, target_lang),
    }
}

/// Render the history view with the same width-aware rules as query output.
/// Keeping this here avoids a second, less readable renderer in `main.rs`.
pub fn render_history_items(items: &[HistoryItem], only_favorites: bool) {
    let title = if only_favorites {
        "收藏生词本"
    } else {
        "历史查询记录"
    };
    let width = content_width();

    println!();
    println!(
        "  {}  {}",
        title.bold().truecolor(TN_CYAN.0, TN_CYAN.1, TN_CYAN.2),
        format!("共 {} 条", items.len()).truecolor(TN_MUTED.0, TN_MUTED.1, TN_MUTED.2)
    );
    println!("{}", divider());

    if items.is_empty() {
        println!(
            "  {}",
            "暂无记录。查询结果会自动保存在这里。".truecolor(TN_MUTED.0, TN_MUTED.1, TN_MUTED.2)
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
                    icon.yellow(),
                    query.truecolor(TN_FG.0, TN_FG.1, TN_FG.2),
                    query_padding,
                    line.truecolor(TN_TRANSLATION.0, TN_TRANSLATION.1, TN_TRANSLATION.2),
                    item.created_at
                        .truecolor(TN_MUTED.0, TN_MUTED.1, TN_MUTED.2)
                );
            } else {
                println!(
                    "  {} {}",
                    " ".repeat(query_width + 5),
                    line.truecolor(TN_TRANSLATION.0, TN_TRANSLATION.1, TN_TRANSLATION.2)
                );
            }
        }
    }
    println!();
}

fn render_word_card(detail: &WordDetail) {
    println!();

    let badge = format!("  {}  ", detail.word)
        .bold()
        .truecolor(TN_BLUE.0, TN_BLUE.1, TN_BLUE.2)
        .on_truecolor(TN_SELECTION.0, TN_SELECTION.1, TN_SELECTION.2);
    println!("  {}", badge);

    let mut phonetics = Vec::new();
    if let Some(ref us) = detail.phonetic_us {
        phonetics.push(format_phonetic("美", us));
    }
    if let Some(ref uk) = detail.phonetic_uk {
        phonetics.push(format_phonetic("英", uk));
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
            println!("  {}", line.truecolor(TN_MUTED.0, TN_MUTED.1, TN_MUTED.2));
        }
    }

    println!("{}", divider());

    if !detail.definitions.is_empty() {
        println!("{}", format_section_title("词典释义", TN_GREEN));
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
                    pos_text
                        .truecolor(TN_ORANGE.0, TN_ORANGE.1, TN_ORANGE.2)
                        .bold(),
                    line.truecolor(TN_FG.0, TN_FG.1, TN_FG.2)
                );
            }
        }
    }

    if !detail.examples.is_empty() {
        if !detail.definitions.is_empty() {
            println!();
        }
        println!("{}", format_section_title("双语例句", TN_MAGENTA));

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
                    line.truecolor(TN_BLUE.0, TN_BLUE.1, TN_BLUE.2)
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
                    line.truecolor(TN_TRANSLATION.0, TN_TRANSLATION.1, TN_TRANSLATION.2)
                );
            }
        }
    }

    println!("{}", divider());
    println!();
}

fn render_sentence_card(original: &str, translated: &str, detected_lang: &str, target_lang: &str) {
    println!();
    let badge = format!(
        "  {} -> {} · Google 翻译  ",
        detected_lang.to_uppercase(),
        target_lang.to_uppercase()
    )
    .bold()
    .truecolor(TN_MAGENTA.0, TN_MAGENTA.1, TN_MAGENTA.2)
    .on_truecolor(TN_SELECTION.0, TN_SELECTION.1, TN_SELECTION.2);
    println!("  {}", badge);
    println!("{}", divider());

    println!("{}", format_section_title("原文", TN_BLUE));
    let text_width = content_width().saturating_sub(2).max(1);
    for line in wrap_display(original, text_width) {
        println!("  {}", line.truecolor(TN_FG.0, TN_FG.1, TN_FG.2));
    }

    println!();
    println!("{}", format_section_title("译文", TN_GREEN));
    for line in wrap_display(translated, text_width) {
        println!("  {}", line.truecolor(TN_GREEN.0, TN_GREEN.1, TN_GREEN.2));
    }

    println!("{}", divider());
    println!();
}

fn format_section_title(label: &str, color: (u8, u8, u8)) -> String {
    format!("  {}  ", label)
        .bold()
        .truecolor(color.0, color.1, color.2)
        .on_truecolor(TN_SELECTION.0, TN_SELECTION.1, TN_SELECTION.2)
        .to_string()
}

fn format_phonetic(label: &str, raw: &str) -> String {
    let clean = raw.trim_matches(|c| c == '/' || c == '[' || c == ']' || c == ' ');
    format!("{} [{}]", label, clean)
        .truecolor(TN_CYAN.0, TN_CYAN.1, TN_CYAN.2)
        .bold()
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
    terminal_width().saturating_sub(4).clamp(1, 100)
}

fn content_width() -> usize {
    card_width().saturating_sub(4).max(1)
}

fn divider() -> String {
    format!(
        "  {}",
        "─"
            .repeat(card_width())
            .truecolor(TN_BORDER.0, TN_BORDER.1, TN_BORDER.2)
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
