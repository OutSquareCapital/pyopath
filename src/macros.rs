use crate::core::ParsedParts;
use crate::separators::{PosixSeparator, WindowsSeparator};
use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};
use std::sync::OnceLock;
macro_rules! create_pure_path_class {
    ($class_name:ident, $separator:ty, $py_name:expr) => {
        #[pyclass(frozen, name = $py_name)]
        pub struct $class_name {
            _raw_path_tuple: Vec<String>,
            str_repr_cached: OnceLock<String>,
            str_repr_original_cached: OnceLock<String>,
            parsed: OnceLock<ParsedParts>,
            _str_normcase_cached: OnceLock<String>,
            _parts_normcase_cached: OnceLock<Vec<String>>,
        }

        impl $class_name {
            fn compute_str_repr(py: Python, path_strs: &[String]) -> PyResult<(String, String)> {
                if path_strs.is_empty() {
                    return Ok((".".to_string(), ".".to_string()));
                }

                // Join all paths (os.path.join will handle it)
                let path_tuple = PyTuple::new(py, path_strs)?;
                let joined_str: String = PyModule::import(py, <$separator>::MODULE_NAME)?
                    .getattr("join")?
                    .call(path_tuple, None)?
                    .extract()?;

                // Normalize path separators for the platform
                let normalized = <$separator>::normalize_path(&joined_str);
                Ok((joined_str, normalized))
            }

            fn str_repr(&self) -> &String {
                self.str_repr_cached.get_or_init(|| {
                    Python::attach(|py| {
                        Self::compute_str_repr(py, &self._raw_path_tuple)
                            .map(|(_, normalized)| normalized)
                            .unwrap_or_else(|_| ".".to_string())
                    })
                })
            }

            fn str_repr_original(&self) -> &String {
                self.str_repr_original_cached.get_or_init(|| {
                    Python::attach(|py| {
                        Self::compute_str_repr(py, &self._raw_path_tuple)
                            .map(|(original, _)| original)
                            .unwrap_or_else(|_| ".".to_string())
                    })
                })
            }

            fn parsed_parts(&self) -> &ParsedParts {
                self.parsed
                    .get_or_init(|| <$separator>::parse(self.str_repr()))
            }

            fn str_normcase(&self) -> &String {
                self._str_normcase_cached
                    .get_or_init(|| <$separator>::normalize_case(self.str_repr()))
            }

            fn parts_normcase(&self) -> &Vec<String> {
                self._parts_normcase_cached.get_or_init(|| {
                    let sep = <$separator>::SEP;
                    self.str_normcase()
                        .split(sep)
                        .map(|s| s.to_string())
                        .collect()
                })
            }

            /// Helper to convert multiple PathLike objects to strings using os.fspath()
            fn extract_path_strs(py: Python, items: &Bound<PyTuple>) -> PyResult<Vec<String>> {
                let pyopath = PyModule::import(py, "pyopath")?;

                items
                    .iter()
                    .map(|item| {
                        let path_str: String = PyModule::import(py, "os")?
                            .getattr("fspath")?
                            .call1((&item,))?
                            .extract()?;

                        // If current separator is different from source, convert
                        let converted = if <$separator>::MODULE_NAME == "posixpath" {
                            // We're PosixPath - if source is WindowsPath, convert \ to /
                            if item.is_instance(&pyopath.getattr("PureWindowsPath")?)? {
                                path_str.replace('\\', "/")
                            } else {
                                path_str
                            }
                        } else {
                            // We're WindowsPath - if source is PosixPath, convert / to \
                            if item.is_instance(&pyopath.getattr("PurePosixPath")?)? {
                                path_str.replace('/', "\\")
                            } else {
                                path_str
                            }
                        };

                        Ok(converted)
                    })
                    .collect()
            }
            /// Create a path from already-parsed parts
            fn from_parsed_parts(parsed: ParsedParts) -> Self {
                let str_repr = <$separator>::format_parsed_parts(&parsed);
                let path = Self {
                    _raw_path_tuple: vec![],
                    str_repr_cached: OnceLock::new(),
                    str_repr_original_cached: OnceLock::new(),
                    parsed: OnceLock::new(),
                    _str_normcase_cached: OnceLock::new(),
                    _parts_normcase_cached: OnceLock::new(),
                };
                let _ = path.str_repr_cached.set(str_repr.clone());
                let _ = path.str_repr_original_cached.set(str_repr);
                let _ = path.parsed.set(parsed);
                path
            }
        }

        #[pymethods]
        impl $class_name {
            #[new]
            #[pyo3(signature = (*args))]
            fn new(py: Python, args: &Bound<PyTuple>) -> PyResult<Self> {
                let path_strs = Self::extract_path_strs(py, args)?;
                Ok(Self {
                    _raw_path_tuple: path_strs,
                    str_repr_cached: OnceLock::new(),
                    str_repr_original_cached: OnceLock::new(),
                    parsed: OnceLock::new(),
                    _str_normcase_cached: OnceLock::new(),
                    _parts_normcase_cached: OnceLock::new(),
                })
            }

            fn __str__(&self) -> String {
                self.str_repr().clone()
            }

            fn __repr__(&self) -> String {
                format!(
                    "{}('{}')",
                    stringify!($class_name),
                    self.str_repr_original()
                )
            }

            fn __eq__(&self, other: &Bound<PyAny>) -> PyResult<bool> {
                match other.extract::<Py<$class_name>>() {
                    Ok(other_py) => Python::attach(|py| {
                        Ok(self.str_normcase() == other_py.borrow(py).str_normcase())
                    }),
                    Err(_) => Ok(false),
                }
            }

            fn __hash__(&self) -> u64 {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                self.str_normcase().hash(&mut hasher);
                hasher.finish()
            }

            fn __truediv__(&self, py: Python, key: String) -> PyResult<Py<Self>> {
                let segments = vec![self.str_repr().clone(), key];
                let segments_tuple = PyTuple::new(py, &segments)?;
                self.with_segments(py, &segments_tuple)
            }

            fn __rtruediv__(&self, py: Python, key: String) -> PyResult<Py<Self>> {
                let segments = vec![key, self.str_repr().clone()];
                let segments_tuple = PyTuple::new(py, &segments)?;
                self.with_segments(py, &segments_tuple)
            }

            #[getter]
            fn drive(&self) -> String {
                self.parsed_parts().drive.clone()
            }

            #[getter]
            fn root(&self) -> String {
                self.parsed_parts().root.clone()
            }

            #[getter]
            fn anchor(&self) -> String {
                self.parsed_parts().anchor()
            }

            #[getter]
            fn parts(&self, py: Python) -> PyResult<Py<PyTuple>> {
                let parts_vec = self.parsed_parts().all_parts();
                Ok(PyTuple::new(py, parts_vec)?.into())
            }

            #[getter]
            fn _raw_path_tuple(&self) -> Vec<String> {
                self._raw_path_tuple.clone()
            }

            #[getter]
            fn _str_normcase(&self) -> String {
                self.str_normcase().clone()
            }

            #[getter]
            fn _parts_normcase(&self) -> Vec<String> {
                self.parts_normcase().clone()
            }

            #[getter]
            fn name(&self) -> String {
                self.parsed_parts().name()
            }

            #[getter]
            fn stem(&self) -> String {
                self.parsed_parts().stem()
            }

            #[getter]
            fn suffix(&self) -> String {
                self.parsed_parts().suffix()
            }

            #[getter]
            fn suffixes(&self) -> Vec<String> {
                self.parsed_parts().suffixes()
            }

            #[getter]
            fn parent(&self, py: Python) -> PyResult<Py<Self>> {
                let parsed = self.parsed_parts();
                let parent_parts = parsed.parent_parts();

                let parent_parsed = ParsedParts {
                    drive: parsed.drive.clone(),
                    root: parsed.root.clone(),
                    parts: parent_parts,
                };

                Py::new(py, Self::from_parsed_parts(parent_parsed))
            }

            fn as_posix(&self) -> String {
                self.str_repr().replace('\\', "/")
            }

            fn is_absolute(&self) -> bool {
                <$separator>::is_absolute(self.parsed_parts())
            }

            #[pyo3(signature = (*pathsegments))]
            fn with_segments(
                &self,
                py: Python,
                pathsegments: &Bound<PyTuple>,
            ) -> PyResult<Py<Self>> {
                Py::new(py, Self::new(py, pathsegments)?)
            }

            #[pyo3(signature = (*paths))]
            fn joinpath(&self, py: Python, paths: &Bound<PyTuple>) -> PyResult<Py<Self>> {
                // with_segments(self, *paths)
                let mut segments = vec![self.str_repr().clone()];
                segments.extend(Self::extract_path_strs(py, paths)?);

                let segments_tuple = PyTuple::new(py, &segments)?;
                self.with_segments(py, &segments_tuple)
            }

            #[getter]
            fn parents<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
                let parsed = self.parsed_parts();

                // Build all parent paths
                let mut parent_objs: Vec<Py<Self>> = Vec::new();
                let mut current_parts = parsed.parts.clone();

                loop {
                    if current_parts.is_empty() {
                        break;
                    }
                    current_parts.pop();

                    let parent_parsed = ParsedParts {
                        drive: parsed.drive.clone(),
                        root: parsed.root.clone(),
                        parts: current_parts.clone(),
                    };

                    let parent_py = Py::new(py, Self::from_parsed_parts(parent_parsed))?;
                    parent_objs.push(parent_py);
                }

                PyList::new(py, parent_objs)
            }

            fn is_relative_to(&self, other: &Bound<PyAny>) -> PyResult<bool> {
                match other.extract::<String>() {
                    Ok(other_str) => {
                        let other_path = <$separator>::parse(&other_str);
                        let self_parsed = self.parsed_parts();

                        // Must have same anchor
                        if self_parsed.drive != other_path.drive
                            || self_parsed.root != other_path.root
                        {
                            return Ok(false);
                        }

                        // self.parts must start with other.parts
                        if other_path.parts.len() > self_parsed.parts.len() {
                            return Ok(false);
                        }

                        for (i, other_part) in other_path.parts.iter().enumerate() {
                            if self_parsed.parts[i] != *other_part {
                                return Ok(false);
                            }
                        }

                        Ok(true)
                    }
                    Err(_) => Ok(false),
                }
            }

            fn relative_to(&self, py: Python, other: &Bound<PyAny>) -> PyResult<Py<Self>> {
                let other_str = other.extract::<String>()?;
                let other_path = <$separator>::parse(&other_str);
                let self_parsed = self.parsed_parts();

                // Must have same anchor
                if self_parsed.drive != other_path.drive || self_parsed.root != other_path.root {
                    return Err(pyo3::exceptions::PyValueError::new_err(format!(
                        "{} is not relative to {}",
                        self.str_repr(),
                        other_str
                    )));
                }

                // self.parts must start with other.parts
                if other_path.parts.len() > self_parsed.parts.len() {
                    return Err(pyo3::exceptions::PyValueError::new_err(format!(
                        "{} is not relative to {}",
                        self.str_repr(),
                        other_str
                    )));
                }

                for (i, other_part) in other_path.parts.iter().enumerate() {
                    if self_parsed.parts[i] != *other_part {
                        return Err(pyo3::exceptions::PyValueError::new_err(format!(
                            "{} is not relative to {}",
                            self.str_repr(),
                            other_str
                        )));
                    }
                }

                // Build relative path from remaining parts
                let remaining: Vec<String> = self_parsed.parts[other_path.parts.len()..].to_vec();
                let relative_parsed = ParsedParts {
                    drive: String::new(),
                    root: String::new(),
                    parts: remaining,
                };

                Py::new(py, Self::from_parsed_parts(relative_parsed))
            }

            fn __lt__(&self, other: &Bound<PyAny>) -> PyResult<bool> {
                match other.extract::<Py<$class_name>>() {
                    Ok(other_py) => Python::attach(|py| {
                        let other_path = other_py.borrow(py);
                        Ok(self.parts_normcase() < other_path.parts_normcase())
                    }),
                    Err(_) => Ok(false),
                }
            }

            fn __le__(&self, other: &Bound<PyAny>) -> PyResult<bool> {
                match other.extract::<Py<$class_name>>() {
                    Ok(other_py) => Python::attach(|py| {
                        let other_path = other_py.borrow(py);
                        Ok(self.parts_normcase() <= other_path.parts_normcase())
                    }),
                    Err(_) => Ok(false),
                }
            }

            fn __gt__(&self, other: &Bound<PyAny>) -> PyResult<bool> {
                match other.extract::<Py<$class_name>>() {
                    Ok(other_py) => Python::attach(|py| {
                        let other_path = other_py.borrow(py);
                        Ok(self.parts_normcase() > other_path.parts_normcase())
                    }),
                    Err(_) => Ok(false),
                }
            }

            fn __ge__(&self, other: &Bound<PyAny>) -> PyResult<bool> {
                match other.extract::<Py<$class_name>>() {
                    Ok(other_py) => Python::attach(|py| {
                        let other_path = other_py.borrow(py);
                        Ok(self.parts_normcase() >= other_path.parts_normcase())
                    }),
                    Err(_) => Ok(false),
                }
            }

            fn __fspath__(&self) -> String {
                self.str_repr().clone()
            }

            fn with_name(&self, py: Python, name: &str) -> PyResult<Py<Self>> {
                let new_parsed = <$separator>::with_name(self.parsed_parts(), name);
                Py::new(py, Self::from_parsed_parts(new_parsed))
            }

            fn with_suffix(&self, py: Python, suffix: &str) -> PyResult<Py<Self>> {
                let new_parsed = <$separator>::with_suffix(self.parsed_parts(), suffix);
                Py::new(py, Self::from_parsed_parts(new_parsed))
            }

            fn with_stem(&self, py: Python, stem: &str) -> PyResult<Py<Self>> {
                let suffix = self.parsed_parts().suffix();
                let new_parsed =
                    <$separator>::with_name(self.parsed_parts(), &format!("{}{}", stem, suffix));
                Py::new(py, Self::from_parsed_parts(new_parsed))
            }

            fn __bytes__(&self, py: Python) -> PyResult<Vec<u8>> {
                PyModule::import(py, "os")?
                    .getattr("fsencode")?
                    .call1((self.str_repr(),))?
                    .extract()
            }

            fn as_uri(&self) -> PyResult<String> {
                let parsed = self.parsed_parts();
                // as_uri only works on absolute paths
                if parsed.drive.is_empty() && parsed.root.is_empty() {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "cannot use as_uri with a relative path",
                    ));
                }

                // Convert path to forward slashes for URI
                let path_uri = self.str_repr().replace('\\', "/");

                // For Windows paths with drive letter: file:///C:/path
                if !parsed.drive.is_empty() {
                    Ok(format!("file:///{}", path_uri))
                } else {
                    // For POSIX paths: file:///path
                    Ok(format!("file://{}", path_uri))
                }
            }

            fn full_match(&self, pattern: &str) -> PyResult<bool> {
                // Simple globbing implementation
                self._glob_match(pattern)
            }
        }

        impl $class_name {
            fn _glob_match(&self, pattern: &str) -> PyResult<bool> {
                // Convert pathlib glob pattern to simple matching
                // ** matches zero or more directories
                // * matches zero or more characters within a directory
                // ? matches exactly one character
                // [seq] matches characters in sequence

                let path_parts: Vec<&str> = self.str_repr().split(['/', '\\'].as_ref()).collect();
                let pattern_parts: Vec<&str> = pattern.split(['/', '\\'].as_ref()).collect();

                self._match_recursive(&path_parts, 0, &pattern_parts, 0)
            }

            fn _match_recursive(
                &self,
                path_parts: &[&str],
                p_idx: usize,
                pattern_parts: &[&str],
                pat_idx: usize,
            ) -> PyResult<bool> {
                // Base cases
                if pat_idx >= pattern_parts.len() {
                    return Ok(p_idx >= path_parts.len());
                }

                if pattern_parts[pat_idx] == "**" {
                    // ** can match zero or more path segments
                    if pat_idx + 1 >= pattern_parts.len() {
                        // ** is the last pattern, matches everything
                        return Ok(true);
                    }

                    // Try matching zero segments (skip **)
                    if self._match_recursive(path_parts, p_idx, pattern_parts, pat_idx + 1)? {
                        return Ok(true);
                    }

                    // Try matching one or more segments
                    if p_idx < path_parts.len() {
                        return self._match_recursive(
                            path_parts,
                            p_idx + 1,
                            pattern_parts,
                            pat_idx,
                        );
                    }

                    return Ok(false);
                }

                if p_idx >= path_parts.len() {
                    return Ok(false);
                }

                // Match current segment
                if self._segment_matches(path_parts[p_idx], pattern_parts[pat_idx])? {
                    return self._match_recursive(
                        path_parts,
                        p_idx + 1,
                        pattern_parts,
                        pat_idx + 1,
                    );
                }

                Ok(false)
            }

            fn _segment_matches(&self, segment: &str, pattern: &str) -> PyResult<bool> {
                if pattern == "*" {
                    return Ok(true);
                }

                let mut s_idx = 0;
                let mut p_idx = 0;
                let s_chars: Vec<char> = segment.chars().collect();
                let p_chars: Vec<char> = pattern.chars().collect();

                while p_idx < p_chars.len() {
                    match p_chars[p_idx] {
                        '*' => {
                            if p_idx + 1 >= p_chars.len() {
                                return Ok(true);
                            }
                            // Find next char after *
                            let next_char = p_chars[p_idx + 1];
                            while s_idx < s_chars.len() && s_chars[s_idx] != next_char {
                                s_idx += 1;
                            }
                            if s_idx >= s_chars.len() {
                                return Ok(false);
                            }
                            p_idx += 1;
                        }
                        '?' => {
                            if s_idx >= s_chars.len() {
                                return Ok(false);
                            }
                            s_idx += 1;
                            p_idx += 1;
                        }
                        _ => {
                            if s_idx >= s_chars.len() || s_chars[s_idx] != p_chars[p_idx] {
                                return Ok(false);
                            }
                            s_idx += 1;
                            p_idx += 1;
                        }
                    }
                }

                Ok(s_idx >= s_chars.len())
            }
        }
    };
}

// ============================================================================
// GENERATE CLASSES
// ============================================================================

create_pure_path_class!(PurePosixPath, PosixSeparator, "PurePosixPath");
create_pure_path_class!(PureWindowsPath, WindowsSeparator, "PureWindowsPath");
