//! Path parsing logic for both POSIX and Windows flavors.

use super::flavor::PathFlavor;

/// Parsed path components.
#[derive(Debug, Clone, Default)]
pub struct ParsedPath {
    /// Drive letter (Windows only, e.g., "C:")
    pub drive: String,
    /// Root separator (e.g., "/" or "\\")
    pub root: String,
    /// Path parts (directories and filename)
    pub parts: Vec<String>,
}

impl ParsedPath {
    /// Parse a path string according to the given flavor.
    #[inline]
    pub fn parse(path: &str, flavor: PathFlavor) -> Self {
        // Fast path for simple names (no separators, no drive letter)
        // This is common in joinpath("foo", "bar")
        if !path.is_empty() && Self::is_simple_name(path, flavor) {
            return Self {
                drive: String::new(),
                root: String::new(),
                parts: vec![path.to_string()],
            };
        }

        match flavor {
            PathFlavor::Posix => Self::parse_posix(path),
            PathFlavor::Windows => Self::parse_windows(path),
        }
    }

    /// Check if path is a simple name (no separators, not special).
    #[inline]
    fn is_simple_name(path: &str, flavor: PathFlavor) -> bool {
        if path == "." || path == ".." {
            return false;
        }
        match flavor {
            PathFlavor::Posix => !path.contains('/'),
            PathFlavor::Windows => {
                !path.contains('/') && !path.contains('\\') && !path.contains(':')
            }
        }
    }

    /// Parse a POSIX path.
    fn parse_posix(path: &str) -> Self {
        if path.is_empty() {
            return Self {
                drive: String::new(),
                root: String::new(),
                parts: vec![],
            };
        }

        let mut root = String::new();
        let mut remaining = path;

        // Handle root - special case for exactly "//" (but not "///")
        if path.starts_with("//") && !path.starts_with("///") {
            root = "//".to_string();
            remaining = &path[2..];
        } else if path.starts_with('/') {
            root = "/".to_string();
            // Skip all leading slashes
            remaining = path.trim_start_matches('/');
        }

        // Split remaining path into parts, filtering empty segments
        let parts: Vec<String> = remaining
            .split('/')
            .filter(|s| !s.is_empty() && *s != ".")
            .map(|s| s.to_string())
            .collect();

        Self {
            drive: String::new(),
            root,
            parts,
        }
    }

    /// Parse a Windows path.
    fn parse_windows(path: &str) -> Self {
        if path.is_empty() {
            return Self {
                drive: String::new(),
                root: String::new(),
                parts: vec![],
            };
        }

        let mut drive = String::new();
        let mut root = String::new();
        let mut remaining = path;

        // Normalize separators for parsing
        let normalized: String = path.replace('/', "\\");
        let norm_ref = normalized.as_str();

        // Check for UNC path: \\server\share or \\?\... or \\.\...
        if let Some(after_slashes) = norm_ref.strip_prefix("\\\\") {
            // Handle \\?\ and \\.\ prefixes
            if after_slashes.starts_with("?\\") || after_slashes.starts_with(".\\") {
                // Verbatim path - \\?\C:\... or \\.\device\...
                let after_prefix = &after_slashes[2..];
                if let Some(pos) = after_prefix.find('\\') {
                    drive = format!("\\\\{}\\{}", &after_slashes[..1], &after_prefix[..pos]);
                    root = "\\".to_string();
                    remaining = &path[4 + pos + 1..];
                } else {
                    drive = format!("\\\\{}{}", &after_slashes[..2], after_prefix);
                    remaining = "";
                }
            } else {
                // UNC path: \\server\share
                let parts_iter: Vec<&str> = after_slashes.splitn(3, '\\').collect();
                if parts_iter.len() >= 2 {
                    drive = format!("\\\\{}\\{}", parts_iter[0], parts_iter[1]);
                    root = "\\".to_string();
                    remaining = if parts_iter.len() > 2 {
                        // Find the position after \\server\share\ in original
                        let skip = 2 + parts_iter[0].len() + 1 + parts_iter[1].len();
                        if path.len() > skip {
                            &path[skip + 1..]
                        } else {
                            ""
                        }
                    } else {
                        ""
                    };
                } else if !parts_iter.is_empty() {
                    // Incomplete UNC: \\server
                    drive = format!("\\\\{}", parts_iter[0]);
                    remaining = "";
                }
            }
        }
        // Check for drive letter: C: or C:\
        else if norm_ref.len() >= 2
            && norm_ref.chars().next().unwrap().is_ascii_alphabetic()
            && norm_ref.chars().nth(1) == Some(':')
        {
            drive = path[..2].to_string();
            remaining = &path[2..];

            // Check for root after drive
            if remaining.starts_with('\\') || remaining.starts_with('/') {
                root = "\\".to_string();
                remaining = remaining.trim_start_matches(['\\', '/']);
            }
        }
        // Check for root without drive
        else if norm_ref.starts_with('\\') {
            root = "\\".to_string();
            remaining = path.trim_start_matches(['\\', '/']);
        }

        // Split remaining path into parts
        let parts: Vec<String> = remaining
            .split(['\\', '/'])
            .filter(|s| !s.is_empty() && *s != ".")
            .map(|s| s.to_string())
            .collect();

        Self { drive, root, parts }
    }

