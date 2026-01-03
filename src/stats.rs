use pyo3::prelude::*;
use std::fs;

/// Stat result object.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct StatResult {
    #[pyo3(get)]
    pub st_mode: u32,
    #[pyo3(get)]
    pub st_size: u64,
    #[pyo3(get)]
    pub st_mtime: f64,
    #[pyo3(get)]
    pub st_atime: f64,
    #[pyo3(get)]
    pub st_ctime: f64,
}

impl StatResult {
    pub(crate) fn from_metadata(metadata: &fs::Metadata) -> Self {
        use std::time::UNIX_EPOCH;

        let st_mode = {
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                metadata.mode()
            }
            #[cfg(not(unix))]
            {
                if metadata.is_dir() {
                    0o40755
                } else if metadata.is_file() {
                    0o100644
                } else {
                    0
                }
            }
        };

        let st_size = metadata.len();

        let st_mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        let st_atime = metadata
            .accessed()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        let st_ctime = metadata
            .created()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        Self {
            st_mode,
            st_size,
            st_mtime,
            st_atime,
            st_ctime,
        }
    }
}

#[pymethods]
impl StatResult {
    fn __repr__(&self) -> String {
        format!(
            "os.stat_result(st_mode={}, st_size={}, st_mtime={}, st_atime={}, st_ctime={})",
            self.st_mode, self.st_size, self.st_mtime, self.st_atime, self.st_ctime
        )
    }
}
