//! Seeded arpeggiator: the evolving melody layer.
//!
//! Rather than holding a static drone, the base track walks a note pattern in
//! time. The pattern is derived from the directory theme's seed, so every folder
//! has its own tune, and it is retriggered with a per-note envelope so the
//! arpeggio reads as plucks instead of a continuous tone. Explorer steps once
//! per beat and softly; Lightcycle steps twice per beat with more gain.

use super::theme::{ModeProfile, MusicTheme};
use crate::config;

/// Iconic melodic motifs selected by the path seed:
/// 0: deadmau5 progressive rolling arpeggio (16-step hypnotic rise and fall)
/// 1: Daft Punk French Touch octave funk (octave bounce & 7ths, TRON / Derezzed style)
/// 2: Skrillex syncopated electro hook (blues / minor pentatonic bite)
/// 3: Dune 2 Desert Sands / Phrygian mystery (exotic intervals & scalar climbs)
const PATTERNS: [&[u32]; 4] = [
    // deadmau5 progressive rolling arp (16 steps)
    &[0, 2, 4, 7, 9, 7, 4, 2, 0, 4, 7, 11, 9, 7, 4, 2],
    // Daft Punk French Touch octave funk (16 steps)
    &[0, 7, 12, 7, 0, 10, 12, 10, 0, 7, 12, 7, 2, 7, 10, 7],
    // Skrillex electro-bass hook (16 steps)
    &[0, 0, 3, 0, 7, 0, 5, 3, 0, 0, 3, 6, 7, 10, 7, 5],
    // Dune 2 Desert Sands / Phrygian mystery (16 steps)
    &[0, 1, 4, 5, 7, 8, 7, 5, 4, 1, 0, 1, 4, 7, 8, 11],
];

/// One frame of arpeggiator output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArpVoice {
    pub freq: f32,
    pub cutoff: f32,
    pub gain: f32,
    pub pan: f32,
    /// True on the frame a new note starts.
    pub triggered: bool,
}

/// Steps a note pattern in time and retriggers a per-note envelope.
#[derive(Debug, Default)]
pub struct ArpState {
    clock: f32,
    level: f32,
    step: u64,
}

impl ArpState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Restarts the pattern, e.g. after a room change.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Number of notes played since the last reset. Test-only.
    #[cfg(test)]
    pub fn step(&self) -> u64 {
        self.step
    }

    /// Advances by `dt` and returns the current voice.
    pub fn update(&mut self, dt: f32, theme: &MusicTheme, profile: ModeProfile) -> ArpVoice {
        let dt = dt.clamp(0.0, 0.25);
        let interval = profile.arp_interval(theme.bpm(profile)).max(0.01);

        self.clock += dt;
        let mut triggered = false;
        // Catch up at most a couple of steps, so a long frame cannot machine-gun.
        let mut caught_up = 0;
        while self.clock >= interval && caught_up < 2 {
            self.clock -= interval;
            self.step += 1;
            self.level = velocity(theme.seed, self.step - 1);
            triggered = true;
            caught_up += 1;
        }

        // Exponential decay of the current note.
        let decay = (-dt / profile.arp_decay_tau().max(0.01)).exp();
        self.level *= decay;

        let note = self.step.saturating_sub(1);
        let (degree, octave) = degree(theme.seed, note);
        let freq = theme.degree_hz(degree, octave);
        // Progressive deadmau5 filter sweep: breathes over a cyclical wave
        let sweep = (self.step as f32 * 0.04).sin() * 0.5 + 0.5;
        let cutoff = (theme.node_cutoff(theme.seed ^ note) + sweep * 3200.0 + degree as f32 * 110.0)
            .clamp(config::music::MUSIC_VOICE_CUTOFF_MIN, 12_000.0);
        let pan = if note % 4 == 0 {
            -0.28
        } else if note % 4 == 2 {
            0.28
        } else {
            0.0
        };
        let gain = self.level * profile.arp_gain();

        ArpVoice {
            freq,
            cutoff,
            gain,
            pan,
            triggered,
        }
    }
}

