//! Galaga constants: the formation, dives and beams.

use bevy::prelude::Color;

/// Metadata-only arena half-extent for the flat arena the field builds on.
pub const GALAGA_HALF_EXTENT: i32 = 16;

/// Half extents of the playable field, in world units.
pub const GALAGA_HALF_X: f32 = 15.0;

pub const GALAGA_HALF_Z: f32 = 11.0;

/// The cycle's fixed Z on the bottom edge.
pub const GALAGA_PLAYER_Z: f32 = -9.0;

pub const GALAGA_PLAYER_SPEED: f32 = 14.0;

pub const GALAGA_PLAYER_RADIUS: f32 = 0.9;

pub const GALAGA_LIVES: u8 = 3;

/// Mercy window after losing a life, and the starting grace.
pub const GALAGA_INVULN: f32 = 1.2;

pub const GALAGA_FIRE_COOLDOWN: f32 = 0.28;

pub const GALAGA_MAX_BEAMS: usize = 8;

pub const GALAGA_BEAM_SPEED: f32 = 34.0;

pub const GALAGA_BEAM_RADIUS: f32 = 0.45;

/// How far ahead of the cycle a beam spawns.
pub const GALAGA_BEAM_MUZZLE: f32 = 1.1;

/// Visual length of a beam bolt.
pub const GALAGA_BEAM_LENGTH: f32 = 1.4;

/// The formation: a `GALAGA_ROWS` by `GALAGA_COLS` grid.
pub const GALAGA_ROWS: usize = 4;

pub const GALAGA_COLS: usize = 8;

pub const GALAGA_CELL_X: f32 = 3.0;

pub const GALAGA_CELL_Z: f32 = 2.4;

/// Top row's starting Z, and how far the whole grid sways side to side.
pub const GALAGA_FORMATION_TOP: f32 = 8.0;

pub const GALAGA_FORMATION_SWAY: f32 = 4.5;

pub const GALAGA_FORMATION_SPEED: f32 = 2.2;

/// How far the formation steps down, and the base seconds between steps.
pub const GALAGA_FORMATION_STEP: f32 = 0.7;

pub const GALAGA_FORMATION_STEP_SECONDS: f32 = 4.0;

/// Divers: speed, seconds between dives, and how sharply they track the cycle.
pub const GALAGA_DIVE_SPEED: f32 = 12.0;

pub const GALAGA_DIVE_COOLDOWN: f32 = 1.4;

pub const GALAGA_DIVE_STEER: f32 = 14.0;

pub const GALAGA_BUG_RADIUS: f32 = 1.0;

/// Visual height of a bug body, for the pooled cube.
pub const GALAGA_BUG_HEIGHT: f32 = 1.2;

/// Overhead camera, framed so the whole field stays in shot.
pub const GALAGA_CAMERA_HEIGHT: f32 = 30.0;

pub const GALAGA_CAMERA_LEAN: f32 = 0.25;

pub const GALAGA_BUG_COLOR: Color = Color::srgb(0.95, 0.2, 0.85);

pub const GALAGA_BEAM_COLOR: Color = Color::srgb(0.3, 0.95, 1.0);
