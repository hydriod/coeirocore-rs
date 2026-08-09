use std::{collections::HashMap, fs::File, path::Path};

use glob::glob;
use pyo3::{
    exceptions::PyOSError,
    prelude::*,
    types::{PyInt, PyList, PyString},
};

use crate::coeiro_manager::speaker_info::{Meta, Style};

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct ModelPath {
    #[pyo3(get, set)]
    pub model_path: String,
    #[pyo3(get, set)]
    pub config_path: String,
}

#[derive(Debug, Clone)]
struct SpeakerInfo {
    pub speaker_name: String,
    pub speaker_uuid: String,
    pub styles: Vec<Style>,
    pub version: String,
}

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct MetaManager {
    speaker_infos: Vec<SpeakerInfo>,
    #[pyo3(get, set)]
    id_model_map: HashMap<i32, ModelPath>,
}

#[pymethods]
impl MetaManager {
    #[new]
    pub fn new() -> PyResult<Self> {
        let mut uuid_list = Vec::<String>::new();
        let mut speaker_infos = Vec::<SpeakerInfo>::new();
        let mut id_model_map = HashMap::<i32, ModelPath>::new();
        for entry in glob("./speaker_info/**/").unwrap() {
            if let Ok(path) = entry {
                match File::open(path.join("metas.json")) {
                    Ok(file) => {
                        match serde_json::from_reader::<File, Meta>(file) {
                            Ok(meta) => {
                                // 重複確認
                                if uuid_list.contains(&meta.speaker_uuid) {
                                    eprintln!(
                                        "SpeakerUuid({})の重複があります。",
                                        meta.speaker_uuid
                                    );
                                    continue;
                                }
                                uuid_list.push(meta.speaker_uuid.clone());

                                speaker_infos.push(meta_to_speaker_info(&meta));
                                id_model_map = append_id_model_map(&meta, id_model_map, &path);
                            }
                            Err(e) => {
                                // スキーマに従わない場合はエラーを出力してスキップ
                                eprintln!("Failed to parse metas.json: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        return Err(PyOSError::new_err(format!(
                            "Failed to open metas.json: {}",
                            e
                        )));
                    }
                }
            }
        }
        Ok(MetaManager {
            speaker_infos,
            id_model_map,
        })
    }

    fn get_metas_dict<'py>(
        &self,
        py: Python<'py>,
    ) -> PyResult<Vec<HashMap<String, Bound<'py, PyAny>>>> {
        let mut result = Vec::new();
        for speaker_info in &self.speaker_infos {
            let mut styles_list = Vec::new();
            for style in &speaker_info.styles {
                let mut style_dict = HashMap::new();
                style_dict.insert(
                    "style_id".to_string(),
                    PyInt::new(py, style.style_id).into_any(),
                );
                style_dict.insert(
                    "style_name".to_string(),
                    PyString::new(py, &style.style_name).into_any(),
                );
                styles_list.push(style_dict);
            }

            let mut speaker_dict = HashMap::new();
            speaker_dict.insert(
                "speaker_name".to_string(),
                PyString::new(py, &speaker_info.speaker_name).into_any(),
            );
            speaker_dict.insert(
                "speaker_uuid".to_string(),
                PyString::new(py, &speaker_info.speaker_uuid).into_any(),
            );
            speaker_dict.insert(
                "version".to_string(),
                PyString::new(py, &speaker_info.version).into_any(),
            );
            speaker_dict.insert(
                "styles".to_string(),
                PyList::new(py, styles_list)?.into_any(),
            );
            result.push(speaker_dict);
        }
        Ok(result)
    }

    pub fn default_style_id(&self) -> i32 {
        0
    }
}

fn meta_to_speaker_info(meta: &Meta) -> SpeakerInfo {
    SpeakerInfo {
        speaker_name: meta.speaker_name.clone(),
        speaker_uuid: meta.speaker_uuid.clone(),
        styles: meta.styles.clone(),
        version: meta.version.clone().unwrap_or("0.0.1".to_string()),
    }
}

fn append_id_model_map(
    meta: &Meta,
    mut id_model_map: HashMap<i32, ModelPath>,
    speaker_path: &Path,
) -> HashMap<i32, ModelPath> {
    for style in &meta.styles {
        if id_model_map.contains_key(&style.style_id) {
            eprintln!("Style ids ({}) are duplicated", style.style_id);
        }

        let model_folder_path = speaker_path.join("model").join(style.style_id.to_string());
        let model_paths = glob(model_folder_path.join("*.pth").to_str().unwrap()).unwrap();
        let mut model_path = None;
        for entry in model_paths {
            if let Ok(valid_model_path) = entry {
                model_path = valid_model_path.into();
                break;
            }
        }
        match model_path {
            Some(model_path) => {
                let model_path = ModelPath {
                    model_path: String::from(model_path.to_str().unwrap()),
                    config_path: String::from(
                        model_folder_path.join("config.yaml").to_str().unwrap(),
                    ),
                };
                id_model_map.insert(style.style_id, model_path);
            }
            None => {
                eprintln!("No model file found for style id: {}", style.style_id);
            }
        }
    }

    id_model_map
}
