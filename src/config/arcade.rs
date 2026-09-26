//! The arcade block: Byte Muncher, Falling Gems, Block Fall, Grid Hopper, Cube Hopper, Grid Bomber and Plinko.

use bevy::prelude::Color;

pub const ARCADE_HALF_EXTENT: i32 = 16;

pub const MUNCHER_COLS: i32 = 9;

pub const MUNCHER_ROWS: i32 = 7;

pub const MUNCHER_PLAYER_SPEED: f32 = 9.0;

pub const MUNCHER_GHOST_SPEED: f32 = 6.5;

pub const MUNCHER_LIVES: u8 = 3;

pub const MUNCHER_INVULN: f32 = 1.4;

pub const MUNCHER_CAMERA_HEIGHT: f32 = 26.0;

pub const MUNCHER_CAMERA_LEAN: f32 = 0.15;

pub const FALLING_GEMS_COLS: usize = 6;

pub const FALLING_GEMS_ROWS: usize = 12;

pub const FALLING_GEMS_COLORS: usize = 4;

pub const FALLING_GEMS_FALL_SECONDS: f32 = 0.55;

pub const FALLING_GEMS_SEED_ROWS: usize = 4;

pub const FALLING_GEMS_CAMERA_BACK: f32 = 28.0;

pub const FALLING_GEMS_COLORS_LIST: [Color; 4] = [
    Color::srgb(0.95, 0.2, 0.4),
    Color::srgb(0.2, 0.85, 0.9),
    Color::srgb(0.9, 0.85, 0.25),
    Color::srgb(0.7, 0.3, 1.0),
];

pub const BLOCK_FALL_COLS: usize = 10;

pub const BLOCK_FALL_ROWS: usize = 20;

pub const BLOCK_FALL_TARGET_LINES: u32 = 10;

pub const BLOCK_FALL_FALL_SECONDS: f32 = 0.5;

pub const BLOCK_FALL_CAMERA_BACK: f32 = 36.0;

pub const BLOCK_FALL_COLORS: [Color; 7] = [
    Color::srgb(0.9, 0.2, 0.4),
    Color::srgb(0.2, 0.8, 0.9),
    Color::srgb(0.9, 0.8, 0.2),
    Color::srgb(0.7, 0.3, 1.0),
    Color::srgb(0.3, 0.9, 0.4),
    Color::srgb(1.0, 0.55, 0.1),
    Color::srgb(0.4, 0.5, 1.0),
];

pub const GRID_HOPPER_COLS: i32 = 9;

pub const GRID_HOPPER_ROWS: i32 = 7;

pub const GRID_HOPPER_LANES: i32 = 5;

pub const GRID_HOPPER_LIVES: u8 = 3;

pub const GRID_HOPPER_INVULN: f32 = 0.9;

pub const GRID_HOPPER_CAMERA_HEIGHT: f32 = 24.0;

pub const GRID_HOPPER_CAMERA_LEAN: f32 = 0.2;

pub const CUBE_HOPPER_ROWS: usize = 4;

pub const CUBE_HOPPER_CUBE_SPACING: f32 = 2.2;

pub const CUBE_HOPPER_CUBE_HEIGHT: f32 = 1.1;

pub const CUBE_HOPPER_ENEMY_STEP: f32 = 0.55;

pub const CUBE_HOPPER_LIVES: u8 = 3;

pub const CUBE_HOPPER_INVULN: f32 = 1.2;

pub const CUBE_HOPPER_CAMERA_HEIGHT: f32 = 20.0;

pub const CUBE_HOPPER_CAMERA_LEAN: f32 = 0.55;

pub const CUBE_HOPPER_CUBE_DIM_COLOR: Color = Color::srgb(0.25, 0.22, 0.45);

pub const CUBE_HOPPER_CUBE_LIT_COLOR: Color = Color::srgb(0.2, 0.95, 0.85);

pub const CUBE_HOPPER_ENEMY_COLOR: Color = Color::srgb(1.0, 0.3, 0.25);

pub const GRID_BOMBER_COLS: i32 = 11;

pub const GRID_BOMBER_ROWS: i32 = 9;

pub const GRID_BOMBER_CRATES: usize = 22;

pub const GRID_BOMBER_FUSE: f32 = 1.8;

pub const GRID_BOMBER_BLAST: i32 = 2;

pub const GRID_BOMBER_MAX_BOMBS: usize = 2;

pub const GRID_BOMBER_LIVES: u8 = 3;

pub const GRID_BOMBER_INVULN: f32 = 1.2;

pub const GRID_BOMBER_CAMERA_HEIGHT: f32 = 26.0;

pub const GRID_BOMBER_CAMERA_LEAN: f32 = 0.2;

pub const GRID_BOMBER_CRATE_COLOR: Color = Color::srgb(0.55, 0.35, 0.2);

pub const GRID_BOMBER_BOMB_COLOR: Color = Color::srgb(0.15, 0.15, 0.2);

pub const PLINKO_WIDTH: f32 = 16.0;

pub const PLINKO_HEIGHT: f32 = 24.0;

pub const PLINKO_BALLS: usize = 10;

pub const PLINKO_TARGET: u32 = 500;

pub const PLINKO_PIN_ROWS: usize = 8;

pub const PLINKO_CAMERA_BACK: f32 = 34.0;

pub const PLINKO_PIN_COLOR: Color = Color::srgb(0.85, 0.9, 1.0);

pub const PLINKO_BALL_COLOR: Color = Color::srgb(0.95, 0.75, 0.3);
