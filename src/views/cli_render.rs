use colored::*;
use crate::api::dict::{QueryOutput, WordDetail};

// Tokyo Night TrueColor RGB Constants
const TN_FG: (u8, u8, u8) = (192, 202, 245);         // #c0caf5 Main text
const TN_TRANSLATION: (u8, u8, u8) = (169, 177, 214); // #a9b1d6 Readable secondary text / Example translation
const TN_BLUE: (u8, u8, u8) = (122, 162, 247);       // #7aa2f7 Accent / Header text / English example
const TN_CYAN: (u8, u8, u8) = (125, 207, 255);       // #7dcfff Phonetics / Numbers
const TN_GREEN: (u8, u8, u8) = (158, 206, 106);      // #9ece6a Definitions header / Translated text
const TN_MAGENTA: (u8, u8, u8) = (187, 154, 247);    // #bb9af7 Examples header / Badge text
const TN_ORANGE: (u8, u8, u8) = (255, 158, 100);     // #ff9e64 POS tags (n., v.)
const TN_SELECTION: (u8, u8, u8) = (40, 52, 73);     // #283449 Badge background
const TN_BORDER: (u8, u8, u8) = (65, 72, 104);       // #414868 Divider line

fn format_section_title(label: &str, color: (u8, u8, u8)) -> String {
    format!(" {} ", label)
        .bold()
        .truecolor(color.0, color.1, color.2)
        .on_truecolor(TN_SELECTION.0, TN_SELECTION.1, TN_SELECTION.2)
        .to_string()
}

pub fn render_cli_output(output: &QueryOutput) {
    match output {
        QueryOutput::Dict(detail) => render_word_card(detail),
        QueryOutput::Sentence { original, translated, detected_lang, target_lang } => {
            render_sentence_card(original, translated, detected_lang, target_lang);
        }
    }
}

fn format_phonetic(label: &str, raw: &str) -> String {
    let clean = raw.trim_matches(|c| c == '/' || c == '[' || c == ']' || c == ' ');
    format!("{} [{}]", label, clean)
        .truecolor(TN_CYAN.0, TN_CYAN.1, TN_CYAN.2)
        .bold()
        .to_string()
}

fn render_word_card(detail: &WordDetail) {
    println!();
    // 词条深色胶囊徽章 (Tokyo Night Blue + Bold + Background #283449)
    let badge = format!("  {}  ", detail.word)
        .bold()
        .truecolor(TN_BLUE.0, TN_BLUE.1, TN_BLUE.2)
        .on_truecolor(TN_SELECTION.0, TN_SELECTION.1, TN_SELECTION.2);
    print!(" {}", badge);

    // 音标部分 (Tokyo Night Cyan，同行紧随徽章)
    let mut phonetics = Vec::new();
    if let Some(ref us) = detail.phonetic_us {
        phonetics.push(format_phonetic("美", us));
    }
    if let Some(ref uk) = detail.phonetic_uk {
        phonetics.push(format_phonetic("英", uk));
    }
    if !phonetics.is_empty() {
        print!("   {}", phonetics.join("   "));
    }
    println!();

    // 分割线 (#414868)
    let divider = "─".repeat(58).truecolor(TN_BORDER.0, TN_BORDER.1, TN_BORDER.2);
    println!("{}", divider);

    // 词性与释义
    if !detail.definitions.is_empty() {
        println!("{}", format_section_title("【词典释义】", TN_GREEN));
        for def in &detail.definitions {
            let pos_tag = if !def.pos.is_empty() {
                let formatted_pos = if def.pos.ends_with('.') {
                    def.pos.clone()
                } else {
                    format!("{}.", def.pos)
                };
                format!("{:>6}", formatted_pos)
                    .bold()
                    .truecolor(TN_ORANGE.0, TN_ORANGE.1, TN_ORANGE.2)
            } else {
                "      ".normal()
            };
            let meanings_str = def.meanings.join("；");
            println!(
                "  {}  {}",
                pos_tag,
                meanings_str.truecolor(TN_FG.0, TN_FG.1, TN_FG.2)
            );
        }
    }

    // 双语例句
    if !detail.examples.is_empty() {
        if !detail.definitions.is_empty() {
            println!();
        }
        println!("{}", format_section_title("【双语例句】", TN_MAGENTA));
        for (i, eg) in detail.examples.iter().enumerate() {
            let num = format!("{}.", i + 1);
            println!(
                "  {} {}",
                num.bold().truecolor(TN_BLUE.0, TN_BLUE.1, TN_BLUE.2),
                eg.orig.truecolor(TN_BLUE.0, TN_BLUE.1, TN_BLUE.2)
            );
            println!(
                "     {}",
                eg.trans.truecolor(TN_TRANSLATION.0, TN_TRANSLATION.1, TN_TRANSLATION.2)
            );
        }
    }

    println!("{}", divider);
    println!();
}

fn render_sentence_card(original: &str, translated: &str, detected_lang: &str, target_lang: &str) {
    println!();
    // 胶囊徽章：[EN -> ZH] Google 翻译，背景色 #283449，文字加粗
    let badge = format!("  [{} -> {}] Google 翻译  ", detected_lang.to_uppercase(), target_lang.to_uppercase())
        .bold()
        .truecolor(TN_MAGENTA.0, TN_MAGENTA.1, TN_MAGENTA.2)
        .on_truecolor(TN_SELECTION.0, TN_SELECTION.1, TN_SELECTION.2);
    println!(" {}", badge);

    let divider = "─".repeat(58).truecolor(TN_BORDER.0, TN_BORDER.1, TN_BORDER.2);
    println!("{}", divider);

    for line in original.lines() {
        println!("  {}", line.truecolor(TN_FG.0, TN_FG.1, TN_FG.2));
    }
    println!();

    for line in translated.lines() {
        println!(
            "  {}",
            line.bold().truecolor(TN_GREEN.0, TN_GREEN.1, TN_GREEN.2)
        );
    }

    println!("{}", divider);
    println!();
}
