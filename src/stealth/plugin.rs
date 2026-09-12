//! The stealth room: its guard bodies and vision cones.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::config;
use crate::lightcycle::LightcycleState;
use crate::plugins::lightcycle::Apart;
use crate::plugins::lightcycle::CharacterAnim;
use crate::plugins::lightcycle::CharacterEntity;
use crate::plugins::lightcycle::LightcycleAssets;
use crate::plugins::lightcycle::PooledPosed;
use crate::state::LightcycleSceneRoot;
use crate::stealth::sim::StealthSim;
use bevy::prelude::*;

/// One patrol's body, keyed into `StealthSim::guards`.
#[derive(Component)]
pub(crate) struct GuardEntity {
    index: usize,
}

/// One patrol's field-of-vision cone, keyed into `StealthSim::guards`.
#[derive(Component)]
pub(crate) struct GuardConeEntity {
    pub(crate) index: usize,
    /// This guard's own cone, because its shape is cut to what the guard can
    /// actually see. It starts as the plain fan and is replaced once the sim's
    /// rays are available, which is on the first frame.
    pub(crate) mesh: Option<Handle<Mesh>>,
}

/// Spawns a stealth room: floor, cover, the door, the character and the patrols
/// with their vision cones.
pub(crate) fn spawn_stealth_room(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    room: &StealthSim,
) {
    let _ = meshes;
    let (half_w, half_h) = (
        config::STEALTH_WIDTH as f32 * 0.5,
        config::STEALTH_HEIGHT as f32 * 0.5,
    );
    let span = config::GRID_SPACING;
    let width = half_w * 2.0 * span;
    let height = half_h * 2.0 * span;

    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.stealth_floor_material.clone()),
        Transform::from_translation(Vec3::new(0.0, -0.1, 0.0)).with_scale(Vec3::new(
            width + span * 2.0,
            0.2,
            height + span * 2.0,
        )),
        Pickable::IGNORE,
    ));

    for cell in &room.cover {
        let position = config::ground_position(cell.0, cell.1);
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.stealth_wall_material.clone()),
            Transform::from_translation(position + Vec3::Y * config::STEALTH_WALL_HEIGHT * 0.5)
                .with_scale(Vec3::new(
                    span * 0.96,
                    config::STEALTH_WALL_HEIGHT,
                    span * 0.96,
                )),
            Pickable::IGNORE,
        ));
    }

    let exit = config::ground_position(room.exit.0, room.exit.1);
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.stealth_exit_material.clone()),
        Transform::from_translation(exit + Vec3::Y * 1.4).with_scale(Vec3::new(
            span * 0.9,
            2.8,
            span * 0.9,
        )),
        Pickable::IGNORE,
    ));

    let facing = |angle: f32| Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 - angle);
    let body = |assets: &LightcycleAssets, scale: f32| {
        children![(
            WorldAssetRoot(assets.tron_scene.clone()),
            Transform::from_rotation(Quat::from_rotation_y(config::TRON_MODEL_YAW))
                .with_scale(Vec3::splat(scale)),
        )]
    };

    let start = config::ground_position(room.character.0, room.character.1);
    commands.spawn((
        LightcycleSceneRoot,
        CharacterEntity,
        CharacterAnim::at(start),
        Transform::from_translation(start).with_rotation(facing(room.heading.angle())),
        Visibility::default(),
        Pickable::IGNORE,
        body(assets, config::STEALTH_CHARACTER_SCALE),
    ));

    for (index, guard) in room.guards.iter().enumerate() {
        let cell = guard.cell();
        let position = config::ground_position(cell.0, cell.1);
        commands.spawn((
            LightcycleSceneRoot,
            GuardEntity { index },
            Transform::from_translation(position)
                .with_rotation(facing(guard.patrol.heading().angle())),
            Visibility::default(),
            Pickable::IGNORE,
            body(
                assets,
                config::STEALTH_CHARACTER_SCALE * config::STEALTH_GUARD_SCALE,
            ),
        ));
        commands.spawn((
            LightcycleSceneRoot,
            GuardConeEntity { index, mesh: None },
            Mesh3d(assets.vision_cone.clone()),
            MeshMaterial3d(assets.stealth_cone_material.clone()),
            Transform::from_translation(position + Vec3::Y * 0.08)
                .with_rotation(facing(guard.vision_angle()))
                .with_scale(Vec3::splat(config::STEALTH_CONE_REACH)),
            Pickable::IGNORE,
        ));
    }
}

/// Walks the patrols and swings their vision cones.
pub(crate) fn sync_stealth_entities(
    state: Res<LightcycleState>,
    mut guards: PooledPosed<GuardEntity, Apart<GuardConeEntity, CharacterEntity>>,
    mut cones: PooledPosed<GuardConeEntity, Apart<GuardEntity, CharacterEntity>>,
) {
    let Some(room) = state.run.as_ref().and_then(|run| run.source_stealth()) else {
        return;
    };
    for (guard, mut transform) in &mut guards {
        if let Some(guard) = room.guards.get(guard.index) {
            let cell = guard.cell();
            transform.translation = config::ground_position(cell.0, cell.1);
            transform.rotation =
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 - guard.patrol.heading().angle());
        }
    }
    for (cone, mut transform) in &mut cones {
        if let Some(guard) = room.guards.get(cone.index) {
            let cell = guard.cell();
            transform.translation = config::ground_position(cell.0, cell.1) + Vec3::Y * 0.08;
            transform.rotation =
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 - guard.vision_angle());
        }
    }
}
