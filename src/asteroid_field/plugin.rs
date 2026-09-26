//! The asteroid field: its rock and beam pool.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::asteroid_field::sim::AsteroidFieldSim;
use crate::config;
use crate::disc::language::SourceGame;
use crate::grid_rider::ActiveRun;
use crate::grid_rider::GridRiderState;
use crate::grid_rider::scene::Apart;
use crate::grid_rider::scene::GridRiderAssets;
use crate::grid_rider::scene::Pooled;
use crate::state::GridRiderSceneRoot;
use bevy::prelude::*;

/// One pooled rock in the asteroid field, keyed into `AsteroidFieldSim::rocks`.
#[derive(Component)]
pub(crate) struct RockEntity {
    index: usize,
}

/// One pooled beam in the asteroid field, keyed into `AsteroidFieldSim::beams`.
#[derive(Component)]
pub(crate) struct BeamEntity {
    index: usize,
}

/// Spawns the pooled rock and beam bodies for an asteroid field.
///
/// The sim drives visibility and transforms; the pool is fixed because a rock
/// only ever splits into a bounded number of children.
pub(crate) fn spawn_asteroid_field(
    commands: &mut Commands,
    assets: &GridRiderAssets,
    sim: &AsteroidFieldSim,
) {
    let rock_count = config::asteroid_field::ASTEROID_FIELD_MAX_ROCKS.max(sim.rocks.len());
    for index in 0..rock_count {
        let live = sim.rocks.get(index);
        let radius = live.map_or(1.0, |rock| rock.size.radius());
        commands.spawn((
            GridRiderSceneRoot,
            RockEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.rock_material.clone()),
            Transform::from_xyz(0.0, radius, 0.0).with_scale(Vec3::splat(radius * 2.0)),
            if live.is_some() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }
    for index in 0..config::asteroid_field::ASTEROID_FIELD_MAX_BEAMS {
        let live = sim.beams.get(index);
        commands.spawn((
            GridRiderSceneRoot,
            BeamEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.beam_material.clone()),
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

/// Places the pooled rocks and beams of an asteroid field. Anything past the
/// live end of the sim's vectors is hidden, so splits and pops need no spawning.
pub(crate) fn sync_asteroid_field_entities(
    state: Res<GridRiderState>,
    mut rocks: Pooled<RockEntity, Apart<BeamEntity>>,
    mut beams: Pooled<BeamEntity, Apart<RockEntity>>,
) {
    // Only an actual asteroid field owns rock entities; a disc-wars ring also
    // carries a (never stepped) field sim, so check the game kind too.
    let Some(run) = state.run.as_ref() else {
        return;
    };
    if run.source_game() != Some(SourceGame::AsteroidField) {
        return;
    }
    let Some(sim) = run.source_asteroid_field() else {
        return;
    };

    for (entity, mut transform, mut visibility) in &mut rocks {
        match sim.rocks.get(entity.index) {
            Some(rock) => {
                let radius = rock.size.radius();
                transform.translation = Vec3::new(rock.x, radius, rock.z);
                transform.rotation =
                    Quat::from_rotation_y(rock.angle) * Quat::from_rotation_x(rock.angle * 0.61);
                transform.scale = Vec3::splat(radius * 2.0);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }

    for (entity, mut transform, mut visibility) in &mut beams {
        match sim.beams.get(entity.index) {
            Some(beam) => {
                transform.translation = Vec3::new(beam.x, 0.35, beam.z);
                transform.rotation = Quat::from_rotation_y(-beam.vz.atan2(beam.vx));
                transform.scale = Vec3::new(config::asteroid_field::ASTEROID_FIELD_BEAM_LENGTH, 0.12, 0.12);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}

/// Focus point and ring radius while the field is live. Once it is decided the
/// camera returns to the chase rig so the player can drive out, and a disc-wars
/// ring keeps the chase rig throughout.
pub(crate) fn field_camera_focus(run: &ActiveRun) -> Option<(Vec3, f32)> {
    if !run.asteroid_field_active() {
        return None;
    }
    let sim = run.source_asteroid_field()?;
    Some((Vec3::new(sim.center.0, 0.0, sim.center.1), sim.radius))
}
