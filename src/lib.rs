use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use std::sync::OnceLock;

// ============================================================================
// SHARED STRUCTURES
// ============================================================================

#[derive(Clone, Debug)]
struct PathData {
    raw_paths: Vec<String>,
}

impl PathData {
    fn new(paths: &[&str]) -> Self {
        Self {
            raw_paths: paths.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn raw_path(&self) -> String {
        if self.raw_paths.len() == 1 {
            self.raw_paths[0].clone()
        } else {
            self.raw_paths.join("/")
        }
    }
}

#[derive(Clone, Debug)]
struct ParsedParts {
    drive: String,
    root: String,
    parts: Vec<String>,
}

impl ParsedParts {
    fn anchor(&self) -> String {
        format!("{}{}", self.drive, self.root)
    }

    fn all_parts(&self) -> Vec<String> {
        let mut result = Vec::new();
        if !self.drive.is_empty() || !self.root.is_empty() {
            result.push(self.anchor());
        }
        result.extend(self.parts.iter().cloned());
        result
    }

    fn name(&self) -> String {
        self.parts.last().cloned().unwrap_or_default()
    }

    fn parent_parts(&self) -> Vec<String> {
        if self.parts.is_empty() {
            self.parts.clone()
        } else {
            self.parts[..self.parts.len() - 1].to_vec()
        }
    }

    fn stem(&self) -> String {
        let name = self.name();
        if let Some(idx) = name.rfind('.') {
            if idx == 0 {
                name
            } else {
                name[..idx].to_string()
            }
        } else {
            name
        }
    }

    fn suffix(&self) -> String {
        let name = self.name();
        if let Some(idx) = name.rfind('.') {
            if idx == 0 {
                String::new()
            } else {
                name[idx..].to_string()
            }
        } else {
            String::new()
        }
    }

    fn suffixes(&self) -> Vec<String> {
        let name = self.name();
        let mut result = Vec::new();
        let mut remaining = name.as_str();

        while let Some(idx) = remaining.find('.') {
            remaining = &remaining[idx + 1..];
            if idx > 0 && !remaining.is_empty() {
                result.push(format!(".{}", remaining));
            }
        }
        result
    }
}

// ============================================================================
// SEPARATORS
// ============================================================================

struct PosixSeparator;
struct WindowsSeparator;

impl PosixSeparator {
    const SEP: char = '/';

    fn sep() -> char {
        Self::SEP
    }

    fn parse(raw_path: &str) -> ParsedParts {
        let (drive, root, rest) = Self::splitroot(raw_path);
        let parts: Vec<String> = rest
            .split('/')
            .filter(|p| !p.is_empty() && *p != ".")
            .map(|s| s.to_string())
            .collect();
        ParsedParts { drive, root, parts }
    }

    fn splitroot(path: &str) -> (String, String, String) {
        if path.starts_with('/') {
            (String::new(), "/".to_string(), path[1..].to_string())
        } else {
            (String::new(), String::new(), path.to_string())
        }
    }

    fn with_name(parsed: &ParsedParts, name: &str) -> String {
        let mut new_parts = parsed.parent_parts();
        new_parts.push(name.to_string());
        let body = new_parts.join("/");
        if parsed.root.is_empty() && parsed.drive.is_empty() {
            if body.is_empty() {
                ".".to_string()
            } else {
                body
            }
        } else {
            format!("{}{}{}", parsed.drive, parsed.root, body)
        }
    }

    fn with_suffix(parsed: &ParsedParts, suffix: &str) -> String {
        let mut new_parts = parsed.parent_parts();
        let stem = parsed.stem();
        new_parts.push(format!("{}{}", stem, suffix));
        let body = new_parts.join("/");
        if parsed.root.is_empty() && parsed.drive.is_empty() {
            if body.is_empty() {
                ".".to_string()
            } else {
                body
            }
        } else {
            format!("{}{}{}", parsed.drive, parsed.root, body)
        }
    }

    fn is_absolute(parsed: &ParsedParts) -> bool {
        !parsed.root.is_empty()
    }
}

impl WindowsSeparator {
    const SEP: char = '\\';

    fn sep() -> char {
        Self::SEP
    }

    fn parse(raw_path: &str) -> ParsedParts {
        let (drive, root, rest) = Self::splitroot(raw_path);
        let parts: Vec<String> = rest
            .split(|c| c == '\\' || c == '/')
            .filter(|p| !p.is_empty() && *p != ".")
            .map(|s| s.to_string())
            .collect();
        ParsedParts { drive, root, parts }
    }

    fn splitroot(path: &str) -> (String, String, String) {
        // Simplified Windows path parsing
        // TODO: Handle UNC paths (\\server\share) properly
        if path.len() >= 2 && path.as_bytes()[1] == b':' {
            // Drive letter: "C:..."
            let drive = path[..2].to_string();
            if path.len() > 2 && (path.as_bytes()[2] == b'\\' || path.as_bytes()[2] == b'/') {
                (drive, "\\".to_string(), path[3..].to_string())
            } else {
                (drive, String::new(), path[2..].to_string())
            }
        } else if path.starts_with("\\\\") {
            // UNC path
            (String::new(), "\\\\".to_string(), path[2..].to_string())
        } else if path.starts_with("\\") || path.starts_with("/") {
            (String::new(), "\\".to_string(), path[1..].to_string())
        } else {
            (String::new(), String::new(), path.to_string())
        }
    }

    fn with_name(parsed: &ParsedParts, name: &str) -> String {
        let mut new_parts = parsed.parent_parts();
        new_parts.push(name.to_string());
        let body = new_parts.join("\\");
        if parsed.root.is_empty() && parsed.drive.is_empty() {
            if body.is_empty() {
                ".".to_string()
            } else {
                body
            }
        } else {
            format!("{}{}{}", parsed.drive, parsed.root, body)
        }
    }

    fn with_suffix(parsed: &ParsedParts, suffix: &str) -> String {
        let mut new_parts = parsed.parent_parts();
        let stem = parsed.stem();
        new_parts.push(format!("{}{}", stem, suffix));
        let body = new_parts.join("\\");
        if parsed.root.is_empty() && parsed.drive.is_empty() {
            if body.is_empty() {
                ".".to_string()
            } else {
                body
            }
        } else {
            format!("{}{}{}", parsed.drive, parsed.root, body)
        }
    }

    fn is_absolute(parsed: &ParsedParts) -> bool {
        !parsed.root.is_empty() || !parsed.drive.is_empty()
    }
}

// ============================================================================
// MACRO: Generates PurePath classes
// ============================================================================

macro_rules! create_pure_path_class {
    ($class_name:ident, $separator:ty, $py_name:expr) => {
        #[pyclass(frozen, name = $py_name)]
        pub struct $class_name {
            data: PathData,
            str_repr: String,
            parsed: OnceLock<ParsedParts>,
        }

        impl $class_name {
            fn parsed_parts(&self) -> &ParsedParts {
                self.parsed
                    .get_or_init(|| <$separator>::parse(&self.data.raw_path()))
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

                let raw_path = if path_strs.len() == 1 {
                    path_strs[0].clone()
                } else {
                    path_strs.join(&<$separator>::sep().to_string())
                };

                Ok(Self {
                    data: PathData::new(&path_strs.iter().map(|s| s.as_str()).collect::<Vec<_>>()),
                    str_repr: raw_path,
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
                let new_instance = Self {
                    data: PathData::new(&[&joined]),
                    str_repr: joined,
                    parsed: OnceLock::new(),
                };
                Py::new(py, new_instance)
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
                } else {
                    format!(
                        "{}{}{}",
                        parsed.drive,
                        parsed.root,
                        parent_parts.join(&sep.to_string())
                    )
                };

                let new_instance = Self {
                    data: PathData::new(&[&parent_str]),
                    str_repr: parent_str,
                    parsed: OnceLock::new(),
                };
                Py::new(py, new_instance)
            }

            fn as_posix(&self) -> String {
                self.str_repr.replace('\\', "/")
            }

            fn is_absolute(&self) -> bool {
                <$separator>::is_absolute(self.parsed_parts())
            }

            fn joinpath(&self, py: Python, paths: &Bound<PyTuple>) -> PyResult<Py<Self>> {
                let mut joined = self.str_repr.clone();
                let sep = <$separator>::sep();
                for item in paths {
                    let s = item.extract::<String>()?;
                    joined = format!("{}{}{}", joined, sep, s);
                }

                let new_instance = Self {
                    data: PathData::new(&[&joined]),
                    str_repr: joined,
                    parsed: OnceLock::new(),
                };
                Py::new(py, new_instance)
            }

            fn with_name(&self, py: Python, name: &str) -> PyResult<Py<Self>> {
                let new_path = <$separator>::with_name(self.parsed_parts(), name);
                let new_instance = Self {
                    data: PathData::new(&[&new_path]),
                    str_repr: new_path,
                    parsed: OnceLock::new(),
                };
                Py::new(py, new_instance)
            }

            fn with_suffix(&self, py: Python, suffix: &str) -> PyResult<Py<Self>> {
                let new_path = <$separator>::with_suffix(self.parsed_parts(), suffix);
                let new_instance = Self {
                    data: PathData::new(&[&new_path]),
                    str_repr: new_path,
                    parsed: OnceLock::new(),
                };
                Py::new(py, new_instance)
            }

            fn with_stem(&self, py: Python, stem: &str) -> PyResult<Py<Self>> {
                let suffix = self.parsed_parts().suffix();
                let new_path =
                    <$separator>::with_name(self.parsed_parts(), &format!("{}{}", stem, suffix));
                let new_instance = Self {
                    data: PathData::new(&[&new_path]),
                    str_repr: new_path,
                    parsed: OnceLock::new(),
                };
                Py::new(py, new_instance)
            }
        }
    };
}

// ============================================================================
// GENERATE CLASSES
// ============================================================================

create_pure_path_class!(PurePosixPath, PosixSeparator, "PurePosixPath");
create_pure_path_class!(PureWindowsPath, WindowsSeparator, "PureWindowsPath");

// Platform-specific default
#[cfg(windows)]
pub type PurePath = PureWindowsPath;

#[cfg(unix)]
pub type PurePath = PurePosixPath;

// ============================================================================
// MODULE
// ============================================================================

#[pymodule]
fn pyopath(py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PurePosixPath>()?;
    m.add_class::<PureWindowsPath>()?;

    // Default alias
    #[cfg(windows)]
    m.add("PurePath", py.get_type::<PureWindowsPath>())?;

    #[cfg(unix)]
    m.add("PurePath", py.get_type::<PurePosixPath>())?;

    Ok(())
}
