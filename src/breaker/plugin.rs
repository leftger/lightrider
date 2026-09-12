//! The brick breaker: its ball and brick pool.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::breaker::sim::BreakerSim;
use crate::config;
use crate::lightcycle::LightcycleState;
use crate::plugins::lightcycle::Apart;
use crate::plugins::lightcycle::CharacterEntity;
use crate::plugins::lightcycle::LightcycleAssets;
use crate::plugins::lightcycle::PooledShown;
use crate::state::LightcycleSceneRoot;
use bevy::prelude::*;

/// The breaker's ball.
#[derive(Component)]
pub(crate) struct BallEntity;

/// One brick of a breaker wall, keyed into `BreakerSim::bricks`.
#[derive(Component)]
pub(crate) struct BrickEntity {
    index: usize,
}

/// Spawns a breaker court: the side and ceiling walls, the bricks, and the ball.
///
/// The bike itself is the paddle, so it is left to `update_cycle_transform`.
pub(crate) fn spawn_breaker_court(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    level: &BreakerSim,
) {
    let (width, height) = level.court;
    let thickness = 0.8;
    let depth = config::PLATFORMER_DEPTH;
    // Side walls and ceiling.
    for (x, y, w, h) in [
        (
            -width * 0.5 - thickness * 0.5,
            height * 0.5,
            thickness,
            height,
        ),
        (
            width * 0.5 + thickness * 0.5,
            height * 0.5,
            thickness,
            height,
        ),
        (
            0.0,
            height + thickness * 0.5,
            width + thickness * 2.0,
            thickness,
        ),
    ] {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.court_material.clone()),
            Transform::from_translation(Vec3::new(x, y, -depth * 0.5))
                .with_scale(Vec3::new(w, h, depth)),
            Pickable::IGNORE,
        ));
    }

    for (index, brick) in level.bricks.iter().enumerate() {
        let (bx, by, bw, bh) = level.brick_box(brick);
        commands.spawn((
            LightcycleSceneRoot,
            BrickEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.brick_material.clone()),
            Transform::from_translation(Vec3::new(bx + bw * 0.5, by + bh * 0.5, 0.0))
                .with_scale(Vec3::new(bw, bh, depth * 0.6)),
            if brick.alive {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }

    commands.spawn((
        LightcycleSceneRoot,
        BallEntity,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.ball_material.clone()),
        Transform::from_translation(Vec3::new(level.ball.x, level.ball.y, 0.0))
            .with_scale(Vec3::splat(config::BREAKER_BALL_RADIUS * 2.0)),
        Pickable::IGNORE,
    ));
}

/// Keeps the breaker's ball and bricks glued to its sim.
pub(crate) fn sync_breaker_entities(
    state: Res<LightcycleState>,
    mut ball: Query<&mut Transform, (With<BallEntity>, Without<BrickEntity>)>,
    mut bricks: PooledShown<BrickEntity, Apart<BallEntity, CharacterEntity>>,
) {
    let Some(level) = state
        .run
        .as_ref()
        .and_then(|run| run.source_sim::<BreakerSim>())
    else {
        return;
    };
    for mut transform in &mut ball {
        transform.translation = Vec3::new(level.ball.x, level.ball.y, 0.0);
    }
    for (brick, mut visibility) in &mut bricks {
        *visibility = match level.bricks.get(brick.index) {
            Some(brick) if brick.alive => Visibility::Visible,
            _ => Visibility::Hidden,
        };
    }
}
