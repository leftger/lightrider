use crate::byte_load::ByteLoad;
use crate::filesystem::loader::{DirectoryContents, load_directory};
use bevy::prelude::{Message, Resource};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

/// Background directory-scan bookkeeping.
///
/// The generation counter, the single in-flight task slot and the error string
/// live in [`ByteLoad`]; this adds only the scan's own policy: which reader
/// runs, and the hidden-files flag it needs. Each requested scan replaces the
/// previous one.
#[derive(Resource, Default)]
pub struct DirectoryLoadState(ByteLoad<DirectoryContents>);

impl Deref for DirectoryLoadState {
    type Target = ByteLoad<DirectoryContents>;

    fn deref(&self) -> &ByteLoad<DirectoryContents> {
        &self.0
    }
}

impl DerefMut for DirectoryLoadState {
    fn deref_mut(&mut self) -> &mut ByteLoad<DirectoryContents> {
        &mut self.0
    }
}

impl DirectoryLoadState {
    pub fn begin_scan(&mut self, generation: u64, path: PathBuf, show_hidden: bool) {
        self.0.begin_load(generation, path, move |path| {
            load_directory(path, show_hidden).map_err(|error| error.to_string())
        });
    }
}

#[derive(Message, Debug)]
pub struct DirectoryRequested {
    pub path: PathBuf,
}

#[derive(Message, Debug)]
pub struct DirectoryLoaded {
    pub path: PathBuf,
    pub contents: DirectoryContents,
}

#[derive(Message, Debug)]
pub struct DirectoryLoadFailed {
    pub path: PathBuf,
    pub message: String,
}
