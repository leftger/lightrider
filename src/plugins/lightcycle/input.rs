//! Lightcycle input handling and the cycle transform update.

use super::CycleEntity;
use super::camera::{cycle_cell_pose, pose_rotation, pose_world_position};
use super::step::restart_run;
use crate::bomberman::sim::{BomberPhase, BomberSim};
use crate::config;
use crate::disc::combat::{DiscEvents, PlayerSnapshot};
use crate::disc::language::SourceGame;
use crate::disc::load::WarpRequested;
use crate::frogger::sim::FroggerSim;
use crate::lightcycle::logic::RunPhase;
use crate::lightcycle::{LightcycleState, RunEnvironment};
use crate::load::DirectoryRequested;
use crate::music::sfx::MusicSfx;
use crate::plugins::transition::ModeTransition;
use crate::qbert::sim::QbertSim;
use crate::state::{FloodState, HistoryState, NavigatorResource, PauseState};
use bevy::prelude::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn read_lightcycle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    transition: Res<ModeTransition>,
    time: Res<Time>,
    mut state: ResMut<LightcycleState>,
    mut pause: ResMut<PauseState>,
    mut history: ResMut<HistoryState>,
    mut flood: ResMut<FloodState>,
    mut navigator: ResMut<NavigatorResource>,
    mut requests: MessageWriter<DirectoryRequested>,
    mut effects: MessageWriter<MusicSfx>,
    mut warps: MessageWriter<WarpRequested>,
) {
    if history.notice_timer > 0.0 {
        history.notice_timer = (history.notice_timer - time.delta_secs()).max(0.0);
        if history.notice_timer == 0.0 {
            history.notice.clear();
        }
    }

    // Riding controls belong to the run, not to the flight arriving at it.
    if transition.is_active() {
        state.slow_motion = false;
        return;
    }

    // The pause menu owns the controls while it is up: navigate with W/S or
    // the arrows, warp with Enter/Space/click, and resume with Esc or P.
    let pause_key = keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyP);
    if pause.paused {
        let up = keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp);
        let down = keys.just_pressed(KeyCode::KeyS) || keys.just_pressed(KeyCode::ArrowDown);
        if up {
            pause.warp_index = (pause.warp_index + SourceGame::COUNT - 1) % SourceGame::COUNT;
        }
        if down {
            pause.warp_index = (pause.warp_index + 1) % SourceGame::COUNT;
        }
        if pause_key {
            pause.paused = false;
        }
        let warp = keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::Space)
            || mouse.just_pressed(MouseButton::Left);
        if warp {
            warps.write(WarpRequested {
                game: SourceGame::ALL[pause.warp_index],
            });
        }
        state.slow_motion = false;
        return;
    }
    if pause_key {
        pause.paused = true;
        return;
    }

    // Git time machine: Z rewinds to the previous directory ridden and Y puts
    // the present back. Both re-enter an arena, so the layout returns with it.
    if keys.just_pressed(KeyCode::KeyZ) {
        match history.rewind() {
            Some(path) => {
                history.notice = "REWIND".to_string();
                history.notice_timer = config::HISTORY_NOTICE_SECONDS;
                effects.write(MusicSfx::Seek);
                requests.write(DirectoryRequested { path });
            }
            None => {
                history.notice = "AT THE FIRST COMMIT".to_string();
                history.notice_timer = config::HISTORY_NOTICE_SECONDS;
            }
        }
        return;
    }
    if keys.just_pressed(KeyCode::KeyY) {
        match history.fast_forward() {
            Some(path) => {
                history.notice = "FAST-FORWARD".to_string();
                history.notice_timer = config::HISTORY_NOTICE_SECONDS;
                effects.write(MusicSfx::Seek);
                requests.write(DirectoryRequested { path });
            }
            None => {
                history.notice = "NOTHING TO REDO".to_string();
                history.notice_timer = config::HISTORY_NOTICE_SECONDS;
            }
        }
        return;
    }

    let left = keys.just_pressed(KeyCode::KeyA) || keys.just_pressed(KeyCode::ArrowLeft);
    let right = keys.just_pressed(KeyCode::KeyD) || keys.just_pressed(KeyCode::ArrowRight);
    let restart = keys.just_pressed(KeyCode::KeyR);
    let go_up = keys.just_pressed(KeyCode::KeyU) || keys.just_pressed(KeyCode::Minus);
    let throw = keys.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left);
    let recall = keys.just_pressed(KeyCode::KeyQ);
    // Stealth walks on WASD: `steer` is the held east/west axis, so only the
    // other pair is needed here.
    let move_z = i32::from(keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown))
        - i32::from(keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp));
    // Hop inputs for the arcade block: one cell per tap, like Frogger and
    // Q*bert need.
    let hop_z = i32::from(keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp))
        - i32::from(keys.just_pressed(KeyCode::KeyS) || keys.just_pressed(KeyCode::ArrowDown));
    // Steering is continuous: the asteroid field pivots while the key is held.
    let steer = i32::from(keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight))
        - i32::from(keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft));
    // The surfer's throttle is always open; boost is held, on the same keys
    // that would otherwise walk or jump.
    let boost = keys.pressed(KeyCode::Space)
        || keys.pressed(KeyCode::KeyW)
        || keys.pressed(KeyCode::ArrowUp);
    // Bullet time: hold Shift to slow the ring while lining up a turn or shot.
    state.slow_motion = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    let entering_tower = state.entry_fx.is_some();
    let Some(mut run) = state.run.take() else {
        return;
    };

    // The field is only a turret fight while the rocks are live; after it is won
    // or lost the bike is handed back and drives normally.
    let field_active = run.asteroid_field_active();

    // Which runs steer the shared bike grid this frame? Directories and
    // documents always do, disc wars and snake always do, the field only once it
    // has handed the bike back, and the off-grid games never.
    let drives_grid = match run.source_game() {
        Some(SourceGame::DiscWars | SourceGame::Snake) | None => true,
        Some(SourceGame::Asteroids) => !field_active,
        Some(
            SourceGame::Platformer
            | SourceGame::Breaker
            | SourceGame::Stealth
            | SourceGame::RiverSurfer
            | SourceGame::Galaga
            | SourceGame::PacMan
            | SourceGame::Columns
            | SourceGame::Tetris
            | SourceGame::Frogger
            | SourceGame::Qbert
            | SourceGame::Bomberman
            | SourceGame::Plinko,
        ) => false,
    };
    if drives_grid && run.sim.phase == RunPhase::Running && (left || right) {
        run.sim.queue_turn_input(left, right);
        effects.write(MusicSfx::Turn);
    }

    if run.is_source() && run.sim.phase == RunPhase::Running {
        if field_active {
            // Pivot and shoot. `set_turn` persists for the fixed sub-steps that
            // follow this frame.
            let mut fired = false;
            if let Some(field) = run.source_asteroids_mut() {
                field.set_turn(steer as f32);
                if throw {
                    fired = field.fire();
                }
            }
            if fired {
                effects.write(MusicSfx::Zap);
            }
        } else if let Some(level) = run.source_platformer_mut() {
            // Held to run, tapped to jump.
            level.set_input(steer as f32, throw);
        } else if let Some(level) = run.source_breaker_mut() {
            // Held to slide the bike along the bottom, tapped to serve.
            level.set_input(steer as f32, throw);
        } else if let Some(room) = run.source_stealth_mut() {
            // Hold a direction to keep walking it; let go to stop.
            room.set_input(steer, move_z);
        } else if let Some(surfer) = run.source_surfer_mut() {
            // Steer the hoverbike; the throttle is always open and boost is held.
            surfer.set_input(steer as f32, boost);
        } else if let Some(sim) = run.source_galaga_mut() {
            // Slide along the bottom; the -Z camera mirrors X, so negate steer.
            sim.set_input(-steer as f32, throw);
        } else if let Some(sim) = run.source_pacman_mut() {
            // Hold a direction to keep walking the corridor.
            sim.set_input(steer, move_z);
        } else if let Some(sim) = run.source_columns_mut() {
            // A/D slides the piece, W rotates, Space hard-drops.
            sim.set_input(i32::from(right) - i32::from(left), hop_z > 0, throw);
        } else if let Some(sim) = run.source_tetris_mut() {
            // A/D slides, W rotates, S soft-drops, Space hard-drops.
            sim.set_input(
                i32::from(right) - i32::from(left),
                hop_z > 0,
                move_z > 0,
                throw,
            );
        } else if let Some(sim) = run.source_frogger_mut() {
            // One hop per keypress in any of the four directions. Hop-Z is
            // +1 for W, but the sim counts +Z as the start row, so flip it.
            sim.hop(i32::from(right) - i32::from(left), -hop_z);
        } else if let Some(sim) = run.source_qbert_mut() {
            // Diagonal hops: A/D/W/S each map to a pyramid direction. The -Z
            // camera mirrors X, so swap the east/west edges.
            sim.hop(i32::from(left) - i32::from(right), hop_z);
        } else if let Some(sim) = run.source_bomberman_mut() {
            // Walk on the room grid and plant bombs with Space or click.
            if (steer != 0 || move_z != 0) && sim.phase == BomberPhase::Walking {
                sim.step(steer, move_z);
            }
            if throw {
                sim.plant();
            }
        } else if let Some(sim) = run.source_plinko_mut() {
            // Slide the rail and drop balls with Space or click.
            let slide = sim.aim + steer as f32 * 6.0;
            sim.set_aim(slide);
            if throw {
                sim.drop_ball();
            }
        } else if run.source_game() == Some(SourceGame::DiscWars) {
            // Disc wars: throw and recall. The cycle's movement is unchanged.
            let snapshot = PlayerSnapshot {
                cell: run.sim.cell,
                heading: run.sim.heading,
                running: true,
            };
            if throw {
                let mut events = DiscEvents::default();
                // Aimed at the opponent, so riding and aiming stay separate.
                if let RunEnvironment::Source { sim, .. } = &mut run.environment
                    && let Some(disc) = sim.as_disc_mut()
                {
                    disc.throw_player(snapshot, &run.arena, &mut events);
                }
                if events.player_threw {
                    effects.write(MusicSfx::Beam);
                }
            }
            if recall {
                let mut events = DiscEvents::default();
                if let Some(disc) = run.source_disc_mut() {
                    disc.recall_player(&mut events);
                }
                if events.player_recalled {
                    effects.write(MusicSfx::Turn);
                }
            }
        }
    }

    if restart {
        restart_run(&mut run);
        // The wall that ended the last run would otherwise still be standing
        // past the spawn cell, killing the respawn on its first frame.
        flood.recede();
        // A restarted run starts with the collector idle too, rather than
        // inheriting a sweep that was half way across the old one.
        state.gc_pause = 0.0;
        state.gc_sweep = 0.0;
        state.gc_timer = config::GC_INTERVAL_SECONDS;
        state.clock = 0.0;
        state.crash_fx = None;
        state.entry_fx = None;
    }

    if go_up && !entering_tower {
        effects.write(MusicSfx::Portal);
        if run.is_document() || run.is_source() {
            state.restore_directory = true;
        } else if let Some(parent) = navigator.0.begin_go_to_parent() {
            run.sim.pause_for_directory_change();
            run.entering_label = Some("RET → parent".to_string());
            run.crash_label = None;
            requests.write(DirectoryRequested { path: parent });
        }
    }

    state.run = Some(run);
}

