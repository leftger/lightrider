//! The snake ring: its power-ups and exit lock.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::config;
use crate::lightcycle::LightcycleState;
use crate::lightcycle::logic::Arena;
use crate::lightcycle::scene::LightcycleAssets;
use crate::snake::sim::SnakeSim;
use crate::state::LightcycleSceneRoot;
use bevy::prelude::*;

/// One power-up on a snake ring, keyed into `SnakeSim::food`.
#[derive(Component)]
pub(crate) struct SnakeFoodEntity {
    index: usize,
}

/// The bar sealing a snake ring's exit until enough power-ups are collected.
#[derive(Component)]
pub(crate) struct SnakeGateLock;

/// Spawns a snake ring's power-ups and the bar sealing its exit.
///
/// The power-ups are a fixed pool keyed by index; the sim marks them eaten.
pub(crate) fn spawn_snake_field(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    arena: &Arena,
    snake: &SnakeSim,
) {
    for (index, food) in snake.food.iter().enumerate() {
        commands.spawn((
            LightcycleSceneRoot,
            SnakeFoodEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.snake_food_material.clone()),
            Transform::from_translation(
                config::ground_position(food.cell.0, food.cell.1) + Vec3::Y * 0.5,
            )
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_4))
            .with_scale(Vec3::splat(config::snake::SNAKE_FOOD_SIZE)),
            if food.eaten {
                Visibility::Hidden
            } else {
                Visibility::Visible
            },
            Pickable::IGNORE,
        ));
    }

    // A solid bar in the gate opening until the exit opens. The physical lock is
    // the classification; this is what the rider sees.
    let Some(portal) = arena.parent_portal.as_ref() else {
        return;
    };
    let gate = portal.to;
    commands.spawn((
        LightcycleSceneRoot,
        SnakeGateLock,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.snake_lock_material.clone()),
        Transform::from_translation(config::ground_position(gate.0, gate.1) + Vec3::Y * 0.9)
            .with_scale(Vec3::new(
                config::GRID_SPACING,
                config::lightcycle::LIGHTCYCLE_WALL_HEIGHT * 0.9,
                config::GRID_SPACING,
            )),
        if snake.exit_open {
            Visibility::Hidden
        } else {
            Visibility::Visible
        },
        Pickable::IGNORE,
    ));
}

/// Hides a snake ring's power-ups once eaten, and its exit bar once unlocked.
pub(crate) fn sync_snake_entities(
    state: Res<LightcycleState>,
    mut food: Query<(&SnakeFoodEntity, &mut Visibility), Without<SnakeGateLock>>,
    mut lock: Query<&mut Visibility, (With<SnakeGateLock>, Without<SnakeFoodEntity>)>,
) {
    let Some(snake) = state.run.as_ref().and_then(|run| run.source_snake()) else {
        return;
    };

    for (item, mut visibility) in &mut food {
        *visibility = match snake.food.get(item.index) {
            Some(food) if !food.eaten => Visibility::Visible,
            _ => Visibility::Hidden,
        };
    }

    if let Ok(mut visibility) = lock.single_mut() {
        *visibility = if snake.exit_open {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}
