#[derive(Clone, Debug)]
pub struct ParsedParts {
    pub drive: String,
    pub root: String,
    pub parts: Vec<String>,
}

impl ParsedParts {
    pub fn anchor(&self) -> String {
        format!("{}{}", self.drive, self.root)
    }

    pub fn all_parts(&self) -> Vec<String> {
        let mut result = Vec::new();
        if !self.drive.is_empty() || !self.root.is_empty() {
            result.push(self.anchor());
        }
        result.extend(self.parts.iter().cloned());
        result
    }

    pub fn name(&self) -> String {
        self.parts.last().cloned().unwrap_or_default()
    }

    pub fn parent_parts(&self) -> Vec<String> {
        if self.parts.is_empty() {
            self.parts.clone()
        } else {
            self.parts[..self.parts.len() - 1].to_vec()
        }
    }

    pub fn stem(&self) -> String {
        let name = self.name();
        // Special case: "." and ".." should return themselves
        if name == "." || name == ".." {
            return name;
        }
        // Find the LAST dot, but not if it's at the start of the name
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

    pub fn suffix(&self) -> String {
        let name = self.name();
        // Special case: "." and ".." have no suffix
        if name == "." || name == ".." {
            return String::new();
        }
        // Find the LAST dot, but not if it's at the start of the name
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

    pub fn suffixes(&self) -> Vec<String> {
        let name = self.name();
        // Special case: "." and ".." have no suffixes
        if name == "." || name == ".." {
            return Vec::new();
        }

        let mut result = Vec::new();
        // Find first dot - if it's at position 0, no suffixes
        if let Some(first_dot) = name.find('.') {
            if first_dot == 0 {
                // File starts with dot like ".gitignore" - no suffixes
                return result;
            }
            // For each part after the first split (which is the first dot itself),
            // add ".part" as a suffix
            for part in &name[first_dot..].split('.').collect::<Vec<&str>>()[1..] {
                if !part.is_empty() {
                    result.push(format!(".{}", part));
                }
            }
        }
        result
    }
}
