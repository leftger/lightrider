//! Chase-camera rigs, cycle poses, and per-game focus helpers.

use super::space::{corner_arc, heading_angle};
use super::{CharacterEntity, CharacterPose, ChaseCamera, CycleEntity, CyclePose, Only};
use crate::asteroids::plugin::field_camera_focus;
use crate::breaker::sim::BreakerSim;
use crate::config;
use crate::disc::language::SourceGame;
use crate::galaga::sim::GalagaSim;
use crate::lightcycle::logic::LightcycleSim;
use crate::lightcycle::{ActiveRun, LightcycleState};
use crate::platformer::sim::PlatformerSim;
use crate::plugins::transition::ModeTransition;
use crate::stealth::plugin::hug_camera_shot;
use crate::stealth::sim::StealthSim;
use crate::surfer::plugin::surfer_camera_rig;
use crate::surfer::sim::SurferSim;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

/// Where the chase camera will sit once a run spawns, the point it looks at,
/// and the road the cycle will ride away down.
pub(crate) fn chase_landing_pose(run: &ActiveRun) -> (Transform, Vec3, Vec3) {
    let pose = cycle_cell_pose(&run.sim);
    let cycle = pose_world_position(&pose);
    let (offset, view_forward) = chase_camera_rig(pose_forward(&pose), Vec2::ZERO);
    let focus = cycle + view_forward * config::lightcycle::LIGHTCYCLE_CAMERA_LOOKAHEAD;
    (
        Transform::from_translation(cycle + offset).looking_at(focus, Vec3::Y),
        focus,
        view_forward,
    )
}

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

/// Ground-level world position of the pose; the model's wheels sit at its origin.
pub(crate) fn pose_world_position(pose: &CyclePose) -> Vec3 {
    Vec3::new(
        pose.position.0 * config::GRID_SPACING,
        0.0,
        pose.position.1 * config::GRID_SPACING,
    )
}

pub(crate) fn pose_forward(pose: &CyclePose) -> Vec3 {
    Vec3::new(pose.direction.x, 0.0, pose.direction.y)
}

/// Yaw along the travel direction, then bank into the corner. The bank rotates
/// about the cycle's own +X, which is its direction of travel, so it leaves the
/// forward vector untouched.
pub(crate) fn pose_rotation(pose: &CyclePose) -> Quat {
    let yaw = match pose_forward(pose).try_normalize() {
        Some(forward) => Quat::from_rotation_arc(Vec3::X, forward),
        None => Quat::IDENTITY,
    };
    yaw * Quat::from_rotation_x(pose.lean)
}

/// Continuous cell-space pose for the rendered cycle.
///
/// Straight segments use the raw simulation position and heading. Near
/// queued/applied turns the pose follows a rounded 90-degree arc around the
/// intersection, taking its facing from the arc's tangent, so the cycle steers
/// through the corner instead of sliding around it and rotating afterwards.
pub(crate) fn cycle_cell_pose(sim: &LightcycleSim) -> CyclePose {
    if let Some(arc) = corner_arc(sim) {
        return arc.sample(arc.u);
    }

    let (dx, dz) = sim.heading.delta();
    CyclePose {
        position: (
            sim.cell.0 as f32 + dx as f32 * sim.cell_t,
            sim.cell.1 as f32 + dz as f32 * sim.cell_t,
        ),
        direction: Vec2::new(dx as f32, dz as f32),
        lean: 0.0,
    }
}

/// Samples the rounded corner centered on `corner` at `u`, where 0 is the arc
/// entry (`radius` before the corner, travelling along `incoming`) and 1 is the
/// exit (`radius` past it, travelling along `outgoing`).
pub(crate) fn arc_cell_pose(
    corner: (i32, i32),
    incoming: (i32, i32),
    outgoing: (i32, i32),
    u: f32,
    radius: f32,
) -> CyclePose {
    let center_x = corner.0 as f32 - incoming.0 as f32 * radius + outgoing.0 as f32 * radius;
    let center_z = corner.1 as f32 - incoming.1 as f32 * radius + outgoing.1 as f32 * radius;

    let start_angle = (-outgoing.1 as f32).atan2(-outgoing.0 as f32);
    let end_angle = (incoming.1 as f32).atan2(incoming.0 as f32);

    let mut sweep = end_angle - start_angle;
    if sweep > std::f32::consts::PI {
        sweep -= std::f32::consts::TAU;
    } else if sweep < -std::f32::consts::PI {
        sweep += std::f32::consts::TAU;
    }

    // Positive sweep curves toward the cycle's right, which is also the
    // direction it should bank.
    let u = u.clamp(0.0, 1.0);
    let theta = start_angle + sweep * u;
    let turn_sign = sweep.signum();

    CyclePose {
        position: (
            center_x + radius * theta.cos(),
            center_z + radius * theta.sin(),
        ),
        direction: Vec2::new(-theta.sin(), theta.cos()) * turn_sign,
        // Peaks mid-corner and returns upright by the exit.
        lean: turn_sign
            * config::lightcycle::LIGHTCYCLE_LEAN_ANGLE
            * (std::f32::consts::PI * u).sin(),
    }
}

