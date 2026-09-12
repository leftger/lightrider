//! The vocabulary every lightcycle arena shares.
//!
//! Mini-games spawn their own entity pools, but they move them through the
//! same assets, markers and queries the lightcycle arena uses. Those live
//! here, below both the games and the plugin, so a game never has to reach
//! back up into the plugin to pool its entities.

pub(crate) mod pose;

use self::pose::{chase_base_pitch, wrap_angle};
use crate::config;
use crate::disc::language::SourceLanguage;
use crate::state::{LightcycleSceneRoot, TrailSceneRoot};
use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct LightcycleAssets {
    pub(crate) unit_cube: Handle<Mesh>,
    pub(crate) entry_beam_mesh: Handle<Mesh>,
    pub(crate) entry_halo_mesh: Handle<Mesh>,
    pub(crate) cycle_scene: Handle<WorldAsset>,
    pub(crate) trail_material: Handle<StandardMaterial>,
    /// The translucent wall of the memory flood, and its lit crest.
    pub(crate) flood_material: Handle<StandardMaterial>,
    pub(crate) flood_crest_material: Handle<StandardMaterial>,
    /// The collector's sweep: the visible cause of the GC stall.
    pub(crate) gc_sweep_material: Handle<StandardMaterial>,
    pub(crate) wall_material: Handle<StandardMaterial>,
    pub(crate) city_floor_material: Handle<StandardMaterial>,
    pub(crate) city_foundation_material: Handle<StandardMaterial>,
    pub(crate) city_glass_material: Handle<StandardMaterial>,
    pub(crate) city_accent_materials: [[Handle<StandardMaterial>; 2]; 4],
    pub(crate) portal_material: Handle<StandardMaterial>,
    pub(crate) portal_bar_material: Handle<StandardMaterial>,
    pub(crate) dir_tower_material: Handle<StandardMaterial>,
    pub(crate) file_tower_material: Handle<StandardMaterial>,
    pub(crate) markdown_tower_material: Handle<StandardMaterial>,
    pub(crate) source_tower_material: Handle<StandardMaterial>,
    pub(crate) disc_floor_material: Handle<StandardMaterial>,
    pub(crate) disc_ring_material: Handle<StandardMaterial>,
    pub(crate) disc_plinth_material: Handle<StandardMaterial>,
    pub(crate) disc_hazard_material: Handle<StandardMaterial>,
    pub(crate) disc_opponent_material: Handle<StandardMaterial>,
    pub(crate) disc_player_disc_material: Handle<StandardMaterial>,
    pub(crate) disc_pickup_material: Handle<StandardMaterial>,
    pub(crate) disc_safe_pad_material: Handle<StandardMaterial>,
    /// One accent per [`SourceLanguage`], indexed by [`disc_language_index`].
    pub(crate) disc_accent_materials: [Handle<StandardMaterial>; SourceLanguage::COUNT],
    /// Flat cylinder thrown and returned during a fight.
    pub(crate) disc_mesh: Handle<Mesh>,
    /// Taller cylinder body for the Recognizer opponent.
    pub(crate) recognizer_mesh: Handle<Mesh>,
    /// Blocky rock body and beam tracer for the asteroid field.
    pub(crate) rock_material: Handle<StandardMaterial>,
    pub(crate) beam_material: Handle<StandardMaterial>,
    /// Power-up orb and sealed-exit bar for a snake ring.
    pub(crate) snake_food_material: Handle<StandardMaterial>,
    pub(crate) snake_lock_material: Handle<StandardMaterial>,
    /// The Tron runner, and the slabs and door of a platformer level.
    pub(crate) tron_scene: Handle<WorldAsset>,
    /// The same file as a `Gltf`, for the walk clip the scene cannot expose.
    pub(crate) tron_gltf: Handle<Gltf>,
    pub(crate) platform_material: Handle<StandardMaterial>,
    pub(crate) exit_material: Handle<StandardMaterial>,
    /// Bricks, ball and court walls for the breaker.
    pub(crate) brick_material: Handle<StandardMaterial>,
    pub(crate) ball_material: Handle<StandardMaterial>,
    pub(crate) court_material: Handle<StandardMaterial>,
    /// Floor, cover, guards and their vision cones for the stealth run.
    pub(crate) stealth_floor_material: Handle<StandardMaterial>,
    pub(crate) stealth_wall_material: Handle<StandardMaterial>,
    pub(crate) stealth_cone_material: Handle<StandardMaterial>,
    pub(crate) stealth_exit_material: Handle<StandardMaterial>,
    /// Unit-length cone with the stealth half-angle, scaled by its range.
    pub(crate) vision_cone: Handle<Mesh>,
    /// Water ribbon, rocks, boost gates and the finish gate for the surfer.
    pub(crate) surfer_water_material: Handle<StandardMaterial>,
    pub(crate) surfer_rock_material: Handle<StandardMaterial>,
    pub(crate) surfer_gate_material: Handle<StandardMaterial>,
    pub(crate) surfer_finish_material: Handle<StandardMaterial>,
    /// Bug bodies and beam bolts for the Galaga field.
    pub(crate) galaga_bug_material: Handle<StandardMaterial>,
    pub(crate) galaga_beam_material: Handle<StandardMaterial>,
    /// Arcade block: shared materials for the seven fixed-screen games.
    pub(crate) gem_materials: [Handle<StandardMaterial>; 4],
    pub(crate) tetris_materials: [Handle<StandardMaterial>; 7],
    pub(crate) qbert_cube_dim: Handle<StandardMaterial>,
    pub(crate) qbert_cube_lit: Handle<StandardMaterial>,
    pub(crate) qbert_enemy_material: Handle<StandardMaterial>,
    pub(crate) plinko_pin_material: Handle<StandardMaterial>,
    pub(crate) plinko_ball_material: Handle<StandardMaterial>,
    pub(crate) bomber_crate_material: Handle<StandardMaterial>,
    pub(crate) bomber_bomb_material: Handle<StandardMaterial>,
    pub(crate) document_floor_material: Handle<StandardMaterial>,
    pub(crate) document_rule_material: Handle<StandardMaterial>,
    pub(crate) document_margin_material: Handle<StandardMaterial>,
    pub(crate) document_ink_material: Handle<StandardMaterial>,
    pub(crate) document_heading_material: Handle<StandardMaterial>,
    pub(crate) document_folio_material: Handle<StandardMaterial>,
    pub(crate) document_focus_material: Handle<StandardMaterial>,
    pub(crate) crash_material: Handle<StandardMaterial>,
    pub(crate) entry_beam_material: Handle<StandardMaterial>,
    pub(crate) entry_halo_material: Handle<StandardMaterial>,
}

