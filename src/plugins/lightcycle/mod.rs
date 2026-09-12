use crate::asteroids::{AsteroidsPhase, AsteroidsSim};
use crate::bomberman::{BomberPhase, BomberSim};
use crate::breaker::{BreakerPhase, BreakerSim};
use crate::columns::{ColumnsPhase, ColumnsSim};
use crate::config;
use crate::disc::{
    DiscEvents, DiscLayout, DiscPhase, DiscSim, PlayerSnapshot, SourceGame, SourceLanguage,
    SourceLoadFailed, SourceLoadState, SourceLoaded, SourceRequested, WarpRequested,
    build_capped_disc_arena, build_disc_arena, build_flat_arena,
};
use crate::document::{
    DocumentLayout, DocumentLoadFailed, DocumentLoadState, DocumentLoaded, DocumentRequested,
    build_document_arena_from_parse,
    parse::{ParseLimits, parse_markdown_bytes},
};
use crate::filesystem::FileNode;
use crate::frogger::{FroggerPhase, FroggerSim};
use crate::galaga::{GalagaPhase, GalagaSim};
use crate::lightcycle::logic::{
    Arena, ArenaKind, CellContent, CityStructure, CityStructureKind, CityTheme, CrashReason,
    GatePlacement, Heading, LightcycleSim, ParentPortal, RunPhase, StepOutcome, Wall,
    classify_next_content, road_plates, stable_path_seed,
};
use crate::lightcycle::{ActiveRun, LightcycleState, RunEnvironment, SourceSim};
use crate::load::{DirectoryLoadFailed, DirectoryLoaded, DirectoryRequested};
use crate::music::MusicSfx;
use crate::pacman::{PacPhase, PacSim};
use crate::platformer::{PlatformerPhase, PlatformerSim};
use crate::plinko::{PlinkoPhase, PlinkoSim};
use crate::plugins::transition::{ModeTransition, gods_eye_pose};
use crate::qbert::{QbertPhase, QbertSim};
use crate::snake::SnakeSim;
use crate::state::{
    CacheState, DirectorySceneRoot, FloodState, HistoryState, InteractionMode, LightcycleSceneRoot,
    NavigatorResource, OrbitCameraResource, PauseState, StackMotion, TrailSceneRoot,
};
use crate::stealth::{StealthPhase, StealthSim};
use crate::surfer::{SurferPhase, SurferSim};
use crate::tetris::{TetrisPhase, TetrisSim};
use bevy::asset::RenderAssetUsages;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
struct FloodEntity;

/// The collector's sweep, crossing a directory arena while it stalls the world.
#[derive(Component)]
struct GcSweepEntity;

/// One frame of a directory's call stack: its path depth, and the pose the
/// animation oscillates around.
#[derive(Component)]
struct StackFrameEntity {
    level: usize,
    base: Vec3,
}

/// Small mesh burst emitted at the crash point.
#[derive(Component)]
struct CrashDebris {
    velocity: Vec3,
    life: f32,
    max_life: f32,
    initial_scale: f32,
}

/// Entity owned by the directory-tower transport effect.
#[derive(Component)]
struct EntryTransportEntity;

#[derive(Component)]
struct EntryBeam;

#[derive(Component)]
struct EntryHalo {
    phase: f32,
}

/// A post or lintel of the parent gate.
#[derive(Component)]
struct GateFrame;

/// A light bar sweeping up through the parent gate's opening.
#[derive(Component)]
struct GateScanBar {
    /// Position in the sweep at startup, so the bars are evenly spaced.
    offset: f32,
    /// Height of the opening the bar travels up before wrapping.
    travel: f32,
}

#[derive(Component)]
struct CityBeacon {
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
struct CharacterWalk(AnimationNodeIndex);

/// Marks everything spawned beneath an on-foot character.
///
/// The loader creates the `AnimationPlayer` deep inside the scene, so there is
/// nothing to tag at spawn time: the character's subtree is walked instead,
/// which also picks up descendants that only appear a frame or two later.
#[derive(Component)]
struct CharacterModel;

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

/// Wraps an angle into `[-PI, PI)` so recentering unwinds the short way round
/// however many times a drag has spun the camera about the cycle.
fn wrap_angle(angle: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    (angle + PI).rem_euclid(TAU) - PI
}

/// Lane markings use a district's primary accent, so ground seams use the
/// secondary one to stay readable against them.
const CITY_TRIM_ACCENT: usize = 1;

/// Index into [`LightcycleAssets::disc_accent_materials`].
fn disc_language_index(language: SourceLanguage) -> usize {
    SourceLanguage::ALL
        .iter()
        .position(|candidate| *candidate == language)
        .unwrap_or(0)
}

/// Lit transmissive sheet: the directional light and the arena behind it show
/// through, with a cyan tint and a hard specular so it reads as glass rather
/// than an unlit neon brick.
fn trail_glass_material() -> StandardMaterial {
    StandardMaterial {
        base_color: config::LIGHTCYCLE_TRAIL_COLOR,
        perceptual_roughness: 0.08,
        metallic: 0.02,
        specular_transmission: 0.92,
        thickness: 0.28,
        ior: 1.45,
        attenuation_color: config::LIGHTCYCLE_TRAIL_ATTENUATION,
        attenuation_distance: 0.8,
        emissive: LinearRgba::from(config::PCB_TRACE_COLOR) * config::PCB_TRACE_EMISSIVE,
        clearcoat: 1.0,
        clearcoat_perceptual_roughness: 0.06,
        double_sided: true,
        cull_mode: None,
        ..default()
    }
}

fn in_lightcycle_mode(mode: Res<InteractionMode>) -> bool {
    *mode == InteractionMode::Lightcycle
}

fn build_active_run(path: &Path, nodes: Vec<FileNode>) -> ActiveRun {
    let cells: HashMap<_, _> = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (tower_position(node.grid_pos), index))
        .collect();
    let parent_gate = path
        .parent()
        .is_some()
        .then(|| GatePlacement::for_path(path, config::LIGHTCYCLE_PORTAL_WIDTH_CELLS));
    let mut arena = Arena::from_nodes(
        cells.keys().copied(),
        parent_gate,
        config::LIGHTCYCLE_ARENA_PADDING,
        config::LIGHTCYCLE_MIN_ARENA_SPAN,
    );
    arena.generate_city(
        path,
        cells.keys().copied(),
        config::LIGHTCYCLE_CITY_STRUCTURE_SEED_CHANCE,
        config::LIGHTCYCLE_TOWER_STRIDE,
    );

    let sim = spawn_sim(&arena, &cells);

    ActiveRun {
        sim,
        arena,
        environment: RunEnvironment::Directory { nodes, cells },
        crash_label: None,
        entering_label: None,
    }
}

fn build_document_run(path: &Path, bytes: &[u8]) -> ActiveRun {
    let parsed = parse_markdown_bytes(bytes, ParseLimits::default(), false);
    let (arena, layout) = build_document_arena_from_parse(path, parsed);
    let sim = spawn_sim(&arena, &HashMap::new());
    ActiveRun {
        sim,
        arena,
        environment: RunEnvironment::Document {
            path: path.to_path_buf(),
            name: path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("document")
                .to_string(),
            layout,
            focused_block: None,
        },
        crash_label: None,
        entering_label: None,
    }
}

/// Builds the run for a source file: a fresh player cycle at the ring center
/// plus the sim for whichever game the language hosts.
fn build_source_run(path: &Path, language: SourceLanguage, bytes: &[u8]) -> ActiveRun {
    let game = language.game();
    let (arena, layout) = match game {
        // The field and the snake ring want a bounded playfield whatever the
        // file size; only disc wars scales its coliseum with the file.
        SourceGame::Asteroids => {
            build_capped_disc_arena(path, language, bytes, config::ASTEROIDS_RADIUS_CELLS)
        }
        SourceGame::Snake => {
            build_capped_disc_arena(path, language, bytes, config::SNAKE_RADIUS_CELLS)
        }
        SourceGame::DiscWars => build_disc_arena(path, language, bytes),
        // The off-grid games carry a metadata-only arena; their level is their
        // own. The half-extent only sizes the (unused) metadata box.
        SourceGame::Platformer | SourceGame::Breaker | SourceGame::Stealth => {
            build_flat_arena(path, language, bytes, config::PLATFORMER_HALF_EXTENT)
        }
        SourceGame::RiverSurfer => {
            build_flat_arena(path, language, bytes, config::SURFER_HALF_EXTENT)
        }
        SourceGame::Galaga => build_flat_arena(path, language, bytes, config::GALAGA_HALF_EXTENT),
        // The whole arcade block shares one arena extent; each game's board is
        // small enough to fit inside it.
        SourceGame::PacMan
        | SourceGame::Columns
        | SourceGame::Tetris
        | SourceGame::Frogger
        | SourceGame::Qbert
        | SourceGame::Bomberman
        | SourceGame::Plinko => build_flat_arena(path, language, bytes, config::ARCADE_HALF_EXTENT),
    };
    let sim = LightcycleSim::start(layout.player_spawn, layout.player_spawn_heading);
    let sim_state = match game {
        SourceGame::DiscWars => SourceSim::DiscWars(DiscSim::new(&layout)),
        SourceGame::Asteroids => {
            let mut field = Box::new(AsteroidsSim::new(
                layout.seed,
                ring_center_world(&layout),
                ring_radius_world(&layout),
            ));
            // The parked cycle starts on the ring's spawn heading so the aim
            // angle is already meaningful when the field ends and the bike is
            // handed back.
            field.angle = heading_facing(layout.player_spawn_heading);
            SourceSim::Asteroids(field)
        }
        SourceGame::Snake => SourceSim::Snake(SnakeSim::new(
            layout.seed,
            layout.player_spawn,
            &ring_food_cells(&arena),
            config::SNAKE_FOOD_TARGET,
        )),
        SourceGame::Platformer => SourceSim::Platformer(Box::new(PlatformerSim::new(
            layout.seed,
            level_metres(&layout),
        ))),
        SourceGame::Breaker => SourceSim::Breaker(Box::new(BreakerSim::new(layout.seed))),
        SourceGame::Stealth => SourceSim::Stealth(Box::new(StealthSim::new(layout.seed))),
        SourceGame::RiverSurfer => {
            SourceSim::Surfer(Box::new(SurferSim::new(layout.seed, layout.signals.lines)))
        }
        SourceGame::Galaga => {
            SourceSim::Galaga(Box::new(GalagaSim::new(layout.seed, layout.signals.lines)))
        }
        SourceGame::PacMan => SourceSim::PacMan(Box::new(PacSim::new(layout.seed))),
        SourceGame::Columns => {
            SourceSim::Columns(Box::new(ColumnsSim::new(layout.seed, layout.signals.lines)))
        }
        SourceGame::Tetris => {
            SourceSim::Tetris(Box::new(TetrisSim::new(layout.seed, layout.signals.lines)))
        }
        SourceGame::Frogger => SourceSim::Frogger(Box::new(FroggerSim::new(layout.seed))),
        SourceGame::Qbert => SourceSim::Qbert(Box::new(QbertSim::new(layout.seed))),
        SourceGame::Bomberman => SourceSim::Bomberman(Box::new(BomberSim::new(layout.seed))),
        SourceGame::Plinko => SourceSim::Plinko(Box::new(PlinkoSim::new(layout.seed))),
    };
    ActiveRun {
        sim,
        arena,
        environment: RunEnvironment::Source {
            path: path.to_path_buf(),
            name: path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("source")
                .to_string(),
            language,
            layout: Box::new(layout),
            sim: sim_state,
            focused_block: None,
        },
        crash_label: None,
        entering_label: None,
    }
}

/// Level length for an off-grid run, in metres: longer file, longer level.
fn level_metres(layout: &DiscLayout) -> f32 {
    layout.signals.lines as f32 * config::PLATFORMER_METRES_PER_LINE
}

/// Middle of a ring, in world units.
fn ring_center_world(layout: &DiscLayout) -> (f32, f32) {
    (
        layout.center.0 as f32 * config::GRID_SPACING,
        layout.center.1 as f32 * config::GRID_SPACING,
    )
}

/// Inner radius of a ring wall, in world units.
fn ring_radius_world(layout: &DiscLayout) -> f32 {
    layout.radius as f32 * config::GRID_SPACING
}

/// Driveable cells of a ring, in a stable order, for scattering power-ups.
fn ring_food_cells(arena: &Arena) -> Vec<(i32, i32)> {
    arena.roads.iter().copied().collect()
}

/// True when `cell` is a ring's close gate.
fn is_ring_gate(arena: &Arena, cell: (i32, i32)) -> bool {
    arena
        .parent_portal
        .as_ref()
        .is_some_and(|portal| portal.contains(cell))
}

fn spawn_sim(arena: &Arena, cells: &HashMap<(i32, i32), usize>) -> LightcycleSim {
    let blocked =
        |cell: (i32, i32)| cells.contains_key(&cell) || arena.street_walls.contains(&cell);

    if let Some((spawn, heading)) = arena.spawn_with_runway(
        blocked,
        config::LIGHTCYCLE_SPAWN_SEARCH_RADIUS,
        config::LIGHTCYCLE_SPAWN_RUNWAY_CELLS,
    ) {
        return LightcycleSim::start(spawn, heading);
    }

    // Nowhere to ride at all: hold the run until a restart or another folder
    // replaces the map.
    let cell = arena
        .nearest_empty_cell(blocked, config::LIGHTCYCLE_SPAWN_SEARCH_RADIUS)
        .unwrap_or_else(|| arena.center());
    LightcycleSim::ready(cell, Heading::PosX)
}

/// Starts the flight between the two modes when `M` is pressed.
///
/// Nothing about the world changes here. The run for a lightcycle mode is built
/// but parked, because the flight has to know which road it is diving into and
/// where its chase rig will end up before [`apply_mode_swap`] puts that arena on
/// screen at the top of the climb.
fn toggle_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mode: Res<InteractionMode>,
    mut transition: ResMut<ModeTransition>,
    mut state: ResMut<LightcycleState>,
    navigator: Res<NavigatorResource>,
    mut orbit: ResMut<OrbitCameraResource>,
    camera: Single<&Transform, With<Camera3d>>,
) {
    if !keys.just_pressed(KeyCode::KeyM) || transition.is_active() {
        return;
    }

    let from = **camera;
    match *mode {
        InteractionMode::Lightcycle => {
            // Park the orbit rig now rather than letting it lerp home after the
            // swap, so the flight can aim at the exact pose the explorer camera
            // will hold when it takes over.
            orbit.reset_target();
            orbit.target = Vec3::ZERO;
            let to =
                Transform::from_translation(orbit.position()).looking_at(orbit.target, Vec3::Y);
            // Rise out of the street the cycle is on, so the overhead shot
            // arrives holding the heading the run was riding.
            let road = state
                .run
                .as_ref()
                .map_or(Vec3::X, |run| pose_forward(&cycle_cell_pose(&run.sim)));
            let apex = gods_eye_pose(orbit.target, road);
            transition.start(InteractionMode::Explorer, from, apex, to, orbit.target);
        }
        InteractionMode::Explorer => {
            let run = build_active_run(&navigator.0.current_path, navigator.0.entries.clone());
            let (to, focus, road) = chase_landing_pose(&run);
            let apex = gods_eye_pose(focus, road);
            state.pending_run = Some(run);
            transition.start(InteractionMode::Lightcycle, from, apex, to, focus);
        }
    }
}

/// Tears down the old world and builds the new one, at the top of the flight's
/// climb, where the camera is highest and the flash covers the frame.
#[allow(clippy::too_many_arguments)]
fn apply_mode_swap(
    mut transition: ResMut<ModeTransition>,
    mut mode: ResMut<InteractionMode>,
    mut state: ResMut<LightcycleState>,
    navigator: Res<NavigatorResource>,
    mut cache: ResMut<CacheState>,
    mut flood: ResMut<FloodState>,
    mut history: ResMut<HistoryState>,
    assets: Res<LightcycleAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    old_lightcycle_entities: SceneEntities,
) {
    let Some(target) = transition.pending_swap() else {
        return;
    };

    despawn_lightcycle_entities(&mut commands, &old_lightcycle_entities);
    state.clock = 0.0;
    state.run = None;
    state.crash_fx = None;
    state.entry_fx = None;
    state.restore_directory = false;

    if target == InteractionMode::Lightcycle
        && let Some(run) = state.pending_run.take()
    {
        // Riding out of the explorer view spawns the same directory run the
        // loader would, so it has to be dressed and logged the same way. The
        // opening room's `DirectoryLoaded` arrived while the explorer camera was
        // still up and was dropped, which left that one room without its stack,
        // its highway plates or its flood.
        let path = navigator.0.current_path.clone();
        // The first ride is a grace period: the room is dressed like any other,
        // but nothing in it is hunting you yet.
        state.grace_room = !state.rides_started;
        state.rides_started = true;
        spawn_run_entities(&mut commands, &assets, &mut meshes, &run);
        decorate_directory_run(
            &mut commands,
            &assets,
            &mut meshes,
            &mut state,
            &mut flood,
            &path,
            &run,
        );
        // The room you started in is a room like any other: it belongs in the
        // commit log, and having been there counts as a cache hit later.
        history.commit(&path);
        let hit = !cache.visited.insert(path);
        state.cache_boost = if hit {
            config::CACHE_BOOST_SECONDS
        } else {
            0.0
        };
        state.run = Some(run);
    }

    state.pending_run = None;
    *mode = target;
    transition.mark_swapped();
}

fn despawn_lightcycle_entities(commands: &mut Commands, old_lightcycle_entities: &SceneEntities) {
    for entity in old_lightcycle_entities {
        commands.entity(entity).despawn();
    }
}

