//! The asteroid-field mini-game, with its Bevy wiring in `plugins::grid_rider`.
//!
//! [`sim`] is the Bevy-free state machine; the plugin owns the entities, the
//! camera and the sound.

pub mod sim;

pub(crate) mod plugin;
