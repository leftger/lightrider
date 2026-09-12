use crate::config;
use crate::disc::language::SourceLanguage;
use crate::disc::load::{SourceLoadFailed, SourceLoaded, SourceRequested, WarpRequested};
use crate::document::load::{DocumentLoadFailed, DocumentLoaded, DocumentRequested};
use crate::lightcycle::LightcycleState;
use crate::state::{
    CacheState, FloodState, HistoryState, InteractionMode, LightcycleSceneRoot, PauseState,
    StackMotion, TrailSceneRoot,
};
use bevy::prelude::*;
pub(crate) mod assets;
pub(crate) mod camera;
pub(crate) mod character;
pub(crate) mod city;
pub(crate) mod decor;
pub(crate) mod entities;
pub(crate) mod entry;
pub(crate) mod input;
pub(crate) mod load;
pub(crate) mod run;
pub(crate) mod space;
pub(crate) mod step;
pub(crate) mod trail;

use self::assets::setup_lightcycle_assets;
use self::camera::{arc_cell_pose, chase_base_pitch, update_chase_camera};
use self::character::{
    drive_character_walk, fit_guard_cones, prepare_character_walk, tag_character_model,
};
use self::decor::{
    animate_city_beacons, animate_parent_gate, animate_stack_frames, update_flood, update_gc_sweep,
    wrap_angle,
};
use self::entities::{
    sync_character_entities, sync_directory_scene_visibility, sync_disc_entities,
};
use self::entry::{
    animate_entry_effect, cleanup_orphaned_entry_effect, spawn_crash_effect, spawn_entry_effect,
    update_crash_effects,
};
use self::input::{read_lightcycle_input, update_cycle_transform};
use self::load::{
    apply_document_load_failure, apply_load_failure, apply_source_load_failure,
    handle_warp_requests, poll_document_loads, poll_source_loads, reset_on_directory_loaded,
    reset_on_document_loaded, reset_on_source_loaded, start_document_loads, start_source_loads,
};
use self::run::{
    apply_district_ambience, apply_mode_swap, in_lightcycle_mode, restore_directory_arena,
    toggle_mode,
};
use self::step::step_lightcycle;
use self::trail::update_trail_mesh;
use crate::asteroids::plugin::sync_asteroid_entities;
use crate::bomberman::plugin::sync_bomberman_entities;
use crate::breaker::plugin::sync_breaker_entities;
use crate::columns::plugin::sync_columns_entities;
use crate::disc::plugin::animate_disc_pickups;
use crate::disc::plugin::update_disc_focus;
use crate::document::plugin::update_document_focus;
use crate::frogger::plugin::sync_frogger_entities;
use crate::galaga::plugin::sync_galaga_entities;
use crate::pacman::plugin::sync_pacman_entities;
use crate::plinko::plugin::sync_plinko_entities;
use crate::qbert::plugin::sync_qbert_entities;
use crate::snake::plugin::sync_snake_entities;
use crate::stealth::plugin::sync_stealth_entities;
use crate::tetris::plugin::sync_tetris_entities;

pub struct LightcyclePlugin;