fn spawn_run_entities(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    run: &ActiveRun,
) {
    let pose = cycle_cell_pose(&run.sim);
    commands.spawn((
        LightcycleSceneRoot,
        CycleEntity,
        Transform::from_translation(pose_world_position(&pose)).with_rotation(pose_rotation(&pose)),
        Visibility::default(),
        ChaseCamera {
            forward: pose_forward(&pose),
            look: Vec2::ZERO,
        },
        Pickable::IGNORE,
        children![(
            WorldAssetRoot(assets.cycle_scene.clone()),
            Transform::from_rotation(Quat::from_rotation_y(config::LIGHTCYCLE_MODEL_YAW))
                .with_scale(Vec3::splat(config::LIGHTCYCLE_MODEL_SCALE)),
        )],
    ));

    spawn_city_floor(commands, assets, meshes, &run.arena);
    match &run.environment {
        RunEnvironment::Directory { .. } => {
            spawn_road_markings(commands, assets, meshes, &run.arena);
            spawn_city_structures(commands, assets, meshes, &run.arena);
            spawn_arena_walls(commands, assets, &run.arena);
            spawn_towers(commands, assets, meshes, run);
        }
        RunEnvironment::Document { layout, .. } => {
            spawn_document_page(commands, assets, meshes, &run.arena, layout);
            spawn_arena_walls(commands, assets, &run.arena);
            spawn_document_focus_marker(commands, assets, run);
        }
        RunEnvironment::Source {
            language,
            layout,
            sim,
            ..
        } => match sim {
            // The field shares the ring and its gate, but stands alone: no
            // hazards, pickups or opponent, and its own pooled rocks/beams.
            SourceSim::Asteroids(field) => {
                spawn_ring_shell(commands, meshes, assets, &run.arena, layout, *language);
                spawn_asteroid_field(commands, assets, field);
            }
            // Snake is the ordinary grid run plus power-ups and a locked gate.
            SourceSim::Snake(snake) => {
                spawn_ring_shell(commands, meshes, assets, &run.arena, layout, *language);
                spawn_snake_field(commands, assets, &run.arena, snake);
            }
            SourceSim::Platformer(level) => {
                spawn_platformer_level(commands, assets, meshes, *language, level);
            }
            SourceSim::Breaker(level) => {
                spawn_breaker_court(commands, assets, level);
            }
            SourceSim::Stealth(room) => {
                spawn_stealth_room(commands, assets, meshes, room);
            }
            SourceSim::Surfer(surfer) => {
                spawn_surfer_course(commands, assets, meshes, surfer);
            }
            SourceSim::Galaga(sim) => {
                spawn_galaga_field(commands, assets, sim);
            }
            SourceSim::PacMan(sim) => {
                spawn_pac_maze(commands, assets, meshes, sim);
            }
            SourceSim::Columns(sim) => {
                spawn_gem_well(commands, assets, sim);
            }
            SourceSim::Tetris(sim) => {
                spawn_tetris_board(commands, assets, sim);
            }
            SourceSim::Frogger(sim) => {
                spawn_frogger_highway(commands, assets, sim);
            }
            SourceSim::Qbert(sim) => {
                spawn_qbert_pyramid(commands, assets, sim);
            }
            SourceSim::Bomberman(sim) => {
                spawn_bomber_room(commands, assets, meshes, sim);
            }
            SourceSim::Plinko(sim) => {
                spawn_plinko_board(commands, assets, meshes, sim);
            }
            SourceSim::DiscWars(disc) => {
                spawn_disc_arena(
                    commands, assets, meshes, &run.arena, layout, *language, disc,
                );
                spawn_disc_focus_marker(commands, assets, run);
            }
        },
    }
    spawn_trail_ribbon(commands, assets, meshes, run);
}

fn spawn_trail_ribbon(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    run: &ActiveRun,
) {
    commands.spawn((
        TrailSceneRoot,
        Mesh3d(meshes.add(build_trail_mesh(&run.sim))),
        MeshMaterial3d(assets.trail_material.clone()),
        Pickable::IGNORE,
    ));
}

/// Builds a disc-wars ring: circular floor, ring wall, plinth, hazards, safe
/// pads, close gate, pickups, and the two fighters.
/// Ring wall, corner plinth and close gate. Shared by every source arena, so
/// the asteroid field gets the same containment and the same way out.
fn spawn_ring_shell(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &LightcycleAssets,
    arena: &Arena,
    layout: &DiscLayout,
    language: SourceLanguage,
) {
    let accent = assets.disc_accent_materials[disc_language_index(language)].clone();
    let center = layout.center;
    let radius = layout.radius as f32;

    // Split the lethal fill into the ring band and the corner fill, so the
    // ring reads as a wall and the rest as ground the ring sits in.
    let mut ring_cells = Vec::new();
    let mut plinth_cells = Vec::new();
    for &cell in &arena.street_walls {
        let dx = (cell.0 - center.0) as f32;
        let dz = (cell.1 - center.1) as f32;
        if (dx * dx + dz * dz).sqrt() <= radius + 1.0 {
            ring_cells.push(cell);
        } else {
            plinth_cells.push(cell);
        }
    }
    spawn_disc_cube_layer(
        commands,
        meshes,
        &ring_cells,
        assets.disc_ring_material.clone(),
        1.7,
        1.7,
    );
    spawn_disc_cube_layer(
        commands,
        meshes,
        &plinth_cells,
        assets.disc_plinth_material.clone(),
        0.08,
        2.0,
    );

    spawn_disc_gate(commands, assets, arena, layout, &accent);
}

fn spawn_disc_arena(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    arena: &Arena,
    layout: &DiscLayout,
    language: SourceLanguage,
    disc: &DiscSim,
) {
    let center = layout.center;
    spawn_ring_shell(commands, meshes, assets, arena, layout, language);

    let hazards: Vec<_> = layout.hazards.iter().copied().collect();
    spawn_disc_cube_layer(
        commands,
        meshes,
        &hazards,
        assets.disc_hazard_material.clone(),
        0.06,
        1.5,
    );
    let pads: Vec<_> = layout.safe_pads.iter().copied().collect();
    spawn_disc_cube_layer(
        commands,
        meshes,
        &pads,
        assets.disc_safe_pad_material.clone(),
        0.05,
        1.3,
    );

    for (index, spot) in layout.pickups.iter().enumerate() {
        let taken = disc.taken.contains(&index);
        let phase = index as f32 / layout.pickups.len().max(1) as f32;
        commands.spawn((
            LightcycleSceneRoot,
            DiscPickupEntity { index, phase },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.disc_pickup_material.clone()),
            Transform::from_translation(
                config::ground_position(spot.cell.0, spot.cell.1) + Vec3::Y * 0.45,
            )
            .with_scale(Vec3::splat(0.3)),
            if taken {
                Visibility::Hidden
            } else {
                Visibility::Visible
            },
            Pickable::IGNORE,
        ));
    }

    let opponent_visible = if disc.opponent.alive {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    commands.spawn((
        LightcycleSceneRoot,
        OpponentEntity,
        Mesh3d(assets.recognizer_mesh.clone()),
        MeshMaterial3d(assets.disc_opponent_material.clone()),
        Transform::from_translation(
            config::ground_position(disc.opponent.cell.0, disc.opponent.cell.1)
                + Vec3::Y * (config::RECOGNIZER_HEIGHT * 0.5),
        ),
        opponent_visible,
        Pickable::IGNORE,
    ));

    let player_disc_visible = if disc.player_disc.is_some() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    commands.spawn((
        LightcycleSceneRoot,
        PlayerDiscEntity,
        Mesh3d(assets.disc_mesh.clone()),
        MeshMaterial3d(assets.disc_player_disc_material.clone()),
        Transform::from_translation(config::ground_position(center.0, center.1)),
        player_disc_visible,
        Pickable::IGNORE,
    ));

    let opponent_disc_visible = if disc.opponent.disc.is_some() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    commands.spawn((
        LightcycleSceneRoot,
        OpponentDiscEntity,
        Mesh3d(assets.disc_mesh.clone()),
        MeshMaterial3d(assets.disc_opponent_material.clone()),
        Transform::from_translation(config::ground_position(center.0, center.1)),
        opponent_disc_visible,
        Pickable::IGNORE,
    ));
}

/// Spawns the pooled rock and beam bodies for an asteroid field.
///
/// The sim drives visibility and transforms; the pool is fixed because a rock
/// only ever splits into a bounded number of children.
fn spawn_asteroid_field(commands: &mut Commands, assets: &LightcycleAssets, sim: &AsteroidsSim) {
    let rock_count = config::ASTEROIDS_MAX_ROCKS.max(sim.rocks.len());
    for index in 0..rock_count {
        let live = sim.rocks.get(index);
        let radius = live.map_or(1.0, |rock| rock.size.radius());
        commands.spawn((
            LightcycleSceneRoot,
            RockEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.rock_material.clone()),
            Transform::from_xyz(0.0, radius, 0.0).with_scale(Vec3::splat(radius * 2.0)),
            if live.is_some() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }
    for index in 0..config::ASTEROIDS_MAX_BEAMS {
        let live = sim.beams.get(index);
        commands.spawn((
            LightcycleSceneRoot,
            BeamEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.beam_material.clone()),
            Transform::from_xyz(0.0, 0.35, 0.0),
            if live.is_some() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }
}

/// Spawns the pooled bug and beam bodies for a Galaga field.
///
/// The formation is a fixed grid, so the bug pool never grows; the sim drives
/// transforms and visibility.
fn spawn_galaga_field(commands: &mut Commands, assets: &LightcycleAssets, sim: &GalagaSim) {
    for (index, bug) in sim.bugs.iter().enumerate() {
        commands.spawn((
            LightcycleSceneRoot,
            BugEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.galaga_bug_material.clone()),
            Transform::from_xyz(bug.x, config::GALAGA_BUG_HEIGHT * 0.5, bug.z).with_scale(
                Vec3::new(
                    config::GALAGA_BUG_RADIUS * 2.0,
                    config::GALAGA_BUG_HEIGHT,
                    config::GALAGA_BUG_RADIUS * 2.0,
                ),
            ),
            if bug.alive {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }
    for index in 0..config::GALAGA_MAX_BEAMS {
        let live = sim.beams.get(index);
        commands.spawn((
            LightcycleSceneRoot,
            GalagaBeamEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.galaga_beam_material.clone()),
            Transform::from_xyz(0.0, 0.35, 0.0),
            if live.is_some() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }
}

/// Spawns the Pac-Man maze: static walls plus pooled dots and ghosts.
fn spawn_pac_maze(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    _meshes: &mut Assets<Mesh>,
    sim: &PacSim,
) {
    for row in 0..config::PAC_ROWS {
        for col in 0..config::PAC_COLS {
            if !PacSim::solid((col, row)) {
                continue;
            }
            let (x, z) = PacSim::center((col, row));
            commands.spawn((
                LightcycleSceneRoot,
                Mesh3d(assets.unit_cube.clone()),
                MeshMaterial3d(assets.stealth_wall_material.clone()),
                Transform::from_xyz(x, config::GRID_SPACING * 0.5, z)
                    .with_scale(Vec3::splat(config::GRID_SPACING)),
                Visibility::Visible,
                Pickable::IGNORE,
            ));
        }
    }
    for (index, ghost) in sim.ghosts.iter().enumerate() {
        commands.spawn((
            LightcycleSceneRoot,
            GhostEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.galaga_bug_material.clone()),
            Transform::from_xyz(ghost.x, 0.9, ghost.z).with_scale(Vec3::splat(1.5)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
    for &cell in &sim.dots {
        let (x, z) = PacSim::center(cell);
        commands.spawn((
            LightcycleSceneRoot,
            DotEntity { cell },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.snake_food_material.clone()),
            Transform::from_xyz(x, 0.35, z).with_scale(Vec3::splat(0.55)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
}

/// Spawns the pooled gem cells of a Columns well.
fn spawn_gem_well(commands: &mut Commands, assets: &LightcycleAssets, sim: &ColumnsSim) {
    let rendered = sim.render_board();
    for (index, colour) in rendered.iter().copied().enumerate() {
        let col = index % config::COLUMNS_COLS;
        let row = index / config::COLUMNS_COLS;
        let x = (col as f32 - (config::COLUMNS_COLS - 1) as f32 * 0.5) * 1.6;
        // Row 0 is the top of the well, so higher rows sit lower on screen.
        // The whole well is lifted above the arena floor.
        let y = ((config::COLUMNS_ROWS - 1) - row) as f32 * 1.6 + 0.8;
        commands.spawn((
            LightcycleSceneRoot,
            GemEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(
                assets.gem_materials[colour.unwrap_or(0) as usize % config::COLUMNS_GEM_COLORS]
                    .clone(),
            ),
            Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(1.5)),
            if colour.is_some() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }

    // A visible frame marks the playfield: side walls plus a floor bar.
    let board_half = config::COLUMNS_COLS as f32 * 0.8;
    let wall_x = board_half + 0.55;
    let board_height = config::COLUMNS_ROWS as f32 * 1.6;
    let wall_scale = Vec3::new(0.4, board_height + 0.6, 0.5);
    for x in [-wall_x, wall_x] {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.stealth_wall_material.clone()),
            Transform::from_xyz(x, board_height * 0.5 + 0.3, 0.0).with_scale(wall_scale),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.stealth_wall_material.clone()),
        Transform::from_xyz(0.0, 0.2, 0.0).with_scale(Vec3::new(wall_x * 2.0 + 0.8, 0.4, 0.5)),
        Visibility::Visible,
        Pickable::IGNORE,
    ));
}

/// Spawns the pooled block cells of a Tetris board.
fn spawn_tetris_board(commands: &mut Commands, assets: &LightcycleAssets, sim: &TetrisSim) {
    let rendered = sim.render_board();
    for (index, colour) in rendered.iter().copied().enumerate() {
        let col = index % config::TETRIS_COLS;
        let row = index / config::TETRIS_COLS;
        let x = (col as f32 - (config::TETRIS_COLS - 1) as f32 * 0.5) * 1.2;
        // Row 0 is the top of the board, so higher rows sit lower on screen.
        // The whole board is lifted above the arena floor.
        let y = ((config::TETRIS_ROWS - 1) - row) as f32 * 1.2 + 0.6;
        commands.spawn((
            LightcycleSceneRoot,
            BlockEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.tetris_materials[colour.unwrap_or(0) as usize % 7].clone()),
            Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(1.15)),
            if colour.is_some() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }

    // A visible frame marks the playfield: side walls plus a floor bar.
    let board_half = config::TETRIS_COLS as f32 * 0.6;
    let wall_x = board_half + 0.55;
    let board_height = config::TETRIS_ROWS as f32 * 1.2;
    let wall_scale = Vec3::new(0.4, board_height + 0.6, 0.5);
    for x in [-wall_x, wall_x] {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.stealth_wall_material.clone()),
            Transform::from_xyz(x, board_height * 0.5 + 0.3, 0.0).with_scale(wall_scale),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.stealth_wall_material.clone()),
        Transform::from_xyz(0.0, 0.15, 0.0).with_scale(Vec3::new(wall_x * 2.0 + 0.8, 0.3, 0.5)),
        Visibility::Visible,
        Pickable::IGNORE,
    ));
}

/// Spawns the pooled obstacle cubes of a Frogger highway.
fn spawn_frogger_highway(commands: &mut Commands, assets: &LightcycleAssets, sim: &FroggerSim) {
    let cells = sim.obstacle_cells();
    for (index, &cell) in cells.iter().enumerate() {
        let (x, z) = FroggerSim::center(cell);
        commands.spawn((
            LightcycleSceneRoot,
            FrogObstacleEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.galaga_bug_material.clone()),
            Transform::from_xyz(x, 0.6, z).with_scale(Vec3::splat(1.9)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
}

/// Spawns the Q*bert pyramid cubes and its pooled enemies.
fn spawn_qbert_pyramid(commands: &mut Commands, assets: &LightcycleAssets, sim: &QbertSim) {
    for row in 0..config::QBERT_ROWS {
        for index in 0..=row {
            let (x, z) = QbertSim::cube_position(row, index);
            let lit = sim.lit[QbertSim::cube_index(row, index)];
            let y =
                (config::QBERT_ROWS as f32 - 1.0 - row as f32) * config::QBERT_CUBE_HEIGHT * 0.5;
            commands.spawn((
                LightcycleSceneRoot,
                QbertCubeEntity { row, index },
                Mesh3d(assets.unit_cube.clone()),
                MeshMaterial3d(if lit {
                    assets.qbert_cube_lit.clone()
                } else {
                    assets.qbert_cube_dim.clone()
                }),
                Transform::from_xyz(x, y, z).with_scale(Vec3::new(
                    config::QBERT_CUBE_SPACING * 0.9,
                    config::QBERT_CUBE_HEIGHT,
                    config::QBERT_CUBE_SPACING * 0.9,
                )),
                Visibility::Visible,
                Pickable::IGNORE,
            ));
        }
    }
    for (index, enemy) in sim.enemies.iter().enumerate() {
        let (x, z) = QbertSim::cube_position(enemy.row, enemy.index);
        let y =
            (config::QBERT_ROWS as f32 - 1.0 - enemy.row as f32) * config::QBERT_CUBE_HEIGHT * 0.5
                + config::QBERT_CUBE_HEIGHT * 0.8;
        commands.spawn((
            LightcycleSceneRoot,
            QbertEnemyEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.qbert_enemy_material.clone()),
            Transform::from_xyz(x, y, z).with_scale(Vec3::splat(1.4)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
}

/// Spawns the Bomberman room: crates, bombs, walls and the exit marker.
fn spawn_bomber_room(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    _meshes: &mut Assets<Mesh>,
    sim: &BomberSim,
) {
    for row in 0..config::BOMBER_ROWS {
        for col in 0..config::BOMBER_COLS {
            let cell = (col, row);
            let (x, z) = BomberSim::center(cell);
            let is_border = col == 0
                || col == config::BOMBER_COLS - 1
                || row == 0
                || row == config::BOMBER_ROWS - 1;
            if is_border {
                commands.spawn((
                    LightcycleSceneRoot,
                    Mesh3d(assets.unit_cube.clone()),
                    MeshMaterial3d(assets.stealth_wall_material.clone()),
                    Transform::from_xyz(x, 1.0, z).with_scale(Vec3::splat(2.0)),
                    Visibility::Visible,
                    Pickable::IGNORE,
                ));
            } else if sim.crates.contains(&cell) {
                commands.spawn((
                    LightcycleSceneRoot,
                    BomberCrateEntity { cell },
                    Mesh3d(assets.unit_cube.clone()),
                    MeshMaterial3d(assets.bomber_crate_material.clone()),
                    Transform::from_xyz(x, 0.8, z).with_scale(Vec3::splat(1.8)),
                    Visibility::Visible,
                    Pickable::IGNORE,
                ));
            }
        }
    }
    let (ex, ez) = BomberSim::center(sim.exit);
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.stealth_exit_material.clone()),
        Transform::from_xyz(ex, 0.5, ez).with_scale(Vec3::new(1.9, 0.4, 1.9)),
        Visibility::Visible,
        Pickable::IGNORE,
    ));
    for index in 0..config::BOMBER_MAX_BOMBS {
        commands.spawn((
            LightcycleSceneRoot,
            BomberBombEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.bomber_bomb_material.clone()),
            Transform::from_xyz(0.0, 0.6, 0.0),
            Visibility::Hidden,
            Pickable::IGNORE,
        ));
    }
}

/// Spawns the Plinko board: static pins and buckets plus pooled balls.
fn spawn_plinko_board(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    _meshes: &mut Assets<Mesh>,
    sim: &PlinkoSim,
) {
    // A dark backdrop behind the board makes the pins and balls read clearly.
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.qbert_cube_dim.clone()),
        Transform::from_xyz(0.0, 0.0, -0.35).with_scale(Vec3::new(
            config::PLINKO_WIDTH + 1.5,
            config::PLINKO_HEIGHT + 1.5,
            0.2,
        )),
        Visibility::Visible,
        Pickable::IGNORE,
    ));
    // The drop rail along the top, where the cycle slides to aim.
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.stealth_wall_material.clone()),
        Transform::from_xyz(0.0, config::PLINKO_HEIGHT * 0.5 + 0.35, 0.0).with_scale(Vec3::new(
            config::PLINKO_WIDTH + 1.0,
            0.6,
            0.6,
        )),
        Visibility::Visible,
        Pickable::IGNORE,
    ));
    for pin in &sim.pins {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.plinko_pin_material.clone()),
            Transform::from_xyz(pin.x, pin.y, 0.0).with_scale(Vec3::splat(0.62)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
    for slot in 0..8 {
        let x = (slot as f32 - 3.5) * 2.0;
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.disc_accent_materials[slot].clone()),
            Transform::from_xyz(x, -config::PLINKO_HEIGHT * 0.5 - 1.0, 0.0)
                .with_scale(Vec3::new(1.9, 1.3, 0.9)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
    for index in 0..config::PLINKO_BALLS {
        commands.spawn((
            LightcycleSceneRoot,
            PlinkoBallEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.plinko_ball_material.clone()),
            Transform::from_xyz(0.0, config::PLINKO_HEIGHT * 0.5, 0.0).with_scale(Vec3::splat(1.1)),
            Visibility::Hidden,
            Pickable::IGNORE,
        ));
    }
}

/// Spawns a snake ring's power-ups and the bar sealing its exit.
///
/// The power-ups are a fixed pool keyed by index; the sim marks them eaten.
fn spawn_snake_field(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    arena: &Arena,
    snake: &SnakeSim,
) {
    for (index, food) in snake.food.iter().enumerate() {
        commands.spawn((
            LightcycleSceneRoot,
            SnakeFoodEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.snake_food_material.clone()),
            Transform::from_translation(
                config::ground_position(food.cell.0, food.cell.1) + Vec3::Y * 0.5,
            )
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_4))
            .with_scale(Vec3::splat(config::SNAKE_FOOD_SIZE)),
            if food.eaten {
                Visibility::Hidden
            } else {
                Visibility::Visible
            },
            Pickable::IGNORE,
        ));
    }

    // A solid bar in the gate opening until the exit opens. The physical lock is
    // the classification; this is what the rider sees.
    let Some(portal) = arena.parent_portal.as_ref() else {
        return;
    };
    let gate = portal.to;
    commands.spawn((
        LightcycleSceneRoot,
        SnakeGateLock,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.snake_lock_material.clone()),
        Transform::from_translation(config::ground_position(gate.0, gate.1) + Vec3::Y * 0.9)
            .with_scale(Vec3::new(
                config::GRID_SPACING,
                config::LIGHTCYCLE_WALL_HEIGHT * 0.9,
                config::GRID_SPACING,
            )),
        if snake.exit_open {
            Visibility::Hidden
        } else {
            Visibility::Visible
        },
        Pickable::IGNORE,
    ));
}

