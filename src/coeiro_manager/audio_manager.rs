use espnet_burn::text::phoneme_tokenizer::pyopenjtalk_g2p_prosody;
use numpy::PyArray1;
use pyo3::{
    exceptions::{PyKeyError, PyRuntimeError},
    prelude::*,
};
use rubato::{FixedSync, Fft, Resampler};
use rubato::audioadapter_buffers::owned::InterleavedOwned;
use rsworld::{cheaptrick, d4c, harvest, stonemask, synthesis as world_synthesis};
use rsworld_sys::{CheapTrickOption, D4COption, HarvestOption};

use crate::coeiro_manager::{
    espnet_model::EspnetModel,
    meta_manager::{MetaManager, ModelPath},
};

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct AudioManager {
    fs: i32,
    use_gpu: bool,
    meta_manager: MetaManager,
    previous_style_id: i32,
    previous_speed_scale: f64,
    current_speaker_model: EspnetModel,
}

#[pymethods]
impl AudioManager {
    #[new]
    pub fn new(fs: Option<i32>, use_gpu: Option<bool>) -> PyResult<Self> {
        let fs = fs.unwrap_or(44100);
        let use_gpu = use_gpu.unwrap_or(false);

        let meta_manager = MetaManager::new()?;
        let previous_style_id = meta_manager.first_style_id();
        let previous_speed_scale = 1.0;
        let model_path = get_model_path(&meta_manager, previous_style_id)?;
        let current_speaker_model = EspnetModel::new(
            &model_path.config_path,
            &model_path.model_path,
            previous_speed_scale,
            use_gpu,
        )?;

        Ok(AudioManager {
            fs,
            use_gpu,
            meta_manager,
            previous_style_id,
            previous_speed_scale,
            current_speaker_model,
        })
    }

    #[pyo3(signature = (
        text,
        style_id,
        speed_scale = 1.0,
        volume_scale = 1.0,
        pitch_scale = 0.0,
        intonation_scale = 1.0,
        pre_phoneme_length = 0.0,
        post_phoneme_length = 0.0,
        output_sampling_rate = 44100
    ))]
    fn synthesis<'py>(
        &mut self,
        py: Python<'py>,
        text: &Bound<'py, PyAny>,
        style_id: i32,
        speed_scale: f64,
        volume_scale: f64,
        pitch_scale: f64,
        intonation_scale: f64,
        pre_phoneme_length: f64,
        post_phoneme_length: f64,
        output_sampling_rate: i32,
    ) -> PyResult<Bound<'py, PyArray1<f32>>> {
        // speaker_load
        if self.previous_style_id != style_id || self.previous_speed_scale != speed_scale {
            let model_path = get_model_path(&self.meta_manager, style_id)?;
            self.current_speaker_model = EspnetModel::new(
                &model_path.config_path,
                &model_path.model_path,
                1.0 / speed_scale,
                self.use_gpu,
            )?;
            self.previous_style_id = style_id;
            self.previous_speed_scale = speed_scale;
        }

        // synthesis
        // textは文字列(日本語)もしくはトークン列（音素+アクセント記号）を受け付ける
        let tokens: Vec<String> = if let Ok(text) = text.extract::<String>() {
            pyopenjtalk_g2p_prosody(&text, true)
                .map_err(|e| PyRuntimeError::new_err(format!("テキストのg2pに失敗しました: {e}")))?
        } else {
            text.extract()?
        };
        let wav = self.current_speaker_model.make_voice_tokens(tokens)?;

        // post-processing
        let mut wav = trim(wav);
        if volume_scale != 1.0 {
            wav = volume(wav, volume_scale);
        }
        if pitch_scale != 0.0 || intonation_scale != 1.0 {
            wav = pitch_intonation(wav, self.fs, pitch_scale, intonation_scale);
        }
        if pre_phoneme_length != 0.0 || post_phoneme_length != 0.0 {
            wav = sil(wav, self.fs, pre_phoneme_length, post_phoneme_length);
        }
        if output_sampling_rate != self.fs {
            wav = resampling(wav, self.fs, output_sampling_rate)?;
        }

        Ok(PyArray1::from_vec(py, wav))
    }

    #[staticmethod]
    pub fn trim(wav: Vec<f32>) -> Vec<f32> {
        trim(wav)
    }

    #[staticmethod]
    pub fn volume(wav: Vec<f32>, volume_scale: f64) -> Vec<f32> {
        volume(wav, volume_scale)
    }

    #[staticmethod]
    pub fn pitch_intonation(
        wav: Vec<f32>,
        fs: i32,
        pitch_scale: f64,
        intonation_scale: f64,
    ) -> Vec<f32> {
        pitch_intonation(wav, fs, pitch_scale, intonation_scale)
    }

    #[staticmethod]
    pub fn sil(wav: Vec<f32>, fs: i32, pre_phoneme_length: f64, post_phoneme_length: f64) -> Vec<f32> {
        sil(wav, fs, pre_phoneme_length, post_phoneme_length)
    }

    #[staticmethod]
    pub fn resampling(wav: Vec<f32>, fs: i32, output_sampling_rate: i32) -> PyResult<Vec<f32>> {
        resampling(wav, fs, output_sampling_rate)
    }
}

/// id_model_mapからスタイルIDに対応するモデルパスを取り出す。
fn get_model_path(meta_manager: &MetaManager, style_id: i32) -> PyResult<ModelPath> {
    meta_manager
        .get_model_path(style_id)
        .ok_or_else(|| PyKeyError::new_err(format!("style_id({style_id})のモデルが見つかりません")))
}

