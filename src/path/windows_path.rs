//! WindowsPath - Windows-specific concrete path.

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyTuple};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use crate::pure_path::PurePath;
use crate::pure_path::flavor::PathFlavor;
use crate::pure_path::parsing::ParsedPath;

use super::path::{
    StatResult, path_absolute, path_cwd, path_exists, path_glob, path_home, path_is_dir,
    path_is_file, path_is_symlink, path_iterdir, path_lstat, path_mkdir, path_read_bytes,
    path_read_text, path_readlink, path_rename, path_resolve, path_rglob, path_rmdir, path_stat,
    path_touch, path_unlink, path_write_bytes, path_write_text,
};

/// A Windows path with filesystem operations.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct WindowsPath {
    pub(crate) inner: PurePath,
}

impl PartialEq for WindowsPath {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for WindowsPath {}

impl Hash for WindowsPath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl WindowsPath {
    fn to_pathbuf(&self) -> PathBuf {
        PathBuf::from(self.inner.to_str())
    }
}

#[pymethods]
impl WindowsPath {
    #[new]
    #[pyo3(signature = (*args))]
    fn new(args: &Bound<'_, PyTuple>) -> PyResult<Self> {
        let inner = PurePath::from_args_with_flavor(args, PathFlavor::Windows)?;
        Ok(Self { inner })
    }

    // ==================== Properties ====================

    #[getter]
    fn drive(&self) -> &str {
        self.inner.get_drive()
    }

    #[getter]
    fn root(&self) -> &str {
        self.inner.get_root()
    }

    #[getter]
    fn anchor(&self) -> String {
        self.inner.get_anchor()
    }

    #[getter]
    fn parts(&self) -> Vec<String> {
        self.inner.get_parts()
    }

    #[getter]
    fn name(&self) -> &str {
        self.inner.get_name()
    }

    #[getter]
    fn suffix(&self) -> String {
        self.inner.get_suffix()
    }

    #[getter]
    fn suffixes(&self) -> Vec<String> {
        self.inner.get_suffixes()
    }

    #[getter]
    fn stem(&self) -> String {
        self.inner.get_stem()
    }

    #[getter]
    fn parent(&self) -> Self {
        Self {
            inner: self.inner.get_parent(),
        }
    }

    #[getter]
    fn parents(&self) -> Vec<Self> {
        self.inner
            .get_parents()
            .into_iter()
            .map(|p| Self { inner: p })
            .collect()
    }

    fn is_absolute(&self) -> bool {
        self.inner.get_is_absolute()
    }

    fn is_relative_to(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let other_path = if let Ok(p) = other.extract::<Self>() {
            p.inner
        } else {
            let s: String = other.extract()?;
            PurePath {
                parsed: ParsedPath::parse(&s, self.inner.flavor),
                flavor: self.inner.flavor,
            }
        };
        Ok(self.inner.get_is_relative_to(&other_path))
    }

    #[pyo3(signature = (other, walk_up=false))]
    fn relative_to(&self, other: &Bound<'_, PyAny>, walk_up: bool) -> PyResult<Self> {
        let other_path = if let Ok(p) = other.extract::<Self>() {
            p.inner
        } else {
            let s: String = other.extract()?;
            PurePath {
                parsed: ParsedPath::parse(&s, self.inner.flavor),
                flavor: self.inner.flavor,
            }
        };
        Ok(Self {
            inner: self.inner.compute_relative_to(&other_path, walk_up)?,
        })
    }

    #[pyo3(signature = (*args))]
    fn joinpath(&self, args: &Bound<'_, PyTuple>) -> PyResult<Self> {
        Ok(Self {
            inner: self.inner.compute_joinpath(args)?,
        })
    }

    fn with_name(&self, name: &str) -> PyResult<Self> {
        Ok(Self {
            inner: self.inner.compute_with_name(name)?,
        })
    }

    fn with_stem(&self, stem: &str) -> PyResult<Self> {
        Ok(Self {
            inner: self.inner.compute_with_stem(stem)?,
        })
    }

