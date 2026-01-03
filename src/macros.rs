use crate::core::ParsedParts;
use crate::separators::{PosixSeparator, WindowsSeparator};
use pyo3::exceptions::PyTypeError;
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
            fn new(paths: &Bound<PyAny>) -> PyResult<Self> {
                let path_strs = if let Ok(s) = paths.extract::<String>() {
                    vec![s]
                } else if let Ok(tuple) = paths.cast::<PyTuple>() {
                    let mut strs = Vec::new();
                    for item in tuple {
                        strs.push(item.extract::<String>()?);
                    }
                    strs
                } else {
                    return Err(PyTypeError::new_err(
                        "Path() argument must be a string or sequence of strings",
                    ));
                };

                let str_repr = if path_strs.len() == 1 {
                    path_strs[0].clone()
                } else {
                    path_strs.join(&<$separator>::sep().to_string())
                };

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

            #[pyo3(signature = (*paths))]
            fn joinpath(&self, py: Python, paths: &Bound<PyTuple>) -> PyResult<Py<Self>> {
                let mut joined = self.str_repr.clone();
                let sep = <$separator>::sep();
                for item in paths {
                    let s = item.extract::<String>()?;
                    joined = format!("{}{}{}", joined, sep, s);
                }

                Py::new(
                    py,
                    Self {
                        str_repr: joined,
                        parsed: OnceLock::new(),
                    },
                )
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
        }
    };
}

// ============================================================================
// GENERATE CLASSES
// ============================================================================

create_pure_path_class!(PurePosixPath, PosixSeparator, "PurePosixPath");
create_pure_path_class!(PureWindowsPath, WindowsSeparator, "PureWindowsPath");
