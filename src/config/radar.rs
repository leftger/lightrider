//! The stealth radar overlay constants.

use bevy::prelude::Color;

pub const RADAR_CELL_SIZE: f32 = 7.0;

pub const RADAR_CELL_GAP: f32 = 1.0;

pub const RADAR_MARGIN: f32 = 16.0;

pub const RADAR_PANEL_COLOR: Color = Color::srgba(0.01, 0.05, 0.03, 0.75);

/// Open floor, which is left to the panel: the map draws what is in the way.
pub const RADAR_FLOOR_COLOR: Color = Color::srgba(0.04, 0.14, 0.09, 0.0);

pub const RADAR_SOLID_COLOR: Color = Color::srgb(0.10, 0.45, 0.24);

pub const RADAR_CONE_COLOR: Color = Color::srgba(0.20, 0.78, 0.95, 0.5);

pub const RADAR_GUARD_COLOR: Color = Color::srgb(0.95, 0.28, 0.34);

pub const RADAR_PLAYER_COLOR: Color = Color::srgb(0.80, 1.0, 0.88);