/// Spawns a platformer level: a backdrop, the platforms with neon lips, the
/// exit door and the runner.
fn spawn_platformer_level(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    language: SourceLanguage,
    level: &PlatformerSim,
) {
    let _ = meshes;
    let depth = config::PLATFORMER_DEPTH;
    let lip = assets.disc_accent_materials[disc_language_index(language)].clone();

    // A dark slab behind the level so the platforms read against something.
    let span = level.length + 60.0;
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.platform_material.clone()),
        Transform::from_translation(Vec3::new(level.length * 0.5, 8.0, -depth))
            .with_scale(Vec3::new(span, 46.0, 1.0)),
        Pickable::IGNORE,
    ));

    for platform in &level.platforms {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.platform_material.clone()),
            Transform::from_translation(Vec3::new(
                platform.x + platform.w * 0.5,
                platform.y - platform.h * 0.5,
                0.0,
            ))
            .with_scale(Vec3::new(platform.w, platform.h, depth)),
            Pickable::IGNORE,
        ));
        // The lip marks the surface the runner lands on.
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(lip.clone()),
            Transform::from_translation(Vec3::new(platform.x + platform.w * 0.5, platform.y, 0.0))
                .with_scale(Vec3::new(platform.w, 0.14, depth + 0.2)),
            Pickable::IGNORE,
        ));
    }

    let door = level.exit_box();
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.exit_material.clone()),
        Transform::from_translation(Vec3::new(door.x + door.w * 0.5, door.y - door.h * 0.5, 0.0))
            .with_scale(Vec3::new(door.w, door.h, depth * 0.4)),
        Pickable::IGNORE,
    ));

    commands.spawn((
        LightcycleSceneRoot,
        CharacterEntity,
        CharacterAnim::at(Vec3::new(level.runner.x, level.runner.y, 0.0)),
        Transform::from_translation(Vec3::new(level.runner.x, level.runner.y, 0.0)),
        Visibility::default(),
        Pickable::IGNORE,
        children![(
            WorldAssetRoot(assets.tron_scene.clone()),
            Transform::from_rotation(Quat::from_rotation_y(config::TRON_MODEL_YAW))
                .with_scale(Vec3::splat(config::TRON_MODEL_SCALE)),
        )],
    ));
}

/// Spawns a breaker court: the side and ceiling walls, the bricks, and the ball.
///
/// The bike itself is the paddle, so it is left to `update_cycle_transform`.
fn spawn_breaker_court(commands: &mut Commands, assets: &LightcycleAssets, level: &BreakerSim) {
    let (width, height) = level.court;
    let thickness = 0.8;
    let depth = config::PLATFORMER_DEPTH;
    // Side walls and ceiling.
    for (x, y, w, h) in [
        (
            -width * 0.5 - thickness * 0.5,
            height * 0.5,
            thickness,
            height,
        ),
        (
            width * 0.5 + thickness * 0.5,
            height * 0.5,
            thickness,
            height,
        ),
        (
            0.0,
            height + thickness * 0.5,
            width + thickness * 2.0,
            thickness,
        ),
    ] {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.court_material.clone()),
            Transform::from_translation(Vec3::new(x, y, -depth * 0.5))
                .with_scale(Vec3::new(w, h, depth)),
            Pickable::IGNORE,
        ));
    }

    for (index, brick) in level.bricks.iter().enumerate() {
        let (bx, by, bw, bh) = level.brick_box(brick);
        commands.spawn((
            LightcycleSceneRoot,
            BrickEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.brick_material.clone()),
            Transform::from_translation(Vec3::new(bx + bw * 0.5, by + bh * 0.5, 0.0))
                .with_scale(Vec3::new(bw, bh, depth * 0.6)),
            if brick.alive {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }

    commands.spawn((
        LightcycleSceneRoot,
        BallEntity,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.ball_material.clone()),
        Transform::from_translation(Vec3::new(level.ball.x, level.ball.y, 0.0))
            .with_scale(Vec3::splat(config::BREAKER_BALL_RADIUS * 2.0)),
        Pickable::IGNORE,
    ));
}

/// Spawns a stealth room: floor, cover, the door, the character and the patrols
/// with their vision cones.
fn spawn_stealth_room(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    room: &StealthSim,
) {
    let _ = meshes;
    let (half_w, half_h) = (
        config::STEALTH_WIDTH as f32 * 0.5,
        config::STEALTH_HEIGHT as f32 * 0.5,
    );
    let span = config::GRID_SPACING;
    let width = half_w * 2.0 * span;
    let height = half_h * 2.0 * span;

    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.stealth_floor_material.clone()),
        Transform::from_translation(Vec3::new(0.0, -0.1, 0.0)).with_scale(Vec3::new(
            width + span * 2.0,
            0.2,
            height + span * 2.0,
        )),
        Pickable::IGNORE,
    ));

    for cell in &room.cover {
        let position = config::ground_position(cell.0, cell.1);
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.stealth_wall_material.clone()),
            Transform::from_translation(position + Vec3::Y * config::STEALTH_WALL_HEIGHT * 0.5)
                .with_scale(Vec3::new(
                    span * 0.96,
                    config::STEALTH_WALL_HEIGHT,
                    span * 0.96,
                )),
            Pickable::IGNORE,
        ));
    }

    let exit = config::ground_position(room.exit.0, room.exit.1);
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.stealth_exit_material.clone()),
        Transform::from_translation(exit + Vec3::Y * 1.4).with_scale(Vec3::new(
            span * 0.9,
            2.8,
            span * 0.9,
        )),
        Pickable::IGNORE,
    ));

    let facing = |angle: f32| Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 - angle);
    let body = |assets: &LightcycleAssets, scale: f32| {
        children![(
            WorldAssetRoot(assets.tron_scene.clone()),
            Transform::from_rotation(Quat::from_rotation_y(config::TRON_MODEL_YAW))
                .with_scale(Vec3::splat(scale)),
        )]
    };

    let start = config::ground_position(room.character.0, room.character.1);
    commands.spawn((
        LightcycleSceneRoot,
        CharacterEntity,
        CharacterAnim::at(start),
        Transform::from_translation(start).with_rotation(facing(room.heading.angle())),
        Visibility::default(),
        Pickable::IGNORE,
        body(assets, config::STEALTH_CHARACTER_SCALE),
    ));

    for (index, guard) in room.guards.iter().enumerate() {
        let cell = guard.cell();
        let position = config::ground_position(cell.0, cell.1);
        commands.spawn((
            LightcycleSceneRoot,
            GuardEntity { index },
            Transform::from_translation(position)
                .with_rotation(facing(guard.patrol.heading().angle())),
            Visibility::default(),
            Pickable::IGNORE,
            body(
                assets,
                config::STEALTH_CHARACTER_SCALE * config::STEALTH_GUARD_SCALE,
            ),
        ));
        commands.spawn((
            LightcycleSceneRoot,
            GuardConeEntity { index, mesh: None },
            Mesh3d(assets.vision_cone.clone()),
            MeshMaterial3d(assets.stealth_cone_material.clone()),
            Transform::from_translation(position + Vec3::Y * 0.08)
                .with_rotation(facing(guard.vision_angle()))
                .with_scale(Vec3::splat(config::STEALTH_CONE_REACH)),
            Pickable::IGNORE,
        ));
    }
}

/// Spawns a river surfer course: the water ribbon that follows the sim's
/// centreline, the rocks, the boost gates and the finish gate. The bike itself
/// is the shared cycle, posed from the sim every frame.
fn spawn_surfer_course(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    surfer: &SurferSim,
) {
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(meshes.add(surfer_river_mesh(surfer))),
        MeshMaterial3d(assets.surfer_water_material.clone()),
        Pickable::IGNORE,
    ));

    for rock in &surfer.rocks {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.surfer_rock_material.clone()),
            Transform::from_translation(Vec3::new(
                rock.x,
                config::SURFER_ROCK_HEIGHT * 0.5,
                rock.z,
            ))
            .with_scale(Vec3::new(
                rock.radius * 2.0,
                config::SURFER_ROCK_HEIGHT,
                rock.radius * 2.0,
            )),
            Pickable::IGNORE,
        ));
    }

    for gate in &surfer.gates {
        spawn_surfer_gate(
            commands,
            assets,
            gate.x,
            gate.z,
            config::SURFER_GATE_SPAN,
            config::SURFER_GATE_HEIGHT,
            &assets.surfer_gate_material,
        );
    }

    let finish_x = surfer.centerline(surfer.length);
    spawn_surfer_gate(
        commands,
        assets,
        finish_x,
        surfer.length,
        surfer.width * 0.9,
        config::SURFER_FINISH_HEIGHT,
        &assets.surfer_finish_material,
    );
}

/// Two posts and a lintel framing a gate opening across the river.
fn spawn_surfer_gate(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    x: f32,
    z: f32,
    span: f32,
    height: f32,
    material: &Handle<StandardMaterial>,
) {
    let post = Vec3::new(0.18, height, 0.18);
    for side in [-1.0_f32, 1.0] {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(Vec3::new(x + side * span, height * 0.5, z))
                .with_scale(post),
            Pickable::IGNORE,
        ));
    }
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(Vec3::new(x, height, z)).with_scale(Vec3::new(
            span * 2.0 + 0.36,
            0.18,
            0.18,
        )),
        Pickable::IGNORE,
    ));
}

/// The water ribbon, tessellated along the sim's centreline so the visual
/// banks match the gameplay banks exactly.
fn surfer_river_mesh(surfer: &SurferSim) -> Mesh {
    let start = -config::SURFER_RIVER_MARGIN;
    let end = surfer.length + config::SURFER_RIVER_MARGIN;
    let steps = ((end - start) / config::SURFER_RIVER_SAMPLE).ceil() as usize;
    let mut positions = Vec::with_capacity((steps + 1) * 2);
    let mut normals = Vec::with_capacity((steps + 1) * 2);
    for step in 0..=steps {
        let z = start + (end - start) * step as f32 / steps as f32;
        let center = surfer.centerline(z);
        positions.push([center - surfer.width, 0.0, z]);
        positions.push([center + surfer.width, 0.0, z]);
        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
    }
    let mut indices = Vec::with_capacity(steps * 6);
    for step in 0..steps as u32 {
        let a = step * 2;
        let b = step * 2 + 1;
        let c = (step + 1) * 2;
        let d = (step + 1) * 2 + 1;
        indices.extend_from_slice(&[a, c, b, b, c, d]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

/// A floor fan: a centre point, then one rim point per ray, each reaching as far
/// as that ray can see. Radii are in the mesh's own units.
fn cone_positions(half_angle: f32, radii: &[f32]) -> Vec<[f32; 3]> {
    let segments = radii.len().saturating_sub(1).max(1);
    let mut positions = Vec::with_capacity(radii.len() + 1);
    positions.push([0.0, 0.0, 0.0]);
    for (step, reach) in radii.iter().enumerate() {
        let t = step as f32 / segments as f32;
        let angle = -half_angle + t * half_angle * 2.0;
        positions.push([angle.sin() * reach, 0.0, angle.cos() * reach]);
    }
    positions
}

/// Moves an existing cone's rim out to `radii`, leaving its topology alone.
fn refit_cone(mesh: &mut Mesh, half_angle: f32, radii: &[f32]) {
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, cone_positions(half_angle, radii));
}

/// A cone whose rays reach `radii`: the shape of what a guard can actually see.
fn cone_mesh(half_angle: f32, radii: &[f32]) -> Mesh {
    let positions = cone_positions(half_angle, radii);
    let segments = radii.len().saturating_sub(1).max(1);
    let mut indices = Vec::new();
    for step in 0..segments as u32 {
        indices.extend_from_slice(&[0, step + 1, step + 2]);
    }
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

/// Unit-length flat cone with the stealth half-angle, opening along local `+Z`.
///
/// Used as a guard's cone until its real shape has been measured, so the first
/// frame is not a hole in the floor.
fn vision_cone_mesh(half_angle: f32, segments: usize) -> Mesh {
    let radii = vec![1.0; segments + 1];
    cone_mesh(half_angle, &radii)
}

/// Cuts each guard's cone to what it can actually see, rebuilding the mesh on the
/// first frame and refitting it after that.
///
/// Detection samples line of sight, so a cone that ignores cover tells the player
/// a lie about where they are safe.
fn fit_guard_cones(
    state: Res<LightcycleState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cones: Query<(&mut GuardConeEntity, &mut Transform, &mut Mesh3d)>,
) {
    let Some(room) = state.run.as_ref().and_then(|run| run.source_stealth()) else {
        return;
    };
    for (mut cone, mut transform, mut mesh) in &mut cones {
        let radii = room.vision_radii(cone.index, config::STEALTH_CONE_SEGMENTS);
        match cone.mesh.clone() {
            Some(handle) => {
                if let Some(mut geometry) = meshes.get_mut(&handle) {
                    refit_cone(&mut geometry, config::STEALTH_VISION_HALF_ANGLE, &radii);
                }
            }
            None => {
                // The radii are in cells, so the scale drops to one cell from the
                // fixed reach the placeholder fan was drawn at.
                let handle = meshes.add(cone_mesh(config::STEALTH_VISION_HALF_ANGLE, &radii));
                mesh.0 = handle.clone();
                transform.scale = Vec3::splat(config::GRID_SPACING);
                cone.mesh = Some(handle);
            }
        }
    }
}

/// Merges same-sized cuboids at `cells` and spawns them as one batched entity.
fn spawn_disc_cube_layer(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    cells: &[(i32, i32)],
    material: Handle<StandardMaterial>,
    height: f32,
    footprint: f32,
) {
    for chunk in cells.chunks(config::MESH_CHUNK_SIZE) {
        let mut chunk = chunk.iter();
        let Some(&first) = chunk.next() else {
            continue;
        };
        let mut mesh = disc_cube(first, height, footprint);
        for &cell in chunk {
            mesh.merge(&disc_cube(cell, height, footprint))
                .expect("disc cuboid meshes must be merge-compatible");
        }
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material.clone()),
            Pickable::IGNORE,
        ));
    }
}

