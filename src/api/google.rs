use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AppError, Result};

const MAX_QUERY_CHARS: usize = 12_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    pub original: String,
    pub translated: String,
    pub detected_lang: String,
    pub target_lang: String,
}

pub fn detect_target_lang(text: &str) -> (&'static str, &'static str) {
    let has_cjk = text.chars().any(|c| {
        matches!(
            c,
            '\u{3400}'..='\u{4dbf}'
                | '\u{4e00}'..='\u{9fff}'
                | '\u{3040}'..='\u{30ff}'
                | '\u{ac00}'..='\u{d7af}'
        )
    });

    if has_cjk {
        ("auto", "en")
    } else {
        ("auto", "zh-CN")
    }
}

fn is_language_code(value: &str) -> bool {
    let value = value.trim();
    (value.len() == 2 && value.chars().all(|c| c.is_ascii_alphabetic()))
        || (value.len() == 5
            && value.as_bytes()[2] == b'-'
            && value[..2].chars().all(|c| c.is_ascii_alphabetic())
            && value[3..].chars().all(|c| c.is_ascii_alphabetic()))
}

fn parse_google_response(value: &Value) -> Result<(String, String)> {
    let mut translated = String::new();
    let mut detected_lang = None;

    if let Some(root) = value.as_array() {
        if let Some(segments) = root.first().and_then(Value::as_array) {
            if segments.first().and_then(Value::as_array).is_some() {
                for segment in segments {
                    if let Some(parts) = segment.as_array() {
                        if let Some(text) = parts.first().and_then(Value::as_str) {
                            translated.push_str(text);
                        }
                    }
                }
            } else if let Some(text) = segments.first().and_then(Value::as_str) {
                // Some responses use the compact shape [["translated", "source"]].
                translated.push_str(text);
                detected_lang = segments
                    .get(1)
                    .and_then(Value::as_str)
                    .filter(|value| is_language_code(value))
                    .map(str::to_owned);
            }
        }

        if detected_lang.is_none() {
            detected_lang = root
                .get(2)
                .and_then(Value::as_str)
                .filter(|value| is_language_code(value))
                .map(str::to_owned);
        }
    }

    if translated.is_empty() {
        translated = value
            .get("translatedText")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
    }

    if detected_lang.is_none() {
        detected_lang = value
            .get("src")
            .and_then(Value::as_str)
            .filter(|value| is_language_code(value))
            .map(str::to_owned);
    }

    if translated.trim().is_empty() {
        return Err(AppError::Protocol("未能解析到有效翻译结果".to_string()));
    }

    Ok((
        translated.trim().to_string(),
        detected_lang.unwrap_or_else(|| "auto".to_string()),
    ))
}

pub async fn translate_text(
    client: &Client,
    text: &str,
    sl: Option<&str>,
    tl: Option<&str>,
) -> Result<TranslationResult> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput("待翻译文本不能为空".to_string()));
    }
    if trimmed.chars().count() > MAX_QUERY_CHARS {
        return Err(AppError::InvalidInput(format!(
            "待翻译文本过长，最多支持 {} 个字符",
            MAX_QUERY_CHARS
        )));
    }

    let (default_sl, default_tl) = detect_target_lang(trimmed);
    let sl = sl.unwrap_or(default_sl);
    let tl = tl.unwrap_or(default_tl);

    let response = client
        .get("https://clients5.google.com/translate_a/t")
        .query(&[
            ("client", "dict-chrome-ex"),
            ("sl", sl),
            ("tl", tl),
            ("q", trimmed),
        ])
        .send()
        .await?
        .error_for_status()?;

    let json_value: Value = response.json().await?;
    let (translated, detected_lang) = parse_google_response(&json_value)?;

    Ok(TranslationResult {
        original: trimmed.to_string(),
        translated,
        detected_lang,
        target_lang: tl.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_segmented_google_response() {
        let value = json!([
            [["你好", "hello"], ["，", ","], ["世界", "world"]],
            null,
            "en"
        ]);
        let parsed = parse_google_response(&value).unwrap();

        assert_eq!(parsed.0, "你好，世界");
        assert_eq!(parsed.1, "en");
    }

    #[test]
    fn parses_compact_google_response() {
        let value = json!([["你好", "en"]]);
        let parsed = parse_google_response(&value).unwrap();

        assert_eq!(parsed.0, "你好");
        assert_eq!(parsed.1, "en");
    }

    #[test]
    fn detects_cjk_target_language() {
        assert_eq!(detect_target_lang("你好"), ("auto", "en"));
        assert_eq!(detect_target_lang("hello"), ("auto", "zh-CN"));
    }
}
