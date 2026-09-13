//! Bevy wiring for the procedural music.
//!
//! This plugin owns the [`MusicState`] resource and the control-side systems.
//! Synthesis runs on the audio thread inside [`crate::music::engine`]; here we
//! only decide *what* it should play:
//!
//! - a directory change rebuilds the path-seeded theme and the entry list,
//! - the interaction mode selects the calm or action profile,
//! - every frame, the listener position (camera target or lightcycle cell) is
//!   turned into per-entry voice parameters by the proximity mixer.

use crate::config;
use crate::lightcycle::logic::{stable_path_seed, RunPhase};
use crate::lightcycle::{LightcycleState, RunEnvironment};
use crate::load::DirectoryLoaded;
use crate::music::arp::ArpState;
use crate::music::engine::AudioHandle;
use crate::music::proximity::VoiceMixer;
use crate::music::proximity::{Listener, NodePoint};
use crate::music::score::full_code;
use crate::music::score::{arp_message, base_filter_message, voice_message, wall_message};
use crate::music::sfx::MusicSfx;
use crate::music::theme::{ModeProfile, MusicTheme};
use crate::state::{FloodState, InteractionMode, OrbitCameraResource};
use bevy::prelude::*;
use std::f32::consts::TAU;
use std::fmt::Write as _;

pub struct MusicPlugin;

/// Music state shared by the control systems.
#[derive(Resource)]
pub struct MusicState {
    pub handle: AudioHandle,
    pub enabled: bool,
    pub volume: f32,
    pub theme: Option<MusicTheme>,
    pub profile: ModeProfile,
    pub mixer: VoiceMixer,
    /// The evolving melody layer.
    pub arp: ArpState,
    /// The current directory's entries, as musical points on the ground plane.
    pub nodes: Vec<NodePoint>,
    /// Reused buffer for the per-frame `send_msg` payload.
    params: String,
    /// Last payload sent, so an idle listener does not re-send every frame.
    last_params: String,
    /// Time accumulated toward the next parameter publish.
    param_clock: f32,
}

impl MusicState {
    /// Starts the audio thread with the requested initial settings.
    pub fn with_settings(enabled: bool, volume: f32) -> Self {
        let handle = AudioHandle::start();
        let volume = volume.clamp(
            config::music::MUSIC_MIN_VOLUME,
            config::music::MUSIC_MAX_VOLUME,
        );
        handle.set_volume(volume);
        handle.set_enabled(enabled);
        Self {
            handle,
            enabled,
            volume,
            theme: None,
            profile: ModeProfile::Calm,
            mixer: VoiceMixer::new(),
            arp: ArpState::new(),
            nodes: Vec::new(),
            params: String::new(),
            last_params: String::new(),
            param_clock: 0.0,
        }
    }
}

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        // `MusicState` is inserted by `main` so the CLI options can seed it.
        app.add_message::<MusicSfx>().add_systems(
            Update,
            (
                report_audio_status,
                play_sfx,
                sync_profile_with_mode,
                rebuild_for_directory,
                update_proximity,
                read_music_keys,
                apply_master_gain,
            )
                .chain(),
        );
    }
}

/// Plays the one-shot gameplay effects (crash, turn, beam, portal). The audio
/// thread owns each effect's envelope, so this only forwards the trigger.
fn play_sfx(mut effects: MessageReader<MusicSfx>, music: Res<MusicState>) {
    for sfx in effects.read() {
        music.handle.sfx(*sfx);
    }
}

/// Logs the audio thread's outcome once it settles, so a missing device is
/// explained instead of silently swallowing the music.
fn report_audio_status(mut reported: Local<bool>, music: Res<MusicState>) {
    if *reported {
        return;
    }
    let status = music.handle.status();
    if status.available {
        eprintln!(
            "raptor music: {} at {} Hz, {} channels",
            status.device.as_deref().unwrap_or("unknown device"),
            status.sample_rate,
            status.channels
        );
        *reported = true;
    } else if let Some(message) = &status.message
        && message != "starting"
    {
        eprintln!("raptor music disabled: {message}");
        *reported = true;
    }
}