    /// Returns the anchor (drive + root).
    #[inline]
    pub fn anchor(&self) -> String {
        if self.drive.is_empty() && self.root.is_empty() {
            String::new()
        } else if self.drive.is_empty() {
            self.root.clone()
        } else if self.root.is_empty() {
            self.drive.clone()
        } else {
            format!("{}{}", self.drive, self.root)
        }
    }

    /// Check if anchor is empty without allocating.
    #[inline]
    pub fn has_anchor(&self) -> bool {
        !self.drive.is_empty() || !self.root.is_empty()
    }

    /// Returns all parts including anchor as first element if present.
    pub fn all_parts(&self, _flavor: PathFlavor) -> Vec<String> {
        let has_anchor = self.has_anchor();
        let capacity = if has_anchor { 1 } else { 0 } + self.parts.len();
        let mut result = Vec::with_capacity(capacity);

        if has_anchor {
            result.push(self.anchor());
        }

        result.extend(self.parts.iter().cloned());

        result
    }

    /// Converts the parsed path back to a string.
    pub fn to_string(&self, flavor: PathFlavor) -> String {
        let sep = flavor.sep();
        let anchor = self.anchor();

        if self.parts.is_empty() {
            if anchor.is_empty() {
                return ".".to_string();
            }
            return anchor;
        }

        let parts_str = self.parts.join(&sep.to_string());

        if anchor.is_empty() {
            parts_str
        } else if self.root.is_empty() {
            // Drive without root (e.g., "C:foo")
            format!("{}{}", anchor, parts_str)
        } else {
            format!("{}{}", anchor, parts_str)
        }
    }

    /// Check if this path is absolute.
    pub fn is_absolute(&self, flavor: PathFlavor) -> bool {
        match flavor {
            PathFlavor::Posix => !self.root.is_empty(),
            PathFlavor::Windows => {
                // Windows requires both drive and root for absolute path
                // OR UNC path (which has drive as \\server\share and root as \)
                !self.drive.is_empty() && !self.root.is_empty()
            }
        }
    }

    /// Get the name (final component).
    pub fn name(&self) -> &str {
        self.parts.last().map(|s| s.as_str()).unwrap_or("")
    }

    /// Get the suffix (file extension including dot).
    pub fn suffix(&self) -> String {
        let name = self.name();
        if name.is_empty() || name == "." || name == ".." {
            return String::new();
        }

        // Find the last dot that's not at the start
        if let Some(dot_pos) = name.rfind('.') {
            if dot_pos > 0 {
                return name[dot_pos..].to_string();
            }
        }
        String::new()
    }

    /// Get all suffixes.
    pub fn suffixes(&self) -> Vec<String> {
        let name = self.name();
        if name.is_empty() || name == "." || name == ".." {
            return vec![];
        }

        let mut result = Vec::new();
        let mut remaining = name;

        // Skip leading dot for hidden files
        if remaining.starts_with('.') {
            remaining = &remaining[1..];
        }

        for (i, c) in remaining.char_indices() {
            if c == '.' {
                result.push(format!(
                    ".{}",
                    &remaining[i + 1..].split('.').next().unwrap_or("")
                ));
            }
        }

        result
    }

    /// Get the stem (name without final suffix).
    pub fn stem(&self) -> String {
        let name = self.name();
        if name.is_empty() || name == "." || name == ".." {
            return name.to_string();
        }

        let suffix = self.suffix();
        if suffix.is_empty() {
            name.to_string()
        } else {
            name[..name.len() - suffix.len()].to_string()
        }
    }

