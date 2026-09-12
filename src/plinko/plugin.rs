//! The Plinko board: its ball pool.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::config;
use crate::lightcycle::LightcycleState;
use crate::lightcycle::scene::CycleEntity;
use crate::lightcycle::scene::LightcycleAssets;
use crate::plinko::sim::PlinkoSim;
use crate::state::LightcycleSceneRoot;
use bevy::prelude::*;

/// One pooled ball on the Plinko board, keyed into `PlinkoSim::balls`.
#[derive(Component)]
pub(crate) struct PlinkoBallEntity {
    index: usize,
}

/// Spawns the Plinko board: static pins and buckets plus pooled balls.
pub(crate) fn spawn_plinko_board(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    _meshes: &mut Assets<Mesh>,
    sim: &PlinkoSim,
) {
    // A dark backdrop behind the board makes the pins and balls read clearly.
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.qbert_cube_dim.clone()),
        Transform::from_xyz(0.0, 0.0, -0.35).with_scale(Vec3::new(
            config::arcade::PLINKO_WIDTH + 1.5,
            config::arcade::PLINKO_HEIGHT + 1.5,
            0.2,
        )),
        Visibility::Visible,
        Pickable::IGNORE,
    ));
    // The drop rail along the top, where the cycle slides to aim.
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.stealth_wall_material.clone()),
        Transform::from_xyz(0.0, config::arcade::PLINKO_HEIGHT * 0.5 + 0.35, 0.0)
            .with_scale(Vec3::new(config::arcade::PLINKO_WIDTH + 1.0, 0.6, 0.6)),
        Visibility::Visible,
        Pickable::IGNORE,
    ));
    for pin in &sim.pins {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.plinko_pin_material.clone()),
            Transform::from_xyz(pin.x, pin.y, 0.0).with_scale(Vec3::splat(0.62)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
    for slot in 0..8 {
        let x = (slot as f32 - 3.5) * 2.0;
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.disc_accent_materials[slot].clone()),
            Transform::from_xyz(x, -config::arcade::PLINKO_HEIGHT * 0.5 - 1.0, 0.0)
                .with_scale(Vec3::new(1.9, 1.3, 0.9)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
    for index in 0..config::arcade::PLINKO_BALLS {
        commands.spawn((
            LightcycleSceneRoot,
            PlinkoBallEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.plinko_ball_material.clone()),
            Transform::from_xyz(0.0, config::arcade::PLINKO_HEIGHT * 0.5, 0.0)
                .with_scale(Vec3::splat(1.1)),
            Visibility::Hidden,
            Pickable::IGNORE,
        ));
    }
}

/// Places the pooled balls of a Plinko board.
pub(crate) fn sync_plinko_entities(
    state: Res<LightcycleState>,
    mut balls: Query<(&PlinkoBallEntity, &mut Transform, &mut Visibility), Without<CycleEntity>>,
) {
    let Some(sim) = state
        .run
        .as_ref()
        .and_then(|run| run.source_sim::<PlinkoSim>())
    else {
        return;
    };
    for (entity, mut transform, mut visibility) in &mut balls {
        match sim.balls.get(entity.index) {
            Some(ball) => {
                transform.translation = Vec3::new(ball.x, ball.y, 0.0);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}
