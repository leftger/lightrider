//! Bevy-free side-scrolling platformer over a procedurally generated level.
//!
//! The level is a run of flat platforms marching along `+X`, each a little
//! higher or lower than the last, with gaps that always stay inside what the
//! runner's jump can clear. Reaching the door at the far end leaves the run;
//! falling off the bottom ends it.
//!
//! Everything lives in metres on the `X`/`Y` plane (`Z` is unused), which is why
//! this module has nothing to do with the cell grid the bike games share.

use crate::config;
use crate::minigame::{GameInput, GameSound, GameTick, SourceGameSim};
use crate::rng::Rng;

/// One flat platform. `y` is its top surface, `x` its left edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Platform {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Platform {
    fn right(&self) -> f32 {
        self.x + self.w
    }

    fn bottom(&self) -> f32 {
        self.y - self.h
    }
}

/// The runner's body and momentum. `y` is the feet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Runner {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub on_ground: bool,
    /// `1.0` facing `+X`, `-1.0` facing `-X`.
    pub facing: f32,
}

impl Runner {
    fn half_width() -> f32 {
        config::PLATFORMER_RUNNER_WIDTH * 0.5
    }

    fn top(&self) -> f32 {
        self.y + config::PLATFORMER_RUNNER_HEIGHT
    }

    fn left(&self) -> f32 {
        self.x - Self::half_width()
    }

    fn right(&self) -> f32 {
        self.x + Self::half_width()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformerPhase {
    Running,
    /// Touched the door at the end of the level.
    Won,
    /// Fell into the gap below the level.
    Lost,
}

impl PlatformerPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Running => "RUNNING",
            Self::Won => "CLEARED",
            Self::Lost => "FELL",
        }
    }
}

/// What one fixed step produced, for sound and labels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlatformerEvents {
    pub jumped: bool,
    pub landed: bool,
    pub won: bool,
    pub fell: bool,
}

/// One input frame: a walk axis and an edge-triggered jump.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PlatformerInput {
    /// `-1.0`, `0.0` or `1.0`.
    pub walk: f32,
    pub jump: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlatformerSim {
    pub platforms: Vec<Platform>,
    pub runner: Runner,
    /// Bottom-centre of the exit door.
    pub exit: (f32, f32),
    pub phase: PlatformerPhase,
    /// Level length in metres, for the HUD progress readout.
    pub length: f32,
    /// Latched from the keyboard each frame. `jump` is an edge the first step
    /// that can use it consumes; the walk axis persists across substeps.
    pub input: PlatformerInput,
    seed: u64,
}

impl PlatformerSim {
    /// Builds a level from `seed`. The same file always produces the same
    /// level, and every gap is inside the runner's jump envelope.
    pub fn new(seed: u64, metres: f32) -> Self {
        let length = metres.clamp(config::PLATFORMER_LENGTH_MIN, config::PLATFORMER_LENGTH_MAX);
        let platforms = generate(seed, length);
        let last = *platforms.last().expect("a level always has a start pad");
        let exit = (last.x + last.w * 0.5, last.y);
        let start = platforms[0];
        Self {
            platforms,
            runner: Runner {
                x: start.x + 1.5,
                y: start.y,
                vx: 0.0,
                vy: 0.0,
                on_ground: true,
                facing: 1.0,
            },
            exit,
            phase: PlatformerPhase::Running,
            length,
            input: PlatformerInput::default(),
            seed,
        }
    }

    /// Where the floor gives up: anything below this is the void.
    pub fn kill_plane(&self) -> f32 {
        self.platforms
            .iter()
            .map(|platform| platform.bottom())
            .fold(f32::MAX, f32::min)
            - config::PLATFORMER_KILL_DEPTH
    }

    /// How far along the level the runner is, in `0.0..=1.0`.
    pub fn progress(&self) -> f32 {
        (self.runner.x / self.exit.0.max(1.0)).clamp(0.0, 1.0)
    }

    /// The door's footprint, as a platform-like box, for rendering and hits.
    pub fn exit_box(&self) -> Platform {
        Platform {
            x: self.exit.0 - config::PLATFORMER_EXIT_WIDTH * 0.5,
            y: self.exit.1 + config::PLATFORMER_EXIT_HEIGHT,
            w: config::PLATFORMER_EXIT_WIDTH,
            h: config::PLATFORMER_EXIT_HEIGHT,
        }
    }

