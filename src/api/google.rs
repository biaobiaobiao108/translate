use serde::{Deserialize, Serialize};
use reqwest::Client;
use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    pub original: String,
    pub translated: String,
    pub detected_lang: String,
    pub target_lang: String,
}

pub fn detect_target_lang(text: &str) -> (&'static str, &'static str) {
    let has_chinese = text.chars().any(|c| ('\u{4e00}'..='\u{9fa5}').contains(&c));
    if has_chinese {
        ("auto", "en")
    } else {
        ("auto", "zh-CN")
    }
}

pub async fn translate_text(client: &Client, text: &str, sl: Option<&str>, tl: Option<&str>) -> Result<TranslationResult> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::General("待翻译文本不能为空".to_string()));
    }

    let (default_sl, default_tl) = detect_target_lang(trimmed);
    let sl = sl.unwrap_or(default_sl);
    let tl = tl.unwrap_or(default_tl);

    let url = "https://clients5.google.com/translate_a/t";
    let resp = client
        .get(url)
        .query(&[
            ("client", "dict-chrome-ex"),
            ("sl", sl),
            ("tl", tl),
            ("q", trimmed),
        ])
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(AppError::General(format!(
            "Google 翻译接口请求失败，HTTP 状态码: {}",
            resp.status()
        )));
    }

    let json_val: serde_json::Value = resp.json().await?;
    let mut translated = String::new();
    let mut detected_lang = "auto".to_string();

    // clients5 返回格式: [["你好","en"]] 或 [["生活就像一盒巧克力。","en"]]
    if let Some(arr) = json_val.as_array() {
        if let Some(first_item) = arr.first() {
            if let Some(sub_arr) = first_item.as_array() {
                if let Some(t) = sub_arr.first().and_then(|v| v.as_str()) {
                    translated.push_str(t);
                }
                if let Some(lang) = sub_arr.get(1).and_then(|v| v.as_str()) {
                    detected_lang = lang.to_string();
                }
            } else if let Some(t) = first_item.as_str() {
                translated.push_str(t);
            }
        }
    }

    if detected_lang == "auto" {
        if let Some(src) = json_val.get("src").and_then(|s| s.as_str()) {
            detected_lang = src.to_string();
        }
    }

    if translated.trim().is_empty() {
        return Err(AppError::General("未能解析到有效翻译结果".to_string()));
    }

    Ok(TranslationResult {
        original: trimmed.to_string(),
        translated: translated.trim().to_string(),
        detected_lang,
        target_lang: tl.to_string(),
    })
}
