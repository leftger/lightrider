//! Source-language identity for a disc-wars ring.
//!
//! Each language is a ring type, not a different game (see
//! `docs/disc-wars-plan.md`, section 1). The allowlist itself lives in
//! [`crate::filesystem::language`]; this module owns what a language means to
//! the ring: the game it opens, the compiler name the opponent dais carries,
//! the ring accent colour, and the arpeggiator tint applied while it is open.

use crate::config;
use crate::filesystem::language::SourceLanguage;
use bevy::prelude::{Color, Vec3};

/// Which mini-game a source file opens.
///
/// The ring geometry is shared; only the game inside it changes. Python files
/// are a snake run, C files an asteroid field, Rust/C++ a disc-wars ring, TOML
/// files a river-surfer course, JSON files a Swarm Shooter field, Go files a Byte Muncher
/// maze, Ruby files a Falling Gems well, YAML files Block Fall, JavaScript files
/// Grid Hopper, Zig files Cube Hopper, PHP files Grid Bomber, and R files a Plinko board.
/// Moving a game to another extension is a one-line change in
/// [`SourceLanguage::game`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceGame {
    DiscWars,
    AsteroidField,
    Snake,
    Platformer,
    Breaker,
    Stealth,
    RiverSurfer,
    SwarmShooter,
    ByteMuncher,
    FallingGems,
    BlockFall,
    GridHopper,
    CubeHopper,
    GridBomber,
    Plinko,
}

impl SourceGame {
    /// How many games there are, for fixed-size tables and the warp menu.
    pub const COUNT: usize = 15;

    /// Every game, in menu order.
    pub const ALL: [SourceGame; Self::COUNT] = [
        SourceGame::DiscWars,
        SourceGame::AsteroidField,
        SourceGame::Snake,
        SourceGame::Platformer,
        SourceGame::Breaker,
        SourceGame::Stealth,
        SourceGame::RiverSurfer,
        SourceGame::SwarmShooter,
        SourceGame::ByteMuncher,
        SourceGame::FallingGems,
        SourceGame::BlockFall,
        SourceGame::GridHopper,
        SourceGame::CubeHopper,
        SourceGame::GridBomber,
        SourceGame::Plinko,
    ];

    /// Display name used by the pause/warp menu.
    pub fn label(self) -> &'static str {
        match self {
            Self::DiscWars => "Disc Wars",
            Self::AsteroidField => "Asteroid Field",
            Self::Snake => "Snake",
            Self::Platformer => "Platformer",
            Self::Breaker => "Brick Breaker",
            Self::Stealth => "Stealth",
            Self::RiverSurfer => "River Surfer",
            Self::SwarmShooter => "Swarm Shooter",
            Self::ByteMuncher => "Byte Muncher",
            Self::FallingGems => "Falling Gems",
            Self::BlockFall => "Block Fall",
            Self::GridHopper => "Grid Hopper",
            Self::CubeHopper => "Cube Hopper",
            Self::GridBomber => "Grid Bomber",
            Self::Plinko => "Plinko",
        }
    }

    /// Whether a run of this game steers the shared cycle grid. Disc wars, snake
    /// and the asteroid field ride the grid; every off-grid game drives itself.
    /// The field only counts once its rocks are gone, which the caller checks.
    pub fn drives_grid(self) -> bool {
        matches!(self, Self::DiscWars | Self::Snake | Self::AsteroidField)
    }

    /// The fixed camera an arcade-block board is watched from, if this game has
    /// one: where the camera stands and what it looks at. `None` for every game
    /// that keeps the chase camera.
    pub fn arcade_camera(self) -> Option<(Vec3, Vec3)> {
        let (translation, target) = match self {
            Self::ByteMuncher => (
                Vec3::new(
                    0.0,
                    config::arcade::MUNCHER_CAMERA_HEIGHT,
                    config::arcade::MUNCHER_CAMERA_HEIGHT * config::arcade::MUNCHER_CAMERA_LEAN,
                ),
                Vec3::ZERO,
            ),
            Self::GridHopper => (
                Vec3::new(
                    0.0,
                    config::arcade::GRID_HOPPER_CAMERA_HEIGHT,
                    config::arcade::GRID_HOPPER_CAMERA_HEIGHT * config::arcade::GRID_HOPPER_CAMERA_LEAN,
                ),
                Vec3::ZERO,
            ),
            Self::CubeHopper => (
                Vec3::new(
                    0.0,
                    config::arcade::CUBE_HOPPER_CAMERA_HEIGHT,
                    -config::arcade::CUBE_HOPPER_CAMERA_HEIGHT * config::arcade::CUBE_HOPPER_CAMERA_LEAN,
                ),
                Vec3::new(0.0, 1.0, 0.0),
            ),
            Self::GridBomber => (
                Vec3::new(
                    0.0,
                    config::arcade::GRID_BOMBER_CAMERA_HEIGHT,
                    config::arcade::GRID_BOMBER_CAMERA_HEIGHT * config::arcade::GRID_BOMBER_CAMERA_LEAN,
                ),
                Vec3::ZERO,
            ),
            Self::FallingGems => (
                Vec3::new(0.0, 10.4, config::arcade::FALLING_GEMS_CAMERA_BACK),
                Vec3::new(0.0, 10.4, 0.0),
            ),
            Self::BlockFall => (
                Vec3::new(0.0, 12.0, config::arcade::BLOCK_FALL_CAMERA_BACK),
                Vec3::new(0.0, 12.0, 0.0),
            ),
            Self::Plinko => (
                Vec3::new(0.0, 0.0, config::arcade::PLINKO_CAMERA_BACK),
                Vec3::ZERO,
            ),
            _ => return None,
        };
        Some((translation, target))
    }
}

