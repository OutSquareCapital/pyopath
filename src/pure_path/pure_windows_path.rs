//! PureWindowsPath - Pure path with Windows semantics.

use pyo3::prelude::*;
use pyo3::types::PyTuple;
use std::hash::{Hash, Hasher};

use super::flavor::PathFlavor;
use super::parsing::ParsedPath;
use super::pure_path::PurePath;

/// A pure path with Windows semantics (backslashes, case-insensitive).
#[pyclass(frozen)]
#[derive(Clone)]
pub struct PureWindowsPath {
    pub(crate) inner: PurePath,
}

impl PartialEq for PureWindowsPath {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for PureWindowsPath {}

impl Hash for PureWindowsPath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

#[pymethods]
impl PureWindowsPath {
    #[new]
    #[pyo3(signature = (*args))]
    fn new(args: &Bound<'_, PyTuple>) -> PyResult<Self> {
        let inner = PurePath::from_args_with_flavor(args, PathFlavor::Windows)?;
        Ok(Self { inner })
    }

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

    fn __str__(&self) -> String {
        self.inner.to_str()
    }

    fn __repr__(&self) -> String {
        format!("PureWindowsPath('{}')", self.inner.to_str())
    }

    fn __fspath__(&self) -> String {
        self.inner.to_str()
    }

    fn __truediv__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let other_str: String = other.extract()?;
        Ok(Self {
            inner: PurePath {
                parsed: self.inner.parsed.join(&ParsedPath::parse(&other_str, PathFlavor::Windows), PathFlavor::Windows),
                flavor: PathFlavor::Windows,
            },
        })
    }

    fn __rtruediv__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let other_str: String = other.extract()?;
        let other_parsed = ParsedPath::parse(&other_str, PathFlavor::Windows);
        Ok(Self {
            inner: PurePath {
                parsed: other_parsed.join(&self.inner.parsed, PathFlavor::Windows),
                flavor: PathFlavor::Windows,
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
