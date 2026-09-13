//! Crash debris, camera shake, and the directory-tower transport effect.

use super::decor::smoothstep;
use super::space::cycle_world_position;
use crate::config;
use crate::lightcycle::logic::RunPhase;
use crate::lightcycle::scene::Apart;
use crate::lightcycle::scene::CrashDebris;
use crate::lightcycle::scene::CycleEntity;
use crate::lightcycle::scene::EntryBeam;
use crate::lightcycle::scene::EntryHalo;
use crate::lightcycle::scene::EntryTransportEntity;
use crate::lightcycle::scene::LightcycleAssets;
use crate::lightcycle::scene::Only;
use crate::lightcycle::{ActiveRun, LightcycleState};
use crate::load::DirectoryRequested;
use crate::music::sfx::MusicSfx;
use crate::state::{LightcycleSceneRoot, NavigatorResource};
use avian3d::prelude::Position;
use bevy::prelude::*;

/// Ends a source run through the shared crash path, so the burst, the shake, the
/// label and `R` behave the same as a grid crash.
pub(crate) fn crash_source(
    run: &mut ActiveRun,
    label: &str,
    effects: &mut MessageWriter<MusicSfx>,
) {
    if run.sim.phase != RunPhase::Running {
        return;
    }
    effects.write(MusicSfx::GameOver);
    run.sim.phase = RunPhase::Crashed;
    run.crash_label = Some(label.to_string());
    run.entering_label = None;
}

pub(crate) fn spawn_crash_effect(
    mut state: ResMut<LightcycleState>,
    assets: Res<LightcycleAssets>,
    mut commands: Commands,
    cycle_query: Query<&Transform, With<CycleEntity>>,
) {
    let Some(fx) = state.crash_fx.as_mut() else {
        return;
    };
    if fx.spawned {
        return;
    }
    fx.spawned = true;

    let origin = if let Ok(transform) = cycle_query.single() {
        transform.translation + Vec3::Y * config::lightcycle::LIGHTCYCLE_CYCLE_HEIGHT * 0.5
    } else {
        let Some(run) = state.run.as_ref() else {
            return;
        };
        cycle_world_position(&run.sim)
            + Vec3::Y * config::lightcycle::LIGHTCYCLE_CYCLE_HEIGHT * 0.5
    };
    let count = 18;

    for index in 0..count {
        let angle = index as f32 / count as f32 * std::f32::consts::TAU;
        let speed = 5.0 + (index % 5) as f32 * 1.3;
        let horizontal = Vec3::new(angle.cos(), 0.0, angle.sin());
        let velocity = horizontal * speed + Vec3::Y * (4.0 + (index % 4) as f32 * 1.1);
        let initial_scale = 0.18 + (index % 4) as f32 * 0.05;
        let life = 0.55 + (index % 3) as f32 * 0.1;

        commands.spawn((
            LightcycleSceneRoot,
            CrashDebris {
                velocity,
                life,
                max_life: life,
                initial_scale,
            },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(if index % 3 == 0 {
                assets.trail_material.clone()
            } else {
                assets.crash_material.clone()
            }),
            Transform::from_translation(origin)
                .with_rotation(Quat::from_rotation_y(angle))
                .with_scale(Vec3::splat(initial_scale)),
            Pickable::IGNORE,
        ));
    }
}

pub(crate) fn update_crash_effects(
    time: Res<Time>,
    mut state: ResMut<LightcycleState>,
    mut commands: Commands,
    mut debris: Query<(Entity, &mut Transform, &mut CrashDebris)>,
) {
    let delta = time.delta_secs();
    let gravity = -18.0;

    for (entity, mut transform, mut piece) in &mut debris {
        piece.life -= delta;
        piece.velocity.y += gravity * delta;
        transform.translation += piece.velocity * delta;

        let life_ratio = (piece.life / piece.max_life).max(0.0);
        let scale = piece.initial_scale * life_ratio + 0.02;
        transform.scale = Vec3::splat(scale);

        if piece.life <= 0.0 {
            commands.entity(entity).despawn();
        }
    }

    if let Some(fx) = state.crash_fx.as_mut() {
        fx.timer -= delta;
        if fx.timer <= 0.0 {
            state.crash_fx = None;
        }
    }
}

/// Removes transport geometry when a restart cancels the timeline before a new
/// arena arrives. Normal directory loads remove it with the rest of the old
/// lightcycle scene.
pub(crate) fn cleanup_orphaned_entry_effect(
    state: Res<LightcycleState>,
    mut commands: Commands,
    effects: Query<Entity, With<EntryTransportEntity>>,
) {
    if state.entry_fx.is_some() {
        return;
    }
    for entity in &effects {
        commands.entity(entity).despawn();
    }
}

