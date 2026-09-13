//! The lightcycle arena: the city, its district themes, ground plates, trail, the memory flood, the collector and the call stack.

use bevy::prelude::Color;

pub const LIGHTCYCLE_CELLS_PER_SEC: f32 = 3.5;

pub const LIGHTCYCLE_FIXED_STEP: f32 = 1.0 / 60.0;

pub const LIGHTCYCLE_MAX_SUBSTEPS: usize = 4;

/// How early (in cells) the rendered path begins curving before an intersection.
pub const LIGHTCYCLE_TURN_RADIUS: f32 = 0.4;

/// How far the cycle banks into a corner, in radians.
pub const LIGHTCYCLE_LEAN_ANGLE: f32 = 0.55;

pub const LIGHTCYCLE_CRASH_FX_DURATION: f32 = 0.6;

/// Duration of the Recognizer-style directory transport. The filesystem load
/// starts at the apex so the animation remains visible even for cached folders.
pub const LIGHTCYCLE_ENTRY_FX_DURATION: f32 = 1.15;

pub const LIGHTCYCLE_ENTRY_FX_REQUEST_AT: f32 = 0.82;

pub const LIGHTCYCLE_ENTRY_BEAM_HEIGHT: f32 = 15.0;

pub const LIGHTCYCLE_ENTRY_BEAM_RADIUS: f32 = 1.45;

pub const LIGHTCYCLE_ENTRY_HALO_COUNT: usize = 7;

pub const LIGHTCYCLE_ENTRY_HALO_HEIGHT: f32 = 10.5;

pub const LIGHTCYCLE_ENTRY_HALO_INNER_RADIUS: f32 = 1.25;

pub const LIGHTCYCLE_ENTRY_HALO_OUTER_RADIUS: f32 = 1.5;

pub const LIGHTCYCLE_ARENA_PADDING: i32 = 2;

/// Smallest arena side, in cells. Arenas are square, so a folder with one or two
/// entries still gets a plaza, streets, and room to turn around.
pub const LIGHTCYCLE_MIN_ARENA_SPAN: i32 = 13;

/// Percentage of buildable cells that seed a short architecture run.
pub const LIGHTCYCLE_CITY_STRUCTURE_SEED_CHANCE: u8 = 18;

pub const LIGHTCYCLE_SPAWN_SEARCH_RADIUS: i32 = 4096;

/// Clear cells the spawn search tries to leave straight ahead of the cycle. At
/// `LIGHTCYCLE_CELLS_PER_SEC` this is over a second and a half of runway, so
/// landing in an unfamiliar folder leaves time to read the streets and pick a
/// turn instead of reacting to whatever sits in the next cell.
pub const LIGHTCYCLE_SPAWN_RUNWAY_CELLS: i32 = 6;

/// Tower lattice spacing in cells. Each original grid row/column is multiplied
/// by this stride, leaving plazas and a two-cell development strip between
/// neighboring filesystem landmarks.
pub const LIGHTCYCLE_TOWER_STRIDE: i32 = 5;

pub const LIGHTCYCLE_TOWER_SIZE: f32 = 2.2;

/// glTF scene rendered as the player's cycle, relative to the `assets` directory.
pub const LIGHTCYCLE_MODEL_ASSET: &str = "models/light_cycle/scene.gltf";

/// Uniform scale for the cycle model. The source asset is 3.31 units long, so
/// this renders the cycle just under one grid cell long.
pub const LIGHTCYCLE_MODEL_SCALE: f32 = 0.72;

/// The model's nose already points down its local +X, which is also the axis
/// gameplay rotates onto the direction of travel, so the mesh needs no spin.
pub const LIGHTCYCLE_MODEL_YAW: f32 = 0.0;

/// Rendered height of the scaled cycle model.
pub const LIGHTCYCLE_CYCLE_HEIGHT: f32 = 1.0;

pub const LIGHTCYCLE_TRAIL_HEIGHT: f32 = 1.85;

/// Thin enough to read as a sheet of glass rather than a stack of bricks.
pub const LIGHTCYCLE_TRAIL_THICKNESS: f32 = 0.14;

