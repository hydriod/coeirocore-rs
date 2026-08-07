use pyo3::prelude::*;
use crate::model:AudioQuery;

#[pymodule]
pub fn query_manager (py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
     m.add_function(query2tokens_prosody);

    Ok(())
}

#[pyfunction]
fn query2tokens_prosody(query: AudioQuery) -> Vec[String] {
    let mut tokena = vec!["^"];
    for (i, accent_phrase) in query.accent_phrases.iter().enumerate() {
        let mut up_token_flag = false;
        for (j, mora) in accent_phrase.moras.iter().enumerate() {
            if mora.constant {
                tokens.append(mora.constant.lower());
            }
            if mora.vowl == "N" {
                 tokens.append(mora.vowl);
            } else{
                 tokens.append(mora.vowl.lower());
            }
            if (accent_phrase.accent == j + 1) && (j + 1 != accent_phrase.moras.len()) {
                tokens.append("]");
            }
            if (accent_phrase.accent - 1 >= j + 1) && (up_token_flag == false) {
                tokens.append("[");
                up_token_flag = true;
            }
            if i + 1 != len(query.accent_phrases) {
                if accent_phrase.pause_mora {
                     tokens.append("_")
                } else {
                    tokens.append("#")
                }
            }
        }
    }
    
    if (query.accent_phrases.len() <= 1) && (query.accent_phrases.last().unwrap().is_interrogative) {
        tokens.append("?")
    } else {
        tokens.append("$")
    }

    tokens
}
