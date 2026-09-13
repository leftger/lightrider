//! The background byte-loading state machine the document and source loaders
//! share.
//!
//! Both spelled out the same generation counter, the same single in-flight task
//! slot and the same error string, differing only in what they read and what they
//! call it; the disc side even documented itself as mirroring the document side.
//! The reader is passed in now instead of being written twice.

use bevy::tasks::{IoTaskPool, Task, futures::check_ready};
use std::path::{Path, PathBuf};

/// A finished read: which request it answers, what was asked for, and what came
/// back.
pub struct ByteLoadResult<T> {
    pub generation: u64,
    pub path: PathBuf,
    pub result: Result<T, String>,
}

/// Tracks one background read at a time, so a result that arrives after the
/// player has moved on can be recognised as stale.
///
/// The payload is generic so a directory scan, a document and a source file all
/// share the same bookkeeping; each loader supplies its own reader and result.
pub struct ByteLoad<T> {
    pub generation: u64,
    pub loading: bool,
    pub pending_task: Option<Task<ByteLoadResult<T>>>,
    pub last_error: Option<String>,
}

impl<T> Default for ByteLoad<T> {
    fn default() -> Self {
        Self {
            generation: 0,
            loading: false,
            pending_task: None,
            last_error: None,
        }
    }
}

impl<T: Send + 'static> ByteLoad<T> {
    /// Starts a new request, invalidating whatever was in flight.
    pub fn next_generation(&mut self) -> u64 {
        self.generation += 1;
        self.generation
    }

    /// Takes a finished read, if one is ready.
    pub fn poll(&mut self) -> Option<ByteLoadResult<T>> {
        let result = check_ready(self.pending_task.as_mut()?)?;
        self.pending_task = None;
        if result.generation == self.generation {
            self.loading = false;
        }
        Some(result)
    }

    /// Hands the read to the IO pool.
    pub fn begin_load(
        &mut self,
        generation: u64,
        path: PathBuf,
        read: impl FnOnce(&Path) -> Result<T, String> + Send + 'static,
    ) {
        self.loading = true;
        self.last_error = None;
        self.pending_task = Some(IoTaskPool::get().spawn(async move {
            let result = read(&path);
            ByteLoadResult {
                generation,
                path,
                result,
            }
        }));
    }
}

/// Reads a file, refusing anything wildly past `cap` and truncating anything
/// merely past it. `what` names the thing in the error, so the status line can
/// say which file was too big.
pub fn read_capped(path: &Path, cap: usize, what: &str) -> Result<Vec<u8>, String> {
    let metadata = std::fs::metadata(path).map_err(|error| error.to_string())?;
    if metadata.is_dir() {
        return Err("path is a directory".to_string());
    }
    if metadata.len() > cap as u64 * 4 {
        return Err(format!("{what} larger than {cap} bytes"));
    }
    let mut bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    if bytes.len() > cap {
        bytes.truncate(cap);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::read_capped;
    use std::io::Write;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("raptor-byte-load-{}-{name}", std::process::id()));
        path
    }

    #[test]
    fn a_small_file_reads_straight_through() {
        let path = temp_path("small.txt");
        std::fs::File::create(&path)
            .unwrap()
            .write_all(b"hello")
            .unwrap();
        assert_eq!(read_capped(&path, 64, "file").unwrap(), b"hello");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn a_file_past_the_cap_is_truncated() {
        let path = temp_path("long.txt");
        std::fs::File::create(&path)
            .unwrap()
            .write_all(b"0123456789")
            .unwrap();
        assert_eq!(read_capped(&path, 4, "file").unwrap(), b"0123");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn a_file_far_past_the_cap_is_refused() {
        let path = temp_path("huge.txt");
        std::fs::File::create(&path)
            .unwrap()
            .write_all(b"0123456789")
            .unwrap();
        let error = read_capped(&path, 2, "file").unwrap_err();
        assert!(error.contains("file larger than 2 bytes"), "{error}");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn a_directory_is_rejected() {
        assert!(read_capped(&std::env::temp_dir(), 64, "file").is_err());
    }

    #[test]
    fn a_missing_file_is_an_error_not_a_panic() {
        assert!(read_capped(&temp_path("missing.txt"), 64, "file").is_err());
    }
}