/// How far behind the cycle origin the wall is born, in cells. Matches the
/// scaled model's rear axle so the sheet appears to leave the tail, not the
/// cell center.
pub const LIGHTCYCLE_TRAIL_TAIL: f32 = 0.42;

/// Distance along the ribbon, in cells, over which the wall grows from a
/// meniscus at the tail to full height.
pub const LIGHTCYCLE_TRAIL_EMANATE: f32 = 0.55;

pub const LIGHTCYCLE_TRAIL_SPAWN_HEIGHT: f32 = 0.16;

pub const LIGHTCYCLE_WALL_HEIGHT: f32 = 1.4;

pub const LIGHTCYCLE_WALL_THICKNESS: f32 = 0.2;

pub const LIGHTCYCLE_CITY_FOUNDATION_HEIGHT: f32 = 0.28;

pub const LIGHTCYCLE_CITY_STRUCTURE_SIZE: f32 = 1.9;

pub const LIGHTCYCLE_CITY_BARRIER_HEIGHT: f32 = 1.15;

pub const LIGHTCYCLE_CITY_GLASS_HEIGHT: f32 = 2.1;

pub const LIGHTCYCLE_CITY_PYLON_HEIGHT: f32 = 3.4;

pub const LIGHTCYCLE_CITY_FIN_THICKNESS: f32 = 0.22;

pub const LIGHTCYCLE_CITY_CAP_HEIGHT: f32 = 0.1;

/// Neon light-line laid where a structure or arena wall meets the ground. The
/// foundations and the floor are both nearly black, so without this skirt the
/// two surfaces merge into one shape and the base of a wall is invisible.
pub const LIGHTCYCLE_CITY_BASE_TRIM_HEIGHT: f32 = 0.07;

/// How far the skirt sticks out past the surface it outlines, in world units.
pub const LIGHTCYCLE_CITY_BASE_TRIM_OVERHANG: f32 = 0.24;

pub const LIGHTCYCLE_CITY_BEACON_LIMIT: usize = 24;

/// How far the lane markings sit above the floor.
pub const MARKING_HEIGHT: f32 = 0.025;

/// Ceiling on the road cells that get lane markings.
///
/// This used to crop to the 16,384 cells nearest the arena centre, which left
/// the lanes and their junction pads missing from the outer two thirds of a big
/// district. Markings are flat quads now rather than boxes, so covering every
/// road in a huge directory costs less than the old crop did; the ceiling is
/// only here to bound a pathological arena.
pub const LIGHTCYCLE_CITY_ROAD_RENDER_LIMIT: usize = 262_144;

pub const LIGHTCYCLE_PORTAL_HEIGHT: f32 = 2.6;

/// How many wall cells the parent gate covers. Wide enough that reaching the
/// parent directory does not need single-cell precision.
pub const LIGHTCYCLE_PORTAL_WIDTH_CELLS: i32 = 3;

/// Thickness of the gate's posts and lintel.
pub const LIGHTCYCLE_PORTAL_FRAME_THICKNESS: f32 = 0.28;

/// Radians per second of the gate frame's brightness pulse.
pub const LIGHTCYCLE_PORTAL_PULSE_SPEED: f32 = 3.2;

/// Light bars sweeping up through the gate's opening.
pub const LIGHTCYCLE_PORTAL_BAR_COUNT: usize = 4;

/// Full sweeps of the opening per second.
pub const LIGHTCYCLE_PORTAL_BAR_SPEED: f32 = 0.45;

pub const LIGHTCYCLE_PORTAL_BAR_HEIGHT: f32 = 0.18;

pub const LIGHTCYCLE_PORTAL_BAR_ALPHA: f32 = 0.5;

pub const LIGHTCYCLE_TRAIL_COLOR: Color = Color::srgba(0.55, 0.95, 1.0, 1.0);

/// Tint of light passing through the trail, slightly greener than the surface
/// so the sheet reads as thick glass rather than a cyan decal.
pub const LIGHTCYCLE_TRAIL_ATTENUATION: Color = Color::srgba(0.35, 0.9, 0.85, 1.0);

