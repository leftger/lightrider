pub mod logic;
pub(crate) mod scene;

use crate::asteroids::sim::AsteroidsSim;
use crate::bomberman::sim::BomberSim;
use crate::breaker::sim::BreakerSim;
use crate::columns::sim::ColumnsSim;
use crate::disc::combat::DiscSim;
use crate::disc::language::SourceGame;
use crate::disc::layout::DiscLayout;
use crate::document::layout::DocumentLayout;
use crate::filesystem::language::SourceLanguage;
use crate::filesystem::node::FileNode;
use crate::frogger::sim::FroggerSim;
use crate::galaga::sim::GalagaSim;
use crate::pacman::sim::PacSim;
use crate::platformer::sim::PlatformerSim;
use crate::plinko::sim::PlinkoSim;
use crate::qbert::sim::QbertSim;
use crate::snake::sim::SnakeSim;
use crate::stealth::sim::StealthSim;
use crate::surfer::sim::SurferSim;
use crate::tetris::sim::TetrisSim;
use bevy::prelude::Resource;
use std::collections::HashMap;
use std::path::PathBuf;

/// Snapshot of one active lightcycle run over a loaded directory, document, or
/// disc-wars ring.
#[derive(Debug, Clone)]
pub struct ActiveRun {
    pub sim: logic::LightcycleSim,
    pub arena: logic::Arena,
    pub environment: RunEnvironment,
    /// Human-readable label shown after a crash.
    pub crash_label: Option<String>,
    /// Human-readable label shown while waiting for a folder/parent/document load.
    pub entering_label: Option<String>,
}

/// The sim a source ring is actually playing. Carrying exactly one keeps the
/// game kind and its state from drifting apart: a disc-wars ring has no
/// asteroid field to accidentally read (and vice versa).
#[derive(Debug, Clone)]
pub enum SourceSim {
    DiscWars(DiscSim),
    /// Boxed: the field holds two entity vectors.
    Asteroids(Box<AsteroidsSim>),
    Snake(SnakeSim),
    /// Boxed: the level holds every platform in the run.
    Platformer(Box<PlatformerSim>),
    Breaker(Box<BreakerSim>),
    Stealth(Box<StealthSim>),
    Surfer(Box<SurferSim>),
    Galaga(Box<GalagaSim>),
    PacMan(Box<PacSim>),
    Columns(Box<ColumnsSim>),
    Tetris(Box<TetrisSim>),
    Frogger(Box<FroggerSim>),
    Qbert(Box<QbertSim>),
    Bomberman(Box<BomberSim>),
    Plinko(Box<PlinkoSim>),
}

/// The mini-games that step from `dt` alone, and the ring game each one is.
///
/// One line per game: the variant's identity, its typed accessors and its
/// [`SourceGame`] all come from the table below, so adding a game means adding
/// a line rather than editing three parallel matches.
macro_rules! source_sims {
    ($($variant:ident => $game:ident),+ $(,)?) => {
        impl SourceSim {
            /// Which game this state belongs to.
            pub fn game(&self) -> SourceGame {
                match self {
                    Self::DiscWars(_) => SourceGame::DiscWars,
                    Self::Asteroids(_) => SourceGame::Asteroids,
                    Self::Snake(_) => SourceGame::Snake,
                    $(Self::$variant(_) => SourceGame::$game,)+
                }
            }

            /// The uniform mini-games — the ones that step themselves — behind
            /// one trait object. Disc wars, the asteroid field and snake drive
            /// the cycle themselves, so they stay concrete.
            pub fn as_game(&self) -> Option<&dyn crate::minigame::SourceGameSim> {
                match self {
                    $(Self::$variant(sim) => Some(&**sim),)+
                    _ => None,
                }
            }

            /// Mutable [`Self::as_game`].
            pub fn as_game_mut(&mut self) -> Option<&mut dyn crate::minigame::SourceGameSim> {
                match self {
                    $(Self::$variant(sim) => Some(&mut **sim),)+
                    _ => None,
                }
            }
        }
    };
}

