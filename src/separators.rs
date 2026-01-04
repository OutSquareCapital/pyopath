use crate::core::ParsedParts;

pub struct PosixSeparator;
pub struct WindowsSeparator;

impl PosixSeparator {
    const SEP: char = '/';
    pub const MODULE_NAME: &'static str = "posixpath";

    pub fn sep() -> char {
        Self::SEP
    }

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
            .split('/')
            .filter(|p| !p.is_empty() && *p != ".")
            .map(|s| s.to_string())
            .collect();
        ParsedParts { drive, root, parts }
    }

    pub fn splitroot(path: &str) -> (String, String, String) {
        if let Some(rest) = path.strip_prefix('/') {
            (String::new(), "/".to_string(), rest.to_string())
        } else {
            (String::new(), String::new(), path.to_string())
        }
    }

    pub fn with_name(parsed: &ParsedParts, name: &str) -> String {
        let mut new_parts = parsed.parent_parts();
        new_parts.push(name.to_string());
        let body = new_parts.join("/");
        if parsed.root.is_empty() && parsed.drive.is_empty() {
            if body.is_empty() {
                ".".to_string()
            } else {
                body
            }
        } else if body.is_empty() {
            // Just root, no body
            parsed.root.clone()
        } else {
            // root is "/" so we join directly
            format!("{}{}", parsed.root, body)
        }
    }

    pub fn with_suffix(parsed: &ParsedParts, suffix: &str) -> String {
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
        } else if body.is_empty() {
            // Just root, no body
            parsed.root.clone()
        } else {
            // root is "/" so we join directly
            format!("{}{}", parsed.root, body)
        }
    }

    pub fn is_absolute(parsed: &ParsedParts) -> bool {
        !parsed.root.is_empty()
    }
}

impl WindowsSeparator {
    const SEP: char = '\\';
    pub const MODULE_NAME: &'static str = "ntpath";

    pub fn sep() -> char {
        Self::SEP
    }

    /// Normalize a path by converting / to \\ for Windows
    pub fn normalize_path(path: &str) -> String {
        path.replace('/', "\\")
    }

    /// On Windows, case-insensitive: convert to lowercase
    pub fn normalize_case(path: &str) -> String {
        path.to_lowercase()
    }

    pub fn parse(raw_path: &str) -> ParsedParts {
        let normalized = Self::normalize_path(raw_path);
        let (drive, root, rest) = Self::splitroot(&normalized);
        let parts: Vec<String> = rest
            .split(['\\', '/'])
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
            let parts: Vec<&str> = rest.split(['\\', '/']).collect();
            if parts.len() >= 2 {
                // \\server\share is the drive, \ is root, rest is the path
                let drive = format!("\\\\{}\\{}", parts[0], parts[1]);
                let body = parts[2..].join("\\");
                (drive, "\\".to_string(), body)
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
                (drive, "\\".to_string(), path[3..].to_string())
            } else {
                (drive, String::new(), path[2..].to_string())
            }
        } else if let Some(rest) = path.strip_prefix("\\") {
            // Backslash at start, but NOT absolute on Windows without drive
            (String::new(), "\\".to_string(), rest.to_string())
        } else {
            (String::new(), String::new(), path.to_string())
        }
    }

    pub fn with_name(parsed: &ParsedParts, name: &str) -> String {
        let mut new_parts = parsed.parent_parts();
        new_parts.push(name.to_string());
        let body = new_parts.join("\\");
        if parsed.root.is_empty() && parsed.drive.is_empty() {
            if body.is_empty() {
                ".".to_string()
            } else {
                body
            }
        } else if body.is_empty() {
            // Just drive + root, no body
            format!("{}{}", parsed.drive, parsed.root)
        } else {
            // root is "\\" so we join directly
            format!("{}{}{}", parsed.drive, parsed.root, body)
        }
    }

    pub fn with_suffix(parsed: &ParsedParts, suffix: &str) -> String {
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
        } else if body.is_empty() {
            // Just drive + root, no body
            format!("{}{}", parsed.drive, parsed.root)
        } else {
            // root is "\\" so we join directly
            format!("{}{}{}", parsed.drive, parsed.root, body)
        }
    }

    pub fn is_absolute(parsed: &ParsedParts) -> bool {
        // On Windows, absolute means has a drive letter
        !parsed.drive.is_empty()
    }
}