impl Plugin for LightcyclePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InteractionMode>()
            .init_resource::<LightcycleState>()
            .init_resource::<PauseState>()
            .init_resource::<CacheState>()
            .init_resource::<FloodState>()
            .init_resource::<HistoryState>()
            .init_resource::<StackMotion>()
            .add_message::<DocumentRequested>()
            .add_message::<DocumentLoaded>()
            .add_message::<DocumentLoadFailed>()
            .add_message::<SourceRequested>()
            .add_message::<SourceLoaded>()
            .add_message::<SourceLoadFailed>()
            .add_message::<WarpRequested>()
            .add_systems(Startup, setup_lightcycle_assets)
            .add_systems(
                Update,
                (
                    (
                        toggle_mode,
                        apply_mode_swap,
                        reset_on_directory_loaded,
                        start_document_loads,
                        poll_document_loads,
                        reset_on_document_loaded,
                        start_source_loads,
                        poll_source_loads,
                        reset_on_source_loaded,
                        handle_warp_requests,
                        apply_load_failure,
                        apply_document_load_failure,
                        apply_source_load_failure,
                        sync_directory_scene_visibility,
                    ),
                    (
                        read_lightcycle_input.run_if(in_lightcycle_mode),
                        step_lightcycle.run_if(in_lightcycle_mode),
                        update_flood.run_if(in_lightcycle_mode),
                        update_gc_sweep.run_if(in_lightcycle_mode),
                        restore_directory_arena.run_if(in_lightcycle_mode),
                        spawn_crash_effect.run_if(in_lightcycle_mode),
                        update_crash_effects.run_if(in_lightcycle_mode),
                        update_trail_mesh.run_if(in_lightcycle_mode),
                        animate_parent_gate.run_if(in_lightcycle_mode),
                        animate_city_beacons.run_if(in_lightcycle_mode),
                        animate_stack_frames.run_if(in_lightcycle_mode),
                        update_document_focus.run_if(in_lightcycle_mode),
                        update_disc_focus.run_if(in_lightcycle_mode),
                        sync_disc_entities.run_if(in_lightcycle_mode),
                    ),
                    (
                        sync_asteroid_entities.run_if(in_lightcycle_mode),
                        sync_galaga_entities.run_if(in_lightcycle_mode),
                        sync_pacman_entities.run_if(in_lightcycle_mode),
                        sync_columns_entities.run_if(in_lightcycle_mode),
                        sync_tetris_entities.run_if(in_lightcycle_mode),
                        sync_frogger_entities.run_if(in_lightcycle_mode),
                        sync_qbert_entities.run_if(in_lightcycle_mode),
                        sync_bomberman_entities.run_if(in_lightcycle_mode),
                        sync_plinko_entities.run_if(in_lightcycle_mode),
                        sync_snake_entities.run_if(in_lightcycle_mode),
                        sync_character_entities.run_if(in_lightcycle_mode),
                        tag_character_model.run_if(in_lightcycle_mode),
                        prepare_character_walk.run_if(in_lightcycle_mode),
                        drive_character_walk.run_if(in_lightcycle_mode),
                        sync_breaker_entities.run_if(in_lightcycle_mode),
                        sync_stealth_entities.run_if(in_lightcycle_mode),
                        fit_guard_cones.run_if(in_lightcycle_mode),
                        animate_disc_pickups.run_if(in_lightcycle_mode),
                        update_cycle_transform.run_if(in_lightcycle_mode),
                        update_chase_camera.run_if(in_lightcycle_mode),
                    ),
                )
                    .chain()
                    .after(crate::plugins::filesystem::apply_loaded),
            )
            .add_systems(
                Update,
                (
                    cleanup_orphaned_entry_effect
                        .after(read_lightcycle_input)
                        .before(spawn_entry_effect),
                    spawn_entry_effect.after(step_lightcycle),
                    animate_entry_effect
                        .after(spawn_entry_effect)
                        .after(update_cycle_transform)
                        .before(update_chase_camera),
                )
                    .run_if(in_lightcycle_mode),
            )
            // Outside the chained group: that tuple is already at Bevy's arity
            // limit, and the tint only has to land after the run it describes.
            .add_systems(Update, apply_district_ambience.after(apply_mode_swap));
    }
}

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
    level: usize,
    base: Vec3,
}

/// Small mesh burst emitted at the crash point.
#[derive(Component)]
pub(crate) struct CrashDebris {
    velocity: Vec3,
    life: f32,
    max_life: f32,
    initial_scale: f32,
}

/// Entity owned by the directory-tower transport effect.
#[derive(Component)]
pub(crate) struct EntryTransportEntity;

#[derive(Component)]
pub(crate) struct EntryBeam;

#[derive(Component)]
pub(crate) struct EntryHalo {
    phase: f32,
}

/// A post or lintel of the parent gate.
#[derive(Component)]
struct GateFrame;

/// A light bar sweeping up through the parent gate's opening.
#[derive(Component)]
pub(crate) struct GateScanBar {
    /// Position in the sweep at startup, so the bars are evenly spaced.
    offset: f32,
    /// Height of the opening the bar travels up before wrapping.
    travel: f32,
}

