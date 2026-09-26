//! The Byte Muncher maze: its dot and ghost pools.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::byte_muncher::sim::ByteMuncherSim;
use crate::config;
use crate::grid_rider::GridRiderState;
use crate::grid_rider::scene::Apart;
use crate::grid_rider::scene::GridRiderAssets;
use crate::grid_rider::scene::Pooled;
use crate::grid_rider::scene::PooledShown;
use crate::state::GridRiderSceneRoot;
use bevy::prelude::*;

/// One pooled dot in the Byte Muncher maze, keyed by its cell.
#[derive(Component)]
pub(crate) struct DotEntity {
    cell: (i32, i32),
}

/// One pooled ghost in the Byte Muncher maze, keyed into `ByteMuncherSim::ghosts`.
#[derive(Component)]
pub(crate) struct GhostEntity {
    index: usize,
}

/// Spawns the Byte Muncher maze: static walls plus pooled dots and ghosts.
pub(crate) fn spawn_byte_muncher_maze(
    commands: &mut Commands,
    assets: &GridRiderAssets,
    _meshes: &mut Assets<Mesh>,
    sim: &ByteMuncherSim,
) {
    for row in 0..config::arcade::MUNCHER_ROWS {
        for col in 0..config::arcade::MUNCHER_COLS {
            if !ByteMuncherSim::solid((col, row)) {
                continue;
            }
            let (x, z) = ByteMuncherSim::center((col, row));
            commands.spawn((
                GridRiderSceneRoot,
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
            GridRiderSceneRoot,
            GhostEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.swarm_bug_material.clone()),
            Transform::from_xyz(ghost.x, 0.9, ghost.z).with_scale(Vec3::splat(1.5)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
    for &cell in &sim.dots {
        let (x, z) = ByteMuncherSim::center(cell);
        commands.spawn((
            GridRiderSceneRoot,
            DotEntity { cell },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.snake_food_material.clone()),
            Transform::from_xyz(x, 0.35, z).with_scale(Vec3::splat(0.55)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
}

/// Places the pooled dots and ghosts of a Byte Muncher maze.
pub(crate) fn sync_byte_muncher_entities(
    state: Res<GridRiderState>,
    mut dots: PooledShown<DotEntity, Apart<GhostEntity>>,
    mut ghosts: Pooled<GhostEntity, Apart<DotEntity>>,
) {
    let Some(sim) = state
        .run
        .as_ref()
        .and_then(|run| run.source_sim::<ByteMuncherSim>())
    else {
        return;
    };
    for (entity, mut visibility) in &mut dots {
        *visibility = if sim.dots.contains(&entity.cell) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (entity, mut transform, mut visibility) in &mut ghosts {
        match sim.ghosts.get(entity.index) {
            Some(ghost) => {
                transform.translation = Vec3::new(ghost.x, 0.9, ghost.z);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}
