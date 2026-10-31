use pyo3::prelude::*;

#[pymodule]
pub fn model(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Mora>()?;
    m.add_class::<AccentPhrase>()?;
    m.add_class::<AudioQuery>()?;

    Ok(())
}

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct Mora {
    #[pyo3(get, set)]
    pub text: String,
    #[pyo3(get, set)]
    pub consonant: Option<String>,
    #[pyo3(get, set)]
    pub consonant_length: Option<f64>,
    #[pyo3(get, set)]
    pub vowel: String,
    #[pyo3(get, set)]
    pub vowel_length: f64,
    #[pyo3(get, set)]
    pub pitch: f64,
}

#[pymethods]
impl Mora {
    #[new]
    fn new(
        text: String,
        consonant: Option<String>,
        consonant_length: Option<f64>,
        vowel: String,
        vowel_length: f64,
        pitch: f64,
    ) -> Self {
        Mora {
            text,
            consonant,
            consonant_length,
            vowel,
            vowel_length,
            pitch,
        }
    }
}

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct AccentPhrase {
    #[pyo3(get, set)]
    pub moras: Vec<Mora>,
    #[pyo3(get, set)]
    pub accent: i32,
    #[pyo3(get, set)]
    pub pause_mora: Option<Mora>,
    #[pyo3(get, set)]
    pub is_interrogative: bool,
}

#[pymethods]
impl AccentPhrase {
    #[new]
    fn new(
        moras: Vec<Mora>,
        accent: i32,
        pause_mora: Option<Mora>,
        is_interrogative: bool,
    ) -> Self {
        AccentPhrase {
            moras,
            accent,
            pause_mora,
            is_interrogative,
        }
    }
}

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct AudioQuery {
    #[pyo3(get, set, name = "accentPhrases")]
    pub accent_phrases: Vec<AccentPhrase>,
    #[pyo3(get, set, name = "speedScale")]
    pub speed_scale: f64,
    #[pyo3(get, set, name = "pitchScale")]
    pub pitch_scale: f64,
    #[pyo3(get, set, name = "intonationScale")]
    pub intonation_scale: f64,
    #[pyo3(get, set, name = "volumeScale")]
    pub volume_scale: f64,
    #[pyo3(get, set, name = "prePhonemeLength")]
    pub pre_phoneme_length: f64,
    #[pyo3(get, set, name = "postPhonemeLength")]
    pub post_phoneme_length: f64,
    #[pyo3(get, set, name = "outputSamplingRate")]
    pub output_sampling_rate: i32,
    #[pyo3(get, set, name = "outputStereo")]
    pub output_stereo: bool,
    #[pyo3(get, set, name = "kana")]
    pub kana: Option<String>,
}

#[pymethods]
impl AudioQuery {
    #[allow(clippy::too_many_arguments)]
    #[new]
    #[pyo3(signature = (
        accent_phrases,
        speed_scale,
        pitch_scale,
        intonation_scale,
        volume_scale,
        pre_phoneme_length,
        post_phoneme_length,
        output_sampling_rate,
        output_stereo,
        kana = None,
    ))]
    fn new(
        accent_phrases: Vec<AccentPhrase>,
        speed_scale: f64,
        pitch_scale: f64,
        intonation_scale: f64,
        volume_scale: f64,
        pre_phoneme_length: f64,
        post_phoneme_length: f64,
        output_sampling_rate: i32,
        output_stereo: bool,
        kana: Option<String>,
    ) -> Self {
        AudioQuery {
            accent_phrases,
            speed_scale,
            pitch_scale,
            intonation_scale,
            volume_scale,
            pre_phoneme_length,
            post_phoneme_length,
            output_sampling_rate,
            output_stereo,
            kana,
        }
    }
}