fn disc_cube(cell: (i32, i32), height: f32, footprint: f32) -> Mesh {
    Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(
            config::ground_position(cell.0, cell.1) + Vec3::Y * (height * 0.5),
        )
        .with_scale(Vec3::new(footprint, height, footprint)),
    )
}

/// Posts and lintel framing the corridor mouth, so the close gate reads as a
/// door in the ring wall.
fn spawn_disc_gate(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    arena: &Arena,
    layout: &DiscLayout,
    accent: &Handle<StandardMaterial>,
) {
    let Some(portal) = arena.parent_portal else {
        return;
    };
    let center = layout.center;
    let span_of = |cell: (i32, i32)| (cell.0 - center.0).abs() + (cell.1 - center.1).abs();
    let outer = if span_of(portal.from) > span_of(portal.to) {
        portal.from
    } else {
        portal.to
    };
    let base = config::ground_position(outer.0, outer.1);
    // The corridor runs along the axis from the center to the outer cell, so
    // the door opening is perpendicular to it.
    let opening_axis = if (outer.0 - center.0) == 0 {
        Vec3::X
    } else {
        Vec3::Z
    };
    let post = Vec3::new(0.24, 2.2, 0.24);
    for side in [-1.0_f32, 1.0] {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(accent.clone()),
            Transform::from_translation(base + opening_axis * (side * 1.05) + Vec3::Y * 1.1)
                .with_scale(post),
            Pickable::IGNORE,
        ));
    }
    let lintel_scale = if opening_axis == Vec3::X {
        Vec3::new(2.5, 0.22, 0.24)
    } else {
        Vec3::new(0.24, 0.22, 2.5)
    };
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(accent.clone()),
        Transform::from_translation(base + Vec3::Y * 2.25).with_scale(lintel_scale),
        Pickable::IGNORE,
    ));
}

fn spawn_disc_focus_marker(commands: &mut Commands, assets: &LightcycleAssets, run: &ActiveRun) {
    let RunEnvironment::Source { language, .. } = &run.environment else {
        return;
    };
    let pose = cycle_cell_pose(&run.sim);
    commands.spawn((
        LightcycleSceneRoot,
        DocumentFocusMarker,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.disc_accent_materials[disc_language_index(*language)].clone()),
        Transform::from_translation(pose_world_position(&pose) + Vec3::Y * 0.08)
            .with_scale(Vec3::new(1.4, 0.08, 1.4)),
        Pickable::IGNORE,
    ));
}

/// Where the on-foot character should be and which way it faces.
pub(crate) struct CharacterPose {
    /// Ground position the figure is walking toward.
    target: Vec3,
    yaw: f32,
    /// True when the sim moves in grid steps that need easing out.
    smooth: bool,
}

/// The unit vector a heading points along, in the `(x, z)` the world is built on.
fn unit_of(heading: Heading) -> (f32, f32) {
    let angle = heading_angle(heading);
    (angle.cos(), angle.sin())
}

/// Builds the walk graph for the character's player and attaches it. The glTF
/// loader makes the `AnimationPlayer` but leaves the graph to us.
fn prepare_character_walk(
    mut commands: Commands,
    assets: Res<LightcycleAssets>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    players: Query<Entity, (With<AnimationPlayer>, Without<CharacterWalk>)>,
) {
    if players.is_empty() {
        return;
    }
    let Some(clip) = gltfs
        .get(&assets.tron_gltf)
        .and_then(|gltf| gltf.named_animations.get(config::WALK_CLIP))
        .cloned()
    else {
        return;
    };
    for entity in &players {
        let (graph, nodes) = AnimationGraph::from_clips([clip.clone()]);
        if let Some(index) = nodes.first().copied() {
            commands.entity(entity).insert((
                AnimationGraphHandle(graphs.add(graph)),
                CharacterWalk(index),
            ));
        }
    }
}

/// Plays the walk while the character is moving and rewinds it when it stops,
/// so it never stands mid-stride. The rate is matched to ground speed: a clip
/// run at the wrong speed makes the feet skate.
fn drive_character_walk(
    state: Res<LightcycleState>,
    assets: Res<LightcycleAssets>,
    gltfs: Res<Assets<Gltf>>,
    mut reported: Local<bool>,
    mut character: Query<(&mut AnimationPlayer, &CharacterWalk), With<CharacterModel>>,
    mut guards: Query<(&mut AnimationPlayer, &CharacterWalk), Without<CharacterModel>>,
) {
    let Some(run) = state.run.as_ref() else {
        return;
    };
    // Say once, when the character's player first exists, what the animation
    // plumbing actually found. If the character ever walks without animating,
    // this line is the first thing to look at.
    if !*reported && !(character.is_empty() && guards.is_empty()) {
        *reported = true;
        let clips: Vec<Box<str>> = gltfs
            .get(&assets.tron_gltf)
            .map(|gltf| gltf.named_animations.keys().cloned().collect())
            .unwrap_or_default();
        info!(
            "walk animation: {} character player(s), {} other, {:?} clip, asset carries {clips:?}",
            character.iter().len(),
            guards.iter().len(),
            config::WALK_CLIP,
        );
    }
    // The patrol step rate, and the character's own ground speed in world units
    // per second. The guards walk continuously, so their clip must not stop
    // just because the player is waiting for them to pass.
    let step_speed = config::STEALTH_WALK_SPEED;
    let character_speed = if let Some(room) = run.source_stealth() {
        if room.walking { step_speed } else { 0.0 }
    } else if let Some(level) = run.source_platformer() {
        level.runner.vx.abs()
    } else {
        0.0
    };
    let guard_speed = if run.source_stealth().is_some() {
        step_speed
    } else {
        0.0
    };

    // The character's own player is tagged; every other player in the scene
    // belongs to a guard, which keeps walking while the player waits.
    walk_players(character.iter_mut(), character_speed);
    walk_players(guards.iter_mut(), guard_speed);
}

/// Tags every entity under an on-foot character, however deep.
fn tag_character_model(
    mut commands: Commands,
    characters: Query<Entity, With<CharacterEntity>>,
    children: Query<&Children>,
    tagged: Query<(), With<CharacterModel>>,
) {
    for character in &characters {
        let mut stack = vec![character];
        while let Some(entity) = stack.pop() {
            if tagged.get(entity).is_err() {
                commands.entity(entity).insert(CharacterModel);
            }
            if let Ok(kids) = children.get(entity) {
                stack.extend(kids.iter());
            }
        }
    }
}

/// Drives one set of players at a ground speed in world units per second. Zero
/// stops them, which is what standing still has to look like.
fn walk_players<'a>(
    players: impl Iterator<Item = (Mut<'a, AnimationPlayer>, &'a CharacterWalk)>,
    speed: f32,
) {
    for (mut player, walk) in players {
        if speed <= 0.05 {
            if player.is_playing_animation(walk.0) {
                player.stop(walk.0);
            }
            continue;
        }
        let rate = (speed / config::WALK_CLIP_GROUND).clamp(0.3, 2.5);
        let active = player.play(walk.0);
        // Bevy's default repeat mode is `Never`: the clip plays once and then
        // parks on its last frame, which reads as a character sliding along
        // frozen mid-stride. Loop it, and rewind it if a previous pass already
        // completed, since `play` never restarts an active animation.
        if active.is_finished() {
            active.replay();
        }
        active.set_speed(rate).repeat();
    }
}

fn disc_entity_position(cell: (i32, i32)) -> Vec3 {
    config::ground_position(cell.0, cell.1) + Vec3::Y * 0.35
}

/// Spin and bob the waiting pickups so they read as collectible.
fn animate_disc_pickups(time: Res<Time>, mut pickups: Query<(&DiscPickupEntity, &mut Transform)>) {
    let elapsed = time.elapsed_secs();
    for (pickup, mut transform) in &mut pickups {
        let bob = (elapsed * 2.2 + pickup.phase * std::f32::consts::TAU).sin() * 0.12;
        transform.translation.y = 0.45 + bob;
        transform.rotate_y(0.03);
    }
}

fn city_cap_mesh(structure: &CityStructure) -> Mesh {
    let body_height = city_body_height(structure);
    let cap_height = config::LIGHTCYCLE_CITY_CAP_HEIGHT;
    let mut scale = city_body_scale(structure, body_height);
    scale.y = cap_height;
    scale.x += 0.08;
    scale.z += 0.08;
    let position = config::ground_position(structure.cell.0, structure.cell.1)
        + Vec3::Y * (config::LIGHTCYCLE_CITY_FOUNDATION_HEIGHT + body_height + cap_height * 0.5);
    Mesh::from(Cuboid::default())
        .transformed_by(Transform::from_translation(position).with_scale(scale))
}

/// Glowing skirt around a structure's footprint. It is wider than the
/// foundation, so the visible part is a neon border tracing where the wall
/// stops and the floor starts.
fn city_base_trim_mesh(structure: &CityStructure) -> Mesh {
    let height = config::LIGHTCYCLE_CITY_BASE_TRIM_HEIGHT;
    let size = config::LIGHTCYCLE_CITY_STRUCTURE_SIZE + config::LIGHTCYCLE_CITY_BASE_TRIM_OVERHANG;
    let position =
        config::ground_position(structure.cell.0, structure.cell.1) + Vec3::Y * (height * 0.5);
    Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(position).with_scale(Vec3::new(size, height, size)),
    )
}

fn spawn_document_page(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    arena: &Arena,
    layout: &DocumentLayout,
) {
    spawn_document_rules(commands, assets, meshes, arena);
    spawn_document_walls(commands, assets, meshes, arena);
    spawn_document_arches(commands, assets, meshes, layout);
    spawn_document_glyphs(commands, assets, meshes, layout);
}

fn spawn_document_rules(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    arena: &Arena,
) {
    let Some(mesh) = document_rule_mesh(arena) else {
        return;
    };
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(assets.document_rule_material.clone()),
        Pickable::IGNORE,
    ));
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(meshes.add(document_margin_mesh(arena))),
        MeshMaterial3d(assets.document_margin_material.clone()),
        Pickable::IGNORE,
    ));
}

fn document_rule_mesh(arena: &Arena) -> Option<Mesh> {
    let spacing = config::GRID_SPACING;
    let mut merged: Option<Mesh> = None;
    let mut push = |mesh: Mesh| {
        if let Some(existing) = &mut merged {
            existing
                .merge(&mesh)
                .expect("document rule meshes must be merge-compatible");
        } else {
            merged = Some(mesh);
        }
    };

    for z in arena.min.1..=arena.max.1 {
        let center = Vec3::new(
            (arena.min.0 + arena.max.0) as f32 * spacing * 0.5,
            0.02,
            z as f32 * spacing,
        );
        let width = (arena.max.0 - arena.min.0 + 1) as f32 * spacing;
        push(Mesh::from(Cuboid::default()).transformed_by(
            Transform::from_translation(center).with_scale(Vec3::new(width, 0.03, 0.06)),
        ));
    }
    merged
}

fn document_margin_mesh(arena: &Arena) -> Mesh {
    let spacing = config::GRID_SPACING;
    let margin_x = (arena.min.0 as f32 - 0.15) * spacing;
    let center = Vec3::new(
        margin_x,
        0.03,
        (arena.min.1 + arena.max.1) as f32 * spacing * 0.5,
    );
    let depth = (arena.max.1 - arena.min.1 + 1) as f32 * spacing;
    Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(center).with_scale(Vec3::new(0.08, 0.04, depth)),
    )
}

fn spawn_document_walls(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    arena: &Arena,
) {
    let cells: Vec<_> = arena.street_walls.iter().copied().collect();
    for chunk in cells.chunks(config::MESH_CHUNK_SIZE) {
        let mut chunk = chunk.iter();
        let Some(&first) = chunk.next() else {
            continue;
        };
        let mut mesh = document_wall_mesh(first);
        for &cell in chunk {
            mesh.merge(&document_wall_mesh(cell))
                .expect("document wall meshes must be merge-compatible");
        }
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(assets.document_ink_material.clone()),
            Pickable::IGNORE,
        ));
    }
}

fn document_wall_mesh(cell: (i32, i32)) -> Mesh {
    let height = 1.35;
    Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(config::world_position(cell.0, cell.1, height))
            .with_scale(Vec3::new(1.7, height, 0.55)),
    )
}

fn spawn_document_arches(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    layout: &DocumentLayout,
) {
    let headings: Vec<_> = layout
        .blocks
        .iter()
        .filter(|block| matches!(block.kind, crate::document::DocBlockKind::Heading(_)))
        .collect();
    for chunk in headings.chunks(config::MESH_CHUNK_SIZE) {
        let mut chunk = chunk.iter();
        let Some(first) = chunk.next() else {
            continue;
        };
        let mut mesh = document_arch_mesh(first);
        for block in chunk {
            mesh.merge(&document_arch_mesh(block))
                .expect("document arch meshes must be merge-compatible");
        }
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(assets.document_heading_material.clone()),
            Pickable::IGNORE,
        ));
    }
}

fn document_arch_mesh(block: &crate::document::PlacedBlock) -> Mesh {
    let (x, z) = block.landmark;
    let origin = config::ground_position(x, z);
    let (span, depth) = if block.along_x {
        (2.4, 0.28)
    } else {
        (0.28, 2.4)
    };
    let left = Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(origin + Vec3::new(-span * 0.35, 1.1, -depth * 0.35))
            .with_scale(Vec3::new(0.22, 2.2, 0.22)),
    );
    let mut mesh = left;
    mesh.merge(
        &Mesh::from(Cuboid::default()).transformed_by(
            Transform::from_translation(origin + Vec3::new(span * 0.35, 1.1, depth * 0.35))
                .with_scale(Vec3::new(0.22, 2.2, 0.22)),
        ),
    )
    .expect("arch posts must merge");
    mesh.merge(&Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(origin + Vec3::Y * 2.25).with_scale(Vec3::new(
            span,
            0.22,
            depth.max(0.4),
        )),
    ))
    .expect("arch lintel must merge");
    mesh
}

fn spawn_document_glyphs(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    layout: &DocumentLayout,
) {
    let mut remaining = config::DOCUMENT_MAX_GLYPHS;
    let mut heading_mesh: Option<Mesh> = None;
    let mut plaque_mesh: Option<Mesh> = None;
    for block in &layout.blocks {
        if remaining == 0 {
            break;
        }
        let (mesh, used) = document_glyph_line_mesh(block, remaining);
        remaining = remaining.saturating_sub(used);
        if used == 0 {
            continue;
        }
        let target = if matches!(block.kind, crate::document::DocBlockKind::Heading(_)) {
            &mut heading_mesh
        } else {
            &mut plaque_mesh
        };
        if let Some(existing) = target {
            existing
                .merge(&mesh)
                .expect("glyph meshes must be merge-compatible");
        } else {
            *target = Some(mesh);
        }
    }

    if let Some(mesh) = heading_mesh {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(assets.document_heading_material.clone()),
            Pickable::IGNORE,
        ));
    }
    if let Some(mesh) = plaque_mesh {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(assets.document_ink_material.clone()),
            Pickable::IGNORE,
        ));
    }
}

fn document_glyph_line_mesh(
    block: &crate::document::PlacedBlock,
    remaining: usize,
) -> (Mesh, usize) {
    let origin = config::ground_position(block.landmark.0, block.landmark.1);
    let heading = matches!(block.kind, crate::document::DocBlockKind::Heading(_));
    let pixel = if heading { 0.09 } else { 0.055 };
    let height = if heading { 2.55 } else { 0.85 };
    let advance = document_line_advance(block.along_x);
    let max_chars = remaining.min(if heading {
        config::DOCUMENT_HEADING_GLYPHS
    } else {
        config::DOCUMENT_PARAGRAPH_GLYPHS
    });
    let chars: Vec<char> = block.preview.chars().take(max_chars).collect();
    let mut mesh: Option<Mesh> = None;
    let mut used = 0usize;
    for (index, ch) in chars.iter().enumerate() {
        let glyph = glyph_pixels(*ch);
        used += 1;
        let offset = glyph_char_offset(advance, index, chars.len(), pixel);
        for (row, row_bits) in glyph.iter().enumerate() {
            for col in 0..8 {
                if row_bits & (1 << col) == 0 {
                    continue;
                }
                let local = glyph_pixel_offset(advance, col, row, pixel);
                let cube = Mesh::from(Cuboid::default()).transformed_by(
                    Transform::from_translation(origin + Vec3::Y * height + offset + local)
                        .with_scale(Vec3::splat(pixel * 0.85)),
                );
                if let Some(existing) = &mut mesh {
                    existing
                        .merge(&cube)
                        .expect("glyph pixels must be merge-compatible");
                } else {
                    mesh = Some(cube);
                }
            }
        }
    }
    (
        mesh.unwrap_or_else(|| collapsed_document_glyph(origin)),
        used,
    )
}

/// Direction a block's text reads in, which is also the axis its glyph columns
/// run along.
///
/// A paragraph walls off one side of its spine cells, so the reader always
/// arrives from the other side: `+Z` for a row laid along X, `+X` for one laid
/// along Z. Screen right for those two viewpoints is `+X` and `-Z`, and text has
/// to read toward screen right.
fn document_line_advance(along_x: bool) -> Vec3 {
    if along_x { Vec3::X } else { Vec3::NEG_Z }
}

