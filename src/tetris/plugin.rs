//! The Tetris board: its block pool.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::config;
use crate::lightcycle::LightcycleState;
use crate::plugins::lightcycle::LightcycleAssets;
use crate::plugins::lightcycle::OutOfCycle;
use crate::plugins::lightcycle::PooledTinted;
use crate::state::LightcycleSceneRoot;
use crate::tetris::sim::TetrisSim;
use bevy::prelude::*;

/// One pooled cell of the Tetris board, keyed by its row-major index.
#[derive(Component)]
pub(crate) struct BlockEntity {
    index: usize,
}

/// Spawns the pooled block cells of a Tetris board.
pub(crate) fn spawn_tetris_board(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    sim: &TetrisSim,
) {
    let rendered = sim.render_board();
    for (index, colour) in rendered.iter().copied().enumerate() {
        let col = index % config::TETRIS_COLS;
        let row = index / config::TETRIS_COLS;
        let x = (col as f32 - (config::TETRIS_COLS - 1) as f32 * 0.5) * 1.2;
        // Row 0 is the top of the board, so higher rows sit lower on screen.
        // The whole board is lifted above the arena floor.
        let y = ((config::TETRIS_ROWS - 1) - row) as f32 * 1.2 + 0.6;
        commands.spawn((
            LightcycleSceneRoot,
            BlockEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.tetris_materials[colour.unwrap_or(0) as usize % 7].clone()),
            Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(1.15)),
            if colour.is_some() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }

    // A visible frame marks the playfield: side walls plus a floor bar.
    let board_half = config::TETRIS_COLS as f32 * 0.6;
    let wall_x = board_half + 0.55;
    let board_height = config::TETRIS_ROWS as f32 * 1.2;
    let wall_scale = Vec3::new(0.4, board_height + 0.6, 0.5);
    for x in [-wall_x, wall_x] {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.stealth_wall_material.clone()),
            Transform::from_xyz(x, board_height * 0.5 + 0.3, 0.0).with_scale(wall_scale),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.stealth_wall_material.clone()),
        Transform::from_xyz(0.0, 0.15, 0.0).with_scale(Vec3::new(wall_x * 2.0 + 0.8, 0.3, 0.5)),
        Visibility::Visible,
        Pickable::IGNORE,
    ));
}

/// Places the pooled block cells of a Tetris board.
pub(crate) fn sync_tetris_entities(
    state: Res<LightcycleState>,
    assets: Res<LightcycleAssets>,
    mut blocks: PooledTinted<BlockEntity, OutOfCycle>,
) {
    let Some(sim) = state.run.as_ref().and_then(|run| run.source_tetris()) else {
        return;
    };
    let rendered = sim.render_board();
    for (entity, mut transform, mut visibility, mut material) in &mut blocks {
        match rendered.get(entity.index) {
            Some(Some(colour)) => {
                let col = entity.index % config::TETRIS_COLS;
                let row = entity.index / config::TETRIS_COLS;
                transform.translation = Vec3::new(
                    (col as f32 - (config::TETRIS_COLS - 1) as f32 * 0.5) * 1.2,
                    ((config::TETRIS_ROWS - 1) - row) as f32 * 1.2 + 0.6,
                    0.0,
                );
                material.0 = assets.tetris_materials[*colour as usize % 7].clone();
                *visibility = Visibility::Visible;
            }
            _ => *visibility = Visibility::Hidden,
        }
    }
}
