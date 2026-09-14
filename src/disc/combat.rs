//! Bevy-free disc-wars combat, stepped on the shared lightcycle fixed clock.
//!
//! Movement of the *player* stays in [`crate::lightcycle::logic::LightcycleSim`]
//! so the cell grid, queued turns, trail, crash FX, chase camera and music
//! profile are all reused unchanged. This module adds only what the fight needs:
//! the thrown discs, the opponent's own movement and throwing, pickup effects,
//! the hazard fuse, and best-of-three round bookkeeping.
//!
//! Everything here is a pure function of its inputs so it can be unit-tested on
//! a plain thread, like `lightcycle::logic`.

use crate::config;
use crate::disc::layout::{DiscLayout, PickupKind};
use crate::grid::chebyshev;
use crate::lightcycle::logic::{Arena, CrashReason, Heading, Turn};

/// Snapshot of the player's shared cycle state for one step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerSnapshot {
    pub cell: (i32, i32),
    pub heading: Heading,
    /// False while the player's run is crashed or paused.
    pub running: bool,
    pub world_pos: Option<(f32, f32)>,
    pub world_dir: Option<(f32, f32)>,
}

/// Which side of the match is currently being decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscPhase {
    Fighting,
    Won,
    Lost,
}

impl DiscPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Fighting => "FIGHT",
            Self::Won => "WIN",
            Self::Lost => "LOSE",
        }
    }
}

/// One disc in flight.
#[derive(Debug, Clone, PartialEq)]
pub struct Disc {
    pub cell: (i32, i32),
    pub heading: Heading,
    /// Progress from the current cell toward the next, in cells.
    pub progress: f32,
    /// Cells left before the disc turns back (outbound) or expires (returning).
    pub budget: i32,
    pub returning: bool,
    /// Walls the disc may still phase through (Glitch).
    pub glitch_left: i32,
    /// Remaining 90-degree bounces at a wall (Fork).
    pub fork_left: i32,
    /// Lethal on the return path as well as the way out (Spike).
    pub spike: bool,
    pub speed: f32,
    pub world_pos: Option<(f32, f32)>,
    pub world_vel: Option<(f32, f32)>,
}

impl Disc {
    fn step_distance(&self) -> f32 {
        self.speed
    }

    fn has_spike(&self) -> bool {
        self.spike
    }
}

/// The Recognizer-style opponent: a disc-wielding NPC, not a second bike.
#[derive(Debug, Clone, PartialEq)]
pub struct Opponent {
    pub cell: (i32, i32),
    pub heading: Heading,
    pub alive: bool,
    pub move_clock: f32,
    pub throw_clock: f32,
    /// Seconds spent winding up a telegraphed throw. Zero outside a charge.
    pub windup: f32,
    pub disc: Option<Disc>,
}

/// Persistent player buffs collected from pickups.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscEffects {
    pub shield: u8,
    pub spike: bool,
    pub glitch: bool,
    pub fork: bool,
    pub heavy: bool,
    pub widens: i32,
    /// Seconds of intangibility left (Phase).
    pub invuln: f32,
    /// Hazard steps left before the fuse burns out.
    pub hazard_fuse: i32,
}

impl Default for DiscEffects {
    fn default() -> Self {
        Self {
            shield: 0,
            spike: false,
            glitch: false,
            fork: false,
            heavy: false,
            widens: 0,
            invuln: 0.0,
            hazard_fuse: config::disc::DISC_HAZARD_FUSE,
        }
    }
}

impl DiscEffects {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn apply(&mut self, kind: PickupKind) {
        match kind {
            PickupKind::Glitch => self.glitch = true,
            PickupKind::Split => self.widens += 1,
            PickupKind::Fork => self.fork = true,
            PickupKind::Heavy => self.heavy = true,
            PickupKind::Widen => self.widens += 1,
            PickupKind::Spike => self.spike = true,
            PickupKind::Shield => {
                self.shield = (self.shield + 1).min(config::disc::DISC_SHIELD_MAX)
            }
            PickupKind::Phase => self.invuln = config::disc::DISC_PHASE_SECONDS,
            PickupKind::Recharge => self.hazard_fuse = config::disc::DISC_HAZARD_FUSE,
        }
    }
}

/// What happened over one [`DiscSim::update`], for the Bevy layer to react to.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiscEvents {
    pub player_threw: bool,
    pub player_recalled: bool,
    pub opponent_threw: bool,
    pub opponent_hit: bool,
    pub shielded: bool,
    pub collected: Vec<PickupKind>,
    /// The player must derezz this frame (hazard, or an opponent disc).
    pub player_derezz: Option<CrashReason>,
    /// The round paused and the player should respawn at this cell/heading.
    pub respawn: Option<((i32, i32), Heading)>,
    /// The match ended on this frame.
    pub match_over: Option<DiscPhase>,
}

/// One match of disc wars over a generated ring.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscSim {
    pub phase: DiscPhase,
    pub round: u8,
    pub player_score: u8,
    pub opponent_score: u8,
    pub player_disc: Option<Disc>,
    pub opponent: Opponent,
    pub effects: DiscEffects,
    /// Pickup spot indices already collected.
    pub taken: Vec<usize>,
    pub player_spawn: (i32, i32),
    pub player_spawn_heading: Heading,
    /// Counts down between rounds; combat is paused while positive.
    pub round_delay: f32,
    /// A round ended and a respawn is owed once the delay elapses.
    pending_respawn: bool,
    /// Last player cell, so hazard cost is per cell entry, not per frame.
    last_cell: Option<(i32, i32)>,
    /// The most recent pickup the player collected, for the status line.
    pub last_pickup: Option<PickupKind>,
    /// How hard the opponent presses: dense files field an aggressive fighter,
    /// comment-heavy files a defensive one.
    pub aggression: f32,
}

impl DiscSim {
    pub fn new(layout: &DiscLayout) -> Self {
        let heading = layout.player_spawn_heading;
        let aggression =
            (layout.signals.density() - layout.signals.comment_ratio()).clamp(-1.0, 1.0);
        Self {
            phase: DiscPhase::Fighting,
            round: 1,
            player_score: 0,
            opponent_score: 0,
            player_disc: None,
            opponent: Opponent {
                cell: layout.opponent_spawn,
                heading: crate::disc::layout::heading_toward(
                    layout.opponent_spawn,
                    layout.player_spawn,
                ),
                alive: true,
                move_clock: 0.0,
                throw_clock: config::disc::DISC_SPAWN_GRACE,
                windup: 0.0,
                disc: None,
            },
            effects: DiscEffects::default(),
            taken: Vec::new(),
            player_spawn: layout.player_spawn,
            player_spawn_heading: heading,
            round_delay: 0.0,
            pending_respawn: false,
            last_cell: Some(layout.player_spawn),
            last_pickup: None,
            aggression,
        }
    }