/// librosa.effects.trim(top_db=30) 相当。
/// librosaはSTFTの振幅スペクトログラム（フレーム長2048、hop長512）のdB最大値で
/// 判定するが、ここではフレームごとのRMS [dB] で近似している。
fn trim(wav: Vec<f32>) -> Vec<f32> {
    const FRAME_LENGTH: usize = 2048;
    const HOP_LENGTH: usize = 512;
    const TOP_DB: f64 = 30.0;

    if wav.is_empty() {
        return wav;
    }

    let mut frame_dbs = Vec::new();
    let mut i = 0;
    while i < wav.len() {
        let end = (i + FRAME_LENGTH).min(wav.len());
        let mean_square = wav[i..end]
            .iter()
            .map(|&x| x * x)
            .sum::<f32>()
            / (end - i) as f32;
        frame_dbs.push(10.0 * (mean_square.max(f32::EPSILON) as f64).log10());
        i += HOP_LENGTH;
    }

    let max_db = frame_dbs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let threshold = max_db - TOP_DB;
    let first = frame_dbs.iter().position(|&db| db > threshold);
    let last = frame_dbs.iter().rposition(|&db| db > threshold);
    let (first, last) = match (first, last) {
        (Some(first), Some(last)) => (first, last),
        _ => return Vec::new(),
    };

    let start = (first * HOP_LENGTH).min(wav.len());
    let end = (((last + 1) * HOP_LENGTH).min(wav.len())).max(start + 1);
    wav[start..end].to_vec()
}

fn volume(wav: Vec<f32>, volume_scale: f64) -> Vec<f32> {
    let volume_scale = volume_scale as f32;
    wav.into_iter().map(|x| x * volume_scale).collect()
}

fn pitch_intonation(wav: Vec<f32>, fs: i32, pitch_scale: f64, intonation_scale: f64) -> Vec<f32> {
    if wav.is_empty() {
        return wav;
    }

    // Python側は wav.astype(np.float64) でWORLDに渡す
    let x: Vec<f64> = wav.iter().map(|&v| v as f64).collect();
    let (mut f0, sp, ap) = get_world(&x, fs);
    if f0.is_empty() {
        return wav;
    }

    // pitch
    if pitch_scale != 0.0 {
        let factor = 2f64.powf(pitch_scale);
        f0.iter_mut().for_each(|f| *f *= factor);
    }

    // intonation
    if intonation_scale != 1.0 {
        let m = f0.iter().sum::<f64>() / f0.len() as f64;
        let s = (f0
            .iter()
            .map(|f| (f - m) * (f - m))
            .sum::<f64>()
            / f0.len() as f64)
            .sqrt();
        f0.iter_mut()
            .for_each(|f| *f = ((*f - m) / s) * (s * intonation_scale) + m);
    }

    // pw.synthesize(f0, sp, ap, fs) 相当（frame_period=5.0ms）
    world_synthesis(&f0, &sp, &ap, 5.0, fs)
        .into_iter()
        .map(|v| v as f32)
        .collect()
}

/// 前後に無音を挿入する。
fn sil(wav: Vec<f32>, fs: i32, pre_phoneme_length: f64, post_phoneme_length: f64) -> Vec<f32> {
    // np.zeros(int(fs * len)) 相当（int()は0方向に切り捨て）
    let pre_pause = (fs as f64 * pre_phoneme_length).max(0.0) as usize;
    let post_pause = (fs as f64 * post_phoneme_length).max(0.0) as usize;

    let mut result = Vec::with_capacity(pre_pause + wav.len() + post_pause);
    result.extend(std::iter::repeat_n(0.0f32, pre_pause));
    result.extend(wav);
    result.extend(std::iter::repeat_n(0.0f32, post_pause));
    result
}

/// resampy.resample(..., filter="kaiser_fast") 相当。
/// rubato の同期FFTリサンプラー（BlackmanHarris2窓）で近似している。
fn resampling(wav: Vec<f32>, fs: i32, output_sampling_rate: i32) -> PyResult<Vec<f32>> {
    if wav.is_empty() {
        return Ok(wav);
    }

    let mut resampler = Fft::<f32>::new(
        fs.max(1) as usize,
        output_sampling_rate.max(1) as usize,
        1024,
        1,
        FixedSync::Input,
    )
    .map_err(|e| PyRuntimeError::new_err(format!("リサンプラーの生成に失敗しました: {e}")))?;

    let input_len = wav.len();
    let buffer_in = InterleavedOwned::new_from(wav, 1, input_len).map_err(|e| {
        PyRuntimeError::new_err(format!("リサンプリング入力バッファの生成に失敗しました: {e}"))
    })?;

    let buffer_out = resampler
        .process_all(&buffer_in, input_len, None)
        .map_err(|e| PyRuntimeError::new_err(format!("リサンプリングに失敗しました: {e}")))?;
    Ok(buffer_out.take_data())
}

// https://github.com/JeremyCCHsu/Python-Wrapper-for-World-Vocoder/blob/3a7c99a32c717deb8e66bde64b5e60b1a4afce79/demo/demo.py
fn get_world(x: &[f64], fs: i32) -> (Vec<f64>, Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let x = x.to_vec();
    let harvest_option = HarvestOption::new();
    let (t_h, f0_raw) = harvest(&x, fs, &harvest_option);
    let f0_h = stonemask(&x, fs, &t_h, &f0_raw);
    let mut cheaptrick_option = CheapTrickOption::new(fs);
    let sp_h = cheaptrick(&x, fs, &t_h, &f0_h, &mut cheaptrick_option);
    let ap_h = d4c(&x, fs, &t_h, &f0_h, &D4COption::new());
    (f0_h, sp_h, ap_h)
}
