//! The platformer level geometry and its runner.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::config;
use crate::disc::language::SourceLanguage;
use crate::disc::plugin::disc_language_index;
use crate::platformer::sim::PlatformerSim;
use crate::plugins::lightcycle::CharacterAnim;
use crate::plugins::lightcycle::CharacterEntity;
use crate::plugins::lightcycle::LightcycleAssets;
use crate::state::LightcycleSceneRoot;
use bevy::prelude::*;

/// Spawns a platformer level: a backdrop, the platforms with neon lips, the
/// exit door and the runner.
pub(crate) fn spawn_platformer_level(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    language: SourceLanguage,
    level: &PlatformerSim,
) {
    let _ = meshes;
    let depth = config::PLATFORMER_DEPTH;
    let lip = assets.disc_accent_materials[disc_language_index(language)].clone();

    // A dark slab behind the level so the platforms read against something.
    let span = level.length + 60.0;
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.platform_material.clone()),
        Transform::from_translation(Vec3::new(level.length * 0.5, 8.0, -depth))
            .with_scale(Vec3::new(span, 46.0, 1.0)),
        Pickable::IGNORE,
    ));

    for platform in &level.platforms {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.platform_material.clone()),
            Transform::from_translation(Vec3::new(
                platform.x + platform.w * 0.5,
                platform.y - platform.h * 0.5,
                0.0,
            ))
            .with_scale(Vec3::new(platform.w, platform.h, depth)),
            Pickable::IGNORE,
        ));
        // The lip marks the surface the runner lands on.
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(lip.clone()),
            Transform::from_translation(Vec3::new(platform.x + platform.w * 0.5, platform.y, 0.0))
                .with_scale(Vec3::new(platform.w, 0.14, depth + 0.2)),
            Pickable::IGNORE,
        ));
    }

    let door = level.exit_box();
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.exit_material.clone()),
        Transform::from_translation(Vec3::new(door.x + door.w * 0.5, door.y - door.h * 0.5, 0.0))
            .with_scale(Vec3::new(door.w, door.h, depth * 0.4)),
        Pickable::IGNORE,
    ));

    commands.spawn((
        LightcycleSceneRoot,
        CharacterEntity,
        CharacterAnim::at(Vec3::new(level.runner.x, level.runner.y, 0.0)),
        Transform::from_translation(Vec3::new(level.runner.x, level.runner.y, 0.0)),
        Visibility::default(),
        Pickable::IGNORE,
        children![(
            WorldAssetRoot(assets.tron_scene.clone()),
            Transform::from_rotation(Quat::from_rotation_y(config::TRON_MODEL_YAW))
                .with_scale(Vec3::splat(config::TRON_MODEL_SCALE)),
        )],
    ));
}
