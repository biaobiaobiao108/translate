use colored::*;
use crate::api::dict::{QueryOutput, WordDetail};

// Tokyo Night TrueColor RGB Constants
const TN_BG_BADGE: (u8, u8, u8) = (40, 52, 73);      // Selection / badge background
const TN_FG: (u8, u8, u8) = (192, 202, 245);         // Main text
const TN_BLUE: (u8, u8, u8) = (122, 162, 247);       // Primary accent / Header
const TN_CYAN: (u8, u8, u8) = (125, 207, 255);       // Secondary accent / Phonetics
const TN_GREEN: (u8, u8, u8) = (158, 206, 106);      // Definitions section header
const TN_MAGENTA: (u8, u8, u8) = (187, 154, 247);    // Examples header
const TN_ORANGE: (u8, u8, u8) = (255, 158, 100);     // POS tags (n., v.)
const TN_COMMENT: (u8, u8, u8) = (86, 95, 137);      // Dividers / Subdued text
const TN_BORDER: (u8, u8, u8) = (65, 72, 104);       // Box border

pub fn render_cli_output(output: &QueryOutput) {
    match output {
        QueryOutput::Dict(detail) => render_word_card(detail),
        QueryOutput::Sentence { original, translated, detected_lang, target_lang } => {
            render_sentence_card(original, translated, detected_lang, target_lang);
        }
    }
}

fn render_word_card(detail: &WordDetail) {
    println!();
    // 词条大标题 (Tokyo Night Blue + Bold)
    let title = format!("  {}  ", detail.word)
        .bold()
        .truecolor(TN_BLUE.0, TN_BLUE.1, TN_BLUE.2)
        .on_truecolor(TN_BG_BADGE.0, TN_BG_BADGE.1, TN_BG_BADGE.2);
    print!("{}", title);

    // 音标部分 (Tokyo Night Cyan)
    let mut phonetics = Vec::new();
    if let Some(ref us) = detail.phonetic_us {
        phonetics.push(format!("美 {}", us).truecolor(TN_CYAN.0, TN_CYAN.1, TN_CYAN.2).to_string());
    }
    if let Some(ref uk) = detail.phonetic_uk {
        phonetics.push(format!("英 {}", uk).truecolor(TN_CYAN.0, TN_CYAN.1, TN_CYAN.2).to_string());
    }
    if !phonetics.is_empty() {
        print!("   {}", phonetics.join("   "));
    }
    println!("\n");

    // 渐变风格分割线
    let divider = "─".repeat(58).truecolor(TN_BORDER.0, TN_BORDER.1, TN_BORDER.2);
    println!("{}", divider);

    // 词性与释义
    if !detail.definitions.is_empty() {
        println!(
            "{}",
            " 【词典释义】"
                .bold()
                .truecolor(TN_GREEN.0, TN_GREEN.1, TN_GREEN.2)
        );
        for def in &detail.definitions {
            let pos_tag = if !def.pos.is_empty() {
                format!("{:>6}", def.pos)
                    .bold()
                    .truecolor(TN_ORANGE.0, TN_ORANGE.1, TN_ORANGE.2)
            } else {
                "      ".normal()
            };
            let meanings_str = def.meanings.join("； ");
            println!(
                "  {}  {}",
                pos_tag,
                meanings_str.truecolor(TN_FG.0, TN_FG.1, TN_FG.2)
            );
        }
        println!();
    }

    // 双语例句
    if !detail.examples.is_empty() {
        println!(
            "{}",
            " 【双语例句】"
                .bold()
                .truecolor(TN_MAGENTA.0, TN_MAGENTA.1, TN_MAGENTA.2)
        );
        for (i, eg) in detail.examples.iter().enumerate() {
            println!(
                "  {}. {}",
                (i + 1)
                    .to_string()
                    .bold()
                    .truecolor(TN_CYAN.0, TN_CYAN.1, TN_CYAN.2),
                eg.orig.truecolor(TN_FG.0, TN_FG.1, TN_FG.2)
            );
            println!(
                "     {}",
                eg.trans.truecolor(TN_COMMENT.0, TN_COMMENT.1, TN_COMMENT.2)
            );
        }
        println!();
    }

    println!("{}", divider);
    println!();
}

fn render_sentence_card(original: &str, translated: &str, detected_lang: &str, target_lang: &str) {
    println!();
    let lang_badge = format!(" [{} -> {}] ", detected_lang.to_uppercase(), target_lang.to_uppercase())
        .bold()
        .truecolor(TN_BLUE.0, TN_BLUE.1, TN_BLUE.2)
        .on_truecolor(TN_BG_BADGE.0, TN_BG_BADGE.1, TN_BG_BADGE.2);
    let engine_label = "Google 翻译".bold().truecolor(TN_CYAN.0, TN_CYAN.1, TN_CYAN.2);
    println!("{} {}", lang_badge, engine_label);

    let divider = "─".repeat(58).truecolor(TN_BORDER.0, TN_BORDER.1, TN_BORDER.2);
    println!("{}", divider);

    println!(
        "{}",
        " 原文:".truecolor(TN_COMMENT.0, TN_COMMENT.1, TN_COMMENT.2)
    );
    for line in original.lines() {
        println!("   {}", line.truecolor(TN_FG.0, TN_FG.1, TN_FG.2));
    }
    println!();

    println!(
        "{}",
        " 译文:".bold().truecolor(TN_GREEN.0, TN_GREEN.1, TN_GREEN.2)
    );
    for line in translated.lines() {
        println!(
            "   {}",
            line.bold().truecolor(TN_GREEN.0, TN_GREEN.1, TN_GREEN.2)
        );
    }

    println!("{}", divider);
    println!();
}
