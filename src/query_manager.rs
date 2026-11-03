use pyo3::prelude::*;
use pyo3::FromPyObject;

#[pymodule]
pub fn query_manager(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(query2tokens_prosody, py)?)?;

    Ok(())
}

#[derive(FromPyObject)]
struct AccentPhrase {
    moras: Vec<Mora>,
    accent: i32,
    pause_mora: Option<Mora>,
    is_interrogative: bool,
}

#[derive(FromPyObject)]
#[allow(dead_code)]
struct Mora {
    text: String,
    consonant: Option<String>,
    consonant_length: Option<f64>,
    vowel: String,
    vowel_length: f64,
    pitch: f64,
}

#[pyfunction]
fn query2tokens_prosody(query: &Bound<'_, PyAny>) -> PyResult<Vec<String>> {
    // 引数は coeirocore_rs.model.AudioQuery(pyclass)、pydantic の voicevox_engine.model.AudioQuery
    // のどちらでも受け付けられるように getattr ベースで抽出する
    let accent_phrases = match query.getattr("accent_phrases") {
        Ok(obj) => obj,
        Err(_) => query.getattr("accentPhrases")?,
    }
    .extract::<Vec<AccentPhrase>>()?;
    let mut tokens = vec!["^".to_string()];
    for (i, accent_phrase) in accent_phrases.iter().enumerate() {
        let mut up_token_flag = false;
        for (j, mora) in accent_phrase.moras.iter().enumerate() {
            if let Some(consonant) = &mora.consonant {
                tokens.push(consonant.to_lowercase());
            }
            if mora.vowel == "N" {
                tokens.push(String::from("N"));
            } else {
                tokens.push(mora.vowel.to_lowercase());
            }
            if (accent_phrase.accent == j as i32 + 1) && (j + 1 != accent_phrase.moras.len()) {
                tokens.push("]".to_string());
            }
            if (accent_phrase.accent > j as i32 + 1) && (!up_token_flag) {
                tokens.push("[".to_string());
                up_token_flag = true;
            }
        }
        // pause_mora(無音区間)はPython元実装と同様にアクセントフレームごとに1つ追加する
        if i + 1 != accent_phrases.len() {
            if accent_phrase.pause_mora.is_some() {
                tokens.push("_".to_string())
            } else {
                tokens.push("#".to_string())
            }
        }
    }

    // Python元実装と同様、空のaccent_phrasesは IndexError を除いて "$" を返す
    if let Some(last) = accent_phrases.last() {
        if last.is_interrogative {
            tokens.push("?".to_string())
        } else {
            tokens.push("$".to_string())
        }
    } else {
        tokens.push("$".to_string())
    }

    Ok(tokens)
}
