//! Moved out of `super` by the modularity pass: collapsed_document_glyph, document_arch_mesh, document_glyph_line_mesh, document_line_advance, document_margin_mesh, document_rule_mesh, document_wall_mesh, glyph_char_offset, glyph_pixel_offset, glyph_pixels, spawn_document_arches, spawn_document_focus_marker, spawn_document_glyphs, spawn_document_page, spawn_document_rules, spawn_document_walls.
//!
//! Nothing about them changed in the move.

use super::*;

pub(crate) fn spawn_document_page(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    arena: &Arena,
    layout: &DocumentLayout,
) {
    spawn_document_rules(commands, assets, meshes, arena);
    spawn_document_walls(commands, assets, meshes, arena);
    spawn_document_arches(commands, assets, meshes, layout);
    spawn_document_glyphs(commands, assets, meshes, layout);
}

pub(crate) fn spawn_document_rules(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    arena: &Arena,
) {
    let Some(mesh) = document_rule_mesh(arena) else {
        return;
    };
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(assets.document_rule_material.clone()),
        Pickable::IGNORE,
    ));
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(meshes.add(document_margin_mesh(arena))),
        MeshMaterial3d(assets.document_margin_material.clone()),
        Pickable::IGNORE,
    ));
}

pub(crate) fn document_rule_mesh(arena: &Arena) -> Option<Mesh> {
    let spacing = config::GRID_SPACING;
    let mut merged: Option<Mesh> = None;
    let mut push = |mesh: Mesh| {
        if let Some(existing) = &mut merged {
            existing
                .merge(&mesh)
                .expect("document rule meshes must be merge-compatible");
        } else {
            merged = Some(mesh);
        }
    };

    for z in arena.min.1..=arena.max.1 {
        let center = Vec3::new(
            (arena.min.0 + arena.max.0) as f32 * spacing * 0.5,
            0.02,
            z as f32 * spacing,
        );
        let width = (arena.max.0 - arena.min.0 + 1) as f32 * spacing;
        push(Mesh::from(Cuboid::default()).transformed_by(
            Transform::from_translation(center).with_scale(Vec3::new(width, 0.03, 0.06)),
        ));
    }
    merged
}

pub(crate) fn document_margin_mesh(arena: &Arena) -> Mesh {
    let spacing = config::GRID_SPACING;
    let margin_x = (arena.min.0 as f32 - 0.15) * spacing;
    let center = Vec3::new(
        margin_x,
        0.03,
        (arena.min.1 + arena.max.1) as f32 * spacing * 0.5,
    );
    let depth = (arena.max.1 - arena.min.1 + 1) as f32 * spacing;
    Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(center).with_scale(Vec3::new(0.08, 0.04, depth)),
    )
}

pub(crate) fn spawn_document_walls(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    arena: &Arena,
) {
    let cells: Vec<_> = arena.street_walls.iter().copied().collect();
    for chunk in cells.chunks(config::MESH_CHUNK_SIZE) {
        let mut chunk = chunk.iter();
        let Some(&first) = chunk.next() else {
            continue;
        };
        let mut mesh = document_wall_mesh(first);
        for &cell in chunk {
            mesh.merge(&document_wall_mesh(cell))
                .expect("document wall meshes must be merge-compatible");
        }
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(assets.document_ink_material.clone()),
            Pickable::IGNORE,
        ));
    }
}

pub(crate) fn document_wall_mesh(cell: (i32, i32)) -> Mesh {
    let height = 1.35;
    Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(config::world_position(cell.0, cell.1, height))
            .with_scale(Vec3::new(1.7, height, 0.55)),
    )
}

pub(crate) fn spawn_document_arches(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    layout: &DocumentLayout,
) {
    let headings: Vec<_> = layout
        .blocks
        .iter()
        .filter(|block| matches!(block.kind, crate::document::parse::DocBlockKind::Heading(_)))
        .collect();
    for chunk in headings.chunks(config::MESH_CHUNK_SIZE) {
        let mut chunk = chunk.iter();
        let Some(first) = chunk.next() else {
            continue;
        };
        let mut mesh = document_arch_mesh(first);
        for block in chunk {
            mesh.merge(&document_arch_mesh(block))
                .expect("document arch meshes must be merge-compatible");
        }
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(assets.document_heading_material.clone()),
            Pickable::IGNORE,
        ));
    }
}

pub(crate) fn document_arch_mesh(block: &crate::document::layout::PlacedBlock) -> Mesh {
    let (x, z) = block.landmark;
    let origin = config::ground_position(x, z);
    let (span, depth) = if block.along_x {
        (2.4, 0.28)
    } else {
        (0.28, 2.4)
    };
    let left = Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(origin + Vec3::new(-span * 0.35, 1.1, -depth * 0.35))
            .with_scale(Vec3::new(0.22, 2.2, 0.22)),
    );
    let mut mesh = left;
    mesh.merge(
        &Mesh::from(Cuboid::default()).transformed_by(
            Transform::from_translation(origin + Vec3::new(span * 0.35, 1.1, depth * 0.35))
                .with_scale(Vec3::new(0.22, 2.2, 0.22)),
        ),
    )
    .expect("arch posts must merge");
    mesh.merge(&Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(origin + Vec3::Y * 2.25).with_scale(Vec3::new(
            span,
            0.22,
            depth.max(0.4),
        )),
    ))
    .expect("arch lintel must merge");
    mesh
}

