//! Continuous motorcycle-style physics controller for the lightcycle using Avian 3D.

use avian3d::prelude::*;
use bevy::prelude::*;
use crate::config;
use crate::disc::load::SourceRequested;
use crate::document::load::DocumentRequested;
use crate::lightcycle::logic::{CrashReason, RunPhase};
use crate::lightcycle::scene::CycleEntity;
use crate::lightcycle::{EntryFx, LightcycleState, RunEnvironment};
use crate::load::DirectoryRequested;
use crate::music::sfx::MusicSfx;
use crate::plugins::lightcycle::run::in_lightcycle_mode;
use crate::plugins::lightcycle::space::nearest_heading;
use crate::plugins::lightcycle::trail::{
    collapsed_trail_mesh, push_quad, trail_mesh_from,
};
use crate::plugins::transition::ModeTransition;
use crate::state::{NavigatorResource, PauseState, TrailSceneRoot};

/// Collision layers for separating the cycle, static architecture, hazards, and sensor portals.
#[derive(PhysicsLayer, Default)]
pub enum GameLayer {
    #[default]
    Default,
    Cycle,
    Environment,
    Hazard,
    SensorZone,
}

/// Trigger component attached to directory towers.
#[derive(Component)]
pub struct DirectorySensor(pub usize);

/// Trigger component attached to markdown document towers.
#[derive(Component)]
pub struct DocumentSensor(pub usize);

/// Trigger component attached to source code towers.
#[derive(Component)]
pub struct SourceSensor(pub usize);

/// Trigger component attached to the parent gate.
#[derive(Component)]
pub struct ParentPortalSensor;

/// Marker for solid obstacles (walls, city architecture).
#[derive(Component)]
pub struct SolidObstacle;

/// Component attached to non-openable file towers that cause fatal crashes.
#[derive(Component)]
pub struct NonOpenableFile(pub usize);

/// Continuous motorcycle-like physics controller for the lightcycle.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct LightcyclePhysics {
    /// Throttle input in [-1.0, 1.0] (-1 is reverse/brake, +1 is accelerate).
    pub throttle: f32,
    /// Steering input in [-1.0, 1.0] (-1 is left, +1 is right).
    pub steer: f32,
    /// Handbrake/drift toggle (holding Space).
    pub drifting: bool,
    /// Boost toggle (holding Shift).
    pub boosting: bool,

    /// Current forward speed in world units per second.
    pub current_speed: f32,
    /// Maximum forward speed in normal cruising.
    pub max_speed: f32,
    /// Maximum forward speed during boost.
    pub boost_speed: f32,
    /// Forward acceleration force.
    pub acceleration: f32,
    /// Braking deceleration rate.
    pub braking: f32,
    /// Reverse speed limit.
    pub reverse_speed: f32,

    /// Maximum turn rate (yaw) in radians per second at reference speed.
    pub max_turn_rate: f32,
    /// Reference speed for turn rate scaling.
    pub turn_reference_speed: f32,
    /// Heading angle in radians on the XZ ground plane (0 is +X, PI/2 is +Z).
    pub heading: f32,

    /// Current bank/roll angle in radians around the travel axis.
    pub current_lean: f32,
    /// Target bank/roll angle in radians based on centripetal acceleration.
    pub target_lean: f32,
    /// Maximum lean angle in radians (~55 degrees).
    pub max_lean_angle: f32,
    /// Smoothing speed for leaning.
    pub lean_smoothing: f32,

    /// Lateral tire grip factor: 1.0 = sharp bicycle grip, 0.2 = loose power-slide.
    pub lateral_grip: f32,
    /// Normal lateral grip when cruising.
    pub normal_grip: f32,
    /// Reduced lateral grip when drifting.
    pub drift_grip: f32,
    /// Timer for trail self-collision immunity while rebounding off a solid obstacle.
    pub rebound_timer: f32,
}

