//! Background byte loader for source files.
//!
//! The state machine and the capped reader live in [`crate::byte_load`]; what is
//! left here is this loader's policy: the source cap, so a generated or minified
//! dump cannot stall a frame, and the name it goes by when it is refused.
//! Truncation is reported back so the status line can say so.

use crate::byte_load::{ByteLoad, read_capped};
use crate::config;
use crate::disc::language::SourceGame;
use bevy::prelude::{Message, Resource};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

#[derive(Resource, Default)]
pub struct SourceLoadState(ByteLoad<Vec<u8>>);

impl Deref for SourceLoadState {
    type Target = ByteLoad<Vec<u8>>;

    fn deref(&self) -> &ByteLoad<Vec<u8>> {
        &self.0
    }
}

impl DerefMut for SourceLoadState {
    fn deref_mut(&mut self) -> &mut ByteLoad<Vec<u8>> {
        &mut self.0
    }
}

impl SourceLoadState {
    pub fn begin_load(&mut self, generation: u64, path: PathBuf) {
        self.0.begin_load(generation, path, load_source_bytes);
    }
}

#[derive(Message, Debug)]
pub struct SourceRequested {
    pub path: PathBuf,
}

/// Warp straight into a game without needing a real file of that language.
#[derive(Message, Debug)]
pub struct WarpRequested {
    pub game: SourceGame,
}

#[derive(Message, Debug)]
pub struct SourceLoaded {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
}

#[derive(Message, Debug)]
pub struct SourceLoadFailed {
    pub path: PathBuf,
    pub message: String,
}

fn load_source_bytes(path: &std::path::Path) -> Result<Vec<u8>, String> {
    read_capped(path, config::SOURCE_MAX_BYTES, "source file")
}

#[cfg(test)]
mod tests {
    use super::load_source_bytes;
    use std::io::Write;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("raptor-source-load-{}-{name}", std::process::id()));
        path
    }

    /// The shared reader covers the error cases; what matters here is that the
    /// source loader is wired to it.
    #[test]
    fn reading_a_source_file_returns_its_bytes() {
        let path = temp_path("ok.rs");
        std::fs::File::create(&path)
            .unwrap()
            .write_all(b"fn main() {}")
            .unwrap();
        assert_eq!(load_source_bytes(&path).unwrap(), b"fn main() {}");
        let _ = std::fs::remove_file(path);
    }
}
