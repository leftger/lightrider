//! Procedural-music constants: voices, mixing and effect timings.

/// Hard cap on simultaneously modulated per-entry voices. The voice bank is
/// always compiled at this width so switching profiles never rebuilds the graph.
pub const MUSIC_MAX_VOICES: usize = 8;

/// Master volume when music starts, before `[` / `]` adjustments.
pub const MUSIC_DEFAULT_VOLUME: f32 = 0.35;

pub const MUSIC_VOLUME_STEP: f32 = 0.1;

pub const MUSIC_MIN_VOLUME: f32 = 0.0;

pub const MUSIC_MAX_VOLUME: f32 = 1.0;

/// Calm (Explorer) base tempo range, in BPM.
pub const MUSIC_CALM_BPM_MIN: f32 = 72.0;

pub const MUSIC_CALM_BPM_MAX: f32 = 80.0;

/// Action (Lightcycle) tempo is the folder theme scaled by this factor.
/// Base 75-80 BPM * 1.6 gives 120-128 BPM (quintessential French/electro house).
pub const MUSIC_ACTION_TEMPO_MULTIPLIER: f32 = 1.6;

/// How many per-entry voices may sound at once per profile.
pub const MUSIC_CALM_VOICE_BUDGET: usize = 4;

pub const MUSIC_ACTION_VOICE_BUDGET: usize = MUSIC_MAX_VOICES;

/// Distance (world units; the grid spacing is [`GRID_SPACING`]) at which a
/// block's voice has fallen to half weight.
pub const MUSIC_CALM_PROXIMITY_RADIUS: f32 = 6.0;

pub const MUSIC_ACTION_PROXIMITY_RADIUS: f32 = 13.0;

/// Voices beyond `radius * this` are ignored entirely.
pub const MUSIC_PROXIMITY_CUTOFF_MULTIPLIER: f32 = 3.0;

/// Peak gain a single proximity voice may reach.
pub const MUSIC_CALM_GAIN_CEILING: f32 = 0.22;

pub const MUSIC_ACTION_GAIN_CEILING: f32 = 0.5;

/// One-pole time constant for voice gain / pan / filter smoothing, in seconds.
pub const MUSIC_CALM_SMOOTHING_TAU: f32 = 0.15;

pub const MUSIC_ACTION_SMOOTHING_TAU: f32 = 0.04;

/// A voice slot is released once its node's weight drops below this, and a new
/// node must exceed [`MUSIC_SLOT_ACQUIRE_THRESHOLD`] to take a free slot. The gap
/// between the two gives the assignment hysteresis, so voices do not thrash
/// between nearby blocks.
pub const MUSIC_SLOT_RELEASE_THRESHOLD: f32 = 0.12;

pub const MUSIC_SLOT_ACQUIRE_THRESHOLD: f32 = 0.25;

/// Stereo spread applied to a voice at the edge of the proximity radius.
pub const MUSIC_PAN_RANGE: f32 = 0.8;

/// Voice filter sweep: cutoff moves from the node's base cutoff up to
/// `base + weight * span` as the listener closes in.
pub const MUSIC_VOICE_CUTOFF_SPAN: f32 = 2400.0;

pub const MUSIC_VOICE_CUTOFF_MIN: f32 = 300.0;

pub const MUSIC_CALM_PAD_GAIN: f32 = 0.14;

/// Dune 2 desert brass war-horn drone gain in Calm mode (toned down to half power).
pub const MUSIC_CALM_DUNE_GAIN: f32 = 0.06;

/// Subtle ambient desert heartbeat pulse in Calm mode.
pub const MUSIC_CALM_PULSE_GAIN: f32 = 0.08;

/// Daft Punk 4-on-the-floor kick drum gain.
pub const MUSIC_ACTION_KICK_GAIN: f32 = 0.26;

/// Skrillex / electro backbeat snare gain.
pub const MUSIC_ACTION_SNARE_GAIN: f32 = 0.16;

/// deadmau5 driving eighth-note offbeat hi-hat gain.
pub const MUSIC_ACTION_HAT_GAIN: f32 = 0.12;

/// deadmau5 clean sub-bass gain.
pub const MUSIC_ACTION_SUB_GAIN: f32 = 0.18;

/// Daft Punk pumping electro sawtooth bass gain.
pub const MUSIC_ACTION_BASS_GAIN: f32 = 0.16;

/// Skrillex modulated wobble growl bass gain.
pub const MUSIC_ACTION_GROWL_GAIN: f32 = 0.14;

/// Daft Punk / TRON: Legacy synth lead gain.
pub const MUSIC_ACTION_LEAD_GAIN: f32 = 0.12;

/// Action French Touch sidechain pumping depth on synths (fraction of full gain).
pub const MUSIC_ACTION_PUMP_DEPTH: f32 = 0.65;

