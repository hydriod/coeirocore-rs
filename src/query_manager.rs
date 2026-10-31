use crate::model::AudioQuery;
use pyo3::prelude::*;

#[pymodule]
pub fn query_manager(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(query2tokens_prosody, py)?)?;

    Ok(())
}

#[pyfunction]
fn query2tokens_prosody(query: &Bound<'_, AudioQuery>) -> PyResult<Vec<String>> {
    let query = query.extract::<AudioQuery>()?;
    let mut tokens = vec!["^".to_string()];
    for (i, accent_phrase) in query.accent_phrases.iter().enumerate() {
        let mut up_token_flag = false;
        for (j, mora) in accent_phrase.moras.iter().enumerate() {
            if let Some(consonant) = mora.consonant.clone() {
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
        if i + 1 != query.accent_phrases.len() {
            if accent_phrase.pause_mora.is_some() {
                tokens.push("_".to_string())
            } else {
                tokens.push("#".to_string())
            }
        }
    }

    // Python元実装と同様、空のaccent_phrasesは IndexError を除いて "$" を返す
    if let Some(last) = query.accent_phrases.last() {
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
