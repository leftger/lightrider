//! Background byte loader for markdown documents.
//!
//! The state machine and the capped reader live in [`crate::byte_load`]; what is
//! left here is this loader's policy: which cap applies, and what the file is
//! called when it is refused.

use crate::byte_load::{ByteLoad, read_capped};
use crate::config;
use bevy::prelude::{Message, Resource};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

#[derive(Resource, Default)]
pub struct DocumentLoadState(ByteLoad);

impl Deref for DocumentLoadState {
    type Target = ByteLoad;

    fn deref(&self) -> &ByteLoad {
        &self.0
    }
}

impl DerefMut for DocumentLoadState {
    fn deref_mut(&mut self) -> &mut ByteLoad {
        &mut self.0
    }
}

impl DocumentLoadState {
    pub fn begin_load(&mut self, generation: u64, path: PathBuf) {
        self.0.begin_load(generation, path, load_document_bytes);
    }
}

#[derive(Message, Debug)]
pub struct DocumentRequested {
    pub path: PathBuf,
}

#[derive(Message, Debug)]
pub struct DocumentLoaded {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
}

#[derive(Message, Debug)]
pub struct DocumentLoadFailed {
    pub path: PathBuf,
    pub message: String,
}

fn load_document_bytes(path: &std::path::Path) -> Result<Vec<u8>, String> {
    read_capped(path, config::DOCUMENT_MAX_BYTES, "document")
}

#[cfg(test)]
mod tests {
    use super::load_document_bytes;
    use std::io::Write;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "raptor-document-load-{}-{name}",
            std::process::id()
        ));
        path
    }

    /// The shared reader covers the error cases; what matters here is that the
    /// document loader is wired to it.
    #[test]
    fn reading_a_document_returns_its_bytes() {
        let path = temp_path("ok.md");
        std::fs::File::create(&path)
            .unwrap()
            .write_all(b"# title")
            .unwrap();
        assert_eq!(load_document_bytes(&path).unwrap(), b"# title");
        let _ = std::fs::remove_file(path);
    }
}
