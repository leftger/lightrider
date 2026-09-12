//! Brick-breaker constants: the court, ball and wall.

use bevy::prelude::Color;

pub const BREAKER_COLS: i32 = 9;

pub const BREAKER_ROWS: i32 = 6;

pub const BREAKER_WIDTH: f32 = 22.0;

pub const BREAKER_HEIGHT: f32 = 26.0;

/// Gap between the ceiling and the first row of bricks.
pub const BREAKER_WALL_TOP: f32 = 3.5;

pub const BREAKER_BRICK_WIDTH: f32 = 1.9;

pub const BREAKER_BRICK_HEIGHT: f32 = 0.9;

pub const BREAKER_BRICK_GAP: f32 = 0.35;

/// Chance a brick above the bottom row is missing, so the wall is not a slab.
pub const BREAKER_HOLE_CHANCE: f32 = 0.12;

pub const BREAKER_PADDLE_Y: f32 = 1.4;

pub const BREAKER_PADDLE_WIDTH: f32 = 4.2;

pub const BREAKER_PADDLE_HEIGHT: f32 = 0.6;

pub const BREAKER_PADDLE_SPEED: f32 = 15.0;

/// How much the bike is scaled up to read as a paddle.
pub const BREAKER_PADDLE_SCALE: f32 = 1.8;

pub const BREAKER_LAUNCH_ANGLE: f32 = 0.35;

pub const BREAKER_MAX_DEFLECT: f32 = 0.85;

/// Largest distance the ball may travel in one collision slice.
pub const BREAKER_MAX_STEP: f32 = 0.25;

pub const BREAKER_BALL_RADIUS: f32 = 0.34;

pub const BREAKER_BALL_SPEED: f32 = 13.0;

pub const BREAKER_CAMERA_BACK: f32 = 44.0;

pub const BREAKER_BRICK_COLOR: Color = Color::srgb(0.35, 0.55, 1.0);

pub const BREAKER_BALL_COLOR: Color = Color::srgb(1.0, 0.96, 0.72);

pub const BREAKER_WALL_COLOR: Color = Color::srgb(0.17, 0.22, 0.32);