#[derive(Component)]
pub(crate) struct CityBeacon {
    base_height: f32,
    phase: f32,
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
    base: Vec3,
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
pub(crate) struct CharacterWalk(AnimationNodeIndex);

/// Marks everything spawned beneath an on-foot character.
///
/// The loader creates the `AnimationPlayer` deep inside the scene, so there is
/// nothing to tag at spawn time: the character's subtree is walked instead,
/// which also picks up descendants that only appear a frame or two later.
#[derive(Component)]
pub(crate) struct CharacterModel;

// The query types the sync systems are written in. Every minigame keeps a pool of
// entities that are spawned once and then shown, hidden and moved to match its
// sim, and writing that query out per system made the signatures unreadable: the
// alias says which marker keys the pool and what the sim drives on it.
//
// `F` is the filter that keeps sibling pools apart, since two pools of the same
// shape would otherwise match each other's entities.

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

/// Direction the chase camera is currently following.
///
/// This trails the cycle's own heading so a corner reads as the cycle swinging
/// across the frame. Locking the camera to the cycle instead makes the world
/// appear to rotate around a stationary bike. It lives on the cycle so each run
/// starts from the spawn heading.
#[derive(Component)]
pub(crate) struct ChaseCamera {
    forward: Vec3,
    /// Right-drag free look, in radians: `x` swings the rig around the cycle and
    /// `y` raises it above the default chase pitch. Held only while dragging;
    /// releasing the button eases it back to zero.
    look: Vec2,
}

impl ChaseCamera {
    /// Folds one frame of right-drag mouse motion into the free-look offset.
    ///
    /// Both axes are negated to match the explorer's orbit camera, which
    /// measures its own yaw and pitch in the opposite sense. Pitch is clamped
    /// here rather than only at render time so holding a drag past the limit
    /// cannot bank up rotation that the next drag has to spend undoing.
    fn apply_look_drag(&mut self, delta: Vec2) {
        let base_pitch = chase_base_pitch();
        self.look.x = wrap_angle(self.look.x - delta.x * config::CAMERA_ROTATION_SPEED);
        self.look.y = (self.look.y - delta.y * config::CAMERA_ROTATION_SPEED).clamp(
            config::lightcycle::LIGHTCYCLE_CAMERA_MIN_PITCH - base_pitch,
            config::lightcycle::LIGHTCYCLE_CAMERA_MAX_PITCH - base_pitch,
        );
    }

    /// Eases free look back behind the cycle after the button is released, with
    /// a frame-rate independent time constant.
    fn recenter_look(&mut self, delta_seconds: f32) {
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

/// Lane markings use a district's primary accent, so ground seams use the
/// secondary one to stay readable against them.
const CITY_TRIM_ACCENT: usize = 1;

fn despawn_lightcycle_entities(commands: &mut Commands, old_lightcycle_entities: &SceneEntities) {
    for entity in old_lightcycle_entities {
        commands.entity(entity).despawn();
    }
}

/// Where the on-foot character should be and which way it faces.
pub(crate) struct CharacterPose {
    /// Ground position the figure is walking toward.
    target: Vec3,
    yaw: f32,
    /// True when the sim moves in grid steps that need easing out.
    smooth: bool,
}

/// One flat quad of ground marking: where it sits, and how far it reaches on
/// each ground axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MarkingQuad {
    center: Vec3,
    half_x: f32,
    half_z: f32,
}

/// Continuous render pose for the cycle, in cell coordinates.
pub(crate) struct CyclePose {
    position: (f32, f32),
    /// Unit travel direction; the cycle's nose points along it.
    direction: Vec2,
    /// Bank angle about the travel direction, in radians. Zero outside corners.
    lean: f32,
}

/// The live corner the cycle is riding, if any.
#[derive(Clone, Copy)]
pub(crate) struct CornerArc {
    corner: (i32, i32),
    incoming: (i32, i32),
    outgoing: (i32, i32),
    u: f32,
    radius: f32,
}

impl CornerArc {
    fn sample(self, u: f32) -> CyclePose {
        arc_cell_pose(self.corner, self.incoming, self.outgoing, u, self.radius)
    }
}

/// One wall-hug camera pose: where the camera stands and what it looks at, both
/// as offsets from the character's cell centre, plus the camera height.
pub(crate) struct HugShot {
    pub(crate) offset: Vec3,
    pub(crate) look: Vec3,
    pub(crate) height: f32,
}

#[cfg(test)]
mod tests;