/// Offset of one character's origin from the middle of its line.
fn glyph_char_offset(advance: Vec3, index: usize, count: usize, pixel: f32) -> Vec3 {
    advance * ((index as f32 - (count as f32 - 1.0) * 0.5) * pixel * 9.0)
}

/// Offset of one glyph pixel from its own character's origin.
///
/// Columns run along the same `advance` the characters are placed along, so a
/// letterform cannot end up mirrored against the order of the line it sits in.
/// font8x8 packs each row least-significant bit first, making column 0 the
/// letter's leftmost pixel, so it belongs at the near end of `advance`. Row 0 is
/// the top of the glyph and belongs at the top of the line.
fn glyph_pixel_offset(advance: Vec3, col: usize, row: usize, pixel: f32) -> Vec3 {
    advance * (col as f32 * pixel) + Vec3::Y * ((7 - row) as f32 * pixel)
}

fn collapsed_document_glyph(origin: Vec3) -> Mesh {
    Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(origin + Vec3::Y * 0.2).with_scale(Vec3::splat(0.08)),
    )
}

fn glyph_pixels(character: char) -> [u8; 8] {
    if (character as u32) < 128 {
        font8x8::legacy::BASIC_LEGACY[character as usize]
    } else {
        // Unsupported glyphs keep a diamond placeholder; the folio panel shows
        // the original Unicode.
        [
            0b00011000, 0b00111100, 0b01111110, 0b11111111, 0b01111110, 0b00111100, 0b00011000,
            0b00000000,
        ]
    }
}

fn spawn_document_focus_marker(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    run: &ActiveRun,
) {
    let pose = cycle_cell_pose(&run.sim);
    commands.spawn((
        LightcycleSceneRoot,
        DocumentFocusMarker,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.document_focus_material.clone()),
        Transform::from_translation(pose_world_position(&pose) + Vec3::Y * 0.08)
            .with_scale(Vec3::new(1.4, 0.08, 1.4)),
        Pickable::IGNORE,
    ));
}

/// One flat quad of ground marking: where it sits, and how far it reaches on
/// each ground axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MarkingQuad {
    center: Vec3,
    half_x: f32,
    half_z: f32,
}

/// Splits a rail's extent around an optional gap, dropping segments too short to
/// be worth drawing.
fn rail_segments(min: f32, max: f32, gap: Option<(f32, f32)>) -> Vec<(f32, f32)> {
    let Some((gap_min, gap_max)) = gap else {
        return vec![(min, max)];
    };

    [(min, gap_min.min(max)), (gap_max.max(min), max)]
        .into_iter()
        .filter(|(start, end)| end - start > 0.01)
        .collect()
}

/// World-space extent of the gate along its wall, covering exactly the cells the
/// simulation accepts.
fn gate_world_span(portal: &ParentPortal) -> (f32, f32) {
    let spacing = config::GRID_SPACING;
    let (from, _) = portal.along_span();
    let start = (from as f32 - 0.5) * spacing;
    (start, start + portal.width_cells() as f32 * spacing)
}

/// Builds the animated parent gate: two posts, a lintel, and light bars that
/// sweep up through the opening.
///
/// Everything is parented to a root whose rotation puts the wall's axis on local
/// +X, so the pieces below are laid out once instead of per wall.
fn spawn_parent_gate(commands: &mut Commands, assets: &LightcycleAssets, arena: &Arena) {
    let Some(portal) = arena.parent_portal else {
        return;
    };

    let height = config::LIGHTCYCLE_PORTAL_HEIGHT;
    let frame = config::LIGHTCYCLE_PORTAL_FRAME_THICKNESS;
    let depth = config::LIGHTCYCLE_WALL_THICKNESS * 3.0;
    let (span_min, span_max) = gate_world_span(&portal);
    let opening = span_max - span_min;
    let center = (span_min + span_max) * 0.5;
    let plane = wall_plane(arena, portal.wall);

    let (translation, rotation) = match portal.wall {
        Wall::NegZ | Wall::PosZ => (Vec3::new(center, 0.0, plane), Quat::IDENTITY),
        Wall::NegX | Wall::PosX => (
            Vec3::new(plane, 0.0, center),
            Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        ),
    };

    let frame_material = if arena.kind == ArenaKind::Document {
        assets.document_folio_material.clone()
    } else {
        assets.portal_material.clone()
    };
    let bar_material = if arena.kind == ArenaKind::Document {
        assets.document_focus_material.clone()
    } else {
        assets.portal_bar_material.clone()
    };

    let half = opening * 0.5;
    let mut gate = commands.spawn((
        LightcycleSceneRoot,
        Transform::from_translation(translation).with_rotation(rotation),
        Visibility::default(),
    ));

    gate.with_children(|frames| {
        let mut piece = |translation: Vec3, scale: Vec3| {
            frames.spawn((
                GateFrame,
                Mesh3d(assets.unit_cube.clone()),
                MeshMaterial3d(frame_material.clone()),
                Transform::from_translation(translation).with_scale(scale),
                Pickable::IGNORE,
            ));
        };

        // Posts on the cell boundaries the gate starts and ends at.
        piece(
            Vec3::new(-half, height * 0.5, 0.0),
            Vec3::new(frame, height, depth),
        );
        piece(
            Vec3::new(half, height * 0.5, 0.0),
            Vec3::new(frame, height, depth),
        );
        // Lintel spanning them.
        piece(
            Vec3::new(0.0, height, 0.0),
            Vec3::new(opening + frame, frame, depth),
        );

        let bars = config::LIGHTCYCLE_PORTAL_BAR_COUNT;
        for index in 0..bars {
            frames.spawn((
                GateScanBar {
                    offset: index as f32 / bars as f32,
                    travel: height,
                },
                Mesh3d(assets.unit_cube.clone()),
                MeshMaterial3d(bar_material.clone()),
                Transform::from_scale(Vec3::new(
                    opening - frame,
                    config::LIGHTCYCLE_PORTAL_BAR_HEIGHT,
                    depth * 0.5,
                )),
                Pickable::IGNORE,
            ));
        }
    });
}

/// Brightness of the gate frame at `elapsed`, from 0 at the pulse's trough to 1
/// at its peak.
fn gate_pulse(elapsed: f32) -> f32 {
    0.5 + 0.5 * (elapsed * config::LIGHTCYCLE_PORTAL_PULSE_SPEED).sin()
}

/// Height a bar has swept to within its opening, wrapping back to the ground
/// once it reaches the lintel.
fn gate_bar_height(bar: &GateScanBar, elapsed: f32) -> f32 {
    (bar.offset + elapsed * config::LIGHTCYCLE_PORTAL_BAR_SPEED).fract() * bar.travel
}

/// Pulses the gate frame and sweeps its light bars upward, so a gate reads as
/// live and is easy to pick out from the surrounding wall.
fn animate_parent_gate(
    time: Res<Time>,
    assets: Res<LightcycleAssets>,
    state: Res<LightcycleState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut bars: Query<(&GateScanBar, &mut Transform)>,
) {
    let elapsed = time.elapsed_secs();
    let document = state.run.as_ref().is_some_and(ActiveRun::is_document);
    let (handle, dim, bright) = if document {
        (
            &assets.document_folio_material,
            config::DOCUMENT_FOLIO_DIM_COLOR,
            config::DOCUMENT_FOLIO_COLOR,
        )
    } else {
        (
            &assets.portal_material,
            config::LIGHTCYCLE_PORTAL_DIM_COLOR,
            config::LIGHTCYCLE_PORTAL_COLOR,
        )
    };

    if let Some(mut material) = materials.get_mut(handle) {
        material.base_color = dim.mix(&bright, gate_pulse(elapsed));
    }

    for (bar, mut transform) in &mut bars {
        transform.translation.y = gate_bar_height(bar, elapsed);
    }
}

fn animate_city_beacons(time: Res<Time>, mut beacons: Query<(&CityBeacon, &mut Transform)>) {
    let elapsed = time.elapsed_secs();
    for (beacon, mut transform) in &mut beacons {
        let wave = (elapsed * 2.4 + beacon.phase * std::f32::consts::TAU).sin();
        transform.translation.y = beacon.base_height + wave * 0.18;
        transform.scale = Vec3::splat(0.2 + (wave * 0.5 + 0.5) * 0.08);
        transform.rotate_y(0.018);
    }
}

/// Bobs the call-stack frames, so a directory's stack reads as live hardware
/// rather than a static prop.
fn animate_stack_frames(
    time: Res<Time>,
    pause: Res<PauseState>,
    mut motion: ResMut<StackMotion>,
    mut frames: Query<(&StackFrameEntity, &mut Transform)>,
) {
    if pause.paused {
        return;
    }
    // The plunge clock only runs while there is a stack to move.
    let plunge = if frames.is_empty() {
        None
    } else {
        motion.advance(time.delta_secs())
    };
    let elapsed = time.elapsed_secs();
    for (frame, mut transform) in &mut frames {
        let level = frame.level;
        // A frame's rest height mirrored below the floor is the same distance
        // down as it is up, so the dive is twice the height it rests at.
        let dive = plunge.map_or(0.0, |progress| stack_plunge(level, progress));
        let sink = -2.0 * frame.base.y * dive;
        let (glide_x, glide_z) = stack_frame_glide(level, elapsed);
        let (rock_x, rock_z) = stack_frame_rock(level, elapsed);
        transform.translation =
            frame.base + Vec3::new(glide_x, sink + stack_frame_hover(level, elapsed), glide_z);
        transform.rotation = Quat::from_euler(EulerRot::XZY, rock_x, 0.0, rock_z);
    }
}

/// How deep into its plunge a frame is: `0.0` at its resting height through
/// `1.0` at the same distance mirrored below the floor, for `progress` through
/// the whole event.
///
/// Levels lag the one above them, so the stack cascades: the top frame leads the
/// dive and is first back, and the deepest frame arrives last.
pub fn stack_plunge(level: usize, progress: f32) -> f32 {
    let lag = level as f32 * config::STACK_PLUNGE_STAGGER;
    // The stagger is spread across the event, so the last level still finishes
    // exactly as the event does.
    let span = 1.0 - config::STACK_PLUNGE_STAGGER * (config::STACK_FRAME_MAX - 1) as f32;
    let local = ((progress - lag) / span.max(0.1)).clamp(0.0, 1.0);
    let hold = config::STACK_PLUNGE_HOLD;
    let leg = (1.0 - hold) * 0.5;
    // `smoothstep` eases each leg, so the stack accelerates away from its rest
    // height and settles back into it instead of snapping.
    if local <= leg {
        smoothstep(local / leg)
    } else if local >= leg + hold {
        smoothstep((1.0 - local) / leg)
    } else {
        1.0
    }
}

/// How far a frame floats above its resting height at `elapsed`.
pub fn stack_frame_hover(level: usize, elapsed: f32) -> f32 {
    let phase =
        elapsed * config::STACK_FRAME_HOVER_SPEED + level as f32 * config::STACK_FRAME_PHASE_STEP;
    phase.sin() * config::STACK_FRAME_HOVER
}

/// The frame's slow drift off centre, on X and Z. The two axes run at different
/// rates, so it wanders rather than tracing the same circle forever.
pub fn stack_frame_glide(level: usize, elapsed: f32) -> (f32, f32) {
    let phase =
        elapsed * config::STACK_FRAME_GLIDE_SPEED + level as f32 * config::STACK_FRAME_PHASE_STEP;
    (
        phase.sin() * config::STACK_FRAME_GLIDE,
        (phase * 0.77 + 1.3).cos() * config::STACK_FRAME_GLIDE,
    )
}

/// The frame's tilt about X and Z at `elapsed`, in radians. The two axes run at
/// different rates, so a frame never repeats the same attitude twice in a row.
pub fn stack_frame_rock(level: usize, elapsed: f32) -> (f32, f32) {
    let offset = level as f32 * config::STACK_FRAME_PHASE_STEP;
    let x = (elapsed * config::STACK_FRAME_ROCK_SPEED + offset).sin() * config::STACK_FRAME_ROCK;
    let z = (elapsed * config::STACK_FRAME_ROCK_SPEED * 0.63 + offset * 1.7).cos()
        * config::STACK_FRAME_ROCK;
    (x, z)
}

#[allow(clippy::too_many_arguments)]
fn reset_on_directory_loaded(
    mut loaded: MessageReader<DirectoryLoaded>,
    mode: Res<InteractionMode>,
    mut state: ResMut<LightcycleState>,
    mut cache: ResMut<CacheState>,
    mut flood: ResMut<FloodState>,
    mut history: ResMut<HistoryState>,
    assets: Res<LightcycleAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mut effects: MessageWriter<MusicSfx>,
    old_lightcycle_entities: SceneEntities,
) {
    if *mode != InteractionMode::Lightcycle {
        return;
    }

    for event in loaded.read() {
        despawn_lightcycle_entities(&mut commands, &old_lightcycle_entities);

        let run = build_active_run(&event.path, event.contents.nodes.clone());
        spawn_run_entities(&mut commands, &assets, &mut meshes, &run);
        // Opening a document is never the first ride, so its room is armed.
        state.grace_room = false;
        decorate_directory_run(
            &mut commands,
            &assets,
            &mut meshes,
            &mut state,
            &mut flood,
            &event.path,
            &run,
        );
        history.commit(&event.path);

        // A directory already opened is a cache hit: a short speed surge for
        // the rest of the run, plus the disk-head seek on the hop in.
        let hit = !cache.visited.insert(event.path.clone());
        state.cache_boost = if hit {
            config::CACHE_BOOST_SECONDS
        } else {
            0.0
        };
        state.gc_timer = config::GC_INTERVAL_SECONDS;
        state.gc_pause = 0.0;
        state.gc_sweep = 0.0;
        effects.write(MusicSfx::Seek);

        state.clock = 0.0;
        state.crash_fx = None;
        state.entry_fx = None;
        state.restore_directory = false;
        state.run = Some(run);
    }
}

/// Stacks a translucent plate above the arena for each level of the directory
/// path (the call stack), lays the hex-dump highway along its roads, rings a
/// quarantined vault with pylons, and spawns the memory-flood wall.
fn decorate_directory_run(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    state: &mut LightcycleState,
    flood: &mut FloodState,
    path: &Path,
    run: &ActiveRun,
) {
    // Whether this room hunts the rider, or is only dressed. The caller decides,
    // because only it knows whether this ride is the first one.
    let armed = !state.grace_room;
    let span = config::GRID_SPACING;
    let center_x = (run.arena.min.0 + run.arena.max.0) as f32 * 0.5 * span;
    let center_z = (run.arena.min.1 + run.arena.max.1) as f32 * 0.5 * span;

    // Call stack: one open frame per path level, floating over the arena's
    // edge. The border is chunky enough to read as structure, and the middle is
    // left open so the road below stays visible.
    let depth = path.components().count().min(config::STACK_FRAME_MAX);
    if depth > 0 {
        let margin = config::STACK_FRAME_MARGIN * span;
        let frame = meshes.add(stack_frame_mesh(
            (run.arena.max.0 - run.arena.min.0 + 1) as f32 * span + margin * 2.0,
            (run.arena.max.1 - run.arena.min.1 + 1) as f32 * span + margin * 2.0,
        ));
        for level in 0..depth {
            let base = Vec3::new(
                center_x,
                config::STACK_FRAME_BASE_Y + level as f32 * config::STACK_FRAME_SPACING,
                center_z,
            );
            commands.spawn((
                LightcycleSceneRoot,
                StackFrameEntity { level, base },
                Mesh3d(frame.clone()),
                MeshMaterial3d(assets.trail_material.clone()),
                Transform::from_translation(base),
                Visibility::Visible,
                Pickable::IGNORE,
            ));
        }
    }

    // Hex-dump highway: the district's own procedural plate layout, wearing the
    // district's accents.
    let theme = city_theme_index(run.arena.city_theme);
    for plate in road_plates(stable_path_seed(path), &run.arena.roads) {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.city_accent_materials[theme][plate.accent].clone()),
            Transform::from_xyz(plate.cell.0 as f32 * span, 0.07, plate.cell.1 as f32 * span)
                .with_rotation(Quat::from_rotation_y(plate.yaw))
                .with_scale(Vec3::new(
                    span * 0.22 * plate.scale,
                    0.06,
                    span * 0.62 * plate.scale,
                )),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }

    // Quarantine vault: risky directory names get warning pylons at the corners.
    let quarantined = is_quarantined(path);
    state.quarantined = quarantined;
    if quarantined {
        let px = (run.arena.max.0 - run.arena.min.0) as f32 * span;
        let pz = (run.arena.max.1 - run.arena.min.1) as f32 * span;
        for (dx, dz) in [(-0.5, -0.5), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)] {
            commands.spawn((
                LightcycleSceneRoot,
                Mesh3d(assets.unit_cube.clone()),
                MeshMaterial3d(assets.galaga_bug_material.clone()),
                Transform::from_xyz(center_x + dx * px, 3.0, center_z + dz * pz)
                    .with_scale(Vec3::new(1.2, 6.0, 1.2)),
                Visibility::Visible,
                Pickable::IGNORE,
            ));
        }
    }

    // The flood starts at the arena's low-Z edge and rises toward high Z. A
    // grace room never arms it, and never spawns the wall at all.
    flood.min_z = run.arena.min.1 as f32;
    flood.max_z = run.arena.max.1 as f32;
    flood.center_x = center_x;
    flood.width = ((run.arena.max.0 - run.arena.min.0) as f32 + 2.0) * span;
    flood.plane = flood.min_z;
    flood.timer = 0.0;
    flood.active = armed;
    flood.delay = if quarantined {
        config::FLOOD_DELAY_SECONDS * 0.6
    } else {
        config::FLOOD_DELAY_SECONDS
    };
    if armed {
        commands.spawn((
            LightcycleSceneRoot,
            FloodEntity,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.flood_material.clone()),
            Transform::from_xyz(
                flood.center_x,
                config::FLOOD_HEIGHT * 0.5,
                flood.min_z * span,
            )
            .with_scale(Vec3::new(flood.width, config::FLOOD_HEIGHT, 0.4)),
            Visibility::Hidden,
            Pickable::IGNORE,
            // A brighter band along the crest: a translucent sheet on its own reads
            // as a scan line, the lit top edge makes it a wall.
            children![(
                Mesh3d(assets.unit_cube.clone()),
                MeshMaterial3d(assets.flood_crest_material.clone()),
                Transform::from_xyz(0.0, 0.5, 0.0).with_scale(Vec3::new(1.0, 0.06, 1.2)),
            )],
        ));
    }

    // The collector's sweep rides the same arena bounds as the flood, so it
    // needs no state of its own beyond its countdown.
    state.gc_sweep = 0.0;
    commands.spawn((
        LightcycleSceneRoot,
        GcSweepEntity,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.gc_sweep_material.clone()),
        Transform::from_xyz(
            flood.center_x,
            config::GC_SWEEP_HEIGHT * 0.5,
            flood.min_z * span,
        )
        .with_scale(Vec3::new(
            flood.width,
            config::GC_SWEEP_HEIGHT,
            config::GC_SWEEP_THICKNESS,
        )),
        Visibility::Hidden,
        Pickable::IGNORE,
    ));
}

