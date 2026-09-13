//! The Galaga field: its bug and beam pool.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::config;
use crate::galaga::sim::GalagaSim;
use crate::lightcycle::LightcycleState;
use crate::lightcycle::scene::Apart;
use crate::lightcycle::scene::LightcycleAssets;
use crate::lightcycle::scene::Pooled;
use crate::state::LightcycleSceneRoot;
use bevy::prelude::*;

/// One pooled bug in the Galaga field, keyed into `GalagaSim::bugs`.
#[derive(Component)]
pub(crate) struct BugEntity {
    index: usize,
}

/// One pooled beam in the Galaga field, keyed into `GalagaSim::beams`.
#[derive(Component)]
pub(crate) struct GalagaBeamEntity {
    index: usize,
}

/// Spawns the pooled bug and beam bodies for a Galaga field.
///
/// The formation is a fixed grid, so the bug pool never grows; the sim drives
/// transforms and visibility.
pub(crate) fn spawn_galaga_field(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    sim: &GalagaSim,
) {
    for (index, bug) in sim.bugs.iter().enumerate() {
        commands.spawn((
            LightcycleSceneRoot,
            BugEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.galaga_bug_material.clone()),
            Transform::from_xyz(bug.x, config::galaga::GALAGA_BUG_HEIGHT * 0.5, bug.z).with_scale(
                Vec3::new(
                    config::galaga::GALAGA_BUG_RADIUS * 2.0,
                    config::galaga::GALAGA_BUG_HEIGHT,
                    config::galaga::GALAGA_BUG_RADIUS * 2.0,
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
    for index in 0..config::galaga::GALAGA_MAX_BEAMS {
        let live = sim.beams.get(index);
        commands.spawn((
            LightcycleSceneRoot,
            GalagaBeamEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.galaga_beam_material.clone()),
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

/// Places the pooled bugs and beams of a Galaga field. Dead bugs and spent
/// beams are hidden rather than despawned, so the pool never needs to grow.
pub(crate) fn sync_galaga_entities(
    state: Res<LightcycleState>,
    mut bugs: Pooled<BugEntity, Apart<GalagaBeamEntity>>,
    mut beams: Pooled<GalagaBeamEntity, Apart<BugEntity>>,
) {
    let Some(sim) = state
        .run
        .as_ref()
        .and_then(|run| run.source_sim::<GalagaSim>())
    else {
        return;
    };

    for (entity, mut transform, mut visibility) in &mut bugs {
        match sim.bugs.get(entity.index) {
            Some(bug) if bug.alive => {
                transform.translation =
                    Vec3::new(bug.x, config::galaga::GALAGA_BUG_HEIGHT * 0.5, bug.z);
                *visibility = Visibility::Visible;
            }
            _ => *visibility = Visibility::Hidden,
        }
    }

    for (entity, mut transform, mut visibility) in &mut beams {
        match sim.beams.get(entity.index) {
            Some(beam) => {
                transform.translation = Vec3::new(beam.x, 0.35, beam.z);
                transform.scale = Vec3::new(0.12, 0.12, config::galaga::GALAGA_BEAM_LENGTH);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}
