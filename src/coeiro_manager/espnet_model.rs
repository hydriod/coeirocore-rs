use std::fs;
use std::path::Path;
use std::sync::Arc;

use burn::backend::flex::Flex;
use burn::prelude::*;
use espnet_burn::text::{
    phoneme_tokenizer::pyopenjtalk_g2p_prosody,
    token_id_converter::TokenIDConverter,
};
use espnet_burn::tts::espnet_tts::model::TTSSynthesisOptions;
use espnet_burn::tts::inference::Text2Speech;
use numpy::PyArray1;
use pyo3::{
    exceptions::{PyFileNotFoundError, PyRuntimeError},
    prelude::*,
};
use yaml_rust2::YamlLoader;

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct EspnetModel {
    tts_model: Arc<Text2Speech<Flex>>,
    token_id_converter: TokenIDConverter,
}

impl std::fmt::Debug for EspnetModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EspnetModel")
            .field("token_list_repr", &self.token_id_converter.token_list_repr)
            .finish()
    }
}

/// config.yaml から TokenIDConverter を生成する。
fn token_id_converter_from_config(config_path: &str) -> PyResult<TokenIDConverter> {
    let content = fs::read_to_string(config_path).map_err(|e| {
        PyFileNotFoundError::new_err(format!("configパス({config_path})の読み込みに失敗しました: {e}"))
    })?;
    let yamls = YamlLoader::load_from_str(&content).map_err(|e| {
        PyRuntimeError::new_err(format!("configパス({config_path})のパースに失敗しました: {e}"))
    })?;
    let token_list: Vec<String> = yamls[0]["token_list"]
        .as_vec()
        .ok_or_else(|| {
            PyRuntimeError::new_err(format!("configパス({config_path})にtoken_listがありません"))
        })?
        .iter()
        .map(|t| t.as_str().unwrap_or_default().to_string())
        .collect();

    TokenIDConverter::new(token_list, Some("<unk>".to_string()))
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pymethods]
impl EspnetModel {
    #[new]
    #[pyo3(signature = (config_path, model_path, speed_scale = 1.0, use_gpu = false))]
    pub fn new(config_path: &str, model_path: &str, speed_scale: f64, use_gpu: bool) -> PyResult<Self> {
        // burn(CPU) バックエンドのみ対応
        if use_gpu {
            return Err(PyRuntimeError::new_err(
                "GPU(use_gpu=True)は現時点で未対応です",
            ));
        }

        // Only for FastSpeech & FastSpeech2 & VITS: speed_control_alpha
        // Only for VITS: noise_scale / noise_scale_dur
        let options = TTSSynthesisOptions::new_vits(speed_scale, 0.333, 0.333);
        let tts_model = Text2Speech::<Flex>::new(
            Some(Path::new(config_path)),
            Some(Path::new(model_path)),
            &Default::default(),
            Some(0),
            options,
        )
        .map_err(|e| PyRuntimeError::new_err(format!("モデルの初期化に失敗しました: {e}")))?;

        let token_id_converter = token_id_converter_from_config(config_path)?;

        Ok(EspnetModel {
            tts_model: Arc::new(tts_model),
            token_id_converter,
        })
    }

    #[staticmethod]
    fn text2tokens(text: &str) -> PyResult<Vec<String>> {
        pyopenjtalk_g2p_prosody(text, true)
            .map_err(|e| PyRuntimeError::new_err(format!("テキストのg2pに失敗しました: {e}")))
    }

    fn tokens2ids<'py>(
        &self,
        py: Python<'py>,
        tokens: Vec<String>,
    ) -> PyResult<Bound<'py, PyArray1<i64>>> {
        let ids: Vec<i64> = self
            .token_id_converter
            .tokens2ids(tokens)
            .into_iter()
            .map(|i| i as i64)
            .collect();
        Ok(PyArray1::from_vec(py, ids))
    }

    fn make_voice<'py>(
        &self,
        py: Python<'py>,
        text: &str,
        _seed: i64,
    ) -> PyResult<Bound<'py, PyArray1<f32>>> {
        // Python側で np.random.seed(seed) / torch.manual_seed(seed) を行っていたが、
        // burn 側に乱数種のAPIがないため現状は無視する
        let tokens = pyopenjtalk_g2p_prosody(text, true)
            .map_err(|e| PyRuntimeError::new_err(format!("テキストのg2pに失敗しました: {e}")))?;
        let wav = self.make_voice_tokens(tokens)?;
        Ok(PyArray1::from_vec(py, wav))
    }
}

impl EspnetModel {
    /// トークン列（音素+アクセント記号）から音声を生成する。
    /// 出力は float32 の wav サンプル列。
    pub(crate) fn make_voice_tokens(&self, tokens: Vec<String>) -> PyResult<Vec<f32>> {
        let ids: Vec<i64> = self
            .token_id_converter
            .tokens2ids(tokens)
            .into_iter()
            .map(|i| i as i64)
            .collect();
        let tensor = Tensor::<Flex, 1, Int>::from_data(
            TensorData::new(ids.clone(), [ids.len()]),
            &Default::default(),
        );

        let output = self
            .tts_model
            .forward(tensor, None)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let wav = output
            .wav
            .into_data()
            .convert::<f32>()
            .to_vec()
            .map_err(|e| PyRuntimeError::new_err(format!("wavデータの変換に失敗しました: {e}")))?;

        Ok(wav)
    }
}