/// Emissive of the trail sheet: a lit PCB trace rather than a plain glass wall.
pub const PCB_TRACE_COLOR: Color = Color::srgb(0.25, 0.98, 0.8);

pub const PCB_TRACE_EMISSIVE: f32 = 2.4;

/// A revisit to an already-visited directory grants a short speed surge.
pub const CACHE_BOOST_SECONDS: f32 = 5.0;

/// Peak ground-speed multiplier at the moment of the hit.
pub const CACHE_BOOST_SCALE: f32 = 1.4;

/// Seconds between garbage-collector sweeps of a directory arena.
pub const GC_INTERVAL_SECONDS: f32 = 24.0;

/// How long the collector stalls the world once it runs.
pub const GC_PAUSE_SECONDS: f32 = 0.3;

/// Simulated-time scale during the collector's pause.
pub const GC_SLOW_SCALE: f32 = 0.2;

/// How long the collector's visible sweep takes to cross the arena.
///
/// The sweep is the reason for the stall, so it lasts longer than the stall
/// itself: you see the wave coming, the world stumbles as it passes, and the
/// wave carries on to the far edge.
pub const GC_SWEEP_SECONDS: f32 = 1.4;

pub const GC_SWEEP_HEIGHT: f32 = 2.6;

pub const GC_SWEEP_THICKNESS: f32 = 0.35;

pub const GC_SWEEP_COLOR: Color = Color::srgba(0.55, 1.0, 0.85, 0.45);

/// The call stack: one open frame per path level, floating over the arena's
/// edge. Solid plates across the arena blocked too much of the road, so the
/// middle of each level is left open and only the border is drawn. They hover,
/// glide and rock, so the space above the city is never still.
pub const STACK_FRAME_BASE_Y: f32 = 8.0;

pub const STACK_FRAME_SPACING: f32 = 2.2;

/// Width of the frame's border bars, and their thickness.
pub const STACK_FRAME_BAR: f32 = 2.4;

pub const STACK_FRAME_THICKNESS: f32 = 0.3;

/// How far the frames overhang the arena walls.
pub const STACK_FRAME_MARGIN: f32 = 1.5;

/// Vertical hover: amplitude in world units, and the rate of the oscillation.
pub const STACK_FRAME_HOVER: f32 = 0.38;

pub const STACK_FRAME_HOVER_SPEED: f32 = 1.1;

/// Horizontal glide: radius of the slow drift, and its rate.
pub const STACK_FRAME_GLIDE: f32 = 0.8;

pub const STACK_FRAME_GLIDE_SPEED: f32 = 0.35;

/// Rocking: peak tilt in radians, and the rate.
pub const STACK_FRAME_ROCK: f32 = 0.03;

pub const STACK_FRAME_ROCK_SPEED: f32 = 0.7;

/// Phase added per level, so the stack ripples instead of moving as one slab.
pub const STACK_FRAME_PHASE_STEP: f32 = 0.75;

/// Every so often the whole stack plunges through the arena and comes back:
/// each frame travels from its ceiling height down to the same distance below
/// the floor, rests there for a beat, and rises again.
pub const STACK_PLUNGE_INTERVAL: f32 = 15.0;

pub const STACK_PLUNGE_SECONDS: f32 = 4.0;

/// Progress each level lags the one above it, so the stack cascades down and
/// refills from the top on the way back.
pub const STACK_PLUNGE_STAGGER: f32 = 0.06;

/// Fraction of the plunge the stack spends at the bottom.
pub const STACK_PLUNGE_HOLD: f32 = 0.3;

pub const STACK_FRAME_MAX: usize = 6;

/// Hex-dump highway: the data plates a district lays along its roads.
///
/// The layout is procedural (see `ViaPattern`), and so is the count: it tracks
/// the size of the district, so a huge room is not left with the handful of
/// plates that suited a small one. The ceiling keeps the draw calls bounded.
pub const PLATE_DENSITY: f32 = 0.06;

pub const PLATE_MIN: usize = 18;

