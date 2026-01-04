use crate::core::ParsedParts;

pub struct PosixSeparator;
pub struct WindowsSeparator;

impl PosixSeparator {
    pub const SEP: char = '/';
    pub const MODULE_NAME: &'static str = "posixpath";

    /// On Posix, no normalization needed
    pub fn normalize_path(path: &str) -> String {
        path.to_string()
    }

    /// On Posix, case-sensitive: return as-is
    pub fn normalize_case(path: &str) -> String {
        path.to_string()
    }

    pub fn parse(raw_path: &str) -> ParsedParts {
        let (drive, root, rest) = Self::splitroot(raw_path);
        let parts: Vec<String> = rest
            .split(Self::SEP)
            .filter(|p| !p.is_empty() && *p != ".")
            .map(|s| s.to_string())
            .collect();
        ParsedParts { drive, root, parts }
    }

    pub fn splitroot(path: &str) -> (String, String, String) {
        if let Some(rest) = path.strip_prefix(Self::SEP) {
            (String::new(), Self::SEP.to_string(), rest.to_string())
        } else {
            (String::new(), String::new(), path.to_string())
        }
    }

    pub fn with_name(parsed: &ParsedParts, name: &str) -> ParsedParts {
        let mut new_parts = parsed.parent_parts();
        new_parts.push(name.to_string());
        ParsedParts {
            drive: parsed.drive.clone(),
            root: parsed.root.clone(),
            parts: new_parts,
        }
    }

    pub fn with_suffix(parsed: &ParsedParts, suffix: &str) -> ParsedParts {
        let mut new_parts = parsed.parent_parts();
        let stem = parsed.stem();
        new_parts.push(format!("{}{}", stem, suffix));
        ParsedParts {
            drive: parsed.drive.clone(),
            root: parsed.root.clone(),
            parts: new_parts,
        }
    }

    pub fn is_absolute(parsed: &ParsedParts) -> bool {
        !parsed.root.is_empty()
    }

    /// Format ParsedParts back to a string path
    /// Equivalent to Python's _format_parsed_parts
    pub fn format_parsed_parts(parsed: &ParsedParts) -> String {
        if !parsed.drive.is_empty() || !parsed.root.is_empty() {
            // Has anchor: drive + root + parts
            format!(
                "{}{}{}",
                parsed.drive,
                parsed.root,
                parsed.parts.join(&Self::SEP.to_string())
            )
        } else if !parsed.parts.is_empty()
            && parsed.parts[0].len() >= 2
            && parsed.parts[0].as_bytes()[1] == b':'
        {
            // First part looks like a drive letter - add "." prefix
            let mut parts_with_dot = vec![".".to_string()];
            parts_with_dot.extend(parsed.parts.clone());
            parts_with_dot.join(&Self::SEP.to_string())
        } else {
            // No anchor, just join parts
            let joined = parsed.parts.join(&Self::SEP.to_string());
            if joined.is_empty() {
                ".".to_string()
            } else {
                joined
            }
        }
    }
}

impl WindowsSeparator {
    pub const SEP: char = '\\';
    pub const MODULE_NAME: &'static str = "ntpath";

    /// Normalize a path by converting / to \\ for Windows
    pub fn normalize_path(path: &str) -> String {
        path.replace(PosixSeparator::SEP, &Self::SEP.to_string())
    }

    /// On Windows, case-insensitive: convert to lowercase
    pub fn normalize_case(path: &str) -> String {
        path.to_lowercase()
    }

    pub fn parse(raw_path: &str) -> ParsedParts {
        let normalized = Self::normalize_path(raw_path);
        let (drive, root, rest) = Self::splitroot(&normalized);
        let parts: Vec<String> = rest
            .split([Self::SEP, PosixSeparator::SEP])
            .filter(|p| !p.is_empty() && *p != ".")
            .map(|s| s.to_string())
            .collect();
        ParsedParts { drive, root, parts }
    }

    pub fn splitroot(path: &str) -> (String, String, String) {
        // Handle UNC paths (\\server\share)
        if let Some(rest) = path.strip_prefix("\\\\") {
            // UNC path: \\server\share\file
            // Need to find the share part
            let parts: Vec<&str> = rest.split([Self::SEP, PosixSeparator::SEP]).collect();
            if parts.len() >= 2 {
                // \\server\share is the drive, \ is root, rest is the path
                let drive = format!("\\\\{}\\{}", parts[0], parts[1]);
                let body = parts[2..].join(&Self::SEP.to_string());
                (drive, Self::SEP.to_string(), body)
            } else if parts.len() == 1 {
                // Just \\server without share
                let drive = format!("\\\\{}", parts[0]);
                (drive, String::new(), String::new())
            } else {
                // Edge case: just \\
                (String::new(), "\\\\".to_string(), String::new())
            }
        } else if path.len() >= 2 && path.as_bytes()[1] == b':' {
            // Drive letter: "C:..."
            let drive = path[..2].to_string();
            if path.len() > 2 && (path.as_bytes()[2] == b'\\' || path.as_bytes()[2] == b'/') {
                // C:\... or C:/... → Both make it absolute with drive
                (drive, Self::SEP.to_string(), path[3..].to_string())
            } else {
                (drive, String::new(), path[2..].to_string())
            }
        } else if let Some(rest) = path.strip_prefix(Self::SEP) {
            // Backslash at start, but NOT absolute on Windows without drive
            (String::new(), Self::SEP.to_string(), rest.to_string())
        } else {
            (String::new(), String::new(), path.to_string())
        }
    }

    pub fn with_name(parsed: &ParsedParts, name: &str) -> ParsedParts {
        let mut new_parts = parsed.parent_parts();
        new_parts.push(name.to_string());
        ParsedParts {
            drive: parsed.drive.clone(),
            root: parsed.root.clone(),
            parts: new_parts,
        }
    }

    pub fn with_suffix(parsed: &ParsedParts, suffix: &str) -> ParsedParts {
        let mut new_parts = parsed.parent_parts();
        let stem = parsed.stem();
        new_parts.push(format!("{}{}", stem, suffix));
        ParsedParts {
            drive: parsed.drive.clone(),
            root: parsed.root.clone(),
            parts: new_parts,
        }
    }

    pub fn is_absolute(parsed: &ParsedParts) -> bool {
        // On Windows, absolute means has a drive letter
        !parsed.drive.is_empty()
    }

    /// Format ParsedParts back to a string path
    /// Equivalent to Python's _format_parsed_parts
    pub fn format_parsed_parts(parsed: &ParsedParts) -> String {
        if !parsed.drive.is_empty() || !parsed.root.is_empty() {
            // Has anchor: drive + root + parts
            format!(
                "{}{}{}",
                parsed.drive,
                parsed.root,
                parsed.parts.join(&Self::SEP.to_string())
            )
        } else if !parsed.parts.is_empty()
            && parsed.parts[0].len() >= 2
            && parsed.parts[0].as_bytes()[1] == b':'
        {
            // First part looks like a drive letter - add "." prefix
            let mut parts_with_dot = vec![".".to_string()];
            parts_with_dot.extend(parsed.parts.clone());
            parts_with_dot.join(&Self::SEP.to_string())
        } else {
            // No anchor, just join parts
            let joined = parsed.parts.join(&Self::SEP.to_string());
            if joined.is_empty() {
                ".".to_string()
            } else {
                joined
            }
        }
    }
}
