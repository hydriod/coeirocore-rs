use pyo3::prelude::*;
pub mod coeiro_manager;
pub mod model;
pub mod query_manager;

/// サブモジュールをフルネーム(例: "coeirocore_rs.model")で sys.modules に登録し、
/// 親モジュールにも属性として追加する。
/// `from coeirocore_rs.coeiro_manager import X` のような
/// サブモジュール経由の import は sys.modules 登録がないと解決できない
/// (import システムは親モジュールの __path__ から実ファイルを探しに行くため)。
fn register_submodule(
    py: Python<'_>,
    parent: &Bound<'_, PyModule>,
    name: &str,
    init: fn(Python<'_>, &Bound<'_, PyModule>) -> PyResult<()>,
) -> PyResult<()> {
    let full_name = format!("coeirocore_rs.{name}");
    let submodule = PyModule::new(py, &full_name)?;
    init(py, &submodule)?;
    // 注意: 親モジュールの __dict__ (sys.__dict__) に書いても sys.modules には反映されない。
    // sys.modules 辞書そのものに登録する。
    py.import("sys")?
        .getattr("modules")?
        .cast_into::<pyo3::types::PyDict>()?
        .set_item(&full_name, &submodule)?;
    parent.add_submodule(&submodule)?;
    Ok(())
}

/// A Python module implemented in Rust.
#[pymodule]
fn coeirocore_rs(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    register_submodule(py, m, "model", crate::model::model)?;
    register_submodule(py, m, "query_manager", crate::query_manager::query_manager)?;
    register_submodule(
        py,
        m,
        "coeiro_manager",
        crate::coeiro_manager::coeiro_manager,
    )?;

    Ok(())
}
