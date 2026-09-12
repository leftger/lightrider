//! Building, swapping, spawning and restoring lightcycle runs.

use super::city::{
    city_palette, city_theme_index, spawn_arena_walls, spawn_city_floor, spawn_city_structures,
    spawn_road_markings, spawn_towers, tower_position,
};
use super::decor::{decorate_directory_run, mix_linear};
use super::despawn_lightcycle_entities;
use super::space::{
    heading_facing, level_metres, ring_center_world, ring_food_cells, ring_radius_world,
};
use super::trail::spawn_trail_ribbon;
use crate::asteroids::plugin::spawn_asteroid_field;
use crate::asteroids::sim::AsteroidsSim;
use crate::bomberman::plugin::spawn_bomber_room;
use crate::bomberman::sim::BomberSim;
use crate::breaker::plugin::spawn_breaker_court;
use crate::breaker::sim::BreakerSim;
use crate::columns::plugin::spawn_gem_well;
use crate::columns::sim::ColumnsSim;
use crate::config;
use crate::disc::combat::DiscSim;
use crate::disc::language::{SourceGame, SourceLanguage};
use crate::disc::layout::{build_capped_disc_arena, build_disc_arena, build_flat_arena};
use crate::disc::plugin::{spawn_disc_arena, spawn_disc_focus_marker, spawn_ring_shell};
use crate::document::layout::build_document_arena_from_parse;
use crate::document::parse::{ParseLimits, parse_markdown_bytes};
use crate::document::plugin::{spawn_document_focus_marker, spawn_document_page};
use crate::filesystem::node::FileNode;
use crate::frogger::plugin::spawn_frogger_highway;
use crate::frogger::sim::FroggerSim;
use crate::galaga::plugin::spawn_galaga_field;
use crate::galaga::sim::GalagaSim;
use crate::lightcycle::logic::{Arena, GatePlacement, Heading, LightcycleSim};
use crate::lightcycle::scene::ChaseCamera;
use crate::lightcycle::scene::CycleEntity;
use crate::lightcycle::scene::LightcycleAssets;
use crate::lightcycle::scene::SceneEntities;
use crate::lightcycle::scene::pose::chase_landing_pose;
use crate::lightcycle::scene::pose::cycle_cell_pose;
use crate::lightcycle::scene::pose::pose_forward;
use crate::lightcycle::scene::pose::pose_rotation;
use crate::lightcycle::scene::pose::pose_world_position;
use crate::lightcycle::{ActiveRun, LightcycleState, RunEnvironment, SourceSim};
use crate::pacman::plugin::spawn_pac_maze;
use crate::pacman::sim::PacSim;
use crate::platformer::plugin::spawn_platformer_level;
use crate::platformer::sim::PlatformerSim;
use crate::plinko::plugin::spawn_plinko_board;
use crate::plinko::sim::PlinkoSim;
use crate::plugins::transition::{ModeTransition, gods_eye_pose};
use crate::qbert::plugin::spawn_qbert_pyramid;
use crate::qbert::sim::QbertSim;
use crate::snake::plugin::spawn_snake_field;
use crate::snake::sim::SnakeSim;
use crate::state::{
    CacheState, FloodState, HistoryState, InteractionMode, LightcycleSceneRoot, NavigatorResource,
    OrbitCameraResource,
};
use crate::stealth::plugin::spawn_stealth_room;
use crate::stealth::sim::StealthSim;
use crate::surfer::plugin::spawn_surfer_course;
use crate::surfer::sim::SurferSim;
use crate::tetris::plugin::spawn_tetris_board;
use crate::tetris::sim::TetrisSim;
use bevy::prelude::*;
use std::collections::HashMap;
use std::path::Path;

pub(crate) fn in_lightcycle_mode(mode: Res<InteractionMode>) -> bool {
    *mode == InteractionMode::Lightcycle
}

