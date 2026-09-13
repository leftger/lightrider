//! The fixed-step simulation: one step per arena and per mini-game.

use super::decor::is_ring_gate;
use super::entry::crash_source;
use super::run::spawn_sim;
use super::space::{
    heading_facing, nearest_heading, ring_center_world, ring_food_cells, ring_radius_world,
};
use crate::asteroids::sim::{AsteroidsPhase, AsteroidsSim};
use crate::config;
use crate::disc::combat::{DiscPhase, DiscSim, PlayerSnapshot};
use crate::disc::language::SourceGame;
use crate::disc::load::SourceRequested;
use crate::disc::plugin::disc_crash_label;
use crate::document::load::DocumentRequested;
use crate::lightcycle::logic::{
    CellContent, CrashReason, LightcycleSim, RunPhase, StepOutcome, classify_next_content,
};
use crate::lightcycle::{ActiveRun, LightcycleState, RunEnvironment, SourceSim};
use crate::load::DirectoryRequested;
use crate::minigame::GameSound;
use crate::music::sfx::MusicSfx;
use crate::plugins::transition::ModeTransition;
use crate::snake::sim::SnakeSim;
use crate::state::{NavigatorResource, PauseState};
use bevy::prelude::*;
use std::collections::HashMap;

pub(crate) fn restart_run(run: &mut ActiveRun) {
    match &mut run.environment {
        RunEnvironment::Directory { cells, .. } => {
            let cells = cells.clone();
            run.sim = spawn_sim(&run.arena, &cells);
        }
        RunEnvironment::Document { .. } => {
            run.sim = spawn_sim(&run.arena, &HashMap::new());
        }
        RunEnvironment::Source { layout, sim, .. } => {
            let spawn = layout.player_spawn;
            let heading = layout.player_spawn_heading;
            let seed = layout.seed;
            let center = ring_center_world(layout);
            let radius = ring_radius_world(layout);
            let food_cells = ring_food_cells(&run.arena);
            run.sim = LightcycleSim::start(spawn, heading);
            match sim {
                SourceSim::DiscWars(disc) => *disc = DiscSim::new(layout),
                SourceSim::Asteroids(field) => {
                    let mut fresh = AsteroidsSim::new(seed, center, radius);
                    fresh.angle = heading_facing(heading);
                    **field = fresh;
                }
                SourceSim::Snake(snake) => {
                    *snake =
                        SnakeSim::new(seed, spawn, &food_cells, config::snake::SNAKE_FOOD_TARGET);
                }
                // Each sim re-rolls itself from its own stored seed, so a
                // restart lays out exactly the same level.
                other => {
                    if let Some(game) = other.as_game_mut() {
                        game.restart();
                    }
                }
            }
        }
    }
    run.crash_label = None;
    run.entering_label = None;
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn step_lightcycle(
    time: Res<Time>,
    transition: Res<ModeTransition>,
    pause: Res<PauseState>,
    mut state: ResMut<LightcycleState>,
    mut navigator: ResMut<NavigatorResource>,
    mut requests: MessageWriter<DirectoryRequested>,
    mut documents: MessageWriter<DocumentRequested>,
    mut sources: MessageWriter<SourceRequested>,
    mut effects: MessageWriter<MusicSfx>,
) {
    // The arena exists from the top of the climb onward, but the camera is
    // still diving toward it. Hold the cycle on its spawn cell until it lands,
    // so the run starts from the shot the player is given rather than partway
    // down the first street.
    if transition.is_active() {
        state.clock = 0.0;
        return;
    }

    // The pause menu freezes the sim in place; no clock accrues, so there is
    // no catch-up burst when the run resumes.
    if pause.paused {
        state.slow_motion = false;
        return;
    }

    let Some(mut run) = state.run.take() else {
        return;
    };

    // A ring keeps ticking even while the player is crashed or waiting between
    // rounds, so the fight can pay out its respawn. Other arenas hold still.
    let source_run = run.is_source();
    if run.sim.phase != RunPhase::Running && !source_run {
        state.run = Some(run);
        return;
    }

    state.clock += time.delta_secs();
    let max_catch_up = config::lightcycle::LIGHTCYCLE_FIXED_STEP
        * config::lightcycle::LIGHTCYCLE_MAX_SUBSTEPS as f32;
    if state.clock > max_catch_up {
        state.clock = max_catch_up;
    }

    let fixed_step = config::lightcycle::LIGHTCYCLE_FIXED_STEP;
    // Directory arenas run the collector on a timer: when it fires the world
    // stalls for a beat, then the sweep passes and play resumes. A revisit to
    // an already-opened directory rides a cache-hit surge instead.
    let dt = time.delta_secs();
    if !source_run {
        state.cache_boost = (state.cache_boost - dt).max(0.0);
        state.gc_sweep = (state.gc_sweep - dt).max(0.0);
        if state.gc_pause > 0.0 {
            state.gc_pause = (state.gc_pause - dt).max(0.0);
        } else {
            state.gc_timer -= dt;
            if state.gc_timer <= 0.0 {
                state.gc_timer = config::lightcycle::GC_INTERVAL_SECONDS;
                state.gc_pause = config::lightcycle::GC_PAUSE_SECONDS;
                state.gc_sweep = config::lightcycle::GC_SWEEP_SECONDS;
                effects.write(MusicSfx::Seek);
            }
        }
    }

    // Bullet time stretches the simulated step without changing the real-time
    // cadence, so the bike, its disc, and the opponent all slow together.
    let mut step = if source_run && state.slow_motion {
        fixed_step * config::disc::DISC_BULLET_TIME_SCALE
    } else {
        fixed_step
    };
    if !source_run && state.gc_pause > 0.0 {
        step *= config::lightcycle::GC_SLOW_SCALE;
    }
    if !source_run && state.cache_boost > 0.0 {
        let heat = state.cache_boost / config::lightcycle::CACHE_BOOST_SECONDS;
        step *= 1.0 + (config::lightcycle::CACHE_BOOST_SCALE - 1.0) * heat;
    }
    let mut substeps = 0;

    while state.clock >= fixed_step && substeps < config::lightcycle::LIGHTCYCLE_MAX_SUBSTEPS {
        state.clock -= fixed_step;
        substeps += 1;

        let outcome = {
            let arena = &run.arena;
            let sim = &mut run.sim;
            match &run.environment {
                RunEnvironment::Directory { .. } | RunEnvironment::Document { .. } => {
                    StepOutcome::Moved
                }
                RunEnvironment::Source { sim: source, .. } => match source {
                    // Parked while the rocks are live: nothing to advance, and
                    // the field itself is stepped after the loop.
                    SourceSim::Asteroids(field) if field.is_active() => StepOutcome::Moved,
                    // Once the field is decided the cycle is handed back, so it
                    // drives again and can ride out through the gate.
                    SourceSim::Asteroids(_) => sim.advance(
                        step * config::lightcycle::LIGHTCYCLE_CELLS_PER_SEC,
                        |next, state| {
                            classify_next_content(
                                next,
                                arena,
                                state,
                                &HashMap::new(),
                                |_| false,
                                |_| false,
                                |_| false,
                            )
                        },
                    ),
                    // Snake drives the ordinary grid, but the exit is a solid
                    // wall until enough power-ups have been eaten. The tail is
                    // capped after every step so it stays finite.
                    SourceSim::Snake(snake) => {
                        let locked = !snake.exit_open;
                        let outcome = sim.advance(
                            step * config::lightcycle::LIGHTCYCLE_CELLS_PER_SEC,
                            |next, state| {
                                if locked && is_ring_gate(arena, next) {
                                    return CellContent::Wall;
                                }
                                classify_next_content(
                                    next,
                                    arena,
                                    state,
                                    &HashMap::new(),
                                    |_| false,
                                    |_| false,
                                    |_| false,
                                )
                            },
                        );
                        snake.trim_tail(sim);
                        outcome
                    }
                    // The off-grid games drive their own sims, so the shared
                    // grid has nothing to advance.
                    SourceSim::Platformer(_)
                    | SourceSim::Breaker(_)
                    | SourceSim::Stealth(_)
                    | SourceSim::Surfer(_)
                    | SourceSim::Galaga(_)
                    | SourceSim::PacMan(_)
                    | SourceSim::Columns(_)
                    | SourceSim::Tetris(_)
                    | SourceSim::Frogger(_)
                    | SourceSim::Qbert(_)
                    | SourceSim::Bomberman(_)
                    | SourceSim::Plinko(_) => StepOutcome::Moved,
                    SourceSim::DiscWars(disc) => {
                        // The opponent's body and its live disc are lethal cells
                        // in the same grid model the cycle already uses.
                        let opponent = disc.opponent_cell();
                        let opponent_disc = disc.opponent_disc_cell();
                        sim.advance(
                            step * config::lightcycle::LIGHTCYCLE_CELLS_PER_SEC,
                            |next, state| {
                                if Some(next) == opponent {
                                    return CellContent::Opponent;
                                }
                                if Some(next) == opponent_disc {
                                    return CellContent::OpponentDisc;
                                }
                                classify_next_content(
                                    next,
                                    arena,
                                    state,
                                    &HashMap::new(),
                                    |_| false,
                                    |_| false,
                                    |_| false,
                                )
                            },
                        )
                    }
                },
            }
        };

        match outcome {
            StepOutcome::Moved => {}
            StepOutcome::Crashed(reason) => {
                let crash_cell = run.sim.next_cell();
                let label = match reason {
                    CrashReason::File => run
                        .directory_cells()
                        .and_then(|cells| cells.get(&crash_cell).copied())
                        .and_then(|index| {
                            run.directory_nodes()
                                .and_then(|nodes| nodes.get(index))
                                .map(|node| format!("file {}", node.name))
                        })
                        .unwrap_or_else(|| "file".to_string()),
                    CrashReason::Trail => "your trail".to_string(),
                    CrashReason::Opponent => "the recognizer".to_string(),
                    CrashReason::Disc => "a disc".to_string(),
                    CrashReason::Hazard => "a hazard tile".to_string(),
                    // Snake's gate is solid until the exit opens.
                    CrashReason::Wall
                        if run.source_snake().is_some() && is_ring_gate(&run.arena, crash_cell) =>
                    {
                        "the locked exit".to_string()
                    }
                    CrashReason::Wall if run.arena.street_walls.contains(&crash_cell) => {
                        if run.is_document() {
                            "paragraph".to_string()
                        } else if run.is_source() {
                            "ring wall".to_string()
                        } else {
                            "street barrier".to_string()
                        }
                    }
                    CrashReason::Wall => "a dead bus line".to_string(),
                };
                run.crash_label = Some(label);
                run.entering_label = None;
            }
            StepOutcome::EnteringDir(index) => {
                let details = run
                    .directory_nodes()
                    .and_then(|nodes| nodes.get(index))
                    .map(|node| (node.name.clone(), node.path.clone()));
                if let Some((name, path)) = details {
                    run.entering_label = Some(format!("DMA → {name}"));
                    run.crash_label = None;
                    effects.write(MusicSfx::Beam);
                    state.entry_fx = Some(crate::lightcycle::EntryFx::new(
                        path,
                        config::lightcycle::LIGHTCYCLE_ENTRY_FX_DURATION,
                    ));
                } else {
                    run.sim.phase = RunPhase::Crashed;
                    run.crash_label = Some("unmapped address".to_string());
                }
            }
            StepOutcome::EnteringDocument(index) => {
                let details = run
                    .directory_nodes()
                    .and_then(|nodes| nodes.get(index))
                    .map(|node| (node.name.clone(), node.path.clone()));
                if let Some((name, path)) = details {
                    run.entering_label = Some(name);
                    run.crash_label = None;
                    effects.write(MusicSfx::Beam);
                    documents.write(DocumentRequested { path });
                } else {
                    run.sim.phase = RunPhase::Running;
                    run.crash_label = Some("unreadable sector".to_string());
                }
            }
            StepOutcome::EnteringSource(index) => {
                let details = run
                    .directory_nodes()
                    .and_then(|nodes| nodes.get(index))
                    .map(|node| (node.name.clone(), node.path.clone()));
                if let Some((name, path)) = details {
                    run.entering_label = Some(name);
                    run.crash_label = None;
                    effects.write(MusicSfx::Beam);
                    sources.write(SourceRequested { path });
                } else {
                    run.sim.phase = RunPhase::Running;
                    run.crash_label = Some("unmapped sector".to_string());
                }
            }
            StepOutcome::GoToParent => {
                if let Some(parent) = navigator.0.begin_go_to_parent() {
                    run.entering_label = Some("RET → parent".to_string());
                    run.crash_label = None;
                    effects.write(MusicSfx::Portal);
                    requests.write(DirectoryRequested { path: parent });
                } else {
                    run.sim.phase = RunPhase::Crashed;
                    run.crash_label = Some("a dead bus line".to_string());
                }
            }
            StepOutcome::CloseDocument => {
                state.restore_directory = true;
            }
        }

        if source_run {
            let cleared = match run.source_game() {
                Some(SourceGame::Asteroids) => {
                    step_asteroid_field(&mut run, step, &mut effects);
                    false
                }
                Some(SourceGame::Snake) => {
                    step_snake(&mut run, &mut effects);
                    false
                }
                Some(SourceGame::DiscWars) => {
                    step_disc_fight(&mut run, step, &mut effects);
                    false
                }
                // Everything else is a uniform `dt`-driven game.
                Some(_) => step_uniform(&mut run, step, &mut effects),
                None => false,
            };
            // Clearing a level is this run's version of riding out the gate.
            if cleared {
                state.restore_directory = true;
            }
        }

        if run.sim.phase == RunPhase::Crashed && state.crash_fx.is_none() {
            state.crash_fx = Some(crate::lightcycle::CrashFx::new(
                config::lightcycle::LIGHTCYCLE_CRASH_FX_DURATION,
            ));
            effects.write(MusicSfx::GameOver);
        }

        if run.sim.phase != RunPhase::Running && !source_run {
            break;
        }
    }

    state.run = Some(run);
}

/// Steps the asteroid field and routes its feedback into sound and labels.
pub(crate) fn step_asteroid_field(
    run: &mut ActiveRun,
    dt: f32,
    effects: &mut MessageWriter<MusicSfx>,
) {
    let events = {
        let Some(field) = run.source_asteroids_mut() else {
            return;
        };
        field.update(dt)
    };

    for _ in 0..events.destroyed {
        effects.write(MusicSfx::Portal);
    }
    if events.lost_life {
        effects.write(MusicSfx::Crash);
    }
    if events.cleared {
        run.crash_label = None;
        run.entering_label = None;
        effects.write(MusicSfx::Victory);
    }

    let lost = run
        .source_asteroids()
        .is_some_and(|field| field.phase == AsteroidsPhase::Lost);
    if lost {
        run.crash_label = Some("the rock field".to_string());
        run.entering_label = None;
    }

    if events.ended {
        // Hand the wheel back on the frame the field is decided: the bike keeps
        // the facing the player was holding and can drive to the gate to leave.
        let facing = run.source_asteroids_mut().map(|field| {
            field.set_turn(0.0);
            nearest_heading(field.angle)
        });
        if let Some(facing) = facing {
            run.sim.heading = facing;
        }
    }
}

/// Steps a snake run: collects power-ups, opens the exit and cues the death.
pub(crate) fn step_snake(run: &mut ActiveRun, effects: &mut MessageWriter<MusicSfx>) {
    let cell = run.sim.cell;
    let Some(events) = run.source_snake_mut().map(|snake| snake.eat(cell)) else {
        return;
    };
    if events.ate {
        effects.write(MusicSfx::Portal);
    }
    if events.opened_exit {
        effects.write(MusicSfx::Beam);
    }

    // The base grid crash ends the run; the rider keeps their crash FX, and the
    // snake only adds the sound once.
    if run.sim.phase == RunPhase::Crashed
        && run
            .source_snake_mut()
            .is_some_and(|snake| snake.note_crash())
    {
        effects.write(MusicSfx::Crash);
    }
}

/// Steps a uniform mini-game and folds its tick back into the shared run.
///
/// Every source game but disc wars, the asteroid field and snake answers a step
/// with a [`GameTick`]: the sounds to play, whether it cleared, and whether the
/// run is over. The game owns that policy beside its sim; this only plays it.
fn step_uniform(run: &mut ActiveRun, dt: f32, effects: &mut MessageWriter<MusicSfx>) -> bool {
    let tick = {
        let RunEnvironment::Source { sim, .. } = &mut run.environment else {
            return false;
        };
        let Some(game) = sim.as_game_mut() else {
            return false;
        };
        game.tick(dt)
    };

    for sound in tick.sounds {
        effects.write(match sound {
            GameSound::Turn => MusicSfx::Turn,
            GameSound::Beam => MusicSfx::Beam,
            GameSound::Portal => MusicSfx::Portal,
            GameSound::Victory => MusicSfx::Victory,
            GameSound::Crash => MusicSfx::Crash,
            GameSound::Zap => MusicSfx::Zap,
        });
    }

    if tick.lost {
        // The shared crash path owns the crash sound, the phase change and the
        // label, so a mini-game's loss reads like any other.
        let label = tick.label.as_deref().unwrap_or("the crash");
        crash_source(run, label, effects);
    }

    tick.cleared
}

/// Steps one ring's fight and folds its events back into the shared run.
pub(crate) fn step_disc_fight(run: &mut ActiveRun, dt: f32, effects: &mut MessageWriter<MusicSfx>) {
    let snapshot = PlayerSnapshot {
        cell: run.sim.cell,
        heading: run.sim.heading,
        running: run.sim.phase == RunPhase::Running,
    };
    let events = {
        let arena = &run.arena;
        let RunEnvironment::Source { sim, layout, .. } = &mut run.environment else {
            return;
        };
        let Some(disc) = sim.as_disc_mut() else {
            return;
        };
        disc.update(dt, snapshot, arena, layout)
    };

    // On the final blow the fanfare replaces the crash, so the win lands clean
    // instead of the derezz thud sitting on top of it.
    let match_won = events.match_over == Some(DiscPhase::Won);
    if (events.opponent_hit || events.player_derezz.is_some()) && !match_won {
        effects.write(MusicSfx::Crash);
    }
    if events.shielded || events.player_recalled || events.opponent_threw {
        effects.write(MusicSfx::Turn);
    }
    for _ in &events.collected {
        effects.write(MusicSfx::Portal);
    }

    if let Some(reason) = events.player_derezz
        && run.sim.phase == RunPhase::Running
    {
        run.sim.phase = RunPhase::Crashed;
        run.sim.crash_reason = Some(reason);
        run.crash_label = Some(disc_crash_label(reason));
        run.entering_label = None;
    }

    if let Some((cell, heading)) = events.respawn {
        run.sim = LightcycleSim::start(cell, heading);
        run.crash_label = None;
        run.entering_label = None;
    }

    if let Some(phase) = events.match_over {
        match phase {
            DiscPhase::Lost => {
                if let RunEnvironment::Source { language, .. } = &run.environment {
                    run.crash_label = Some(language.crash_flavor(0).to_string());
                    run.entering_label = None;
                }
            }
            DiscPhase::Won => {
                run.crash_label = None;
                run.entering_label = None;
                effects.write(MusicSfx::Victory);
            }
            DiscPhase::Fighting => {}
        }
    }
}