/// Explorer is calm, Lightcycle is action. Recompiling the base graph is done
/// by the audio thread behind a short fade.
fn sync_profile_with_mode(mode: Res<InteractionMode>, mut music: ResMut<MusicState>) {
    let profile = ModeProfile::from_mode(*mode);
    if profile == music.profile {
        return;
    }
    music.profile = profile;
    music.mixer.clear();
    music.arp.reset();
    if let Some(theme) = &music.theme {
        music
            .handle
            .set_code(&full_code(theme, profile), theme.bpm(profile));
    }
}

/// A loaded directory becomes a theme plus an entry list. Because the theme is
/// seeded from the path, revisiting a folder restores the same piece.
fn rebuild_for_directory(
    mut loaded: MessageReader<DirectoryLoaded>,
    mode: Res<InteractionMode>,
    mut music: ResMut<MusicState>,
) {
    for event in loaded.read() {
        let theme = MusicTheme::from_path(&event.path);
        let profile = ModeProfile::from_mode(*mode);

        music.nodes = event
            .contents
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                let position = config::ground_position(node.grid_pos.0, node.grid_pos.1);
                NodePoint {
                    index,
                    x: position.x,
                    z: position.z,
                    node_seed: stable_path_seed(&node.path),
                    is_dir: node.is_dir,
                }
            })
            .collect();

        music.mixer.clear();
        music.arp.reset();
        music
            .handle
            .set_code(&full_code(&theme, profile), theme.bpm(profile));
        music.theme = Some(theme);
        music.profile = profile;
    }
}

/// The listener is the point the camera is looking at in Explorer, and the bike
/// in Lightcycle. Nearby entries swell their voices.
fn update_proximity(
    time: Res<Time>,
    mode: Res<InteractionMode>,
    orbit: Res<OrbitCameraResource>,
    lightcycle: Res<LightcycleState>,
    flood: Option<Res<FloodState>>,
    camera_query: Query<&Transform, With<Camera3d>>,
    mut music: ResMut<MusicState>,
) {
    let listener = match *mode {
        InteractionMode::Explorer => Listener {
            x: orbit.target.x,
            z: orbit.target.z,
        },
        InteractionMode::Lightcycle => {
            let Some(run) = &lightcycle.run else {
                return;
            };
            let position = config::ground_position(run.sim.cell.0, run.sim.cell.1);
            Listener {
                x: position.x,
                z: position.z,
            }
        }
    };

    let MusicState {
        handle,
        theme,
        profile,
        mixer,
        arp,
        nodes,
        params,
        last_params,
        param_clock,
        ..
    } = &mut *music;
    let Some(theme) = theme.as_ref() else {
        return;
    };

    let dt = time.delta_secs();
    let targets = mixer.update(listener, nodes, theme, *profile, dt);

    // While a disc-wars ring is open the folder theme stays the seed, but the
    // language tints the arpeggiator: Rust steps harder, Python pumps slower.
    let (arp_rate, arp_gain) = match lightcycle.run.as_ref().map(|run| &run.environment) {
        Some(RunEnvironment::Source { language, .. }) => {
            (language.arp_rate_scale(), language.arp_gain_scale())
        }
        _ => (1.0, 1.0),
    };

    // Approach warning growl for the red memory flood wall.
    // Attenuates with distance and maps to 3D space using the camera's orientation.
    let (wall_gain, wall_pan, wall_rate) = if *mode == InteractionMode::Lightcycle
        && let Some(run) = &lightcycle.run
        && let Some(flood) = flood.as_ref()
    {
        let (cam_pos, cam_right) = if let Some(cam) = camera_query.iter().next() {
            (cam.translation, cam.rotation * Vec3::X)
        } else {
            let pos = config::ground_position(run.sim.cell.0, run.sim.cell.1);
            (pos, Vec3::X)
        };
        compute_wall_audio_params(
            cam_pos,
            cam_right,
            flood.plane,
            flood.center_x,
            flood.width,
            flood.active,
            run.sim.phase == RunPhase::Running,
        )
    } else {
        (0.0, 0.0, 2.0)
    };

    // The evolving melody plus a slow filter sweep on the base voices. The
    // mixer and arp advance every frame, but the payload is only published at a
    // fixed rate so a fast frame loop cannot starve the audio thread.
    let arp_voice = arp.update(dt * arp_rate, theme, *profile);
    let sweep = 0.5 + 0.5 * (time.elapsed_secs() * profile.sweep_rate() * TAU).sin();

    let publish_interval = 1.0 / config::music::MUSIC_PARAMS_HZ.max(1.0);
    *param_clock += dt;
    if *param_clock < publish_interval {
        return;
    }
    *param_clock = (*param_clock % publish_interval).min(publish_interval);

    params.clear();
    for target in targets {
        let _ = write!(
            params,
            "{}",
            voice_message(
                target.slot,
                target.freq,
                target.cutoff,
                target.gain,
                target.pan
            )
        );
    }
    // A new note opens the filter briefly, so the attack is brighter than the
    // tail.
    let arp_cutoff = if arp_voice.triggered {
        (arp_voice.cutoff * 1.35).min(12_000.0)
    } else {
        arp_voice.cutoff
    };
    let _ = write!(
        params,
        "{}{}{}",
        arp_message(
            arp_voice.freq,
            arp_cutoff,
            arp_voice.gain * arp_gain,
            arp_voice.pan
        ),
        base_filter_message(theme, *profile, sweep),
        wall_message(wall_gain, wall_pan, wall_rate),
    );
    if params != last_params {
        handle.set_voice_params(params);
        last_params.clear();
        last_params.push_str(params);
    }
}