/// Creates a translucent column and a stack of independent neon rings around
/// the stopped cycle. Navigation is deliberately not started here: the update
/// system below waits until the beam reaches its apex.
pub(crate) fn spawn_entry_effect(
    mut state: ResMut<LightcycleState>,
    assets: Res<LightcycleAssets>,
    mut commands: Commands,
    cycle_query: Query<&Transform, With<CycleEntity>>,
) {
    let Some(fx) = state.entry_fx.as_mut() else {
        return;
    };
    if fx.spawned {
        return;
    }
    fx.spawned = true;

    let origin = if let Ok(transform) = cycle_query.single() {
        Vec3::new(transform.translation.x, 0.0, transform.translation.z)
    } else {
        let Some(run) = state.run.as_ref() else {
            return;
        };
        cycle_world_position(&run.sim)
    };
    commands.spawn((
        LightcycleSceneRoot,
        EntryTransportEntity,
        EntryBeam,
        Mesh3d(assets.entry_beam_mesh.clone()),
        MeshMaterial3d(assets.entry_beam_material.clone()),
        Transform::from_translation(
            origin + Vec3::Y * (config::lightcycle::LIGHTCYCLE_ENTRY_BEAM_HEIGHT * 0.5),
        )
        .with_scale(Vec3::new(0.02, 1.0, 0.02)),
        Pickable::IGNORE,
    ));

    for index in 0..config::lightcycle::LIGHTCYCLE_ENTRY_HALO_COUNT {
        let phase = index as f32 / config::lightcycle::LIGHTCYCLE_ENTRY_HALO_COUNT as f32;
        commands.spawn((
            LightcycleSceneRoot,
            EntryTransportEntity,
            EntryHalo { phase },
            Mesh3d(assets.entry_halo_mesh.clone()),
            MeshMaterial3d(assets.entry_halo_material.clone()),
            Transform::from_translation(origin),
            Pickable::IGNORE,
        ));
    }
}

/// Brightness/size envelope for the whole transport. The quick rise makes the
/// collision read as a capture; the tail collapses as the old arena disappears.
pub(crate) fn entry_effect_envelope(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    if progress < 0.18 {
        smoothstep(progress / 0.18)
    } else {
        1.0 - smoothstep((progress - 0.18) / 0.82)
    }
}

/// Height and scale of one halo in the repeating upward sweep.
pub(crate) fn entry_halo_pose(progress: f32, phase: f32) -> (f32, f32) {
    let sweep = (progress * 2.0 + phase).fract();
    let height = 0.35 + sweep * config::lightcycle::LIGHTCYCLE_ENTRY_HALO_HEIGHT;
    let ring_envelope = (std::f32::consts::PI * sweep).sin().max(0.0);
    let scale = entry_effect_envelope(progress) * (0.35 + ring_envelope * 0.85);
    (height, scale)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn animate_entry_effect(
    time: Res<Time>,
    mut state: ResMut<LightcycleState>,
    mut navigator: ResMut<NavigatorResource>,
    mut requests: MessageWriter<DirectoryRequested>,
    mut beam: Query<&mut Transform, Only<EntryBeam, EntryHalo, CycleEntity>>,
    mut halos: Query<(&EntryHalo, &mut Transform), Apart<EntryBeam>>,
    mut cycle: Query<(&mut Transform, Option<&mut Position>), Only<CycleEntity, EntryBeam, EntryHalo>>,
) {
    let Some(fx) = state.entry_fx.as_mut() else {
        return;
    };
    fx.elapsed += time.delta_secs();
    let progress = fx.progress();
    let envelope = entry_effect_envelope(progress);

    if let Ok(mut transform) = beam.single_mut() {
        let radius = 0.15 + envelope * 0.85;
        transform.scale = Vec3::new(radius, 1.0, radius);
    }
    for (halo, mut transform) in &mut halos {
        let (height, scale) = entry_halo_pose(progress, halo.phase);
        transform.translation.y = height;
        transform.scale = Vec3::splat(scale.max(0.001));
        transform.rotate_y(time.delta_secs() * (1.8 + halo.phase));
    }

    // The cycle rises into the beam only after capture is established.
    if let Ok((mut transform, mut position)) = cycle.single_mut() {
        let lift = smoothstep((progress - 0.28) / 0.72);
        let lift_height = lift * config::lightcycle::LIGHTCYCLE_ENTRY_HALO_HEIGHT * 0.72;
        transform.translation.y = lift_height;
        if let Some(pos) = position.as_mut() {
            pos.0.y = lift_height;
        }
        transform.scale = Vec3::splat(1.0 - lift * 0.72);
    }

    if !fx.requested && progress >= config::lightcycle::LIGHTCYCLE_ENTRY_FX_REQUEST_AT {
        fx.requested = true;
        let target = fx.target.clone();
        navigator.0.begin_navigate_to(&target);
        requests.write(DirectoryRequested { path: target });
    }
}