#[derive(Component)]
pub(crate) struct CycleEntity;

/// The rising memory-flood wall of a directory run.
#[derive(Component)]
pub(crate) struct FloodEntity;

/// The collector's sweep, crossing a directory arena while it stalls the world.
#[derive(Component)]
pub(crate) struct GcSweepEntity;

/// One frame of a directory's call stack: its path depth, and the pose the
/// animation oscillates around.
#[derive(Component)]
pub(crate) struct StackFrameEntity {
    pub(crate) level: usize,
    pub(crate) base: Vec3,
}

/// Small mesh burst emitted at the crash point.
#[derive(Component)]
pub(crate) struct CrashDebris {
    pub(crate) velocity: Vec3,
    pub(crate) life: f32,
    pub(crate) max_life: f32,
    pub(crate) initial_scale: f32,
}

/// Entity owned by the directory-tower transport effect.
#[derive(Component)]
pub(crate) struct EntryTransportEntity;

#[derive(Component)]
pub(crate) struct EntryBeam;

#[derive(Component)]
pub(crate) struct EntryHalo {
    pub(crate) phase: f32,
}

/// A post or lintel of the parent gate.
#[derive(Component)]
pub(crate) struct GateFrame;

/// A light bar sweeping up through the parent gate's opening.
#[derive(Component)]
pub(crate) struct GateScanBar {
    /// Position in the sweep at startup, so the bars are evenly spaced.
    pub(crate) offset: f32,
    /// Height of the opening the bar travels up before wrapping.
    pub(crate) travel: f32,
}

#[derive(Component)]
pub(crate) struct CityBeacon {
    pub(crate) base_height: f32,
    pub(crate) phase: f32,
}

#[derive(Component)]
pub(crate) struct DocumentFocusMarker;

/// The Tron runner, on a platformer level or a stealth run.
#[derive(Component)]
pub(crate) struct CharacterEntity;

/// Easing state for the on-foot character.
///
/// The walk itself is the asset's animation clip, so the only thing left to
/// track here is `base`: the ground position the figure is easing toward, which
/// smooths the stealth sim's whole-cell steps into a glide.
#[derive(Component)]
pub(crate) struct CharacterAnim {
    pub(crate) base: Vec3,
}

impl CharacterAnim {
    pub(crate) fn at(base: Vec3) -> Self {
        Self { base }
    }
}

/// The character's walk clip, once its animation graph has been built.
///
/// The glTF loader creates the `AnimationPlayer` but no graph, so one is built
/// from the clip the asset carries.
#[derive(Component)]
pub(crate) struct CharacterWalk(pub(crate) AnimationNodeIndex);

/// Marks everything spawned beneath an on-foot character.
///
/// The loader creates the `AnimationPlayer` deep inside the scene, so there is
/// nothing to tag at spawn time: the character's subtree is walked instead,
/// which also picks up descendants that only appear a frame or two later.
#[derive(Component)]
pub(crate) struct CharacterModel;

/// One flat quad of ground marking: where it sits, and how far it reaches on
/// each ground axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MarkingQuad {
    pub(crate) center: Vec3,
    pub(crate) half_x: f32,
    pub(crate) half_z: f32,
}