impl Default for LightcyclePhysics {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl LightcyclePhysics {
    pub fn new(initial_heading: f32) -> Self {
        Self {
            throttle: 0.0,
            steer: 0.0,
            drifting: false,
            boosting: false,
            current_speed: 14.0, // initial launch speed
            max_speed: 22.0,
            boost_speed: 34.0,
            acceleration: 26.0,
            braking: 35.0,
            reverse_speed: 8.0,
            max_turn_rate: 3.2,
            turn_reference_speed: 12.0,
            heading: initial_heading,
            current_lean: 0.0,
            target_lean: 0.0,
            max_lean_angle: 0.95, // ~54.4 degrees
            lean_smoothing: 12.0,
            lateral_grip: 18.0,
            normal_grip: 18.0,
            drift_grip: 3.5,
            rebound_timer: 0.0,
        }
    }
}

/// Adaptive continuous trail ribbon storing high-resolution keyframes on the ground.
#[derive(Component)]
pub struct ContinuousTrail {
    /// 2D positions in world coordinates (X, Z).
    pub points: Vec<Vec2>,
    /// Cumulative distance from the start to each point.
    pub cumulative_lengths: Vec<f32>,
    /// Maximum trail length in world units before older segments fade.
    pub max_length: f32,
    /// Self-collision grace distance in world units (tail to older segments).
    pub grace_distance: f32,
    /// Minimum distance to append a new keyframe.
    pub min_sample_dist: f32,
    /// Minimum angle change to append a new keyframe.
    pub min_sample_angle: f32,
    /// Last sampled heading angle.
    pub last_sampled_heading: f32,
}

impl Default for ContinuousTrail {
    fn default() -> Self {
        Self {
            points: Vec::with_capacity(128),
            cumulative_lengths: Vec::with_capacity(128),
            max_length: 160.0,
            grace_distance: 4.0,
            min_sample_dist: 0.9,
            min_sample_angle: 0.045, // ~2.6 degrees
            last_sampled_heading: 0.0,
        }
    }
}

impl ContinuousTrail {
    pub fn append(&mut self, point: Vec2, heading: f32) {
        if self.points.is_empty() {
            self.points.push(point);
            self.cumulative_lengths.push(0.0);
            self.last_sampled_heading = heading;
            return;
        }

        let last_point = *self.points.last().unwrap();
        let dist = last_point.distance(point);
        let angle_diff = (heading - self.last_sampled_heading).abs();

        if dist >= self.min_sample_dist || angle_diff >= self.min_sample_angle {
            let total = *self.cumulative_lengths.last().unwrap_or(&0.0) + dist;
            self.points.push(point);
            self.cumulative_lengths.push(total);
            self.last_sampled_heading = heading;

            // Trim old points when exceeding max length
            while self.points.len() > 2 {
                let total = *self.cumulative_lengths.last().unwrap_or(&0.0);
                let first_seg = self.cumulative_lengths[1];
                if total - first_seg > self.max_length {
                    self.points.remove(0);
                    self.cumulative_lengths.remove(0);
                } else {
                    break;
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.cumulative_lengths.clear();
    }
}

/// Computes the squared distance from point `p` to segment `ab`.
pub fn point_to_segment_dist_sq(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq < 1e-6 {
        return (p - a).length_squared();
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    let proj = a + ab * t;
    (p - proj).length_squared()
}

/// Fast 2D segment distance collision check against the trail.
pub fn check_trail_collision(cycle_pos: Vec2, trail: &ContinuousTrail, cycle_radius: f32) -> bool {
    if trail.points.len() < 3 {
        return false;
    }
    let total_dist = *trail.cumulative_lengths.last().unwrap_or(&0.0);
    let thresh_sq = (cycle_radius + config::lightcycle::LIGHTCYCLE_TRAIL_THICKNESS * 0.5).powi(2);

    for i in 0..trail.points.len().saturating_sub(1) {
        let seg_dist = trail.cumulative_lengths[i + 1];
        // Skip segments within the self-collision grace distance from the tail
        if total_dist - seg_dist < trail.grace_distance {
            continue;
        }
        let dist_sq = point_to_segment_dist_sq(cycle_pos, trail.points[i], trail.points[i + 1]);
        if dist_sq <= thresh_sq {
            return true;
        }
    }
    false
}

/// Reads user input for continuous motorcycle physics.
pub fn read_continuous_physics_input(
    keys: Res<ButtonInput<KeyCode>>,
    pause: Res<PauseState>,
    transition: Res<ModeTransition>,
    state: Res<LightcycleState>,
    mut cycle: Query<&mut LightcyclePhysics, With<CycleEntity>>,
) {
    if state.classic_mode || pause.paused || transition.is_active() {
        return;
    }
    let Ok(mut physics) = cycle.single_mut() else {
        return;
    };

    let forward = keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp);
    let backward = keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown);
    physics.throttle = match (forward, backward) {
        (true, false) => 1.0,
        (false, true) => -1.0,
        _ => 0.0,
    };

    let left = keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft);
    let right = keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight);
    physics.steer = match (left, right) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    };

