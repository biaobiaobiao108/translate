use crate::api::microsoft::translate_text;
use crate::error::{AppError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub orig: String,
    pub trans: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionGroup {
    pub pos: String,
    pub meanings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordDetail {
    pub word: String,
    pub phonetic_us: Option<String>,
    pub phonetic_uk: Option<String>,
    pub tags: Vec<String>,
    pub definitions: Vec<DefinitionGroup>,
    pub examples: Vec<Example>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryOutput {
    Dict(WordDetail),
    Sentence {
        original: String,
        translated: String,
        detected_lang: String,
        target_lang: String,
    },
}

#[derive(Debug, Deserialize)]
struct YoudaoResponse {
    ec: Option<YoudaoEc>,
    #[serde(rename = "blng_sents_part")]
    bilingual_sentences: Option<BilingualSentences>,
}

#[derive(Debug, Deserialize)]
struct YoudaoEc {
    exam_type: Option<Vec<String>>,
    word: Option<Vec<YoudaoWord>>,
}

#[derive(Debug, Deserialize)]
struct YoudaoWord {
    usphone: Option<String>,
    ukphone: Option<String>,
    trs: Option<Vec<YoudaoTranslation>>,
}

#[derive(Debug, Deserialize)]
struct YoudaoTranslation {
    tr: Option<Vec<YoudaoTranslationPart>>,
}

#[derive(Debug, Deserialize)]
struct YoudaoTranslationPart {
    l: Option<YoudaoLanguagePart>,
}

#[derive(Debug, Deserialize)]
struct YoudaoLanguagePart {
    i: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct BilingualSentences {
    #[serde(rename = "sentence-pair")]
    sentence_pair: Option<Vec<BilingualSentence>>,
}

#[derive(Debug, Deserialize)]
struct BilingualSentence {
    sentence: Option<String>,
    #[serde(rename = "sentence-translation")]
    translation: Option<String>,
}

pub async fn fetch_dict_detail(client: &Client, word: &str) -> Result<WordDetail> {
    let url = "https://dict.youdao.com/jsonapi";
    let resp = client
        .get(url)
        .query(&[
            ("q", word),
            (
                "dicts",
                "{\"count\":2,\"dicts\":[[\"ec\"],[\"blng_sents_part\"]]}",
            ),
        ])
        .send()
        .await?
        .error_for_status()?;

    let json: YoudaoResponse = resp.json().await?;

    parse_youdao_response(json, word)
}

fn parse_youdao_response(json: YoudaoResponse, word: &str) -> Result<WordDetail> {
    let mut us_phone = None;
    let mut uk_phone = None;
    let mut tags = Vec::new();
    let mut definitions = Vec::new();
    let mut examples = Vec::new();

    // 提取 ec (英汉释义和音标)
    if let Some(ec) = json.ec {
        if let Some(exam_types) = ec.exam_type {
            tags.extend(exam_types);
        }

        if let Some(word) = ec.word.and_then(|words| words.into_iter().next()) {
            if let Some(us) = word.usphone.filter(|phone| !phone.is_empty()) {
                us_phone = Some(format!("/ {} /", us));
            }
            if let Some(uk) = word.ukphone.filter(|phone| !phone.is_empty()) {
                uk_phone = Some(format!("/ {} /", uk));
            }

            for translation in word.trs.unwrap_or_default() {
                for part in translation.tr.unwrap_or_default() {
                    for line in part.l.and_then(|language| language.i).unwrap_or_default() {
                        let trimmed = line.trim();
                        if let Some(dot_idx) = trimmed.find('.') {
                            let pos = &trimmed[..=dot_idx];
                            let meaning = trimmed[dot_idx + 1..].trim();
                            definitions.push(DefinitionGroup {
                                pos: pos.to_string(),
                                meanings: vec![meaning.to_string()],
                            });
                        } else {
                            definitions.push(DefinitionGroup {
                                pos: String::new(),
                                meanings: vec![trimmed.to_string()],
                            });
                        }
                    }
                }
            }
        }
    }

    // 提取双语例句
    if let Some(sentences) = json.bilingual_sentences {
        for pair in sentences
            .sentence_pair
            .unwrap_or_default()
            .into_iter()
            .take(3)
        {
            if let (Some(orig), Some(trans)) = (pair.sentence, pair.translation) {
                if !orig.is_empty() && !trans.is_empty() {
                    examples.push(Example {
                        orig: orig.to_string(),
                        trans: trans.to_string(),
                    });
                }
            }
        }
    }

    if definitions.is_empty() {
        return Err(AppError::NotFound(word.to_string()));
    }

    Ok(WordDetail {
        word: word.to_string(),
        phonetic_us: us_phone,
        phonetic_uk: uk_phone,
        tags,
        definitions,
        examples,
    })
}

fn is_dictionary_query(trimmed: &str, force_sentence: bool) -> bool {
    !force_sentence
        && trimmed.chars().count() <= 64
        && !trimmed.chars().any(|c| c.is_whitespace())
        && trimmed
            .chars()
            .all(|c| c.is_alphabetic() || matches!(c, '-' | '\'' | '’'))
}

pub async fn smart_query(
    client: &Client,
    query: &str,
    force_sentence: bool,
) -> Result<QueryOutput> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput("查询内容不能为空".to_string()));
    }

    if is_dictionary_query(trimmed, force_sentence) {
        if let Ok(detail) = fetch_dict_detail(client, trimmed).await {
            return Ok(QueryOutput::Dict(detail));
        }
    }

    // 句子翻译或未命中词典，调用微软翻译
    let trans = translate_text(client, trimmed, None, None).await?;
    Ok(QueryOutput::Sentence {
        original: trans.original,
        translated: trans.translated,
        detected_lang: trans.detected_lang,
        target_lang: trans.target_lang,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse(value: serde_json::Value) -> Result<WordDetail> {
        let response = serde_json::from_value::<YoudaoResponse>(value).unwrap();
        parse_youdao_response(response, "rust")
    }

    #[test]
    fn parses_dictionary_card_data() {
        let detail = parse(json!({
            "ec": {
                "exam_type": ["CET6"],
                "word": [{
                    "usphone": "rʌst",
                    "ukphone": "rʌst",
                    "trs": [{
                        "tr": [{
                            "l": { "i": ["n. 锈", "v. 生锈"] }
                        }]
                    }]
                }]
            },
            "blng_sents_part": {
                "sentence-pair": [{
                    "sentence": "The pipe is covered with rust.",
                    "sentence-translation": "管子上覆盖着锈。"
                }]
            }
        }))
        .unwrap();

        assert_eq!(detail.phonetic_us.as_deref(), Some("/ rʌst /"));
        assert_eq!(detail.tags, vec!["CET6"]);
        assert_eq!(detail.definitions.len(), 2);
        assert_eq!(detail.definitions[0].pos, "n.");
        assert_eq!(detail.definitions[0].meanings, vec!["锈"]);
        assert_eq!(detail.examples[0].trans, "管子上覆盖着锈。");
    }

    #[test]
    fn reports_missing_dictionary_entry() {
        let error = parse(json!({ "ec": { "word": [] } })).unwrap_err();
        assert!(matches!(error, AppError::NotFound(word) if word == "rust"));
    }

    #[test]
    fn classifies_dictionary_queries() {
        assert!(is_dictionary_query("rust", false));
        assert!(is_dictionary_query("don't", false));
        assert!(!is_dictionary_query("hello world", false));
        assert!(!is_dictionary_query("rust", true));
    }
}