    fn with_suffix(&self, suffix: &str) -> PyResult<Self> {
        Ok(Self {
            inner: self.inner.compute_with_suffix(suffix)?,
        })
    }

    fn as_posix(&self) -> String {
        self.inner.get_as_posix()
    }

    // ==================== Filesystem operations ====================

    fn exists(&self) -> bool {
        path_exists(&self.to_pathbuf())
    }

    fn is_file(&self) -> bool {
        path_is_file(&self.to_pathbuf())
    }

    fn is_dir(&self) -> bool {
        path_is_dir(&self.to_pathbuf())
    }

    fn is_symlink(&self) -> bool {
        path_is_symlink(&self.to_pathbuf())
    }

    fn absolute(&self) -> PyResult<Self> {
        let abs = path_absolute(&self.to_pathbuf())?;
        Ok(Self {
            inner: PurePath {
                parsed: ParsedPath::parse(&abs.to_string_lossy(), self.inner.flavor),
                flavor: self.inner.flavor,
            },
        })
    }

    #[pyo3(signature = (strict=false))]
    fn resolve(&self, strict: bool) -> PyResult<Self> {
        let resolved = path_resolve(&self.to_pathbuf(), strict)?;
        Ok(Self {
            inner: PurePath {
                parsed: ParsedPath::parse(&resolved.to_string_lossy(), self.inner.flavor),
                flavor: self.inner.flavor,
            },
        })
    }

    fn readlink(&self) -> PyResult<Self> {
        let target = path_readlink(&self.to_pathbuf())?;
        Ok(Self {
            inner: PurePath {
                parsed: ParsedPath::parse(&target.to_string_lossy(), self.inner.flavor),
                flavor: self.inner.flavor,
            },
        })
    }

    fn stat(&self) -> PyResult<StatResult> {
        path_stat(&self.to_pathbuf())
    }

    fn lstat(&self) -> PyResult<StatResult> {
        path_lstat(&self.to_pathbuf())
    }

    #[pyo3(signature = (mode=0o777, parents=false, exist_ok=false))]
    fn mkdir(&self, mode: u32, parents: bool, exist_ok: bool) -> PyResult<()> {
        path_mkdir(&self.to_pathbuf(), mode, parents, exist_ok)
    }

    fn rmdir(&self) -> PyResult<()> {
        path_rmdir(&self.to_pathbuf())
    }

    #[pyo3(signature = (missing_ok=false))]
    fn unlink(&self, missing_ok: bool) -> PyResult<()> {
        path_unlink(&self.to_pathbuf(), missing_ok)
    }

    fn rename(&self, target: &Self) -> PyResult<Self> {
        path_rename(&self.to_pathbuf(), &target.to_pathbuf())?;
        Ok(target.clone())
    }

    fn replace(&self, target: &Self) -> PyResult<Self> {
        path_rename(&self.to_pathbuf(), &target.to_pathbuf())?;
        Ok(target.clone())
    }

    #[pyo3(signature = (exist_ok=true))]
    fn touch(&self, exist_ok: bool) -> PyResult<()> {
        path_touch(&self.to_pathbuf(), exist_ok)
    }

    #[pyo3(signature = (encoding=None))]
    fn read_text(&self, encoding: Option<&str>) -> PyResult<String> {
        path_read_text(&self.to_pathbuf(), encoding)
    }

    #[pyo3(signature = (data, encoding=None))]
    fn write_text(&self, data: &str, encoding: Option<&str>) -> PyResult<usize> {
        path_write_text(&self.to_pathbuf(), data, encoding)
    }

    fn read_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let data = path_read_bytes(&self.to_pathbuf())?;
        Ok(PyBytes::new(py, &data))
    }

    fn write_bytes(&self, data: &[u8]) -> PyResult<usize> {
        path_write_bytes(&self.to_pathbuf(), data)
    }