/// Sidechain pump rate in cycles per beat. 1.0 = one pump per beat (quarter-note ducking).
pub const MUSIC_ACTION_PUMP_RATE: f32 = 1.0;

/// Reverb plate mix level applied to output.
pub const MUSIC_REVERB_PLATE_MIX: f32 = 0.08;

/// Seconds the graph crossfade takes when a room or profile changes. Long
/// enough to read as a blend, short enough to feel responsive while browsing.
pub const MUSIC_CROSSFADE_SECONDS: f32 = 0.7;

/// How often voice parameters are published to the audio thread, in Hz. The
/// payload is coalesced, so a faster frame loop cannot backlog the audio thread.
pub const MUSIC_PARAMS_HZ: f32 = 60.0;

/// Arpeggiator. `BEATS` is step length in beats, so tempo drives the rate:
/// Explorer gets one soft note per beat, Lightcycle gets rapid sixteenth notes
/// (4 per beat) for driving progressive electro (deadmau5 / Daft Punk "Derezzed").
pub const MUSIC_CALM_ARP_BEATS: f32 = 1.0;

pub const MUSIC_ACTION_ARP_BEATS: f32 = 4.0;

pub const MUSIC_CALM_ARP_TAU: f32 = 0.28;

pub const MUSIC_ACTION_ARP_TAU: f32 = 0.09;

pub const MUSIC_CALM_ARP_GAIN: f32 = 0.10;

pub const MUSIC_ACTION_ARP_GAIN: f32 = 0.14;

/// Slow filter sweep applied to the base voices, in cycles per second.
pub const MUSIC_CALM_SWEEP_RATE: f32 = 0.05;

pub const MUSIC_ACTION_SWEEP_RATE: f32 = 0.12;

/// Per-language arpeggiator tint while a ring is open: Rust arpeggiates harder,
/// Python pumps slower. Multipliers on the folder theme, which stays the seed.
pub const MUSIC_DISC_RUST_ARP_RATE: f32 = 1.25;

pub const MUSIC_DISC_C_ARP_RATE: f32 = 1.0;

pub const MUSIC_DISC_CPP_ARP_RATE: f32 = 1.1;

pub const MUSIC_DISC_PYTHON_ARP_RATE: f32 = 0.75;

pub const MUSIC_DISC_SLINT_ARP_RATE: f32 = 0.9;

pub const MUSIC_DISC_LUA_ARP_RATE: f32 = 1.15;

pub const MUSIC_DISC_SHELL_ARP_RATE: f32 = 1.35;

pub const MUSIC_DISC_TOML_ARP_RATE: f32 = 1.05;

pub const MUSIC_DISC_JSON_ARP_RATE: f32 = 1.3;

pub const MUSIC_DISC_GO_ARP_RATE: f32 = 1.1;

pub const MUSIC_DISC_RUBY_ARP_RATE: f32 = 0.9;

pub const MUSIC_DISC_YAML_ARP_RATE: f32 = 0.8;

pub const MUSIC_DISC_JS_ARP_RATE: f32 = 1.4;

pub const MUSIC_DISC_ZIG_ARP_RATE: f32 = 1.25;

pub const MUSIC_DISC_PHP_ARP_RATE: f32 = 1.2;

pub const MUSIC_DISC_R_ARP_RATE: f32 = 0.7;

pub const MUSIC_DISC_RUST_ARP_GAIN: f32 = 1.2;

pub const MUSIC_DISC_C_ARP_GAIN: f32 = 1.0;

pub const MUSIC_DISC_CPP_ARP_GAIN: f32 = 1.1;

pub const MUSIC_DISC_PYTHON_ARP_GAIN: f32 = 0.85;

pub const MUSIC_DISC_SLINT_ARP_GAIN: f32 = 0.95;

pub const MUSIC_DISC_LUA_ARP_GAIN: f32 = 1.05;

pub const MUSIC_DISC_SHELL_ARP_GAIN: f32 = 0.9;

pub const MUSIC_DISC_TOML_ARP_GAIN: f32 = 1.0;

pub const MUSIC_DISC_JSON_ARP_GAIN: f32 = 1.05;

pub const MUSIC_DISC_GO_ARP_GAIN: f32 = 1.0;

pub const MUSIC_DISC_RUBY_ARP_GAIN: f32 = 0.95;

pub const MUSIC_DISC_YAML_ARP_GAIN: f32 = 0.85;

pub const MUSIC_DISC_JS_ARP_GAIN: f32 = 1.1;

pub const MUSIC_DISC_ZIG_ARP_GAIN: f32 = 1.15;

pub const MUSIC_DISC_PHP_ARP_GAIN: f32 = 1.0;

pub const MUSIC_DISC_R_ARP_GAIN: f32 = 0.8;
