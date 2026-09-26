//! The Grid Hopper highway: its obstacle pool.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::grid_hopper::sim::GridHopperSim;
use crate::grid_rider::GridRiderState;
use crate::grid_rider::scene::GridRiderAssets;
use crate::grid_rider::scene::OutOfCycle;
use crate::grid_rider::scene::Pooled;
use crate::state::GridRiderSceneRoot;
use bevy::prelude::*;

/// One pooled obstacle cube on the Grid Hopper highway.
#[derive(Component)]
pub(crate) struct FrogObstacleEntity {
    index: usize,
}

/// Spawns the pooled obstacle cubes of a Grid Hopper highway.
pub(crate) fn spawn_grid_hopper_highway(
    commands: &mut Commands,
    assets: &GridRiderAssets,
    sim: &GridHopperSim,
) {
    let cells = sim.obstacle_cells();
    for (index, &cell) in cells.iter().enumerate() {
        let (x, z) = GridHopperSim::center(cell);
        commands.spawn((
            GridRiderSceneRoot,
            FrogObstacleEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.swarm_bug_material.clone()),
            Transform::from_xyz(x, 0.6, z).with_scale(Vec3::splat(1.9)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
}

/// Places the pooled obstacle cubes of a Grid Hopper highway.
pub(crate) fn sync_grid_hopper_entities(
    state: Res<GridRiderState>,
    mut obstacles: Pooled<FrogObstacleEntity, OutOfCycle>,
) {
    let Some(sim) = state
        .run
        .as_ref()
        .and_then(|run| run.source_sim::<GridHopperSim>())
    else {
        return;
    };
    let cells = sim.obstacle_cells();
    for (entity, mut transform, mut visibility) in &mut obstacles {
        match cells.get(entity.index) {
            Some(&cell) => {
                let (x, z) = GridHopperSim::center(cell);
                transform.translation = Vec3::new(x, 0.6, z);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}
