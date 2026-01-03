//! Path - Concrete path with filesystem operations.

use crate::flavor::PathFlavor;
use crate::parsing::ParsedPath;
use crate::pure_path::PurePath;
use crate::stats::StatResult;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyTuple};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

macro_rules! impl_from_pure_path {
    ($type:ty) => {
        impl $type {
            pub fn from_pure_path(pure: PurePath) -> Self {
                Self { inner: pure }
            }
        }

        impl crate::glob_iter::FromPurePath for $type {
            fn from_pure_path(pure: PurePath) -> Self {
                Self { inner: pure }
            }
        }
    };
}

/// A pure path with POSIX semantics (forward slashes, case-sensitive).
#[pyclass(frozen)]
pub struct PurePosixPath {
    pub(crate) inner: PurePath,
}

impl_path_wrapper_traits!(PurePosixPath);
impl_pure_path_methods!(PurePosixPath, PathFlavor::Posix, "PurePosixPath");

/// A pure path with Windows semantics (backslashes, case-insensitive).
#[pyclass(frozen)]
pub struct PureWindowsPath {
    pub(crate) inner: PurePath,
}

impl_path_wrapper_traits!(PureWindowsPath);
impl_pure_path_methods!(PureWindowsPath, PathFlavor::Windows, "PureWindowsPath");

/// A concrete path that provides filesystem operations.
/// On Windows, behaves like WindowsPath.
/// On POSIX, behaves like PosixPath.
#[pyclass(frozen)]
pub struct Path {
    pub(crate) inner: PurePath,
}

impl_from_pure_path!(Path);
impl_path_wrapper_traits!(Path);
impl_concrete_path_methods!(Path, PathFlavor::current(), "Path", PathGlobIterator);
impl_glob_iterator!(PathGlobIterator, Path);

/// A POSIX path with filesystem operations.
#[pyclass(frozen)]
pub struct PosixPath {
    pub(crate) inner: PurePath,
}

impl_from_pure_path!(PosixPath);
impl_path_wrapper_traits!(PosixPath);
impl_concrete_path_methods!(
    PosixPath,
    PathFlavor::Posix,
    "PosixPath",
    PosixPathGlobIterator
);
impl_glob_iterator!(PosixPathGlobIterator, PosixPath);

/// A Windows path with filesystem operations.
#[pyclass(frozen)]
pub struct WindowsPath {
    pub(crate) inner: PurePath,
}

impl_from_pure_path!(WindowsPath);
impl_path_wrapper_traits!(WindowsPath);
impl_concrete_path_methods!(
    WindowsPath,
    PathFlavor::Windows,
    "WindowsPath",
    WindowsPathGlobIterator
);
impl_glob_iterator!(WindowsPathGlobIterator, WindowsPath);