    physics.drifting = keys.pressed(KeyCode::Space);
    physics.boosting = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
}

/// Main continuous physics step for the lightcycle: steering, acceleration, drift, and dynamic lean.
#[allow(clippy::too_many_arguments)]
pub fn step_continuous_physics(
    time: Res<Time>,
    pause: Res<PauseState>,
    transition: Res<ModeTransition>,
    flood: Res<crate::state::FloodState>,
    mut state: ResMut<LightcycleState>,
    mut effects: MessageWriter<MusicSfx>,
    mut cycle_query: Query<(
        &mut Transform,
        &mut LinearVelocity,
        &mut LightcyclePhysics,
        &mut ContinuousTrail,
    ), With<CycleEntity>>,
) {
    if state.classic_mode || pause.paused || transition.is_active() {
        return;
    }

    let Some(mut run) = state.run.take() else {
        return;
    };

    let Ok((mut transform, mut linear_velocity, mut physics, mut trail)) = cycle_query.single_mut() else {
        state.run = Some(run);
        return;
    };

    // If crashed or not running, zero velocity and return
    if run.sim.phase != RunPhase::Running {
        linear_velocity.0 = Vec3::ZERO;
        physics.current_speed = 0.0;
        physics.throttle = 0.0;
        physics.current_lean = 0.0;
        state.run = Some(run);
        return;
    }

    let dt = time.delta_secs();
    physics.rebound_timer = (physics.rebound_timer - dt).max(0.0);

    // 1. Steering with speed scaling
    let speed_ref = physics.turn_reference_speed;
    let current_speed_abs = physics.current_speed.abs().max(3.0);
    let speed_factor = (speed_ref / current_speed_abs).sqrt().clamp(0.45, 1.3);
    let yaw_rate = physics.steer * physics.max_turn_rate * speed_factor;
    physics.heading += yaw_rate * dt;

    // 2. Forward and Right vectors on XZ ground plane
    let fwd_dir = Vec3::new(physics.heading.cos(), 0.0, physics.heading.sin());
    let right_dir = Vec3::new(-physics.heading.sin(), 0.0, physics.heading.cos());

    // 3. Longitudinal Acceleration / Braking
    let target_top = if physics.boosting {
        physics.boost_speed
    } else {
        physics.max_speed
    };

    if physics.throttle > 0.0 {
        physics.current_speed += physics.acceleration * physics.throttle * dt;
        if physics.current_speed > target_top {
            physics.current_speed = target_top;
        }
    } else if physics.throttle < 0.0 {
        physics.current_speed += physics.braking * physics.throttle * dt;
        if physics.current_speed < -physics.reverse_speed {
            physics.current_speed = -physics.reverse_speed;
        }
    } else {
        // Natural rolling friction / engine drag
        let drag = 6.0 * dt;
        if physics.current_speed > 0.0 {
            physics.current_speed = (physics.current_speed - drag).max(0.0);
        } else if physics.current_speed < 0.0 {
            physics.current_speed = (physics.current_speed + drag).min(0.0);
        }
    }

    // 4. Lateral tire grip & drift dynamics
    let current_vel = linear_velocity.0;
    let mut right_vel = current_vel.dot(right_dir);
    let grip = if physics.drifting {
        physics.drift_grip
    } else {
        physics.lateral_grip
    };
    right_vel *= (1.0 - grip * dt).clamp(0.0, 1.0);

    // Apply combined velocity to Avian physics body
    linear_velocity.0 = fwd_dir * physics.current_speed + right_dir * right_vel;

    // 5. Dynamic Lean Angle (Centripetal Acceleration Roll)
    let centripetal_accel = physics.current_speed * yaw_rate;
    physics.target_lean = (centripetal_accel / 9.81).clamp(-physics.max_lean_angle, physics.max_lean_angle);
    let blend = (physics.lean_smoothing * dt).min(1.0);
    physics.current_lean += (physics.target_lean - physics.current_lean) * blend;

    // 6. Update Visual Transform Rotation (Yaw + Dynamic Roll around local travel axis)
    let yaw_quat = Quat::from_rotation_arc(Vec3::X, fwd_dir);
    transform.rotation = yaw_quat * Quat::from_rotation_x(physics.current_lean);

    // 7. Update Adaptive Trail Keyframes
    let tail_offset = config::lightcycle::LIGHTCYCLE_TRAIL_TAIL * config::GRID_SPACING;
    let tail_pos = Vec2::new(
        transform.translation.x - fwd_dir.x * tail_offset,
        transform.translation.z - fwd_dir.z * tail_offset,
    );
    trail.append(tail_pos, physics.heading);

    // 8. Memory Flood Sweep (Red Sweeping Bar) Collision Detection
    if flood.active && flood.timer > flood.delay {
        let flood_z = flood.plane * config::GRID_SPACING;
        let min_x = (run.arena.min.0 as f32 - 1.0) * config::GRID_SPACING;
        let max_x = (run.arena.max.0 as f32 + 1.0) * config::GRID_SPACING;
        if transform.translation.z <= flood_z + 0.5
            && transform.translation.x >= min_x
            && transform.translation.x <= max_x
        {
            run.sim.phase = RunPhase::Crashed;
            run.sim.crash_reason = Some(CrashReason::Hazard);
            run.crash_label = Some("a buffer overflow".to_string());
            linear_velocity.0 = Vec3::ZERO;
            physics.current_speed = 0.0;
            effects.write(MusicSfx::GameOver);
            state.crash_fx = Some(crate::lightcycle::CrashFx::new(
                config::lightcycle::LIGHTCYCLE_CRASH_FX_DURATION,
            ));
        }
    }

    // 9. Trail Collision Detection (suppressed while recovering from obstacle rebound)
    let cycle_pos_2d = Vec2::new(transform.translation.x, transform.translation.z);
    if physics.rebound_timer <= 0.0 && check_trail_collision(cycle_pos_2d, &trail, 0.48) {
        run.sim.phase = RunPhase::Crashed;
        run.sim.crash_reason = Some(CrashReason::Trail);
        run.crash_label = Some("your trail".to_string());
        linear_velocity.0 = Vec3::ZERO;
        physics.current_speed = 0.0;
        effects.write(MusicSfx::GameOver);
        state.crash_fx = Some(crate::lightcycle::CrashFx::new(
            config::lightcycle::LIGHTCYCLE_CRASH_FX_DURATION,
        ));
    }

    // 9. Sync cell coordinate and heading back to ActiveRun for UI/radar
    let cell_x = (transform.translation.x / config::GRID_SPACING).round() as i32;
    let cell_z = (transform.translation.z / config::GRID_SPACING).round() as i32;
    run.sim.cell = (cell_x, cell_z);
    run.sim.heading = nearest_heading(physics.heading);
    run.sim.cell_t = 0.0;
    run.world_position = Some((transform.translation.x, transform.translation.z));
    run.world_heading = Some(physics.heading);
    run.world_velocity = Some((linear_velocity.0.x, linear_velocity.0.z));

    if !state.restore_directory
        && (run.is_source() || run.is_document())
        && run.arena.parent_portal.as_ref().is_some_and(|p| p.contains(run.sim.cell))
    {
        effects.write(MusicSfx::Portal);
        state.restore_directory = true;
    }

    state.run = Some(run);
}

