//! The Q*bert pyramid: its cube and enemy pools.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::config;
use crate::lightcycle::LightcycleState;
use crate::plugins::lightcycle::LightcycleAssets;
use crate::plugins::lightcycle::Pooled;
use crate::plugins::lightcycle::PooledPosedTinted;
use crate::qbert::sim::QbertSim;
use crate::state::LightcycleSceneRoot;
use bevy::prelude::*;

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

/// Spawns the Q*bert pyramid cubes and its pooled enemies.
pub(crate) fn spawn_qbert_pyramid(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    sim: &QbertSim,
) {
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

/// Relights Q*bert cubes and places the pooled enemies.
pub(crate) fn sync_qbert_entities(
    state: Res<LightcycleState>,
    assets: Res<LightcycleAssets>,
    mut cubes: PooledPosedTinted<QbertCubeEntity, Without<QbertEnemyEntity>>,
    mut enemies: Pooled<QbertEnemyEntity, Without<QbertCubeEntity>>,
) {
    let Some(sim) = state.run.as_ref().and_then(|run| run.source_qbert()) else {
        return;
    };
    for (entity, mut transform, mut material) in &mut cubes {
        let lit = sim.lit.get(QbertSim::cube_index(entity.row, entity.index));
        let (x, z) = QbertSim::cube_position(entity.row, entity.index);
        let y =
            (config::QBERT_ROWS as f32 - 1.0 - entity.row as f32) * config::QBERT_CUBE_HEIGHT * 0.5;
        transform.translation = Vec3::new(x, y, z);
        material.0 = if lit == Some(&true) {
            assets.qbert_cube_lit.clone()
        } else {
            assets.qbert_cube_dim.clone()
        };
    }
    for (entity, mut transform, mut visibility) in &mut enemies {
        match sim.enemies.get(entity.index) {
            Some(enemy) => {
                let (x, z) = QbertSim::cube_position(enemy.row, enemy.index);
                let y = (config::QBERT_ROWS as f32 - 1.0 - enemy.row as f32)
                    * config::QBERT_CUBE_HEIGHT
                    * 0.5
                    + config::QBERT_CUBE_HEIGHT * 0.8;
                transform.translation = Vec3::new(x, y, z);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}
