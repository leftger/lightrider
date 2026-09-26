//! The Grid Bomber room: its crate and bomb pools.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::config;
use crate::grid_bomber::sim::GridBomberSim;
use crate::grid_rider::GridRiderState;
use crate::grid_rider::scene::Apart;
use crate::grid_rider::scene::GridRiderAssets;
use crate::grid_rider::scene::Pooled;
use crate::grid_rider::scene::PooledShown;
use crate::state::GridRiderSceneRoot;
use bevy::prelude::*;

/// One pooled crate in the Grid Bomber room, keyed by its cell.
#[derive(Component)]
pub(crate) struct BomberCrateEntity {
    cell: (i32, i32),
}

/// One pooled bomb in the Grid Bomber room, keyed into `GridBomberSim::bombs`.
#[derive(Component)]
pub(crate) struct BomberBombEntity {
    index: usize,
}

/// Spawns the Grid Bomber room: crates, bombs, walls and the exit marker.
pub(crate) fn spawn_grid_bomber_room(
    commands: &mut Commands,
    assets: &GridRiderAssets,
    _meshes: &mut Assets<Mesh>,
    sim: &GridBomberSim,
) {
    for row in 0..config::arcade::GRID_BOMBER_ROWS {
        for col in 0..config::arcade::GRID_BOMBER_COLS {
            let cell = (col, row);
            let (x, z) = GridBomberSim::center(cell);
            let is_border = col == 0
                || col == config::arcade::GRID_BOMBER_COLS - 1
                || row == 0
                || row == config::arcade::GRID_BOMBER_ROWS - 1;
            if is_border {
                commands.spawn((
                    GridRiderSceneRoot,
                    Mesh3d(assets.unit_cube.clone()),
                    MeshMaterial3d(assets.stealth_wall_material.clone()),
                    Transform::from_xyz(x, 1.0, z).with_scale(Vec3::splat(2.0)),
                    Visibility::Visible,
                    Pickable::IGNORE,
                ));
            } else if sim.crates.contains(&cell) {
                commands.spawn((
                    GridRiderSceneRoot,
                    BomberCrateEntity { cell },
                    Mesh3d(assets.unit_cube.clone()),
                    MeshMaterial3d(assets.grid_bomber_crate_material.clone()),
                    Transform::from_xyz(x, 0.8, z).with_scale(Vec3::splat(1.8)),
                    Visibility::Visible,
                    Pickable::IGNORE,
                ));
            }
        }
    }
    let (ex, ez) = GridBomberSim::center(sim.exit);
    commands.spawn((
        GridRiderSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.stealth_exit_material.clone()),
        Transform::from_xyz(ex, 0.5, ez).with_scale(Vec3::new(1.9, 0.4, 1.9)),
        Visibility::Visible,
        Pickable::IGNORE,
    ));
    for index in 0..config::arcade::GRID_BOMBER_MAX_BOMBS {
        commands.spawn((
            GridRiderSceneRoot,
            BomberBombEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.grid_bomber_bomb_material.clone()),
            Transform::from_xyz(0.0, 0.6, 0.0),
            Visibility::Hidden,
            Pickable::IGNORE,
        ));
    }
}

/// Places the pooled crates and bombs of a Grid Bomber room.
pub(crate) fn sync_grid_bomber_entities(
    state: Res<GridRiderState>,
    mut crates: PooledShown<BomberCrateEntity, Apart<BomberBombEntity>>,
    mut bombs: Pooled<BomberBombEntity, Apart<BomberCrateEntity>>,
) {
    let Some(sim) = state
        .run
        .as_ref()
        .and_then(|run| run.source_sim::<GridBomberSim>())
    else {
        return;
    };
    for (entity, mut visibility) in &mut crates {
        *visibility = if sim.crates.contains(&entity.cell) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (entity, mut transform, mut visibility) in &mut bombs {
        match sim.bombs.get(entity.index) {
            Some(bomb) => {
                let (x, z) = GridBomberSim::center(bomb.cell);
                transform.translation = Vec3::new(x, 0.6, z);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}