    /// The cardinal a throw should leave along.
    ///
    /// Riding and aiming are decoupled: a clear shot down the opponent's row or
    /// column wins, otherwise the dominant axis toward them, so a thrown disc
    /// still travels their way when exact alignment is not available. Falls back
    /// to the rider's own facing when there is no opponent.
    pub fn aim_heading(&self, from: (i32, i32), facing: Heading, arena: &Arena) -> Heading {
        let Some(target) = self.opponent_cell() else {
            return facing;
        };
        if from.0 == target.0 && line_of_sight(arena, from, target) {
            return vertical_heading(from, target);
        }
        if from.1 == target.1 && line_of_sight(arena, from, target) {
            return horizontal_heading(from, target);
        }
        crate::disc::layout::heading_toward(from, target)
    }

    /// True when the rider has a clear shot at the opponent right now.
    pub fn has_clear_shot(&self, from: (i32, i32), arena: &Arena) -> bool {
        self.opponent_cell()
            .is_some_and(|target| line_of_sight(arena, from, target))
    }

    /// Throws the player's disc if it is ready, aimed at the opponent.
    /// Returns whether the disc left.
    pub fn throw_player(
        &mut self,
        player: PlayerSnapshot,
        arena: &Arena,
        events: &mut DiscEvents,
    ) -> bool {
        if self.phase != DiscPhase::Fighting || self.player_disc.is_some() || !player.running {
            return false;
        }
        let glitch = if self.effects.glitch { 1 } else { 0 };
        let fork = if self.effects.fork { 1 } else { 0 };
        let range = config::disc::DISC_RANGE
            + self.effects.widens * config::disc::DISC_RANGE_BONUS
            + if self.effects.heavy { 1 } else { 0 };
        let speed = if self.effects.heavy {
            config::disc::DISC_SPEED * config::disc::DISC_HEAVY_SPEED_SCALE
        } else {
            config::disc::DISC_SPEED
        };

        let (heading, world_pos, world_vel) = if let Some(dir) = player.world_dir {
            let len = (dir.0 * dir.0 + dir.1 * dir.1).sqrt();
            let norm_dir = if len > 1e-4 {
                (dir.0 / len, dir.1 / len)
            } else {
                let delta = player.heading.delta();
                (delta.0 as f32, delta.1 as f32)
            };
            let start_pos = player.world_pos.unwrap_or((
                player.cell.0 as f32 * config::GRID_SPACING,
                player.cell.1 as f32 * config::GRID_SPACING,
            ));
            let launch_pos = (
                start_pos.0 + norm_dir.0 * 1.2,
                start_pos.1 + norm_dir.1 * 1.2,
            );
            let speed_world = speed * config::GRID_SPACING;
            let vel = (norm_dir.0 * speed_world, norm_dir.1 * speed_world);
            (player.heading, Some(launch_pos), Some(vel))
        } else {
            (
                self.aim_heading(player.cell, player.heading, arena),
                None,
                None,
            )
        };

        self.player_disc = Some(Disc {
            cell: player.cell,
            heading,
            progress: 0.0,
            budget: range,
            returning: false,
            glitch_left: glitch,
            fork_left: fork,
            spike: self.effects.spike,
            speed,
            world_pos,
            world_vel,
        });
        // The one-shot throw blessings are spent.
        self.effects.glitch = false;
        self.effects.fork = false;
        events.player_threw = true;
        true
    }

    /// Calls the player's disc home early.
    pub fn recall_player(&mut self, events: &mut DiscEvents) -> bool {
        let Some(disc) = self.player_disc.as_mut() else {
            return false;
        };
        if disc.returning {
            return false;
        }
        disc.returning = true;
        disc.heading = disc.heading.opposite();
        disc.budget = config::disc::DISC_RANGE;
        if let (Some(_), Some(vel)) = (disc.world_pos, &mut disc.world_vel) {
            vel.0 = -vel.0;
            vel.1 = -vel.1;
        }
        events.player_recalled = true;
        true
    }

    /// The opponent's body blocks the player like a live obstacle.
    pub fn opponent_cell(&self) -> Option<(i32, i32)> {
        self.opponent.alive.then_some(self.opponent.cell)
    }

    /// The opponent's live disc is lethal to the player.
    #[allow(dead_code)]
    pub fn opponent_disc_cell(&self) -> Option<(i32, i32)> {
        self.opponent.disc.as_ref().map(|disc| disc.cell)
    }

    /// Advances the fight by `dt`. `player` is the shared cycle's current pose.
    pub fn update(
        &mut self,
        dt: f32,
        player: PlayerSnapshot,
        arena: &Arena,
        layout: &DiscLayout,
    ) -> DiscEvents {
        let mut events = DiscEvents::default();
        if dt <= 0.0 {
            return events;
        }

        self.effects.invuln = (self.effects.invuln - dt).max(0.0);

        // Between rounds, hold everything still and pay out the respawn.
        if self.round_delay > 0.0 {
            self.round_delay -= dt;
            if self.round_delay <= 0.0 && self.pending_respawn {
                self.pending_respawn = false;
                self.respawn_round(layout);
                events.respawn = Some((self.player_spawn, self.player_spawn_heading));
            }
            return events;
        }

        if self.phase != DiscPhase::Fighting {
            return events;
        }

        if !player.running {
            self.lose_round(&mut events);
            return events;
        }

        self.tick_hazards(player, layout, &mut events);
        self.collect_pickups(player, layout, &mut events);
        self.update_player_disc(dt, player, arena, &mut events);
        self.update_opponent(dt, player, arena, layout, &mut events);
        self.update_opponent_disc(dt, player, arena, &mut events);
        self.check_body_contact(player, &mut events);
        events
    }

    /// Applies the hazard fuse for the cell the player just entered.
    fn tick_hazards(
        &mut self,
        player: PlayerSnapshot,
        layout: &DiscLayout,
        events: &mut DiscEvents,
    ) {
        if self.last_cell == Some(player.cell) {
            return;
        }
        self.last_cell = Some(player.cell);

        if layout.safe_pads.contains(&player.cell) {
            self.effects.hazard_fuse = config::disc::DISC_HAZARD_FUSE;
            return;
        }
        if !layout.hazards.contains(&player.cell) {
            self.effects.hazard_fuse = config::disc::DISC_HAZARD_FUSE;
            return;
        }

        self.effects.hazard_fuse -= 1;
        if self.effects.hazard_fuse <= 0 {
            events.player_derezz = Some(CrashReason::Hazard);
        }
    }

