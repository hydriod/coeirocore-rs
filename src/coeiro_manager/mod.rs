pub mod audio_manager;
pub mod espnet_model;
pub mod meta_manager;
pub mod speaker_info;

pub use crate::coeiro_manager::audio_manager::AudioManager;
pub use crate::coeiro_manager::espnet_model::EspnetModel;
pub use crate::coeiro_manager::meta_manager::{MetaManager, ModelPath};
use pyo3::prelude::*;

#[pymodule]
pub fn coeiro_manager(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ModelPath>()?;
    m.add_class::<MetaManager>()?;
    m.add_class::<EspnetModel>()?;
    m.add_class::<AudioManager>()?;

    Ok(())
}
