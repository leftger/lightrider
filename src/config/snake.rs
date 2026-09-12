//! Snake constants: ring size, power-ups and tail growth.

use bevy::prelude::Color;

/// Ring radius, in cells, for a snake run. Tighter than a disc ring so the
/// growing tail stays a threat.
pub const SNAKE_RADIUS_CELLS: i32 = 10;

/// Power-ups to collect before the exit opens.
pub const SNAKE_FOOD_TARGET: usize = 6;

/// Tail length at the start, in cells.
pub const SNAKE_TAIL_START: usize = 8;

/// Cells of tail gained per power-up.
pub const SNAKE_TAIL_GROWTH: usize = 4;

/// Preferred clear space between a power-up and the rider's spawn.
pub const SNAKE_MIN_FOOD_DISTANCE: i32 = 4;

/// Rendered power-up size.
pub const SNAKE_FOOD_SIZE: f32 = 0.55;

pub const SNAKE_FOOD_COLOR: Color = Color::srgb(0.45, 1.0, 0.35);

pub const SNAKE_GATE_COLOR: Color = Color::srgb(1.0, 0.22, 0.25);