    fn collect_pickups(
        &mut self,
        player: PlayerSnapshot,
        layout: &DiscLayout,
        events: &mut DiscEvents,
    ) {
        let Some(index) = layout.pickup_at(player.cell) else {
            return;
        };
        if self.taken.contains(&index) {
            return;
        }
        self.taken.push(index);
        let kind = layout.pickups[index].kind;
        self.effects.apply(kind);
        self.last_pickup = Some(kind);
        events.collected.push(kind);
    }

    fn update_player_disc(
        &mut self,
        dt: f32,
        player: PlayerSnapshot,
        arena: &Arena,
        events: &mut DiscEvents,
    ) {
        let Some(mut disc) = self.player_disc.take() else {
            return;
        };
        let opponent_pos = if let Some(opp_cell) = self.opponent_cell() {
            (
                opp_cell.0 as f32 * config::GRID_SPACING,
                opp_cell.1 as f32 * config::GRID_SPACING,
            )
        } else {
            (f32::MAX, f32::MAX)
        };
        let outcome = if disc.world_pos.is_some() {
            advance_disc_physics(&mut disc, dt, arena, player, opponent_pos)
        } else {
            advance_disc(
                &mut disc,
                dt,
                arena,
                player.cell,
                DiscTarget::Opponent,
                self.opponent_cell().unwrap_or((i32::MAX, i32::MAX)),
            )
        };
        match outcome {
            DiscFlight::Flying => self.player_disc = Some(disc),
            DiscFlight::Caught | DiscFlight::Expired => {}
            DiscFlight::HitOpponent => {
                // Only an outbound disc hits, unless it carries Spike.
                if !disc.returning || disc.has_spike() {
                    self.opponent.alive = false;
                    self.opponent.disc = None;
                    events.opponent_hit = true;
                    self.win_round(events);
                } else {
                    self.player_disc = Some(disc);
                }
            }
            DiscFlight::HitPlayer => self.player_disc = Some(disc),
        }
    }

    fn update_opponent_disc(
        &mut self,
        dt: f32,
        player: PlayerSnapshot,
        arena: &Arena,
        events: &mut DiscEvents,
    ) {
        let Some(mut disc) = self.opponent.disc.take() else {
            return;
        };
        let outcome = advance_disc(
            &mut disc,
            dt,
            arena,
            self.opponent.cell,
            DiscTarget::Player,
            player.cell,
        );
        match outcome {
            DiscFlight::Flying => self.opponent.disc = Some(disc),
            DiscFlight::Caught | DiscFlight::Expired | DiscFlight::HitOpponent => {}
            DiscFlight::HitPlayer => {
                if self.effects.invuln > 0.0 {
                    // Phase floor: the disc passes straight through.
                } else if self.effects.shield > 0 {
                    self.effects.shield -= 1;
                    events.shielded = true;
                } else {
                    events.player_derezz = Some(CrashReason::Disc);
                }
            }
        }
    }

    /// Riding into the opponent's body derezzes the player; a live opponent disc
    /// on the player's cell is caught by the classifier during movement, so only
    /// the body needs checking here.
    fn check_body_contact(&mut self, player: PlayerSnapshot, events: &mut DiscEvents) {
        if self.opponent.alive
            && self.opponent.cell == player.cell
            && events.player_derezz.is_none()
        {
            events.player_derezz = Some(CrashReason::Opponent);
        }
    }

    fn update_opponent(
        &mut self,
        dt: f32,
        player: PlayerSnapshot,
        arena: &Arena,
        _layout: &DiscLayout,
        events: &mut DiscEvents,
    ) {
        if !self.opponent.alive {
            return;
        }

        self.opponent.throw_clock -= dt;
        let has_shot = line_of_sight(arena, self.opponent.cell, player.cell);
        let ready = self.opponent.disc.is_none() && self.opponent.throw_clock <= 0.0;
        let mut charging = false;
        if ready && has_shot {
            // Charge in the open before firing, so the shot is telegraphed.
            charging = true;
            self.opponent.windup += dt;
            if self.opponent.windup >= config::disc::DISC_OPPONENT_WINDUP {
                self.opponent.windup = 0.0;
                charging = false;
                self.opponent.disc = Some(Disc {
                    cell: self.opponent.cell,
                    heading: crate::disc::layout::heading_toward(self.opponent.cell, player.cell),
                    progress: 0.0,
                    budget: config::disc::DISC_RANGE,
                    returning: false,
                    glitch_left: 0,
                    fork_left: 0,
                    spike: false,
                    speed: config::disc::DISC_SPEED * 0.9,
                    world_pos: None,
                    world_vel: None,
                });
                self.opponent.throw_clock =
                    config::disc::DISC_OPPONENT_THROW_COOLDOWN * (1.0 - 0.3 * self.aggression);
                events.opponent_threw = true;
            }
        } else {
            self.opponent.windup = 0.0;
        }

        // Hold the line while charging: a deliberate pause the rider can read,
        // rather than a drive-by that also walks it off its own firing line.
        if !charging {
            self.opponent.move_clock +=
                config::disc::DISC_OPPONENT_SPEED * (1.0 + 0.3 * self.aggression) * dt;
        }
        let mut guard = 0;
        while self.opponent.move_clock >= 1.0 && guard < 4 {
            self.opponent.move_clock -= 1.0;
            guard += 1;
            let next = self.choose_opponent_step(player, arena);
            if let Some((cell, heading)) = next {
                self.opponent.cell = cell;
                self.opponent.heading = heading;
            } else {
                break;
            }
        }
    }

    fn choose_opponent_step(
        &self,
        player: PlayerSnapshot,
        arena: &Arena,
    ) -> Option<((i32, i32), Heading)> {
        let toward = crate::disc::layout::heading_toward(self.opponent.cell, player.cell);
        let dodge = self.player_disc.as_ref().and_then(|disc| {
            let aligned =
                disc.cell.0 == self.opponent.cell.0 || disc.cell.1 == self.opponent.cell.1;
            let close =
                chebyshev(disc.cell, self.opponent.cell) <= config::disc::DISC_OPPONENT_DODGE_RANGE;
            // Only flinch at a disc that is actually about to arrive; a distant
            // throw should not make it abandon its own firing line.
            (aligned && close)
                .then(|| crate::disc::layout::heading_toward(self.opponent.cell, disc.cell))
        });
        let mut candidates = vec![
            toward,
            toward.turn(Turn::Left),
            toward.turn(Turn::Right),
            toward.opposite(),
        ];
        if let Some(dodge) = dodge {
            // Sidestepping the incoming disc wins over walking into it.
            candidates.retain(|heading| *heading != dodge);
            candidates.push(dodge.turn(Turn::Left));
            candidates.push(dodge.turn(Turn::Right));
        }
        candidates.into_iter().find_map(|heading| {
            let (dx, dz) = heading.delta();
            let cell = (self.opponent.cell.0 + dx, self.opponent.cell.1 + dz);
            (arena.roads.contains(&cell) && cell != player.cell).then_some((cell, heading))
        })
    }

