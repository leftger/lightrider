//! Disc-wars constants: ring sizing, combat and the Recognizer.

use bevy::prelude::Color;

/// Smallest and largest ring radius, in cells. A stub file gets a tiny practice
/// ring; a long one gets a full coliseum.
pub const DISC_RADIUS_MIN: i32 = 8;

pub const DISC_RADIUS_MAX: i32 = 18;

/// How far the close gate corridor runs outward from the ring wall.
pub const DISC_GATE_DEPTH: i32 = 3;

pub const DISC_MAX_GALLERIES: usize = 12;

pub const DISC_MAX_HAZARDS: usize = 24;

pub const DISC_MAX_SAFE_PADS: usize = 24;

pub const DISC_MAX_PICKUPS: usize = 6;

/// Rounds won needed to take the match (best of three).
pub const DISC_WIN_SCORE: u8 = 2;

/// Pause between rounds before both fighters respawn.
pub const DISC_ROUND_DELAY: f32 = 1.6;

/// Cells a disc flies before it turns back.
pub const DISC_RANGE: i32 = 8;

/// Extra range granted by Split / Widen.
pub const DISC_RANGE_BONUS: i32 = 2;

/// Disc flight speed, in cells per second.
pub const DISC_SPEED: f32 = 9.0;

/// Opponent ground speed, in cells per second. Slower than the bike so the
/// Recognizer can be lined up and hit, but still a moving target.
pub const DISC_OPPONENT_SPEED: f32 = 1.7;

/// The opponent only bothers dodging a disc this close (Chebyshev cells). A
/// distant disc leaves it advancing instead of sliding off its own firing line.
pub const DISC_OPPONENT_DODGE_RANGE: i32 = 4;

/// Cells of slack around the opponent's body that still count as a hit for the
/// player's disc. The Recognizer moves in grid steps, so an exact-cell rule
/// makes a moving target nearly impossible to land on. Its own disc keeps an
/// exact rule, so dodging its shots still matters.
pub const DISC_PLAYER_HIT_SLACK: i32 = 1;

/// Seconds between opponent throws while it has line of sight.
pub const DISC_OPPONENT_THROW_COOLDOWN: f32 = 1.05;

/// Seconds the Recognizer winds up after acquiring line of sight, before it
/// fires. Its dais swells while charging, so the shot is telegraphed.
pub const DISC_OPPONENT_WINDUP: f32 = 0.45;

/// Quiet time at the start of a round before the opponent may line up a shot,
/// so a fresh spawn is not immediately punished.
pub const DISC_SPAWN_GRACE: f32 = 1.2;

/// Bullet time: while Shift is held in a ring, everything steps at this
/// fraction of real time, giving the rider room to line up a turn or a throw.
pub const DISC_BULLET_TIME_SCALE: f32 = 0.4;

/// Hazard cells a rider may cross in a row before the fuse burns out. Stepping
/// off the hazard resets it, so one or two are survivable.
pub const DISC_HAZARD_FUSE: i32 = 3;

/// How long a Phase pickup leaves the rider untouchable.
pub const DISC_PHASE_SECONDS: f32 = 1.2;

pub const DISC_SHIELD_MAX: u8 = 2;

/// Heavy disc speed multiplier.
pub const DISC_HEAVY_SPEED_SCALE: f32 = 0.6;

pub const DISC_FLOOR_COLOR: Color = Color::srgb(0.02, 0.03, 0.05);

pub const DISC_RING_COLOR: Color = Color::srgb(0.05, 0.09, 0.14);

pub const DISC_HAZARD_COLOR: Color = Color::srgb(1.0, 0.18, 0.12);

pub const DISC_PICKUP_COLOR: Color = Color::srgb(0.95, 0.9, 0.3);

pub const DISC_OPPONENT_COLOR: Color = Color::srgb(1.0, 0.55, 0.08);

pub const DISC_PLAYER_DISC_COLOR: Color = Color::srgb(0.6, 0.98, 1.0);

pub const DISC_SAFE_PAD_COLOR: Color = Color::srgb(0.1, 0.5, 0.3);

/// Flat cylinder used for a thrown disc.
pub const DISC_MESH_RADIUS: f32 = 0.55;

pub const DISC_MESH_THICKNESS: f32 = 0.14;

/// Stubby cylinder body for the Recognizer opponent.
pub const RECOGNIZER_RADIUS: f32 = 0.6;

pub const RECOGNIZER_HEIGHT: f32 = 1.5;

/// One ring identity per source language, mirroring the plan's table.
pub const DISC_RUST_ACCENT: Color = Color::srgb(0.0, 0.9, 1.0);

pub const DISC_C_ACCENT: Color = Color::srgb(1.0, 0.56, 0.07);

pub const DISC_CPP_ACCENT: Color = Color::srgb(1.0, 0.03, 0.72);

pub const DISC_PYTHON_ACCENT: Color = Color::srgb(0.2, 0.85, 0.25);

pub const DISC_SLINT_ACCENT: Color = Color::srgb(0.55, 0.45, 1.0);

pub const DISC_LUA_ACCENT: Color = Color::srgb(0.15, 0.35, 0.95);

pub const DISC_SHELL_ACCENT: Color = Color::srgb(0.65, 0.75, 0.2);

pub const DISC_TOML_ACCENT: Color = Color::srgb(0.95, 0.72, 0.28);

pub const DISC_JSON_ACCENT: Color = Color::srgb(0.95, 0.25, 0.2);

pub const DISC_GO_ACCENT: Color = Color::srgb(0.0, 0.65, 0.75);

pub const DISC_RUBY_ACCENT: Color = Color::srgb(0.75, 0.05, 0.35);

pub const DISC_YAML_ACCENT: Color = Color::srgb(0.6, 0.6, 0.65);

pub const DISC_JS_ACCENT: Color = Color::srgb(0.95, 0.85, 0.05);

pub const DISC_ZIG_ACCENT: Color = Color::srgb(0.9, 0.45, 0.02);

pub const DISC_PHP_ACCENT: Color = Color::srgb(0.35, 0.25, 0.75);

pub const DISC_R_ACCENT: Color = Color::srgb(0.55, 0.85, 0.9);
