//! The on-foot runner: its model asset, walk clip and scale.

use super::platformer::PLATFORMER_RUNNER_HEIGHT;

/// The Tron runner. CC-BY-4.0: see the credits in the README.
pub const TRON_MODEL_ASSET: &str = "models/tron_character/scene.gltf";

/// Height of the loaded character in world units, feet on the origin.
///
/// The asset is a rigged re-export: 19 joints, one walk clip, and the original
/// inch-scale node matrices baked into the vertex data, so the vertices now
/// span 23.607 units on their own. Measured from the `POSITION` accessor
/// bounds. Dividing by the raw authored mesh height instead is a 39x error,
/// which once left the runner a twentieth of a unit tall: present, glowing, and
/// invisible.
pub const TRON_MODEL_HEIGHT: f32 = 23.607;

/// Scaled so the runner stands exactly as tall as its collision box.
pub const TRON_MODEL_SCALE: f32 = PLATFORMER_RUNNER_HEIGHT / TRON_MODEL_HEIGHT;

/// Extra yaw applied to the character model on top of the entity's facing.
///
/// The asset's own forward is `+Z`, which is what the facing yaws in the plugin
/// already target, so this stays at zero. A half turn here points the runner
/// exactly backwards, which is how it walked until it was noticed.
pub const TRON_MODEL_YAW: f32 = 0.0;

/// Name of the walk clip the character asset carries.
pub const WALK_CLIP: &str = "Walk";

/// Ground the clip covers in one cycle. Measured from the foot's travel in the
/// rigged cycle (24 frames at 24 fps), and used to convert ground speed into
/// clip playback speed.
pub const WALK_CLIP_GROUND: f32 = 7.3;