/// Toggle (`N`) and volume (`[` / `]`). Runs in both Explorer and Lightcycle so
/// the music can always be silenced.
fn read_music_keys(keys: Res<ButtonInput<KeyCode>>, mut music: ResMut<MusicState>) {
    if keys.just_pressed(KeyCode::KeyN) {
        music.enabled = !music.enabled;
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        music.volume = (music.volume - config::music::MUSIC_VOLUME_STEP).clamp(
            config::music::MUSIC_MIN_VOLUME,
            config::music::MUSIC_MAX_VOLUME,
        );
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        music.volume = (music.volume + config::music::MUSIC_VOLUME_STEP).clamp(
            config::music::MUSIC_MIN_VOLUME,
            config::music::MUSIC_MAX_VOLUME,
        );
    }
}

/// Pushes the current volume/enabled state to the audio thread. The thread
/// smooths the change, so this is safe to run every frame.
fn apply_master_gain(music: Res<MusicState>) {
    music.handle.set_volume(music.volume);
    music.handle.set_enabled(music.enabled);
}

/// Computes the (gain, pan, rate) for the approaching red wall warning growl.
///
/// Distance attenuation increases volume quadratically as the wall approaches,
/// and the pulse rate quickens from 1.8 Hz to 3.4 Hz.
/// Stereo pan maps the relative direction of the closest point on the wall
/// onto the camera's local right axis (`cam_right`).
pub fn compute_wall_audio_params(
    cam_pos: Vec3,
    cam_right: Vec3,
    flood_plane: f32,
    flood_center_x: f32,
    flood_width: f32,
    flood_active: bool,
    is_running: bool,
) -> (f32, f32, f32) {
    if !flood_active || !is_running {
        return (0.0, 0.0, 2.0);
    }

    let wall_z = flood_plane * config::GRID_SPACING;
    let half_width = (flood_width * 0.5).max(1.0);
    let min_x = flood_center_x - half_width;
    let max_x = flood_center_x + half_width;
    let closest_x = cam_pos.x.clamp(min_x, max_x);
    let closest_y = cam_pos.y.clamp(0.0, config::lightcycle::FLOOD_HEIGHT);
    let closest_wall = Vec3::new(closest_x, closest_y, wall_z);

    let to_wall = closest_wall - cam_pos;
    let dist = to_wall.length();

    const MAX_WALL_AUDIBLE_DIST: f32 = 48.0;
    if dist < MAX_WALL_AUDIBLE_DIST {
        let closeness = (1.0 - dist / MAX_WALL_AUDIBLE_DIST).clamp(0.0, 1.0);
        let gain = closeness * closeness * 0.32;
        let rate = 1.8 + closeness * 1.6;
        let dir = if dist > 0.001 {
            to_wall / dist
        } else {
            Vec3::ZERO
        };
        let pan = (dir.dot(cam_right) * 0.85).clamp(-0.85, 0.85);
        (gain, pan, rate)
    } else {
        (0.0, 0.0, 1.8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_audio_is_silent_when_inactive_or_not_running() {
        let (gain, _, _) = compute_wall_audio_params(
            Vec3::ZERO,
            Vec3::X,
            0.0,
            0.0,
            20.0,
            false, // inactive
            true,
        );
        assert_eq!(gain, 0.0);

        let (gain, _, _) = compute_wall_audio_params(
            Vec3::ZERO,
            Vec3::X,
            0.0,
            0.0,
            20.0,
            true,
            false, // crashed / not running
        );
        assert_eq!(gain, 0.0);
    }

    #[test]
    fn wall_audio_is_silent_when_far_away() {
        // Wall at Z = -100.0 (grid -50.0), camera at Z = 0.0
        let (gain, _, _) = compute_wall_audio_params(
            Vec3::ZERO,
            Vec3::X,
            -50.0,
            0.0,
            20.0,
            true,
            true,
        );
        assert_eq!(gain, 0.0);
    }

    #[test]
    fn wall_audio_gain_and_rate_increase_as_wall_approaches() {
        // Wall at 30 units away
        let (gain_far, _, rate_far) = compute_wall_audio_params(
            Vec3::new(0.0, 0.0, 30.0),
            Vec3::X,
            0.0, // wall at Z = 0
            0.0,
            20.0,
            true,
            true,
        );

        // Wall at 10 units away
        let (gain_close, _, rate_close) = compute_wall_audio_params(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::X,
            0.0, // wall at Z = 0
            0.0,
            20.0,
            true,
            true,
        );

        assert!(gain_far > 0.0, "far wall should be audible within 48 units");
        assert!(gain_close > gain_far, "closer wall must have higher gain");
        assert!(rate_close > rate_far, "closer wall must have faster pulse rate");
    }

    #[test]
    fn wall_audio_pans_with_camera_orientation() {
        // Wall is behind the origin at Z = -10.0 (grid plane = -5.0)
        let cam_pos = Vec3::ZERO;
        let flood_plane = -5.0; // wall_z = -10.0

        // 1. Camera facing forward (+Z): cam_right is +X. Wall is directly behind.
        let (_, pan_forward, _) = compute_wall_audio_params(
            cam_pos,
            Vec3::X,
            flood_plane,
            0.0,
            20.0,
            true,
            true,
        );
        assert!(pan_forward.abs() < 1e-4, "direct behind should be centered pan");

        // 2. Camera turned 90 deg right (facing +X): cam_right is +Z.
        // Wall is at -Z, so wall is to camera's left!
        let (_, pan_turn_right, _) = compute_wall_audio_params(
            cam_pos,
            Vec3::Z,
            flood_plane,
            0.0,
            20.0,
            true,
            true,
        );
        assert!(pan_turn_right < -0.5, "wall to camera's left must pan negative (left)");

        // 3. Camera turned 90 deg left (facing -X): cam_right is -Z.
        // Wall is at -Z, so wall is to camera's right!
        let (_, pan_turn_left, _) = compute_wall_audio_params(
            cam_pos,
            -Vec3::Z,
            flood_plane,
            0.0,
            20.0,
            true,
            true,
        );
        assert!(pan_turn_left > 0.5, "wall to camera's right must pan positive (right)");
    }
}

