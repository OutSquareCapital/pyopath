use pyo3::prelude::*;
mod core;
mod macros;
mod separators;
use macros::{PurePosixPath, PureWindowsPath};
// Platform-specific default
#[cfg(windows)]
pub type PurePath = PureWindowsPath;

#[cfg(unix)]
pub type PurePath = PurePosixPath;

#[pymodule]
fn pyopath(py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PurePosixPath>()?;
    m.add_class::<PureWindowsPath>()?;

    // Default alias
    #[cfg(windows)]
    m.add("PurePath", py.get_type::<PureWindowsPath>())?;

    #[cfg(unix)]
    m.add("PurePath", py.get_type::<PurePosixPath>())?;

    Ok(())
}
