use crate::asteroids::sim::{AsteroidsPhase, AsteroidsSim};
use crate::bomberman::sim::{BomberPhase, BomberSim};
use crate::breaker::sim::{BreakerPhase, BreakerSim};
use crate::columns::sim::{ColumnsPhase, ColumnsSim};
use crate::config;
use crate::disc::combat::{DiscEvents, DiscPhase, DiscSim, PlayerSnapshot};
use crate::disc::language::{SourceGame, SourceLanguage};
use crate::disc::layout::{
    DiscLayout, build_capped_disc_arena, build_disc_arena, build_flat_arena,
};
use crate::disc::load::{
    SourceLoadFailed, SourceLoadState, SourceLoaded, SourceRequested, WarpRequested,
};
use crate::document::layout::{DocumentLayout, build_document_arena_from_parse};
use crate::document::load::{
    DocumentLoadFailed, DocumentLoadState, DocumentLoaded, DocumentRequested,
};
use crate::document::parse::{ParseLimits, parse_markdown_bytes};
use crate::filesystem::node::FileNode;
use crate::frogger::sim::{FroggerPhase, FroggerSim};
use crate::galaga::sim::{GalagaPhase, GalagaSim};
use crate::lightcycle::logic::{
    Arena, ArenaKind, CellContent, CityStructure, CityStructureKind, CityTheme, CrashReason,
    GatePlacement, Heading, LightcycleSim, ParentPortal, RunPhase, StepOutcome, Wall,
    classify_next_content, road_plates, stable_path_seed,
};
use crate::lightcycle::{ActiveRun, LightcycleState, RunEnvironment, SourceSim};
use crate::load::{DirectoryLoadFailed, DirectoryLoaded, DirectoryRequested};
use crate::music::sfx::MusicSfx;
use crate::pacman::sim::{PacPhase, PacSim};
use crate::platformer::sim::{PlatformerPhase, PlatformerSim};
use crate::plinko::sim::{PlinkoPhase, PlinkoSim};
use crate::plugins::transition::{ModeTransition, gods_eye_pose};
use crate::qbert::sim::{QbertPhase, QbertSim};
use crate::snake::sim::SnakeSim;
use crate::state::{
    CacheState, DirectorySceneRoot, FloodState, HistoryState, InteractionMode, LightcycleSceneRoot,
    NavigatorResource, OrbitCameraResource, PauseState, StackMotion, TrailSceneRoot,
};
use crate::stealth::sim::{StealthPhase, StealthSim};
use crate::surfer::sim::{SurferPhase, SurferSim};
use crate::tetris::sim::{TetrisPhase, TetrisSim};
use bevy::asset::RenderAssetUsages;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
mod space;
pub(crate) use space::*;
mod input;
pub(crate) use input::*;
mod disc;
pub(crate) use disc::*;
mod fields;
pub(crate) use fields::*;
mod run;
pub(crate) use run::*;
mod document;
pub(crate) use document::*;
mod load;
pub(crate) use load::*;
mod character;
pub(crate) use character::*;
mod entry;
pub(crate) use entry::*;
mod decor;
pub(crate) use decor::*;
mod trail;
pub(crate) use trail::*;
mod camera;
pub(crate) use camera::*;
mod step;
pub(crate) use step::*;
mod city;
pub(crate) use city::*;
mod entities;
pub(crate) use entities::*;
mod assets;
pub(crate) use assets::*;

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
    unit_cube: Handle<Mesh>,
    entry_beam_mesh: Handle<Mesh>,
    entry_halo_mesh: Handle<Mesh>,
    cycle_scene: Handle<WorldAsset>,
    trail_material: Handle<StandardMaterial>,
    /// The translucent wall of the memory flood, and its lit crest.
    flood_material: Handle<StandardMaterial>,
    flood_crest_material: Handle<StandardMaterial>,
    /// The collector's sweep: the visible cause of the GC stall.
    gc_sweep_material: Handle<StandardMaterial>,
    wall_material: Handle<StandardMaterial>,
    city_floor_material: Handle<StandardMaterial>,
    city_foundation_material: Handle<StandardMaterial>,
    city_glass_material: Handle<StandardMaterial>,
    city_accent_materials: [[Handle<StandardMaterial>; 2]; 4],
    portal_material: Handle<StandardMaterial>,
    portal_bar_material: Handle<StandardMaterial>,
    dir_tower_material: Handle<StandardMaterial>,
    file_tower_material: Handle<StandardMaterial>,
    markdown_tower_material: Handle<StandardMaterial>,
    source_tower_material: Handle<StandardMaterial>,
    disc_floor_material: Handle<StandardMaterial>,
    disc_ring_material: Handle<StandardMaterial>,
    disc_plinth_material: Handle<StandardMaterial>,
    disc_hazard_material: Handle<StandardMaterial>,
    disc_opponent_material: Handle<StandardMaterial>,
    disc_player_disc_material: Handle<StandardMaterial>,
    disc_pickup_material: Handle<StandardMaterial>,
    disc_safe_pad_material: Handle<StandardMaterial>,
    /// One accent per [`SourceLanguage`], indexed by [`disc_language_index`].
    disc_accent_materials: [Handle<StandardMaterial>; SourceLanguage::COUNT],
    /// Flat cylinder thrown and returned during a fight.
    disc_mesh: Handle<Mesh>,
    /// Taller cylinder body for the Recognizer opponent.
    recognizer_mesh: Handle<Mesh>,
    /// Blocky rock body and beam tracer for the asteroid field.
    rock_material: Handle<StandardMaterial>,
    beam_material: Handle<StandardMaterial>,
    /// Power-up orb and sealed-exit bar for a snake ring.
    snake_food_material: Handle<StandardMaterial>,
    snake_lock_material: Handle<StandardMaterial>,
    /// The Tron runner, and the slabs and door of a platformer level.
    tron_scene: Handle<WorldAsset>,
    /// The same file as a `Gltf`, for the walk clip the scene cannot expose.
    tron_gltf: Handle<Gltf>,
    platform_material: Handle<StandardMaterial>,
    exit_material: Handle<StandardMaterial>,
    /// Bricks, ball and court walls for the breaker.
    brick_material: Handle<StandardMaterial>,
    ball_material: Handle<StandardMaterial>,
    court_material: Handle<StandardMaterial>,
    /// Floor, cover, guards and their vision cones for the stealth run.
    stealth_floor_material: Handle<StandardMaterial>,
    stealth_wall_material: Handle<StandardMaterial>,
    stealth_cone_material: Handle<StandardMaterial>,
    stealth_exit_material: Handle<StandardMaterial>,
    /// Unit-length cone with the stealth half-angle, scaled by its range.
    vision_cone: Handle<Mesh>,
    /// Water ribbon, rocks, boost gates and the finish gate for the surfer.
    surfer_water_material: Handle<StandardMaterial>,
    surfer_rock_material: Handle<StandardMaterial>,
    surfer_gate_material: Handle<StandardMaterial>,
    surfer_finish_material: Handle<StandardMaterial>,
    /// Bug bodies and beam bolts for the Galaga field.
    galaga_bug_material: Handle<StandardMaterial>,
    galaga_beam_material: Handle<StandardMaterial>,
    /// Arcade block: shared materials for the seven fixed-screen games.
    gem_materials: [Handle<StandardMaterial>; 4],
    tetris_materials: [Handle<StandardMaterial>; 7],
    qbert_cube_dim: Handle<StandardMaterial>,
    qbert_cube_lit: Handle<StandardMaterial>,
    qbert_enemy_material: Handle<StandardMaterial>,
    plinko_pin_material: Handle<StandardMaterial>,
    plinko_ball_material: Handle<StandardMaterial>,
    bomber_crate_material: Handle<StandardMaterial>,
    bomber_bomb_material: Handle<StandardMaterial>,
    document_floor_material: Handle<StandardMaterial>,
    document_rule_material: Handle<StandardMaterial>,
    document_margin_material: Handle<StandardMaterial>,
    document_ink_material: Handle<StandardMaterial>,
    document_heading_material: Handle<StandardMaterial>,
    document_folio_material: Handle<StandardMaterial>,
    document_focus_material: Handle<StandardMaterial>,
    crash_material: Handle<StandardMaterial>,
    entry_beam_material: Handle<StandardMaterial>,
    entry_halo_material: Handle<StandardMaterial>,
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

