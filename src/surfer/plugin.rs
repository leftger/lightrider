//! The river surfer course: water, rocks and gates.
//!
//! The Bevy side of the game lives here, beside its Bevy-free [`super::sim`].

use crate::config;
use crate::plugins::lightcycle::LightcycleAssets;
use crate::state::LightcycleSceneRoot;
use crate::surfer::sim::SurferSim;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;

/// Spawns a river surfer course: the water ribbon that follows the sim's
/// centreline, the rocks, the boost gates and the finish gate. The bike itself
/// is the shared cycle, posed from the sim every frame.
pub(crate) fn spawn_surfer_course(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    surfer: &SurferSim,
) {
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(meshes.add(surfer_river_mesh(surfer))),
        MeshMaterial3d(assets.surfer_water_material.clone()),
        Pickable::IGNORE,
    ));

    for rock in &surfer.rocks {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.surfer_rock_material.clone()),
            Transform::from_translation(Vec3::new(
                rock.x,
                config::SURFER_ROCK_HEIGHT * 0.5,
                rock.z,
            ))
            .with_scale(Vec3::new(
                rock.radius * 2.0,
                config::SURFER_ROCK_HEIGHT,
                rock.radius * 2.0,
            )),
            Pickable::IGNORE,
        ));
    }

    for gate in &surfer.gates {
        spawn_surfer_gate(
            commands,
            assets,
            gate.x,
            gate.z,
            config::SURFER_GATE_SPAN,
            config::SURFER_GATE_HEIGHT,
            &assets.surfer_gate_material,
        );
    }

    let finish_x = surfer.centerline(surfer.length);
    spawn_surfer_gate(
        commands,
        assets,
        finish_x,
        surfer.length,
        surfer.width * 0.9,
        config::SURFER_FINISH_HEIGHT,
        &assets.surfer_finish_material,
    );
}

/// Two posts and a lintel framing a gate opening across the river.
pub(crate) fn spawn_surfer_gate(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    x: f32,
    z: f32,
    span: f32,
    height: f32,
    material: &Handle<StandardMaterial>,
) {
    let post = Vec3::new(0.18, height, 0.18);
    for side in [-1.0_f32, 1.0] {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(Vec3::new(x + side * span, height * 0.5, z))
                .with_scale(post),
            Pickable::IGNORE,
        ));
    }
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(Vec3::new(x, height, z)).with_scale(Vec3::new(
            span * 2.0 + 0.36,
            0.18,
            0.18,
        )),
        Pickable::IGNORE,
    ));
}

/// The water ribbon, tessellated along the sim's centreline so the visual
/// banks match the gameplay banks exactly.
pub(crate) fn surfer_river_mesh(surfer: &SurferSim) -> Mesh {
    let start = -config::SURFER_RIVER_MARGIN;
    let end = surfer.length + config::SURFER_RIVER_MARGIN;
    let steps = ((end - start) / config::SURFER_RIVER_SAMPLE).ceil() as usize;
    let mut positions = Vec::with_capacity((steps + 1) * 2);
    let mut normals = Vec::with_capacity((steps + 1) * 2);
    for step in 0..=steps {
        let z = start + (end - start) * step as f32 / steps as f32;
        let center = surfer.centerline(z);
        positions.push([center - surfer.width, 0.0, z]);
        positions.push([center + surfer.width, 0.0, z]);
        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
    }
    let mut indices = Vec::with_capacity(steps * 6);
    for step in 0..steps as u32 {
        let a = step * 2;
        let b = step * 2 + 1;
        let c = (step + 1) * 2;
        let d = (step + 1) * 2 + 1;
        indices.extend_from_slice(&[a, c, b, b, c, d]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

/// The river surfer's chase rig: lower and closer than the street rig, so the
/// water and the gates read as a course rather than a flyover. Same free-look
/// orbit, same pitch clamps.
pub(crate) fn surfer_camera_rig(forward: Vec3, look: Vec2) -> (Vec3, Vec3) {
    let view_forward = Quat::from_rotation_y(look.x) * forward;
    let pitch = (config::SURFER_CAMERA_HEIGHT.atan2(config::SURFER_CAMERA_DISTANCE) + look.y)
        .clamp(
            config::LIGHTCYCLE_CAMERA_MIN_PITCH,
            config::LIGHTCYCLE_CAMERA_MAX_PITCH,
        );
    let radius = Vec2::new(config::SURFER_CAMERA_DISTANCE, config::SURFER_CAMERA_HEIGHT).length();
    let offset = Vec3::Y * (radius * pitch.sin()) - view_forward * (radius * pitch.cos());
    (offset, view_forward)
}
