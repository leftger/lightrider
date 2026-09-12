//! The TRON city: floors, towers, arterial roads and road markings.

use super::decor::{gate_world_span, spawn_parent_gate};
use super::space::{city_base_trim_mesh, city_cap_mesh};
use super::trail::rail_segments;
use super::{CITY_TRIM_ACCENT, CityBeacon, LightcycleAssets, MarkingQuad};
use crate::config;
use crate::filesystem::node::FileNode;
use crate::lightcycle::logic::{
    Arena, ArenaKind, CityStructure, CityStructureKind, CityTheme, Wall,
};
use crate::lightcycle::{ActiveRun, RunEnvironment};
use crate::state::LightcycleSceneRoot;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

pub(crate) fn city_palette() -> [[(Color, LinearRgba); 2]; 4] {
    [
        [
            (config::LIGHTCYCLE_CITY_CYAN, LinearRgba::rgb(0.0, 2.6, 3.4)),
            (
                config::LIGHTCYCLE_CITY_BLUE,
                LinearRgba::rgb(0.15, 1.0, 3.0),
            ),
        ],
        [
            (
                config::LIGHTCYCLE_CITY_MAGENTA,
                LinearRgba::rgb(3.4, 0.03, 2.0),
            ),
            (config::LIGHTCYCLE_CITY_CYAN, LinearRgba::rgb(0.0, 2.4, 3.2)),
        ],
        [
            (
                config::LIGHTCYCLE_CITY_VIOLET,
                LinearRgba::rgb(1.8, 0.18, 3.4),
            ),
            (config::LIGHTCYCLE_CITY_PINK, LinearRgba::rgb(3.4, 0.2, 1.2)),
        ],
        [
            (
                config::LIGHTCYCLE_CITY_AMBER,
                LinearRgba::rgb(3.4, 1.1, 0.03),
            ),
            (config::LIGHTCYCLE_CITY_CYAN, LinearRgba::rgb(0.0, 2.4, 3.2)),
        ],
    ]
}

pub(crate) fn city_theme_index(theme: CityTheme) -> usize {
    match theme {
        CityTheme::Cyan => 0,
        CityTheme::Magenta => 1,
        CityTheme::Violet => 2,
        CityTheme::Amber => 3,
    }
}

pub(crate) fn tower_position(grid_pos: (i32, i32)) -> (i32, i32) {
    (
        grid_pos.0 * config::LIGHTCYCLE_TOWER_STRIDE,
        grid_pos.1 * config::LIGHTCYCLE_TOWER_STRIDE,
    )
}

pub(crate) fn spawn_city_floor(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    arena: &Arena,
) {
    let spacing = config::GRID_SPACING;
    let span_x = (arena.max.0 - arena.min.0 + 1) as f32 * spacing;
    let span_z = (arena.max.1 - arena.min.1 + 1) as f32 * spacing;
    let center = Vec3::new(
        (arena.min.0 + arena.max.0) as f32 * spacing * 0.5,
        -0.08,
        (arena.min.1 + arena.max.1) as f32 * spacing * 0.5,
    );
    let mesh = Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(center).with_scale(Vec3::new(span_x, 0.12, span_z)),
    );
    let material = match arena.kind {
        ArenaKind::Document => assets.document_floor_material.clone(),
        ArenaKind::Disc => assets.disc_floor_material.clone(),
        ArenaKind::Directory => assets.city_floor_material.clone(),
    };
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        Pickable::IGNORE,
    ));
}

