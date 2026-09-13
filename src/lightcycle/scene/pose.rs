//! Cycle and character poses, and the maths behind them.
//!
//! Everything here is pure: given a sim state or a direction it returns a
//! pose, so the renderers, the camera and the mini-games can all share it.

use crate::config;
use crate::lightcycle::ActiveRun;
use crate::lightcycle::logic::{Heading, LightcycleSim};
use bevy::prelude::*;

/// Where the on-foot character should be and which way it faces.
pub(crate) struct CharacterPose {
    /// Ground position the figure is walking toward.
    pub(crate) target: Vec3,
    pub(crate) yaw: f32,
    /// True when the sim moves in grid steps that need easing out.
    pub(crate) smooth: bool,
}

/// Continuous render pose for the cycle, in cell coordinates.
pub(crate) struct CyclePose {
    pub(crate) position: (f32, f32),
    /// Unit travel direction; the cycle's nose points along it.
    pub(crate) direction: Vec2,
    /// Bank angle about the travel direction, in radians. Zero outside corners.
    pub(crate) lean: f32,
}

/// The live corner the cycle is riding, if any.
#[derive(Clone, Copy)]
pub(crate) struct CornerArc {
    pub(crate) corner: (i32, i32),
    pub(crate) incoming: (i32, i32),
    pub(crate) outgoing: (i32, i32),
    pub(crate) u: f32,
    pub(crate) radius: f32,
}

impl CornerArc {
    pub(crate) fn sample(self, u: f32) -> CyclePose {
        arc_cell_pose(self.corner, self.incoming, self.outgoing, u, self.radius)
    }
}

/// One wall-hug camera pose: where the camera stands and what it looks at, both
/// as offsets from the character's cell centre, plus the camera height.
pub(crate) struct HugShot {
    pub(crate) offset: Vec3,
    pub(crate) look: Vec3,
    pub(crate) height: f32,
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

pub(crate) fn corner_arc(sim: &LightcycleSim) -> Option<CornerArc> {
    let radius = config::lightcycle::LIGHTCYCLE_TURN_RADIUS;

    // Approaching a queued turn: the first half of the arc happens just before
    // the cycle reaches the intersection cell.
    if let Some(turn) = sim.queued_turn
        && sim.cell_t >= 1.0 - radius
    {
        let incoming = sim.heading.delta();
        let outgoing = sim.heading.turn(turn).delta();
        let u = ((sim.cell_t - (1.0 - radius)) / radius) * 0.5;
        return Some(CornerArc {
            corner: sim.next_cell(),
            incoming,
            outgoing,
            u,
            radius,
        });
    }

    // Just applied a turn: render the second half of the arc after leaving the
    // intersection cell. The previous trail cell tells us the incoming heading.
    if sim.queued_turn.is_none()
        && sim.cell_t <= radius
        && let Some(&previous) = sim.trail.last()
    {
        let incoming = (sim.cell.0 - previous.0, sim.cell.1 - previous.1);
        let outgoing = sim.heading.delta();
        let is_turn = incoming.0 * outgoing.0 + incoming.1 * outgoing.1 == 0;
        if is_turn {
            let u = 0.5 + (sim.cell_t / radius) * 0.5;
            return Some(CornerArc {
                corner: sim.cell,
                incoming,
                outgoing,
                u,
                radius,
            });
        }
    }

    None
}

/// Facing of a grid heading, in radians.
pub(crate) fn heading_angle(heading: Heading) -> f32 {
    heading.angle()
}

/// The unit vector a heading points along, in the `(x, z)` the world is built on.
pub(crate) fn unit_of(heading: Heading) -> (f32, f32) {
    let angle = heading_angle(heading);
    (angle.cos(), angle.sin())
}

/// Wraps an angle into `[-PI, PI)` so recentering unwinds the short way round
/// however many times a drag has spun the camera about the cycle.
pub(crate) fn wrap_angle(angle: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    (angle + PI).rem_euclid(TAU) - PI
}
