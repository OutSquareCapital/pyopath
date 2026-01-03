//! Glob iterator for lazy path yielding.

use crate::flavor::PathFlavor;
use crate::pure_path::PurePath;
use pyo3::prelude::*;
use std::path::PathBuf;

use std::fs;

/// Trait for types that can be created from a PurePath

/// Simple glob pattern matching
pub struct GlobPattern {
    pattern: String,
    #[allow(dead_code)]
    is_recursive: bool,
}

impl GlobPattern {
    pub fn new(pattern: &str) -> Self {
        let is_recursive = pattern.contains("**");
        Self {
            pattern: pattern.to_string(),
            is_recursive,
        }
    }

    /// Check if a filename matches the pattern
    pub fn matches(&self, name: &str) -> bool {
        // Simple implementation for now - just handle * wildcard
        if self.pattern == "*" {
            return true;
        }

        // Handle *.ext pattern
        if let Some(stripped) = self.pattern.strip_prefix("*.") {
            return name.ends_with(&format!(".{}", stripped));
        }

        // Handle *suffix pattern
        if let Some(suffix) = self.pattern.strip_prefix('*') {
            return name.ends_with(suffix);
        }

        // Handle prefix* pattern
        if let Some(prefix) = self.pattern.strip_suffix('*') {
            return name.starts_with(prefix);
        }

        // Exact match
        name == self.pattern
    }
}

/// Iterator over glob results using std::fs::read_dir
pub struct FastGlobIter {
    pattern: GlobPattern,
    current_dirs: Vec<PathBuf>,
    current_entries: Option<fs::ReadDir>,
}

impl FastGlobIter {
    pub fn new(dir: PathBuf, pattern: &str) -> PyResult<Self> {
        let pattern_obj = GlobPattern::new(pattern);

        // For recursive patterns, start with base dir in queue
        let current_dirs = if pattern_obj.is_recursive {
            vec![dir.clone()]
        } else {
            vec![]
        };

        // For non-recursive, read immediately
        let current_entries = if !pattern_obj.is_recursive {
            Some(
                fs::read_dir(&dir)
                    .map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))?,
            )
        } else {
            None
        };

        Ok(Self {
            pattern: pattern_obj,
            current_dirs,
            current_entries,
        })
    }

    fn extract_final_pattern(&self) -> &str {
        // For **/*.txt, extract *.txt
        if let Some(pos) = self.pattern.pattern.rfind("**/") {
            &self.pattern.pattern[pos + 3..]
        } else {
            &self.pattern.pattern
        }
    }
}

impl Iterator for FastGlobIter {
    type Item = PyResult<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we have current entries, process them
            if let Some(entries) = self.current_entries.as_mut() {
                match entries.next() {
                    Some(Ok(entry)) => {
                        let path = entry.path();

                        // If recursive, add subdirectories to queue
                        if self.pattern.is_recursive {
                            if let Ok(metadata) = entry.metadata() {
                                if metadata.is_dir() {
                                    self.current_dirs.push(path.clone());
                                }
                            }
                        }

                        // Check if filename matches
                        if let Some(name) = entry.file_name().to_str() {
                            let final_pattern = self.extract_final_pattern();
                            if GlobPattern::new(final_pattern).matches(name) {
                                return Some(Ok(path));
                            }
                        }
                        // Continue loop if pattern doesn't match
                    }
                    Some(Err(e)) => {
                        return Some(Err(pyo3::exceptions::PyOSError::new_err(e.to_string())));
                    }
                    None => {
                        // Current directory exhausted
                        self.current_entries = None;
                    }
                }
            } else if !self.current_dirs.is_empty() {
                // Get next directory to process
                let next_dir = self.current_dirs.remove(0);
                match fs::read_dir(&next_dir) {
                    Ok(read_dir) => {
                        self.current_entries = Some(read_dir);
                    }
                    Err(_) => {
                        // Skip directories we can't read
                        continue;
                    }
                }
            } else {
                // No more entries and no more directories
                return None;
            }
        }
    }
}

pub trait FromPurePath {
    fn from_pure_path(pure: PurePath) -> Self;
}

/// Internal generic iterator for glob results
/// T must implement FromPurePath
pub struct GlobIteratorInner<T: FromPurePath> {
    glob_iter: FastGlobIter,
    /// The flavor to use for parsing paths
    flavor: PathFlavor,
    /// Phantom data for the path type
    _marker: std::marker::PhantomData<T>,
}

impl<T: FromPurePath> GlobIteratorInner<T> {
    pub fn new(base_dir: PathBuf, pattern: &str, flavor: PathFlavor) -> PyResult<Self> {
        let glob_iter = FastGlobIter::new(base_dir, pattern)?;

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
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}
