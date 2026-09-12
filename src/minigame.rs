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

/// One frame of player input, in the shared lightcycle vocabulary.
///
/// The plugin reads the keyboard and mouse once and hands every game the same
/// frame; a game maps the parts it cares about onto its own controls.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GameInput {
    /// Held left/right, for games that slide continuously.
    pub left: bool,
    pub right: bool,
    /// Held east/west axis: `-1`, `0` or `1`.
    pub steer: i32,
    /// Held north/south axis: `-1`, `0` or `1`.
    pub move_z: i32,
    /// Tapped north/south axis: `-1`, `0` or `1`.
    pub hop_z: i32,
    /// Held boost / throttle.
    pub boost: bool,
    /// Edge-triggered action: Space or left click.
    pub action: bool,
}

impl GameInput {
    /// `1` right, `-1` left, for games that step a whole cell per tap.
    pub fn right_x(&self) -> i32 {
        i32::from(self.right) - i32::from(self.left)
    }
}

/// The step a mini-game plays. Implemented beside each sim.
pub trait SourceGameSim {
    /// Advances the game one fixed step.
    fn tick(&mut self, dt: f32) -> GameTick;

    /// Feeds one input frame to the game.
    fn input(&mut self, input: &GameInput);

    /// This game's segment of the lightcycle status line, wrapped around the
    /// `inner` text the run has built so far. `ring` is the file's name and
    /// `language` the compiler or language it was recognised as.
    fn status_line(&self, ring: &str, language: &str, inner: &str) -> String;
}
