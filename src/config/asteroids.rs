//! Asteroid-field constants: rocks, beams, lives and scoring.

use bevy::prelude::Color;

/// Ring radius, in cells, for an asteroid field. Big enough that rocks have to
/// cross some ground before they reach the parked cycle, and still small enough
/// that the camera can frame the whole playfield from above.
pub const ASTEROIDS_RADIUS_CELLS: i32 = 9;

/// Pivot speed of the parked cycle, in radians per second.
pub const ASTEROIDS_TURN_RATE: f32 = 2.6;

/// Seconds between shots.
pub const ASTEROIDS_FIRE_COOLDOWN: f32 = 0.22;

/// Beam speed, in world units per second.
pub const ASTEROIDS_BEAM_SPEED: f32 = 26.0;

/// How long a beam lives before it fizzles.
pub const ASTEROIDS_BEAM_LIFE: f32 = 1.3;

/// Rendered beam length and collision thickness.
pub const ASTEROIDS_BEAM_LENGTH: f32 = 1.1;

pub const ASTEROIDS_BEAM_RADIUS: f32 = 0.18;

/// Collision radius of the parked cycle.
pub const ASTEROIDS_BIKE_RADIUS: f32 = 1.0;

/// Lives before the field is lost.
pub const ASTEROIDS_LIVES: u8 = 3;

/// Mercy window after losing a life.
pub const ASTEROIDS_INVULN: f32 = 1.8;

/// Rocks within this radius are derezzed by the respawn shockwave.
pub const ASTEROIDS_SHOCKWAVE: f32 = 7.5;

/// Rock radius per size tier, in world units. Kept well under the cycle's own
/// radius: a rock you can see around is a rock you can shoot.
pub const ASTEROIDS_ROCK_LARGE: f32 = 1.5;

pub const ASTEROIDS_ROCK_MEDIUM: f32 = 0.95;

pub const ASTEROIDS_ROCK_SMALL: f32 = 0.5;

/// Drift speed range for a fresh rock. Slow enough to line up a shot across the
/// field before it arrives.
pub const ASTEROIDS_ROCK_SPEED_MIN: f32 = 1.3;

pub const ASTEROIDS_ROCK_SPEED_MAX: f32 = 2.8;

/// Half-angle a split sends its two children away from the parent's path.
pub const ASTEROIDS_SPLIT_SPREAD: f32 = 0.7;

/// Large rocks in the opening wave.
pub const ASTEROIDS_WAVE_SIZE: usize = 4;

/// Entity pool caps; the sim never grows past these by design.
pub const ASTEROIDS_MAX_ROCKS: usize = 32;

pub const ASTEROIDS_MAX_BEAMS: usize = 16;

/// Top-down camera height as a multiple of the ring radius.
pub const ASTEROIDS_CAMERA_FIT: f32 = 2.4;

/// How far back from straight-down the field camera leans, as a fraction of its
/// height. A little lean reads as 3D without distorting the aim.
pub const ASTEROIDS_CAMERA_LEAN: f32 = 0.5;

pub const ASTEROIDS_ROCK_COLOR: Color = Color::srgb(0.44, 0.5, 0.6);

pub const ASTEROIDS_ROCK_CORE_COLOR: Color = Color::srgb(1.0, 0.66, 0.26);

pub const ASTEROIDS_BEAM_COLOR: Color = Color::srgb(0.6, 1.0, 1.0);
