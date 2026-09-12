//! The Pac-Man maze: its dot and ghost pools.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::config;
use crate::lightcycle::LightcycleState;
use crate::pacman::sim::PacSim;
use crate::plugins::lightcycle::Apart;
use crate::plugins::lightcycle::LightcycleAssets;
use crate::plugins::lightcycle::Pooled;
use crate::plugins::lightcycle::PooledShown;
use crate::state::LightcycleSceneRoot;
use bevy::prelude::*;

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

/// Spawns the Pac-Man maze: static walls plus pooled dots and ghosts.
pub(crate) fn spawn_pac_maze(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    _meshes: &mut Assets<Mesh>,
    sim: &PacSim,
) {
    for row in 0..config::arcade::PAC_ROWS {
        for col in 0..config::arcade::PAC_COLS {
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

/// Places the pooled dots and ghosts of a Pac-Man maze.
pub(crate) fn sync_pacman_entities(
    state: Res<LightcycleState>,
    mut dots: PooledShown<DotEntity, Apart<GhostEntity>>,
    mut ghosts: Pooled<GhostEntity, Apart<DotEntity>>,
) {
    let Some(sim) = state
        .run
        .as_ref()
        .and_then(|run| run.source_sim::<PacSim>())
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
