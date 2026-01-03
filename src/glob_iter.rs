//! Glob iterator for lazy path yielding.

use crate::flavor::PathFlavor;
use crate::pure_path::PurePath;
use pyo3::prelude::*;

/// Trait for types that can be created from a PurePath
pub trait FromPurePath {
    fn from_pure_path(pure: PurePath) -> Self;
}

/// Internal generic iterator for glob results
/// T must implement FromPurePath
pub struct GlobIteratorInner<T: FromPurePath> {
    /// The glob iterator from the glob crate
    glob_iter: glob::Paths,
    /// The flavor to use for parsing paths
    flavor: PathFlavor,
    /// Phantom data for the path type
    _marker: std::marker::PhantomData<T>,
}

impl<T: FromPurePath> GlobIteratorInner<T> {
    pub fn new(pattern: &str, flavor: PathFlavor) -> PyResult<Self> {
        let glob_iter = glob::glob(pattern)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        Ok(Self {
            glob_iter,
            flavor,
            _marker: std::marker::PhantomData,
        })
    }

    /// Get next path from the iterator
    pub fn next_path(&mut self) -> PyResult<Option<T>> {
        match self.glob_iter.next() {
            Some(Ok(path)) => {
                let path_str = path.to_string_lossy();
                let parsed = crate::parsing::ParsedPath::parse(&path_str, self.flavor);
                let pure_path = PurePath::new_with_parsed(parsed, self.flavor);
                Ok(Some(T::from_pure_path(pure_path)))
            }
            Some(Err(e)) => Err(pyo3::exceptions::PyOSError::new_err(e.to_string())),
            None => Ok(None),
        }
    }
}
