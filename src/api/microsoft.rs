use std::sync::RwLock;
use std::time::{Duration, Instant};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AppError, Result};

const MAX_QUERY_CHARS: usize = 10_000;
const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    pub original: String,
    pub translated: String,
    pub detected_lang: String,
    pub target_lang: String,
}

#[derive(Clone)]
struct BingAuth {
    ig: String,
    iid: String,
    key: String,
    token: String,
    cached_at: Instant,
}

static BING_AUTH: RwLock<Option<BingAuth>> = RwLock::new(None);

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
        ("auto-detect", "en")
    } else {
        ("auto-detect", "zh-Hans")
    }
}

fn normalize_lang_code(code: &str) -> &str {
    if code.eq_ignore_ascii_case("auto") || code.eq_ignore_ascii_case("auto-detect") {
        "auto-detect"
    } else if code.eq_ignore_ascii_case("zh")
        || code.eq_ignore_ascii_case("zh-cn")
        || code.eq_ignore_ascii_case("zh-hans")
    {
        "zh-Hans"
    } else if code.eq_ignore_ascii_case("zh-tw")
        || code.eq_ignore_ascii_case("zh-hk")
        || code.eq_ignore_ascii_case("zh-hant")
    {
        "zh-Hant"
    } else {
        code
    }
}

async fn fetch_bing_auth(client: &Client) -> Result<BingAuth> {
    let endpoints = ["https://cn.bing.com/translator", "https://www.bing.com/translator"];
    let mut last_error = None;

    for url in endpoints {
        let resp = match client
            .get(url)
            .header("User-Agent", USER_AGENT)
            .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                last_error = Some(e);
                continue;
            }
        };

        if let Ok(html) = resp.text().await {
            if let Some(auth) = parse_bing_html_auth(&html) {
                return Ok(auth);
            }
        }
    }

    if let Some(err) = last_error {
        Err(AppError::Network(err))
    } else {
        Err(AppError::Protocol("无法获取微软翻译验证凭证".to_string()))
    }
}

fn parse_bing_html_auth(html: &str) -> Option<BingAuth> {
    // 1. Extract IG (supports IG:"...", IG="...", or _IG="...")
    let ig = if let Some(idx) = html.find("IG:\"") {
        let start = idx + 4;
        let end = html[start..].find('"')? + start;
        html[start..end].to_string()
    } else if let Some(idx) = html.find("IG=\"") {
        let start = idx + 4;
        let end = html[start..].find('"')? + start;
        html[start..end].to_string()
    } else if let Some(idx) = html.find("_IG=\"") {
        let start = idx + 5;
        let end = html[start..].find('"')? + start;
        html[start..end].to_string()
    } else {
        return None;
    };

    // 2. Extract IID (fallback to translator.5023 if not found)
    let iid = if let Some(iid_idx) = html.find("data-iid=\"") {
        let iid_start = iid_idx + 10;
        if let Some(iid_end_rel) = html[iid_start..].find('"') {
            html[iid_start..iid_start + iid_end_rel].to_string()
        } else {
            "translator.5023".to_string()
        }
    } else {
        "translator.5023".to_string()
    };

    // 3. Extract params_AbusePreventionHelper = [key, "token", ttl]
    let abuse_idx = html.find("params_AbusePreventionHelper")?;
    let bracket_start = abuse_idx + html[abuse_idx..].find('[')? + 1;
    let bracket_end = bracket_start + html[bracket_start..].find(']')?;
    let params_str = &html[bracket_start..bracket_end];

    let parts: Vec<&str> = params_str.split(',').collect();
    if parts.len() < 2 {
        return None;
    }

    let key = parts[0].trim().to_string();
    let token = parts[1].trim().trim_matches('"').to_string();

    if key.is_empty() || token.is_empty() || ig.is_empty() {
        return None;
    }

    Some(BingAuth {
        ig,
        iid,
        key,
        token,
        cached_at: Instant::now(),
    })
}

async fn get_or_refresh_auth(client: &Client, force_refresh: bool) -> Result<BingAuth> {
    if !force_refresh {
        if let Ok(guard) = BING_AUTH.read() {
            if let Some(ref auth) = *guard {
                if auth.cached_at.elapsed() < Duration::from_secs(1800) {
                    return Ok(auth.clone());
                }
            }
        }
    }

    let fresh = fetch_bing_auth(client).await?;
    if let Ok(mut guard) = BING_AUTH.write() {
        *guard = Some(fresh.clone());
    }
    Ok(fresh)
}

