//! Platformer constants: level layout, physics and the exit door.

use bevy::prelude::Color;

pub const PLATFORMER_RUNNER_WIDTH: f32 = 0.8;

pub const PLATFORMER_RUNNER_HEIGHT: f32 = 1.7;

pub const PLATFORMER_WALK_SPEED: f32 = 7.0;

pub const PLATFORMER_WALK_ACCEL: f32 = 44.0;

pub const PLATFORMER_FRICTION: f32 = 40.0;

pub const PLATFORMER_JUMP_SPEED: f32 = 9.5;

pub const PLATFORMER_GRAVITY: f32 = 22.0;

/// Jump envelope: ~2.0 m of rise and ~6.0 m of reach at full speed, so these
/// stay comfortably clearable.
pub const PLATFORMER_MIN_GAP: f32 = 1.5;

pub const PLATFORMER_MAX_GAP: f32 = 4.0;

pub const PLATFORMER_MAX_STEP: f32 = 1.2;

pub const PLATFORMER_HEIGHT_MAX: f32 = 6.0;

pub const PLATFORMER_MIN_WIDTH: f32 = 2.5;

pub const PLATFORMER_MAX_WIDTH: f32 = 6.0;

pub const PLATFORMER_START_PAD: f32 = 7.0;

pub const PLATFORMER_GOAL_WIDTH: f32 = 8.0;

pub const PLATFORMER_PLATFORM_THICKNESS: f32 = 0.8;

/// A vertical overlap shallower than this is a landing, not a wall.
pub const PLATFORMER_STEP_TOLERANCE: f32 = 0.45;

pub const PLATFORMER_KILL_DEPTH: f32 = 7.0;

pub const PLATFORMER_LENGTH_MIN: f32 = 60.0;

pub const PLATFORMER_LENGTH_MAX: f32 = 240.0;

pub const PLATFORMER_EXIT_WIDTH: f32 = 2.0;

pub const PLATFORMER_EXIT_HEIGHT: f32 = 3.0;

/// Metres of level per line of source, so a longer file is a longer level.
pub const PLATFORMER_METRES_PER_LINE: f32 = 1.6;

pub const PLATFORMER_CAMERA_BACK: f32 = 30.0;

pub const PLATFORMER_CAMERA_HEIGHT: f32 = 4.0;

pub const PLATFORMER_CAMERA_AHEAD: f32 = 8.0;

pub const PLATFORMER_CAMERA_LERP: f32 = 5.0;

/// Half-extent of the metadata-only arena the off-grid games carry, in cells.
pub const PLATFORMER_HALF_EXTENT: i32 = 16;

/// How thick the platforms and backdrop are in Z, so the side view has body.
pub const PLATFORMER_DEPTH: f32 = 9.0;

pub const PLATFORMER_PLATFORM_COLOR: Color = Color::srgb(0.15, 0.2, 0.32);

pub const PLATFORMER_EXIT_COLOR: Color = Color::srgb(0.7, 0.95, 1.0);
