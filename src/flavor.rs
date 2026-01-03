//! Path flavor definitions - POSIX vs Windows semantics.

/// Path flavor determines parsing and comparison semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathFlavor {
    Posix,
    Windows,
}

impl PathFlavor {
    /// Returns the separator for this flavor.
    pub fn sep(&self) -> char {
        match self {
            PathFlavor::Posix => '/',
            PathFlavor::Windows => '\\',
        }
    }

    /// Returns the alternate separator for this flavor (if any).
    pub fn altsep(&self) -> Option<char> {
        match self {
            PathFlavor::Posix => None,
            PathFlavor::Windows => Some('/'),
        }
    }

    /// Check if a character is a separator for this flavor.
    pub fn is_sep(&self, c: char) -> bool {
        c == self.sep() || self.altsep().is_some_and(|alt| c == alt)
    }

    /// Returns the current platform's flavor.
    #[cfg(windows)]
    pub fn current() -> Self {
        PathFlavor::Windows
    }

    /// Returns the current platform's flavor.
    #[cfg(not(windows))]
    pub fn current() -> Self {
        PathFlavor::Posix
    }
}