source_sims! {
    Platformer => Platformer,
    Breaker => Breaker,
    Stealth => Stealth,
    Surfer => RiverSurfer,
    Galaga => Galaga,
    PacMan => PacMan,
    Columns => Columns,
    Tetris => Tetris,
    Frogger => Frogger,
    Qbert => Qbert,
    Bomberman => Bomberman,
    Plinko => Plinko,
}

impl SourceSim {
    /// The uniform game behind this sim, recovered as its concrete type.
    ///
    /// The game's own wiring is the only caller, so it always names its own
    /// type; a game that is not `T` simply yields `None`.
    pub fn sim<T: crate::minigame::SourceGameSim + 'static>(&self) -> Option<&T> {
        let game = self.as_game()?;
        crate::minigame::AsAny::as_any(game).downcast_ref::<T>()
    }

    pub fn as_disc(&self) -> Option<&DiscSim> {
        match self {
            Self::DiscWars(disc) => Some(disc),
            _ => None,
        }
    }

    pub fn as_disc_mut(&mut self) -> Option<&mut DiscSim> {
        match self {
            Self::DiscWars(disc) => Some(disc),
            _ => None,
        }
    }

    pub fn as_asteroids(&self) -> Option<&AsteroidsSim> {
        match self {
            Self::Asteroids(field) => Some(field),
            _ => None,
        }
    }

    pub fn as_asteroids_mut(&mut self) -> Option<&mut AsteroidsSim> {
        match self {
            Self::Asteroids(field) => Some(field),
            _ => None,
        }
    }

    pub fn as_snake(&self) -> Option<&SnakeSim> {
        match self {
            Self::Snake(snake) => Some(snake),
            _ => None,
        }
    }

    pub fn as_snake_mut(&mut self) -> Option<&mut SnakeSim> {
        match self {
            Self::Snake(snake) => Some(snake),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RunEnvironment {
    Directory {
        nodes: Vec<FileNode>,
        cells: HashMap<(i32, i32), usize>,
    },
    Document {
        path: PathBuf,
        name: String,
        layout: DocumentLayout,
        focused_block: Option<usize>,
    },
    /// A ring generated from a source file. Which game is played inside it is
    /// chosen by the language; the ring geometry, close gate and restore path
    /// are shared by all of them.
    ///
    /// The player's own movement always stays in [`ActiveRun::sim`]. Disc wars
    /// and snake drive it normally; the asteroid field parks it and only pivots,
    /// stepping [`SourceSim::Asteroids`] instead.
    Source {
        path: PathBuf,
        name: String,
        language: SourceLanguage,
        /// Boxed: a ring's layout is much larger than the other variants.
        layout: Box<DiscLayout>,
        sim: SourceSim,
        focused_block: Option<usize>,
    },
}

impl ActiveRun {
    pub fn directory_cells(&self) -> Option<&HashMap<(i32, i32), usize>> {
        match &self.environment {
            RunEnvironment::Directory { cells, .. } => Some(cells),
            _ => None,
        }
    }

    pub fn directory_nodes(&self) -> Option<&[FileNode]> {
        match &self.environment {
            RunEnvironment::Directory { nodes, .. } => Some(nodes),
            _ => None,
        }
    }

    pub fn is_document(&self) -> bool {
        matches!(self.environment, RunEnvironment::Document { .. })
    }

    pub fn is_source(&self) -> bool {
        matches!(self.environment, RunEnvironment::Source { .. })
    }

    /// Which mini-game this run is, for source runs only.
    pub fn source_game(&self) -> Option<SourceGame> {
        match &self.environment {
            RunEnvironment::Source { sim, .. } => Some(sim.game()),
            _ => None,
        }
    }

    /// The uniform mini-game behind this source run, for stepping and input.
    pub fn source_game_mut(&mut self) -> Option<&mut dyn crate::minigame::SourceGameSim> {
        match &mut self.environment {
            RunEnvironment::Source { sim, .. } => sim.as_game_mut(),
            _ => None,
        }
    }

    /// The uniform game behind this run, recovered as its concrete type.
    pub fn source_sim<T: crate::minigame::SourceGameSim + 'static>(&self) -> Option<&T> {
        match &self.environment {
            RunEnvironment::Source { sim, .. } => sim.sim::<T>(),
            _ => None,
        }
    }

    pub fn source_disc_mut(&mut self) -> Option<&mut DiscSim> {
        match &mut self.environment {
            RunEnvironment::Source { sim, .. } => sim.as_disc_mut(),
            _ => None,
        }
    }

    pub fn source_disc(&self) -> Option<&DiscSim> {
        match &self.environment {
            RunEnvironment::Source { sim, .. } => sim.as_disc(),
            _ => None,
        }
    }

    pub fn source_asteroids(&self) -> Option<&AsteroidsSim> {
        match &self.environment {
            RunEnvironment::Source { sim, .. } => sim.as_asteroids(),
            _ => None,
        }
    }

    pub fn source_asteroids_mut(&mut self) -> Option<&mut AsteroidsSim> {
        match &mut self.environment {
            RunEnvironment::Source { sim, .. } => sim.as_asteroids_mut(),
            _ => None,
        }
    }

    pub fn source_snake(&self) -> Option<&SnakeSim> {
        match &self.environment {
            RunEnvironment::Source { sim, .. } => sim.as_snake(),
            _ => None,
        }
    }

    pub fn source_snake_mut(&mut self) -> Option<&mut SnakeSim> {
        match &mut self.environment {
            RunEnvironment::Source { sim, .. } => sim.as_snake_mut(),
            _ => None,
        }
    }

    /// True only while an asteroid field is actually being played.
    pub fn asteroid_field_active(&self) -> bool {
        self.source_asteroids()
            .is_some_and(|field| field.is_active())
    }
}

/// Bevy resource for lightcycle-only state.
#[derive(Resource, Default)]
pub struct LightcycleState {
    pub run: Option<ActiveRun>,
    /// Run built when a mode transition started, held back until the transition
    /// reaches the point where the old world is swapped out for it.
    pub pending_run: Option<ActiveRun>,
    /// Accumulated time used for fixed-step simulation.
    pub clock: f32,
    /// Active crash animation state (debris burst + camera shake).
    pub crash_fx: Option<CrashFx>,
    /// Active directory-tower transport animation.
    pub entry_fx: Option<EntryFx>,
    /// Rebuild the containing directory arena after leaving a document.
    pub restore_directory: bool,
    /// Bullet time is held down while riding a disc-wars ring. Set from input
    /// each frame; the ring's fixed step is scaled while it is true.
    pub slow_motion: bool,
    /// Seconds of cache-hit ground-speed surge remaining.
    pub cache_boost: f32,
    /// Countdown to the next garbage-collector sweep of a directory arena.
    pub gc_timer: f32,
    /// Seconds left of the collector's world stall.
    pub gc_pause: f32,
    /// Seconds left of the collector's visible sweep across the arena.
    pub gc_sweep: f32,
    /// True while the current directory is a quarantined vault.
    pub quarantined: bool,
    /// Set once the player has ridden at all. The opening room is a grace
    /// period: nothing is hunting them while they learn the controls.
    pub rides_started: bool,
    /// True while the current room is that grace period, which the HUD says out
    /// loud so the missing hazards do not read as a bug.
    pub grace_room: bool,
}

/// Timeline for the crash animation.
#[derive(Debug, Clone, Copy)]
pub struct CrashFx {
    pub timer: f32,
    pub duration: f32,
    pub spawned: bool,
}

impl CrashFx {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: duration,
            duration,
            spawned: false,
        }
    }
}

/// Timeline and deferred navigation target for a directory-tower transport.
///
/// The directory request waits until the visual reaches its bright apex so a
/// fast filesystem scan cannot replace the old arena before the effect appears.
#[derive(Debug, Clone)]
pub struct EntryFx {
    pub elapsed: f32,
    pub duration: f32,
    pub target: PathBuf,
    pub spawned: bool,
    pub requested: bool,
}

impl EntryFx {
    pub fn new(target: PathBuf, duration: f32) -> Self {
        Self {
            elapsed: 0.0,
            duration,
            target,
            spawned: false,
            requested: false,
        }
    }

    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }
}
