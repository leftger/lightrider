//! Mode-transition constants: the zoom between explorer and lightcycle.

use bevy::prelude::Color;

/// Length of the flight between the explorer's orbit rig and the lightcycle's
/// chase rig, in seconds. It covers a climb to a satellite view of the city and
/// a zoom back down into one road, so it is longer than a plain cut.
pub const MODE_TRANSITION_DURATION: f32 = 1.6;

/// Point in the flight where the old world is torn down and the new one is
/// built, as a fraction of the duration. This is the top of the climb, and both
/// worlds are flattened into the ground plane by the time it arrives.
pub const MODE_TRANSITION_SWAP_AT: f32 = 0.45;

/// Height of the satellite view above the road it is aimed at, in world units.
pub const MODE_TRANSITION_GODS_EYE_HEIGHT: f32 = 58.0;

/// How far back down the road the satellite view sits, in world units. Tipping
/// the shot well off vertical keeps a skyline in frame, so the city sinking and
/// the next one rising read as the ground remaking itself rather than as a cut
/// between two overhead maps.
pub const MODE_TRANSITION_GODS_EYE_BACKOFF: f32 = 30.0;

/// Fraction of the flight the outgoing world spends sinking into the ground
/// before the swap, and the incoming one spends growing back out of it after.
pub const MODE_TRANSITION_DEREZ_WINDOW: f32 = 0.26;

pub const MODE_TRANSITION_REZ_WINDOW: f32 = 0.32;

/// How much the flight's altitude lags its ground track on the way down, as an
/// exponent. Above 1 the camera runs out over the road before dropping onto it,
/// and the climb applies the reciprocal, so it gains height before it travels.
pub const MODE_TRANSITION_ALTITUDE_BIAS: f32 = 1.8;

/// Fraction the field of view widens by during the dive, which exaggerates the
/// speed of the descent.
pub const MODE_TRANSITION_FOV_KICK: f32 = 0.18;

pub const MODE_TRANSITION_LIGHTCYCLE_TINT: Color = Color::srgb(0.55, 0.95, 1.0);

pub const MODE_TRANSITION_EXPLORER_TINT: Color = Color::srgb(0.25, 1.0, 0.45);

/// Height the rez wave climbs to as the new world grows, in world units. Tall
/// enough to clear the skyline, low enough to stay under the diving camera.
pub const MODE_TRANSITION_REZ_HEIGHT: f32 = 18.0;

/// Side length of the rez wave sheet, in world units. It has to cover the whole
/// frame at the swap, where it sits on the ground and carries the redraw.
pub const MODE_TRANSITION_REZ_SPAN: f32 = 140.0;

pub const MODE_TRANSITION_REZ_ALPHA: f32 = 0.38;
