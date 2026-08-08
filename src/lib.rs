use pyo3::prelude::*;
pub mod model;
pub mod query_manager;
/// A Python module implemented in Rust.
#[pymodule]
fn coeirocore_rs (py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    let model = PyModule::new(py, "model")?;
    crate::model::model(py, &model)?;
    m.add_submodule(&model)?;
    
    let query_manager = PyModule::new(py, "query_manager")?;
    crate::query_manager::query_manager(py, &query_manager)?;
    m.add_submodule(&query_manager)?;

    Ok(())
}
