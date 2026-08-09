use std::fs;

use espnet2::{
    bin::tts_inference::Text2Speech,
    text::{phoneme_tokenizer::openjtalk_g2p_prosody, token_id_converter::TokenIDConverter},
};
use numpy::PyArray1;
use pyo3::{
    exceptions::{PyFileNotFoundError, PyRuntimeError, PyTimeoutError},
    prelude::*,
};
use yaml_rust2::{YamlLoader, parser::Parser};

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct EspnetModel {
    tts_model: Text2Speech,
    token_id_converter: TokenIDConverter,
}

#[pymethods]
impl EspnetModel {
    #[new]
    pub fn new(
        config_path: &str,
        model_path: &str,
        speed_scale: f32,
        use_gpu: bool,
    ) -> PyResult<Self> {
        let device = if use_gpu { "gpu" } else { "cpu" };
        let tts_model = Text2Speech(
            config_path,
            model_path,
            device,
            0,
            speed_scale,
            0.333,
            0.333,
        );

        let content =
            fs::read_to_string(config_path).map_err(|e| PyFileNotFoundError::new_err(e))?;
        let yaml = YamlLoader::load_from_str(&content).map_err(|e| PyRuntimeError::new_err(e))?;
        let yaml = &yaml[0];

        let token_id_converter = TokenIDConverter(yaml["token_list"].as_hash(), "<unk>");

        EspnetModel {
            tts_model,
            token_id_converter,
        }
    }

    #[staticmethod]
    fn text2tokens(text: &str) -> PyResult<Vec<String>> {
        Ok(openjtalk_g2p_prosody(&text))
    }

    fn token2ids<'py>(
        &self,
        py: Python<'py>,
        tokens: Vec<String>,
    ) -> PyResult<Bound<'py, PyArray1<i64>>> {
        Ok(PyArray1::from_vec(
            py,
            &self.token_id_converter.token2ids(tokens),
        ))
    }

    fn make_voice<'py>(
        &self,
        py: Python<'py>,
        text: &str,
        seed: i64,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let output = self.tts_model.__call__(text);
        let wav = output["wav"];

        wav
    }
}