/// The player's thrown disc.
#[derive(Component)]
pub(crate) struct PlayerDiscEntity;

/// The Recognizer opponent's body.
#[derive(Component)]
pub(crate) struct OpponentEntity;

/// The opponent's disc.
#[derive(Component)]
pub(crate) struct OpponentDiscEntity;

/// One pickup waiting on a ring floor, keyed into `DiscLayout::pickups`.
#[derive(Component)]
pub(crate) struct DiscPickupEntity {
    index: usize,
    phase: f32,
}

/// One pooled rock in the asteroid field, keyed into `AsteroidsSim::rocks`.
#[derive(Component)]
pub(crate) struct RockEntity {
    index: usize,
}

/// One pooled beam in the asteroid field, keyed into `AsteroidsSim::beams`.
#[derive(Component)]
pub(crate) struct BeamEntity {
    index: usize,
}

/// One pooled bug in the Galaga field, keyed into `GalagaSim::bugs`.
#[derive(Component)]
pub(crate) struct BugEntity {
    index: usize,
}

/// One pooled beam in the Galaga field, keyed into `GalagaSim::beams`.
#[derive(Component)]
pub(crate) struct GalagaBeamEntity {
    index: usize,
}

/// One pooled dot in the Pac-Man maze, keyed by its cell.
#[derive(Component)]
pub(crate) struct DotEntity {
    cell: (i32, i32),
}

/// One pooled ghost in the Pac-Man maze, keyed into `PacSim::ghosts`.
#[derive(Component)]
pub(crate) struct GhostEntity {
    index: usize,
}

/// One pooled cell of the Columns well, keyed by its row-major index.
#[derive(Component)]
pub(crate) struct GemEntity {
    index: usize,
}

