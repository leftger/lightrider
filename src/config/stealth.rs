//! Stealth constants: the room, patrols and detection.

use super::GRID_SPACING;
use super::character::TRON_MODEL_HEIGHT;
use bevy::prelude::Color;

pub const STEALTH_WIDTH: i32 = 21;

pub const STEALTH_HEIGHT: i32 = 15;

pub const STEALTH_COVER: usize = 16;

/// Seconds per cell of creep, for the character and the guards alike.
pub const STEALTH_STEP_SECONDS: f32 = 0.26;

/// The pace a step covers ground, in world units per second. The figure is moved
/// between cells at exactly this speed, and the walk clip is played to match it:
/// easing to each cell instead makes the travel pulse once per step, which the
/// camera inherits as a bob.
pub const STEALTH_WALK_SPEED: f32 = GRID_SPACING / STEALTH_STEP_SECONDS;

pub const STEALTH_VISION_RANGE: f32 = 7.0;

pub const STEALTH_VISION_HALF_ANGLE: f32 = 0.5;

pub const STEALTH_SCAN_SWEEP: f32 = 0.45;

pub const STEALTH_SCAN_RATE: f32 = 1.1;

pub const STEALTH_DETECT_RATE: f32 = 0.7;

pub const STEALTH_DECAY: f32 = 0.45;

/// Cell spacing of the line-of-sight samples.
pub const STEALTH_SIGHT_SAMPLE: f32 = 0.4;

pub const STEALTH_CONE_SEGMENTS: usize = 18;

/// Third-person stealth camera for walking: high enough to read the room and its
/// patrols, which is the view the player spends almost all of their time in and
/// the one the room was tuned around.
pub const STEALTH_CAMERA_HEIGHT: f32 = 11.0;

pub const STEALTH_CAMERA_DISTANCE: f32 = 13.0;

/// Height above the character's feet that the camera aims at.
pub const STEALTH_CAMERA_LOOK: f32 = 1.2;

/// The view while backed against a wall. The camera acts like an imaginary
/// second figure standing off the wall and looking back at the real one, so the
/// figure, the wall he is hugging, the corner and the corridor round the corner
/// all share the frame instead of competing for it.
///
/// It is low on purpose. The tension in this shot is being down at the figure's
/// level rather than above the room, and at this height the wall reads as an edge
/// across the corner of the frame instead of a wall through the middle of it.
pub const STEALTH_HUG_CAMERA_HEIGHT: f32 = 2.8;

/// Out from the hugged face of the wall, in world units, when the figure is right
/// at the corner. A wall's face is just over a unit from the centre of its cell,
/// so this clears it without standing so far off that the corner leaves the frame.
pub const STEALTH_HUG_CAMERA_OUT: f32 = 2.6;

/// Each extra cell of wall between the figure and the corner pushes the camera
/// this much farther out, so the corner and the corridor behind it stay inside
/// the frame instead of slipping past its edge.
pub const STEALTH_HUG_CAMERA_OUT_STEP: f32 = 2.4;

/// How far past the end of the wall the camera sits, looking back across the
/// corner at the figure.
pub const STEALTH_HUG_CAMERA_PAST: f32 = 1.5;

/// A corner farther than this many cells away is not worth standing out for; the
/// camera stays beside the figure and looks down the corridor instead, which is
/// the same shot the wall hug uses until the figure reaches the edge.
pub const STEALTH_HUG_CORNER_STEPS: i32 = 2;

/// For that no-corner shot: how far behind the figure, along the wall, the camera
/// sits. Far enough that the figure and the wall beside him both sit inside the
/// frame while the corridor ahead runs away from them.
pub const STEALTH_HUG_CAMERA_BACK: f32 = 3.5;

/// And how far ahead of the figure the no-corner shot aims, so the corridor he is
/// looking down stays in the middle of the frame rather than at its edge.
pub const STEALTH_HUG_CAMERA_AIM: f32 = 1.5;

/// How many cells of wall to follow looking for its corner.
pub const STEALTH_PEEK_STEPS: i32 = 4;

/// How far the figure is pushed toward a wall it is backed against, in world
/// units, so the pose reads as leaning on the wall rather than standing a cell
/// short of it.
pub const STEALTH_HUG_LEAN: f32 = 0.85;

/// How quickly the camera catches up with the character, per second. Fast
/// enough that its lag stays a steady offset rather than swinging the aim. This
/// is also what eases it down into the wall-hug view and back up out of it.
pub const STEALTH_CAMERA_LERP: f32 = 10.0;

/// How fast the camera turns round the figure, in radians per second. A quarter
/// turn takes about six tenths of a second: slow enough to watch the swing rather
/// than have it read as a cut, which is what made it confusing before.
pub const STEALTH_SWING_RATE: f32 = 2.6;

/// How far along a hugged wall to look when deciding which side has more floor
/// to show, in cells. A bound on the search, not on what the player can see.
pub const STEALTH_PEEK_RUN: i32 = 12;

pub const STEALTH_WALL_HEIGHT: f32 = 2.4;

/// Guards as a fraction of the character's height.
pub const STEALTH_GUARD_SCALE: f32 = 0.85;

/// The stealth figure's height in world units, sized against the grid cell so
/// it reads from the overhead camera. Kept just under the cover walls so they
/// still look like cover.
pub const STEALTH_CHARACTER_HEIGHT: f32 = GRID_SPACING * 0.9;

pub const STEALTH_CHARACTER_SCALE: f32 = STEALTH_CHARACTER_HEIGHT / TRON_MODEL_HEIGHT;

/// The vision cone's reach in world units. The sim measures sight in cells, so
/// the drawn cone has to be converted the same way the guards' cells are.
pub const STEALTH_CONE_REACH: f32 = STEALTH_VISION_RANGE * GRID_SPACING;

pub const STEALTH_FLOOR_COLOR: Color = Color::srgb(0.1, 0.13, 0.19);

pub const STEALTH_WALL_COLOR: Color = Color::srgb(0.28, 0.38, 0.52);

pub const STEALTH_CONE_COLOR: Color = Color::srgb(1.0, 0.34, 0.2);

pub const STEALTH_EXIT_COLOR: Color = Color::srgb(0.55, 1.0, 0.7);
