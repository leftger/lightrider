//! What a source-file mini-game tells the lightcycle about a step.
//!
//! Each game already owns its Bevy-free rules in its own `sim` module. This
//! trait lets it own the noise and the wording too: a game answers a step with
//! what it cleared, whether the run is over, what to call the crash, and which
//! sound cues to play. The lightcycle just plays them, so the per-game policy
//! lives next to the game rather than in one long match in the plugin.
//!
//! The types here stay free of Bevy so the sims remain unit-testable.

/// A gameplay sound a mini-game asks the run for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameSound {
    Turn,
    Beam,
    Portal,
    Victory,
    Crash,
    Zap,
}

/// One fixed step of a mini-game, folded into what the run has to do about it.
#[derive(Debug, Default)]
pub struct GameTick {
    /// The level was beaten this step.
    pub cleared: bool,
    /// The run ended this step. The run routes this through the shared crash
    /// path with [`GameTick::label`].
    pub lost: bool,
    /// What to call the crash, read only when `lost` is set.
    pub label: Option<String>,
    /// Sound cues for this step, in the order the game wants them played.
    pub sounds: Vec<GameSound>,
}

impl GameTick {
    pub fn sound(&mut self, sound: GameSound) {
        self.sounds.push(sound);
    }
}

/// The step a mini-game plays. Implemented beside each sim.
pub trait SourceGameSim {
    /// Advances the game one fixed step.
    fn tick(&mut self, dt: f32) -> GameTick;

    /// This game's segment of the lightcycle status line, wrapped around the
    /// `inner` text the run has built so far. `ring` is the file's name and
    /// `language` the compiler or language it was recognised as.
    fn status_line(&self, ring: &str, language: &str, inner: &str) -> String;
}