pub(crate) fn spawn_city_structures(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    arena: &Arena,
) {
    let all: Vec<_> = arena.structures.iter().collect();
    spawn_structure_layer(
        commands,
        meshes,
        &all,
        assets.city_foundation_material.clone(),
        city_foundation_mesh,
    );

    let glass: Vec<_> = arena
        .structures
        .iter()
        .filter(|structure| structure.kind == CityStructureKind::GlassFin)
        .collect();
    spawn_structure_layer(
        commands,
        meshes,
        &glass,
        assets.city_glass_material.clone(),
        city_body_mesh,
    );

    let theme = city_theme_index(arena.city_theme);
    for accent in 0..2 {
        let solid: Vec<_> = arena
            .structures
            .iter()
            .filter(|structure| {
                structure.accent as usize == accent && structure.kind != CityStructureKind::GlassFin
            })
            .collect();
        spawn_structure_layer(
            commands,
            meshes,
            &solid,
            assets.city_foundation_material.clone(),
            city_body_mesh,
        );

        let lit: Vec<_> = arena
            .structures
            .iter()
            .filter(|structure| structure.accent as usize == accent)
            .collect();
        spawn_structure_layer(
            commands,
            meshes,
            &lit,
            assets.city_accent_materials[theme][accent].clone(),
            city_cap_mesh,
        );
    }

    // One color for every ground seam, distinct from the lane markings, so the
    // edge you can crash into never reads as a stripe you can drive along.
    spawn_structure_layer(
        commands,
        meshes,
        &all,
        assets.city_accent_materials[theme][CITY_TRIM_ACCENT].clone(),
        city_base_trim_mesh,
    );

    for structure in arena
        .structures
        .iter()
        .filter(|structure| structure.kind == CityStructureKind::Pylon)
        .take(config::LIGHTCYCLE_CITY_BEACON_LIMIT)
    {
        let body_height = city_body_height(structure);
        let (_, emissive) = city_palette()[theme][structure.accent as usize];
        let phase = structure.pulse_phase as f32 / 3.0;
        commands.spawn((
            LightcycleSceneRoot,
            CityBeacon {
                base_height: config::LIGHTCYCLE_CITY_FOUNDATION_HEIGHT + body_height + 0.32,
                phase,
            },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.city_accent_materials[theme][structure.accent as usize].clone()),
            Transform::from_translation(
                config::ground_position(structure.cell.0, structure.cell.1)
                    + Vec3::Y * (config::LIGHTCYCLE_CITY_FOUNDATION_HEIGHT + body_height + 0.32),
            )
            .with_scale(Vec3::splat(0.22 + emissive.red.min(1.0) * 0.04)),
            Pickable::IGNORE,
        ));
    }
}

pub(crate) fn spawn_structure_layer(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    structures: &[&CityStructure],
    material: Handle<StandardMaterial>,
    build: fn(&CityStructure) -> Mesh,
) {
    for chunk in structures.chunks(config::MESH_CHUNK_SIZE) {
        let mut chunk = chunk.iter();
        let Some(first) = chunk.next() else {
            continue;
        };
        let mut mesh = build(first);
        for structure in chunk {
            mesh.merge(&build(structure))
                .expect("city structure meshes must be merge-compatible");
        }
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material.clone()),
            Pickable::IGNORE,
        ));
    }
}

pub(crate) fn city_foundation_mesh(structure: &CityStructure) -> Mesh {
    let height = config::LIGHTCYCLE_CITY_FOUNDATION_HEIGHT;
    Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(config::world_position(
            structure.cell.0,
            structure.cell.1,
            height,
        ))
        .with_scale(Vec3::new(
            config::LIGHTCYCLE_CITY_STRUCTURE_SIZE,
            height,
            config::LIGHTCYCLE_CITY_STRUCTURE_SIZE,
        )),
    )
}

pub(crate) fn city_body_height(structure: &CityStructure) -> f32 {
    let tier = structure.height_tier as f32;
    match structure.kind {
        CityStructureKind::Barrier => config::LIGHTCYCLE_CITY_BARRIER_HEIGHT + tier * 0.24,
        CityStructureKind::GlassFin => config::LIGHTCYCLE_CITY_GLASS_HEIGHT + tier * 0.4,
        CityStructureKind::Pylon => config::LIGHTCYCLE_CITY_PYLON_HEIGHT + tier * 0.7,
    }
}

pub(crate) fn city_body_scale(structure: &CityStructure, height: f32) -> Vec3 {
    let size = config::LIGHTCYCLE_CITY_STRUCTURE_SIZE;
    match structure.kind {
        CityStructureKind::Barrier => {
            if structure.along_x {
                Vec3::new(size, height, size * 0.42)
            } else {
                Vec3::new(size * 0.42, height, size)
            }
        }
        CityStructureKind::GlassFin => {
            if structure.along_x {
                Vec3::new(size, height, config::LIGHTCYCLE_CITY_FIN_THICKNESS)
            } else {
                Vec3::new(config::LIGHTCYCLE_CITY_FIN_THICKNESS, height, size)
            }
        }
        CityStructureKind::Pylon => Vec3::new(0.5, height, 0.5),
    }
}

pub(crate) fn city_body_mesh(structure: &CityStructure) -> Mesh {
    let height = city_body_height(structure);
    let position = config::ground_position(structure.cell.0, structure.cell.1)
        + Vec3::Y * (config::LIGHTCYCLE_CITY_FOUNDATION_HEIGHT + height * 0.5);
    Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(position).with_scale(city_body_scale(structure, height)),
    )
}

