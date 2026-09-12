//! Moved out of `super` by the modularity pass: restart_run, step_asteroid_field, step_bomberman, step_breaker, step_cell, step_columns, step_disc_fight, step_frogger, step_galaga, step_lightcycle, step_pacman, step_platformer, step_plinko, step_qbert, step_snake, step_stealth, step_surfer, step_tetris.
//!
//! Nothing about them changed in the move.

use super::*;

/// One cell along `heading`. This mirrors the sim's own step for a view-only
/// walk along a wall, so a mismatch could only ever misplace the camera.
pub(crate) fn step_cell(cell: (i32, i32), heading: Heading) -> (i32, i32) {
    match heading {
        Heading::PosX => (cell.0 + 1, cell.1),
        Heading::NegX => (cell.0 - 1, cell.1),
        Heading::PosZ => (cell.0, cell.1 + 1),
        Heading::NegZ => (cell.0, cell.1 - 1),
    }
}

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
                    *snake = SnakeSim::new(seed, spawn, &food_cells, config::SNAKE_FOOD_TARGET);
                }
                // Each sim re-rolls itself from its own stored seed, so a
                // restart lays out exactly the same level.
                SourceSim::Platformer(level) => level.restart(),
                SourceSim::Breaker(level) => level.restart(),
                SourceSim::Stealth(room) => room.restart(),
                SourceSim::Surfer(surfer) => surfer.restart(),
                SourceSim::Galaga(sim) => sim.restart(),
                SourceSim::PacMan(sim) => sim.restart(),
                SourceSim::Columns(sim) => sim.restart(),
                SourceSim::Tetris(sim) => sim.restart(),
                SourceSim::Frogger(sim) => sim.restart(),
                SourceSim::Qbert(sim) => sim.restart(),
                SourceSim::Bomberman(sim) => sim.restart(),
                SourceSim::Plinko(sim) => sim.restart(),
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
    let max_catch_up = config::LIGHTCYCLE_FIXED_STEP * config::LIGHTCYCLE_MAX_SUBSTEPS as f32;
    if state.clock > max_catch_up {
        state.clock = max_catch_up;
    }

    let fixed_step = config::LIGHTCYCLE_FIXED_STEP;
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
                state.gc_timer = config::GC_INTERVAL_SECONDS;
                state.gc_pause = config::GC_PAUSE_SECONDS;
                state.gc_sweep = config::GC_SWEEP_SECONDS;
                effects.write(MusicSfx::Seek);
            }
        }
    }

    // Bullet time stretches the simulated step without changing the real-time
    // cadence, so the bike, its disc, and the opponent all slow together.
    let mut step = if source_run && state.slow_motion {
        fixed_step * config::DISC_BULLET_TIME_SCALE
    } else {
        fixed_step
    };
    if !source_run && state.gc_pause > 0.0 {
        step *= config::GC_SLOW_SCALE;
    }
    if !source_run && state.cache_boost > 0.0 {
        let heat = state.cache_boost / config::CACHE_BOOST_SECONDS;
        step *= 1.0 + (config::CACHE_BOOST_SCALE - 1.0) * heat;
    }
    let mut substeps = 0;

    while state.clock >= fixed_step && substeps < config::LIGHTCYCLE_MAX_SUBSTEPS {
        state.clock -= fixed_step;
        substeps += 1;

        let outcome = {
            let arena = &run.arena;
            let sim = &mut run.sim;
            match &run.environment {
                RunEnvironment::Directory { nodes, cells } => {
                    sim.advance(step * config::LIGHTCYCLE_CELLS_PER_SEC, |next, sim| {
                        classify_next_content(
                            next,
                            arena,
                            sim,
                            cells,
                            |index| nodes[index].is_dir,
                            |index| nodes[index].is_markdown(),
                            |index| nodes[index].is_source(),
                        )
                    })
                }
                RunEnvironment::Document { .. } => {
                    sim.advance(step * config::LIGHTCYCLE_CELLS_PER_SEC, |next, sim| {
                        classify_next_content(
                            next,
                            arena,
                            sim,
                            &HashMap::new(),
                            |_| false,
                            |_| false,
                            |_| false,
                        )
                    })
                }
                RunEnvironment::Source { sim: source, .. } => match source {
                    // Parked while the rocks are live: nothing to advance, and
                    // the field itself is stepped after the loop.
                    SourceSim::Asteroids(field) if field.is_active() => StepOutcome::Moved,
                    // Once the field is decided the cycle is handed back, so it
                    // drives again and can ride out through the gate.
                    SourceSim::Asteroids(_) => {
                        sim.advance(step * config::LIGHTCYCLE_CELLS_PER_SEC, |next, state| {
                            classify_next_content(
                                next,
                                arena,
                                state,
                                &HashMap::new(),
                                |_| false,
                                |_| false,
                                |_| false,
                            )
                        })
                    }
                    // Snake drives the ordinary grid, but the exit is a solid
                    // wall until enough power-ups have been eaten. The tail is
                    // capped after every step so it stays finite.
                    SourceSim::Snake(snake) => {
                        let locked = !snake.exit_open;
                        let outcome =
                            sim.advance(step * config::LIGHTCYCLE_CELLS_PER_SEC, |next, state| {
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
                            });
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
                        sim.advance(step * config::LIGHTCYCLE_CELLS_PER_SEC, |next, state| {
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
                        })
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
                        config::LIGHTCYCLE_ENTRY_FX_DURATION,
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
                Some(SourceGame::Platformer) => step_platformer(&mut run, step, &mut effects),
                Some(SourceGame::Breaker) => step_breaker(&mut run, step, &mut effects),
                Some(SourceGame::Stealth) => step_stealth(&mut run, step, &mut effects),
                Some(SourceGame::RiverSurfer) => step_surfer(&mut run, step, &mut effects),
                Some(SourceGame::Galaga) => step_galaga(&mut run, step, &mut effects),
                Some(SourceGame::PacMan) => step_pacman(&mut run, step, &mut effects),
                Some(SourceGame::Columns) => step_columns(&mut run, step, &mut effects),
                Some(SourceGame::Tetris) => step_tetris(&mut run, step, &mut effects),
                Some(SourceGame::Frogger) => step_frogger(&mut run, step, &mut effects),
                Some(SourceGame::Qbert) => step_qbert(&mut run, step, &mut effects),
                Some(SourceGame::Bomberman) => step_bomberman(&mut run, step, &mut effects),
                Some(SourceGame::Plinko) => step_plinko(&mut run, step, &mut effects),
                Some(SourceGame::DiscWars) => {
                    step_disc_fight(&mut run, step, &mut effects);
                    false
                }
                None => false,
            };
            // Clearing a level is this run's version of riding out the gate.
            if cleared {
                state.restore_directory = true;
            }
        }

        if run.sim.phase == RunPhase::Crashed && state.crash_fx.is_none() {
            state.crash_fx = Some(crate::lightcycle::CrashFx::new(
                config::LIGHTCYCLE_CRASH_FX_DURATION,
            ));
            effects.write(MusicSfx::Crash);
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

/// Steps a platformer level. Returns `true` on the frame the exit is reached,
/// which hands the run back to the directory it came from.
pub(crate) fn step_platformer(
    run: &mut ActiveRun,
    dt: f32,
    effects: &mut MessageWriter<MusicSfx>,
) -> bool {
    let (events, fell) = {
        let Some(level) = run.source_platformer_mut() else {
            return false;
        };
        let events = level.update(dt);
        (events, level.phase == PlatformerPhase::Lost)
    };
    if events.jumped {
        effects.write(MusicSfx::Zap);
    }
    if events.won {
        effects.write(MusicSfx::Victory);
    }
    // A fall ends the run through the shared crash path, so the burst, the
    // shake, the label and `R` all behave like any other crash.
    if fell {
        crash_source(run, "the void under the level", effects);
    }
    events.won
}

/// Steps a breaker court. Returns `true` when the wall is cleared, which hands
/// the run back to the directory.
pub(crate) fn step_breaker(
    run: &mut ActiveRun,
    dt: f32,
    effects: &mut MessageWriter<MusicSfx>,
) -> bool {
    let (events, missed) = {
        let Some(level) = run.source_breaker_mut() else {
            return false;
        };
        let events = level.update(dt);
        (events, level.phase == BreakerPhase::Missed)
    };
    if events.launched {
        effects.write(MusicSfx::Beam);
    }
    if events.bounced_off_paddle {
        effects.write(MusicSfx::Turn);
    }
    for _ in 0..events.broke_bricks {
        effects.write(MusicSfx::Portal);
    }
    if events.cleared {
        effects.write(MusicSfx::Victory);
    }
    // The wall below the bike is the one that ends it.
    if missed {
        crash_source(run, "the ball past the bike", effects);
    }
    events.cleared
}

/// Steps a stealth run. Returns `true` when the character reaches the door.
pub(crate) fn step_stealth(
    run: &mut ActiveRun,
    dt: f32,
    effects: &mut MessageWriter<MusicSfx>,
) -> bool {
    let (events, caught) = {
        let Some(room) = run.source_stealth_mut() else {
            return false;
        };
        let events = room.update(dt);
        (events, room.phase == StealthPhase::Caught)
    };
    if events.spotted {
        effects.write(MusicSfx::Zap);
    }
    if events.escaped {
        effects.write(MusicSfx::Victory);
    }
    if caught {
        crash_source(run, "a patrol", effects);
    }
    events.escaped
}

/// Steps a river surfer run. Returns `true` when the bike crosses the finish.
pub(crate) fn step_surfer(
    run: &mut ActiveRun,
    dt: f32,
    effects: &mut MessageWriter<MusicSfx>,
) -> bool {
    let (events, crashed) = {
        let Some(surfer) = run.source_surfer_mut() else {
            return false;
        };
        let events = surfer.update(dt);
        (events, surfer.phase == SurferPhase::Crashed)
    };
    if events.boosted {
        effects.write(MusicSfx::Beam);
    }
    if events.finished {
        effects.write(MusicSfx::Victory);
    }
    if crashed {
        let label = if events.banked {
            "the riverbank"
        } else {
            "a rock in the river"
        };
        crash_source(run, label, effects);
    }
    events.finished
}

/// Steps a Galaga field. Returns `true` when the formation is cleared, which
/// hands the run back to the directory.
pub(crate) fn step_galaga(
    run: &mut ActiveRun,
    dt: f32,
    effects: &mut MessageWriter<MusicSfx>,
) -> bool {
    let (events, lost) = {
        let Some(sim) = run.source_galaga_mut() else {
            return false;
        };
        let events = sim.update(dt);
        (events, sim.phase == GalagaPhase::Lost)
    };
    if events.fired {
        effects.write(MusicSfx::Beam);
    }
    for _ in 0..events.killed {
        effects.write(MusicSfx::Portal);
    }
    if events.lost_life {
        effects.write(MusicSfx::Crash);
    }
    if events.cleared {
        effects.write(MusicSfx::Victory);
    }
    if lost {
        let label = if events.overrun {
            "the swarm reached the cycle"
        } else {
            "the swarm"
        };
        crash_source(run, label, effects);
    }
    events.cleared
}

/// Steps a Pac-Man maze. Returns `true` when every dot is eaten.
pub(crate) fn step_pacman(
    run: &mut ActiveRun,
    dt: f32,
    effects: &mut MessageWriter<MusicSfx>,
) -> bool {
    let (events, caught) = {
        let Some(sim) = run.source_pacman_mut() else {
            return false;
        };
        let events = sim.update(dt);
        (events, sim.phase == PacPhase::Caught)
    };
    if events.dots > 0 {
        effects.write(MusicSfx::Portal);
    }
    if events.lost_life {
        effects.write(MusicSfx::Crash);
    }
    if events.cleared {
        effects.write(MusicSfx::Victory);
    }
    if caught {
        crash_source(run, "a ghost in the maze", effects);
    }
    events.cleared
}

/// Steps a Columns well. Returns `true` when the well is empty.
pub(crate) fn step_columns(
    run: &mut ActiveRun,
    dt: f32,
    effects: &mut MessageWriter<MusicSfx>,
) -> bool {
    let (events, lost) = {
        let Some(sim) = run.source_columns_mut() else {
            return false;
        };
        let events = sim.update(dt);
        (events, sim.phase == ColumnsPhase::Lost)
    };
    if events.matched > 0 {
        effects.write(MusicSfx::Portal);
    }
    if events.landed {
        effects.write(MusicSfx::Beam);
    }
    if events.cleared {
        effects.write(MusicSfx::Victory);
    }
    if lost {
        crash_source(run, "the gem well", effects);
    }
    events.cleared
}

/// Steps a Tetris board. Returns `true` once the line target is met.
pub(crate) fn step_tetris(
    run: &mut ActiveRun,
    dt: f32,
    effects: &mut MessageWriter<MusicSfx>,
) -> bool {
    let (events, lost) = {
        let Some(sim) = run.source_tetris_mut() else {
            return false;
        };
        let events = sim.update(dt);
        (events, sim.phase == TetrisPhase::Lost)
    };
    if events.lines > 0 {
        effects.write(MusicSfx::Portal);
    }
    if events.locked {
        effects.write(MusicSfx::Beam);
    }
    if events.cleared {
        effects.write(MusicSfx::Victory);
    }
    if lost {
        crash_source(run, "the stack of indentation", effects);
    }
    events.cleared
}

/// Steps a Frogger highway. Returns `true` when the far row is reached.
pub(crate) fn step_frogger(
    run: &mut ActiveRun,
    dt: f32,
    effects: &mut MessageWriter<MusicSfx>,
) -> bool {
    let (events, splatted) = {
        let Some(sim) = run.source_frogger_mut() else {
            return false;
        };
        let events = sim.update(dt);
        (events, sim.phase == FroggerPhase::Splatted)
    };
    if events.splatted {
        effects.write(MusicSfx::Crash);
    }
    if events.cleared {
        effects.write(MusicSfx::Victory);
    }
    if splatted {
        crash_source(run, "the async highway", effects);
    }
    events.cleared
}

/// Steps a Q*bert pyramid. Returns `true` once every cube is lit.
pub(crate) fn step_qbert(
    run: &mut ActiveRun,
    dt: f32,
    effects: &mut MessageWriter<MusicSfx>,
) -> bool {
    let (events, lost) = {
        let Some(sim) = run.source_qbert_mut() else {
            return false;
        };
        let events = sim.update(dt);
        (events, sim.phase == QbertPhase::Lost)
    };
    if events.lost_life {
        effects.write(MusicSfx::Crash);
    }
    if events.cleared {
        effects.write(MusicSfx::Victory);
    }
    if lost {
        crash_source(run, "the pyramid edge", effects);
    }
    events.cleared
}

/// Steps a Bomberman room. Returns `true` once the exit is reached.
pub(crate) fn step_bomberman(
    run: &mut ActiveRun,
    dt: f32,
    effects: &mut MessageWriter<MusicSfx>,
) -> bool {
    let (events, lost) = {
        let Some(sim) = run.source_bomberman_mut() else {
            return false;
        };
        let events = sim.update(dt);
        (events, sim.phase == BomberPhase::Lost)
    };
    if events.crates > 0 {
        effects.write(MusicSfx::Portal);
    }
    if events.lost_life {
        effects.write(MusicSfx::Crash);
    }
    if events.cleared {
        effects.write(MusicSfx::Victory);
    }
    if lost {
        crash_source(run, "your own bomb", effects);
    }
    events.cleared
}

/// Steps a Plinko board. Returns `true` when the rack beats the target.
pub(crate) fn step_plinko(
    run: &mut ActiveRun,
    dt: f32,
    effects: &mut MessageWriter<MusicSfx>,
) -> bool {
    let (events, lost) = {
        let Some(sim) = run.source_plinko_mut() else {
            return false;
        };
        let events = sim.update(dt);
        (events, sim.phase == PlinkoPhase::Lost)
    };
    if events.scored > 0 {
        effects.write(MusicSfx::Beam);
    }
    if events.cleared {
        effects.write(MusicSfx::Victory);
    }
    if lost {
        crash_source(run, "the data", effects);
    }
    events.cleared
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
