//! The Falling Gems well: its gem pool.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::config;
use crate::falling_gems::sim::FallingGemsSim;
use crate::grid_rider::GridRiderState;
use crate::grid_rider::scene::GridRiderAssets;
use crate::grid_rider::scene::OutOfCycle;
use crate::grid_rider::scene::PooledTinted;
use crate::state::GridRiderSceneRoot;
use bevy::prelude::*;

/// One pooled cell of the Falling Gems well, keyed by its row-major index.
#[derive(Component)]
pub(crate) struct GemEntity {
    index: usize,
}

/// Spawns the pooled gem cells of a Falling Gems well.
pub(crate) fn spawn_gem_well(
    commands: &mut Commands,
    assets: &GridRiderAssets,
    sim: &FallingGemsSim,
) {
    let rendered = sim.render_board();
    for (index, colour) in rendered.iter().copied().enumerate() {
        let col = index % config::arcade::FALLING_GEMS_COLS;
        let row = index / config::arcade::FALLING_GEMS_COLS;
        let x = (col as f32 - (config::arcade::FALLING_GEMS_COLS - 1) as f32 * 0.5) * 1.6;
        // Row 0 is the top of the well, so higher rows sit lower on screen.
        // The whole well is lifted above the arena floor.
        let y = ((config::arcade::FALLING_GEMS_ROWS - 1) - row) as f32 * 1.6 + 0.8;
        commands.spawn((
            GridRiderSceneRoot,
            GemEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(
                assets.gem_materials
                    [colour.unwrap_or(0) as usize % config::arcade::FALLING_GEMS_COLORS]
                    .clone(),
            ),
            Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(1.5)),
            if colour.is_some() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }

    // A visible frame marks the playfield: side walls plus a floor bar.
    let board_half = config::arcade::FALLING_GEMS_COLS as f32 * 0.8;
    let wall_x = board_half + 0.55;
    let board_height = config::arcade::FALLING_GEMS_ROWS as f32 * 1.6;
    let wall_scale = Vec3::new(0.4, board_height + 0.6, 0.5);
    for x in [-wall_x, wall_x] {
        commands.spawn((
            GridRiderSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.stealth_wall_material.clone()),
            Transform::from_xyz(x, board_height * 0.5 + 0.3, 0.0).with_scale(wall_scale),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
    commands.spawn((
        GridRiderSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.stealth_wall_material.clone()),
        Transform::from_xyz(0.0, 0.2, 0.0).with_scale(Vec3::new(wall_x * 2.0 + 0.8, 0.4, 0.5)),
        Visibility::Visible,
        Pickable::IGNORE,
    ));
}

/// Places the pooled gem cells of a Falling Gems well.
pub(crate) fn sync_falling_gems_entities(
    state: Res<GridRiderState>,
    assets: Res<GridRiderAssets>,
    mut gems: PooledTinted<GemEntity, OutOfCycle>,
) {
    let Some(sim) = state
        .run
        .as_ref()
        .and_then(|run| run.source_sim::<FallingGemsSim>())
    else {
        return;
    };
    let rendered = sim.render_board();
    for (entity, mut transform, mut visibility, mut material) in &mut gems {
        match rendered.get(entity.index) {
            Some(Some(colour)) => {
                let col = entity.index % config::arcade::FALLING_GEMS_COLS;
                let row = entity.index / config::arcade::FALLING_GEMS_COLS;
                transform.translation = Vec3::new(
                    (col as f32 - (config::arcade::FALLING_GEMS_COLS - 1) as f32 * 0.5) * 1.6,
                    ((config::arcade::FALLING_GEMS_ROWS - 1) - row) as f32 * 1.6 + 0.8,
                    0.0,
                );
                material.0 = assets.gem_materials
                    [*colour as usize % config::arcade::FALLING_GEMS_COLORS]
                    .clone();
                *visibility = Visibility::Visible;
            }
            _ => *visibility = Visibility::Hidden,
        }
    }
}