/// Handles collisions between the cycle, filesystem sensors, and solid obstacles.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn handle_lightcycle_collisions(
    mut collision_events: MessageReader<CollisionStart>,
    mut state: ResMut<LightcycleState>,
    mut navigator: ResMut<NavigatorResource>,
    mut requests: MessageWriter<DirectoryRequested>,
    mut documents: MessageWriter<DocumentRequested>,
    mut sources: MessageWriter<SourceRequested>,
    mut effects: MessageWriter<MusicSfx>,
    mut cycle_query: Query<(
        Entity,
        &Transform,
        &mut LinearVelocity,
        Option<&mut LightcyclePhysics>,
        Option<&CollidingEntities>,
    ), With<CycleEntity>>,
    sensors: Query<(
        Option<&DirectorySensor>,
        Option<&DocumentSensor>,
        Option<&SourceSensor>,
        Option<&ParentPortalSensor>,
        Option<&NonOpenableFile>,
        Option<&SolidObstacle>,
        Option<&Transform>,
        Option<&Restitution>,
    ), Without<CycleEntity>>,
) {
    if state.classic_mode {
        return;
    }
    let Ok((cycle_entity, cycle_transform, mut linear_velocity, mut maybe_physics, maybe_colliding)) = cycle_query.single_mut() else {
        return;
    };
    let Some(mut run) = state.run.take() else {
        return;
    };

    if run.sim.phase != RunPhase::Running {
        state.run = Some(run);
        return;
    }

    let mut colliding_targets = Vec::new();
    for event in collision_events.read() {
        let other = if event.collider1 == cycle_entity || event.body1 == Some(cycle_entity) {
            event.collider2
        } else if event.collider2 == cycle_entity || event.body2 == Some(cycle_entity) {
            event.collider1
        } else {
            continue;
        };
        colliding_targets.push(other);
    }

    if let Some(colliding) = maybe_colliding {
        for &other in &colliding.0 {
            if !colliding_targets.contains(&other) {
                colliding_targets.push(other);
            }
        }
    }

    let mut triggered = false;

    for other in colliding_targets {
        let Ok((dir_sensor, doc_sensor, src_sensor, parent_sensor, non_openable, solid, maybe_obs_transform, maybe_restitution)) = sensors.get(other) else {
            continue;
        };

        if let Some(DirectorySensor(index)) = dir_sensor {
            let details = run
                .directory_nodes()
                .and_then(|nodes| nodes.get(*index))
                .map(|node| (node.name.clone(), node.path.clone()));
            if let Some((name, path)) = details {
                run.sim.phase = RunPhase::EnteringDir;
                run.entering_label = Some(format!("DMA → {name}"));
                run.crash_label = None;
                effects.write(MusicSfx::Beam);
                state.entry_fx = Some(EntryFx::new(
                    path,
                    config::lightcycle::LIGHTCYCLE_ENTRY_FX_DURATION,
                ));
                triggered = true;
            }
            break;
        }

        if let Some(DocumentSensor(index)) = doc_sensor {
            let details = run
                .directory_nodes()
                .and_then(|nodes| nodes.get(*index))
                .map(|node| (node.name.clone(), node.path.clone()));
            if let Some((name, path)) = details {
                run.sim.phase = RunPhase::EnteringDir;
                run.entering_label = Some(name);
                run.crash_label = None;
                effects.write(MusicSfx::Beam);
                documents.write(DocumentRequested { path });
                triggered = true;
            }
            break;
        }

        if let Some(SourceSensor(index)) = src_sensor {
            let details = run
                .directory_nodes()
                .and_then(|nodes| nodes.get(*index))
                .map(|node| (node.name.clone(), node.path.clone()));
            if let Some((name, path)) = details {
                run.sim.phase = RunPhase::EnteringDir;
                run.entering_label = Some(name);
                run.crash_label = None;
                effects.write(MusicSfx::Beam);
                sources.write(SourceRequested { path });
                triggered = true;
            }
            break;
        }

        if parent_sensor.is_some() {
            if run.is_document() || run.is_source() {
                effects.write(MusicSfx::Portal);
                state.restore_directory = true;
                triggered = true;
            } else if let Some(parent) = navigator.0.begin_go_to_parent() {
                run.sim.pause_for_directory_change();
                run.entering_label = Some("RET → parent".to_string());
                run.crash_label = None;
                effects.write(MusicSfx::Portal);
                requests.write(DirectoryRequested { path: parent });
                triggered = true;
            }
            break;
        }

        if let Some(NonOpenableFile(index)) = non_openable {
            let file_name = run
                .directory_nodes()
                .and_then(|nodes| nodes.get(*index))
                .map(|node| node.name.clone());
            run.sim.phase = RunPhase::Crashed;
            run.sim.crash_reason = Some(CrashReason::File);
            run.crash_label = file_name.or_else(|| Some("unreadable file".to_string()));
            linear_velocity.0 = Vec3::ZERO;
            if let Some(ref mut phys) = maybe_physics {
                phys.current_speed = 0.0;
            }
            effects.write(MusicSfx::GameOver);
            state.crash_fx = Some(crate::lightcycle::CrashFx::new(
                config::lightcycle::LIGHTCYCLE_CRASH_FX_DURATION,
            ));
            triggered = true;
            break;
        }

        if solid.is_some() {
            let (cycle_pos, obs_pos, obs_scale) = if let Some(obs_tf) = maybe_obs_transform {
                (
                    Vec2::new(cycle_transform.translation.x, cycle_transform.translation.z),
                    Vec2::new(obs_tf.translation.x, obs_tf.translation.z),
                    obs_tf.scale,
                )
            } else {
                (
                    Vec2::new(cycle_transform.translation.x, cycle_transform.translation.z),
                    Vec2::ZERO,
                    Vec3::splat(1.0),
                )
            };

            let hx = obs_scale.x * 0.5;
            let hz = obs_scale.z * 0.5;
            let clamped_x = cycle_pos.x.clamp(obs_pos.x - hx, obs_pos.x + hx);
            let clamped_z = cycle_pos.y.clamp(obs_pos.y - hz, obs_pos.y + hz);
            let closest = Vec2::new(clamped_x, clamped_z);
            let delta = cycle_pos - closest;

            let normal_2d = if delta.length_squared() > 1e-5 {
                delta.normalize()
            } else {
                let dx = cycle_pos.x - obs_pos.x;
                let dz = cycle_pos.y - obs_pos.y;
                let dist_x = hx - dx.abs();
                let dist_z = hz - dz.abs();
                if dist_x < dist_z {
                    Vec2::new(if dx != 0.0 { dx.signum() } else { 1.0 }, 0.0)
                } else {
                    Vec2::new(0.0, if dz != 0.0 { dz.signum() } else { 1.0 })
                }
            };

            let fwd = if let Some(ref phys) = maybe_physics {
                Vec2::new(phys.heading.cos(), phys.heading.sin())
            } else {
                let v = Vec2::new(linear_velocity.0.x, linear_velocity.0.z);
                if v.length_squared() > 1e-4 {
                    v.normalize()
                } else {
                    Vec2::new(1.0, 0.0)
                }
            };

            let normal_3d = Vec3::new(normal_2d.x, 0.0, normal_2d.y);
            let dot = fwd.dot(normal_2d);

            if dot < 0.0 {
                // Avian dynamic restitution scales the reflection elasticity
                let restitution = maybe_restitution.map_or(0.65, |r| r.coefficient);
                let mut reflected = fwd - (1.0 + restitution) * dot * normal_2d;
                if reflected.length_squared() > 1e-4 {
                    reflected = reflected.normalize();
                } else {
                    reflected = normal_2d;
                }

                // Avian's XPBD constraint solver handles non-penetration position projection,
                // eliminating manual push_out translation teleportation that causes corner clipping.
                let incoming_speed = if let Some(ref phys) = maybe_physics {
                    phys.current_speed.abs().max(linear_velocity.0.length())
                } else {
                    linear_velocity.0.length()
                };
                let rebound_speed = (incoming_speed * restitution).clamp(6.0, 16.0);
                linear_velocity.0 = Vec3::new(reflected.x, 0.0, reflected.y) * rebound_speed + normal_3d * 2.0;

                if let Some(ref mut phys) = maybe_physics {
                    phys.heading = reflected.y.atan2(reflected.x);
                    phys.current_speed = rebound_speed;
                    phys.current_lean = -phys.current_lean * 0.4;
                    phys.rebound_timer = 0.8;
                }

                let mut jolt = crate::lightcycle::CrashFx::new(0.22);
                jolt.spawned = true;
                state.crash_fx = Some(jolt);

                effects.write(MusicSfx::Crash);
            } else if let Some(ref mut phys) = maybe_physics {
                phys.rebound_timer = 0.8;
            }

            triggered = true;
            break;
        }
    }

    if !triggered
        && let RunEnvironment::Directory { cells, nodes } = &run.environment
    {
        let cx = (cycle_transform.translation.x / config::GRID_SPACING).round() as i32;
        let cz = (cycle_transform.translation.z / config::GRID_SPACING).round() as i32;
        if let Some(&index) = cells.get(&(cx, cz)) {
            let tower_pos = config::ground_position(cx, cz);
            let dist_sq = (cycle_transform.translation.x - tower_pos.x).powi(2)
                + (cycle_transform.translation.z - tower_pos.z).powi(2);
            let trigger_radius = config::lightcycle::LIGHTCYCLE_TOWER_SIZE * 0.75 + 0.45;
            if dist_sq <= trigger_radius * trigger_radius {
                let node = &nodes[index];
                if node.is_dir {
                    run.sim.phase = RunPhase::EnteringDir;
                    run.entering_label = Some(format!("DMA → {}", node.name));
                    run.crash_label = None;
                    effects.write(MusicSfx::Beam);
                    state.entry_fx = Some(EntryFx::new(
                        node.path.clone(),
                        config::lightcycle::LIGHTCYCLE_ENTRY_FX_DURATION,
                    ));
                } else if node.is_markdown() {
                    run.sim.phase = RunPhase::EnteringDir;
                    run.entering_label = Some(node.name.clone());
                    run.crash_label = None;
                    effects.write(MusicSfx::Beam);
                    documents.write(DocumentRequested {
                        path: node.path.clone(),
                    });
                } else if node.is_source() {
                    run.sim.phase = RunPhase::EnteringDir;
                    run.entering_label = Some(node.name.clone());
                    run.crash_label = None;
                    effects.write(MusicSfx::Beam);
                    sources.write(SourceRequested {
                        path: node.path.clone(),
                    });
                } else {
                    run.sim.phase = RunPhase::Crashed;
                    run.sim.crash_reason = Some(CrashReason::File);
                    run.crash_label = Some(node.name.clone());
                    linear_velocity.0 = Vec3::ZERO;
                    if let Some(ref mut phys) = maybe_physics {
                        phys.current_speed = 0.0;
                    }
                    effects.write(MusicSfx::GameOver);
                    state.crash_fx = Some(crate::lightcycle::CrashFx::new(
                        config::lightcycle::LIGHTCYCLE_CRASH_FX_DURATION,
                    ));
                }
            }
        }
    }

    state.run = Some(run);
}

