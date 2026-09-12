//! River-surfer constants: the river, rocks and gates.

use bevy::prelude::Color;

/// Metadata-only arena half-extent for the flat arena the surfer builds on.
pub const SURFER_HALF_EXTENT: i32 = 16;

/// Course length per source line, in world units, clamped below.
pub const SURFER_METRES_PER_LINE: f32 = 0.5;

pub const SURFER_MIN_LENGTH: f32 = 150.0;

pub const SURFER_MAX_LENGTH: f32 = 380.0;

/// Half width of the playable river ribbon. Wide enough that a rock and a
/// passable gap on one side of it always fit.
pub const SURFER_HALF_WIDTH: f32 = 6.5;

/// The centreline sways with a seeded sine of this amplitude and wavelength.
pub const SURFER_RIVER_AMP: f32 = 8.0;

pub const SURFER_RIVER_WAVELENGTH: f32 = 95.0;

/// The bike always has the throttle open; boost lifts it to this.
pub const SURFER_BASE_SPEED: f32 = 14.0;

pub const SURFER_BOOST_SPEED: f32 = 21.0;

/// How quickly the bike eases between base and boost speed.
pub const SURFER_ACCEL: f32 = 3.0;

/// Steering authority, in radians per second at full stick.
pub const SURFER_TURN_RATE: f32 = 2.3;

/// Bike collision radius against rocks and banks. Kept small, so a rock only
/// bites when the visible bike actually touches it.
pub const SURFER_BOAT_RADIUS: f32 = 0.6;

/// Hover height over the water, plus the wave bob below.
pub const SURFER_HOVER_HEIGHT: f32 = 0.9;

pub const SURFER_WAVE_AMPLITUDE: f32 = 0.22;

pub const SURFER_WAVE_RATE: f32 = 3.2;

pub const SURFER_WAVE_SPACE: f32 = 0.12;

/// Clear water behind the start and beyond the finish line.
pub const SURFER_START_CLEAR: f32 = 18.0;

pub const SURFER_FINISH_MARGIN: f32 = 20.0;

/// Obstacles and boost gates scattered down the course. Rocks are spaced by
/// the distance below, so the field is never denser than one rock per stretch
/// of clear water.
pub const SURFER_ROCK_SPACING: f32 = 17.0;

pub const SURFER_MAX_ROCKS: usize = 12;

pub const SURFER_ROCK_RADIUS: f32 = 1.25;

/// Minimum clear water kept open beside every rock, wider than the bike, so
/// there is always a line through even when the boost is held.
pub const SURFER_ROCK_CLEAR_GAP: f32 = 1.1;

pub const SURFER_ROCK_HEIGHT: f32 = 2.4;

pub const SURFER_GATES: usize = 7;

/// Half width of a gate opening; ride through the middle to claim it.
pub const SURFER_GATE_SPAN: f32 = 4.5;

pub const SURFER_GATE_HEIGHT: f32 = 4.0;

pub const SURFER_FINISH_HEIGHT: f32 = 6.0;

/// River ribbon tessellation and how far it extends past each end.
pub const SURFER_RIVER_SAMPLE: f32 = 2.0;

pub const SURFER_RIVER_MARGIN: f32 = 12.0;

/// Chase camera: low and close to the water, looking down the river.
pub const SURFER_CAMERA_DISTANCE: f32 = 9.0;

pub const SURFER_CAMERA_HEIGHT: f32 = 1.6;

pub const SURFER_CAMERA_LOOKAHEAD: f32 = 5.0;

pub const SURFER_WATER_COLOR: Color = Color::srgba(0.07, 0.42, 0.85, 0.85);

pub const SURFER_ROCK_COLOR: Color = Color::srgb(0.24, 0.28, 0.34);

pub const SURFER_GATE_COLOR: Color = Color::srgb(0.1, 0.9, 1.0);

pub const SURFER_FINISH_COLOR: Color = Color::srgb(0.55, 1.0, 0.7);