impl SourceLanguage {
    /// How many languages the allowlist covers, for fixed-size tables.
    pub const COUNT: usize = 16;

    /// Every language, for tests that need to cover them all.
    pub const ALL: [SourceLanguage; SourceLanguage::COUNT] = [
        SourceLanguage::Rust,
        SourceLanguage::C,
        SourceLanguage::Cpp,
        SourceLanguage::Python,
        SourceLanguage::Slint,
        SourceLanguage::Lua,
        SourceLanguage::Shell,
        SourceLanguage::Toml,
        SourceLanguage::Json,
        SourceLanguage::Go,
        SourceLanguage::Ruby,
        SourceLanguage::Yaml,
        SourceLanguage::JavaScript,
        SourceLanguage::Zig,
        SourceLanguage::Php,
        SourceLanguage::R,
    ];

    /// Short name used in the HUD and the dais label.
    pub fn name(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::Python => "python",
            Self::Slint => "slint",
            Self::Lua => "lua",
            Self::Shell => "shell",
            Self::Toml => "toml",
            Self::Json => "json",
            Self::Go => "go",
            Self::Ruby => "ruby",
            Self::Yaml => "yaml",
            Self::JavaScript => "js",
            Self::Zig => "zig",
            Self::Php => "php",
            Self::R => "r",
        }
    }

    /// Which mini-game a file of this language hosts. See [`SourceGame`].
    pub fn game(self) -> SourceGame {
        match self {
            Self::Python => SourceGame::Snake,
            Self::C => SourceGame::AsteroidField,
            Self::Rust | Self::Cpp => SourceGame::DiscWars,
            Self::Slint => SourceGame::Platformer,
            Self::Lua => SourceGame::Breaker,
            Self::Shell => SourceGame::Stealth,
            Self::Toml => SourceGame::RiverSurfer,
            Self::Json => SourceGame::SwarmShooter,
            Self::Go => SourceGame::ByteMuncher,
            Self::Ruby => SourceGame::FallingGems,
            Self::Yaml => SourceGame::BlockFall,
            Self::JavaScript => SourceGame::GridHopper,
            Self::Zig => SourceGame::CubeHopper,
            Self::Php => SourceGame::GridBomber,
            Self::R => SourceGame::Plinko,
        }
    }

    /// A representative language hosting the given game, for the warp menu's
    /// synthetic file.
    pub fn for_game(game: SourceGame) -> Self {
        Self::ALL
            .iter()
            .copied()
            .find(|language| language.game() == game)
            .unwrap_or(Self::Rust)
    }

    /// Tool the "compiler" opponent is themed after. Flavor text only; nothing
    /// here is a real diagnostic.
    pub fn compiler(self) -> &'static str {
        match self {
            Self::Rust => "rustc",
            Self::C => "cc",
            Self::Cpp => "c++",
            Self::Python => "CPython",
            Self::Slint => "slint-build",
            Self::Lua => "lua",
            Self::Shell => "sh",
            Self::Toml => "taplo",
            Self::Json => "jq",
            Self::Go => "go",
            Self::Ruby => "ruby",
            Self::Yaml => "yamllint",
            Self::JavaScript => "node",
            Self::Zig => "zig",
            Self::Php => "php",
            Self::R => "Rscript",
        }
    }

    /// Accent used for the ring wall trim, the gate, and the floor markings.
    pub fn accent(self) -> Color {
        match self {
            Self::Rust => config::disc::DISC_RUST_ACCENT,
            Self::C => config::disc::DISC_C_ACCENT,
            Self::Cpp => config::disc::DISC_CPP_ACCENT,
            Self::Python => config::disc::DISC_PYTHON_ACCENT,
            Self::Slint => config::disc::DISC_SLINT_ACCENT,
            Self::Lua => config::disc::DISC_LUA_ACCENT,
            Self::Shell => config::disc::DISC_SHELL_ACCENT,
            Self::Toml => config::disc::DISC_TOML_ACCENT,
            Self::Json => config::disc::DISC_JSON_ACCENT,
            Self::Go => config::disc::DISC_GO_ACCENT,
            Self::Ruby => config::disc::DISC_RUBY_ACCENT,
            Self::Yaml => config::disc::DISC_YAML_ACCENT,
            Self::JavaScript => config::disc::DISC_JS_ACCENT,
            Self::Zig => config::disc::DISC_ZIG_ACCENT,
            Self::Php => config::disc::DISC_PHP_ACCENT,
            Self::R => config::disc::DISC_R_ACCENT,
        }
    }

    /// Arpeggiator gain multiplier while the ring is open.
    pub fn arp_gain_scale(self) -> f32 {
        match self {
            Self::Rust => config::music::MUSIC_DISC_RUST_ARP_GAIN,
            Self::C => config::music::MUSIC_DISC_C_ARP_GAIN,
            Self::Cpp => config::music::MUSIC_DISC_CPP_ARP_GAIN,
            Self::Python => config::music::MUSIC_DISC_PYTHON_ARP_GAIN,
            Self::Slint => config::music::MUSIC_DISC_SLINT_ARP_GAIN,
            Self::Lua => config::music::MUSIC_DISC_LUA_ARP_GAIN,
            Self::Shell => config::music::MUSIC_DISC_SHELL_ARP_GAIN,
            Self::Toml => config::music::MUSIC_DISC_TOML_ARP_GAIN,
            Self::Json => config::music::MUSIC_DISC_JSON_ARP_GAIN,
            Self::Go => config::music::MUSIC_DISC_GO_ARP_GAIN,
            Self::Ruby => config::music::MUSIC_DISC_RUBY_ARP_GAIN,
            Self::Yaml => config::music::MUSIC_DISC_YAML_ARP_GAIN,
            Self::JavaScript => config::music::MUSIC_DISC_JS_ARP_GAIN,
            Self::Zig => config::music::MUSIC_DISC_ZIG_ARP_GAIN,
            Self::Php => config::music::MUSIC_DISC_PHP_ARP_GAIN,
            Self::R => config::music::MUSIC_DISC_R_ARP_GAIN,
        }
    }

    /// Arpeggiator step-rate multiplier while the ring is open. Rust steps
    /// harder, Python pumps slower, so the fight reads differently per language
    /// without changing the folder's key or scale.
    pub fn arp_rate_scale(self) -> f32 {
        match self {
            Self::Rust => config::music::MUSIC_DISC_RUST_ARP_RATE,
            Self::C => config::music::MUSIC_DISC_C_ARP_RATE,
            Self::Cpp => config::music::MUSIC_DISC_CPP_ARP_RATE,
            Self::Python => config::music::MUSIC_DISC_PYTHON_ARP_RATE,
            Self::Slint => config::music::MUSIC_DISC_SLINT_ARP_RATE,
            Self::Lua => config::music::MUSIC_DISC_LUA_ARP_RATE,
            Self::Shell => config::music::MUSIC_DISC_SHELL_ARP_RATE,
            Self::Toml => config::music::MUSIC_DISC_TOML_ARP_RATE,
            Self::Json => config::music::MUSIC_DISC_JSON_ARP_RATE,
            Self::Go => config::music::MUSIC_DISC_GO_ARP_RATE,
            Self::Ruby => config::music::MUSIC_DISC_RUBY_ARP_RATE,
            Self::Yaml => config::music::MUSIC_DISC_YAML_ARP_RATE,
            Self::JavaScript => config::music::MUSIC_DISC_JS_ARP_RATE,
            Self::Zig => config::music::MUSIC_DISC_ZIG_ARP_RATE,
            Self::Php => config::music::MUSIC_DISC_PHP_ARP_RATE,
            Self::R => config::music::MUSIC_DISC_R_ARP_RATE,
        }
    }

    /// Deterministic crash flavor, e.g. `error[E0308]`. Never a real diagnostic.
    pub fn crash_flavor(self, seed: u64) -> &'static str {
        match self {
            Self::Rust => "error[E0308]: mismatched types",
            Self::C => {
                if seed & 1 == 0 {
                    "Segmentation fault (core dumped)"
                } else {
                    "warning: implicit declaration of function"
                }
            }
            Self::Cpp => {
                if seed & 1 == 0 {
                    "error: no matching function for call"
                } else {
                    "undefined reference to `vtable'"
                }
            }
            Self::Python => {
                if seed & 1 == 0 {
                    "Traceback (most recent call last)"
                } else {
                    "IndentationError: unexpected indent"
                }
            }
            Self::Slint => {
                if seed & 1 == 0 {
                    "error: unknown property"
                } else {
                    "error: cannot convert to length"
                }
            }
            Self::Lua => {
                if seed & 1 == 0 {
                    "attempt to index a nil value"
                } else {
                    "unexpected symbol near '='"
                }
            }
            Self::Shell => {
                if seed & 1 == 0 {
                    "command not found"
                } else {
                    "unbound variable"
                }
            }
            Self::Toml => {
                if seed & 1 == 0 {
                    "invalid value: expected string"
                } else {
                    "duplicate key in table"
                }
            }
            Self::Json => {
                if seed & 1 == 0 {
                    "unexpected end of JSON input"
                } else {
                    "invalid value: expected `,`"
                }
            }
            Self::Go => {
                if seed & 1 == 0 {
                    "panic: runtime error: index out of range"
                } else {
                    "fatal error: all goroutines are asleep - deadlock!"
                }
            }
            Self::Ruby => {
                if seed & 1 == 0 {
                    "NoMethodError: undefined method for nil"
                } else {
                    "SyntaxError: unexpected end-of-input"
                }
            }
            Self::Yaml => {
                if seed & 1 == 0 {
                    "mapping values are not allowed in this context"
                } else {
                    "found character that cannot start any token"
                }
            }
            Self::JavaScript => {
                if seed & 1 == 0 {
                    "TypeError: cannot read properties of undefined"
                } else {
                    "SyntaxError: Unexpected token"
                }
            }
            Self::Zig => {
                if seed & 1 == 0 {
                    "error: expected type expression, found '}'"
                } else {
                    "error: overflow in arithmetic operation"
                }
            }
            Self::Php => {
                if seed & 1 == 0 {
                    "PHP Fatal error: Uncaught Error: Call to undefined function"
                } else {
                    "Parse error: syntax error, unexpected end of file"
                }
            }
            Self::R => {
                if seed & 1 == 0 {
                    "Error: object 'x' not found"
                } else {
                    "Error in parse: unexpected symbol"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SourceLanguage;
    use std::path::PathBuf;

    #[test]
    fn the_allowlist_covers_the_planned_extensions() {
        for (path, language) in [
            ("main.rs", SourceLanguage::Rust),
            ("lib.c", SourceLanguage::C),
            ("api.h", SourceLanguage::C),
            ("engine.cpp", SourceLanguage::Cpp),
            ("widget.hpp", SourceLanguage::Cpp),
            ("tool.py", SourceLanguage::Python),
            ("stubs.pyi", SourceLanguage::Python),
            ("app.slint", SourceLanguage::Slint),
            ("init.lua", SourceLanguage::Lua),
            ("build.sh", SourceLanguage::Shell),
            ("run.zsh", SourceLanguage::Shell),
            ("Cargo.toml", SourceLanguage::Toml),
            ("data.json", SourceLanguage::Json),
            ("main.go", SourceLanguage::Go),
            ("gems.rb", SourceLanguage::Ruby),
            ("config.yaml", SourceLanguage::Yaml),
            ("config.yml", SourceLanguage::Yaml),
            ("app.js", SourceLanguage::JavaScript),
            ("app.mjs", SourceLanguage::JavaScript),
            ("main.zig", SourceLanguage::Zig),
            ("index.php", SourceLanguage::Php),
            ("analysis.r", SourceLanguage::R),
        ] {
            assert_eq!(
                SourceLanguage::from_path(&PathBuf::from(path)),
                Some(language),
                "{path}"
            );
        }
    }

    #[test]
    fn non_source_extensions_are_excluded() {
        for path in [
            "README.md",
            "photo.png",
            "Cargo.lock",
            "archive.tar.gz",
            "noext",
        ] {
            assert_eq!(
                SourceLanguage::from_path(&PathBuf::from(path)),
                None,
                "{path}"
            );
        }
    }

    #[test]
    fn detection_is_case_insensitive() {
        assert_eq!(
            SourceLanguage::from_path(&PathBuf::from("MAIN.RS")),
            Some(SourceLanguage::Rust)
        );
    }

    #[test]
    fn every_language_has_a_distinct_identity() {
        let mut names = std::collections::BTreeSet::new();
        let mut accents = std::collections::BTreeSet::new();
        for language in SourceLanguage::ALL {
            assert!(names.insert(language.name()));
            assert!(!language.compiler().is_empty());
            accents.insert(format!("{:?}", language.accent()));
        }
        assert_eq!(accents.len(), SourceLanguage::ALL.len());
    }

    #[test]
    fn each_language_hosts_its_own_game() {
        use super::SourceGame;
        assert_eq!(SourceLanguage::Python.game(), SourceGame::Snake);
        assert_eq!(SourceLanguage::C.game(), SourceGame::AsteroidField);
        for language in [SourceLanguage::Rust, SourceLanguage::Cpp] {
            assert_eq!(language.game(), SourceGame::DiscWars);
        }
        assert_eq!(SourceLanguage::Toml.game(), SourceGame::RiverSurfer);
        assert_eq!(SourceLanguage::Json.game(), SourceGame::SwarmShooter);
        assert_eq!(SourceLanguage::Go.game(), SourceGame::ByteMuncher);
        assert_eq!(SourceLanguage::Ruby.game(), SourceGame::FallingGems);
        assert_eq!(SourceLanguage::Yaml.game(), SourceGame::BlockFall);
        assert_eq!(SourceLanguage::JavaScript.game(), SourceGame::GridHopper);
        assert_eq!(SourceLanguage::Zig.game(), SourceGame::CubeHopper);
        assert_eq!(SourceLanguage::Php.game(), SourceGame::GridBomber);
        assert_eq!(SourceLanguage::R.game(), SourceGame::Plinko);
    }
}