/// Updates the 3D ribbon mesh using the continuous trail keyframes.
pub fn update_continuous_trail_mesh(
    state: Res<LightcycleState>,
    cycle_query: Query<(&Transform, &LightcyclePhysics, &ContinuousTrail), With<CycleEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    trail_mesh_query: Query<&Mesh3d, With<TrailSceneRoot>>,
) {
    if state.classic_mode {
        return;
    }
    let Ok((transform, physics, trail)) = cycle_query.single() else {
        return;
    };
    let Ok(mesh3d) = trail_mesh_query.single() else {
        return;
    };
    let Some(mut mesh) = meshes.get_mut(mesh3d.id()) else {
        return;
    };

    let tail_offset = config::lightcycle::LIGHTCYCLE_TRAIL_TAIL * config::GRID_SPACING;
    let fwd_dir = Vec3::new(physics.heading.cos(), 0.0, physics.heading.sin());
    let current_tail = Vec2::new(
        transform.translation.x - fwd_dir.x * tail_offset,
        transform.translation.z - fwd_dir.z * tail_offset,
    );

    let mut points = trail.points.clone();
    if points.is_empty() {
        points.push(current_tail);
    }
    points.push(current_tail);

    *mesh = build_continuous_ribbon_mesh(&points);
}

/// Constructs the extruded 3D ribbon mesh from world-space points.
pub fn build_continuous_ribbon_mesh(points: &[Vec2]) -> Mesh {
    if points.len() < 2 {
        let first = points.first().copied().unwrap_or(Vec2::ZERO);
        return collapsed_trail_mesh((first.x / config::GRID_SPACING, first.y / config::GRID_SPACING));
    }

    let heights = continuous_trail_heights(points);
    let half_thick = config::lightcycle::LIGHTCYCLE_TRAIL_THICKNESS * 0.5;

    let stations: Vec<(Vec3, Vec3, f32)> = points
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let origin = Vec3::new(point.x, 0.0, point.y);
            let tangent = continuous_tangent(points, index);
            let side = Vec3::Y.cross(tangent).normalize_or_zero() * half_thick;
            (origin, side, heights[index])
        })
        .collect();

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    for window in stations.windows(2) {
        let (a_origin, a_side, a_height) = window[0];
        let (b_origin, b_side, b_height) = window[1];

        let a_left = a_origin - a_side;
        let a_right = a_origin + a_side;
        let b_left = b_origin - b_side;
        let b_right = b_origin + b_side;
        let a_left_top = a_left + Vec3::Y * a_height;
        let a_right_top = a_right + Vec3::Y * a_height;
        let b_left_top = b_left + Vec3::Y * b_height;
        let b_right_top = b_right + Vec3::Y * b_height;

        push_quad(&mut positions, &mut normals, &mut indices, a_left, b_left, b_left_top, a_left_top);
        push_quad(&mut positions, &mut normals, &mut indices, a_right, a_right_top, b_right_top, b_right);
        push_quad(&mut positions, &mut normals, &mut indices, a_left_top, b_left_top, b_right_top, a_right_top);
        push_quad(&mut positions, &mut normals, &mut indices, a_left, a_right, b_right, b_left);
    }

    let (origin, side, height) = stations[0];
    push_quad(&mut positions, &mut normals, &mut indices, origin - side, origin - side + Vec3::Y * height, origin + side + Vec3::Y * height, origin + side);
    let (origin, side, height) = stations[stations.len() - 1];
    push_quad(&mut positions, &mut normals, &mut indices, origin - side, origin + side, origin + side + Vec3::Y * height, origin - side + Vec3::Y * height);

    trail_mesh_from(positions, normals, indices)
}

