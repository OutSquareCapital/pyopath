//! Path - Concrete path with filesystem operations.

use crate::flavor::PathFlavor;
use crate::parsing::ParsedPath;
use crate::pure_path::PurePath;
use crate::stats::StatResult;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyTuple};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

/// A pure path with POSIX semantics (forward slashes, case-sensitive).
#[pyclass(frozen)]
#[derive(Clone)]
pub struct PurePosixPath {
    pub(crate) inner: PurePath,
}

impl_path_wrapper_traits!(PurePosixPath);
impl_pure_path_methods!(PurePosixPath, PathFlavor::Posix, "PurePosixPath");

/// A pure path with Windows semantics (backslashes, case-insensitive).
#[pyclass(frozen)]
#[derive(Clone)]
pub struct PureWindowsPath {
    pub(crate) inner: PurePath,
}

impl_path_wrapper_traits!(PureWindowsPath);
impl_pure_path_methods!(PureWindowsPath, PathFlavor::Windows, "PureWindowsPath");

/// A concrete path that provides filesystem operations.
/// On Windows, behaves like WindowsPath.
/// On POSIX, behaves like PosixPath.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct Path {
    pub(crate) inner: PurePath,
}

impl_path_wrapper_traits!(Path);
impl_concrete_path_methods!(Path, PathFlavor::current(), "Path");

/// A POSIX path with filesystem operations.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct PosixPath {
    pub(crate) inner: PurePath,
}

impl_path_wrapper_traits!(PosixPath);
impl_concrete_path_methods!(PosixPath, PathFlavor::Posix, "PosixPath");

/// A Windows path with filesystem operations.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct WindowsPath {
    pub(crate) inner: PurePath,
}

impl_path_wrapper_traits!(WindowsPath);
impl_concrete_path_methods!(WindowsPath, PathFlavor::Windows, "WindowsPath");