    /// Join with another path.
    pub fn join(&self, other: &ParsedPath, flavor: PathFlavor) -> ParsedPath {
        // If other is absolute, it replaces self entirely
        if other.is_absolute(flavor) {
            return other.clone();
        }

        // On Windows, if other has a different drive, it replaces self
        if flavor == PathFlavor::Windows && !other.drive.is_empty() && other.drive != self.drive {
            return other.clone();
        }

        // On Windows, if other has a root (but no drive), keep self's drive
        if flavor == PathFlavor::Windows && !other.root.is_empty() {
            return ParsedPath {
                drive: self.drive.clone(),
                root: other.root.clone(),
                parts: other.parts.clone(),
            };
        }

        // Otherwise, concatenate parts
        let mut new_parts = self.parts.clone();
        new_parts.extend(other.parts.iter().cloned());

        ParsedPath {
            drive: self.drive.clone(),
            root: self.root.clone(),
            parts: new_parts,
        }
    }

    /// Join with another path in place (mutates self).
    pub fn join_mut(&mut self, other: &ParsedPath, flavor: PathFlavor) {
        // If other is absolute, it replaces self entirely
        if other.is_absolute(flavor) {
            self.drive = other.drive.clone();
            self.root = other.root.clone();
            self.parts = other.parts.clone();
            return;
        }

        // On Windows, if other has a different drive, it replaces self
        if flavor == PathFlavor::Windows && !other.drive.is_empty() && other.drive != self.drive {
            self.drive = other.drive.clone();
            self.root = other.root.clone();
            self.parts = other.parts.clone();
            return;
        }

        // On Windows, if other has a root (but no drive), keep self's drive
        if flavor == PathFlavor::Windows && !other.root.is_empty() {
            self.root = other.root.clone();
            self.parts = other.parts.clone();
            return;
        }

        // Otherwise, just extend parts
        self.parts.extend(other.parts.iter().cloned());
    }

    /// Get the parent path.
    pub fn parent(&self) -> ParsedPath {
        if self.parts.is_empty() {
            // Already at root or empty - return self
            return self.clone();
        }

        let mut new_parts = self.parts.clone();
        new_parts.pop();

        ParsedPath {
            drive: self.drive.clone(),
            root: self.root.clone(),
            parts: new_parts,
        }
    }

    /// Case-fold for comparison (Windows only).
    pub fn case_fold(&self, flavor: PathFlavor) -> ParsedPath {
        if flavor == PathFlavor::Posix {
            return self.clone();
        }

        ParsedPath {
            drive: self.drive.to_lowercase(),
            root: self.root.clone(),
            parts: self.parts.iter().map(|s| s.to_lowercase()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_posix_simple() {
        let p = ParsedPath::parse("foo/bar", PathFlavor::Posix);
        assert_eq!(p.parts, vec!["foo", "bar"]);
        assert!(p.drive.is_empty());
        assert!(p.root.is_empty());
    }

    #[test]
    fn test_posix_absolute() {
        let p = ParsedPath::parse("/foo/bar", PathFlavor::Posix);
        assert_eq!(p.parts, vec!["foo", "bar"]);
        assert_eq!(p.root, "/");
        assert!(p.is_absolute(PathFlavor::Posix));
    }

    #[test]
    fn test_posix_double_slash() {
        let p = ParsedPath::parse("//foo/bar", PathFlavor::Posix);
        assert_eq!(p.root, "//");
        assert_eq!(p.parts, vec!["foo", "bar"]);
    }

    #[test]
    fn test_windows_drive() {
        let p = ParsedPath::parse("C:\\foo\\bar", PathFlavor::Windows);
        assert_eq!(p.drive, "C:");
        assert_eq!(p.root, "\\");
        assert_eq!(p.parts, vec!["foo", "bar"]);
        assert!(p.is_absolute(PathFlavor::Windows));
    }

    #[test]
    fn test_windows_unc() {
        let p = ParsedPath::parse("\\\\server\\share\\foo", PathFlavor::Windows);
        assert_eq!(p.drive, "\\\\server\\share");
        assert_eq!(p.root, "\\");
        assert_eq!(p.parts, vec!["foo"]);
    }
}