pub(crate) fn build_active_run(path: &Path, nodes: Vec<FileNode>) -> ActiveRun {
    let cells: HashMap<_, _> = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (tower_position(node.grid_pos), index))
        .collect();
    let parent_gate = path
        .parent()
        .is_some()
        .then(|| GatePlacement::for_path(path, config::lightcycle::LIGHTCYCLE_PORTAL_WIDTH_CELLS));
    let mut arena = Arena::from_nodes(
        cells.keys().copied(),
        parent_gate,
        config::lightcycle::LIGHTCYCLE_ARENA_PADDING,
        config::lightcycle::LIGHTCYCLE_MIN_ARENA_SPAN,
    );
    arena.generate_city(
        path,
        cells.keys().copied(),
        config::lightcycle::LIGHTCYCLE_CITY_STRUCTURE_SEED_CHANCE,
        config::lightcycle::LIGHTCYCLE_TOWER_STRIDE,
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

pub(crate) fn build_document_run(path: &Path, bytes: &[u8]) -> ActiveRun {
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
pub(crate) fn build_source_run(path: &Path, language: SourceLanguage, bytes: &[u8]) -> ActiveRun {
    let game = language.game();
    let (arena, layout) = match game {
        // The field and the snake ring want a bounded playfield whatever the
        // file size; only disc wars scales its coliseum with the file.
        SourceGame::Asteroids => build_capped_disc_arena(
            path,
            language,
            bytes,
            config::asteroids::ASTEROIDS_RADIUS_CELLS,
        ),
        SourceGame::Snake => {
            build_capped_disc_arena(path, language, bytes, config::snake::SNAKE_RADIUS_CELLS)
        }
        SourceGame::DiscWars => build_disc_arena(path, language, bytes),
        // The off-grid games carry a metadata-only arena; their level is their
        // own. The half-extent only sizes the (unused) metadata box.
        SourceGame::Platformer | SourceGame::Breaker | SourceGame::Stealth => build_flat_arena(
            path,
            language,
            bytes,
            config::platformer::PLATFORMER_HALF_EXTENT,
        ),
        SourceGame::RiverSurfer => {
            build_flat_arena(path, language, bytes, config::surfer::SURFER_HALF_EXTENT)
        }
        SourceGame::Galaga => {
            build_flat_arena(path, language, bytes, config::galaga::GALAGA_HALF_EXTENT)
        }
        // The whole arcade block shares one arena extent; each game's board is
        // small enough to fit inside it.
        SourceGame::PacMan
        | SourceGame::Columns
        | SourceGame::Tetris
        | SourceGame::Frogger
        | SourceGame::Qbert
        | SourceGame::Bomberman
        | SourceGame::Plinko => {
            build_flat_arena(path, language, bytes, config::arcade::ARCADE_HALF_EXTENT)
        }
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
            config::snake::SNAKE_FOOD_TARGET,
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

pub(crate) fn spawn_sim(arena: &Arena, cells: &HashMap<(i32, i32), usize>) -> LightcycleSim {
    let blocked =
        |cell: (i32, i32)| cells.contains_key(&cell) || arena.street_walls.contains(&cell);

    if let Some((spawn, heading)) = arena.spawn_with_runway(
        blocked,
        config::lightcycle::LIGHTCYCLE_SPAWN_SEARCH_RADIUS,
        config::lightcycle::LIGHTCYCLE_SPAWN_RUNWAY_CELLS,
    ) {
        return LightcycleSim::start(spawn, heading);
    }

    // Nowhere to ride at all: hold the run until a restart or another folder
    // replaces the map.
    let cell = arena
        .nearest_empty_cell(blocked, config::lightcycle::LIGHTCYCLE_SPAWN_SEARCH_RADIUS)
        .unwrap_or_else(|| arena.center());
    LightcycleSim::ready(cell, Heading::PosX)
}

/// Starts the flight between the two modes when `M` is pressed.
///
/// Nothing about the world changes here. The run for a lightcycle mode is built
/// but parked, because the flight has to know which road it is diving into and
/// where its chase rig will end up before [`apply_mode_swap`] puts that arena on
/// screen at the top of the climb.
pub(crate) fn toggle_mode(
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
pub(crate) fn apply_mode_swap(
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
            config::lightcycle::CACHE_BOOST_SECONDS
        } else {
            0.0
        };
        state.run = Some(run);
    }

    state.pending_run = None;
    *mode = target;
    transition.mark_swapped();
}

pub(crate) fn spawn_run_entities(
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
            Transform::from_rotation(Quat::from_rotation_y(
                config::lightcycle::LIGHTCYCLE_MODEL_YAW
            ))
            .with_scale(Vec3::splat(config::lightcycle::LIGHTCYCLE_MODEL_SCALE)),
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

/// Paints the sky and the key light with the district's own accent.
///
/// The street plan and the theme already come from the directory's seed; this
/// carries that identity into the ambience, so two directories do not just
/// differ in layout but in light and air. Explorer mode keeps the plain
/// background, so the filesystem view stays neutral.
pub(crate) fn apply_district_ambience(
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
        config::lightcycle::DISTRICT_SKY_MIX,
    ));
    light.color = Color::LinearRgba(mix_linear(
        LinearRgba::WHITE,
        accent,
        config::lightcycle::DISTRICT_LIGHT_MIX,
    ));
    // Districts differ in brightness as well as hue, deterministically.
    let spread = (index as f32 - 1.5) * config::lightcycle::DISTRICT_LIGHT_SPREAD;
    light.illuminance = DirectionalLight::default().illuminance * (1.0 + spread);
}

pub(crate) fn restore_directory_arena(
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
