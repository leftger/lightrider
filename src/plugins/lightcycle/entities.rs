//! Pooled-entity sync systems: show, hide and move a mini-game's entity pool.

use super::camera::character_pose;
use super::disc::disc_entity_position;
use super::{
    Apart, BallEntity, BeamEntity, BlockEntity, BomberBombEntity, BomberCrateEntity, BrickEntity,
    BugEntity, CharacterAnim, CharacterEntity, CycleEntity, DiscPickupEntity, DotEntity, Fighter,
    FreeOf, FrogObstacleEntity, GalagaBeamEntity, GemEntity, GhostEntity, GuardConeEntity,
    GuardEntity, LightcycleAssets, OpponentDiscEntity, OpponentEntity, OutOfCycle,
    PlayerDiscEntity, PlinkoBallEntity, Pooled, PooledPosed, PooledPosedTinted, PooledShown,
    PooledTinted, QbertCubeEntity, QbertEnemyEntity, RockEntity, SnakeFoodEntity, SnakeGateLock,
};
use crate::bomberman::sim::BomberSim;
use crate::config;
use crate::disc::language::SourceGame;
use crate::frogger::sim::FroggerSim;
use crate::lightcycle::LightcycleState;
use crate::qbert::sim::QbertSim;
use crate::state::{DirectorySceneRoot, InteractionMode};
use bevy::prelude::*;

