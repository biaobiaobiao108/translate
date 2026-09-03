use serde::{Deserialize, Serialize};
use reqwest::Client;
use crate::error::{AppError, Result};
use crate::api::google::translate_text;

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

pub async fn fetch_dict_detail(client: &Client, word: &str) -> Result<WordDetail> {
    let url = "https://dict.youdao.com/jsonapi";
    let resp = client
        .get(url)
        .query(&[
            ("q", word),
            ("dicts", "{\"count\":2,\"dicts\":[[\"ec\"],[\"blng_sents_part\"]]}"),
        ])
        .send()
        .await?;

    let json: serde_json::Value = resp.json().await?;

    let mut us_phone = None;
    let mut uk_phone = None;
    let mut tags = Vec::new();
    let mut definitions = Vec::new();
    let mut examples = Vec::new();

    // 提取 ec (英汉释义和音标)
    if let Some(ec) = json.get("ec") {
        if let Some(exam_types) = ec.get("exam_type").and_then(|t| t.as_array()) {
            for t in exam_types {
                if let Some(s) = t.as_str() {
                    tags.push(s.to_string());
                }
            }
        }

        if let Some(word_arr) = ec.get("word").and_then(|w| w.as_array()) {
            if let Some(w) = word_arr.first() {
                if let Some(us) = w.get("usphone").and_then(|p| p.as_str()) {
                    if !us.is_empty() {
                        us_phone = Some(format!("/ {} /", us));
                    }
                }
                if let Some(uk) = w.get("ukphone").and_then(|p| p.as_str()) {
                    if !uk.is_empty() {
                        uk_phone = Some(format!("/ {} /", uk));
                    }
                }

                if let Some(trs) = w.get("trs").and_then(|t| t.as_array()) {
                    for tr in trs {
                        if let Some(tr_sub) = tr.get("tr").and_then(|t| t.as_array()) {
                            for sub in tr_sub {
                                if let Some(i_arr) = sub.get("l").and_then(|l| l.get("i")).and_then(|i| i.as_array()) {
                                    for i_val in i_arr {
                                        if let Some(line) = i_val.as_str() {
                                            let trimmed = line.trim();
                                            if let Some(dot_idx) = trimmed.find('.') {
                                                let pos = &trimmed[..=dot_idx];
                                                let mean = trimmed[dot_idx + 1..].trim();
                                                definitions.push(DefinitionGroup {
                                                    pos: pos.to_string(),
                                                    meanings: vec![mean.to_string()],
                                                });
                                            } else {
                                                definitions.push(DefinitionGroup {
                                                    pos: "".to_string(),
                                                    meanings: vec![trimmed.to_string()],
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 提取双语例句
    if let Some(blng) = json.get("blng_sents_part") {
        if let Some(pair_arr) = blng.get("sentence-pair").and_then(|p| p.as_array()) {
            for pair in pair_arr.iter().take(3) {
                let orig = pair.get("sentence").and_then(|s| s.as_str()).unwrap_or("");
                let trans = pair.get("sentence-translation").and_then(|s| s.as_str()).unwrap_or("");
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
        return Err(AppError::General("未能在词典中找到该词条".to_string()));
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

pub async fn smart_query(client: &Client, query: &str, force_sentence: bool) -> Result<QueryOutput> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err(AppError::General("查询内容不能为空".to_string()));
    }

    let word_count = trimmed.split_whitespace().count();
    let is_single_word = word_count <= 2 && !trimmed.contains('\n') && !trimmed.ends_with('.') && !trimmed.ends_with('。');

    if !force_sentence && is_single_word {
        if let Ok(detail) = fetch_dict_detail(client, trimmed).await {
            return Ok(QueryOutput::Dict(detail));
        }
    }

    // 句子翻译或未命中词典，调用 Google 翻译
    let trans = translate_text(client, trimmed, None, None).await?;
    Ok(QueryOutput::Sentence {
        original: trans.original,
        translated: trans.translated,
        detected_lang: trans.detected_lang,
        target_lang: trans.target_lang,
    })
}
