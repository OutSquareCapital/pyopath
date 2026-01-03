//! PurePath - Generic pure path (platform-aware).

use pyo3::prelude::*;
use pyo3::types::PyTuple;
use std::hash::{Hash, Hasher};

use super::flavor::PathFlavor;
use super::parsing::ParsedPath;

/// A generic pure path that uses the current platform's flavor.
/// On Windows, behaves like PureWindowsPath.
/// On POSIX, behaves like PurePosixPath.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct PurePath {
    pub(crate) parsed: ParsedPath,
    pub(crate) flavor: PathFlavor,
}

impl PartialEq for PurePath {
    fn eq(&self, other: &Self) -> bool {
        if self.flavor != other.flavor {
            return false;
        }
        let self_folded = self.parsed.case_fold(self.flavor);
        let other_folded = other.parsed.case_fold(other.flavor);
        self_folded.drive == other_folded.drive
            && self_folded.root == other_folded.root
            && self_folded.parts == other_folded.parts
    }
}

impl Eq for PurePath {}

impl Hash for PurePath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let folded = self.parsed.case_fold(self.flavor);
        folded.drive.hash(state);
        folded.root.hash(state);
        folded.parts.hash(state);
    }
}

impl PurePath {
    /// Create from args with a specific flavor.
    pub fn from_args_with_flavor(args: &Bound<'_, PyTuple>, flavor: PathFlavor) -> PyResult<Self> {
        if args.is_empty() {
            return Ok(Self {
                parsed: ParsedPath::parse("", flavor),
                flavor,
            });
        }

        let mut result = ParsedPath::parse("", flavor);

        for arg in args.iter() {
            let path_str: String = arg.extract()?;
            let other = ParsedPath::parse(&path_str, flavor);
            result = result.join(&other, flavor);
        }

        Ok(Self {
            parsed: result,
            flavor,
        })
    }

    /// Get drive (public for reuse).
    pub fn get_drive(&self) -> &str {
        &self.parsed.drive
    }

    /// Get root (public for reuse).
    pub fn get_root(&self) -> &str {
        &self.parsed.root
    }

    /// Get anchor (public for reuse).
    pub fn get_anchor(&self) -> String {
        self.parsed.anchor()
    }

    /// Get parts (public for reuse).
    pub fn get_parts(&self) -> Vec<String> {
        self.parsed.all_parts(self.flavor)
    }

    /// Get name (public for reuse).
    pub fn get_name(&self) -> &str {
        self.parsed.name()
    }

    /// Get suffix (public for reuse).
    pub fn get_suffix(&self) -> String {
        self.parsed.suffix()
    }

    /// Get suffixes (public for reuse).
    pub fn get_suffixes(&self) -> Vec<String> {
        self.parsed.suffixes()
    }

    /// Get stem (public for reuse).
    pub fn get_stem(&self) -> String {
        self.parsed.stem()
    }

    /// Get parent (public for reuse).
    pub fn get_parent(&self) -> Self {
        Self {
            parsed: self.parsed.parent(),
            flavor: self.flavor,
        }
    }

    /// Get parents (public for reuse).
    pub fn get_parents(&self) -> Vec<Self> {
        let mut result = Vec::new();
        let mut current = self.parsed.parent();

        loop {
            let parent_path = Self {
                parsed: current.clone(),
                flavor: self.flavor,
            };

            // Stop if we've reached empty path (no drive, no root, no parts)
            if current.parts.is_empty() && current.root.is_empty() && current.drive.is_empty() {
                break;
            }

            result.push(parent_path);

            // If we're at root (has root but no parts), stop after adding it
            if current.parts.is_empty() && !current.root.is_empty() {
                break;
            }

            let next = current.parent();
            if next.parts == current.parts
                && next.root == current.root
                && next.drive == current.drive
            {
                break;
            }
            current = next;
        }

        result
    }

    /// Check if absolute (public for reuse).
    pub fn get_is_absolute(&self) -> bool {
        self.parsed.is_absolute(self.flavor)
    }

    /// Check if relative to other (public for reuse).
    pub fn get_is_relative_to(&self, other: &Self) -> bool {
        if self.flavor != other.flavor {
            return false;
        }

        let self_folded = self.parsed.case_fold(self.flavor);
        let other_folded = other.parsed.case_fold(other.flavor);

        if self_folded.drive != other_folded.drive || self_folded.root != other_folded.root {
            return false;
        }

        if other_folded.parts.len() > self_folded.parts.len() {
            return false;
        }

        self_folded.parts.starts_with(&other_folded.parts)
    }

    /// Get string representation (public for reuse).
    pub fn to_str(&self) -> String {
        self.parsed.to_string(self.flavor)
    }

