use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AppError, Result};
use crate::text::{sanitize_terminal_text, truncate_chars};

const MAX_QUERY_CHARS: usize = 10_000;
const MAX_ERROR_BODY_CHARS: usize = 300;
const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    pub original: String,
    pub translated: String,
    pub detected_lang: String,
    pub target_lang: String,
}

struct BingAuth {
    ig: String,
    iid: String,
    key: String,
    token: String,
    origin: &'static str,
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
    let origins = ["https://cn.bing.com", "https://www.bing.com"];
    let mut last_error = None;

    for origin in origins {
        let url = format!("{origin}/translator");
        let resp = match client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                last_error = Some(AppError::Network(e));
                continue;
            }
        };

        let status = resp.status();
        let html = match resp.text().await {
            Ok(html) => html,
            Err(error) => {
                last_error = Some(AppError::Network(error));
                continue;
            }
        };

        if !status.is_success() {
            last_error = Some(AppError::Protocol(format!(
                "微软翻译认证页返回状态码: {status}"
            )));
            continue;
        }

        if let Some(auth) = parse_bing_html_auth(&html, origin) {
            return Ok(auth);
        }

        last_error = Some(AppError::Protocol(format!(
            "无法从 {url} 解析微软翻译验证凭证"
        )));
    }

    Err(last_error.unwrap_or_else(|| AppError::Protocol("无法获取微软翻译验证凭证".to_string())))
}

fn parse_bing_html_auth(html: &str, origin: &'static str) -> Option<BingAuth> {
    // 1. Extract IG (supports IG:"...", IG="...", or _IG="...")
    let ig = if let Some(idx) = html.find("IG:\"") {
        let start = idx + 4;
        let end = html[start..].find('"')? + start;
        html[start..end].to_string()
    } else if let Some(idx) = html.find("IG=\"") {
        let start = idx + 4;
        let end = html[start..].find('"')? + start;
        html[start..end].to_string()
    } else {
        let idx = html.find("_IG=\"")?;
        let start = idx + 5;
        let end = html[start..].find('"')? + start;
        html[start..end].to_string()
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
        origin,
    })
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
            let safe_message =
                truncate_chars(&sanitize_terminal_text(error_msg), MAX_ERROR_BODY_CHARS);
            return Err(AppError::Protocol(format!(
                "微软翻译服务错误: {}",
                safe_message
            )));
        }
    }

    Err(AppError::Protocol("未能解析到有效微软翻译结果".to_string()))
}

fn is_auth_failure(status: StatusCode, value: Option<&Value>) -> bool {
    if matches!(status.as_u16(), 400 | 401 | 403) {
        return true;
    }

    value
        .and_then(|json| json.get("statusCode"))
        .and_then(Value::as_i64)
        .is_some_and(|code| matches!(code, 400 | 401 | 403))
}

fn request_error(status: StatusCode, body: &str) -> String {
    let safe_body = sanitize_terminal_text(body);
    let excerpt = truncate_chars(safe_body.trim(), MAX_ERROR_BODY_CHARS);
    if excerpt.is_empty() {
        format!("微软翻译请求失败 (状态码: {status})")
    } else {
        format!("微软翻译请求失败 (状态码: {status}): {excerpt}")
    }
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

    // Bing 的页面 token 会过期；认证类失败时重新获取一次凭证再试。
    let mut retry_done = false;
    loop {
        let auth = fetch_bing_auth(client).await?;
        let url = format!(
            "{}/ttranslatev3?isVertical=1&IG={}&IID={}",
            auth.origin, auth.ig, auth.iid
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
            .header("Referer", format!("{}/translator", auth.origin))
            .form(&form_params)
            .send()
            .await?;

        let status = response.status();
        let body_text = response.text().await?;
        let json_value = serde_json::from_str::<Value>(&body_text).ok();

        if !retry_done && is_auth_failure(status, json_value.as_ref()) {
            retry_done = true;
            continue;
        }

        if status.is_success() {
            let json_value = json_value.ok_or_else(|| {
                AppError::Protocol(format!("微软翻译返回了无法解析的响应 (状态码: {status})"))
            })?;
            let (translated, detected_lang) = parse_bing_response(&json_value)?;
            return Ok(TranslationResult {
                original: trimmed.to_string(),
                translated,
                detected_lang,
                target_lang: tl.to_string(),
            });
        }

        return Err(AppError::Protocol(request_error(status, &body_text)));
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
    fn sanitizes_bing_error_message() {
        let value = json!({ "errorMessage": "bad\u{1b}[31m request" });
        let error = parse_bing_response(&value).unwrap_err();
        assert!(!error.to_string().contains('\u{1b}'));
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
        let auth = parse_bing_html_auth(sample_html, "https://cn.bing.com").unwrap();
        assert_eq!(auth.ig, "1A942AFD4BE84E8D9E1256A02397720D");
        assert_eq!(auth.iid, "translator.5023");
        assert_eq!(auth.key, "1790167341344");
        assert_eq!(auth.token, "test_token_abc_123");
        assert_eq!(auth.origin, "https://cn.bing.com");
    }

    #[test]
    fn parses_colon_style_auth() {
        let sample_html = r#"
            <script>var IG:"ABC123"</script>
            <script>var params_AbusePreventionHelper = [123, "token", 3600000];</script>
        "#;
        let auth = parse_bing_html_auth(sample_html, "https://www.bing.com").unwrap();
        assert_eq!(auth.ig, "ABC123");
        assert_eq!(auth.origin, "https://www.bing.com");
    }

    #[test]
    fn recognizes_auth_failures() {
        assert!(is_auth_failure(StatusCode::BAD_REQUEST, None));
        assert!(!is_auth_failure(StatusCode::TOO_MANY_REQUESTS, None));
        assert!(is_auth_failure(
            StatusCode::OK,
            Some(&json!({ "statusCode": 403 }))
        ));
    }

    #[test]
    fn request_error_is_sanitized_and_bounded() {
        let message = request_error(
            StatusCode::BAD_GATEWAY,
            &format!("\u{1b}[31m{}", "x".repeat(500)),
        );
        assert!(!message.contains('\u{1b}'));
        assert!(message.chars().count() < 400);
    }
}
