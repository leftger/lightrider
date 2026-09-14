//! Chase-camera rigs, cycle poses, and per-game focus helpers.

use crate::asteroids::plugin::field_camera_focus;
use crate::breaker::sim::BreakerSim;
use crate::config;
use crate::disc::language::SourceGame;
use crate::galaga::sim::GalagaSim;
use crate::lightcycle::scene::CharacterEntity;
use crate::lightcycle::scene::ChaseCamera;
use crate::lightcycle::scene::CycleEntity;
use crate::lightcycle::scene::Only;
use crate::lightcycle::scene::pose::CharacterPose;
use crate::lightcycle::scene::pose::heading_angle;
use crate::lightcycle::scene::pose::{advance_chase_forward, chase_camera_rig};
use crate::lightcycle::{ActiveRun, LightcycleState};
use crate::platformer::sim::PlatformerSim;
use crate::plugins::transition::ModeTransition;
use crate::stealth::plugin::hug_camera_shot;
use crate::stealth::sim::StealthSim;
use crate::surfer::plugin::surfer_camera_rig;
use crate::surfer::sim::SurferSim;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

pub(crate) fn character_pose(run: &ActiveRun) -> Option<CharacterPose> {
    if let Some(level) = run.source_sim::<PlatformerSim>() {
        return Some(CharacterPose {
            target: Vec3::new(level.runner.x, level.runner.y, 0.0),
            // A quarter turn each way, not a half: the model's forward is
            // `+Z`, so facing along the level's `X` axis means pointing it at
            // `+X` or `-X`.
            yaw: if level.runner.facing >= 0.0 {
                std::f32::consts::FRAC_PI_2
            } else {
                -std::f32::consts::FRAC_PI_2
            },
            smooth: false,
        });
    }
    run.source_sim::<StealthSim>().map(|room| {
        // Backed against a wall, the figure is leaned into it. Standing a whole
        // cell short reads as not quite touching, which loses the pose entirely;
        // `hug` is the wall's direction, so the lean is toward it. It eases in and
        // out through the same smoothing as the walking, so nothing snaps.
        let stand = config::ground_position(room.character.0, room.character.1);
        let target = match room.hug {
            Some(wall) => {
                let angle = heading_angle(wall);
                stand + Vec3::new(angle.cos(), 0.0, angle.sin()) * config::stealth::STEALTH_HUG_LEAN
            }
            None => stand,
        };
        CharacterPose {
            target,
            // The sim already faces the figure away from a wall it is hugging, so
            // this is the walking facing in every case.
            yaw: std::f32::consts::FRAC_PI_2 - heading_angle(room.heading),
            smooth: true,
        }
    })
}