pub(crate) fn sync_directory_scene_visibility(
    mode: Res<InteractionMode>,
    mut directory_scene: Query<&mut Visibility, With<DirectorySceneRoot>>,
) {
    let wanted = if *mode == InteractionMode::Explorer {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    // Writing unconditionally marks every entity in the scene changed each
    // frame, which makes Bevy redo visibility propagation for all of them.
    for mut visibility in &mut directory_scene {
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
}

/// Keeps the disc, opponent, opponent disc, and pickups glued to the sim.
pub(crate) fn sync_disc_entities(
    state: Res<LightcycleState>,
    mut player_disc: Fighter<PlayerDiscEntity, OpponentEntity, OpponentDiscEntity>,
    mut opponent: Fighter<OpponentEntity, PlayerDiscEntity, OpponentDiscEntity>,
    mut opponent_disc: Fighter<OpponentDiscEntity, PlayerDiscEntity, OpponentEntity>,
    mut pickups: PooledShown<
        DiscPickupEntity,
        FreeOf<PlayerDiscEntity, OpponentEntity, OpponentDiscEntity>,
    >,
) {
    let Some(run) = state.run.as_ref() else {
        return;
    };
    let Some(disc) = run.source_disc() else {
        return;
    };

    if let Ok((mut transform, mut visibility)) = player_disc.single_mut() {
        match disc.player_disc.as_ref() {
            Some(flying) => {
                transform.translation = disc_entity_position(flying.cell);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
    if let Ok((mut transform, mut visibility)) = opponent.single_mut() {
        if disc.opponent.alive {
            // The opponent steps a whole cell at a time. Render it partway to the
            // cell it is walking into so it glides instead of teleporting; while
            // it charges it stands exactly on its cell, so the shot is readable.
            let progress = if disc.opponent.windup > 0.0 {
                0.0
            } else {
                disc.opponent.move_clock.clamp(0.0, 1.0)
            };
            let (dx, dz) = disc.opponent.heading.delta();
            transform.translation =
                config::ground_position(disc.opponent.cell.0, disc.opponent.cell.1)
                    + Vec3::new(dx as f32, 0.0, dz as f32) * (progress * config::GRID_SPACING)
                    + Vec3::Y * (config::RECOGNIZER_HEIGHT * 0.5);
            // Swell while winding up, so its shot is telegraphed.
            let charge = (disc.opponent.windup / config::DISC_OPPONENT_WINDUP).clamp(0.0, 1.0);
            transform.scale =
                Vec3::new(1.0 + charge * 0.35, 1.0 - charge * 0.2, 1.0 + charge * 0.35);
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
    if let Ok((mut transform, mut visibility)) = opponent_disc.single_mut() {
        match disc.opponent.disc.as_ref() {
            Some(flying) => {
                transform.translation = disc_entity_position(flying.cell);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
    for (pickup, mut visibility) in &mut pickups {
        *visibility = if disc.taken.contains(&pickup.index) {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
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

/// Places the pooled bugs and beams of a Galaga field. Dead bugs and spent
/// beams are hidden rather than despawned, so the pool never needs to grow.
pub(crate) fn sync_galaga_entities(
    state: Res<LightcycleState>,
    mut bugs: Pooled<BugEntity, Apart<GalagaBeamEntity>>,
    mut beams: Pooled<GalagaBeamEntity, Apart<BugEntity>>,
) {
    let Some(sim) = state.run.as_ref().and_then(|run| run.source_galaga()) else {
        return;
    };

    for (entity, mut transform, mut visibility) in &mut bugs {
        match sim.bugs.get(entity.index) {
            Some(bug) if bug.alive => {
                transform.translation = Vec3::new(bug.x, config::GALAGA_BUG_HEIGHT * 0.5, bug.z);
                *visibility = Visibility::Visible;
            }
            _ => *visibility = Visibility::Hidden,
        }
    }

    for (entity, mut transform, mut visibility) in &mut beams {
        match sim.beams.get(entity.index) {
            Some(beam) => {
                transform.translation = Vec3::new(beam.x, 0.35, beam.z);
                transform.scale = Vec3::new(0.12, 0.12, config::GALAGA_BEAM_LENGTH);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}

/// Places the pooled dots and ghosts of a Pac-Man maze.
pub(crate) fn sync_pacman_entities(
    state: Res<LightcycleState>,
    mut dots: PooledShown<DotEntity, Apart<GhostEntity>>,
    mut ghosts: Pooled<GhostEntity, Apart<DotEntity>>,
) {
    let Some(sim) = state.run.as_ref().and_then(|run| run.source_pacman()) else {
        return;
    };
    for (entity, mut visibility) in &mut dots {
        *visibility = if sim.dots.contains(&entity.cell) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (entity, mut transform, mut visibility) in &mut ghosts {
        match sim.ghosts.get(entity.index) {
            Some(ghost) => {
                transform.translation = Vec3::new(ghost.x, 0.9, ghost.z);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}

/// Places the pooled gem cells of a Columns well.
pub(crate) fn sync_columns_entities(
    state: Res<LightcycleState>,
    assets: Res<LightcycleAssets>,
    mut gems: PooledTinted<GemEntity, OutOfCycle>,
) {
    let Some(sim) = state.run.as_ref().and_then(|run| run.source_columns()) else {
        return;
    };
    let rendered = sim.render_board();
    for (entity, mut transform, mut visibility, mut material) in &mut gems {
        match rendered.get(entity.index) {
            Some(Some(colour)) => {
                let col = entity.index % config::COLUMNS_COLS;
                let row = entity.index / config::COLUMNS_COLS;
                transform.translation = Vec3::new(
                    (col as f32 - (config::COLUMNS_COLS - 1) as f32 * 0.5) * 1.6,
                    ((config::COLUMNS_ROWS - 1) - row) as f32 * 1.6 + 0.8,
                    0.0,
                );
                material.0 =
                    assets.gem_materials[*colour as usize % config::COLUMNS_GEM_COLORS].clone();
                *visibility = Visibility::Visible;
            }
            _ => *visibility = Visibility::Hidden,
        }
    }
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

/// Places the pooled obstacle cubes of a Frogger highway.
pub(crate) fn sync_frogger_entities(
    state: Res<LightcycleState>,
    mut obstacles: Pooled<FrogObstacleEntity, OutOfCycle>,
) {
    let Some(sim) = state.run.as_ref().and_then(|run| run.source_frogger()) else {
        return;
    };
    let cells = sim.obstacle_cells();
    for (entity, mut transform, mut visibility) in &mut obstacles {
        match cells.get(entity.index) {
            Some(&cell) => {
                let (x, z) = FroggerSim::center(cell);
                transform.translation = Vec3::new(x, 0.6, z);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}

/// Relights Q*bert cubes and places the pooled enemies.
pub(crate) fn sync_qbert_entities(
    state: Res<LightcycleState>,
    assets: Res<LightcycleAssets>,
    mut cubes: PooledPosedTinted<QbertCubeEntity, Without<QbertEnemyEntity>>,
    mut enemies: Pooled<QbertEnemyEntity, Without<QbertCubeEntity>>,
) {
    let Some(sim) = state.run.as_ref().and_then(|run| run.source_qbert()) else {
        return;
    };
    for (entity, mut transform, mut material) in &mut cubes {
        let lit = sim.lit.get(QbertSim::cube_index(entity.row, entity.index));
        let (x, z) = QbertSim::cube_position(entity.row, entity.index);
        let y =
            (config::QBERT_ROWS as f32 - 1.0 - entity.row as f32) * config::QBERT_CUBE_HEIGHT * 0.5;
        transform.translation = Vec3::new(x, y, z);
        material.0 = if lit == Some(&true) {
            assets.qbert_cube_lit.clone()
        } else {
            assets.qbert_cube_dim.clone()
        };
    }
    for (entity, mut transform, mut visibility) in &mut enemies {
        match sim.enemies.get(entity.index) {
            Some(enemy) => {
                let (x, z) = QbertSim::cube_position(enemy.row, enemy.index);
                let y = (config::QBERT_ROWS as f32 - 1.0 - enemy.row as f32)
                    * config::QBERT_CUBE_HEIGHT
                    * 0.5
                    + config::QBERT_CUBE_HEIGHT * 0.8;
                transform.translation = Vec3::new(x, y, z);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}

/// Places the pooled crates and bombs of a Bomberman room.
pub(crate) fn sync_bomberman_entities(
    state: Res<LightcycleState>,
    mut crates: PooledShown<BomberCrateEntity, Apart<BomberBombEntity>>,
    mut bombs: Pooled<BomberBombEntity, Apart<BomberCrateEntity>>,
) {
    let Some(sim) = state.run.as_ref().and_then(|run| run.source_bomberman()) else {
        return;
    };
    for (entity, mut visibility) in &mut crates {
        *visibility = if sim.crates.contains(&entity.cell) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (entity, mut transform, mut visibility) in &mut bombs {
        match sim.bombs.get(entity.index) {
            Some(bomb) => {
                let (x, z) = BomberSim::center(bomb.cell);
                transform.translation = Vec3::new(x, 0.6, z);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}

/// Places the pooled balls of a Plinko board.
pub(crate) fn sync_plinko_entities(
    state: Res<LightcycleState>,
    mut balls: Query<(&PlinkoBallEntity, &mut Transform, &mut Visibility), Without<CycleEntity>>,
) {
    let Some(sim) = state.run.as_ref().and_then(|run| run.source_plinko()) else {
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

/// Poses the on-foot character, whichever game it belongs to.
pub(crate) fn sync_character_entities(
    state: Res<LightcycleState>,
    time: Res<Time>,
    mut character: Query<(&mut Transform, &mut CharacterAnim), Without<CycleEntity>>,
) {
    let Some(run) = state.run.as_ref() else {
        return;
    };
    let Some(pose) = character_pose(run) else {
        return;
    };

    let dt = time.delta_secs();
    for (mut transform, mut anim) in &mut character {
        // The stealth sim moves in whole cells; easing toward the cell turns
        // that into a glide, and the walk clip supplies the limbs. The
        // platformer's physics is already continuous.
        let base = if pose.smooth {
            // Cover the ground at the pace the sim steps, instead of easing to
            // each cell and waiting. The second term only bites once the figure
            // has fallen behind, so a frame hitch does not leave it trailing.
            let to_target = pose.target - anim.base;
            let distance = to_target.length();
            let travel = config::STEALTH_WALK_SPEED.max(distance * 2.0) * dt;
            if distance <= travel {
                pose.target
            } else {
                anim.base + to_target / distance * travel
            }
        } else {
            pose.target
        };
        anim.base = base;
        transform.translation = base;
        transform.rotation = Quat::from_rotation_y(pose.yaw);
    }
}

/// Keeps the breaker's ball and bricks glued to its sim.
pub(crate) fn sync_breaker_entities(
    state: Res<LightcycleState>,
    mut ball: Query<&mut Transform, (With<BallEntity>, Without<BrickEntity>)>,
    mut bricks: PooledShown<BrickEntity, Apart<BallEntity, CharacterEntity>>,
) {
    let Some(level) = state.run.as_ref().and_then(|run| run.source_breaker()) else {
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

/// Walks the patrols and swings their vision cones.
pub(crate) fn sync_stealth_entities(
    state: Res<LightcycleState>,
    mut guards: PooledPosed<GuardEntity, Apart<GuardConeEntity, CharacterEntity>>,
    mut cones: PooledPosed<GuardConeEntity, Apart<GuardEntity, CharacterEntity>>,
) {
    let Some(room) = state.run.as_ref().and_then(|run| run.source_stealth()) else {
        return;
    };
    for (guard, mut transform) in &mut guards {
        if let Some(guard) = room.guards.get(guard.index) {
            let cell = guard.cell();
            transform.translation = config::ground_position(cell.0, cell.1);
            transform.rotation =
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 - guard.patrol.heading().angle());
        }
    }
    for (cone, mut transform) in &mut cones {
        if let Some(guard) = room.guards.get(cone.index) {
            let cell = guard.cell();
            transform.translation = config::ground_position(cell.0, cell.1) + Vec3::Y * 0.08;
            transform.rotation =
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 - guard.vision_angle());
        }
    }
}