    /// Latches a frame's input. `walk` is held, so it simply replaces what was
    /// there; `jump` is an edge and accumulates until a step consumes it.
    ///
    /// The edge has to be sticky because input is read once per rendered frame
    /// while steps only run on a 60Hz accumulator: above 60Hz some frames run
    /// no step at all, and a press latched on one of those would otherwise be
    /// overwritten by the next frame's release before anything could see it.
    pub fn set_input(&mut self, walk: f32, jump: bool) {
        self.input.walk = walk;
        self.input.jump |= jump;
    }

    /// Advances one fixed step from the latched [`Self::input`].
    pub fn update(&mut self, dt: f32) -> PlatformerEvents {
        // Consume the jump edge but keep walking for the rest of the frame's
        // substeps.
        let input = std::mem::take(&mut self.input);
        self.input.walk = input.walk;

        let mut events = PlatformerEvents::default();
        if self.phase != PlatformerPhase::Running {
            return events;
        }

        let walk = input.walk.clamp(-1.0, 1.0);
        if walk != 0.0 {
            self.runner.facing = walk.signum();
        }

        // Snappy on the ground, floaty in the air.
        let control = if self.runner.on_ground { 1.0 } else { 0.55 };
        self.runner.vx += walk * config::PLATFORMER_WALK_ACCEL * control * dt;
        if walk == 0.0 && self.runner.on_ground {
            let damping = config::PLATFORMER_FRICTION * dt;
            self.runner.vx -= self.runner.vx.clamp(-damping, damping);
        }
        self.runner.vx = self.runner.vx.clamp(
            -config::PLATFORMER_WALK_SPEED,
            config::PLATFORMER_WALK_SPEED,
        );

        if input.jump && self.runner.on_ground {
            self.runner.vy = config::PLATFORMER_JUMP_SPEED;
            self.runner.on_ground = false;
            events.jumped = true;
        }

        self.runner.vy -= config::PLATFORMER_GRAVITY * dt;

        let was_grounded = self.runner.on_ground;
        self.move_x(dt);
        self.move_y(dt);
        if self.runner.on_ground && !was_grounded {
            events.landed = true;
        }

        if self.runner.y < self.kill_plane() {
            self.phase = PlatformerPhase::Lost;
            events.fell = true;
            return events;
        }
        if overlaps(&self.runner, &self.exit_box()) {
            self.phase = PlatformerPhase::Won;
            events.won = true;
        }
        events
    }

    /// Horizontal pass: move, then push back out of anything solid.
    ///
    /// A shallow overlap at a platform's top is a landing in progress, not a
    /// wall, so it is left for the vertical pass. Without this the runner is
    /// shoved off every ledge it tries to land on.
    fn move_x(&mut self, dt: f32) {
        self.runner.x += self.runner.vx * dt;
        for platform in self.platforms.clone() {
            if !overlaps(&self.runner, &platform) {
                continue;
            }
            if self.runner.y >= platform.y - config::PLATFORMER_STEP_TOLERANCE {
                continue;
            }
            if self.runner.vx > 0.0 {
                self.runner.x = platform.x - Runner::half_width();
            } else if self.runner.vx < 0.0 {
                self.runner.x = platform.right() + Runner::half_width();
            }
            self.runner.vx = 0.0;
        }
    }

    /// Vertical pass: land on tops, bump heads on undersides.
    fn move_y(&mut self, dt: f32) {
        self.runner.y += self.runner.vy * dt;
        self.runner.on_ground = false;
        for platform in self.platforms.clone() {
            if !overlaps(&self.runner, &platform) {
                continue;
            }
            if self.runner.vy <= 0.0 {
                self.runner.y = platform.y;
                self.runner.vy = 0.0;
                self.runner.on_ground = true;
            } else if self.runner.y < platform.bottom() {
                // Genuinely underneath: bump the head on the underside.
                self.runner.y = platform.bottom() - config::PLATFORMER_RUNNER_HEIGHT;
                self.runner.vy = 0.0;
            } else {
                // Rising beside a ledge: stop against it. Treating this as a
                // head bump would fling the runner below the ledge it is
                // standing next to.
                self.runner.vx = 0.0;
                self.runner.vy = 0.0;
            }
        }
    }

    /// Restarts the level from the same seed, as `R` does.
    pub fn restart(&mut self) {
        let fresh = Self::new(self.seed, self.length);
        self.platforms = fresh.platforms;
        self.runner = fresh.runner;
        self.exit = fresh.exit;
        self.phase = PlatformerPhase::Running;
        self.input = PlatformerInput::default();
    }
}

fn overlaps(runner: &Runner, platform: &Platform) -> bool {
    runner.right() > platform.x
        && runner.left() < platform.right()
        && runner.top() > platform.bottom()
        && runner.y < platform.y
}