/// Paints the sky and the key light with the district's own accent.
///
/// The street plan and the theme already come from the directory's seed; this
/// carries that identity into the ambience, so two directories do not just
/// differ in layout but in light and air. Explorer mode keeps the plain
/// background, so the filesystem view stays neutral.
fn apply_district_ambience(
    mode: Res<InteractionMode>,
    state: Res<LightcycleState>,
    mut clear: ResMut<ClearColor>,
    light: Single<&mut DirectionalLight>,
    mut applied: Local<Option<Option<usize>>>,
) {
    let riding = *mode == InteractionMode::Lightcycle;
    let theme = (riding)
        .then(|| {
            state
                .run
                .as_ref()
                .map(|run| city_theme_index(run.arena.city_theme))
        })
        .flatten();
    if *applied == Some(theme) {
        return;
    }
    *applied = Some(theme);

    let mut light = light.into_inner();
    let Some(index) = theme else {
        clear.0 = config::BACKGROUND_COLOR;
        let default_light = DirectionalLight::default();
        light.color = default_light.color;
        light.illuminance = default_light.illuminance;
        return;
    };

    let palette = city_palette();
    let accent = palette[index][0].1;
    clear.0 = Color::LinearRgba(mix_linear(
        config::BACKGROUND_COLOR.into(),
        accent,
        config::DISTRICT_SKY_MIX,
    ));
    light.color = Color::LinearRgba(mix_linear(
        LinearRgba::WHITE,
        accent,
        config::DISTRICT_LIGHT_MIX,
    ));
    // Districts differ in brightness as well as hue, deterministically.
    let spread = (index as f32 - 1.5) * config::DISTRICT_LIGHT_SPREAD;
    light.illuminance = DirectionalLight::default().illuminance * (1.0 + spread);
}

/// Straight linear blend, used for the district tints.
fn mix_linear(from: LinearRgba, to: LinearRgba, amount: f32) -> LinearRgba {
    let amount = amount.clamp(0.0, 1.0);
    LinearRgba::new(
        from.red + (to.red - from.red) * amount,
        from.green + (to.green - from.green) * amount,
        from.blue + (to.blue - from.blue) * amount,
        from.alpha + (to.alpha - from.alpha) * amount,
    )
}

/// True when any component of the path names a well-known heavy or hidden
/// build directory worth a quarantine.
fn is_quarantined(path: &Path) -> bool {
    path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        config::QUARANTINE_NAMES.contains(&name.as_str())
    })
}

/// Builds the outline of one call-stack frame: four bars around an open middle,
/// centred on the origin so it can be lifted to its resting height.
fn stack_frame_mesh(width: f32, depth: f32) -> Mesh {
    let thickness = config::STACK_FRAME_THICKNESS;
    // A chunky bar is a big fraction of a small arena, so cap it below half the
    // span: the frame stays an outline instead of folding into itself.
    let bar = config::STACK_FRAME_BAR.min(width * 0.4).min(depth * 0.4);
    // Measured to the outside of the bars, so they sit on the edge rather than
    // hanging past it.
    let half_x = ((width - bar) * 0.5).max(0.0);
    let half_z = ((depth - bar) * 0.5).max(0.0);
    // The near bar doubles as the base the other three merge onto.
    let mut mesh = Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(Vec3::new(0.0, 0.0, -half_z))
            .with_scale(Vec3::new(width, thickness, bar)),
    );
    let far = Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(Vec3::new(0.0, 0.0, half_z))
            .with_scale(Vec3::new(width, thickness, bar)),
    );
    let side = (depth - bar * 2.0).max(thickness);
    let left = Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(Vec3::new(-half_x, 0.0, 0.0))
            .with_scale(Vec3::new(bar, thickness, side)),
    );
    let right = Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(Vec3::new(half_x, 0.0, 0.0))
            .with_scale(Vec3::new(bar, thickness, side)),
    );
    for segment in [far, left, right] {
        mesh.merge(&segment)
            .expect("stack frame cuboids must be merge-compatible");
    }
    mesh
}

/// Raises the memory flood along a directory run and crashes the rider when it
/// catches them. Contact is a plain grid-Z comparison: the bike is caught once
/// it falls behind the flood's line.
fn update_flood(
    time: Res<Time>,
    pause: Res<PauseState>,
    mut state: ResMut<LightcycleState>,
    mut flood: ResMut<FloodState>,
    mut effects: MessageWriter<MusicSfx>,
    mut walls: Query<(&FloodEntity, &mut Transform, &mut Visibility)>,
) {
    if pause.paused {
        return;
    }
    let directory = state
        .run
        .as_ref()
        .is_some_and(|run| matches!(run.environment, RunEnvironment::Directory { .. }));
    if !directory || !flood.active {
        for (_, _, mut visibility) in &mut walls {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let span = (flood.max_z - flood.min_z).max(1.0);
    flood.timer += time.delta_secs();
    let rising = flood.timer - flood.delay;
    if rising < 0.0 {
        flood.plane = flood.min_z;
        for (_, mut transform, mut visibility) in &mut walls {
            transform.translation.z = flood.min_z * config::GRID_SPACING;
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let progress = (rising / config::FLOOD_CROSSING_SECONDS).min(1.0);
    flood.plane = flood.min_z + progress * span;
    for (_, mut transform, mut visibility) in &mut walls {
        transform.translation.z = flood.plane * config::GRID_SPACING;
        *visibility = Visibility::Visible;
    }

    if let Some(run) = state.run.as_mut()
        && run.sim.phase == RunPhase::Running
        && (run.sim.cell.1 as f32) < flood.plane - 0.5
    {
        run.sim.phase = RunPhase::Crashed;
        run.crash_label = Some("a buffer overflow".to_string());
        state.crash_fx = Some(crate::lightcycle::CrashFx::new(
            config::LIGHTCYCLE_CRASH_FX_DURATION,
        ));
        effects.write(MusicSfx::Crash);
    }

    if progress >= 1.0 {
        // The spill is reclaimed and the flood recedes to the far edge.
        flood.timer = 0.0;
    }
}

/// Where the collector's sweep stands, in grid Z, for the time left on its
/// countdown. It enters at the arena's low-Z edge and leaves at the high-Z one,
/// so `remaining == duration` is the start and `remaining == 0` is the end.
fn gc_sweep_plane(remaining: f32, duration: f32, min_z: f32, max_z: f32) -> f32 {
    let travelled = 1.0 - (remaining / duration.max(f32::EPSILON)).clamp(0.0, 1.0);
    min_z + (max_z - min_z) * travelled
}

/// Draws the collector's sweep. The stall in `step_lightcycle` is what the
/// player feels; this is what tells them why.
fn update_gc_sweep(
    pause: Res<PauseState>,
    state: Res<LightcycleState>,
    mut sweeps: Query<(&GcSweepEntity, &mut Transform, &mut Visibility)>,
) {
    let Ok((_, mut transform, mut visibility)) = sweeps.single_mut() else {
        return;
    };
    if pause.paused || state.gc_sweep <= 0.0 {
        *visibility = Visibility::Hidden;
        return;
    }
    let Some(run) = state.run.as_ref() else {
        *visibility = Visibility::Hidden;
        return;
    };
    let min_z = run.arena.min.1 as f32;
    let max_z = run.arena.max.1 as f32;
    let plane = gc_sweep_plane(state.gc_sweep, config::GC_SWEEP_SECONDS, min_z, max_z);
    transform.translation.z = plane * config::GRID_SPACING;
    *visibility = Visibility::Visible;
}

fn start_document_loads(
    mut requests: MessageReader<DocumentRequested>,
    mut documents: ResMut<DocumentLoadState>,
) {
    for request in requests.read() {
        let generation = documents.next_generation();
        documents.begin_load(generation, request.path.clone());
    }
}

fn poll_document_loads(
    mut documents: ResMut<DocumentLoadState>,
    mut loaded: MessageWriter<DocumentLoaded>,
    mut failed: MessageWriter<DocumentLoadFailed>,
) {
    while let Some(result) = documents.poll() {
        if result.generation != documents.generation {
            continue;
        }
        match result.result {
            Ok(bytes) => {
                loaded.write(DocumentLoaded {
                    path: result.path,
                    bytes,
                });
            }
            Err(message) => {
                failed.write(DocumentLoadFailed {
                    path: result.path,
                    message,
                });
            }
        }
    }
}

fn reset_on_document_loaded(
    mut loaded: MessageReader<DocumentLoaded>,
    mode: Res<InteractionMode>,
    mut state: ResMut<LightcycleState>,
    assets: Res<LightcycleAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    old_lightcycle_entities: SceneEntities,
) {
    if *mode != InteractionMode::Lightcycle {
        return;
    }

    for event in loaded.read() {
        despawn_lightcycle_entities(&mut commands, &old_lightcycle_entities);
        let run = build_document_run(&event.path, &event.bytes);
        spawn_run_entities(&mut commands, &assets, &mut meshes, &run);
        state.clock = 0.0;
        state.crash_fx = None;
        state.entry_fx = None;
        state.restore_directory = false;
        state.run = Some(run);
    }
}

fn apply_load_failure(
    mut failed: MessageReader<DirectoryLoadFailed>,
    mut state: ResMut<LightcycleState>,
    mut commands: Commands,
    effect_entities: Query<Entity, With<EntryTransportEntity>>,
) {
    if failed.read().next().is_none() {
        return;
    }

    if let Some(run) = state.run.as_mut()
        && run.sim.phase == RunPhase::EnteringDir
    {
        run.sim.pending_request = None;
        run.entering_label = None;
        run.sim.phase = RunPhase::Running;
    }
    state.entry_fx = None;
    for entity in &effect_entities {
        commands.entity(entity).despawn();
    }
}

fn apply_document_load_failure(
    mut failed: MessageReader<DocumentLoadFailed>,
    mut documents: ResMut<DocumentLoadState>,
    mut state: ResMut<LightcycleState>,
) {
    let Some(event) = failed.read().next() else {
        return;
    };
    documents.last_error = Some(format!("{}: {}", event.path.display(), event.message));
    if let Some(run) = state.run.as_mut() {
        run.sim.pending_request = None;
        run.entering_label = None;
        run.sim.phase = RunPhase::Running;
        run.crash_label = Some(format!("could not open {}", event.path.display()));
    }
}

fn start_source_loads(
    mut requests: MessageReader<SourceRequested>,
    mut sources: ResMut<SourceLoadState>,
) {
    for request in requests.read() {
        let generation = sources.next_generation();
        sources.begin_load(generation, request.path.clone());
    }
}

fn poll_source_loads(
    mut sources: ResMut<SourceLoadState>,
    mut loaded: MessageWriter<SourceLoaded>,
    mut failed: MessageWriter<SourceLoadFailed>,
) {
    while let Some(result) = sources.poll() {
        if result.generation != sources.generation {
            continue;
        }
        match result.result {
            Ok(bytes) => {
                loaded.write(SourceLoaded {
                    path: result.path,
                    bytes,
                });
            }
            Err(message) => {
                failed.write(SourceLoadFailed {
                    path: result.path,
                    message,
                });
            }
        }
    }
}

fn reset_on_source_loaded(
    mut loaded: MessageReader<SourceLoaded>,
    mode: Res<InteractionMode>,
    mut state: ResMut<LightcycleState>,
    assets: Res<LightcycleAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    old_lightcycle_entities: SceneEntities,
) {
    if *mode != InteractionMode::Lightcycle {
        return;
    }

    for event in loaded.read() {
        let Some(language) = SourceLanguage::from_path(&event.path) else {
            continue;
        };
        despawn_lightcycle_entities(&mut commands, &old_lightcycle_entities);
        let run = build_source_run(&event.path, language, &event.bytes);
        spawn_run_entities(&mut commands, &assets, &mut meshes, &run);
        state.clock = 0.0;
        state.crash_fx = None;
        state.entry_fx = None;
        state.restore_directory = false;
        state.run = Some(run);
    }
}

/// Warps into a game from the pause menu, using a synthetic file of the
/// game's representative language. Mirrors `reset_on_source_loaded` so the
/// swap looks identical to entering a real file.
fn handle_warp_requests(
    mut requests: MessageReader<WarpRequested>,
    mut state: ResMut<LightcycleState>,
    assets: Res<LightcycleAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mut pause: ResMut<PauseState>,
    old_lightcycle_entities: SceneEntities,
) {
    let Some(request) = requests.read().next() else {
        return;
    };
    let language = SourceLanguage::for_game(request.game);
    let bytes = warp_bytes(language);
    let path = PathBuf::from(format!("warp/level.{}", language.name()));
    despawn_lightcycle_entities(&mut commands, &old_lightcycle_entities);
    let run = build_source_run(&path, language, &bytes);
    spawn_run_entities(&mut commands, &assets, &mut meshes, &run);
    state.clock = 0.0;
    state.crash_fx = None;
    state.entry_fx = None;
    state.restore_directory = false;
    state.run = Some(run);
    pause.paused = false;
}

/// A short synthetic source file so the warp menu can build any game without
/// hunting for a real file of that language.
fn warp_bytes(language: SourceLanguage) -> Vec<u8> {
    let text = match language {
        SourceLanguage::Rust => "fn warp() {\n    let x = 1;\n}\nfn chase() {}\n",
        SourceLanguage::C => "int warp(void) {\n    return 1;\n}\nint chase(void) { return 0; }\n",
        SourceLanguage::Cpp => "int warp() {\n    return 1;\n}\nint chase() { return 0; }\n",
        SourceLanguage::Python => "def warp():\n    return 1\n\ndef chase():\n    return 0\n",
        SourceLanguage::Slint => "export component Warp {\n    in property <int> x: 1;\n}\n",
        SourceLanguage::Lua => {
            "local function warp()\n    return 1\nend\nlocal function chase() return 0 end\n"
        }
        SourceLanguage::Shell => "warp() {\n    echo 1\n}\nchase() { echo 0; }\n",
        SourceLanguage::Toml => "[warp]\nvalue = 1\n[chase]\nvalue = 0\n",
        SourceLanguage::Json => "{\n  \"warp\": 1,\n  \"chase\": 0\n}\n",
        SourceLanguage::Go => {
            "package warp\n\nfunc warp() int {\n    return 1\n}\nfunc chase() int { return 0 }\n"
        }
        SourceLanguage::Ruby => "def warp\n  1\nend\n\ndef chase\n  0\nend\n",
        SourceLanguage::Yaml => "warp: 1\nchase: 0\n",
        SourceLanguage::JavaScript => {
            "function warp() {\n    return 1;\n}\nconst chase = () => 0;\n"
        }
        SourceLanguage::Zig => "fn warp() i32 {\n    return 1;\n}\nfn chase() i32 { return 0; }\n",
        SourceLanguage::Php => {
            "<?php\nfunction warp() {\n    return 1;\n}\nfunction chase() { return 0; }\n"
        }
        SourceLanguage::R => "warp <- function() {\n    1\n}\nchase <- function() { 0 }\n",
    };
    text.as_bytes().to_vec()
}

fn apply_source_load_failure(
    mut failed: MessageReader<SourceLoadFailed>,
    mut sources: ResMut<SourceLoadState>,
    mut state: ResMut<LightcycleState>,
) {
    let Some(event) = failed.read().next() else {
        return;
    };
    sources.last_error = Some(format!("{}: {}", event.path.display(), event.message));
    if let Some(run) = state.run.as_mut() {
        run.sim.pending_request = None;
        run.entering_label = None;
        run.sim.phase = RunPhase::Running;
        run.crash_label = Some(format!("could not open {}", event.path.display()));
    }
}

#[allow(clippy::too_many_arguments)]
fn read_lightcycle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    transition: Res<ModeTransition>,
    time: Res<Time>,
    mut state: ResMut<LightcycleState>,
    mut pause: ResMut<PauseState>,
    mut history: ResMut<HistoryState>,
    mut flood: ResMut<FloodState>,
    mut navigator: ResMut<NavigatorResource>,
    mut requests: MessageWriter<DirectoryRequested>,
    mut effects: MessageWriter<MusicSfx>,
    mut warps: MessageWriter<WarpRequested>,
) {
    if history.notice_timer > 0.0 {
        history.notice_timer = (history.notice_timer - time.delta_secs()).max(0.0);
        if history.notice_timer == 0.0 {
            history.notice.clear();
        }
    }

    // Riding controls belong to the run, not to the flight arriving at it.
    if transition.is_active() {
        state.slow_motion = false;
        return;
    }

    // The pause menu owns the controls while it is up: navigate with W/S or
    // the arrows, warp with Enter/Space/click, and resume with Esc or P.
    let pause_key = keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyP);
    if pause.paused {
        let up = keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp);
        let down = keys.just_pressed(KeyCode::KeyS) || keys.just_pressed(KeyCode::ArrowDown);
        if up {
            pause.warp_index = (pause.warp_index + SourceGame::COUNT - 1) % SourceGame::COUNT;
        }
        if down {
            pause.warp_index = (pause.warp_index + 1) % SourceGame::COUNT;
        }
        if pause_key {
            pause.paused = false;
        }
        let warp = keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::Space)
            || mouse.just_pressed(MouseButton::Left);
        if warp {
            warps.write(WarpRequested {
                game: SourceGame::ALL[pause.warp_index],
            });
        }
        state.slow_motion = false;
        return;
    }
    if pause_key {
        pause.paused = true;
        return;
    }

    // Git time machine: Z rewinds to the previous directory ridden and Y puts
    // the present back. Both re-enter an arena, so the layout returns with it.
    if keys.just_pressed(KeyCode::KeyZ) {
        match history.rewind() {
            Some(path) => {
                history.notice = "REWIND".to_string();
                history.notice_timer = config::HISTORY_NOTICE_SECONDS;
                effects.write(MusicSfx::Seek);
                requests.write(DirectoryRequested { path });
            }
            None => {
                history.notice = "AT THE FIRST COMMIT".to_string();
                history.notice_timer = config::HISTORY_NOTICE_SECONDS;
            }
        }
        return;
    }
    if keys.just_pressed(KeyCode::KeyY) {
        match history.fast_forward() {
            Some(path) => {
                history.notice = "FAST-FORWARD".to_string();
                history.notice_timer = config::HISTORY_NOTICE_SECONDS;
                effects.write(MusicSfx::Seek);
                requests.write(DirectoryRequested { path });
            }
            None => {
                history.notice = "NOTHING TO REDO".to_string();
                history.notice_timer = config::HISTORY_NOTICE_SECONDS;
            }
        }
        return;
    }

    let left = keys.just_pressed(KeyCode::KeyA) || keys.just_pressed(KeyCode::ArrowLeft);
    let right = keys.just_pressed(KeyCode::KeyD) || keys.just_pressed(KeyCode::ArrowRight);
    let restart = keys.just_pressed(KeyCode::KeyR);
    let go_up = keys.just_pressed(KeyCode::KeyU) || keys.just_pressed(KeyCode::Minus);
    let throw = keys.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left);
    let recall = keys.just_pressed(KeyCode::KeyQ);
    // Stealth walks on WASD: `steer` is the held east/west axis, so only the
    // other pair is needed here.
    let move_z = i32::from(keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown))
        - i32::from(keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp));
    // Hop inputs for the arcade block: one cell per tap, like Frogger and
    // Q*bert need.
    let hop_z = i32::from(keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp))
        - i32::from(keys.just_pressed(KeyCode::KeyS) || keys.just_pressed(KeyCode::ArrowDown));
    // Steering is continuous: the asteroid field pivots while the key is held.
    let steer = i32::from(keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight))
        - i32::from(keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft));
    // The surfer's throttle is always open; boost is held, on the same keys
    // that would otherwise walk or jump.
    let boost = keys.pressed(KeyCode::Space)
        || keys.pressed(KeyCode::KeyW)
        || keys.pressed(KeyCode::ArrowUp);
    // Bullet time: hold Shift to slow the ring while lining up a turn or shot.
    state.slow_motion = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    let entering_tower = state.entry_fx.is_some();
    let Some(mut run) = state.run.take() else {
        return;
    };

    // The field is only a turret fight while the rocks are live; after it is won
    // or lost the bike is handed back and drives normally.
    let field_active = run.asteroid_field_active();

    // Which runs steer the shared bike grid this frame? Directories and
    // documents always do, disc wars and snake always do, the field only once it
    // has handed the bike back, and the off-grid games never.
    let drives_grid = match run.source_game() {
        Some(SourceGame::DiscWars | SourceGame::Snake) | None => true,
        Some(SourceGame::Asteroids) => !field_active,
        Some(
            SourceGame::Platformer
            | SourceGame::Breaker
            | SourceGame::Stealth
            | SourceGame::RiverSurfer
            | SourceGame::Galaga
            | SourceGame::PacMan
            | SourceGame::Columns
            | SourceGame::Tetris
            | SourceGame::Frogger
            | SourceGame::Qbert
            | SourceGame::Bomberman
            | SourceGame::Plinko,
        ) => false,
    };
    if drives_grid && run.sim.phase == RunPhase::Running && (left || right) {
        run.sim.queue_turn_input(left, right);
        effects.write(MusicSfx::Turn);
    }

    if run.is_source() && run.sim.phase == RunPhase::Running {
        if field_active {
            // Pivot and shoot. `set_turn` persists for the fixed sub-steps that
            // follow this frame.
            let mut fired = false;
            if let Some(field) = run.source_asteroids_mut() {
                field.set_turn(steer as f32);
                if throw {
                    fired = field.fire();
                }
            }
            if fired {
                effects.write(MusicSfx::Zap);
            }
        } else if let Some(level) = run.source_platformer_mut() {
            // Held to run, tapped to jump.
            level.set_input(steer as f32, throw);
        } else if let Some(level) = run.source_breaker_mut() {
            // Held to slide the bike along the bottom, tapped to serve.
            level.set_input(steer as f32, throw);
        } else if let Some(room) = run.source_stealth_mut() {
            // Hold a direction to keep walking it; let go to stop.
            room.set_input(steer, move_z);
        } else if let Some(surfer) = run.source_surfer_mut() {
            // Steer the hoverbike; the throttle is always open and boost is held.
            surfer.set_input(steer as f32, boost);
        } else if let Some(sim) = run.source_galaga_mut() {
            // Slide along the bottom; the -Z camera mirrors X, so negate steer.
            sim.set_input(-steer as f32, throw);
        } else if let Some(sim) = run.source_pacman_mut() {
            // Hold a direction to keep walking the corridor.
            sim.set_input(steer, move_z);
        } else if let Some(sim) = run.source_columns_mut() {
            // A/D slides the piece, W rotates, Space hard-drops.
            sim.set_input(i32::from(right) - i32::from(left), hop_z > 0, throw);
        } else if let Some(sim) = run.source_tetris_mut() {
            // A/D slides, W rotates, S soft-drops, Space hard-drops.
            sim.set_input(
                i32::from(right) - i32::from(left),
                hop_z > 0,
                move_z > 0,
                throw,
            );
        } else if let Some(sim) = run.source_frogger_mut() {
            // One hop per keypress in any of the four directions. Hop-Z is
            // +1 for W, but the sim counts +Z as the start row, so flip it.
            sim.hop(i32::from(right) - i32::from(left), -hop_z);
        } else if let Some(sim) = run.source_qbert_mut() {
            // Diagonal hops: A/D/W/S each map to a pyramid direction. The -Z
            // camera mirrors X, so swap the east/west edges.
            sim.hop(i32::from(left) - i32::from(right), hop_z);
        } else if let Some(sim) = run.source_bomberman_mut() {
            // Walk on the room grid and plant bombs with Space or click.
            if (steer != 0 || move_z != 0) && sim.phase == BomberPhase::Walking {
                sim.step(steer, move_z);
            }
            if throw {
                sim.plant();
            }
        } else if let Some(sim) = run.source_plinko_mut() {
            // Slide the rail and drop balls with Space or click.
            let slide = sim.aim + steer as f32 * 6.0;
            sim.set_aim(slide);
            if throw {
                sim.drop_ball();
            }
        } else if run.source_game() == Some(SourceGame::DiscWars) {
            // Disc wars: throw and recall. The cycle's movement is unchanged.
            let snapshot = PlayerSnapshot {
                cell: run.sim.cell,
                heading: run.sim.heading,
                running: true,
            };
            if throw {
                let mut events = DiscEvents::default();
                // Aimed at the opponent, so riding and aiming stay separate.
                if let RunEnvironment::Source { sim, .. } = &mut run.environment
                    && let Some(disc) = sim.as_disc_mut()
                {
                    disc.throw_player(snapshot, &run.arena, &mut events);
                }
                if events.player_threw {
                    effects.write(MusicSfx::Beam);
                }
            }
            if recall {
                let mut events = DiscEvents::default();
                if let Some(disc) = run.source_disc_mut() {
                    disc.recall_player(&mut events);
                }
                if events.player_recalled {
                    effects.write(MusicSfx::Turn);
                }
            }
        }
    }

    if restart {
        restart_run(&mut run);
        // The wall that ended the last run would otherwise still be standing
        // past the spawn cell, killing the respawn on its first frame.
        flood.recede();
        // A restarted run starts with the collector idle too, rather than
        // inheriting a sweep that was half way across the old one.
        state.gc_pause = 0.0;
        state.gc_sweep = 0.0;
        state.gc_timer = config::GC_INTERVAL_SECONDS;
        state.clock = 0.0;
        state.crash_fx = None;
        state.entry_fx = None;
    }

    if go_up && !entering_tower {
        effects.write(MusicSfx::Portal);
        if run.is_document() || run.is_source() {
            state.restore_directory = true;
        } else if let Some(parent) = navigator.0.begin_go_to_parent() {
            run.sim.pause_for_directory_change();
            run.entering_label = Some("RET → parent".to_string());
            run.crash_label = None;
            requests.write(DirectoryRequested { path: parent });
        }
    }

    state.run = Some(run);
}