pub const PLATE_MAX: usize = 650;

/// How much of a district's accent bleeds into the sky while riding it.
pub const DISTRICT_SKY_MIX: f32 = 0.32;

/// How much of the accent tints the key light over a district.
pub const DISTRICT_LIGHT_MIX: f32 = 0.45;

/// Key-light brightness spread between districts, as a fraction.
pub const DISTRICT_LIGHT_SPREAD: f32 = 0.14;

/// Directory names that open as a quarantined vault chased by the flood.
pub const QUARANTINE_NAMES: [&str; 8] = [
    "node_modules",
    ".git",
    "target",
    "vendor",
    "__pycache__",
    ".venv",
    ".cache",
    "build",
];

/// The flood waits this long before it starts rising.
pub const FLOOD_DELAY_SECONDS: f32 = 18.0;

/// Seconds the flood takes to cross the arena once it starts.
pub const FLOOD_CROSSING_SECONDS: f32 = 42.0;

pub const FLOOD_COLOR: Color = Color::srgba(1.0, 0.25, 0.15, 0.4);

/// The lit band along the flood's crest, so the hazard reads as a wall.
pub const FLOOD_CREST_COLOR: Color = Color::srgba(1.0, 0.62, 0.35, 0.85);

pub const FLOOD_HEIGHT: f32 = 7.0;

pub const LIGHTCYCLE_WALL_COLOR: Color = Color::srgba(0.0, 0.7, 0.6, 1.0);

pub const LIGHTCYCLE_CITY_FLOOR_COLOR: Color = Color::srgb(0.008, 0.012, 0.025);

pub const LIGHTCYCLE_CITY_FOUNDATION_COLOR: Color = Color::srgb(0.018, 0.025, 0.055);

pub const LIGHTCYCLE_CITY_GLASS_COLOR: Color = Color::srgba(0.08, 0.18, 0.32, 0.62);

pub const LIGHTCYCLE_CITY_CYAN: Color = Color::srgb(0.0, 0.9, 1.0);

pub const LIGHTCYCLE_CITY_BLUE: Color = Color::srgb(0.08, 0.38, 1.0);

pub const LIGHTCYCLE_CITY_MAGENTA: Color = Color::srgb(1.0, 0.03, 0.72);

pub const LIGHTCYCLE_CITY_PINK: Color = Color::srgb(1.0, 0.22, 0.48);

pub const LIGHTCYCLE_CITY_VIOLET: Color = Color::srgb(0.58, 0.16, 1.0);

pub const LIGHTCYCLE_CITY_AMBER: Color = Color::srgb(1.0, 0.56, 0.04);

pub const LIGHTCYCLE_PORTAL_COLOR: Color = Color::srgba(1.0, 0.85, 0.1, 1.0);

/// Trough of the gate frame's pulse.
pub const LIGHTCYCLE_PORTAL_DIM_COLOR: Color = Color::srgba(0.32, 0.25, 0.03, 1.0);

pub const LIGHTCYCLE_CAMERA_DISTANCE: f32 = 14.0;

pub const LIGHTCYCLE_CAMERA_HEIGHT: f32 = 8.0;

pub const LIGHTCYCLE_CAMERA_LOOKAHEAD: f32 = 4.0;

/// Time constant for the chase camera easing onto a new heading, in seconds.
/// Without this lag a corner looks like the world rotating around a still bike.
pub const LIGHTCYCLE_CAMERA_TURN_LAG: f32 = 0.28;

/// Pitch limits for right-drag free look, in radians. The floor keeps the
/// camera above the arena floor and the ceiling stops short of straight down.
pub const LIGHTCYCLE_CAMERA_MIN_PITCH: f32 = 0.08;

pub const LIGHTCYCLE_CAMERA_MAX_PITCH: f32 = 1.45;

/// Time constant for free look easing back behind the cycle once the right
/// mouse button is released, in seconds. Slower than the turn lag so letting go
/// reads as the camera settling rather than snapping.
pub const LIGHTCYCLE_CAMERA_LOOK_RECENTER: f32 = 0.45;