/// Lays platforms end to end with gaps and steps the jump can always clear.
fn generate(seed: u64, length: f32) -> Vec<Platform> {
    let thickness = config::PLATFORMER_PLATFORM_THICKNESS;
    let mut platforms = vec![Platform {
        x: 0.0,
        y: 0.0,
        w: config::PLATFORMER_START_PAD,
        h: thickness,
    }];
    let mut rng = Rng::from_state(seed | 1);
    let mut cursor = config::PLATFORMER_START_PAD;
    let mut y = 0.0;

    while cursor < length {
        let gap = config::PLATFORMER_MIN_GAP
            + rng.unit() * (config::PLATFORMER_MAX_GAP - config::PLATFORMER_MIN_GAP);
        let step = (rng.unit() * 2.0 - 1.0) * config::PLATFORMER_MAX_STEP;
        y = (y + step).clamp(0.0, config::PLATFORMER_HEIGHT_MAX);
        let w = config::PLATFORMER_MIN_WIDTH
            + rng.unit() * (config::PLATFORMER_MAX_WIDTH - config::PLATFORMER_MIN_WIDTH);
        platforms.push(Platform {
            x: cursor + gap,
            y,
            w,
            h: thickness,
        });
        cursor += gap + w;
    }

    // The goal pad, on the same level as whatever we ended on.
    let gap = config::PLATFORMER_MIN_GAP + rng.unit() * 1.5;
    platforms.push(Platform {
        x: cursor + gap,
        y,
        w: config::PLATFORMER_GOAL_WIDTH,
        h: thickness,
    });
    platforms
}

impl SourceGameSim for PlatformerSim {
    fn tick(&mut self, dt: f32) -> GameTick {
        let events = self.update(dt);
        let mut tick = GameTick::default();
        if events.jumped {
            tick.sound(GameSound::Zap);
        }
        if events.won {
            tick.sound(GameSound::Victory);
        }
        tick.cleared = events.won;
        if self.phase == PlatformerPhase::Lost {
            tick.lost = true;
            tick.label = Some("the void under the level".to_string());
        }
        tick
    }

    fn status_line(&self, ring: &str, language: &str, inner: &str) -> String {
        let mut status = format!(
            "PLATFORMER {}% | RING: {ring} | {language} | {inner}",
            (self.progress() * 100.0).round() as u32
        );
        status = format!("{status} | {}", self.phase.label());
        status
    }

    fn input(&mut self, input: &GameInput) {
        self.set_input(input.steer as f32, input.action);
    }
}

#[cfg(test)]
mod tests {
    use super::{PlatformerPhase, PlatformerSim};
    use crate::config;

    fn sim(seed: u64) -> PlatformerSim {
        PlatformerSim::new(seed, 120.0)
    }

    #[test]
    fn the_same_seed_builds_the_same_level() {
        assert_eq!(sim(7).platforms, sim(7).platforms);
        assert_ne!(sim(7).platforms, sim(8).platforms);
    }

    #[test]
    fn every_gap_and_step_is_jumpable() {
        // Two seconds of air time at full speed is the envelope; the generated
        // level must stay comfortably inside it.
        for seed in 0..24_u64 {
            let level = sim(seed);
            for pair in level.platforms.windows(2) {
                let (from, to) = (pair[0], pair[1]);
                let gap = to.x - (from.x + from.w);
                assert!(
                    gap <= config::PLATFORMER_MAX_GAP,
                    "seed {seed}: gap of {gap} is too wide to clear"
                );
                let rise = to.y - from.y;
                assert!(
                    rise <= config::PLATFORMER_MAX_STEP + f32::EPSILON,
                    "seed {seed}: step of {rise} is too tall to climb"
                );
            }
        }
    }

    #[test]
    fn gravity_lands_the_runner_on_the_start_pad() {
        let mut level = sim(3);
        level.runner.y += 3.0;
        level.runner.on_ground = false;
        for _ in 0..240 {
            level.update(1.0 / 60.0);
        }
        assert!(level.runner.on_ground, "the runner should settle");
        assert!((level.runner.y - 0.0).abs() < 0.01, "feet on the pad");
        assert_eq!(level.phase, PlatformerPhase::Running);
    }

    #[test]
    fn walking_off_the_start_pad_drops_the_runner() {
        let mut level = sim(3);
        // Walk left off the start pad: the level only extends to the right.
        for _ in 0..400 {
            level.set_input(-1.0, false);
            level.update(1.0 / 60.0);
            if level.phase == PlatformerPhase::Lost {
                break;
            }
        }
        assert_eq!(
            level.phase,
            PlatformerPhase::Lost,
            "the void under the level should end the run"
        );
    }

