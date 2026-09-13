//! The arcade block: Pac-Man, Columns, Tetris, Frogger, Q*bert, Bomberman and Plinko.

use bevy::prelude::Color;

pub const ARCADE_HALF_EXTENT: i32 = 16;

pub const PAC_COLS: i32 = 9;

pub const PAC_ROWS: i32 = 7;

pub const PAC_PLAYER_SPEED: f32 = 9.0;

pub const PAC_GHOST_SPEED: f32 = 6.5;

pub const PAC_LIVES: u8 = 3;

pub const PAC_INVULN: f32 = 1.4;

pub const PAC_CAMERA_HEIGHT: f32 = 26.0;

pub const PAC_CAMERA_LEAN: f32 = 0.15;

pub const COLUMNS_COLS: usize = 6;

pub const COLUMNS_ROWS: usize = 12;

pub const COLUMNS_GEM_COLORS: usize = 4;

pub const COLUMNS_FALL_SECONDS: f32 = 0.55;

pub const COLUMNS_SEED_ROWS: usize = 4;

pub const COLUMNS_CAMERA_BACK: f32 = 28.0;

pub const COLUMNS_GEM_COLORS_LIST: [Color; 4] = [
    Color::srgb(0.95, 0.2, 0.4),
    Color::srgb(0.2, 0.85, 0.9),
    Color::srgb(0.9, 0.85, 0.25),
    Color::srgb(0.7, 0.3, 1.0),
];

pub const TETRIS_COLS: usize = 10;

pub const TETRIS_ROWS: usize = 20;

pub const TETRIS_TARGET_LINES: u32 = 10;

pub const TETRIS_FALL_SECONDS: f32 = 0.5;

pub const TETRIS_CAMERA_BACK: f32 = 36.0;

pub const TETRIS_COLORS: [Color; 7] = [
    Color::srgb(0.9, 0.2, 0.4),
    Color::srgb(0.2, 0.8, 0.9),
    Color::srgb(0.9, 0.8, 0.2),
    Color::srgb(0.7, 0.3, 1.0),
    Color::srgb(0.3, 0.9, 0.4),
    Color::srgb(1.0, 0.55, 0.1),
    Color::srgb(0.4, 0.5, 1.0),
];

pub const FROGGER_COLS: i32 = 9;

pub const FROGGER_ROWS: i32 = 7;

pub const FROGGER_LANES: i32 = 5;

pub const FROGGER_LIVES: u8 = 3;

pub const FROGGER_INVULN: f32 = 0.9;

pub const FROGGER_CAMERA_HEIGHT: f32 = 24.0;

pub const FROGGER_CAMERA_LEAN: f32 = 0.2;

pub const QBERT_ROWS: usize = 4;

pub const QBERT_CUBE_SPACING: f32 = 2.2;

pub const QBERT_CUBE_HEIGHT: f32 = 1.1;

pub const QBERT_ENEMY_STEP: f32 = 0.55;

pub const QBERT_LIVES: u8 = 3;

pub const QBERT_INVULN: f32 = 1.2;

pub const QBERT_CAMERA_HEIGHT: f32 = 20.0;

pub const QBERT_CAMERA_LEAN: f32 = 0.55;

pub const QBERT_CUBE_DIM_COLOR: Color = Color::srgb(0.25, 0.22, 0.45);

pub const QBERT_CUBE_LIT_COLOR: Color = Color::srgb(0.2, 0.95, 0.85);

pub const QBERT_ENEMY_COLOR: Color = Color::srgb(1.0, 0.3, 0.25);

pub const BOMBER_COLS: i32 = 11;

pub const BOMBER_ROWS: i32 = 9;

pub const BOMBER_CRATES: usize = 22;

pub const BOMBER_FUSE: f32 = 1.8;

pub const BOMBER_BLAST: i32 = 2;

pub const BOMBER_MAX_BOMBS: usize = 2;

pub const BOMBER_LIVES: u8 = 3;

pub const BOMBER_INVULN: f32 = 1.2;

pub const BOMBER_CAMERA_HEIGHT: f32 = 26.0;

pub const BOMBER_CAMERA_LEAN: f32 = 0.2;

pub const BOMBER_CRATE_COLOR: Color = Color::srgb(0.55, 0.35, 0.2);

pub const BOMBER_BOMB_COLOR: Color = Color::srgb(0.15, 0.15, 0.2);

pub const PLINKO_WIDTH: f32 = 16.0;

pub const PLINKO_HEIGHT: f32 = 24.0;

pub const PLINKO_BALLS: usize = 10;

pub const PLINKO_TARGET: u32 = 500;

pub const PLINKO_PIN_ROWS: usize = 8;

pub const PLINKO_CAMERA_BACK: f32 = 34.0;

pub const PLINKO_PIN_COLOR: Color = Color::srgb(0.85, 0.9, 1.0);

pub const PLINKO_BALL_COLOR: Color = Color::srgb(0.95, 0.75, 0.3);
