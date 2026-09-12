//! Disc-wars ring geometry: the shell, gallery, gate and pickups.
//!
//! The Bevy side of the arena lives here, beside the Bevy-free [`crate::disc`]
//! modules.

use crate::config;
use crate::disc::combat::DiscSim;
use crate::disc::layout::DiscLayout;
use crate::filesystem::language::SourceLanguage;
use crate::lightcycle::logic::{Arena, CrashReason};
use crate::lightcycle::scene::DocumentFocusMarker;
use crate::lightcycle::scene::Fighter;
use crate::lightcycle::scene::FreeOf;
use crate::lightcycle::scene::LightcycleAssets;
use crate::lightcycle::scene::PooledShown;
use crate::lightcycle::scene::pose::cycle_cell_pose;
use crate::lightcycle::scene::pose::pose_world_position;
use crate::lightcycle::{ActiveRun, LightcycleState, RunEnvironment};
use crate::state::LightcycleSceneRoot;
use bevy::prelude::*;

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
    pub(crate) index: usize,
    pub(crate) phase: f32,
}

/// Tracks which alcove the rider is beside, for the ring's folio panel.
pub(crate) fn update_disc_focus(
    mut state: ResMut<LightcycleState>,
    mut marker: Query<&mut Transform, With<DocumentFocusMarker>>,
) {
    let Some(run) = state.run.as_mut() else {
        return;
    };
    let RunEnvironment::Source {
        layout,
        focused_block,
        ..
    } = &mut run.environment
    else {
        return;
    };
    *focused_block = layout.focused_block(run.sim.cell);
    let Some(index) = *focused_block else {
        return;
    };
    let landmark = layout.blocks[index].landmark;
    if let Ok(mut transform) = marker.single_mut() {
        transform.translation = config::ground_position(landmark.0, landmark.1) + Vec3::Y * 0.08;
    }
}

/// Index into [`LightcycleAssets::disc_accent_materials`].
pub(crate) fn disc_language_index(language: SourceLanguage) -> usize {
    SourceLanguage::ALL
        .iter()
        .position(|candidate| *candidate == language)
        .unwrap_or(0)
}

/// Builds a disc-wars ring: circular floor, ring wall, plinth, hazards, safe
/// pads, close gate, pickups, and the two fighters.
/// Ring wall, corner plinth and close gate. Shared by every source arena, so
/// the asteroid field gets the same containment and the same way out.
pub(crate) fn spawn_ring_shell(
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

pub(crate) fn spawn_disc_arena(
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
                + Vec3::Y * (config::disc::RECOGNIZER_HEIGHT * 0.5),
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

/// Merges same-sized cuboids at `cells` and spawns them as one batched entity.
pub(crate) fn spawn_disc_cube_layer(
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

pub(crate) fn disc_cube(cell: (i32, i32), height: f32, footprint: f32) -> Mesh {
    Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(
            config::ground_position(cell.0, cell.1) + Vec3::Y * (height * 0.5),
        )
        .with_scale(Vec3::new(footprint, height, footprint)),
    )
}

/// Posts and lintel framing the corridor mouth, so the close gate reads as a
/// door in the ring wall.
pub(crate) fn spawn_disc_gate(
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

pub(crate) fn spawn_disc_focus_marker(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    run: &ActiveRun,
) {
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

pub(crate) fn disc_entity_position(cell: (i32, i32)) -> Vec3 {
    config::ground_position(cell.0, cell.1) + Vec3::Y * 0.35
}

/// Spin and bob the waiting pickups so they read as collectible.
pub(crate) fn animate_disc_pickups(
    time: Res<Time>,
    mut pickups: Query<(&DiscPickupEntity, &mut Transform)>,
) {
    let elapsed = time.elapsed_secs();
    for (pickup, mut transform) in &mut pickups {
        let bob = (elapsed * 2.2 + pickup.phase * std::f32::consts::TAU).sin() * 0.12;
        transform.translation.y = 0.45 + bob;
        transform.rotate_y(0.03);
    }
}

pub(crate) fn disc_crash_label(reason: CrashReason) -> String {
    match reason {
        CrashReason::Hazard => "a hazard tile".to_string(),
        CrashReason::Disc => "a disc".to_string(),
        CrashReason::Opponent => "the recognizer".to_string(),
        CrashReason::Wall => "ring wall".to_string(),
        CrashReason::Trail => "your trail".to_string(),
        CrashReason::File => "file".to_string(),
    }
}

/// Keeps the disc, opponent, opponent disc, and pickups glued to the sim.
pub(crate) fn sync_disc_entities(
    state: Res<LightcycleState>,
    mut player_disc: Fighter<PlayerDiscEntity, OpponentEntity, OpponentDiscEntity>,
    mut opponent: Fighter<OpponentEntity, PlayerDiscEntity, OpponentDiscEntity>,
    mut opponent_disc: Fighter<OpponentDiscEntity, PlayerDiscEntity, OpponentEntity>,
    mut pickups: PooledShown<
        DiscPickupEntity,
        FreeOf<PlayerDiscEntity, OpponentEntity, OpponentDiscEntity>,
    >,
) {
    let Some(run) = state.run.as_ref() else {
        return;
    };
    let Some(disc) = run.source_disc() else {
        return;
    };

    if let Ok((mut transform, mut visibility)) = player_disc.single_mut() {
        match disc.player_disc.as_ref() {
            Some(flying) => {
                transform.translation = disc_entity_position(flying.cell);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
    if let Ok((mut transform, mut visibility)) = opponent.single_mut() {
        if disc.opponent.alive {
            // The opponent steps a whole cell at a time. Render it partway to the
            // cell it is walking into so it glides instead of teleporting; while
            // it charges it stands exactly on its cell, so the shot is readable.
            let progress = if disc.opponent.windup > 0.0 {
                0.0
            } else {
                disc.opponent.move_clock.clamp(0.0, 1.0)
            };
            let (dx, dz) = disc.opponent.heading.delta();
            transform.translation =
                config::ground_position(disc.opponent.cell.0, disc.opponent.cell.1)
                    + Vec3::new(dx as f32, 0.0, dz as f32) * (progress * config::GRID_SPACING)
                    + Vec3::Y * (config::disc::RECOGNIZER_HEIGHT * 0.5);
            // Swell while winding up, so its shot is telegraphed.
            let charge =
                (disc.opponent.windup / config::disc::DISC_OPPONENT_WINDUP).clamp(0.0, 1.0);
            transform.scale =
                Vec3::new(1.0 + charge * 0.35, 1.0 - charge * 0.2, 1.0 + charge * 0.35);
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
    if let Ok((mut transform, mut visibility)) = opponent_disc.single_mut() {
        match disc.opponent.disc.as_ref() {
            Some(flying) => {
                transform.translation = disc_entity_position(flying.cell);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
    for (pickup, mut visibility) in &mut pickups {
        *visibility = if disc.taken.contains(&pickup.index) {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}
