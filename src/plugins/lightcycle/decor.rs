//! Ambient scenery: the call stack, the parent gate, the memory flood and the GC sweep.

use super::city::{city_theme_index, wall_plane};
use super::{
    CityBeacon, FloodEntity, GateFrame, GateScanBar, GcSweepEntity, LightcycleAssets,
    StackFrameEntity,
};
use crate::config;
use crate::lightcycle::logic::{
    Arena, ArenaKind, ParentPortal, RunPhase, Wall, road_plates, stable_path_seed,
};
use crate::lightcycle::{ActiveRun, LightcycleState, RunEnvironment};
use crate::music::sfx::MusicSfx;
use crate::state::{FloodState, LightcycleSceneRoot, PauseState, StackMotion};
use bevy::prelude::*;
use std::path::Path;

/// Wraps an angle into `[-PI, PI)` so recentering unwinds the short way round
/// however many times a drag has spun the camera about the cycle.
pub(crate) fn wrap_angle(angle: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    (angle + PI).rem_euclid(TAU) - PI
}

/// True when `cell` is a ring's close gate.
pub(crate) fn is_ring_gate(arena: &Arena, cell: (i32, i32)) -> bool {
    arena
        .parent_portal
        .as_ref()
        .is_some_and(|portal| portal.contains(cell))
}

/// World-space extent of the gate along its wall, covering exactly the cells the
/// simulation accepts.
pub(crate) fn gate_world_span(portal: &ParentPortal) -> (f32, f32) {
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
pub(crate) fn spawn_parent_gate(commands: &mut Commands, assets: &LightcycleAssets, arena: &Arena) {
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
pub(crate) fn gate_pulse(elapsed: f32) -> f32 {
    0.5 + 0.5 * (elapsed * config::LIGHTCYCLE_PORTAL_PULSE_SPEED).sin()
}

/// Height a bar has swept to within its opening, wrapping back to the ground
/// once it reaches the lintel.
pub(crate) fn gate_bar_height(bar: &GateScanBar, elapsed: f32) -> f32 {
    (bar.offset + elapsed * config::LIGHTCYCLE_PORTAL_BAR_SPEED).fract() * bar.travel
}

/// Pulses the gate frame and sweeps its light bars upward, so a gate reads as
/// live and is easy to pick out from the surrounding wall.
pub(crate) fn animate_parent_gate(
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

pub(crate) fn animate_city_beacons(
    time: Res<Time>,
    mut beacons: Query<(&CityBeacon, &mut Transform)>,
) {
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
pub(crate) fn animate_stack_frames(
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

/// Stacks a translucent plate above the arena for each level of the directory
/// path (the call stack), lays the hex-dump highway along its roads, rings a
/// quarantined vault with pylons, and spawns the memory-flood wall.
pub(crate) fn decorate_directory_run(
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

/// Straight linear blend, used for the district tints.
pub(crate) fn mix_linear(from: LinearRgba, to: LinearRgba, amount: f32) -> LinearRgba {
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
pub(crate) fn is_quarantined(path: &Path) -> bool {
    path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        config::QUARANTINE_NAMES.contains(&name.as_str())
    })
}

/// Builds the outline of one call-stack frame: four bars around an open middle,
/// centred on the origin so it can be lifted to its resting height.
pub(crate) fn stack_frame_mesh(width: f32, depth: f32) -> Mesh {
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
pub(crate) fn update_flood(
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
pub(crate) fn gc_sweep_plane(remaining: f32, duration: f32, min_z: f32, max_z: f32) -> f32 {
    let travelled = 1.0 - (remaining / duration.max(f32::EPSILON)).clamp(0.0, 1.0);
    min_z + (max_z - min_z) * travelled
}

/// Draws the collector's sweep. The stall in `step_lightcycle` is what the
/// player feels; this is what tells them why.
pub(crate) fn update_gc_sweep(
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

pub(crate) fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}