fn restore_directory_arena(
    mut state: ResMut<LightcycleState>,
    navigator: Res<NavigatorResource>,
    mut flood: ResMut<FloodState>,
    assets: Res<LightcycleAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    old_lightcycle_entities: SceneEntities,
) {
    if !state.restore_directory {
        return;
    }
    state.restore_directory = false;
    despawn_lightcycle_entities(&mut commands, &old_lightcycle_entities);
    let run = build_active_run(&navigator.0.current_path, navigator.0.entries.clone());
    spawn_run_entities(&mut commands, &assets, &mut meshes, &run);
    // Coming back from a minigame is not a first ride either.
    state.grace_room = false;
    decorate_directory_run(
        &mut commands,
        &assets,
        &mut meshes,
        &mut state,
        &mut flood,
        &navigator.0.current_path,
        &run,
    );
    state.clock = 0.0;
    state.crash_fx = None;
    state.entry_fx = None;
    state.run = Some(run);
}

/// Ends a source run through the shared crash path, so the burst, the shake, the
/// label and `R` behave the same as a grid crash.
fn crash_source(run: &mut ActiveRun, label: &str, effects: &mut MessageWriter<MusicSfx>) {
    if run.sim.phase != RunPhase::Running {
        return;
    }
    effects.write(MusicSfx::Crash);
    run.sim.phase = RunPhase::Crashed;
    run.crash_label = Some(label.to_string());
    run.entering_label = None;
}

fn disc_crash_label(reason: CrashReason) -> String {
    match reason {
        CrashReason::Hazard => "a hazard tile".to_string(),
        CrashReason::Disc => "a disc".to_string(),
        CrashReason::Opponent => "the recognizer".to_string(),
        CrashReason::Wall => "ring wall".to_string(),
        CrashReason::Trail => "your trail".to_string(),
        CrashReason::File => "file".to_string(),
    }
}

fn spawn_crash_effect(
    mut state: ResMut<LightcycleState>,
    assets: Res<LightcycleAssets>,
    mut commands: Commands,
) {
    let Some(fx) = state.crash_fx.as_mut() else {
        return;
    };
    if fx.spawned {
        return;
    }
    fx.spawned = true;

    let Some(run) = state.run.as_ref() else {
        return;
    };
    let origin = cycle_world_position(&run.sim) + Vec3::Y * config::LIGHTCYCLE_CYCLE_HEIGHT * 0.5;
    let count = 18;

    for index in 0..count {
        let angle = index as f32 / count as f32 * std::f32::consts::TAU;
        let speed = 5.0 + (index % 5) as f32 * 1.3;
        let horizontal = Vec3::new(angle.cos(), 0.0, angle.sin());
        let velocity = horizontal * speed + Vec3::Y * (4.0 + (index % 4) as f32 * 1.1);
        let initial_scale = 0.18 + (index % 4) as f32 * 0.05;
        let life = 0.55 + (index % 3) as f32 * 0.1;

        commands.spawn((
            LightcycleSceneRoot,
            CrashDebris {
                velocity,
                life,
                max_life: life,
                initial_scale,
            },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(if index % 3 == 0 {
                assets.trail_material.clone()
            } else {
                assets.crash_material.clone()
            }),
            Transform::from_translation(origin)
                .with_rotation(Quat::from_rotation_y(angle))
                .with_scale(Vec3::splat(initial_scale)),
            Pickable::IGNORE,
        ));
    }
}

fn update_crash_effects(
    time: Res<Time>,
    mut state: ResMut<LightcycleState>,
    mut commands: Commands,
    mut debris: Query<(Entity, &mut Transform, &mut CrashDebris)>,
) {
    let delta = time.delta_secs();
    let gravity = -18.0;

    for (entity, mut transform, mut piece) in &mut debris {
        piece.life -= delta;
        piece.velocity.y += gravity * delta;
        transform.translation += piece.velocity * delta;

        let life_ratio = (piece.life / piece.max_life).max(0.0);
        let scale = piece.initial_scale * life_ratio + 0.02;
        transform.scale = Vec3::splat(scale);

        if piece.life <= 0.0 {
            commands.entity(entity).despawn();
        }
    }

    if let Some(fx) = state.crash_fx.as_mut() {
        fx.timer -= delta;
        if fx.timer <= 0.0 {
            state.crash_fx = None;
        }
    }
}

/// Removes transport geometry when a restart cancels the timeline before a new
/// arena arrives. Normal directory loads remove it with the rest of the old
/// lightcycle scene.
fn cleanup_orphaned_entry_effect(
    state: Res<LightcycleState>,
    mut commands: Commands,
    effects: Query<Entity, With<EntryTransportEntity>>,
) {
    if state.entry_fx.is_some() {
        return;
    }
    for entity in &effects {
        commands.entity(entity).despawn();
    }
}

/// Creates a translucent column and a stack of independent neon rings around
/// the stopped cycle. Navigation is deliberately not started here: the update
/// system below waits until the beam reaches its apex.
fn spawn_entry_effect(
    mut state: ResMut<LightcycleState>,
    assets: Res<LightcycleAssets>,
    mut commands: Commands,
) {
    let Some(fx) = state.entry_fx.as_mut() else {
        return;
    };
    if fx.spawned {
        return;
    }
    fx.spawned = true;

    let Some(run) = state.run.as_ref() else {
        return;
    };
    let origin = cycle_world_position(&run.sim);
    commands.spawn((
        LightcycleSceneRoot,
        EntryTransportEntity,
        EntryBeam,
        Mesh3d(assets.entry_beam_mesh.clone()),
        MeshMaterial3d(assets.entry_beam_material.clone()),
        Transform::from_translation(
            origin + Vec3::Y * (config::LIGHTCYCLE_ENTRY_BEAM_HEIGHT * 0.5),
        )
        .with_scale(Vec3::new(0.02, 1.0, 0.02)),
        Pickable::IGNORE,
    ));

    for index in 0..config::LIGHTCYCLE_ENTRY_HALO_COUNT {
        let phase = index as f32 / config::LIGHTCYCLE_ENTRY_HALO_COUNT as f32;
        commands.spawn((
            LightcycleSceneRoot,
            EntryTransportEntity,
            EntryHalo { phase },
            Mesh3d(assets.entry_halo_mesh.clone()),
            MeshMaterial3d(assets.entry_halo_material.clone()),
            Transform::from_translation(origin),
            Pickable::IGNORE,
        ));
    }
}

/// Brightness/size envelope for the whole transport. The quick rise makes the
/// collision read as a capture; the tail collapses as the old arena disappears.
fn entry_effect_envelope(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    if progress < 0.18 {
        smoothstep(progress / 0.18)
    } else {
        1.0 - smoothstep((progress - 0.18) / 0.82)
    }
}

