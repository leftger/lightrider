//! The Bomberman room: its crate and bomb pools.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::bomberman::sim::BomberSim;
use crate::config;
use crate::lightcycle::LightcycleState;
use crate::lightcycle::scene::Apart;
use crate::lightcycle::scene::LightcycleAssets;
use crate::lightcycle::scene::Pooled;
use crate::lightcycle::scene::PooledShown;
use crate::state::LightcycleSceneRoot;
use bevy::prelude::*;

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

/// Spawns the Bomberman room: crates, bombs, walls and the exit marker.
pub(crate) fn spawn_bomber_room(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    _meshes: &mut Assets<Mesh>,
    sim: &BomberSim,
) {
    for row in 0..config::arcade::BOMBER_ROWS {
        for col in 0..config::arcade::BOMBER_COLS {
            let cell = (col, row);
            let (x, z) = BomberSim::center(cell);
            let is_border = col == 0
                || col == config::arcade::BOMBER_COLS - 1
                || row == 0
                || row == config::arcade::BOMBER_ROWS - 1;
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
    for index in 0..config::arcade::BOMBER_MAX_BOMBS {
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

/// Places the pooled crates and bombs of a Bomberman room.
pub(crate) fn sync_bomberman_entities(
    state: Res<LightcycleState>,
    mut crates: PooledShown<BomberCrateEntity, Apart<BomberBombEntity>>,
    mut bombs: Pooled<BomberBombEntity, Apart<BomberCrateEntity>>,
) {
    let Some(sim) = state
        .run
        .as_ref()
        .and_then(|run| run.source_sim::<BomberSim>())
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
                let (x, z) = BomberSim::center(bomb.cell);
                transform.translation = Vec3::new(x, 0.6, z);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}