pub(crate) fn spawn_towers(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    run: &ActiveRun,
) {
    let RunEnvironment::Directory { nodes, .. } = &run.environment else {
        return;
    };
    for (filter, material) in [
        (
            (|node: &FileNode| node.is_dir) as fn(&FileNode) -> bool,
            assets.dir_tower_material.clone(),
        ),
        (
            |node: &FileNode| !node.is_dir && node.is_source(),
            assets.source_tower_material.clone(),
        ),
        (
            |node: &FileNode| !node.is_dir && node.is_markdown(),
            assets.markdown_tower_material.clone(),
        ),
        (
            |node: &FileNode| !node.is_dir && !node.is_markdown() && !node.is_source(),
            assets.file_tower_material.clone(),
        ),
    ] {
        let matching: Vec<&FileNode> = nodes.iter().filter(|node| filter(node)).collect();
        for chunk in matching.chunks(config::MESH_CHUNK_SIZE) {
            let Some(mesh) = build_tower_chunk_mesh(chunk) else {
                continue;
            };
            commands.spawn((
                LightcycleSceneRoot,
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material.clone()),
                Pickable::IGNORE,
            ));
        }
    }
}

pub(crate) fn build_tower_chunk_mesh(nodes: &[&FileNode]) -> Option<Mesh> {
    let mut nodes = nodes.iter();
    let first = tower_cube_mesh(nodes.next()?);
    let mut mesh = first;
    for node in nodes {
        mesh.merge(&tower_cube_mesh(node))
            .expect("tower cuboid meshes must be merge-compatible");
    }
    Some(mesh)
}

pub(crate) fn tower_cube_mesh(node: &FileNode) -> Mesh {
    let (x, z) = tower_position(node.grid_pos);
    let height = node.calculate_height();
    Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(config::world_position(x, z, height)).with_scale(Vec3::new(
            config::LIGHTCYCLE_TOWER_SIZE,
            height,
            config::LIGHTCYCLE_TOWER_SIZE,
        )),
    )
}

pub(crate) fn spawn_road_markings(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    arena: &Arena,
) {
    let mut cells: Vec<_> = arena.roads.iter().copied().collect();
    // Every road is marked unless the arena is past the ceiling, in which case
    // the cells nearest the middle are the ones kept.
    if cells.len() > config::LIGHTCYCLE_CITY_ROAD_RENDER_LIMIT {
        let center = arena.center();
        cells.sort_unstable_by_key(|cell| {
            ((cell.0 - center.0).abs() + (cell.1 - center.1).abs(), *cell)
        });
        cells.truncate(config::LIGHTCYCLE_CITY_ROAD_RENDER_LIMIT);
    }

    let theme = city_theme_index(arena.city_theme);
    for chunk in cells.chunks(config::MESH_CHUNK_SIZE) {
        // One buffer per chunk, filled straight from the quad descriptions. Going
        // through a mesh per road cell cost a hundred milliseconds on a district
        // with fifty thousand of them, on every directory hop.
        let mut quads = Vec::with_capacity(chunk.len() * 3);
        for cell in chunk {
            push_marking_quads(*cell, &arena.roads, &mut quads);
        }
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(meshes.add(marking_chunk_mesh(&quads))),
            MeshMaterial3d(assets.city_accent_materials[theme][0].clone()),
            Pickable::IGNORE,
        ));
    }
}

/// Appends the quads one road cell contributes: the junction pad, then a lane out
/// to each marked neighbour. Takes the buffer rather than returning one, because
/// a big district asks this for fifty thousand cells at once.
pub(crate) fn push_marking_quads(
    cell: (i32, i32),
    roads: &std::collections::BTreeSet<(i32, i32)>,
    quads: &mut Vec<MarkingQuad>,
) {
    let spacing = config::GRID_SPACING;
    let line_width = 0.075;
    let center = config::ground_position(cell.0, cell.1) + Vec3::Y * config::MARKING_HEIGHT;
    let half = line_width * 1.25;
    quads.push(MarkingQuad {
        center,
        half_x: half,
        half_z: half,
    });
    if roads.contains(&(cell.0 + 1, cell.1)) {
        quads.push(MarkingQuad {
            center: center + Vec3::X * spacing * 0.5,
            half_x: spacing * 0.5,
            half_z: line_width * 0.5,
        });
    }
    if roads.contains(&(cell.0, cell.1 + 1)) {
        quads.push(MarkingQuad {
            center: center + Vec3::Z * spacing * 0.5,
            half_x: line_width * 0.5,
            half_z: spacing * 0.5,
        });
    }
}