/// Height and scale of one halo in the repeating upward sweep.
fn entry_halo_pose(progress: f32, phase: f32) -> (f32, f32) {
    let sweep = (progress * 2.0 + phase).fract();
    let height = 0.35 + sweep * config::LIGHTCYCLE_ENTRY_HALO_HEIGHT;
    let ring_envelope = (std::f32::consts::PI * sweep).sin().max(0.0);
    let scale = entry_effect_envelope(progress) * (0.35 + ring_envelope * 0.85);
    (height, scale)
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

#[allow(clippy::too_many_arguments)]
fn animate_entry_effect(
    time: Res<Time>,
    mut state: ResMut<LightcycleState>,
    mut navigator: ResMut<NavigatorResource>,
    mut requests: MessageWriter<DirectoryRequested>,
    mut beam: Query<&mut Transform, Only<EntryBeam, EntryHalo, CycleEntity>>,
    mut halos: Query<(&EntryHalo, &mut Transform), Apart<EntryBeam>>,
    mut cycle: Query<&mut Transform, Only<CycleEntity, EntryBeam, EntryHalo>>,
) {
    let Some(fx) = state.entry_fx.as_mut() else {
        return;
    };
    fx.elapsed += time.delta_secs();
    let progress = fx.progress();
    let envelope = entry_effect_envelope(progress);

    if let Ok(mut transform) = beam.single_mut() {
        let radius = 0.15 + envelope * 0.85;
        transform.scale = Vec3::new(radius, 1.0, radius);
    }
    for (halo, mut transform) in &mut halos {
        let (height, scale) = entry_halo_pose(progress, halo.phase);
        transform.translation.y = height;
        transform.scale = Vec3::splat(scale.max(0.001));
        transform.rotate_y(time.delta_secs() * (1.8 + halo.phase));
    }

    // The cycle rises into the beam only after capture is established. Its base
    // transform is restored by update_cycle_transform immediately before this
    // system each frame, so this offset cannot accumulate.
    if let Ok(mut transform) = cycle.single_mut() {
        let lift = smoothstep((progress - 0.28) / 0.72);
        transform.translation.y += lift * config::LIGHTCYCLE_ENTRY_HALO_HEIGHT * 0.72;
        transform.scale = Vec3::splat(1.0 - lift * 0.72);
    }

    if !fx.requested && progress >= config::LIGHTCYCLE_ENTRY_FX_REQUEST_AT {
        fx.requested = true;
        let target = fx.target.clone();
        navigator.0.begin_navigate_to(&target);
        requests.write(DirectoryRequested { path: target });
    }
}

/// Rewrites the trail mesh every frame so the live end stays glued to the
/// cycle's tail instead of snapping to the last cell center.
fn update_trail_mesh(
    state: Res<LightcycleState>,
    mut meshes: ResMut<Assets<Mesh>>,
    trail: Query<&Mesh3d, With<TrailSceneRoot>>,
) {
    let Some(run) = state.run.as_ref() else {
        return;
    };
    let Ok(mesh3d) = trail.single() else {
        return;
    };
    let Some(mut mesh) = meshes.get_mut(mesh3d.id()) else {
        return;
    };
    *mesh = build_trail_mesh(&run.sim);
}

fn build_trail_mesh(sim: &LightcycleSim) -> Mesh {
    let points = trail_centerline(sim);
    let heights = trail_heights(&points);
    trail_glass_mesh(&points, &heights)
}

/// Cell-space polyline of the wall: committed trail, the same corner arc the
/// cycle is riding, then trimmed so the live end sits at the tail.
fn trail_centerline(sim: &LightcycleSim) -> Vec<(f32, f32)> {
    let mut points = if sim.trail.is_empty() {
        vec![cell_to_point(sim.cell)]
    } else {
        rounded_polyline(&sim.trail)
    };

    if let Some(arc) = corner_arc(sim) {
        if sim.queued_turn.is_some() {
            points.push(cell_to_point(sim.cell));
        }
        let samples = ((arc.u * 10.0).ceil() as usize).max(2);
        for step in 0..=samples {
            let t = arc.u * step as f32 / samples as f32;
            points.push(arc.sample(t).position);
        }
    } else {
        let pose = cycle_cell_pose(sim);
        points.push(pose.position);
    }

    let points = collapse_near_duplicates(points);
    trim_polyline_end(points, config::LIGHTCYCLE_TRAIL_TAIL)
}

fn trail_heights(points: &[(f32, f32)]) -> Vec<f32> {
    let from_end = distances_from_end(points);
    let emanate = config::LIGHTCYCLE_TRAIL_EMANATE;
    let full = config::LIGHTCYCLE_TRAIL_HEIGHT;
    let spawn = config::LIGHTCYCLE_TRAIL_SPAWN_HEIGHT;

    from_end
        .into_iter()
        .map(|distance| {
            if distance >= emanate {
                full
            } else {
                let t = (distance / emanate).clamp(0.0, 1.0);
                let smooth = t * t * (3.0 - 2.0 * t);
                spawn + (full - spawn) * smooth
            }
        })
        .collect()
}

fn distances_from_end(points: &[(f32, f32)]) -> Vec<f32> {
    if points.is_empty() {
        return Vec::new();
    }

    let mut from_start = vec![0.0; points.len()];
    for index in 1..points.len() {
        from_start[index] =
            from_start[index - 1] + point_distance(points[index - 1], points[index]);
    }
    let total = *from_start.last().unwrap_or(&0.0);
    from_start.into_iter().map(|d| total - d).collect()
}

fn point_distance(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = b.0 - a.0;
    let dz = b.1 - a.1;
    (dx * dx + dz * dz).sqrt()
}

fn collapse_near_duplicates(points: Vec<(f32, f32)>) -> Vec<(f32, f32)> {
    let mut collapsed = Vec::with_capacity(points.len());
    for point in points {
        if collapsed
            .last()
            .is_none_or(|previous| point_distance(*previous, point) > 1e-4)
        {
            collapsed.push(point);
        }
    }
    collapsed
}

/// Shortens the live end of a polyline by `trim` cells so the wall stops at
/// the tail instead of the cycle's origin.
fn trim_polyline_end(mut points: Vec<(f32, f32)>, trim: f32) -> Vec<(f32, f32)> {
    let mut remaining = trim;
    while points.len() >= 2 && remaining > 1e-4 {
        let last = points.len() - 1;
        let a = points[last - 1];
        let b = points[last];
        let length = point_distance(a, b);
        if length <= 1e-4 {
            points.pop();
            continue;
        }
        if remaining >= length {
            points.pop();
            remaining -= length;
        } else {
            let t = 1.0 - remaining / length;
            points[last] = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            remaining = 0.0;
        }
    }
    points
}

/// Extrudes a thin glass slab along `points`. Heights vary so the live end is
/// a meniscus at the tail rather than a chopped cuboid.
fn trail_glass_mesh(points: &[(f32, f32)], heights: &[f32]) -> Mesh {
    if points.len() < 2 || heights.len() != points.len() {
        return collapsed_trail_mesh(points.first().copied().unwrap_or_default());
    }

    let spacing = config::GRID_SPACING;
    let half_thick = config::LIGHTCYCLE_TRAIL_THICKNESS * 0.5;
    let stations: Vec<(Vec3, Vec3, f32)> = points
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let origin = Vec3::new(point.0 * spacing, 0.0, point.1 * spacing);
            let tangent = polyline_tangent(points, index);
            let side = Vec3::Y.cross(tangent).normalize_or_zero() * half_thick;
            (origin, side, heights[index])
        })
        .collect();

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    for window in stations.windows(2) {
        let (a_origin, a_side, a_height) = window[0];
        let (b_origin, b_side, b_height) = window[1];

        let a_left = a_origin - a_side;
        let a_right = a_origin + a_side;
        let b_left = b_origin - b_side;
        let b_right = b_origin + b_side;
        let a_left_top = a_left + Vec3::Y * a_height;
        let a_right_top = a_right + Vec3::Y * a_height;
        let b_left_top = b_left + Vec3::Y * b_height;
        let b_right_top = b_right + Vec3::Y * b_height;

        push_quad(
            &mut positions,
            &mut normals,
            &mut indices,
            a_left,
            b_left,
            b_left_top,
            a_left_top,
        );
        push_quad(
            &mut positions,
            &mut normals,
            &mut indices,
            a_right,
            a_right_top,
            b_right_top,
            b_right,
        );
        push_quad(
            &mut positions,
            &mut normals,
            &mut indices,
            a_left_top,
            b_left_top,
            b_right_top,
            a_right_top,
        );
        push_quad(
            &mut positions,
            &mut normals,
            &mut indices,
            a_left,
            a_right,
            b_right,
            b_left,
        );
    }

    let (origin, side, height) = stations[0];
    push_quad(
        &mut positions,
        &mut normals,
        &mut indices,
        origin - side,
        origin - side + Vec3::Y * height,
        origin + side + Vec3::Y * height,
        origin + side,
    );
    let (origin, side, height) = stations[stations.len() - 1];
    push_quad(
        &mut positions,
        &mut normals,
        &mut indices,
        origin - side,
        origin + side,
        origin + side + Vec3::Y * height,
        origin - side + Vec3::Y * height,
    );

    trail_mesh_from(positions, normals, indices)
}

/// An invisible, zero-area quad standing in for a ribbon too short to draw.
///
/// The trail mesh must never be zero-vertex. Bevy's mesh allocator skips
/// allocating a mesh with an empty vertex buffer but still runs the upload for
/// it, which logs `Use-after-free: attempted to copy element data for an
/// unallocated key` every frame. A run has no ribbon yet for the fraction of a
/// cell it takes the tail to clear its spawn, and again after every restart and
/// folder entry, so this is the common case rather than an edge case.
fn collapsed_trail_mesh(anchor: (f32, f32)) -> Mesh {
    let spacing = config::GRID_SPACING;
    let point = Vec3::new(anchor.0 * spacing, 0.0, anchor.1 * spacing);

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    push_quad(
        &mut positions,
        &mut normals,
        &mut indices,
        point,
        point,
        point,
        point,
    );
    trail_mesh_from(positions, normals, indices)
}

fn trail_mesh_from(positions: Vec<[f32; 3]>, normals: Vec<[f32; 3]>, indices: Vec<u32>) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

fn polyline_tangent(points: &[(f32, f32)], index: usize) -> Vec3 {
    let previous = if index == 0 {
        points[0]
    } else {
        points[index - 1]
    };
    let next = if index + 1 == points.len() {
        points[index]
    } else {
        points[index + 1]
    };
    Vec3::new(next.0 - previous.0, 0.0, next.1 - previous.1).normalize_or_zero()
}

fn push_quad(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    a: Vec3,
    b: Vec3,
    c: Vec3,
    d: Vec3,
) {
    let normal = (b - a).cross(d - a).normalize_or_zero();
    let start = positions.len() as u32;
    for vertex in [a, b, c, d] {
        positions.push(vertex.to_array());
        normals.push(normal.to_array());
    }
    indices.extend_from_slice(&[start, start + 1, start + 2, start, start + 2, start + 3]);
}

fn rounded_polyline(path: &[(i32, i32)]) -> Vec<(f32, f32)> {
    let radius = config::LIGHTCYCLE_TURN_RADIUS;
    let mut points = vec![cell_to_point(path[0])];

    for index in 1..path.len().saturating_sub(1) {
        let previous = path[index - 1];
        let corner = path[index];
        let next = path[index + 1];

        if is_path_turn(previous, corner, next) {
            let incoming = (corner.0 - previous.0, corner.1 - previous.1);
            let outgoing = (next.0 - corner.0, next.1 - corner.1);
            let arc_start = offset_cell_point(corner, incoming, -radius);
            points.push(arc_start);

            let samples = 10;
            for step in 1..=samples {
                let u = step as f32 / samples as f32;
                points.push(arc_cell_pose(corner, incoming, outgoing, u, radius).position);
            }
        } else {
            points.push(cell_to_point(corner));
        }
    }

    if let Some(last) = path.last() {
        points.push(cell_to_point(*last));
    }
    points
}

fn is_path_turn(a: (i32, i32), b: (i32, i32), c: (i32, i32)) -> bool {
    let incoming = (b.0 - a.0, b.1 - a.1);
    let outgoing = (c.0 - b.0, c.1 - b.1);
    incoming.0 * outgoing.0 + incoming.1 * outgoing.1 == 0
}

fn cell_to_point(cell: (i32, i32)) -> (f32, f32) {
    (cell.0 as f32, cell.1 as f32)
}

fn offset_cell_point(cell: (i32, i32), direction: (i32, i32), distance: f32) -> (f32, f32) {
    (
        cell.0 as f32 + direction.0 as f32 * distance,
        cell.1 as f32 + direction.1 as f32 * distance,
    )
}

/// Continuous render pose for the cycle, in cell coordinates.
pub(crate) struct CyclePose {
    position: (f32, f32),
    /// Unit travel direction; the cycle's nose points along it.
    direction: Vec2,
    /// Bank angle about the travel direction, in radians. Zero outside corners.
    lean: f32,
}

fn cycle_world_position(sim: &LightcycleSim) -> Vec3 {
    pose_world_position(&cycle_cell_pose(sim))
}

/// The live corner the cycle is riding, if any.
#[derive(Clone, Copy)]
struct CornerArc {
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

fn corner_arc(sim: &LightcycleSim) -> Option<CornerArc> {
    let radius = config::LIGHTCYCLE_TURN_RADIUS;

    // Approaching a queued turn: the first half of the arc happens just before
    // the cycle reaches the intersection cell.
    if let Some(turn) = sim.queued_turn
        && sim.cell_t >= 1.0 - radius
    {
        let incoming = sim.heading.delta();
        let outgoing = sim.heading.turn(turn).delta();
        let u = ((sim.cell_t - (1.0 - radius)) / radius) * 0.5;
        return Some(CornerArc {
            corner: sim.next_cell(),
            incoming,
            outgoing,
            u,
            radius,
        });
    }

    // Just applied a turn: render the second half of the arc after leaving the
    // intersection cell. The previous trail cell tells us the incoming heading.
    if sim.queued_turn.is_none()
        && sim.cell_t <= radius
        && let Some(&previous) = sim.trail.last()
    {
        let incoming = (sim.cell.0 - previous.0, sim.cell.1 - previous.1);
        let outgoing = sim.heading.delta();
        let is_turn = incoming.0 * outgoing.0 + incoming.1 * outgoing.1 == 0;
        if is_turn {
            let u = 0.5 + (sim.cell_t / radius) * 0.5;
            return Some(CornerArc {
                corner: sim.cell,
                incoming,
                outgoing,
                u,
                radius,
            });
        }
    }

    None
}

fn update_cycle_transform(
    state: Res<LightcycleState>,
    mut cycle: Query<(&mut Transform, &mut Visibility), With<CycleEntity>>,
) {
    let Ok((mut transform, mut visibility)) = cycle.single_mut() else {
        return;
    };
    let Some(run) = state.run.as_ref() else {
        return;
    };

    // The on-foot games park the bike out of sight and pose their own character.
    if run.source_platformer().is_some() || run.source_stealth().is_some() {
        *visibility = Visibility::Hidden;
        return;
    }
    *visibility = Visibility::Visible;

    // In the breaker the bike is the paddle: it slides along the bottom of the
    // court and rebounds the ball.
    if let Some(level) = run.source_breaker() {
        transform.translation = Vec3::new(level.paddle_x, config::BREAKER_PADDLE_Y, 0.0);
        transform.rotation = Quat::IDENTITY;
        transform.scale = Vec3::splat(config::BREAKER_PADDLE_SCALE);
        return;
    }
    transform.scale = Vec3::ONE;

    // The surfer rides the shared bike as a hovercraft over the river, bobbing
    // with the waves and leaning into the steering heading.
    if let Some(surfer) = run.source_surfer() {
        transform.translation = Vec3::new(surfer.x, surfer.height, surfer.z);
        transform.rotation = Quat::from_rotation_arc(
            Vec3::X,
            Vec3::new(surfer.heading.cos(), 0.0, surfer.heading.sin()),
        );
        return;
    }

    // The Galaga field parks the bike on the bottom edge, facing up the field,
    // and slides it side to side.
    if let Some(sim) = run.source_galaga() {
        transform.translation = Vec3::new(sim.player_x, 0.0, config::GALAGA_PLAYER_Z);
        transform.rotation = Quat::from_rotation_arc(Vec3::X, Vec3::Z);
        return;
    }

    // Pac-Man rides the maze corridors.
    if let Some(sim) = run.source_pacman() {
        transform.translation = Vec3::new(sim.x, 0.7, sim.z);
        transform.rotation = Quat::from_rotation_arc(Vec3::X, Vec3::Z);
        return;
    }

    // Columns and Tetris park the bike at the foot of the well; Plinko parks
    // it on the top rail.
    if run.source_columns().is_some() {
        transform.translation = Vec3::new(0.0, 0.0, 3.0);
        transform.rotation = Quat::from_rotation_arc(Vec3::X, Vec3::Y);
        return;
    }
    if run.source_tetris().is_some() {
        transform.translation = Vec3::new(0.0, 0.0, 4.0);
        transform.rotation = Quat::from_rotation_arc(Vec3::X, Vec3::Y);
        return;
    }
    if let Some(sim) = run.source_plinko() {
        transform.translation = Vec3::new(sim.aim, config::PLINKO_HEIGHT * 0.5 - 1.0, 0.0);
        transform.rotation = Quat::from_rotation_arc(Vec3::X, Vec3::Y);
        return;
    }

    // Frogger and Bomberman walk the cycle on their X/Z grids.
    if let Some(sim) = run.source_frogger() {
        let (x, z) = FroggerSim::center(sim.cell);
        transform.translation = Vec3::new(x, 0.7, z);
        transform.rotation = Quat::from_rotation_arc(Vec3::X, Vec3::Z);
        return;
    }
    if let Some(sim) = run.source_bomberman() {
        let (x, z) = BomberSim::center(sim.cell);
        transform.translation = Vec3::new(x, 0.7, z);
        transform.rotation = Quat::from_rotation_arc(Vec3::X, Vec3::Z);
        return;
    }

    // Q*bert perches the bike on its current cube.
    if let Some(sim) = run.source_qbert() {
        let (x, z) = QbertSim::cube_position(sim.row, sim.index);
        let y =
            (config::QBERT_ROWS as f32 - 1.0 - sim.row as f32) * config::QBERT_CUBE_HEIGHT * 0.5
                + config::QBERT_CUBE_HEIGHT * 0.6;
        transform.translation = Vec3::new(x, y, z);
        transform.rotation = Quat::from_rotation_arc(Vec3::X, Vec3::Z);
        return;
    }

    let pose = cycle_cell_pose(&run.sim);
    transform.translation = pose_world_position(&pose);
    // A parked cycle pivots on the spot: its facing is the field's aim angle,
    // not a grid heading. Once the field ends it drives again, so the grid
    // heading takes over — and a disc-wars ring never leaves it in the first
    // place.
    let aiming = if run.asteroid_field_active() {
        run.source_asteroids()
    } else {
        None
    };
    transform.rotation = match aiming {
        Some(sim) => {
            Quat::from_rotation_arc(Vec3::X, Vec3::new(sim.angle.cos(), 0.0, sim.angle.sin()))
        }
        None => pose_rotation(&pose),
    };
}

/// Facing, in radians, for a grid heading, matching the field's aim convention
/// (`0` is `+X`, growing toward `+Z`).
fn heading_facing(heading: Heading) -> f32 {
    heading_angle(heading)
}

/// Facing of a grid heading, in radians.
fn heading_angle(heading: Heading) -> f32 {
    heading.angle()
}

/// The grid heading closest to an aim angle. Used when the field ends so the
/// bike drives off in the direction the player was holding.
fn nearest_heading(angle: f32) -> Heading {
    let (x, z) = (angle.cos(), angle.sin());
    if x.abs() >= z.abs() {
        if x >= 0.0 {
            Heading::PosX
        } else {
            Heading::NegX
        }
    } else if z >= 0.0 {
        Heading::PosZ
    } else {
        Heading::NegZ
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