    /// Get as_posix representation (public for reuse).
    pub fn get_as_posix(&self) -> String {
        self.parsed.to_string(self.flavor).replace('\\', "/")
    }

    /// Compute relative_to (public for reuse).
    pub fn compute_relative_to(&self, other: &Self, walk_up: bool) -> PyResult<Self> {
        if self.flavor != other.flavor {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "cannot compare paths of different flavors",
            ));
        }

        let self_folded = self.parsed.case_fold(self.flavor);
        let other_folded = other.parsed.case_fold(other.flavor);

        let self_anchor = self_folded.anchor();
        let other_anchor = other_folded.anchor();

        if self_anchor != other_anchor {
            let msg = format!(
                "'{}' is not in the subpath of '{}' OR one path is relative and the other is absolute.",
                self.to_str(),
                other.to_str()
            );
            return Err(pyo3::exceptions::PyValueError::new_err(msg));
        }

        if walk_up {
            let mut common_len = 0;
            for (a, b) in self_folded.parts.iter().zip(other_folded.parts.iter()) {
                if a == b {
                    common_len += 1;
                } else {
                    break;
                }
            }

            let ups = other_folded.parts.len() - common_len;
            let mut new_parts: Vec<String> = (0..ups).map(|_| "..".to_string()).collect();
            new_parts.extend(self.parsed.parts[common_len..].iter().cloned());

            Ok(Self {
                parsed: ParsedPath {
                    drive: String::new(),
                    root: String::new(),
                    parts: new_parts,
                    raw: String::new(),
                },
                flavor: self.flavor,
            })
        } else {
            if !self_folded.parts.starts_with(&other_folded.parts) {
                let msg = format!(
                    "'{}' is not in the subpath of '{}' OR one path is relative and the other is absolute.",
                    self.to_str(),
                    other.to_str()
                );
                return Err(pyo3::exceptions::PyValueError::new_err(msg));
            }

            let new_parts = self.parsed.parts[other_folded.parts.len()..].to_vec();

            Ok(Self {
                parsed: ParsedPath {
                    drive: String::new(),
                    root: String::new(),
                    parts: new_parts,
                    raw: String::new(),
                },
                flavor: self.flavor,
            })
        }
    }

    /// Compute joinpath (public for reuse).
    pub fn compute_joinpath(&self, args: &Bound<'_, PyTuple>) -> PyResult<Self> {
        let mut result = self.parsed.clone();

        for arg in args.iter() {
            let other_str: String = arg.extract()?;
            let other_parsed = ParsedPath::parse(&other_str, self.flavor);
            result = result.join(&other_parsed, self.flavor);
        }

        Ok(Self {
            parsed: result,
            flavor: self.flavor,
        })
    }

    /// Compute with_name (public for reuse).
    pub fn compute_with_name(&self, name: &str) -> PyResult<Self> {
        if self.parsed.name().is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "{:?} has an empty name",
                self.to_str()
            )));
        }

        if name.is_empty()
            || name.contains(self.flavor.sep())
            || self.flavor.altsep().is_some_and(|alt| name.contains(alt))
        {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid name {:?}",
                name
            )));
        }

        let mut new_parts = self.parsed.parts.clone();
        if let Some(last) = new_parts.last_mut() {
            *last = name.to_string();
        }

        Ok(Self {
            parsed: ParsedPath {
                drive: self.parsed.drive.clone(),
                root: self.parsed.root.clone(),
                parts: new_parts,
                raw: String::new(),
            },
            flavor: self.flavor,
        })
    }

    /// Compute with_stem (public for reuse).
    pub fn compute_with_stem(&self, stem: &str) -> PyResult<Self> {
        let suffix = self.parsed.suffix();
        let new_name = format!("{}{}", stem, suffix);
        self.compute_with_name(&new_name)
    }

    /// Compute with_suffix (public for reuse).
    pub fn compute_with_suffix(&self, suffix: &str) -> PyResult<Self> {
        if !suffix.is_empty() && !suffix.starts_with('.') {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid suffix {:?}",
                suffix
            )));
        }

        if suffix.len() > 1
            && (suffix[1..].contains(self.flavor.sep())
                || self
                    .flavor
                    .altsep()
                    .is_some_and(|alt| suffix[1..].contains(alt)))
        {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid suffix {:?}",
                suffix
            )));
        }

        if self.parsed.name().is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "{:?} has an empty name",
                self.to_str()
            )));
        }

        let stem = self.parsed.stem();
        let new_name = format!("{}{}", stem, suffix);
        self.compute_with_name(&new_name)
    }

    /// Compare for ordering (public for reuse).
    pub fn compare_lt(&self, other: &Self) -> PyResult<bool> {
        if self.flavor != other.flavor {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "'<' not supported between instances of different path flavors",
            ));
        }
        let self_str = self.parsed.case_fold(self.flavor).to_string(self.flavor);
        let other_str = other.parsed.case_fold(other.flavor).to_string(other.flavor);
        Ok(self_str < other_str)
    }

    /// Compare for ordering (public for reuse).
    pub fn compare_gt(&self, other: &Self) -> PyResult<bool> {
        if self.flavor != other.flavor {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "'>' not supported between instances of different path flavors",
            ));
        }
        let self_str = self.parsed.case_fold(self.flavor).to_string(self.flavor);
        let other_str = other.parsed.case_fold(other.flavor).to_string(other.flavor);
        Ok(self_str > other_str)
    }
}

