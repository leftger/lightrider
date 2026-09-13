//! Which source language a file is written in, by extension.
//!
//! Classifying a file is a filesystem concern, so the allowlist lives down
//! here; `disc::language` keeps what a language *means* to the ring.

use std::path::Path;

/// A source language that may be ridden as a disc-wars ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceLanguage {
    Rust,
    C,
    Cpp,
    Python,
    Slint,
    Lua,
    Shell,
    Toml,
    Json,
    Go,
    Ruby,
    Yaml,
    JavaScript,
    Zig,
    Php,
    R,
}

impl SourceLanguage {
    /// Maps a lowercase file extension to a language, or `None` when the
    /// extension is not part of the allowlist.
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "rs" => Some(Self::Rust),
            // Headers are their own ring for now; the plan leaves linking a
            // header to its `.c` to phase 2.
            "c" | "h" => Some(Self::C),
            "cc" | "cpp" | "cxx" | "hpp" | "hh" | "hxx" => Some(Self::Cpp),
            "py" | "pyi" => Some(Self::Python),
            "slint" => Some(Self::Slint),
            "lua" => Some(Self::Lua),
            "sh" | "bash" | "zsh" => Some(Self::Shell),
            "toml" => Some(Self::Toml),
            "json" => Some(Self::Json),
            "go" => Some(Self::Go),
            "rb" => Some(Self::Ruby),
            "yaml" | "yml" => Some(Self::Yaml),
            "js" | "mjs" => Some(Self::JavaScript),
            "zig" => Some(Self::Zig),
            "php" => Some(Self::Php),
            "r" => Some(Self::R),
            _ => None,
        }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|extension| extension.to_str())
            .and_then(Self::from_extension)
    }
}