fn continuous_tangent(points: &[Vec2], index: usize) -> Vec3 {
    let prev = if index == 0 { points[0] } else { points[index - 1] };
    let next = if index + 1 == points.len() { points[index] } else { points[index + 1] };
    Vec3::new(next.x - prev.x, 0.0, next.y - prev.y).normalize_or_zero()
}

fn continuous_trail_heights(points: &[Vec2]) -> Vec<f32> {
    let emanate = config::lightcycle::LIGHTCYCLE_TRAIL_EMANATE * config::GRID_SPACING;
    let full = config::lightcycle::LIGHTCYCLE_TRAIL_HEIGHT;
    let spawn = config::lightcycle::LIGHTCYCLE_TRAIL_SPAWN_HEIGHT;

    let mut from_start = vec![0.0; points.len()];
    for i in 1..points.len() {
        from_start[i] = from_start[i - 1] + points[i - 1].distance(points[i]);
    }
    let total = *from_start.last().unwrap_or(&0.0);

    from_start
        .into_iter()
        .map(|d| {
            let from_end = total - d;
            if from_end >= emanate {
                full
            } else {
                let t = (from_end / emanate).clamp(0.0, 1.0);
                let smooth = t * t * (3.0 - 2.0 * t);
                spawn + (full - spawn) * smooth
            }
        })
        .collect()
}

pub struct LightcyclePhysicsPlugin;

impl Plugin for LightcyclePhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default())
            .insert_resource(Gravity(Vec3::ZERO))
            .add_systems(
                Update,
                (
                    read_continuous_physics_input.run_if(in_lightcycle_mode),
                    step_continuous_physics.run_if(in_lightcycle_mode),
                    handle_lightcycle_collisions.run_if(in_lightcycle_mode),
                    update_continuous_trail_mesh.run_if(in_lightcycle_mode),
                )
                    .chain(),
            );
    }
}



