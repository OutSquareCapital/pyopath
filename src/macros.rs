//! Macros for reducing code duplication across path types.

/// Format a string for repr() in Python style (using single quotes, escaping as needed)
pub(crate) fn python_repr_string(s: &str) -> String {
    // Python uses single quotes by default, unless the string contains single quotes but no double quotes
    if s.contains('\'') && !s.contains('"') {
        format!("{:?}", s) // Use double quotes if single quote is present
    } else {
        // Use single quotes, manually escape backslashes and single quotes
        let escaped = s.replace('\\', "\\\\").replace('\'', "\\'");
        format!("'{}'", escaped)
    }
}

/// Macro to generate a PyClass wrapper for GlobIterator for a specific path type
#[macro_export]
macro_rules! impl_glob_iterator {
    ($iter_name:ident, $path_type:ty) => {
        #[pyclass]
        pub struct $iter_name {
            inner: crate::glob_iter::GlobIteratorInner<$path_type>,
        }

        #[pymethods]
        impl $iter_name {
            fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
                slf
            }

            fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<Option<$path_type>> {
                slf.inner.next_path()
            }
        }
    };
}

/// Generate common pure path Python methods (getters and methods).
/// This macro generates all the #[pymethods] for pure path types.
#[macro_export]
macro_rules! impl_pure_path_methods {
    ($type:ty, $flavor:expr, $repr_name:literal) => {
        #[pymethods]
        impl $type {
            #[new]
            #[pyo3(signature = (*args))]
            fn new(args: &Bound<'_, PyTuple>) -> PyResult<Self> {
                let inner = PurePath::from_args_with_flavor(args, $flavor)?;
                Ok(Self { inner })
            }

            #[getter]
            fn drive(&self) -> &str {
                &self.inner.parsed.drive
            }

            #[getter]
            fn root(&self) -> &str {
                &self.inner.parsed.root
            }

            #[getter]
            fn anchor(&self) -> String {
                self.inner.parsed.anchor()
            }

            #[getter]
            fn parts<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyTuple>> {
                let parts = self.inner.get_parts();
                Ok(pyo3::types::PyTuple::new(
                    py,
                    parts.iter().map(|s| s.as_str()),
                )?)
            }

            #[getter]
            fn name(&self) -> &str {
                self.inner.parsed.name()
            }

            #[getter]
            fn suffix(&self) -> String {
                self.inner.parsed.suffix()
            }

            #[getter]
            fn suffixes(&self) -> Vec<String> {
                self.inner.parsed.suffixes()
            }

            #[getter]
            fn stem(&self) -> String {
                self.inner.parsed.stem()
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
                let other_path = if let Ok(p) = other.cast::<Self>() {
                    PurePath::new_with_parsed(
                        p.borrow().inner.parsed.clone(),
                        p.borrow().inner.flavor,
                    )
                } else {
                    let s: String = other.extract()?;
                    PurePath::new_with_parsed(
                        ParsedPath::parse(&s, self.inner.flavor),
                        self.inner.flavor,
                    )
                };
                Ok(self.inner.get_is_relative_to(&other_path))
            }

            #[pyo3(signature = (other, walk_up=false))]
            fn relative_to(&self, other: &Bound<'_, PyAny>, walk_up: bool) -> PyResult<Self> {
                let other_path = if let Ok(p) = other.cast::<Self>() {
                    PurePath::new_with_parsed(
                        p.borrow().inner.parsed.clone(),
                        p.borrow().inner.flavor,
                    )
                } else {
                    let s: String = other.extract()?;
                    PurePath::new_with_parsed(
                        ParsedPath::parse(&s, self.inner.flavor),
                        self.inner.flavor,
                    )
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

            fn __str__(&self) -> String {
                self.inner.to_str()
            }

            fn __repr__(&self) -> String {
                format!(
                    concat!($repr_name, "({})"),
                    crate::macros::python_repr_string(&self.inner.to_str())
                )
            }

            fn __fspath__(&self) -> String {
                self.inner.to_str()
            }

            fn __truediv__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
                let other_str: String = other.extract()?;
                Ok(Self {
                    inner: PurePath::new_with_parsed(
                        self.inner
                            .parsed
                            .join(&ParsedPath::parse(&other_str, $flavor), $flavor),
                        $flavor,
                    ),
                })
            }

            fn __rtruediv__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
                let other_str: String = other.extract()?;
                let other_parsed = ParsedPath::parse(&other_str, $flavor);
                Ok(Self {
                    inner: PurePath::new_with_parsed(
                        other_parsed.join(&self.inner.parsed, $flavor),
                        $flavor,
                    ),
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
    };
}

/// Generate filesystem methods for concrete path types.
#[macro_export]
macro_rules! impl_concrete_path_methods {
    ($type:ty, $flavor:expr, $repr_name:literal, $glob_iter:ident) => {
        impl $type {
            fn to_pathbuf(&self) -> PathBuf {
                PathBuf::from(self.inner.to_str())
            }
        }

        #[pymethods]
        impl $type {
            #[new]
            #[pyo3(signature = (*args))]
            fn new(args: &Bound<'_, PyTuple>) -> PyResult<Self> {
                if $flavor != crate::flavor::PathFlavor::current() {
                    return Err(pyo3::exceptions::PyNotImplementedError::new_err(format!(
                        "cannot instantiate '{}' on your system",
                        $repr_name
                    )));
                }
                let inner = PurePath::from_args_with_flavor(args, $flavor)?;
                Ok(Self { inner })
            }

            // ==================== Properties from PurePath ====================

            #[getter]
            fn drive(&self) -> &str {
                &self.inner.parsed.drive
            }

            #[getter]
            fn root(&self) -> &str {
                &self.inner.parsed.root
            }

            #[getter]
            fn anchor(&self) -> String {
                self.inner.parsed.anchor()
            }

            #[getter]
            fn parts<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyTuple>> {
                let parts = self.inner.get_parts();
                Ok(pyo3::types::PyTuple::new(
                    py,
                    parts.iter().map(|s| s.as_str()),
                )?)
            }

            #[getter]
            fn name(&self) -> &str {
                self.inner.parsed.name()
            }

            #[getter]
            fn suffix(&self) -> String {
                self.inner.parsed.suffix()
            }

            #[getter]
            fn suffixes(&self) -> Vec<String> {
                self.inner.parsed.suffixes()
            }

            #[getter]
            fn stem(&self) -> String {
                self.inner.parsed.stem()
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
                let other_path = if let Ok(p) = other.cast::<Self>() {
                    PurePath::new_with_parsed(
                        p.borrow().inner.parsed.clone(),
                        p.borrow().inner.flavor,
                    )
                } else {
                    let s: String = other.extract()?;
                    PurePath::new_with_parsed(
                        ParsedPath::parse(&s, self.inner.flavor),
                        self.inner.flavor,
                    )
                };
                Ok(self.inner.get_is_relative_to(&other_path))
            }

            #[pyo3(signature = (other, walk_up=false))]
            fn relative_to(&self, other: &Bound<'_, PyAny>, walk_up: bool) -> PyResult<Self> {
                let other_path = if let Ok(p) = other.cast::<Self>() {
                    PurePath::new_with_parsed(
                        p.borrow().inner.parsed.clone(),
                        p.borrow().inner.flavor,
                    )
                } else {
                    let s: String = other.extract()?;
                    PurePath::new_with_parsed(
                        ParsedPath::parse(&s, self.inner.flavor),
                        self.inner.flavor,
                    )
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
                self.to_pathbuf().exists()
            }

            fn is_file(&self) -> bool {
                self.to_pathbuf().is_file()
            }

            fn is_dir(&self) -> bool {
                self.to_pathbuf().is_dir()
            }

            fn is_symlink(&self) -> bool {
                self.to_pathbuf().is_symlink()
            }

            fn absolute(&self) -> PyResult<Self> {
                let path = self.to_pathbuf();
                let abs = if path.is_absolute() {
                    path
                } else {
                    std::env::current_dir()
                        .map(|cwd| cwd.join(&path))
                        .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?
                };
                Ok(Self {
                    inner: PurePath::new_with_parsed(
                        ParsedPath::parse(&abs.to_string_lossy(), self.inner.flavor),
                        self.inner.flavor,
                    ),
                })
            }

            #[pyo3(signature = (strict=false))]
            fn resolve(&self, strict: bool) -> PyResult<Self> {
                let path = self.to_pathbuf();
                let resolved = if strict {
                    path.canonicalize()
                        .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?
                } else {
                    path.canonicalize().unwrap_or_else(|_| {
                        std::env::current_dir()
                            .map(|cwd| cwd.join(&path))
                            .unwrap_or(path)
                    })
                };
                Ok(Self {
                    inner: PurePath::new_with_parsed(
                        ParsedPath::parse(&resolved.to_string_lossy(), self.inner.flavor),
                        self.inner.flavor,
                    ),
                })
            }

            fn readlink(&self) -> PyResult<Self> {
                let target = std::fs::read_link(self.to_pathbuf())
                    .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
                Ok(Self {
                    inner: PurePath::new_with_parsed(
                        ParsedPath::parse(&target.to_string_lossy(), self.inner.flavor),
                        self.inner.flavor,
                    ),
                })
            }

            fn stat(&self) -> PyResult<StatResult> {
                let metadata = std::fs::metadata(self.to_pathbuf())
                    .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
                Ok(StatResult::from_metadata(&metadata))
            }

            fn lstat(&self) -> PyResult<StatResult> {
                let metadata = std::fs::symlink_metadata(self.to_pathbuf())
                    .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
                Ok(StatResult::from_metadata(&metadata))
            }

            #[pyo3(signature = (_mode=0o777, parents=false, exist_ok=false))]
            fn mkdir(&self, _mode: u32, parents: bool, exist_ok: bool) -> PyResult<()> {
                let path = self.to_pathbuf();
                let result = if parents {
                    std::fs::create_dir_all(&path)
                } else {
                    std::fs::create_dir(&path)
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

            fn rmdir(&self) -> PyResult<()> {
                std::fs::remove_dir(self.to_pathbuf())
                    .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))
            }

            #[pyo3(signature = (missing_ok=false))]
            fn unlink(&self, missing_ok: bool) -> PyResult<()> {
                match std::fs::remove_file(self.to_pathbuf()) {
                    Ok(()) => Ok(()),
                    Err(e) if missing_ok && e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                    Err(e) => Err(pyo3::exceptions::PyOSError::new_err(e.to_string())),
                }
            }

            fn rename(&self, target: &Self) -> PyResult<Self> {
                std::fs::rename(self.to_pathbuf(), target.to_pathbuf())
                    .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
                Ok(Self {
                    inner: PurePath::new_with_parsed(
                        target.inner.parsed.clone(),
                        target.inner.flavor,
                    ),
                })
            }

            fn replace(&self, target: &Self) -> PyResult<Self> {
                std::fs::rename(self.to_pathbuf(), target.to_pathbuf())
                    .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
                Ok(Self {
                    inner: PurePath::new_with_parsed(
                        target.inner.parsed.clone(),
                        target.inner.flavor,
                    ),
                })
            }

            #[pyo3(signature = (exist_ok=true))]
            fn touch(&self, exist_ok: bool) -> PyResult<()> {
                let path = self.to_pathbuf();
                if path.exists() {
                    if !exist_ok {
                        return Err(pyo3::exceptions::PyFileExistsError::new_err(
                            "File already exists",
                        ));
                    }
                    std::fs::OpenOptions::new()
                        .write(true)
                        .open(&path)
                        .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
                } else {
                    std::fs::File::create(&path)
                        .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
                }
                Ok(())
            }

            #[pyo3(signature = (encoding=None))]
            fn read_text(&self, encoding: Option<&str>) -> PyResult<String> {
                if encoding
                    .is_some_and(|e| e.to_lowercase() != "utf-8" && e.to_lowercase() != "utf8")
                {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "Only UTF-8 encoding is supported",
                    ));
                }
                std::fs::read_to_string(self.to_pathbuf())
                    .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))
            }

            #[pyo3(signature = (data, encoding=None))]
            fn write_text(&self, data: &str, encoding: Option<&str>) -> PyResult<usize> {
                use std::io::Write;
                if encoding
                    .is_some_and(|e| e.to_lowercase() != "utf-8" && e.to_lowercase() != "utf8")
                {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "Only UTF-8 encoding is supported",
                    ));
                }
                let mut file = std::fs::File::create(self.to_pathbuf())
                    .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
                file.write_all(data.as_bytes())
                    .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
                Ok(data.len())
            }

            fn read_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
                let data = std::fs::read(self.to_pathbuf())
                    .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
                Ok(PyBytes::new(py, &data))
            }

            fn write_bytes(&self, data: &[u8]) -> PyResult<usize> {
                use std::io::Write;
                let mut file = std::fs::File::create(self.to_pathbuf())
                    .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
                file.write_all(data)
                    .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
                Ok(data.len())
            }

            fn iterdir(&self) -> PyResult<Vec<Self>> {
                let entries = std::fs::read_dir(self.to_pathbuf())
                    .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
                let mut result = Vec::new();
                for entry in entries {
                    let entry =
                        entry.map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
                    result.push(Self {
                        inner: PurePath::new_with_parsed(
                            ParsedPath::parse(&entry.path().to_string_lossy(), self.inner.flavor),
                            self.inner.flavor,
                        ),
                    });
                }
                Ok(result)
            }

            #[pyo3(signature = (pattern, *, case_sensitive=None, follow_symlinks=None))]
            fn glob(
                &self,
                pattern: &str,
                case_sensitive: Option<bool>,
                follow_symlinks: Option<bool>,
            ) -> PyResult<$glob_iter> {
                let base = self.to_pathbuf();

                let inner = crate::glob_iter::GlobIteratorInner::new(
                    base,
                    pattern,
                    self.inner.flavor,
                    case_sensitive,
                    follow_symlinks,
                )?;

                Ok($glob_iter { inner })
            }

            #[pyo3(signature = (pattern, *, case_sensitive=None, follow_symlinks=None))]
            fn rglob(
                &self,
                pattern: &str,
                case_sensitive: Option<bool>,
                follow_symlinks: Option<bool>,
            ) -> PyResult<$glob_iter> {
                let base = self.to_pathbuf();

                let inner = crate::glob_iter::GlobIteratorInner::new(
                    base,
                    &format!("**/{}", pattern),
                    self.inner.flavor,
                    case_sensitive,
                    follow_symlinks,
                )?;

                Ok($glob_iter { inner })
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
                let cwd = std::env::current_dir()
                    .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?;
                Ok(Self {
                    inner: PurePath::new_with_parsed(
                        ParsedPath::parse(&cwd.to_string_lossy(), $flavor),
                        $flavor,
                    ),
                })
            }

            #[staticmethod]
            fn home() -> PyResult<Self> {
                let home = dirs::home_dir().ok_or_else(|| {
                    pyo3::exceptions::PyRuntimeError::new_err("Could not determine home directory")
                })?;
                Ok(Self {
                    inner: PurePath::new_with_parsed(
                        ParsedPath::parse(&home.to_string_lossy(), $flavor),
                        $flavor,
                    ),
                })
            }

            // ==================== Dunder methods ====================

            fn __str__(&self) -> String {
                self.inner.to_str()
            }

            fn __repr__(&self) -> String {
                format!(
                    concat!($repr_name, "({})"),
                    crate::macros::python_repr_string(&self.inner.to_str())
                )
            }

            fn __fspath__(&self) -> String {
                self.inner.to_str()
            }

            fn __truediv__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
                let other_str: String = other.extract()?;
                let other_parsed = ParsedPath::parse(&other_str, self.inner.flavor);
                Ok(Self {
                    inner: PurePath::new_with_parsed(
                        self.inner.parsed.join(&other_parsed, self.inner.flavor),
                        self.inner.flavor,
                    ),
                })
            }

            fn __rtruediv__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
                let other_str: String = other.extract()?;
                let other_parsed = ParsedPath::parse(&other_str, self.inner.flavor);
                Ok(Self {
                    inner: PurePath::new_with_parsed(
                        other_parsed.join(&self.inner.parsed, self.inner.flavor),
                        self.inner.flavor,
                    ),
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
    };
}

/// Generate common trait implementations for path wrapper types.
#[macro_export]
macro_rules! impl_path_wrapper_traits {
    ($type:ty) => {
        impl PartialEq for $type {
            fn eq(&self, other: &Self) -> bool {
                self.inner == other.inner
            }
        }

        impl Eq for $type {}

        impl Hash for $type {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.inner.hash(state);
            }
        }
    };
}
