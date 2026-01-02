//! Path - Concrete path with filesystem operations.

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyTuple};
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::PathBuf;

use crate::pure_path::PurePath;
use crate::pure_path::flavor::PathFlavor;
use crate::pure_path::parsing::ParsedPath;

/// A concrete path that provides filesystem operations.
/// On Windows, behaves like WindowsPath.
/// On POSIX, behaves like PosixPath.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct Path {
    pub(crate) inner: PurePath,
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for Path {}

impl Hash for Path {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

// ==================== Helper functions for filesystem operations ====================

pub(crate) fn path_exists(path: &PathBuf) -> bool {
    path.exists()
}

pub(crate) fn path_is_file(path: &PathBuf) -> bool {
    path.is_file()
}

pub(crate) fn path_is_dir(path: &PathBuf) -> bool {
    path.is_dir()
}

pub(crate) fn path_is_symlink(path: &PathBuf) -> bool {
    path.is_symlink()
}

pub(crate) fn path_absolute(path: &PathBuf) -> PyResult<PathBuf> {
    if path.is_absolute() {
        Ok(path.clone())
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))
    }
}

pub(crate) fn path_resolve(path: &PathBuf, strict: bool) -> PyResult<PathBuf> {
    if strict {
        path.canonicalize()
            .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))
    } else {
        Ok(path.canonicalize().unwrap_or_else(|_| {
            std::env::current_dir()
                .map(|cwd| cwd.join(path))
                .unwrap_or_else(|_| path.clone())
        }))
    }
}

pub(crate) fn path_readlink(path: &PathBuf) -> PyResult<PathBuf> {
    fs::read_link(path).map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))
}

pub(crate) fn path_stat(path: &PathBuf) -> PyResult<StatResult> {
    let metadata =
        fs::metadata(path).map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
    Ok(StatResult::from_metadata(&metadata))
}

pub(crate) fn path_lstat(path: &PathBuf) -> PyResult<StatResult> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
    Ok(StatResult::from_metadata(&metadata))
}

pub(crate) fn path_mkdir(
    path: &PathBuf,
    _mode: u32,
    parents: bool,
    exist_ok: bool,
) -> PyResult<()> {
    let result = if parents {
        fs::create_dir_all(path)
    } else {
        fs::create_dir(path)
    };

    match result {
        Ok(()) => Ok(()),
        Err(e) if exist_ok && e.kind() == std::io::ErrorKind::AlreadyExists => {
            if path.is_dir() {
                Ok(())
            } else {
                Err(pyo3::exceptions::PyFileExistsError::new_err(e.to_string()))
            }
        }
        Err(e) => Err(pyo3::exceptions::PyOSError::new_err(e.to_string())),
    }
}

pub(crate) fn path_rmdir(path: &PathBuf) -> PyResult<()> {
    fs::remove_dir(path).map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))
}

pub(crate) fn path_unlink(path: &PathBuf, missing_ok: bool) -> PyResult<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if missing_ok && e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(pyo3::exceptions::PyOSError::new_err(e.to_string())),
    }
}

pub(crate) fn path_rename(from: &PathBuf, to: &PathBuf) -> PyResult<()> {
    fs::rename(from, to).map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))
}

pub(crate) fn path_touch(path: &PathBuf, exist_ok: bool) -> PyResult<()> {
    if path.exists() {
        if !exist_ok {
            return Err(pyo3::exceptions::PyFileExistsError::new_err(
                "File already exists",
            ));
        }
        fs::OpenOptions::new()
            .write(true)
            .open(path)
            .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
    } else {
        fs::File::create(path).map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
    }
    Ok(())
}

pub(crate) fn path_read_text(path: &PathBuf, encoding: Option<&str>) -> PyResult<String> {
    if encoding.is_some_and(|e| e.to_lowercase() != "utf-8" && e.to_lowercase() != "utf8") {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "Only UTF-8 encoding is supported",
        ));
    }
    fs::read_to_string(path).map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))
}

pub(crate) fn path_write_text(
    path: &PathBuf,
    data: &str,
    encoding: Option<&str>,
) -> PyResult<usize> {
    if encoding.is_some_and(|e| e.to_lowercase() != "utf-8" && e.to_lowercase() != "utf8") {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "Only UTF-8 encoding is supported",
        ));
    }
    let mut file =
        fs::File::create(path).map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
    file.write_all(data.as_bytes())
        .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
    Ok(data.len())
}

pub(crate) fn path_read_bytes(path: &PathBuf) -> PyResult<Vec<u8>> {
    fs::read(path).map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))
}

pub(crate) fn path_write_bytes(path: &PathBuf, data: &[u8]) -> PyResult<usize> {
    let mut file =
        fs::File::create(path).map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
    file.write_all(data)
        .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
    Ok(data.len())
}