    fn iterdir(&self) -> PyResult<Vec<Self>> {
        let entries = path_iterdir(&self.to_pathbuf())?;
        Ok(entries
            .into_iter()
            .map(|p| Self {
                inner: PurePath {
                    parsed: ParsedPath::parse(&p.to_string_lossy(), self.inner.flavor),
                    flavor: self.inner.flavor,
                },
            })
            .collect())
    }

    fn glob(&self, pattern: &str) -> PyResult<Vec<Self>> {
        let entries = path_glob(&self.to_pathbuf(), pattern)?;
        Ok(entries
            .into_iter()
            .map(|p| Self {
                inner: PurePath {
                    parsed: ParsedPath::parse(&p.to_string_lossy(), self.inner.flavor),
                    flavor: self.inner.flavor,
                },
            })
            .collect())
    }

    fn rglob(&self, pattern: &str) -> PyResult<Vec<Self>> {
        let entries = path_rglob(&self.to_pathbuf(), pattern)?;
        Ok(entries
            .into_iter()
            .map(|p| Self {
                inner: PurePath {
                    parsed: ParsedPath::parse(&p.to_string_lossy(), self.inner.flavor),
                    flavor: self.inner.flavor,
                },
            })
            .collect())
    }

    #[pyo3(signature = (mode="r", buffering=-1, encoding=None, errors=None, newline=None))]
    fn open<'py>(
        &self,
        py: Python<'py>,
        mode: &str,
        buffering: i32,
        encoding: Option<&str>,
        errors: Option<&str>,
        newline: Option<&str>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let builtins = py.import("builtins")?;
        let open_fn = builtins.getattr("open")?;
        open_fn.call1((
            self.inner.to_str(),
            mode,
            buffering,
            encoding,
            errors,
            newline,
        ))
    }

    #[staticmethod]
    fn cwd() -> PyResult<Self> {
        let cwd = path_cwd()?;
        Ok(Self {
            inner: PurePath {
                parsed: ParsedPath::parse(&cwd.to_string_lossy(), PathFlavor::Windows),
                flavor: PathFlavor::Windows,
            },
        })
    }

    #[staticmethod]
    fn home() -> PyResult<Self> {
        let home = path_home()?;
        Ok(Self {
            inner: PurePath {
                parsed: ParsedPath::parse(&home.to_string_lossy(), PathFlavor::Windows),
                flavor: PathFlavor::Windows,
            },
        })
    }

    // ==================== Dunder methods ====================

    fn __str__(&self) -> String {
        self.inner.to_str()
    }

    fn __repr__(&self) -> String {
        format!("WindowsPath('{}')", self.inner.to_str())
    }

    fn __fspath__(&self) -> String {
        self.inner.to_str()
    }

    fn __truediv__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let other_str: String = other.extract()?;
        let other_parsed = ParsedPath::parse(&other_str, self.inner.flavor);
        Ok(Self {
            inner: PurePath {
                parsed: self.inner.parsed.join(&other_parsed, self.inner.flavor),
                flavor: self.inner.flavor,
            },
        })
    }

    fn __rtruediv__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let other_str: String = other.extract()?;
        let other_parsed = ParsedPath::parse(&other_str, self.inner.flavor);
        Ok(Self {
            inner: PurePath {
                parsed: other_parsed.join(&self.inner.parsed, self.inner.flavor),
                flavor: self.inner.flavor,
            },
        })
    }

    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        Hash::hash(self, &mut hasher);
        hasher.finish()
    }

    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }

    fn __ne__(&self, other: &Self) -> bool {
        self != other
    }

    fn __lt__(&self, other: &Self) -> PyResult<bool> {
        self.inner.compare_lt(&other.inner)
    }

    fn __le__(&self, other: &Self) -> PyResult<bool> {
        Ok(self == other || self.inner.compare_lt(&other.inner)?)
    }

    fn __gt__(&self, other: &Self) -> PyResult<bool> {
        self.inner.compare_gt(&other.inner)
    }

    fn __ge__(&self, other: &Self) -> PyResult<bool> {
        Ok(self == other || self.inner.compare_gt(&other.inner)?)
    }
}
