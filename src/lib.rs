use pyo3::prelude::*;
pub mod model;
/// A Python module implemented in Rust.
#[pymodule]
fn coeirocore_rs (py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    let model = PyModule::new(py, "model")?;
    crate::model::model(py, &model)?;
    m.add_submodule(&model)?;

    Ok(())
}