    #[test]
    fn a_jump_leaves_the_ground_and_comes_back() {
        let mut level = sim(1);
        level.set_input(0.0, true);
        let events = level.update(1.0 / 60.0);
        assert!(events.jumped);
        assert!(!level.runner.on_ground);
        let mut landed = false;
        for _ in 0..240 {
            level.set_input(0.0, false);
            if level.update(1.0 / 60.0).landed {
                landed = true;
                break;
            }
        }
        assert!(landed, "what goes up must come down");
    }

    #[test]
    fn a_jump_press_survives_a_frame_that_runs_no_step() {
        // Input is read once per rendered frame, but steps run on a 60Hz
        // accumulator, so a frame on a faster display can run no step at all.
        // Two latches back to back stand in for that: the press must outlive
        // the release that follows it.
        let mut level = sim(1);
        level.set_input(0.0, true);
        level.set_input(0.0, false);
        assert!(
            level.update(1.0 / 60.0).jumped,
            "the jump was swallowed by a frame that ran no step"
        );
        assert!(!level.runner.on_ground);
    }

    #[test]
    fn a_spent_jump_edge_does_not_fire_again() {
        let mut level = sim(1);
        level.set_input(0.0, true);
        assert!(level.update(1.0 / 60.0).jumped);
        // Back on the ground with nothing pressed: a sticky edge must not have
        // outlived the step that consumed it.
        for _ in 0..240 {
            level.set_input(0.0, false);
            assert!(!level.update(1.0 / 60.0).jumped, "a spent edge relaunched");
        }
    }

    #[test]
    fn every_jump_press_lands_at_any_refresh_rate() {
        // Replays the real frame loop: `read_lightcycle_input` latching once a
        // frame, then `step_lightcycle` draining a 1/60 accumulator. Before the
        // edge was made sticky, everything above 60Hz silently ate presses.
        let fixed_step = 1.0 / 60.0;
        let max_substeps = 4;

        for display_hz in [60.0_f32, 75.0, 120.0, 144.0, 240.0] {
            let mut level = sim(3);
            let mut clock = 0.0_f32;
            let (mut pressed, mut jumped) = (0, 0);

            // An odd cadence so presses land on both substep parities.
            for frame in 0..1200 {
                let press = frame % 31 == 0 && level.runner.on_ground;
                pressed += u32::from(press);
                level.set_input(0.0, press);

                clock = (clock + 1.0 / display_hz).min(fixed_step * max_substeps as f32);
                let mut substeps = 0;
                while clock >= fixed_step && substeps < max_substeps {
                    clock -= fixed_step;
                    substeps += 1;
                    jumped += u32::from(level.update(fixed_step).jumped);
                }
            }

            assert!(pressed > 0, "{display_hz}Hz: the harness never pressed");
            assert_eq!(
                pressed,
                jumped,
                "{display_hz}Hz: {} of {pressed} presses never became jumps",
                pressed - jumped
            );
        }
    }

    #[test]
    fn running_right_reaches_the_exit() {
        let mut level = sim(5);
        // Hold right. Jump when the ground runs out just ahead, unless the next
        // ledge is *below* this one, where simply walking off is enough and
        // jumping would overshoot it.
        let mut jumps = 0;
        for _ in 0..12_000 {
            let look = level.runner.x + 0.9;
            let ground_ahead = level
                .platforms
                .iter()
                .any(|platform| look >= platform.x && look <= platform.right());
            let next = level
                .platforms
                .iter()
                .filter(|platform| platform.x > level.runner.x)
                .min_by(|a, b| a.x.total_cmp(&b.x));
            let steps_down = next.is_some_and(|platform| platform.y < level.runner.y - 0.25);
            let jump = level.runner.on_ground && !ground_ahead && !steps_down;
            if jump {
                jumps += 1;
            }
            level.set_input(1.0, jump);
            level.update(1.0 / 60.0);
            if level.phase != PlatformerPhase::Running {
                break;
            }
        }
        assert_eq!(
            level.phase,
            PlatformerPhase::Won,
            "a runner that jumps every gap should finish (jumped {jumps} times)"
        );
        assert!(jumps > 3, "the level should have had obstacles to clear");
    }

    #[test]
    fn restarting_puts_the_runner_back_at_the_start() {
        let mut level = sim(11);
        level.runner.x = 40.0;
        level.phase = PlatformerPhase::Lost;
        level.restart();
        assert_eq!(level.phase, PlatformerPhase::Running);
        assert!(level.runner.x < 3.0);
        assert_eq!(level.platforms, sim(11).platforms);
    }
}