/// Packs quads into one mesh, four vertices and two triangles each. The winding
/// runs counter-clockwise seen from above so the faces point up and survive back
/// face culling.
pub(crate) fn marking_chunk_mesh(quads: &[MarkingQuad]) -> Mesh {
    let mut positions = Vec::with_capacity(quads.len() * 4);
    let mut normals = Vec::with_capacity(quads.len() * 4);
    let mut indices = Vec::with_capacity(quads.len() * 6);
    for quad in quads {
        let base = positions.len() as u32;
        let (x, y, z) = (quad.center.x, quad.center.y, quad.center.z);
        for (dx, dz) in [(-1.0, -1.0), (-1.0, 1.0), (1.0, 1.0), (1.0, -1.0)] {
            positions.push([x + dx * quad.half_x, y, z + dz * quad.half_z]);
            normals.push([0.0, 1.0, 0.0]);
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

/// World-space plane each wall rail sits in, half a cell outside the playable area.
pub(crate) fn wall_plane(arena: &Arena, wall: Wall) -> f32 {
    let spacing = config::GRID_SPACING;
    match wall {
        Wall::NegX => (arena.min.0 as f32 - 0.5) * spacing,
        Wall::PosX => (arena.max.0 as f32 + 0.5) * spacing,
        Wall::NegZ => (arena.min.1 as f32 - 0.5) * spacing,
        Wall::PosZ => (arena.max.1 as f32 + 0.5) * spacing,
    }
}

/// World-space extent of a wall along its own axis, corner to corner.
pub(crate) fn wall_extent(arena: &Arena, wall: Wall) -> (f32, f32) {
    match wall {
        Wall::NegZ | Wall::PosZ => (wall_plane(arena, Wall::NegX), wall_plane(arena, Wall::PosX)),
        Wall::NegX | Wall::PosX => (wall_plane(arena, Wall::NegZ), wall_plane(arena, Wall::PosZ)),
    }
}

pub(crate) fn spawn_arena_walls(commands: &mut Commands, assets: &LightcycleAssets, arena: &Arena) {
    for wall in [Wall::NegX, Wall::PosX, Wall::NegZ, Wall::PosZ] {
        spawn_wall_rail(commands, assets, arena, wall);
    }

    spawn_parent_gate(commands, assets, arena);
}

/// Draws one arena wall, leaving a real opening where the parent gate cuts
/// through it so the gate can be ridden through rather than looked at.
pub(crate) fn spawn_wall_rail(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    arena: &Arena,
    wall: Wall,
) {
    let height = config::LIGHTCYCLE_WALL_HEIGHT;
    let thickness = config::LIGHTCYCLE_WALL_THICKNESS;
    let plane = wall_plane(arena, wall);
    let (min, max) = wall_extent(arena, wall);

    let gap = arena
        .parent_portal
        .filter(|portal| portal.wall == wall)
        .map(|portal| gate_world_span(&portal));

    let trim_height = config::LIGHTCYCLE_CITY_BASE_TRIM_HEIGHT;
    let trim_thickness = thickness + config::LIGHTCYCLE_CITY_BASE_TRIM_OVERHANG;
    let accent = if arena.kind == ArenaKind::Document {
        assets.document_folio_material.clone()
    } else {
        assets.city_accent_materials[city_theme_index(arena.city_theme)][CITY_TRIM_ACCENT].clone()
    };

    for (start, end) in rail_segments(min, max, gap) {
        let center = (start + end) * 0.5;
        let length = end - start;
        let (translation, scale) = match wall {
            Wall::NegZ | Wall::PosZ => (
                Vec3::new(center, height * 0.5, plane),
                Vec3::new(length, height, thickness),
            ),
            Wall::NegX | Wall::PosX => (
                Vec3::new(plane, height * 0.5, center),
                Vec3::new(thickness, height, length),
            ),
        };

        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.wall_material.clone()),
            Transform::from_translation(translation).with_scale(scale),
            Pickable::IGNORE,
        ));

        // Same light-line the structures get, so the perimeter reads as a wall
        // standing on the floor rather than the floor fading into darkness.
        let (trim_translation, trim_scale) = match wall {
            Wall::NegZ | Wall::PosZ => (
                Vec3::new(center, trim_height * 0.5, plane),
                Vec3::new(length, trim_height, trim_thickness),
            ),
            Wall::NegX | Wall::PosX => (
                Vec3::new(plane, trim_height * 0.5, center),
                Vec3::new(trim_thickness, trim_height, length),
            ),
        };

        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(accent.clone()),
            Transform::from_translation(trim_translation).with_scale(trim_scale),
            Pickable::IGNORE,
        ));
    }
}
