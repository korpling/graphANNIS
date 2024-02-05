use pyo3::prelude::*;

#[pyfunction]
fn apply_update() -> String {
    "func".to_string()
}

/// Python bindings for graphANNIS.
#[pymodule]
fn graphannis(py: Python, m: &PyModule) -> PyResult<()> {
    let cs_module = PyModule::new(py, "cs")?;
    cs_module.add_function(wrap_pyfunction!(apply_update, cs_module)?)?;
    m.add_submodule(cs_module)?;
    Ok(())
}
