//! The Swarm Shooter field: its bug and beam pool.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::config;
use crate::swarm_shooter::sim::SwarmShooterSim;
use crate::grid_rider::GridRiderState;
use crate::grid_rider::scene::Apart;
use crate::grid_rider::scene::GridRiderAssets;
use crate::grid_rider::scene::Pooled;
use crate::state::GridRiderSceneRoot;
use bevy::prelude::*;

/// One pooled bug in the Swarm Shooter field, keyed into `SwarmShooterSim::bugs`.
#[derive(Component)]
pub(crate) struct BugEntity {
    index: usize,
}

/// One pooled beam in the Swarm Shooter field, keyed into `SwarmShooterSim::beams`.
#[derive(Component)]
pub(crate) struct SwarmBeamEntity {
    index: usize,
}

/// Spawns the pooled bug and beam bodies for a Swarm Shooter field.
///
/// The formation is a fixed grid, so the bug pool never grows; the sim drives
/// transforms and visibility.
pub(crate) fn spawn_swarm_shooter_field(
    commands: &mut Commands,
    assets: &GridRiderAssets,
    sim: &SwarmShooterSim,
) {
    for (index, bug) in sim.bugs.iter().enumerate() {
        commands.spawn((
            GridRiderSceneRoot,
            BugEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.swarm_bug_material.clone()),
            Transform::from_xyz(bug.x, config::swarm_shooter::SWARM_SHOOTER_BUG_HEIGHT * 0.5, bug.z).with_scale(
                Vec3::new(
                    config::swarm_shooter::SWARM_SHOOTER_BUG_RADIUS * 2.0,
                    config::swarm_shooter::SWARM_SHOOTER_BUG_HEIGHT,
                    config::swarm_shooter::SWARM_SHOOTER_BUG_RADIUS * 2.0,
                ),
            ),
            if bug.alive {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }
    for index in 0..config::swarm_shooter::SWARM_SHOOTER_MAX_BEAMS {
        let live = sim.beams.get(index);
        commands.spawn((
            GridRiderSceneRoot,
            SwarmBeamEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.swarm_beam_material.clone()),
            Transform::from_xyz(0.0, 0.35, 0.0),
            if live.is_some() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }
}

/// Places the pooled bugs and beams of a Swarm Shooter field. Dead bugs and spent
/// beams are hidden rather than despawned, so the pool never needs to grow.
pub(crate) fn sync_swarm_shooter_entities(
    state: Res<GridRiderState>,
    mut bugs: Pooled<BugEntity, Apart<SwarmBeamEntity>>,
    mut beams: Pooled<SwarmBeamEntity, Apart<BugEntity>>,
) {
    let Some(sim) = state
        .run
        .as_ref()
        .and_then(|run| run.source_sim::<SwarmShooterSim>())
    else {
        return;
    };

    for (entity, mut transform, mut visibility) in &mut bugs {
        match sim.bugs.get(entity.index) {
            Some(bug) if bug.alive => {
                transform.translation =
                    Vec3::new(bug.x, config::swarm_shooter::SWARM_SHOOTER_BUG_HEIGHT * 0.5, bug.z);
                *visibility = Visibility::Visible;
            }
            _ => *visibility = Visibility::Hidden,
        }
    }

    for (entity, mut transform, mut visibility) in &mut beams {
        match sim.beams.get(entity.index) {
            Some(beam) => {
                transform.translation = Vec3::new(beam.x, 0.35, beam.z);
                transform.scale = Vec3::new(0.12, 0.12, config::swarm_shooter::SWARM_SHOOTER_BEAM_LENGTH);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}
