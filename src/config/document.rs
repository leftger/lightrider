//! Markdown page-arena constants: layout, glyph budgets and colours.

use bevy::prelude::Color;

pub const DOCUMENT_MAX_BYTES: usize = 256 * 1024;

pub const DOCUMENT_MAX_BLOCKS: usize = 256;

pub const DOCUMENT_MAX_TEXT_CHARS: usize = 2_000;

pub const DOCUMENT_HEADING_GLYPHS: usize = 24;

pub const DOCUMENT_PARAGRAPH_GLYPHS: usize = 12;

pub const DOCUMENT_MAX_GLYPHS: usize = 512;

pub const DOCUMENT_MIN_ARENA_SPAN: i32 = 13;

pub const DOCUMENT_ROW_WIDTH_MIN: i32 = 7;

pub const DOCUMENT_ROW_WIDTH_MAX: i32 = 11;

pub const DOCUMENT_FLOOR_COLOR: Color = Color::srgb(0.86, 0.80, 0.68);

pub const DOCUMENT_RULE_COLOR: Color = Color::srgb(0.62, 0.48, 0.32);

pub const DOCUMENT_INK_COLOR: Color = Color::srgb(0.12, 0.09, 0.07);

pub const DOCUMENT_INK_EMISSIVE: Color = Color::srgb(0.35, 0.18, 0.05);

pub const DOCUMENT_MARGIN_COLOR: Color = Color::srgb(0.72, 0.22, 0.18);

pub const DOCUMENT_HEADING_COLOR: Color = Color::srgb(0.18, 0.12, 0.08);

pub const DOCUMENT_FOLIO_COLOR: Color = Color::srgb(0.42, 0.22, 0.12);

pub const DOCUMENT_FOLIO_DIM_COLOR: Color = Color::srgb(0.18, 0.10, 0.06);

pub const DOCUMENT_FOCUS_COLOR: Color = Color::srgb(0.85, 0.45, 0.12);

pub const MARKDOWN_TOWER_COLOR: Color = Color::srgb(0.92, 0.82, 0.58);
