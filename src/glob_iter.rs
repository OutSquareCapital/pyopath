//! Glob iterator using walkdir and globset.

use crate::flavor::PathFlavor;
use crate::pure_path::PurePath;
use globset::{GlobBuilder, GlobMatcher};
use pyo3::prelude::*;
use std::path::PathBuf;
use walkdir::WalkDir;

pub trait FromPurePath {
    fn from_pure_path(pure: PurePath) -> Self;
}

/// Iterator over glob results
pub struct GlobIteratorInner<T: FromPurePath> {
    iter: walkdir::IntoIter,
    matcher: GlobMatcher,
    base_dir: PathBuf,
    flavor: PathFlavor,
    _marker: std::marker::PhantomData<T>,
}

impl<T: FromPurePath> GlobIteratorInner<T> {
    pub fn new(
        base_dir: PathBuf,
        pattern: &str,
        flavor: PathFlavor,
        case_sensitive: Option<bool>,
        follow_symlinks: Option<bool>,
    ) -> PyResult<Self> {
        let cs = case_sensitive.unwrap_or_else(|| match flavor {
            PathFlavor::Windows => false,
            PathFlavor::Posix => true,
        });
        let follow = follow_symlinks.unwrap_or(true);

        // 1. Split pattern into static prefix and glob pattern
        let (prefix, glob_pattern) = split_pattern(pattern, flavor);

        // 2. Resolve starting directory
        let start_dir = if prefix.is_empty() {
            base_dir.clone()
        } else {
            base_dir.join(prefix)
        };

        // 3. Build GlobMatcher
        let glob = GlobBuilder::new(&glob_pattern)
            .case_insensitive(!cs)
            .literal_separator(true)
            .backslash_escape(true)
            .build()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        let matcher = glob.compile_matcher();

        // 4. Configure WalkDir
        let is_recursive_pattern = glob_pattern.contains("**");
        let max_depth = if is_recursive_pattern {
            usize::MAX
        } else {
            count_components(&glob_pattern, flavor)
        };

        let walker = WalkDir::new(&start_dir)
            .follow_links(follow)
            .max_depth(max_depth)
            .into_iter();

        Ok(Self {
            iter: walker,
            matcher,
            base_dir: start_dir,
            flavor,
            _marker: std::marker::PhantomData,
        })
    }

    pub fn next_path(&mut self) -> PyResult<Option<T>> {
        loop {
            match self.iter.next() {
                Some(Ok(entry)) => {
                    let path = entry.path();

                    if let Ok(stripped) = path.strip_prefix(&self.base_dir) {
                        if self.matcher.is_match(stripped) {
                            let path_str = path.to_string_lossy();
                            let parsed = crate::parsing::ParsedPath::parse(&path_str, self.flavor);
                            let pure_path = PurePath::new_with_parsed(parsed, self.flavor);
                            return Ok(Some(T::from_pure_path(pure_path)));
                        }
                    }
                }
                Some(Err(_)) => {
                    // Ignore errors (like permission denied) to match pathlib behavior
                    continue;
                }
                None => return Ok(None),
            }
        }
    }
}

fn is_magic(s: &str) -> bool {
    s.contains('*') || s.contains('?') || s.contains('[')
}

fn split_pattern(pattern: &str, flavor: PathFlavor) -> (String, String) {
    let sep = flavor.sep();
    let altsep = flavor.altsep();

    let mut parts = Vec::new();
    let mut current_part = String::new();

    for c in pattern.chars() {
        if c == sep || altsep.is_some_and(|a| c == a) {
            parts.push(current_part);
            current_part = String::new();
        } else {
            current_part.push(c);
        }
    }
    parts.push(current_part);

    let mut split_idx = parts.len();
    for (i, part) in parts.iter().enumerate() {
        if is_magic(part) {
            split_idx = i;
            break;
        }
    }

    let prefix_parts = &parts[..split_idx];
    let glob_parts = &parts[split_idx..];

    let prefix = prefix_parts.join(&sep.to_string());
    let glob = glob_parts.join(&sep.to_string());

    (prefix, glob)
}

fn count_components(pattern: &str, flavor: PathFlavor) -> usize {
    let sep = flavor.sep();
    let altsep = flavor.altsep();
    let mut count = 1;
    if pattern.is_empty() {
        return 0;
    }

    for c in pattern.chars() {
        if c == sep || altsep.is_some_and(|a| c == a) {
            count += 1;
        }
    }
    count
}
