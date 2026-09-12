//! The asteroid field: its rock and beam pool.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::asteroids::sim::AsteroidsSim;
use crate::config;
use crate::disc::language::SourceGame;
use crate::lightcycle::LightcycleState;
use crate::plugins::lightcycle::Apart;
use crate::plugins::lightcycle::LightcycleAssets;
use crate::plugins::lightcycle::Pooled;
use crate::state::LightcycleSceneRoot;
use bevy::prelude::*;

/// One pooled rock in the asteroid field, keyed into `AsteroidsSim::rocks`.
#[derive(Component)]
pub(crate) struct RockEntity {
    index: usize,
}

/// One pooled beam in the asteroid field, keyed into `AsteroidsSim::beams`.
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
    assets: &LightcycleAssets,
    sim: &AsteroidsSim,
) {
    let rock_count = config::ASTEROIDS_MAX_ROCKS.max(sim.rocks.len());
    for index in 0..rock_count {
        let live = sim.rocks.get(index);
        let radius = live.map_or(1.0, |rock| rock.size.radius());
        commands.spawn((
            LightcycleSceneRoot,
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
    for index in 0..config::ASTEROIDS_MAX_BEAMS {
        let live = sim.beams.get(index);
        commands.spawn((
            LightcycleSceneRoot,
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
pub(crate) fn sync_asteroid_entities(
    state: Res<LightcycleState>,
    mut rocks: Pooled<RockEntity, Apart<BeamEntity>>,
    mut beams: Pooled<BeamEntity, Apart<RockEntity>>,
) {
    // Only an actual asteroid field owns rock entities; a disc-wars ring also
    // carries a (never stepped) field sim, so check the game kind too.
    let Some(run) = state.run.as_ref() else {
        return;
    };
    if run.source_game() != Some(SourceGame::Asteroids) {
        return;
    }
    let Some(sim) = run.source_asteroids() else {
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
                transform.scale = Vec3::new(config::ASTEROIDS_BEAM_LENGTH, 0.12, 0.12);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}