/// One pooled cell of the Tetris board, keyed by its row-major index.
#[derive(Component)]
pub(crate) struct BlockEntity {
    index: usize,
}

/// One pooled obstacle cube on the Frogger highway.
#[derive(Component)]
pub(crate) struct FrogObstacleEntity {
    index: usize,
}

/// One cube of the Q*bert pyramid, keyed by its row/index pair.
#[derive(Component)]
pub(crate) struct QbertCubeEntity {
    row: usize,
    index: usize,
}

/// One pooled enemy on the Q*bert pyramid, keyed into `QbertSim::enemies`.
#[derive(Component)]
pub(crate) struct QbertEnemyEntity {
    index: usize,
}

/// One pooled crate in the Bomberman room, keyed by its cell.
#[derive(Component)]
pub(crate) struct BomberCrateEntity {
    cell: (i32, i32),
}

/// One pooled bomb in the Bomberman room, keyed into `BomberSim::bombs`.
#[derive(Component)]
pub(crate) struct BomberBombEntity {
    index: usize,
}

/// One pooled ball on the Plinko board, keyed into `PlinkoSim::balls`.
#[derive(Component)]
pub(crate) struct PlinkoBallEntity {
    index: usize,
}

/// One power-up on a snake ring, keyed into `SnakeSim::food`.
#[derive(Component)]
pub(crate) struct SnakeFoodEntity {
    index: usize,
}

/// The bar sealing a snake ring's exit until enough power-ups are collected.
#[derive(Component)]
pub(crate) struct SnakeGateLock;

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
    fn at(base: Vec3) -> Self {
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

/// The breaker's ball.
#[derive(Component)]
pub(crate) struct BallEntity;

/// One brick of a breaker wall, keyed into `BreakerSim::bricks`.
#[derive(Component)]
pub(crate) struct BrickEntity {
    index: usize,
}

/// One patrol's body, keyed into `StealthSim::guards`.
#[derive(Component)]
pub(crate) struct GuardEntity {
    index: usize,
}

/// One patrol's field-of-vision cone, keyed into `StealthSim::guards`.
#[derive(Component)]
pub(crate) struct GuardConeEntity {
    index: usize,
    /// This guard's own cone, because its shape is cut to what the guard can
    /// actually see. It starts as the plain fan and is replaced once the sim's
    /// rays are available, which is on the first frame.
    mesh: Option<Handle<Mesh>>,
}

// The query types the sync systems are written in. Every minigame keeps a pool of
// entities that are spawned once and then shown, hidden and moved to match its
// sim, and writing that query out per system made the signatures unreadable: the
// alias says which marker keys the pool and what the sim drives on it.
//
// `F` is the filter that keeps sibling pools apart, since two pools of the same
// shape would otherwise match each other's entities.

/// A pool of sim-driven entities: the marker that keys it, plus the transform and
/// visibility the sim's own copy of the state drives.
type Pooled<'w, 's, M, F = ()> =
    Query<'w, 's, (&'static M, &'static mut Transform, &'static mut Visibility), F>;

/// A pool whose entries also wear a material the sim refreshes.
type PooledTinted<'w, 's, M, F = ()> = Query<
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
type PooledPosed<'w, 's, M, F = ()> = Query<'w, 's, (&'static M, &'static mut Transform), F>;

/// A pool the sim moves and re-tints.
type PooledPosedTinted<'w, 's, M, F = ()> = Query<
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
type PooledShown<'w, 's, M, F = ()> = Query<'w, 's, (&'static M, &'static mut Visibility), F>;

/// The marker an entity carries, and the lookalikes it must not be confused with.
/// These exclusions are what let Bevy prove two queries cannot collide.
type Only<A, B, C> = (With<A>, Without<B>, Without<C>);

/// One occupant of an arena it shares with others, told apart by its marker.
type Fighter<'w, 's, M, Other, ItsDisc> =
    Query<'w, 's, (&'static mut Transform, &'static mut Visibility), Only<M, Other, ItsDisc>>;

/// Keeps an entity clear of the others that share its arena.
type FreeOf<A, B, C> = (Without<A>, Without<B>, Without<C>);

/// Keeps two pools that share an arena from matching each other's entities.
type Apart<A, B = CycleEntity> = (Without<A>, Without<B>);

/// The lightcycle's own entities, which no minigame pool may claim.
type OutOfCycle = Without<CycleEntity>;

/// Everything a run spawns, so a room change can clear the lot in one query.
type SceneEntities<'w, 's> =
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
            config::LIGHTCYCLE_CAMERA_MIN_PITCH - base_pitch,
            config::LIGHTCYCLE_CAMERA_MAX_PITCH - base_pitch,
        );
    }

    /// Eases free look back behind the cycle after the button is released, with
    /// a frame-rate independent time constant.
    fn recenter_look(&mut self, delta_seconds: f32) {
        if self.look == Vec2::ZERO {
            return;
        }

        let blend = 1.0 - (-delta_seconds / config::LIGHTCYCLE_CAMERA_LOOK_RECENTER).exp();
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
    offset: Vec3,
    look: Vec3,
    height: f32,
}

#[cfg(test)]
mod tests;
