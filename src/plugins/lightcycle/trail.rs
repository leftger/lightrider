//! The liquid-glass trail ribbon that streams off the cycle's tail.

use super::LightcycleAssets;
use super::camera::{arc_cell_pose, cycle_cell_pose};
use super::space::{cell_to_point, corner_arc, is_path_turn, offset_cell_point, point_distance};
use crate::config;
use crate::lightcycle::logic::LightcycleSim;
use crate::lightcycle::{ActiveRun, LightcycleState};
use crate::state::TrailSceneRoot;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

/// Lit transmissive sheet: the directional light and the arena behind it show
/// through, with a cyan tint and a hard specular so it reads as glass rather
/// than an unlit neon brick.
pub(crate) fn trail_glass_material() -> StandardMaterial {
    StandardMaterial {
        base_color: config::LIGHTCYCLE_TRAIL_COLOR,
        perceptual_roughness: 0.08,
        metallic: 0.02,
        specular_transmission: 0.92,
        thickness: 0.28,
        ior: 1.45,
        attenuation_color: config::LIGHTCYCLE_TRAIL_ATTENUATION,
        attenuation_distance: 0.8,
        emissive: LinearRgba::from(config::PCB_TRACE_COLOR) * config::PCB_TRACE_EMISSIVE,
        clearcoat: 1.0,
        clearcoat_perceptual_roughness: 0.06,
        double_sided: true,
        cull_mode: None,
        ..default()
    }
}

pub(crate) fn spawn_trail_ribbon(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    run: &ActiveRun,
) {
    commands.spawn((
        TrailSceneRoot,
        Mesh3d(meshes.add(build_trail_mesh(&run.sim))),
        MeshMaterial3d(assets.trail_material.clone()),
        Pickable::IGNORE,
    ));
}

/// Splits a rail's extent around an optional gap, dropping segments too short to
/// be worth drawing.
pub(crate) fn rail_segments(min: f32, max: f32, gap: Option<(f32, f32)>) -> Vec<(f32, f32)> {
    let Some((gap_min, gap_max)) = gap else {
        return vec![(min, max)];
    };

    [(min, gap_min.min(max)), (gap_max.max(min), max)]
        .into_iter()
        .filter(|(start, end)| end - start > 0.01)
        .collect()
}

/// Rewrites the trail mesh every frame so the live end stays glued to the
/// cycle's tail instead of snapping to the last cell center.
pub(crate) fn update_trail_mesh(
    state: Res<LightcycleState>,
    mut meshes: ResMut<Assets<Mesh>>,
    trail: Query<&Mesh3d, With<TrailSceneRoot>>,
) {
    let Some(run) = state.run.as_ref() else {
        return;
    };
    let Ok(mesh3d) = trail.single() else {
        return;
    };
    let Some(mut mesh) = meshes.get_mut(mesh3d.id()) else {
        return;
    };
    *mesh = build_trail_mesh(&run.sim);
}

pub(crate) fn build_trail_mesh(sim: &LightcycleSim) -> Mesh {
    let points = trail_centerline(sim);
    let heights = trail_heights(&points);
    trail_glass_mesh(&points, &heights)
}

/// Cell-space polyline of the wall: committed trail, the same corner arc the
/// cycle is riding, then trimmed so the live end sits at the tail.
pub(crate) fn trail_centerline(sim: &LightcycleSim) -> Vec<(f32, f32)> {
    let mut points = if sim.trail.is_empty() {
        vec![cell_to_point(sim.cell)]
    } else {
        rounded_polyline(&sim.trail)
    };

    if let Some(arc) = corner_arc(sim) {
        if sim.queued_turn.is_some() {
            points.push(cell_to_point(sim.cell));
        }
        let samples = ((arc.u * 10.0).ceil() as usize).max(2);
        for step in 0..=samples {
            let t = arc.u * step as f32 / samples as f32;
            points.push(arc.sample(t).position);
        }
    } else {
        let pose = cycle_cell_pose(sim);
        points.push(pose.position);
    }

    let points = collapse_near_duplicates(points);
    trim_polyline_end(points, config::LIGHTCYCLE_TRAIL_TAIL)
}

pub(crate) fn trail_heights(points: &[(f32, f32)]) -> Vec<f32> {
    let from_end = distances_from_end(points);
    let emanate = config::LIGHTCYCLE_TRAIL_EMANATE;
    let full = config::LIGHTCYCLE_TRAIL_HEIGHT;
    let spawn = config::LIGHTCYCLE_TRAIL_SPAWN_HEIGHT;

    from_end
        .into_iter()
        .map(|distance| {
            if distance >= emanate {
                full
            } else {
                let t = (distance / emanate).clamp(0.0, 1.0);
                let smooth = t * t * (3.0 - 2.0 * t);
                spawn + (full - spawn) * smooth
            }
        })
        .collect()
}

pub(crate) fn distances_from_end(points: &[(f32, f32)]) -> Vec<f32> {
    if points.is_empty() {
        return Vec::new();
    }

    let mut from_start = vec![0.0; points.len()];
    for index in 1..points.len() {
        from_start[index] =
            from_start[index - 1] + point_distance(points[index - 1], points[index]);
    }
    let total = *from_start.last().unwrap_or(&0.0);
    from_start.into_iter().map(|d| total - d).collect()
}

pub(crate) fn collapse_near_duplicates(points: Vec<(f32, f32)>) -> Vec<(f32, f32)> {
    let mut collapsed = Vec::with_capacity(points.len());
    for point in points {
        if collapsed
            .last()
            .is_none_or(|previous| point_distance(*previous, point) > 1e-4)
        {
            collapsed.push(point);
        }
    }
    collapsed
}

