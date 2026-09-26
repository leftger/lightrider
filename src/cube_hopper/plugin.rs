//! The Cube Hopper pyramid: its cube and enemy pools.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::config;
use crate::cube_hopper::sim::CubeHopperSim;
use crate::grid_rider::GridRiderState;
use crate::grid_rider::scene::GridRiderAssets;
use crate::grid_rider::scene::Pooled;
use crate::grid_rider::scene::PooledPosedTinted;
use crate::state::GridRiderSceneRoot;
use bevy::prelude::*;

/// One cube of the Cube Hopper pyramid, keyed by its row/index pair.
#[derive(Component)]
pub(crate) struct CubeHopperCubeEntity {
    row: usize,
    index: usize,
}

/// One pooled enemy on the Cube Hopper pyramid, keyed into `CubeHopperSim::enemies`.
#[derive(Component)]
pub(crate) struct CubeHopperEnemyEntity {
    index: usize,
}

/// Spawns the Cube Hopper pyramid cubes and its pooled enemies.
pub(crate) fn spawn_cube_hopper_pyramid(
    commands: &mut Commands,
    assets: &GridRiderAssets,
    sim: &CubeHopperSim,
) {
    for row in 0..config::arcade::CUBE_HOPPER_ROWS {
        for index in 0..=row {
            let (x, z) = CubeHopperSim::cube_position(row, index);
            let lit = sim.lit[CubeHopperSim::cube_index(row, index)];
            let y = (config::arcade::CUBE_HOPPER_ROWS as f32 - 1.0 - row as f32)
                * config::arcade::CUBE_HOPPER_CUBE_HEIGHT
                * 0.5;
            commands.spawn((
                GridRiderSceneRoot,
                CubeHopperCubeEntity { row, index },
                Mesh3d(assets.unit_cube.clone()),
                MeshMaterial3d(if lit {
                    assets.cube_lit.clone()
                } else {
                    assets.cube_dim.clone()
                }),
                Transform::from_xyz(x, y, z).with_scale(Vec3::new(
                    config::arcade::CUBE_HOPPER_CUBE_SPACING * 0.9,
                    config::arcade::CUBE_HOPPER_CUBE_HEIGHT,
                    config::arcade::CUBE_HOPPER_CUBE_SPACING * 0.9,
                )),
                Visibility::Visible,
                Pickable::IGNORE,
            ));
        }
    }
    for (index, enemy) in sim.enemies.iter().enumerate() {
        let (x, z) = CubeHopperSim::cube_position(enemy.row, enemy.index);
        let y = (config::arcade::CUBE_HOPPER_ROWS as f32 - 1.0 - enemy.row as f32)
            * config::arcade::CUBE_HOPPER_CUBE_HEIGHT
            * 0.5
            + config::arcade::CUBE_HOPPER_CUBE_HEIGHT * 0.8;
        commands.spawn((
            GridRiderSceneRoot,
            CubeHopperEnemyEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.cube_enemy_material.clone()),
            Transform::from_xyz(x, y, z).with_scale(Vec3::splat(1.4)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
}

/// Relights Cube Hopper cubes and places the pooled enemies.
pub(crate) fn sync_cube_hopper_entities(
    state: Res<GridRiderState>,
    assets: Res<GridRiderAssets>,
    mut cubes: PooledPosedTinted<CubeHopperCubeEntity, Without<CubeHopperEnemyEntity>>,
    mut enemies: Pooled<CubeHopperEnemyEntity, Without<CubeHopperCubeEntity>>,
) {
    let Some(sim) = state
        .run
        .as_ref()
        .and_then(|run| run.source_sim::<CubeHopperSim>())
    else {
        return;
    };
    for (entity, mut transform, mut material) in &mut cubes {
        let lit = sim
            .lit
            .get(CubeHopperSim::cube_index(entity.row, entity.index));
        let (x, z) = CubeHopperSim::cube_position(entity.row, entity.index);
        let y = (config::arcade::CUBE_HOPPER_ROWS as f32 - 1.0 - entity.row as f32)
            * config::arcade::CUBE_HOPPER_CUBE_HEIGHT
            * 0.5;
        transform.translation = Vec3::new(x, y, z);
        material.0 = if lit == Some(&true) {
            assets.cube_lit.clone()
        } else {
            assets.cube_dim.clone()
        };
    }
    for (entity, mut transform, mut visibility) in &mut enemies {
        match sim.enemies.get(entity.index) {
            Some(enemy) => {
                let (x, z) = CubeHopperSim::cube_position(enemy.row, enemy.index);
                let y = (config::arcade::CUBE_HOPPER_ROWS as f32 - 1.0 - enemy.row as f32)
                    * config::arcade::CUBE_HOPPER_CUBE_HEIGHT
                    * 0.5
                    + config::arcade::CUBE_HOPPER_CUBE_HEIGHT * 0.8;
                transform.translation = Vec3::new(x, y, z);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}
