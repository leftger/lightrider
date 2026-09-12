//! World-space maths shared by the arena renderers.

use super::city::{city_body_height, city_body_scale};
use crate::config;
use crate::disc::layout::DiscLayout;
use crate::lightcycle::logic::{Arena, CityStructure, Heading, LightcycleSim};
use crate::lightcycle::scene::pose::cycle_cell_pose;
use crate::lightcycle::scene::pose::heading_angle;
use crate::lightcycle::scene::pose::pose_world_position;
use bevy::prelude::*;

/// Level length for an off-grid run, in metres: longer file, longer level.
pub(crate) fn level_metres(layout: &DiscLayout) -> f32 {
    layout.signals.lines as f32 * config::platformer::PLATFORMER_METRES_PER_LINE
}

/// Middle of a ring, in world units.
pub(crate) fn ring_center_world(layout: &DiscLayout) -> (f32, f32) {
    (
        layout.center.0 as f32 * config::GRID_SPACING,
        layout.center.1 as f32 * config::GRID_SPACING,
    )
}

/// Inner radius of a ring wall, in world units.
pub(crate) fn ring_radius_world(layout: &DiscLayout) -> f32 {
    layout.radius as f32 * config::GRID_SPACING
}

/// Driveable cells of a ring, in a stable order, for scattering power-ups.
pub(crate) fn ring_food_cells(arena: &Arena) -> Vec<(i32, i32)> {
    arena.roads.iter().copied().collect()
}

pub(crate) fn city_cap_mesh(structure: &CityStructure) -> Mesh {
    let body_height = city_body_height(structure);
    let cap_height = config::lightcycle::LIGHTCYCLE_CITY_CAP_HEIGHT;
    let mut scale = city_body_scale(structure, body_height);
    scale.y = cap_height;
    scale.x += 0.08;
    scale.z += 0.08;
    let position = config::ground_position(structure.cell.0, structure.cell.1)
        + Vec3::Y
            * (config::lightcycle::LIGHTCYCLE_CITY_FOUNDATION_HEIGHT
                + body_height
                + cap_height * 0.5);
    Mesh::from(Cuboid::default())
        .transformed_by(Transform::from_translation(position).with_scale(scale))
}

/// Glowing skirt around a structure's footprint. It is wider than the
/// foundation, so the visible part is a neon border tracing where the wall
/// stops and the floor starts.
pub(crate) fn city_base_trim_mesh(structure: &CityStructure) -> Mesh {
    let height = config::lightcycle::LIGHTCYCLE_CITY_BASE_TRIM_HEIGHT;
    let size = config::lightcycle::LIGHTCYCLE_CITY_STRUCTURE_SIZE
        + config::lightcycle::LIGHTCYCLE_CITY_BASE_TRIM_OVERHANG;
    let position =
        config::ground_position(structure.cell.0, structure.cell.1) + Vec3::Y * (height * 0.5);
    Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(position).with_scale(Vec3::new(size, height, size)),
    )
}

pub(crate) fn point_distance(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = b.0 - a.0;
    let dz = b.1 - a.1;
    (dx * dx + dz * dz).sqrt()
}

pub(crate) fn is_path_turn(a: (i32, i32), b: (i32, i32), c: (i32, i32)) -> bool {
    let incoming = (b.0 - a.0, b.1 - a.1);
    let outgoing = (c.0 - b.0, c.1 - b.1);
    incoming.0 * outgoing.0 + incoming.1 * outgoing.1 == 0
}

pub(crate) fn cell_to_point(cell: (i32, i32)) -> (f32, f32) {
    (cell.0 as f32, cell.1 as f32)
}

pub(crate) fn offset_cell_point(
    cell: (i32, i32),
    direction: (i32, i32),
    distance: f32,
) -> (f32, f32) {
    (
        cell.0 as f32 + direction.0 as f32 * distance,
        cell.1 as f32 + direction.1 as f32 * distance,
    )
}

pub(crate) fn cycle_world_position(sim: &LightcycleSim) -> Vec3 {
    pose_world_position(&cycle_cell_pose(sim))
}

/// Facing, in radians, for a grid heading, matching the field's aim convention
/// (`0` is `+X`, growing toward `+Z`).
pub(crate) fn heading_facing(heading: Heading) -> f32 {
    heading_angle(heading)
}

/// The grid heading closest to an aim angle. Used when the field ends so the
/// bike drives off in the direction the player was holding.
pub(crate) fn nearest_heading(angle: f32) -> Heading {
    let (x, z) = (angle.cos(), angle.sin());
    if x.abs() >= z.abs() {
        if x >= 0.0 {
            Heading::PosX
        } else {
            Heading::NegX
        }
    } else if z >= 0.0 {
        Heading::PosZ
    } else {
        Heading::NegZ
    }
}
