//! The Frogger highway: its obstacle pool.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::frogger::sim::FroggerSim;
use crate::lightcycle::LightcycleState;
use crate::lightcycle::scene::LightcycleAssets;
use crate::lightcycle::scene::OutOfCycle;
use crate::lightcycle::scene::Pooled;
use crate::state::LightcycleSceneRoot;
use bevy::prelude::*;

/// One pooled obstacle cube on the Frogger highway.
#[derive(Component)]
pub(crate) struct FrogObstacleEntity {
    index: usize,
}

/// Spawns the pooled obstacle cubes of a Frogger highway.
pub(crate) fn spawn_frogger_highway(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    sim: &FroggerSim,
) {
    let cells = sim.obstacle_cells();
    for (index, &cell) in cells.iter().enumerate() {
        let (x, z) = FroggerSim::center(cell);
        commands.spawn((
            LightcycleSceneRoot,
            FrogObstacleEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.galaga_bug_material.clone()),
            Transform::from_xyz(x, 0.6, z).with_scale(Vec3::splat(1.9)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
}

/// Places the pooled obstacle cubes of a Frogger highway.
pub(crate) fn sync_frogger_entities(
    state: Res<LightcycleState>,
    mut obstacles: Pooled<FrogObstacleEntity, OutOfCycle>,
) {
    let Some(sim) = state
        .run
        .as_ref()
        .and_then(|run| run.source_sim::<FroggerSim>())
    else {
        return;
    };
    let cells = sim.obstacle_cells();
    for (entity, mut transform, mut visibility) in &mut obstacles {
        match cells.get(entity.index) {
            Some(&cell) => {
                let (x, z) = FroggerSim::center(cell);
                transform.translation = Vec3::new(x, 0.6, z);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}