#[pymethods]
impl PurePath {
    /// Create a new PurePath from path segments.
    #[new]
    #[pyo3(signature = (*args))]
    fn new(args: &Bound<'_, PyTuple>) -> PyResult<Self> {
        let flavor = PathFlavor::current();
        Self::from_args_with_flavor(args, flavor)
    }

    #[getter]
    fn drive(&self) -> &str {
        self.get_drive()
    }

    #[getter]
    fn root(&self) -> &str {
        self.get_root()
    }

    #[getter]
    fn anchor(&self) -> String {
        self.get_anchor()
    }

    #[getter]
    fn parts(&self) -> Vec<String> {
        self.get_parts()
    }

    #[getter]
    fn name(&self) -> &str {
        self.get_name()
    }

    #[getter]
    fn suffix(&self) -> String {
        self.get_suffix()
    }

    #[getter]
    fn suffixes(&self) -> Vec<String> {
        self.get_suffixes()
    }

    #[getter]
    fn stem(&self) -> String {
        self.get_stem()
    }

    #[getter]
    fn parent(&self) -> Self {
        self.get_parent()
    }

    #[getter]
    fn parents(&self) -> Vec<Self> {
        self.get_parents()
    }

    fn is_absolute(&self) -> bool {
        self.get_is_absolute()
    }

    fn is_relative_to(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let other_path = if let Ok(p) = other.extract::<Self>() {
            p
        } else {
            let s: String = other.extract()?;
            Self {
                parsed: ParsedPath::parse(&s, self.flavor),
                flavor: self.flavor,
            }
        };
        Ok(self.get_is_relative_to(&other_path))
    }

    #[pyo3(signature = (other, walk_up=false))]
    fn relative_to(&self, other: &Bound<'_, PyAny>, walk_up: bool) -> PyResult<Self> {
        let other_path = if let Ok(p) = other.extract::<Self>() {
            p
        } else {
            let s: String = other.extract()?;
            Self {
                parsed: ParsedPath::parse(&s, self.flavor),
                flavor: self.flavor,
            }
        };
        self.compute_relative_to(&other_path, walk_up)
    }

    #[pyo3(signature = (*args))]
    fn joinpath(&self, args: &Bound<'_, PyTuple>) -> PyResult<Self> {
        self.compute_joinpath(args)
    }

    fn with_name(&self, name: &str) -> PyResult<Self> {
        self.compute_with_name(name)
    }

    fn with_stem(&self, stem: &str) -> PyResult<Self> {
        self.compute_with_stem(stem)
    }

    fn with_suffix(&self, suffix: &str) -> PyResult<Self> {
        self.compute_with_suffix(suffix)
    }

    fn as_posix(&self) -> String {
        self.get_as_posix()
    }

    fn __str__(&self) -> String {
        self.to_str()
    }

    fn __repr__(&self) -> String {
        format!("PurePath('{}')", self.to_str())
    }

    fn __fspath__(&self) -> String {
        self.to_str()
    }

    fn __truediv__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let other_str: String = other.extract()?;
        let other_parsed = ParsedPath::parse(&other_str, self.flavor);
        let result = self.parsed.join(&other_parsed, self.flavor);
        Ok(Self {
            parsed: result,
            flavor: self.flavor,
        })
    }

    fn __rtruediv__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let other_str: String = other.extract()?;
        let other_parsed = ParsedPath::parse(&other_str, self.flavor);
        let result = other_parsed.join(&self.parsed, self.flavor);
        Ok(Self {
            parsed: result,
            flavor: self.flavor,
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
        self.compare_lt(other)
    }

    fn __le__(&self, other: &Self) -> PyResult<bool> {
        Ok(self == other || self.compare_lt(other)?)
    }

    fn __gt__(&self, other: &Self) -> PyResult<bool> {
        self.compare_gt(other)
    }

    fn __ge__(&self, other: &Self) -> PyResult<bool> {
        Ok(self == other || self.compare_gt(other)?)
    }
}