/// Eases the camera's follow direction toward `target` with a frame-rate
/// independent time constant.
pub(crate) fn advance_chase_forward(current: Vec3, target: Vec3, delta: f32) -> Vec3 {
    let blend = 1.0 - (-delta / config::lightcycle::LIGHTCYCLE_CAMERA_TURN_LAG).exp();
    current
        .lerp(target, blend.clamp(0.0, 1.0))
        .try_normalize()
        .unwrap_or(target)
}

/// Places the chase rig around the cycle for a follow direction and free-look
/// offset, returning the camera's offset from the cycle and the direction it
/// views along.
///
/// A zero `look` reproduces the fixed rig: [`config::lightcycle::LIGHTCYCLE_CAMERA_DISTANCE`]
/// behind the direction of travel and [`config::lightcycle::LIGHTCYCLE_CAMERA_HEIGHT`] above
/// it. Free look orbits that same radius so dragging never pushes the camera
/// through the floor or into the cycle.
pub(crate) fn chase_camera_rig(forward: Vec3, look: Vec2) -> (Vec3, Vec3) {
    let view_forward = Quat::from_rotation_y(look.x) * forward;
    let pitch = (chase_base_pitch() + look.y).clamp(
        config::lightcycle::LIGHTCYCLE_CAMERA_MIN_PITCH,
        config::lightcycle::LIGHTCYCLE_CAMERA_MAX_PITCH,
    );
    let radius = chase_rig_radius();
    let offset = Vec3::Y * (radius * pitch.sin()) - view_forward * (radius * pitch.cos());
    (offset, view_forward)
}

/// Pitch of the default chase rig above the cycle, in radians.
pub(crate) fn chase_base_pitch() -> f32 {
    config::lightcycle::LIGHTCYCLE_CAMERA_HEIGHT
        .atan2(config::lightcycle::LIGHTCYCLE_CAMERA_DISTANCE)
}

/// Distance from the cycle to the default chase rig.
pub(crate) fn chase_rig_radius() -> f32 {
    Vec2::new(
        config::lightcycle::LIGHTCYCLE_CAMERA_DISTANCE,
        config::lightcycle::LIGHTCYCLE_CAMERA_HEIGHT,
    )
    .length()
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
    mut cycle: Query<(&Transform, &mut ChaseCamera), Without<Camera3d>>,
) {
    let Ok((cycle, mut chase)) = cycle.single_mut() else {
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
    let arcade_game = state
        .run
        .as_ref()
        .and_then(|run| run.source_game())
        .filter(|game| {
            matches!(
                game,
                SourceGame::PacMan
                    | SourceGame::Columns
                    | SourceGame::Tetris
                    | SourceGame::Frogger
                    | SourceGame::Qbert
                    | SourceGame::Bomberman
                    | SourceGame::Plinko
            )
        });
    if let Some(game) = arcade_game {
        let (translation, target) = match game {
            SourceGame::PacMan => (
                Vec3::new(
                    0.0,
                    config::arcade::PAC_CAMERA_HEIGHT,
                    config::arcade::PAC_CAMERA_HEIGHT * config::arcade::PAC_CAMERA_LEAN,
                ),
                Vec3::ZERO,
            ),
            SourceGame::Frogger => (
                Vec3::new(
                    0.0,
                    config::arcade::FROGGER_CAMERA_HEIGHT,
                    config::arcade::FROGGER_CAMERA_HEIGHT * config::arcade::FROGGER_CAMERA_LEAN,
                ),
                Vec3::ZERO,
            ),
            SourceGame::Qbert => (
                Vec3::new(
                    0.0,
                    config::arcade::QBERT_CAMERA_HEIGHT,
                    -config::arcade::QBERT_CAMERA_HEIGHT * config::arcade::QBERT_CAMERA_LEAN,
                ),
                Vec3::new(0.0, 1.0, 0.0),
            ),
            SourceGame::Bomberman => (
                Vec3::new(
                    0.0,
                    config::arcade::BOMBER_CAMERA_HEIGHT,
                    config::arcade::BOMBER_CAMERA_HEIGHT * config::arcade::BOMBER_CAMERA_LEAN,
                ),
                Vec3::ZERO,
            ),
            SourceGame::Columns => (
                Vec3::new(0.0, 10.4, config::arcade::COLUMNS_CAMERA_BACK),
                Vec3::new(0.0, 10.4, 0.0),
            ),
            SourceGame::Tetris => (
                Vec3::new(0.0, 12.0, config::arcade::TETRIS_CAMERA_BACK),
                Vec3::new(0.0, 12.0, 0.0),
            ),
            SourceGame::Plinko => (
                Vec3::new(0.0, 0.0, config::arcade::PLINKO_CAMERA_BACK),
                Vec3::ZERO,
            ),
            _ => unreachable!("filtered to the arcade block above"),
        };
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
    let mut camera_position = cycle_pos + offset;

    if let Some(fx) = state.crash_fx.as_ref() {
        let intensity = (fx.timer / fx.duration).clamp(0.0, 1.0);
        let t = time.elapsed_secs();
        let shake =
            Vec3::new((t * 83.0).sin(), (t * 97.0).sin(), (t * 71.0).sin()) * (intensity * 0.9);
        camera_position += shake;
    }

    camera.translation = camera_position;
    camera.look_at(look_target, Vec3::Y);
}
