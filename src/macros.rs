use crate::core::ParsedParts;
use crate::separators::{PosixSeparator, WindowsSeparator};
use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};
use std::sync::OnceLock;
macro_rules! create_pure_path_class {
    ($class_name:ident, $separator:ty, $py_name:expr) => {
        #[pyclass(frozen, name = $py_name)]
        pub struct $class_name {
            str_repr: String,
            parsed: OnceLock<ParsedParts>,
        }

        impl $class_name {
            fn parsed_parts(&self) -> &ParsedParts {
                self.parsed
                    .get_or_init(|| <$separator>::parse(&self.str_repr))
            }
        }

        #[pymethods]
        impl $class_name {
            #[new]
            #[pyo3(signature = (*args))]
            fn new(args: &Bound<PyTuple>) -> PyResult<Self> {
                if args.is_empty() {
                    // If no arguments, use current directory
                    return Ok(Self {
                        str_repr: ".".to_string(),
                        parsed: OnceLock::new(),
                    });
                }

                let mut path_strs = Vec::new();
                for item in args {
                    path_strs.push(item.extract::<String>()?);
                }

                // Join paths, resetting when an absolute path is encountered
                let mut str_repr = String::new();
                for path_str in path_strs {
                    let parsed = <$separator>::parse(&path_str);
                    // If this segment is absolute (has drive or root), reset and use it
                    if !parsed.drive.is_empty() || !parsed.root.is_empty() {
                        str_repr = path_str;
                    } else if str_repr.is_empty() {
                        str_repr = path_str;
                    } else {
                        str_repr.push(<$separator>::sep());
                        str_repr.push_str(&path_str);
                    }
                }

                // Normalize slashes for this separator type
                let normalized = <$separator>::normalize_path(&str_repr);

                Ok(Self {
                    str_repr: normalized,
                    parsed: OnceLock::new(),
                })
            }

            fn __str__(&self) -> String {
                self.str_repr.clone()
            }

            fn __repr__(&self) -> String {
                format!("{}('{}')", stringify!($class_name), self.str_repr)
            }

            fn __eq__(&self, other: &Bound<PyAny>) -> PyResult<bool> {
                match other.extract::<Py<$class_name>>() {
                    Ok(other_py) => Python::attach(|py| {
                        let other_path = other_py.borrow(py);
                        Ok(self.str_repr == other_path.str_repr)
                    }),
                    Err(_) => Ok(false),
                }
            }

            fn __hash__(&self) -> u64 {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                self.str_repr.hash(&mut hasher);
                hasher.finish()
            }

            fn __truediv__(&self, py: Python, key: String) -> PyResult<Py<Self>> {
                let sep = <$separator>::sep();
                let joined = format!("{}{}{}", self.str_repr, sep, key);
                Py::new(
                    py,
                    Self {
                        str_repr: joined,
                        parsed: OnceLock::new(),
                    },
                )
            }

            #[getter]
            fn parts(&self) -> Vec<String> {
                self.parsed_parts().all_parts()
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
                let sep = <$separator>::sep();

                let parent_str = if parsed.root.is_empty() && parsed.drive.is_empty() {
                    if parent_parts.is_empty() {
                        ".".to_string()
                    } else {
                        parent_parts.join(&sep.to_string())
                    }
                } else if parent_parts.is_empty() {
                    // Just drive + root, no body
                    format!("{}{}", parsed.drive, parsed.root)
                } else {
                    // root already ends with sep (e.g., "\\" or "/"), join directly
                    format!(
                        "{}{}{}",
                        parsed.drive,
                        parsed.root,
                        parent_parts.join(&sep.to_string())
                    )
                };

                Py::new(
                    py,
                    Self {
                        str_repr: parent_str,
                        parsed: OnceLock::new(),
                    },
                )
            }

            fn as_posix(&self) -> String {
                self.str_repr.replace('\\', "/")
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
                // Collect all path strings using os.fspath for PathLike objects
                let os_module = PyModule::import(py, "os")?;
                let fspath = os_module.getattr("fspath")?;

                let mut path_strs = Vec::new();
                for item in pathsegments {
                    let path_str: String = fspath.call1((item,))?.extract()?;
                    path_strs.push(path_str);
                }

                if path_strs.is_empty() {
                    return Ok(Py::new(
                        py,
                        Self {
                            str_repr: ".".to_string(),
                            parsed: OnceLock::new(),
                        },
                    )?);
                }

                let path_module = PyModule::import(py, <$separator>::MODULE_NAME)?;
                let join_fn = path_module.getattr("join")?;

                let path_tuple = PyTuple::new(py, &path_strs)?;
                let joined_result = join_fn.call(path_tuple, None)?;
                let str_repr: String = joined_result.extract()?;

                // Normalize for consistency
                let normalized = <$separator>::normalize_path(&str_repr);

                Py::new(
                    py,
                    Self {
                        str_repr: normalized,
                        parsed: OnceLock::new(),
                    },
                )
            }

            #[pyo3(signature = (*paths))]
            fn joinpath(&self, py: Python, paths: &Bound<PyTuple>) -> PyResult<Py<Self>> {
                // with_segments(self, *paths)
                let mut segments = vec![self.str_repr.clone()];

                let os_module = PyModule::import(py, "os")?;
                let fspath = os_module.getattr("fspath")?;

                for item in paths {
                    let path_str: String = fspath.call1((item,))?.extract()?;
                    segments.push(path_str);
                }

                let segments_tuple = PyTuple::new(py, &segments)?;
                self.with_segments(py, &segments_tuple)
            }

            #[getter]
            fn parents<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
                let sep = <$separator>::sep();
                let parsed = self.parsed_parts();

                // Build all parent paths
                let mut parent_objs: Vec<Py<Self>> = Vec::new();
                let mut current_parts = parsed.parts.clone();

                loop {
                    if current_parts.is_empty() {
                        break;
                    }
                    current_parts.pop();

                    let parent_str = if parsed.root.is_empty() && parsed.drive.is_empty() {
                        if current_parts.is_empty() {
                            ".".to_string()
                        } else {
                            current_parts.join(&sep.to_string())
                        }
                    } else if current_parts.is_empty() {
                        // Just drive + root
                        format!("{}{}", parsed.drive, parsed.root)
                    } else {
                        // root already ends with sep, join directly
                        format!(
                            "{}{}{}",
                            parsed.drive,
                            parsed.root,
                            current_parts.join(&sep.to_string())
                        )
                    };

                    let parent_py = Py::new(
                        py,
                        Self {
                            str_repr: parent_str,
                            parsed: OnceLock::new(),
                        },
                    )?;
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
                        self.str_repr, other_str
                    )));
                }

                // self.parts must start with other.parts
                if other_path.parts.len() > self_parsed.parts.len() {
                    return Err(pyo3::exceptions::PyValueError::new_err(format!(
                        "{} is not relative to {}",
                        self.str_repr, other_str
                    )));
                }

                for (i, other_part) in other_path.parts.iter().enumerate() {
                    if self_parsed.parts[i] != *other_part {
                        return Err(pyo3::exceptions::PyValueError::new_err(format!(
                            "{} is not relative to {}",
                            self.str_repr, other_str
                        )));
                    }
                }

                // Build relative path from remaining parts
                let remaining: Vec<String> = self_parsed.parts[other_path.parts.len()..].to_vec();
                let sep = <$separator>::sep();
                let relative_str = if remaining.is_empty() {
                    ".".to_string()
                } else {
                    remaining.join(&sep.to_string())
                };

                Py::new(
                    py,
                    Self {
                        str_repr: relative_str,
                        parsed: OnceLock::new(),
                    },
                )
            }

            fn __lt__(&self, other: &Bound<PyAny>) -> PyResult<bool> {
                match other.extract::<Py<$class_name>>() {
                    Ok(other_py) => Python::attach(|py| {
                        let other_path = other_py.borrow(py);
                        Ok(self.str_repr < other_path.str_repr)
                    }),
                    Err(_) => Ok(false),
                }
            }

            fn __le__(&self, other: &Bound<PyAny>) -> PyResult<bool> {
                match other.extract::<Py<$class_name>>() {
                    Ok(other_py) => Python::attach(|py| {
                        let other_path = other_py.borrow(py);
                        Ok(self.str_repr <= other_path.str_repr)
                    }),
                    Err(_) => Ok(false),
                }
            }

            fn __gt__(&self, other: &Bound<PyAny>) -> PyResult<bool> {
                match other.extract::<Py<$class_name>>() {
                    Ok(other_py) => Python::attach(|py| {
                        let other_path = other_py.borrow(py);
                        Ok(self.str_repr > other_path.str_repr)
                    }),
                    Err(_) => Ok(false),
                }
            }

            fn __ge__(&self, other: &Bound<PyAny>) -> PyResult<bool> {
                match other.extract::<Py<$class_name>>() {
                    Ok(other_py) => Python::attach(|py| {
                        let other_path = other_py.borrow(py);
                        Ok(self.str_repr >= other_path.str_repr)
                    }),
                    Err(_) => Ok(false),
                }
            }

            fn __fspath__(&self) -> String {
                self.str_repr.clone()
            }

            fn with_name(&self, py: Python, name: &str) -> PyResult<Py<Self>> {
                let new_path = <$separator>::with_name(self.parsed_parts(), name);
                Py::new(
                    py,
                    Self {
                        str_repr: new_path,
                        parsed: OnceLock::new(),
                    },
                )
            }

            fn with_suffix(&self, py: Python, suffix: &str) -> PyResult<Py<Self>> {
                let new_path = <$separator>::with_suffix(self.parsed_parts(), suffix);
                Py::new(
                    py,
                    Self {
                        str_repr: new_path,
                        parsed: OnceLock::new(),
                    },
                )
            }

            fn with_stem(&self, py: Python, stem: &str) -> PyResult<Py<Self>> {
                let suffix = self.parsed_parts().suffix();
                let new_path =
                    <$separator>::with_name(self.parsed_parts(), &format!("{}{}", stem, suffix));
                Py::new(
                    py,
                    Self {
                        str_repr: new_path,
                        parsed: OnceLock::new(),
                    },
                )
            }

            fn __bytes__(&self) -> PyResult<Vec<u8>> {
                Ok(self.str_repr.as_bytes().to_vec())
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
                let path_uri = self.str_repr.replace('\\', "/");

                // For Windows paths, add extra slash for C: → /C:
                if !parsed.drive.is_empty()
                    && path_uri.starts_with(|c: char| c.is_ascii_alphabetic())
                {
                    Ok(format!("file:///{}", path_uri))
                } else {
                    // For UNC paths
                    Ok(format!("file:{}", path_uri))
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

                let path_parts: Vec<&str> = self.str_repr.split(['/', '\\'].as_ref()).collect();
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