pub(crate) fn spawn_document_glyphs(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    layout: &DocumentLayout,
) {
    let mut remaining = config::DOCUMENT_MAX_GLYPHS;
    let mut heading_mesh: Option<Mesh> = None;
    let mut plaque_mesh: Option<Mesh> = None;
    for block in &layout.blocks {
        if remaining == 0 {
            break;
        }
        let (mesh, used) = document_glyph_line_mesh(block, remaining);
        remaining = remaining.saturating_sub(used);
        if used == 0 {
            continue;
        }
        let target = if matches!(block.kind, crate::document::parse::DocBlockKind::Heading(_)) {
            &mut heading_mesh
        } else {
            &mut plaque_mesh
        };
        if let Some(existing) = target {
            existing
                .merge(&mesh)
                .expect("glyph meshes must be merge-compatible");
        } else {
            *target = Some(mesh);
        }
    }

    if let Some(mesh) = heading_mesh {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(assets.document_heading_material.clone()),
            Pickable::IGNORE,
        ));
    }
    if let Some(mesh) = plaque_mesh {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(assets.document_ink_material.clone()),
            Pickable::IGNORE,
        ));
    }
}

pub(crate) fn document_glyph_line_mesh(
    block: &crate::document::layout::PlacedBlock,
    remaining: usize,
) -> (Mesh, usize) {
    let origin = config::ground_position(block.landmark.0, block.landmark.1);
    let heading = matches!(block.kind, crate::document::parse::DocBlockKind::Heading(_));
    let pixel = if heading { 0.09 } else { 0.055 };
    let height = if heading { 2.55 } else { 0.85 };
    let advance = document_line_advance(block.along_x);
    let max_chars = remaining.min(if heading {
        config::DOCUMENT_HEADING_GLYPHS
    } else {
        config::DOCUMENT_PARAGRAPH_GLYPHS
    });
    let chars: Vec<char> = block.preview.chars().take(max_chars).collect();
    let mut mesh: Option<Mesh> = None;
    let mut used = 0usize;
    for (index, ch) in chars.iter().enumerate() {
        let glyph = glyph_pixels(*ch);
        used += 1;
        let offset = glyph_char_offset(advance, index, chars.len(), pixel);
        for (row, row_bits) in glyph.iter().enumerate() {
            for col in 0..8 {
                if row_bits & (1 << col) == 0 {
                    continue;
                }
                let local = glyph_pixel_offset(advance, col, row, pixel);
                let cube = Mesh::from(Cuboid::default()).transformed_by(
                    Transform::from_translation(origin + Vec3::Y * height + offset + local)
                        .with_scale(Vec3::splat(pixel * 0.85)),
                );
                if let Some(existing) = &mut mesh {
                    existing
                        .merge(&cube)
                        .expect("glyph pixels must be merge-compatible");
                } else {
                    mesh = Some(cube);
                }
            }
        }
    }
    (
        mesh.unwrap_or_else(|| collapsed_document_glyph(origin)),
        used,
    )
}

/// Direction a block's text reads in, which is also the axis its glyph columns
/// run along.
///
/// A paragraph walls off one side of its spine cells, so the reader always
/// arrives from the other side: `+Z` for a row laid along X, `+X` for one laid
/// along Z. Screen right for those two viewpoints is `+X` and `-Z`, and text has
/// to read toward screen right.
pub(crate) fn document_line_advance(along_x: bool) -> Vec3 {
    if along_x { Vec3::X } else { Vec3::NEG_Z }
}

/// Offset of one character's origin from the middle of its line.
pub(crate) fn glyph_char_offset(advance: Vec3, index: usize, count: usize, pixel: f32) -> Vec3 {
    advance * ((index as f32 - (count as f32 - 1.0) * 0.5) * pixel * 9.0)
}

/// Offset of one glyph pixel from its own character's origin.
///
/// Columns run along the same `advance` the characters are placed along, so a
/// letterform cannot end up mirrored against the order of the line it sits in.
/// font8x8 packs each row least-significant bit first, making column 0 the
/// letter's leftmost pixel, so it belongs at the near end of `advance`. Row 0 is
/// the top of the glyph and belongs at the top of the line.
pub(crate) fn glyph_pixel_offset(advance: Vec3, col: usize, row: usize, pixel: f32) -> Vec3 {
    advance * (col as f32 * pixel) + Vec3::Y * ((7 - row) as f32 * pixel)
}

pub(crate) fn collapsed_document_glyph(origin: Vec3) -> Mesh {
    Mesh::from(Cuboid::default()).transformed_by(
        Transform::from_translation(origin + Vec3::Y * 0.2).with_scale(Vec3::splat(0.08)),
    )
}

pub(crate) fn glyph_pixels(character: char) -> [u8; 8] {
    if (character as u32) < 128 {
        font8x8::legacy::BASIC_LEGACY[character as usize]
    } else {
        // Unsupported glyphs keep a diamond placeholder; the folio panel shows
        // the original Unicode.
        [
            0b00011000, 0b00111100, 0b01111110, 0b11111111, 0b01111110, 0b00111100, 0b00011000,
            0b00000000,
        ]
    }
}

pub(crate) fn spawn_document_focus_marker(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    run: &ActiveRun,
) {
    let pose = cycle_cell_pose(&run.sim);
    commands.spawn((
        LightcycleSceneRoot,
        DocumentFocusMarker,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.document_focus_material.clone()),
        Transform::from_translation(pose_world_position(&pose) + Vec3::Y * 0.08)
            .with_scale(Vec3::new(1.4, 0.08, 1.4)),
        Pickable::IGNORE,
    ));
}
