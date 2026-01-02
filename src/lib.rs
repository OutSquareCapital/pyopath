use pyo3::prelude::*;

/// A full-compatibility clone of Python's pathlib implemented in Rust.
#[pymodule]
fn pyopath(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", "0.1.0")?;
    Ok(())
}