pub(crate) fn update_cycle_transform(
    state: Res<LightcycleState>,
    mut cycle: Query<(&mut Transform, &mut Visibility), With<CycleEntity>>,
) {
    let Ok((mut transform, mut visibility)) = cycle.single_mut() else {
        return;
    };
    let Some(run) = state.run.as_ref() else {
        return;
    };

    // The on-foot games park the bike out of sight and pose their own character.
    if run.source_platformer().is_some() || run.source_stealth().is_some() {
        *visibility = Visibility::Hidden;
        return;
    }
    *visibility = Visibility::Visible;

    // In the breaker the bike is the paddle: it slides along the bottom of the
    // court and rebounds the ball.
    if let Some(level) = run.source_breaker() {
        transform.translation = Vec3::new(level.paddle_x, config::BREAKER_PADDLE_Y, 0.0);
        transform.rotation = Quat::IDENTITY;
        transform.scale = Vec3::splat(config::BREAKER_PADDLE_SCALE);
        return;
    }
    transform.scale = Vec3::ONE;

    // The surfer rides the shared bike as a hovercraft over the river, bobbing
    // with the waves and leaning into the steering heading.
    if let Some(surfer) = run.source_surfer() {
        transform.translation = Vec3::new(surfer.x, surfer.height, surfer.z);
        transform.rotation = Quat::from_rotation_arc(
            Vec3::X,
            Vec3::new(surfer.heading.cos(), 0.0, surfer.heading.sin()),
        );
        return;
    }

    // The Galaga field parks the bike on the bottom edge, facing up the field,
    // and slides it side to side.
    if let Some(sim) = run.source_galaga() {
        transform.translation = Vec3::new(sim.player_x, 0.0, config::GALAGA_PLAYER_Z);
        transform.rotation = Quat::from_rotation_arc(Vec3::X, Vec3::Z);
        return;
    }

    // Pac-Man rides the maze corridors.
    if let Some(sim) = run.source_pacman() {
        transform.translation = Vec3::new(sim.x, 0.7, sim.z);
        transform.rotation = Quat::from_rotation_arc(Vec3::X, Vec3::Z);
        return;
    }

    // Columns and Tetris park the bike at the foot of the well; Plinko parks
    // it on the top rail.
    if run.source_columns().is_some() {
        transform.translation = Vec3::new(0.0, 0.0, 3.0);
        transform.rotation = Quat::from_rotation_arc(Vec3::X, Vec3::Y);
        return;
    }
    if run.source_tetris().is_some() {
        transform.translation = Vec3::new(0.0, 0.0, 4.0);
        transform.rotation = Quat::from_rotation_arc(Vec3::X, Vec3::Y);
        return;
    }
    if let Some(sim) = run.source_plinko() {
        transform.translation = Vec3::new(sim.aim, config::PLINKO_HEIGHT * 0.5 - 1.0, 0.0);
        transform.rotation = Quat::from_rotation_arc(Vec3::X, Vec3::Y);
        return;
    }

    // Frogger and Bomberman walk the cycle on their X/Z grids.
    if let Some(sim) = run.source_frogger() {
        let (x, z) = FroggerSim::center(sim.cell);
        transform.translation = Vec3::new(x, 0.7, z);
        transform.rotation = Quat::from_rotation_arc(Vec3::X, Vec3::Z);
        return;
    }
    if let Some(sim) = run.source_bomberman() {
        let (x, z) = BomberSim::center(sim.cell);
        transform.translation = Vec3::new(x, 0.7, z);
        transform.rotation = Quat::from_rotation_arc(Vec3::X, Vec3::Z);
        return;
    }

    // Q*bert perches the bike on its current cube.
    if let Some(sim) = run.source_qbert() {
        let (x, z) = QbertSim::cube_position(sim.row, sim.index);
        let y =
            (config::QBERT_ROWS as f32 - 1.0 - sim.row as f32) * config::QBERT_CUBE_HEIGHT * 0.5
                + config::QBERT_CUBE_HEIGHT * 0.6;
        transform.translation = Vec3::new(x, y, z);
        transform.rotation = Quat::from_rotation_arc(Vec3::X, Vec3::Z);
        return;
    }

    let pose = cycle_cell_pose(&run.sim);
    transform.translation = pose_world_position(&pose);
    // A parked cycle pivots on the spot: its facing is the field's aim angle,
    // not a grid heading. Once the field ends it drives again, so the grid
    // heading takes over — and a disc-wars ring never leaves it in the first
    // place.
    let aiming = if run.asteroid_field_active() {
        run.source_asteroids()
    } else {
        None
    };
    transform.rotation = match aiming {
        Some(sim) => {
            Quat::from_rotation_arc(Vec3::X, Vec3::new(sim.angle.cos(), 0.0, sim.angle.sin()))
        }
        None => pose_rotation(&pose),
    };
}