// A Bevy system: the queries are the reason for both of these, and folding them
// into a SystemParam struct would only move the noise.
#[allow(clippy::too_many_arguments)]
pub(crate) fn update_chase_camera(
    state: Res<LightcycleState>,
    transition: Res<ModeTransition>,
    time: Res<Time>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut camera: Single<&mut Transform, (With<Camera3d>, Without<CycleEntity>)>,
    character: Query<&Transform, Only<CharacterEntity, Camera3d, ChaseCamera>>,
    mut cycle: Query<
        (
            &Transform,
            &mut ChaseCamera,
            Option<&crate::plugins::lightcycle::physics::LightcyclePhysics>,
        ),
        Without<Camera3d>,
    >,
) {
    let Ok((cycle, mut chase, physics)) = cycle.single_mut() else {
        return;
    };
    // The flight owns the camera until it lands on this rig; easing the follow
    // direction or taking a look drag now would move the pose it is aiming for.
    if state.run.is_none() || transition.is_active() {
        return;
    }

    // The platformer is played from the side, riding along with the runner.
    if let Some(level) = state
        .run
        .as_ref()
        .and_then(|run| run.source_sim::<PlatformerSim>())
    {
        let focus = Vec3::new(
            level.runner.x + config::platformer::PLATFORMER_CAMERA_AHEAD,
            (level.runner.y + config::platformer::PLATFORMER_CAMERA_HEIGHT).max(2.0),
            0.0,
        );
        let target = Vec3::new(focus.x, focus.y, config::platformer::PLATFORMER_CAMERA_BACK);
        let blend = 1.0 - (-config::platformer::PLATFORMER_CAMERA_LERP * time.delta_secs()).exp();
        camera.translation = camera.translation.lerp(target, blend);
        camera.look_at(focus, Vec3::Y);
        return;
    }

    // The breaker is played head-on: the whole court stays in frame while the
    // bike slides along the bottom.
    if let Some(level) = state
        .run
        .as_ref()
        .and_then(|run| run.source_sim::<BreakerSim>())
    {
        let centre = Vec3::new(0.0, level.court.1 * 0.5, 0.0);
        camera.translation = Vec3::new(0.0, centre.y, config::breaker::BREAKER_CAMERA_BACK);
        camera.look_at(centre, Vec3::Y);
        return;
    }

    // The stealth run is played from above, like a stakeout.
    if let Some(room) = state
        .run
        .as_ref()
        .and_then(|run| run.source_sim::<StealthSim>())
    {
        // Follow where the figure is actually drawn, not the cell it is walking
        // toward: the sim moves in whole cells, so tracking the cell would lurch
        // the whole view once per step.
        let focus = character
            .single()
            .map(|transform| transform.translation)
            .unwrap_or_else(|_| config::ground_position(room.character.0, room.character.1));
        // The camera holds a bearing round the figure and turns steadily toward
        // whatever the view should be aimed along: round the far side of the peek
        // direction when the player is backed against a wall, and plain +Z
        // otherwise.
        //
        // It turns at a fixed rate rather than easing, because that is what makes
        // the swing watchable: an ease puts nearly all the movement in the first
        // few frames, which is why the perspective read as changing instantly.
        // The radius and height still ease, so entering a run flies in as before.
        // Where the view should sit, and what it should look at, both as offsets
        // from the figure.
        //
        // Backed against a wall, the pose comes from [`hug_camera_shot`]: the
        // camera acts like an imaginary second figure standing off the wall and
        // looking back at the real one, so the figure, the wall he is hugging,
        // the corner and the corridor around it all share the frame.
        let (want_x, want_z, want_height, look) = match hug_camera_shot(room) {
            Some(shot) => (shot.offset.x, shot.offset.z, shot.height, shot.look),
            None => (
                0.0,
                config::stealth::STEALTH_CAMERA_DISTANCE,
                config::stealth::STEALTH_CAMERA_HEIGHT,
                Vec3::Y * config::stealth::STEALTH_CAMERA_LOOK,
            ),
        };
        let want_radius = (want_x * want_x + want_z * want_z).sqrt();
        let aim = want_z.atan2(want_x);
        let offset = camera.translation - focus;
        let bearing = offset.z.atan2(offset.x);
        let radius = (offset.x * offset.x + offset.z * offset.z).sqrt();
        let turn = config::stealth::STEALTH_SWING_RATE * time.delta_secs();
        let to_aim = (aim - bearing + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        let bearing = bearing + to_aim.clamp(-turn, turn);
        let blend = 1.0 - (-config::stealth::STEALTH_CAMERA_LERP * time.delta_secs()).exp();
        let radius = radius + (want_radius - radius) * blend;
        let height = offset.y + (want_height - offset.y) * blend;
        camera.translation =
            focus + Vec3::new(bearing.cos() * radius, height, bearing.sin() * radius);
        camera.look_at(focus + look, Vec3::Y);
        return;
    }

    // The asteroid field is played from above: the whole ring stays in frame, so
    // pivoting the parked cycle does not whip the camera around with it.
    if let Some((center, radius)) = state.run.as_ref().and_then(field_camera_focus) {
        let height = radius * config::asteroids::ASTEROIDS_CAMERA_FIT + 3.0;
        camera.translation = center
            + Vec3::new(
                0.0,
                height,
                height * config::asteroids::ASTEROIDS_CAMERA_LEAN,
            );
        camera.look_at(center, Vec3::Y);
        return;
    }

    // The Galaga field is played from above too: the whole formation stays in
    // frame while the cycle slides along the bottom.
    if state
        .run
        .as_ref()
        .and_then(|run| run.source_sim::<GalagaSim>())
        .is_some()
    {
        let center = Vec3::ZERO;
        let height = config::galaga::GALAGA_CAMERA_HEIGHT;
        // Lean the camera in from -Z so the cycle (parked at -Z) sits at the
        // bottom of the screen and the formation hangs above it.
        camera.translation =
            center + Vec3::new(0.0, height, -height * config::galaga::GALAGA_CAMERA_LEAN);
        camera.look_at(center, Vec3::Y);
        return;
    }

    // The arcade block: each game gets a small fixed camera tailored to its
    // board, independent of the parked cycle. Every other source game (and
    // plain directory riding) keeps the chase camera below.
    let arcade = state
        .run
        .as_ref()
        .and_then(|run| run.source_game())
        .and_then(SourceGame::arcade_camera);
    if let Some((translation, target)) = arcade {
        camera.translation = translation;
        camera.look_at(target, Vec3::Y);
        return;
    }

    let travel = cycle.rotation * Vec3::X;
    chase.forward = advance_chase_forward(
        chase.forward,
        Vec3::new(travel.x, 0.0, travel.z),
        time.delta_secs(),
    );

    if mouse_buttons.pressed(MouseButton::Right) {
        if mouse_motion.delta != Vec2::ZERO {
            chase.apply_look_drag(mouse_motion.delta);
        }
    } else {
        chase.recenter_look(time.delta_secs());
    }

    let cycle_pos = cycle.translation;
    let surfing = state
        .run
        .as_ref()
        .and_then(|run| run.source_sim::<SurferSim>())
        .is_some();
    let (offset, view_forward) = if surfing {
        surfer_camera_rig(chase.forward, chase.look)
    } else {
        chase_camera_rig(chase.forward, chase.look)
    };
    let lookahead = if surfing {
        config::surfer::SURFER_CAMERA_LOOKAHEAD
    } else {
        config::lightcycle::LIGHTCYCLE_CAMERA_LOOKAHEAD
    };
    let look_target = cycle_pos + view_forward * lookahead;

    let speed_multiplier = if let Some(phys) = physics {
        if state.classic_mode {
            1.0
        } else {
            let ratio = (phys.current_speed / phys.max_speed).clamp(0.0, 1.6);
            1.0 + ratio * 0.12
        }
    } else {
        1.0
    };
    let mut camera_position = cycle_pos + offset * speed_multiplier;

    if let Some(fx) = state.crash_fx.as_ref() {
        let intensity = (fx.timer / fx.duration).clamp(0.0, 1.0);
        let t = time.elapsed_secs();
        let shake =
            Vec3::new((t * 83.0).sin(), (t * 97.0).sin(), (t * 71.0).sin()) * (intensity * 0.9);
        camera_position += shake;
    }

    camera.translation = camera_position;

    let up = if let Some(phys) = physics {
        if state.classic_mode {
            Vec3::Y
        } else {
            let bank = phys.current_lean * 0.22;
            Quat::from_axis_angle(view_forward.normalize_or_zero(), -bank) * Vec3::Y
        }
    } else {
        Vec3::Y
    };
    camera.look_at(look_target, up);
}