pub(crate) fn path_iterdir(path: &PathBuf) -> PyResult<Vec<PathBuf>> {
    let entries =
        fs::read_dir(path).map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;

    let mut result = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
        result.push(entry.path());
    }
    Ok(result)
}

pub(crate) fn path_glob(base: &PathBuf, pattern: &str) -> PyResult<Vec<PathBuf>> {
    let full_pattern = base.join(pattern);
    let pattern_str = full_pattern.to_string_lossy();

    let mut result = Vec::new();
    for entry in glob::glob(&pattern_str)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?
    {
        match entry {
            Ok(path) => result.push(path),
            Err(e) => return Err(pyo3::exceptions::PyOSError::new_err(e.to_string())),
        }
    }
    Ok(result)
}

pub(crate) fn path_rglob(base: &PathBuf, pattern: &str) -> PyResult<Vec<PathBuf>> {
    let full_pattern = base.join("**").join(pattern);
    let pattern_str = full_pattern.to_string_lossy();

    let mut result = Vec::new();
    for entry in glob::glob(&pattern_str)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?
    {
        match entry {
            Ok(path) => result.push(path),
            Err(e) => return Err(pyo3::exceptions::PyOSError::new_err(e.to_string())),
        }
    }
    Ok(result)
}

pub(crate) fn path_cwd() -> PyResult<PathBuf> {
    std::env::current_dir().map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))
}

pub(crate) fn path_home() -> PyResult<PathBuf> {
    dirs::home_dir().ok_or_else(|| {
        pyo3::exceptions::PyRuntimeError::new_err("Could not determine home directory")
    })
}

// ==================== Path implementation ====================

impl Path {
    /// Create from PurePath.
    pub fn from_pure(inner: PurePath) -> Self {
        Self { inner }
    }

    /// Get the underlying PathBuf for filesystem operations.
    pub(crate) fn to_pathbuf(&self) -> PathBuf {
        PathBuf::from(self.inner.to_str())
    }

    /// Create from args with a specific flavor.
    pub fn from_args_with_flavor(args: &Bound<'_, PyTuple>, flavor: PathFlavor) -> PyResult<Self> {
        let inner = PurePath::from_args_with_flavor(args, flavor)?;
        Ok(Self { inner })
    }
}

#[pymethods]
impl Path {
    #[new]
    #[pyo3(signature = (*args))]
    fn new(args: &Bound<'_, PyTuple>) -> PyResult<Self> {
        let flavor = PathFlavor::current();
        Self::from_args_with_flavor(args, flavor)
    }

    // ==================== Properties from PurePath ====================

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

    fn is_relative_to(&self, other: &Self) -> bool {
        self.inner.get_is_relative_to(&other.inner)
    }

    #[pyo3(signature = (other, walk_up=false))]
    fn relative_to(&self, other: &Self, walk_up: bool) -> PyResult<Self> {
        Ok(Self {
            inner: self.inner.compute_relative_to(&other.inner, walk_up)?,
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
        let flavor = PathFlavor::current();
        Ok(Self {
            inner: PurePath {
                parsed: ParsedPath::parse(&cwd.to_string_lossy(), flavor),
                flavor,
            },
        })
    }

    #[staticmethod]
    fn home() -> PyResult<Self> {
        let home = path_home()?;
        let flavor = PathFlavor::current();
        Ok(Self {
            inner: PurePath {
                parsed: ParsedPath::parse(&home.to_string_lossy(), flavor),
                flavor,
            },
        })
    }

    // ==================== Dunder methods ====================

    fn __str__(&self) -> String {
        self.inner.to_str()
    }

    fn __repr__(&self) -> String {
        format!("Path('{}')", self.inner.to_str())
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

/// Stat result object.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct StatResult {
    #[pyo3(get)]
    pub st_mode: u32,
    #[pyo3(get)]
    pub st_size: u64,
    #[pyo3(get)]
    pub st_mtime: f64,
    #[pyo3(get)]
    pub st_atime: f64,
    #[pyo3(get)]
    pub st_ctime: f64,
}

impl StatResult {
    pub(crate) fn from_metadata(metadata: &fs::Metadata) -> Self {
        use std::time::UNIX_EPOCH;

        let st_mode = {
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                metadata.mode()
            }
            #[cfg(not(unix))]
            {
                if metadata.is_dir() {
                    0o40755
                } else if metadata.is_file() {
                    0o100644
                } else {
                    0
                }
            }
        };

        let st_size = metadata.len();

        let st_mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        let st_atime = metadata
            .accessed()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        let st_ctime = metadata
            .created()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        Self {
            st_mode,
            st_size,
            st_mtime,
            st_atime,
            st_ctime,
        }
    }
}

#[pymethods]
impl StatResult {
    fn __repr__(&self) -> String {
        format!(
            "os.stat_result(st_mode={}, st_size={}, st_mtime={}, st_atime={}, st_ctime={})",
            self.st_mode, self.st_size, self.st_mtime, self.st_atime, self.st_ctime
        )
    }
}