/// Next (scale degree, octave offset) for a step.
/// Generates progressive phrase development over bars while strictly adhering
/// to the path seed and the directory's scale.
fn degree(seed: u64, note: u64) -> (u32, i32) {
    let pattern_idx = ((seed >> 8) % PATTERNS.len() as u64) as usize;
    let pattern = PATTERNS[pattern_idx];
    let rotation = (seed % pattern.len() as u64) as usize;
    let index = (note as usize + rotation) % pattern.len();
    let bar = (note / pattern.len() as u64) % 4;

    // Procedural progression over a 4-bar cycle:
    // Bar 0: Canonical motif
    // Bar 1: Octave bounce or passing tone
    // Bar 2: Melodic shift within scale
    // Bar 3: Climactic octave lift
    let base_degree = pattern[index];
    let (deg, oct) = match bar {
        1 => {
            let mod_deg = if (seed >> 14).is_multiple_of(2) {
                base_degree
            } else {
                pattern[(index + 1) % pattern.len()]
            };
            (mod_deg, 0)
        }
        2 => {
            let shift = if (seed >> 16).is_multiple_of(2) { 2 } else { 4 };
            (base_degree + shift, 0)
        }
        3 => (base_degree, 1),
        _ => (base_degree, 0),
    };
    (deg, oct)
}

/// Downbeat-accented velocity with subtle deterministic seed micro-groove.
fn velocity(seed: u64, note: u64) -> f32 {
    let base = if note.is_multiple_of(4) {
        1.0
    } else if note.is_multiple_of(2) {
        0.78
    } else {
        0.62
    };
    let micro = (((seed.wrapping_add(note) >> 3) % 5) as f32 - 2.0) * 0.015;
    (base + micro).clamp(0.4, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme() -> MusicTheme {
        MusicTheme::from_seed(0x51ee_d0d0)
    }

    fn run(state: &mut ArpState, profile: ModeProfile, seconds: f32) -> Vec<ArpVoice> {
        let dt = 1.0 / 120.0;
        let steps = (seconds / dt) as usize;
        let mut voices = Vec::with_capacity(steps);
        for _ in 0..steps {
            voices.push(state.update(dt, &theme(), profile));
        }
        voices
    }

    #[test]
    fn action_steps_much_faster_than_calm() {
        let mut calm = ArpState::new();
        let mut action = ArpState::new();
        let calm_notes = run(&mut calm, ModeProfile::Calm, 8.0)
            .iter()
            .filter(|voice| voice.triggered)
            .count();
        let action_notes = run(&mut action, ModeProfile::Action, 8.0)
            .iter()
            .filter(|voice| voice.triggered)
            .count();
        assert!(
            action_notes > calm_notes * 2,
            "action {action_notes} vs calm {calm_notes}"
        );
        assert!(calm_notes >= 6, "calm should still play: {calm_notes}");
    }

    #[test]
    fn the_same_seed_plays_the_same_tune() {
        let mut a = ArpState::new();
        let mut b = ArpState::new();
        let first: Vec<f32> = run(&mut a, ModeProfile::Calm, 12.0)
            .iter()
            .filter(|voice| voice.triggered)
            .map(|voice| voice.freq)
            .collect();
        let second: Vec<f32> = run(&mut b, ModeProfile::Calm, 12.0)
            .iter()
            .filter(|voice| voice.triggered)
            .map(|voice| voice.freq)
            .collect();
        assert_eq!(first, second);
        assert!(first.len() > 4);
    }

    #[test]
    fn every_note_stays_in_the_theme_scale() {
        let theme = theme();
        let intervals = theme.scale.intervals();
        let mut state = ArpState::new();
        for voice in run(&mut state, ModeProfile::Action, 20.0) {
            if !voice.triggered {
                continue;
            }
            let midi = 69.0 + 12.0 * (voice.freq / 440.0).log2();
            let offset = ((midi.round() as i32 - theme.root_midi as i32).rem_euclid(12)) as u8;
            assert!(
                intervals.contains(&offset),
                "freq {} gave offset {offset}",
                voice.freq
            );
        }
    }

    #[test]
    fn the_envelope_decays_between_notes() {
        let mut state = ArpState::new();
        let voices = run(&mut state, ModeProfile::Calm, 4.0);
        let peak = voices
            .iter()
            .position(|voice| voice.triggered)
            .expect("a note");
        let after = voices[peak + 3].gain;
        assert!(after < voices[peak].gain, "envelope did not decay");
    }

    #[test]
    fn reset_starts_the_pattern_over() {
        let mut state = ArpState::new();
        run(&mut state, ModeProfile::Calm, 3.0);
        assert!(state.step() > 0);
        state.reset();
        assert_eq!(state.step(), 0);
    }
}