    fn win_round(&mut self, events: &mut DiscEvents) {
        if self.phase != DiscPhase::Fighting {
            return;
        }
        self.player_score += 1;
        if self.player_score >= config::disc::DISC_WIN_SCORE {
            self.phase = DiscPhase::Won;
            self.player_disc = None;
            events.match_over = Some(DiscPhase::Won);
            return;
        }
        self.round_delay = config::disc::DISC_ROUND_DELAY;
        self.pending_respawn = true;
    }

    fn lose_round(&mut self, events: &mut DiscEvents) {
        if self.phase != DiscPhase::Fighting || self.pending_respawn {
            return;
        }
        self.opponent_score += 1;
        if self.opponent_score >= config::disc::DISC_WIN_SCORE {
            self.phase = DiscPhase::Lost;
            events.match_over = Some(DiscPhase::Lost);
            return;
        }
        self.round_delay = config::disc::DISC_ROUND_DELAY;
        self.pending_respawn = true;
    }

    fn respawn_round(&mut self, layout: &DiscLayout) {
        self.round = (self.round + 1).min(config::disc::DISC_WIN_SCORE * 2 - 1);
        self.player_disc = None;
        self.effects.reset();
        self.last_cell = Some(self.player_spawn);
        self.opponent = Opponent {
            cell: layout.opponent_spawn,
            heading: crate::disc::layout::heading_toward(
                layout.opponent_spawn,
                layout.player_spawn,
            ),
            alive: true,
            move_clock: 0.0,
            throw_clock: config::disc::DISC_SPAWN_GRACE,
            windup: 0.0,
            disc: None,
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiscFlight {
    Flying,
    Caught,
    Expired,
    HitOpponent,
    HitPlayer,
}

/// What a disc is hunting. The player's disc hunts the opponent's body, and the
/// opponent's disc hunts the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiscTarget {
    Opponent,
    Player,
}

/// Moves one disc for `dt` and resolves what it ran into.
///
/// `thrower` is the cell the disc is trying to return to; reaching it while
/// returning is a catch. `target_kind` and `target` describe the body the disc
/// derezzes on contact. Hit resolution deliberately lives with the caller so a
/// returning disc can be non-lethal unless it carries Spike.
fn advance_disc(
    disc: &mut Disc,
    dt: f32,
    arena: &Arena,
    thrower: (i32, i32),
    target_kind: DiscTarget,
    target: (i32, i32),
) -> DiscFlight {
    disc.progress += disc.step_distance() * dt;
    let mut guard = 0;
    while disc.progress >= 1.0 && guard < 64 {
        disc.progress -= 1.0;
        guard += 1;

        if disc.returning && disc.cell == thrower {
            return DiscFlight::Caught;
        }

        let (dx, dz) = disc.heading.delta();
        let next = (disc.cell.0 + dx, disc.cell.1 + dz);
        let walkable = arena.roads.contains(&next);
        if !walkable {
            if disc.glitch_left > 0 {
                disc.glitch_left -= 1;
            } else if disc.fork_left > 0 && !disc.returning {
                disc.fork_left -= 1;
                disc.heading = fork_heading(disc.heading, arena, disc.cell);
            } else if !disc.returning {
                disc.returning = true;
                disc.heading = disc.heading.opposite();
                disc.budget = config::disc::DISC_RANGE;
                continue;
            } else {
                return DiscFlight::Expired;
            }
        }

        disc.cell = next;
        disc.budget -= 1;

        // The player's disc grazes the Recognizer's body: since the opponent
        // steps a whole cell at a time, an exact-cell rule makes a moving target
        // nearly unhittable. Its own disc keeps the exact rule.
        let slack = match target_kind {
            DiscTarget::Opponent => config::disc::DISC_PLAYER_HIT_SLACK,
            DiscTarget::Player => 0,
        };
        if chebyshev(disc.cell, target) <= slack {
            return match target_kind {
                DiscTarget::Opponent => DiscFlight::HitOpponent,
                DiscTarget::Player => DiscFlight::HitPlayer,
            };
        }

        if disc.budget <= 0 {
            if disc.returning {
                return DiscFlight::Expired;
            }
            disc.returning = true;
            disc.heading = disc.heading.opposite();
            disc.budget = config::disc::DISC_RANGE;
        }
    }
    DiscFlight::Flying
}

/// Advances a disc using continuous 2D physics: velocity, wall reflections,
/// Recognizer cylinder hit detection, and homing back to the player on return.
fn advance_disc_physics(
    disc: &mut Disc,
    dt: f32,
    arena: &Arena,
    player: PlayerSnapshot,
    opponent_pos: (f32, f32),
) -> DiscFlight {
    let mut pos = disc.world_pos.unwrap_or((
        disc.cell.0 as f32 * config::GRID_SPACING,
        disc.cell.1 as f32 * config::GRID_SPACING,
    ));
    let mut vel = disc.world_vel.unwrap_or_else(|| {
        let delta = disc.heading.delta();
        let speed_world = disc.speed * config::GRID_SPACING;
        (delta.0 as f32 * speed_world, delta.1 as f32 * speed_world)
    });
    let speed = (vel.0 * vel.0 + vel.1 * vel.1)
        .sqrt()
        .max(disc.speed * config::GRID_SPACING);

    let p_pos = player.world_pos.unwrap_or((
        player.cell.0 as f32 * config::GRID_SPACING,
        player.cell.1 as f32 * config::GRID_SPACING,
    ));

    if disc.returning {
        let dx = p_pos.0 - pos.0;
        let dz = p_pos.1 - pos.1;
        let dist = dx.hypot(dz);
        if dist < 1.4 {
            return DiscFlight::Caught;
        }
        if dist > 1e-4 {
            vel = (dx / dist * speed, dz / dist * speed);
        }
    }

    pos.0 += vel.0 * dt;
    pos.1 += vel.1 * dt;
    disc.progress += dt;
    disc.cell = (
        (pos.0 / config::GRID_SPACING).round() as i32,
        (pos.1 / config::GRID_SPACING).round() as i32,
    );

    // Opponent cylinder collision (RECOGNIZER_RADIUS + DISC_MESH_RADIUS)
    let opp_dx = pos.0 - opponent_pos.0;
    let opp_dz = pos.1 - opponent_pos.1;
    let hit_dist = config::disc::RECOGNIZER_RADIUS + config::disc::DISC_MESH_RADIUS + 0.35;
    if opp_dx.hypot(opp_dz) <= hit_dist {
        return DiscFlight::HitOpponent;
    }

    // Circular ring wall reflection
    let (cx, cz) = arena.center();
    let center = (
        cx as f32 * config::GRID_SPACING,
        cz as f32 * config::GRID_SPACING,
    );
    let half = (arena.max.0 - cx).max(arena.max.1 - cz);
    let radius = ((half - config::disc::DISC_GATE_DEPTH - 1).max(config::disc::DISC_RADIUS_MIN)
        as f32)
        * config::GRID_SPACING;

    let rel_x = pos.0 - center.0;
    let rel_z = pos.1 - center.1;
    let dist_from_center = rel_x.hypot(rel_z);
    let wall_bound = radius - 0.4;

    if dist_from_center >= wall_bound {
        if disc.glitch_left > 0 {
            disc.glitch_left -= 1;
        } else if disc.fork_left > 0 && !disc.returning {
            disc.fork_left -= 1;
            vel = (-vel.1, vel.0);
        } else if !disc.returning {
            let norm_x = -rel_x / dist_from_center;
            let norm_z = -rel_z / dist_from_center;
            let dot = vel.0 * norm_x + vel.1 * norm_z;
            if dot < 0.0 {
                vel.0 -= 2.0 * dot * norm_x;
                vel.1 -= 2.0 * dot * norm_z;
            }
            pos.0 = center.0 - norm_x * (wall_bound - 0.1);
            pos.1 = center.1 - norm_z * (wall_bound - 0.1);
            disc.budget -= 1;
            if disc.budget <= 0 {
                disc.returning = true;
            }
        } else {
            return DiscFlight::Expired;
        }
    } else if !disc.returning && arena.street_walls.contains(&disc.cell) {
        if disc.glitch_left > 0 {
            disc.glitch_left -= 1;
        } else {
            vel.0 = -vel.0;
            vel.1 = -vel.1;
            disc.returning = true;
        }
    }

    if disc.progress > 6.0 {
        return DiscFlight::Expired;
    }

    disc.world_pos = Some(pos);
    disc.world_vel = Some(vel);
    DiscFlight::Flying
}

/// Picks a free 90-degree turn for a forking disc; falls back to reversing.
fn fork_heading(heading: Heading, arena: &Arena, cell: (i32, i32)) -> Heading {
    let left = heading.turn(Turn::Left);
    let right = heading.turn(Turn::Right);
    let open = |heading: Heading| {
        let (dx, dz) = heading.delta();
        arena.roads.contains(&(cell.0 + dx, cell.1 + dz))
    };
    if open(left) {
        left
    } else if open(right) {
        right
    } else {
        heading.opposite()
    }
}

/// True when the two cells share a row or column with a clear road between them.
fn line_of_sight(arena: &Arena, from: (i32, i32), to: (i32, i32)) -> bool {
    if from.0 == to.0 {
        let (a, b) = (from.1.min(to.1), from.1.max(to.1));
        (a + 1..b).all(|z| arena.roads.contains(&(from.0, z)))
    } else if from.1 == to.1 {
        let (a, b) = (from.0.min(to.0), from.0.max(to.0));
        (a + 1..b).all(|x| arena.roads.contains(&(x, from.1)))
    } else {
        false
    }
}

/// Cardinal heading along the X axis toward `to`.
fn horizontal_heading(from: (i32, i32), to: (i32, i32)) -> Heading {
    if to.0 >= from.0 {
        Heading::PosX
    } else {
        Heading::NegX
    }
}

/// Cardinal heading along the Z axis toward `to`.
fn vertical_heading(from: (i32, i32), to: (i32, i32)) -> Heading {
    if to.1 >= from.1 {
        Heading::PosZ
    } else {
        Heading::NegZ
    }
}

#[cfg(test)]
mod tests {
    use super::{DiscPhase, DiscSim, PlayerSnapshot};
    use crate::config;
    use crate::disc::layout::{DiscLayout, build_disc_arena};
    use crate::filesystem::language::SourceLanguage;
    use crate::lightcycle::logic::{Arena, Heading, Turn};
    use std::path::Path;

    fn ring() -> (Arena, DiscLayout) {
        let source = "fn a() {}\nfn b() {}\n// TODO\nunsafe {}\nmatch x {}\npub fn c() {}\npanic!(\"x\");\n/// doc\n#[test]\nfn t() {}\n";
        build_disc_arena(
            Path::new("/src/main.rs"),
            SourceLanguage::Rust,
            source.as_bytes(),
        )
    }

    fn snapshot(sim: &LightcycleSimLike) -> PlayerSnapshot {
        PlayerSnapshot {
            cell: sim.cell,
            heading: sim.heading,
            running: sim.running,
            world_pos: None,
            world_dir: None,
        }
    }

    /// Tiny stand-in mirroring the fields of `LightcycleSim` we care about, so
    /// the tests do not need a full `LightcycleSim`.
    struct LightcycleSimLike {
        cell: (i32, i32),
        heading: Heading,
        running: bool,
    }

    fn player(cell: (i32, i32), heading: Heading) -> LightcycleSimLike {
        LightcycleSimLike {
            cell,
            heading,
            running: true,
        }
    }

    #[test]
    fn a_fresh_match_starts_scoreless_at_round_one() {
        let (_, layout) = ring();
        let sim = DiscSim::new(&layout);
        assert_eq!(sim.phase, DiscPhase::Fighting);
        assert_eq!(sim.round, 1);
        assert_eq!((sim.player_score, sim.opponent_score), (0, 0));
        assert!(sim.opponent.alive);
        assert_eq!(sim.opponent.cell, layout.opponent_spawn);
    }

    #[test]
    fn aim_assist_snaps_to_the_aligned_axis() {
        let (arena, layout) = ring();
        let mut sim = DiscSim::new(&layout);
        let spawn = layout.player_spawn;
        sim.opponent.cell = (spawn.0 + 3, spawn.1);
        sim.opponent.alive = true;
        // Facing across the opponent's row, not down it.
        assert_eq!(sim.aim_heading(spawn, Heading::PosZ, &arena), Heading::PosX);
        assert!(sim.has_clear_shot(spawn, &arena));
    }

    #[test]
    fn aim_assist_falls_back_to_the_dominant_axis() {
        let (arena, layout) = ring();
        let mut sim = DiscSim::new(&layout);
        let spawn = layout.player_spawn;
        sim.opponent.cell = (spawn.0 + 4, spawn.1 + 2);
        sim.opponent.alive = true;
        assert_eq!(sim.aim_heading(spawn, Heading::NegZ, &arena), Heading::PosX);
        assert!(!sim.has_clear_shot(spawn, &arena));
    }

    #[test]
    fn a_thrown_disc_is_aimed_even_when_the_bike_faces_away() {
        let (arena, layout) = ring();
        let mut sim = DiscSim::new(&layout);
        let spawn = layout.player_spawn;
        sim.opponent.cell = (spawn.0 + 3, spawn.1);
        sim.opponent.alive = true;
        let p = player(spawn, Heading::PosZ);
        let mut events = super::DiscEvents::default();
        sim.throw_player(snapshot(&p), &arena, &mut events);
        assert_eq!(sim.player_disc.as_ref().unwrap().heading, Heading::PosX);

        for _ in 0..120 {
            events = sim.update(1.0 / 60.0, snapshot(&p), &arena, &layout);
            if events.opponent_hit {
                break;
            }
        }
        assert!(events.opponent_hit, "the aimed disc should connect");
    }

    #[test]
    fn the_player_disc_grazes_a_body_a_cell_off_the_line() {
        let (arena, layout) = ring();
        let spawn = layout.player_spawn;
        let target = (spawn.0 + 3, spawn.1 + 1);
        let mut flying = disc((spawn.0 + 2, spawn.1), Heading::PosX, false, false);
        assert_eq!(
            super::advance_disc(
                &mut flying,
                1.0,
                &arena,
                spawn,
                super::DiscTarget::Opponent,
                target
            ),
            super::DiscFlight::HitOpponent
        );
    }

    #[test]
    fn the_opponent_disc_still_needs_the_exact_cell() {
        let (arena, layout) = ring();
        let spawn = layout.player_spawn;
        // Travels parallel to the player, one cell off: it must not connect.
        let mut flying = disc((spawn.0 + 2, spawn.1), Heading::NegX, false, false);
        assert_ne!(
            super::advance_disc(
                &mut flying,
                1.0,
                &arena,
                (spawn.0, spawn.1 + 6),
                super::DiscTarget::Player,
                (spawn.0, spawn.1 + 1)
            ),
            super::DiscFlight::HitPlayer
        );
    }

    #[test]
    fn the_opponent_ignores_a_distant_disc() {
        let (arena, layout) = ring();
        let mut sim = DiscSim::new(&layout);
        let spawn = layout.player_spawn;
        sim.opponent.cell = (spawn.0 + 3, spawn.1);
        // Aligned but far outside the flinch range: it should keep closing on
        // the player instead of sliding off its firing line.
        let mut far = disc((spawn.0 + 2, spawn.1), Heading::PosX, false, false);
        far.cell = (spawn.0 + 8, spawn.1);
        sim.player_disc = Some(far);
        let target = PlayerSnapshot {
            cell: spawn,
            heading: Heading::PosX,
            running: true,
            world_pos: None,
            world_dir: None,
        };
        let step = sim
            .choose_opponent_step(target, &arena)
            .expect("the lane toward the player is open");
        assert_eq!(step, ((spawn.0 + 2, spawn.1), Heading::NegX));
    }

    #[test]
    fn opponent_telegraphs_its_throw() {
        let (arena, layout) = ring();
        let mut sim = DiscSim::new(&layout);
        sim.opponent.cell = (layout.player_spawn.0 + 4, layout.player_spawn.1);
        let p = player(layout.player_spawn, Heading::NegX);
        let dt = 1.0 / 60.0;
        let mut elapsed = 0.0;
        let mut threw_at = None;
        for _ in 0..600 {
            let events = sim.update(dt, snapshot(&p), &arena, &layout);
            elapsed += dt;
            if events.opponent_threw {
                threw_at = Some(elapsed);
                break;
            }
        }
        let threw_at = threw_at.expect("the opponent should eventually shoot");
        assert!(
            threw_at
                >= config::disc::DISC_SPAWN_GRACE + config::disc::DISC_OPPONENT_WINDUP - dt * 2.0,
            "shot came at {threw_at}s with no telegraph"
        );
    }

    #[test]
    fn throwing_and_recalling_the_disc_is_stateful() {
        let (arena, layout) = ring();
        let mut sim = DiscSim::new(&layout);
        let mut events = super::DiscEvents::default();
        let p = player(layout.player_spawn, Heading::PosX);
        assert!(sim.throw_player(snapshot(&p), &arena, &mut events));
        assert!(sim.player_disc.is_some());
        assert!(
            !sim.throw_player(snapshot(&p), &arena, &mut events),
            "one disc at a time"
        );
        assert!(sim.recall_player(&mut events));
        assert!(sim.player_disc.as_ref().unwrap().returning);

        // Fly until the disc comes home or expires.
        for _ in 0..600 {
            sim.update(1.0 / 60.0, snapshot(&p), &arena, &layout);
            if sim.player_disc.is_none() {
                break;
            }
        }
        assert!(sim.player_disc.is_none(), "the disc should come home");
    }

    #[test]
    fn an_outbound_disc_derezzes_the_opponent() {
        let (arena, layout) = ring();
        let mut sim = DiscSim::new(&layout);
        // Line the opponent up directly in front of the player.
        sim.opponent.cell = (layout.player_spawn.0 + 3, layout.player_spawn.1);
        sim.opponent.alive = true;
        let p = player(layout.player_spawn, Heading::PosX);
        let mut events = super::DiscEvents::default();
        sim.throw_player(snapshot(&p), &arena, &mut events);

        for _ in 0..120 {
            events = sim.update(1.0 / 60.0, snapshot(&p), &arena, &layout);
            if events.opponent_hit {
                break;
            }
        }
        assert!(events.opponent_hit, "a thrown disc should connect");
        assert!(!sim.opponent.alive);
        assert_eq!(sim.player_score, 1);
    }

    #[test]
    fn a_returning_disc_is_harmless_without_spike() {
        let (arena, layout) = ring();
        let spawn = layout.player_spawn;
        // The rider has stepped off the return lane, so the disc's trip home can
        // cross the opponent instead of being caught first.
        let aside = (spawn.0, spawn.1 + 3);

        let mut sim = DiscSim::new(&layout);
        sim.opponent.cell = (spawn.0 - 1, spawn.1);
        sim.opponent.alive = true;
        sim.player_disc = Some(disc((spawn.0 + 1, spawn.1), Heading::NegX, true, false));
        let mut events = super::DiscEvents::default();
        let p = player(aside, Heading::PosZ);
        for _ in 0..120 {
            sim.update_player_disc(1.0 / 60.0, snapshot(&p), &arena, &mut events);
            if sim.player_disc.is_none() {
                break;
            }
        }
        assert!(!events.opponent_hit, "the return path needs Spike");
    }

    #[test]
    fn spike_makes_the_return_path_lethal() {
        let (arena, layout) = ring();
        let spawn = layout.player_spawn;
        let aside = (spawn.0, spawn.1 + 3);

        let mut sim = DiscSim::new(&layout);
        sim.opponent.cell = (spawn.0 - 1, spawn.1);
        sim.opponent.alive = true;
        sim.player_disc = Some(disc((spawn.0 + 1, spawn.1), Heading::NegX, true, true));
        let mut events = super::DiscEvents::default();
        let p = player(aside, Heading::PosZ);
        for _ in 0..120 {
            sim.update_player_disc(1.0 / 60.0, snapshot(&p), &arena, &mut events);
            if events.opponent_hit {
                break;
            }
        }
        assert!(events.opponent_hit, "spike should kill on the way back");
        assert_eq!(sim.player_score, 1);
    }

    #[test]
    fn an_opponent_disc_derezzes_the_player() {
        let (arena, layout) = ring();
        let mut sim = DiscSim::new(&layout);
        let p = player(layout.player_spawn, Heading::PosX);
        sim.opponent.cell = (layout.player_spawn.0 + 4, layout.player_spawn.1);
        sim.opponent.alive = true;
        sim.opponent.disc = Some(super::Disc {
            cell: (layout.player_spawn.0 + 1, layout.player_spawn.1),
            heading: Heading::NegX,
            progress: 0.0,
            budget: config::disc::DISC_RANGE,
            returning: false,
            glitch_left: 0,
            fork_left: 0,
            spike: false,
            speed: config::disc::DISC_SPEED,
            world_pos: None,
            world_vel: None,
        });

        let mut events = super::DiscEvents::default();
        for _ in 0..60 {
            events = sim.update(1.0 / 60.0, snapshot(&p), &arena, &layout);
            if events.player_derezz.is_some() {
                break;
            }
        }
        assert_eq!(
            events.player_derezz,
            Some(crate::lightcycle::logic::CrashReason::Disc)
        );
    }

    #[test]
    fn a_shield_absorbs_one_opponent_disc() {
        let (arena, layout) = ring();
        let mut sim = DiscSim::new(&layout);
        sim.effects.shield = 1;
        let p = player(layout.player_spawn, Heading::PosX);
        sim.opponent.disc = Some(super::Disc {
            cell: (layout.player_spawn.0 + 1, layout.player_spawn.1),
            heading: Heading::NegX,
            progress: 0.0,
            budget: config::disc::DISC_RANGE,
            returning: false,
            glitch_left: 0,
            fork_left: 0,
            spike: false,
            speed: config::disc::DISC_SPEED,
            world_pos: None,
            world_vel: None,
        });
        let mut events = super::DiscEvents::default();
        for _ in 0..30 {
            events = sim.update(1.0 / 60.0, snapshot(&p), &arena, &layout);
            if events.shielded {
                break;
            }
        }
        assert!(events.shielded);
        assert_eq!(sim.effects.shield, 0);
        assert!(events.player_derezz.is_none());
    }

    #[test]
    fn crossing_three_hazards_in_a_row_burns_the_fuse() {
        let (arena, layout) = ring();
        // Force a deterministic hazard lane next to the spawn.
        let mut layout = layout;
        let base = layout.player_spawn;
        layout.hazards = [
            (base.0 + 1, base.1),
            (base.0 + 2, base.1),
            (base.0 + 3, base.1),
        ]
        .into_iter()
        .collect();
        let mut sim = DiscSim::new(&layout);
        let mut events = super::DiscEvents::default();
        for step in 1..=3 {
            let p = player((base.0 + step, base.1), Heading::PosX);
            events = sim.update(1.0 / 60.0, snapshot(&p), &arena, &layout);
        }
        assert_eq!(
            events.player_derezz,
            Some(crate::lightcycle::logic::CrashReason::Hazard)
        );
    }

    #[test]
    fn leaving_a_hazard_resets_the_fuse() {
        let (arena, layout) = ring();
        let mut layout = layout;
        let base = layout.player_spawn;
        layout.hazards = [(base.0 + 1, base.1), (base.0 + 3, base.1)]
            .into_iter()
            .collect();
        let mut sim = DiscSim::new(&layout);
        let mut events = super::DiscEvents::default();
        for cell in [
            (base.0 + 1, base.1),
            (base.0 + 2, base.1),
            (base.0 + 3, base.1),
        ] {
            let p = player(cell, Heading::PosX);
            events = sim.update(1.0 / 60.0, snapshot(&p), &arena, &layout);
        }
        assert!(events.player_derezz.is_none(), "gaps recharge the fuse");
        assert_eq!(sim.effects.hazard_fuse, config::disc::DISC_HAZARD_FUSE - 1);
    }

    fn disc(cell: (i32, i32), heading: Heading, returning: bool, spike: bool) -> super::Disc {
        super::Disc {
            cell,
            heading,
            progress: 0.0,
            budget: config::disc::DISC_RANGE,
            returning,
            glitch_left: 0,
            fork_left: 0,
            spike,
            speed: config::disc::DISC_SPEED,
            world_pos: None,
            world_vel: None,
        }
    }

    #[test]
    fn best_of_three_ends_at_two_rounds_won() {
        let (_, layout) = ring();
        let mut sim = DiscSim::new(&layout);
        let mut events = super::DiscEvents::default();
        sim.win_round(&mut events);
        assert_eq!(sim.phase, DiscPhase::Fighting);
        assert_eq!(sim.player_score, 1);
        assert_eq!(events.match_over, None);

        sim.round_delay = 0.0;
        sim.pending_respawn = false;
        sim.opponent.alive = true;
        events = super::DiscEvents::default();
        sim.win_round(&mut events);
        assert_eq!(sim.phase, DiscPhase::Won);
        assert_eq!(events.match_over, Some(DiscPhase::Won));
    }

    #[test]
    fn losing_two_rounds_ends_the_match() {
        let (_, layout) = ring();
        let mut sim = DiscSim::new(&layout);
        let mut events = super::DiscEvents::default();
        sim.lose_round(&mut events);
        assert_eq!(sim.phase, DiscPhase::Fighting);
        assert_eq!(sim.opponent_score, 1);

        sim.round_delay = 0.0;
        sim.pending_respawn = false;
        events = super::DiscEvents::default();
        sim.lose_round(&mut events);
        assert_eq!(sim.phase, DiscPhase::Lost);
        assert_eq!(events.match_over, Some(DiscPhase::Lost));
    }

    #[test]
    fn opponent_throws_along_line_of_sight() {
        let (arena, layout) = ring();
        let mut sim = DiscSim::new(&layout);
        // Place the opponent on the player's row with clear road.
        sim.opponent.cell = (layout.player_spawn.0 + 4, layout.player_spawn.1);
        let p = player(layout.player_spawn, Heading::NegX);
        let mut threw = false;
        for _ in 0..240 {
            let events = sim.update(1.0 / 60.0, snapshot(&p), &arena, &layout);
            if events.opponent_threw {
                threw = true;
                break;
            }
        }
        assert!(threw, "the opponent should take the shot");
    }

    #[test]
    fn opponent_keeps_away_from_the_incoming_disc() {
        let (arena, layout) = ring();
        let mut sim = DiscSim::new(&layout);
        let spawn = layout.player_spawn;
        sim.opponent.cell = (spawn.0 + 3, spawn.1);
        // A disc already in the opponent's lane, closing on it.
        sim.player_disc = Some(disc((spawn.0 + 2, spawn.1), Heading::PosX, false, false));
        let p = player(spawn, Heading::NegX);
        let step = sim
            .choose_opponent_step(snapshot(&p), &arena)
            .expect("there is room to dodge");
        assert_ne!(step.0, (spawn.0 + 2, spawn.1), "walked into the disc");
        assert_ne!(step.0.1, spawn.1, "should leave the disc's lane");
    }

    #[test]
    fn a_player_crash_loses_the_round_and_awards_the_opponent() {
        let (arena, layout) = ring();
        let mut sim = DiscSim::new(&layout);
        let crashed = PlayerSnapshot {
            cell: layout.player_spawn,
            heading: Heading::PosX,
            running: false,
            world_pos: None,
            world_dir: None,
        };
        sim.update(1.0 / 60.0, crashed, &arena, &layout);
        assert_eq!(sim.opponent_score, 1);
        assert_eq!(sim.phase, DiscPhase::Fighting);
    }

    #[test]
    fn turn_is_available_for_opponent_steering() {
        // Guards the `Turn` import and the heading maths the AI relies on.
        assert_eq!(Heading::PosX.turn(Turn::Left), Heading::NegZ);
    }

    #[test]
    fn disc_wars_shoots_in_direction_of_travel_with_physics() {
        let (arena, layout) = ring();
        let mut sim = DiscSim::new(&layout);
        let mut events = super::DiscEvents::default();
        let angle = std::f32::consts::FRAC_PI_4; // 45 degrees
        let p = PlayerSnapshot {
            cell: layout.player_spawn,
            heading: Heading::PosX,
            running: true,
            world_pos: Some((0.0, 0.0)),
            world_dir: Some((angle.cos(), angle.sin())),
        };
        assert!(sim.throw_player(p, &arena, &mut events));
        let disc = sim.player_disc.as_ref().expect("disc thrown");
        let (vx, vz) = disc.world_vel.expect("physics velocity");
        let dir_angle = vz.atan2(vx);
        assert!(
            (dir_angle - angle).abs() < 1e-4,
            "disc must shoot directly in direction of travel"
        );

        // Advance with physics
        for _ in 0..10 {
            sim.update(1.0 / 60.0, p, &arena, &layout);
        }
        let flying = sim.player_disc.as_ref().expect("disc still flying");
        let (px, pz) = flying.world_pos.expect("physics pos");
        assert!(
            px > 0.0 && pz > 0.0,
            "disc must travel continuously along 45 degree angle"
        );
    }

    #[test]
    fn disc_wars_physics_disc_derezzes_recognizer() {
        let (arena, layout) = ring();
        let mut sim = DiscSim::new(&layout);
        let mut events = super::DiscEvents::default();
        sim.opponent.cell = (layout.player_spawn.0 + 4, layout.player_spawn.1);
        sim.opponent.alive = true;

        let p = PlayerSnapshot {
            cell: layout.player_spawn,
            heading: Heading::PosX,
            running: true,
            world_pos: Some((
                layout.player_spawn.0 as f32 * config::GRID_SPACING,
                layout.player_spawn.1 as f32 * config::GRID_SPACING,
            )),
            world_dir: Some((1.0, 0.0)), // shoot straight toward opponent
        };
        assert!(sim.throw_player(p, &arena, &mut events));

        for _ in 0..60 {
            events = sim.update(1.0 / 60.0, p, &arena, &layout);
            if events.opponent_hit {
                break;
            }
        }
        assert!(
            events.opponent_hit,
            "physics disc must hit the Recognizer cylinder"
        );
        assert!(!sim.opponent.alive, "opponent must be derezzed");
    }

    #[test]
    fn disc_wars_physics_disc_rebounds_off_ring_wall() {
        let (arena, layout) = ring();
        let mut sim = DiscSim::new(&layout);
        let mut events = super::DiscEvents::default();

        let p = PlayerSnapshot {
            cell: layout.player_spawn,
            heading: Heading::PosX,
            running: true,
            world_pos: Some((0.0, 0.0)),
            world_dir: Some((1.0, 0.0)), // shoot toward right wall
        };
        assert!(sim.throw_player(p, &arena, &mut events));

        let initial_vx = sim.player_disc.as_ref().unwrap().world_vel.unwrap().0;
        assert!(initial_vx > 0.0);

        // Step enough to reach the wall and bounce
        for _ in 0..120 {
            sim.update(1.0 / 60.0, p, &arena, &layout);
            if let Some(ref d) = sim.player_disc
                && let Some(vel) = d.world_vel
                && vel.0 < -1.0
            {
                // Velocity reflected inwards!
                break;
            }
        }
        let disc = sim
            .player_disc
            .as_ref()
            .expect("disc should still be active");
        let vel = disc.world_vel.expect("world vel");
        assert!(vel.0 < 0.0, "disc must reflect inward off ring wall");
    }
}
