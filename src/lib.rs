//! pyopath - A full-compatibility clone of Python's pathlib implemented in Rust.

use pyo3::prelude::*;

#[macro_use]
mod macros;
mod flavor;
mod parsing;
mod path;
mod pure_path;
mod stats;
use path::{Path, PosixPath, PurePosixPath, PureWindowsPath, WindowsPath};
use pure_path::PurePath;

/// A full-compatibility clone of Python's pathlib implemented in Rust.
#[pymodule]
mod pyopath {
    use super::*;

    #[pymodule_export]
    use super::PurePath;

    #[pymodule_export]
    use PurePosixPath;

    #[pymodule_export]
    use PureWindowsPath;

    #[pymodule_export]
    use super::Path;

    #[pymodule_export]
    use super::PosixPath;

    #[pymodule_export]
    use super::WindowsPath;

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add("__version__", "0.1.0")?;
        m.add(
            "__all__",
            vec![
                "PurePath",
                "PurePosixPath",
                "PureWindowsPath",
                "Path",
                "PosixPath",
                "WindowsPath",
            ],
        )?;
        Ok(())
    }
}