/// Direction the chase camera is currently following.
///
/// This trails the cycle's own heading so a corner reads as the cycle swinging
/// across the frame. Locking the camera to the cycle instead makes the world
/// appear to rotate around a stationary bike. It lives on the cycle so each run
/// starts from the spawn heading.
#[derive(Component)]
pub(crate) struct ChaseCamera {
    pub(crate) forward: Vec3,
    /// Right-drag free look, in radians: `x` swings the rig around the cycle and
    /// `y` raises it above the default chase pitch. Held only while dragging;
    /// releasing the button eases it back to zero.
    pub(crate) look: Vec2,
}

impl ChaseCamera {
    /// Folds one frame of right-drag mouse motion into the free-look offset.
    ///
    /// Both axes are negated to match the explorer's orbit camera, which
    /// measures its own yaw and pitch in the opposite sense. Pitch is clamped
    /// here rather than only at render time so holding a drag past the limit
    /// cannot bank up rotation that the next drag has to spend undoing.
    pub(crate) fn apply_look_drag(&mut self, delta: Vec2) {
        let base_pitch = chase_base_pitch();
        self.look.x = wrap_angle(self.look.x - delta.x * config::CAMERA_ROTATION_SPEED);
        self.look.y = (self.look.y - delta.y * config::CAMERA_ROTATION_SPEED).clamp(
            config::lightcycle::LIGHTCYCLE_CAMERA_MIN_PITCH - base_pitch,
            config::lightcycle::LIGHTCYCLE_CAMERA_MAX_PITCH - base_pitch,
        );
    }

    /// Eases free look back behind the cycle after the button is released, with
    /// a frame-rate independent time constant.
    pub(crate) fn recenter_look(&mut self, delta_seconds: f32) {
        if self.look == Vec2::ZERO {
            return;
        }

        let blend =
            1.0 - (-delta_seconds / config::lightcycle::LIGHTCYCLE_CAMERA_LOOK_RECENTER).exp();
        self.look = self.look.lerp(Vec2::ZERO, blend.clamp(0.0, 1.0));

        // An exponential ease never quite arrives, so land it rather than
        // leaving the camera drifting by fractions of a degree forever.
        if self.look.length_squared() < LOOK_RECENTER_SNAP * LOOK_RECENTER_SNAP {
            self.look = Vec2::ZERO;
        }
    }
}

/// Free-look offset below which recentering snaps home, in radians.
const LOOK_RECENTER_SNAP: f32 = 1.0e-3;

/// A pool of sim-driven entities: the marker that keys it, plus the transform and
/// visibility the sim's own copy of the state drives.
pub(crate) type Pooled<'w, 's, M, F = ()> =
    Query<'w, 's, (&'static M, &'static mut Transform, &'static mut Visibility), F>;

/// A pool whose entries also wear a material the sim refreshes.
pub(crate) type PooledTinted<'w, 's, M, F = ()> = Query<
    'w,
    's,
    (
        &'static M,
        &'static mut Transform,
        &'static mut Visibility,
        &'static mut MeshMaterial3d<StandardMaterial>,
    ),
    F,
>;

/// A pool the sim only moves.
pub(crate) type PooledPosed<'w, 's, M, F = ()> =
    Query<'w, 's, (&'static M, &'static mut Transform), F>;

/// A pool the sim moves and re-tints.
pub(crate) type PooledPosedTinted<'w, 's, M, F = ()> = Query<
    'w,
    's,
    (
        &'static M,
        &'static mut Transform,
        &'static mut MeshMaterial3d<StandardMaterial>,
    ),
    F,
>;

/// A pool the sim only shows and hides, because its fate is decided by the sim
/// rather than by where it is.
pub(crate) type PooledShown<'w, 's, M, F = ()> =
    Query<'w, 's, (&'static M, &'static mut Visibility), F>;

/// The marker an entity carries, and the lookalikes it must not be confused with.
/// These exclusions are what let Bevy prove two queries cannot collide.
pub(crate) type Only<A, B, C> = (With<A>, Without<B>, Without<C>);

/// One occupant of an arena it shares with others, told apart by its marker.
pub(crate) type Fighter<'w, 's, M, Other, ItsDisc> =
    Query<'w, 's, (&'static mut Transform, &'static mut Visibility), Only<M, Other, ItsDisc>>;

/// Keeps an entity clear of the others that share its arena.
pub(crate) type FreeOf<A, B, C> = (Without<A>, Without<B>, Without<C>);

/// Keeps two pools that share an arena from matching each other's entities.
pub(crate) type Apart<A, B = CycleEntity> = (Without<A>, Without<B>);

/// The lightcycle's own entities, which no minigame pool may claim.
pub(crate) type OutOfCycle = Without<CycleEntity>;

/// Everything a run spawns, so a room change can clear the lot in one query.
pub(crate) type SceneEntities<'w, 's> =
    Query<'w, 's, Entity, Or<(With<LightcycleSceneRoot>, With<TrailSceneRoot>)>>;