fn parse_bing_response(value: &Value) -> Result<(String, String)> {
    if let Some(arr) = value.as_array() {
        if let Some(first) = arr.first() {
            let detected_lang = first
                .get("detectedLanguage")
                .and_then(|d| d.get("language"))
                .and_then(Value::as_str)
                .unwrap_or("auto")
                .to_string();

            let translated = first
                .get("translations")
                .and_then(Value::as_array)
                .and_then(|t| t.first())
                .and_then(|item| item.get("text"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();

            if !translated.trim().is_empty() {
                return Ok((translated.trim().to_string(), detected_lang));
            }
        }
    }

    if let Some(error_msg) = value.get("errorMessage").and_then(Value::as_str) {
        if !error_msg.is_empty() {
            return Err(AppError::Protocol(format!("微软翻译服务错误: {}", error_msg)));
        }
    }

    Err(AppError::Protocol("未能解析到有效微软翻译结果".to_string()))
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
    let sl = normalize_lang_code(sl.unwrap_or(default_sl));
    let tl = normalize_lang_code(tl.unwrap_or(default_tl));

    // Try first with cached auth, and retry once with fresh auth if 400 occurs
    let mut retry_done = false;
    loop {
        let auth = get_or_refresh_auth(client, retry_done).await?;
        let url = format!(
            "https://cn.bing.com/ttranslatev3?isVertical=1&IG={}&IID={}",
            auth.ig, auth.iid
        );

        let form_params = [
            ("fromLang", sl),
            ("to", tl),
            ("text", trimmed),
            ("key", auth.key.as_str()),
            ("token", auth.token.as_str()),
        ];

        let response = client
            .post(&url)
            .header("User-Agent", USER_AGENT)
            .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
            .header("Referer", "https://cn.bing.com/translator")
            .form(&form_params)
            .send()
            .await?;

        let status = response.status();
        let body_text = response.text().await.unwrap_or_default();

        if status.is_success() {
            if let Ok(json_value) = serde_json::from_str::<Value>(&body_text) {
                // If Bing returned error code (e.g. 400 token expired)
                if json_value.get("statusCode").and_then(Value::as_i64) == Some(400) {
                    if !retry_done {
                        retry_done = true;
                        continue;
                    }
                }

                let (translated, detected_lang) = parse_bing_response(&json_value)?;
                return Ok(TranslationResult {
                    original: trimmed.to_string(),
                    translated,
                    detected_lang,
                    target_lang: tl.to_string(),
                });
            }
        }

        if !retry_done {
            retry_done = true;
            continue;
        }

        return Err(AppError::Protocol(format!(
            "微软翻译请求失败 (状态码: {}): {}",
            status, body_text
        )));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_bing_response_success() {
        let value = json!([
            {
                "detectedLanguage": { "language": "en", "score": 1.0 },
                "translations": [
                    { "text": "你好，世界！", "to": "zh-Hans" }
                ],
                "usedLLM": true
            }
        ]);
        let (translated, detected_lang) = parse_bing_response(&value).unwrap();
        assert_eq!(translated, "你好，世界！");
        assert_eq!(detected_lang, "en");
    }

    #[test]
    fn detects_cjk_target_language() {
        assert_eq!(detect_target_lang("你好"), ("auto-detect", "en"));
        assert_eq!(detect_target_lang("hello"), ("auto-detect", "zh-Hans"));
    }

    #[test]
    fn parses_bing_auth_from_html() {
        let sample_html = r#"
            <script>
            var IG="1A942AFD4BE84E8D9E1256A02397720D";
            </script>
            <div data-iid="translator.5023"></div>
            <script>
            var params_AbusePreventionHelper = [1790167341344,"test_token_abc_123",3600000];
            </script>
        "#;
        let auth = parse_bing_html_auth(sample_html).unwrap();
        assert_eq!(auth.ig, "1A942AFD4BE84E8D9E1256A02397720D");
        assert_eq!(auth.iid, "translator.5023");
        assert_eq!(auth.key, "1790167341344");
        assert_eq!(auth.token, "test_token_abc_123");
    }
}