/// Shortens the live end of a polyline by `trim` cells so the wall stops at
/// the tail instead of the cycle's origin.
pub(crate) fn trim_polyline_end(mut points: Vec<(f32, f32)>, trim: f32) -> Vec<(f32, f32)> {
    let mut remaining = trim;
    while points.len() >= 2 && remaining > 1e-4 {
        let last = points.len() - 1;
        let a = points[last - 1];
        let b = points[last];
        let length = point_distance(a, b);
        if length <= 1e-4 {
            points.pop();
            continue;
        }
        if remaining >= length {
            points.pop();
            remaining -= length;
        } else {
            let t = 1.0 - remaining / length;
            points[last] = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            remaining = 0.0;
        }
    }
    points
}

/// Extrudes a thin glass slab along `points`. Heights vary so the live end is
/// a meniscus at the tail rather than a chopped cuboid.
pub(crate) fn trail_glass_mesh(points: &[(f32, f32)], heights: &[f32]) -> Mesh {
    if points.len() < 2 || heights.len() != points.len() {
        return collapsed_trail_mesh(points.first().copied().unwrap_or_default());
    }

    let spacing = config::GRID_SPACING;
    let half_thick = config::LIGHTCYCLE_TRAIL_THICKNESS * 0.5;
    let stations: Vec<(Vec3, Vec3, f32)> = points
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let origin = Vec3::new(point.0 * spacing, 0.0, point.1 * spacing);
            let tangent = polyline_tangent(points, index);
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

        push_quad(
            &mut positions,
            &mut normals,
            &mut indices,
            a_left,
            b_left,
            b_left_top,
            a_left_top,
        );
        push_quad(
            &mut positions,
            &mut normals,
            &mut indices,
            a_right,
            a_right_top,
            b_right_top,
            b_right,
        );
        push_quad(
            &mut positions,
            &mut normals,
            &mut indices,
            a_left_top,
            b_left_top,
            b_right_top,
            a_right_top,
        );
        push_quad(
            &mut positions,
            &mut normals,
            &mut indices,
            a_left,
            a_right,
            b_right,
            b_left,
        );
    }

    let (origin, side, height) = stations[0];
    push_quad(
        &mut positions,
        &mut normals,
        &mut indices,
        origin - side,
        origin - side + Vec3::Y * height,
        origin + side + Vec3::Y * height,
        origin + side,
    );
    let (origin, side, height) = stations[stations.len() - 1];
    push_quad(
        &mut positions,
        &mut normals,
        &mut indices,
        origin - side,
        origin + side,
        origin + side + Vec3::Y * height,
        origin - side + Vec3::Y * height,
    );

    trail_mesh_from(positions, normals, indices)
}

/// An invisible, zero-area quad standing in for a ribbon too short to draw.
///
/// The trail mesh must never be zero-vertex. Bevy's mesh allocator skips
/// allocating a mesh with an empty vertex buffer but still runs the upload for
/// it, which logs `Use-after-free: attempted to copy element data for an
/// unallocated key` every frame. A run has no ribbon yet for the fraction of a
/// cell it takes the tail to clear its spawn, and again after every restart and
/// folder entry, so this is the common case rather than an edge case.
pub(crate) fn collapsed_trail_mesh(anchor: (f32, f32)) -> Mesh {
    let spacing = config::GRID_SPACING;
    let point = Vec3::new(anchor.0 * spacing, 0.0, anchor.1 * spacing);

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    push_quad(
        &mut positions,
        &mut normals,
        &mut indices,
        point,
        point,
        point,
        point,
    );
    trail_mesh_from(positions, normals, indices)
}

pub(crate) fn trail_mesh_from(
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

pub(crate) fn polyline_tangent(points: &[(f32, f32)], index: usize) -> Vec3 {
    let previous = if index == 0 {
        points[0]
    } else {
        points[index - 1]
    };
    let next = if index + 1 == points.len() {
        points[index]
    } else {
        points[index + 1]
    };
    Vec3::new(next.0 - previous.0, 0.0, next.1 - previous.1).normalize_or_zero()
}

pub(crate) fn push_quad(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    a: Vec3,
    b: Vec3,
    c: Vec3,
    d: Vec3,
) {
    let normal = (b - a).cross(d - a).normalize_or_zero();
    let start = positions.len() as u32;
    for vertex in [a, b, c, d] {
        positions.push(vertex.to_array());
        normals.push(normal.to_array());
    }
    indices.extend_from_slice(&[start, start + 1, start + 2, start, start + 2, start + 3]);
}

pub(crate) fn rounded_polyline(path: &[(i32, i32)]) -> Vec<(f32, f32)> {
    let radius = config::LIGHTCYCLE_TURN_RADIUS;
    let mut points = vec![cell_to_point(path[0])];

    for index in 1..path.len().saturating_sub(1) {
        let previous = path[index - 1];
        let corner = path[index];
        let next = path[index + 1];

        if is_path_turn(previous, corner, next) {
            let incoming = (corner.0 - previous.0, corner.1 - previous.1);
            let outgoing = (next.0 - corner.0, next.1 - corner.1);
            let arc_start = offset_cell_point(corner, incoming, -radius);
            points.push(arc_start);

            let samples = 10;
            for step in 1..=samples {
                let u = step as f32 / samples as f32;
                points.push(arc_cell_pose(corner, incoming, outgoing, u, radius).position);
            }
        } else {
            points.push(cell_to_point(corner));
        }
    }

    if let Some(last) = path.last() {
        points.push(cell_to_point(*last));
    }
    points
}
