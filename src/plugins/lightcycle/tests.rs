use super::camera::{
    arc_cell_pose, chase_camera_rig, chase_rig_radius, cycle_cell_pose, pose_forward,
    pose_rotation, pose_world_position,
};
use super::city::{
    city_body_height, city_body_mesh, city_foundation_mesh, city_palette, city_theme_index,
    marking_chunk_mesh, push_marking_quads,
};
use super::decor::{
    gate_bar_height, gate_pulse, gc_sweep_plane, is_quarantined, stack_frame_glide,
    stack_frame_hover, stack_frame_mesh, stack_frame_rock, stack_plunge, wrap_angle,
};
use super::document::{document_line_advance, glyph_char_offset, glyph_pixel_offset, glyph_pixels};
use super::entry::{entry_effect_envelope, entry_halo_pose};
use super::space::{city_base_trim_mesh, city_cap_mesh, heading_facing, nearest_heading};
use super::trail::{
    build_trail_mesh, rail_segments, trail_centerline, trail_heights, trim_polyline_end,
};
use super::{CITY_TRIM_ACCENT, ChaseCamera, GateScanBar, MarkingQuad};
use crate::config;
use crate::lightcycle::logic::{
    CityStructure, CityStructureKind, CityTheme, Heading, LightcycleSim, Turn,
};
use crate::state::StackMotion;
use crate::stealth::plugin::hug_camera_shot;
use crate::stealth::sim::StealthSim;
use bevy::camera::primitives::MeshAabb;
use bevy::prelude::{Cuboid, Mesh, Vec2, Vec3};
use std::collections::BTreeSet;
use std::path::Path;

const RADIUS: f32 = config::LIGHTCYCLE_TURN_RADIUS;

/// Direction the cycle asset's nose points in its own model space.
///
/// The glTF is Y-up with its length on X, and its canopy peaks near the
/// origin then slopes down to a point toward +X, so the nose is +X.
/// `LIGHTCYCLE_MODEL_YAW` has to rotate this onto the entity's forward
/// axis, and the heading test keeps the two in agreement.
const MODEL_NOSE_AXIS: Vec3 = Vec3::X;

/// A right turn at cell (1, 0): entering along +X, leaving along +Z.
fn right_corner(u: f32) -> super::CyclePose {
    arc_cell_pose((1, 0), (1, 0), (0, 1), u, RADIUS)
}

fn city_structure(kind: CityStructureKind, along_x: bool, tier: u8) -> CityStructure {
    CityStructure {
        cell: (2, -3),
        kind,
        along_x,
        height_tier: tier,
        accent: 0,
        pulse_phase: 0,
    }
}

/// The horizontal view a camera must keep its subjects inside. Half a
/// frame at the narrowest aspect the stealth run has to survive.
const HUG_SHOT_HALF_VIEW: f32 = 30.0 * std::f32::consts::PI / 180.0;

/// Ground-plane angle from `from` to `to`, in radians.
fn horizontal_angle(from: Vec3, to: Vec3) -> f32 {
    let d = to - from;
    d.z.atan2(d.x)
}

/// A room whose character hugs an east wall made of `wall_cells`.
fn hugging_room(wall_cells: &[(i32, i32)]) -> StealthSim {
    let mut room = StealthSim::new(1);
    room.guards.clear();
    room.cover.clear();
    room.character = (0, 0);
    room.cover.extend(wall_cells.iter().copied());
    room.hug = Some(Heading::PosX);
    room.peek = Some(Heading::PosZ);
    room
}

/// Asserts that the imaginary figure at `shot.offset` looking at `shot.look`
/// has every `subject` inside its view.
fn assert_shot_frames(shot: &super::HugShot, subjects: &[(&str, Vec3)]) {
    let camera = shot.offset;
    let view = horizontal_angle(shot.offset, shot.look);
    for (label, point) in subjects {
        let delta = wrap_angle(horizontal_angle(camera, *point) - view).abs();
        assert!(
            delta <= HUG_SHOT_HALF_VIEW,
            "{label} sits {delta:.3} rad from the view centre, past the half-view \
             {HUG_SHOT_HALF_VIEW:.3}",
        );
    }
}

#[test]
fn the_wall_hug_camera_frames_everything_round_the_corner() {
    let room = hugging_room(&[(1, 0)]);
    let shot = hug_camera_shot(&room).expect("a hugging character gets a shot");
    let s = config::GRID_SPACING;
    // The five things the shot has to show at once: the figure, the wall he
    // is hugging, the corner, around the corner, and the corridor right
    // behind the corner.
    assert_shot_frames(
        &shot,
        &[
            ("figure", Vec3::ZERO),
            ("hugged wall", Vec3::new(s * 0.5, 0.0, 0.0)),
            ("corner", Vec3::new(s * 0.5, 0.0, s * 0.5)),
            ("gap", Vec3::new(s, 0.0, s)),
            ("corridor", Vec3::new(s * 2.0, 0.0, s)),
        ],
    );
}

#[test]
fn a_corner_one_cell_on_stands_the_camera_out_farther() {
    let room = hugging_room(&[(1, 0), (1, 1)]);
    let shot = hug_camera_shot(&room).expect("a hugging character gets a shot");
    let s = config::GRID_SPACING;
    // For a wall running east of the figure, the camera's west offset is
    // how far out it stands, and it must grow for a corner that is a cell
    // away or the corridor behind it slips out of frame.
    assert!(
        -shot.offset.x > config::STEALTH_HUG_CAMERA_OUT,
        "the camera should stand out farther than at the corner, got {:?}",
        shot.offset
    );
    assert_shot_frames(
        &shot,
        &[
            ("figure", Vec3::ZERO),
            ("hugged wall", Vec3::new(s * 0.5, 0.0, 0.0)),
            ("corner", Vec3::new(s * 0.5, 0.0, s * 1.5)),
            ("gap", Vec3::new(s, 0.0, s * 2.0)),
            ("corridor", Vec3::new(s * 2.0, 0.0, s * 2.0)),
        ],
    );
}

#[test]
fn a_wall_that_runs_on_gets_a_corridor_shot() {
    let room = hugging_room(&[(1, 0), (1, 1), (1, 2), (1, 3), (1, 4)]);
    let shot = hug_camera_shot(&room).expect("a hugging character gets a shot");
    let s = config::GRID_SPACING;
    // No corner in reach: the camera trails the figure along the wall and
    // looks down the corridor ahead instead of standing out to peek.
    assert!(
        shot.offset.z < 0.0,
        "the camera trails the figure along the wall, got {:?}",
        shot.offset
    );
    assert_shot_frames(
        &shot,
        &[
            ("figure", Vec3::ZERO),
            ("hugged wall", Vec3::new(s * 0.5, 0.0, 0.0)),
            ("corridor ahead", Vec3::new(0.0, 0.0, s * 2.0)),
            ("corridor around the wall", Vec3::new(s, 0.0, s * 2.0)),
        ],
    );
}

#[test]
fn every_city_structure_builds_all_visual_layers() {
    for kind in [
        CityStructureKind::Barrier,
        CityStructureKind::GlassFin,
        CityStructureKind::Pylon,
    ] {
        let structure = city_structure(kind, true, 1);
        assert!(city_foundation_mesh(&structure).count_vertices() > 0);
        assert!(city_body_mesh(&structure).count_vertices() > 0);
        assert!(city_cap_mesh(&structure).count_vertices() > 0);
    }
}

#[test]
fn city_height_tiers_and_silhouettes_are_distinct() {
    for kind in [
        CityStructureKind::Barrier,
        CityStructureKind::GlassFin,
        CityStructureKind::Pylon,
    ] {
        let low = city_structure(kind, true, 0);
        let tall = city_structure(kind, false, 2);
        assert!(city_body_height(&tall) > city_body_height(&low));
    }
}

/// The skirt only delineates the wall/floor seam if it is wider than the
/// foundation it surrounds and sits flat on the ground.
#[test]
fn the_base_trim_outlines_the_footprint_at_ground_level() {
    for kind in [
        CityStructureKind::Barrier,
        CityStructureKind::GlassFin,
        CityStructureKind::Pylon,
    ] {
        let structure = city_structure(kind, true, 1);
        let trim = city_base_trim_mesh(&structure).compute_aabb().unwrap();
        let foundation = city_foundation_mesh(&structure).compute_aabb().unwrap();

        assert!(trim.half_extents.x > foundation.half_extents.x);
        assert!(trim.half_extents.z > foundation.half_extents.z);
        assert!(
            trim.max().y < foundation.max().y,
            "trim should hug the floor, not cover the foundation"
        );
        assert!((trim.min().y).abs() < 1e-5, "trim must start at the ground");
    }
}

#[test]
fn road_markings_join_neighboring_cells() {
    let isolated = BTreeSet::from([(0, 0)]);
    let one_way = BTreeSet::from([(0, 0), (1, 0)]);
    let both_ways = BTreeSet::from([(0, 0), (1, 0), (0, 1)]);
    // A junction pad, plus a lane out to each marked neighbour.
    let count = |roads: &BTreeSet<(i32, i32)>| {
        let mut quads = Vec::new();
        push_marking_quads((0, 0), roads, &mut quads);
        quads.len()
    };
    assert_eq!(count(&isolated), 1);
    assert_eq!(count(&one_way), 2);
    assert_eq!(count(&both_ways), 3);
}

/// Markings are flat quads, so a district of any size can afford them all:
/// no cell may be dropped for being far from the middle.
#[test]
fn lane_markings_cover_a_whole_district() {
    for span in [8_i32, 40, 160] {
        let mut roads = BTreeSet::new();
        for x in 0..span {
            for z in 0..span {
                if x % 4 == 0 || z % 4 == 0 {
                    roads.insert((x, z));
                }
            }
        }
        assert!(roads.len() <= config::LIGHTCYCLE_CITY_ROAD_RENDER_LIMIT);
        let mut quads = Vec::new();
        for cell in &roads {
            push_marking_quads(*cell, &roads, &mut quads);
        }
        assert!(
            quads.len() >= roads.len(),
            "every road cell keeps its junction pad"
        );
        let mesh = marking_chunk_mesh(&quads);
        assert_eq!(mesh.count_vertices(), quads.len() * 4);
        assert_eq!(
            mesh.indices().map(|indices| indices.len()).unwrap_or(0),
            quads.len() * 6
        );
    }
}

/// The quads have to face up: the wrong winding would make every marking in
/// the game invisible to a camera above the floor.
#[test]
fn lane_marking_faces_point_up() {
    let quads = vec![MarkingQuad {
        center: Vec3::new(1.0, 0.025, 2.0),
        half_x: 0.5,
        half_z: 0.25,
    }];
    let mesh = marking_chunk_mesh(&quads);
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|values| values.as_float3())
        .expect("positions");
    let a = Vec3::from(positions[0]);
    let b = Vec3::from(positions[1]);
    let c = Vec3::from(positions[2]);
    let normal = (b - a).cross(c - b);
    assert!(normal.y > 0.0, "quad faces down: {normal:?}");
    assert!(
        positions.iter().all(|p| (p[1] - 0.025).abs() < 1e-6),
        "markings are flat"
    );
}

/// Ground seams must never be drawn in the same color as a lane marking.
#[test]
fn ground_seams_contrast_with_lane_markings_in_every_theme() {
    let palette = city_palette();
    for theme in palette {
        assert_ne!(theme[CITY_TRIM_ACCENT].0, theme[0].0);
    }
}

#[test]
fn every_theme_maps_to_a_two_color_neon_palette() {
    let palette = city_palette();
    for (index, theme) in [
        CityTheme::Cyan,
        CityTheme::Magenta,
        CityTheme::Violet,
        CityTheme::Amber,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(city_theme_index(theme), index);
        assert_ne!(palette[index][0].0, palette[index][1].0);
    }
}

#[test]
fn corner_arc_runs_from_the_incoming_heading_to_the_outgoing_one() {
    let entry = right_corner(0.0);
    let exit = right_corner(1.0);

    assert!((entry.direction - Vec2::new(1.0, 0.0)).length() < 1e-5);
    assert!((exit.direction - Vec2::new(0.0, 1.0)).length() < 1e-5);

    // The arc starts `RADIUS` short of the corner and ends `RADIUS` past it.
    assert!((entry.position.0 - (1.0 - RADIUS)).abs() < 1e-5);
    assert!(entry.position.1.abs() < 1e-5);
    assert!((exit.position.0 - 1.0).abs() < 1e-5);
    assert!((exit.position.1 - RADIUS).abs() < 1e-5);

    // Upright at both ends so straight segments join without a pop.
    assert!(entry.lean.abs() < 1e-5);
    assert!(exit.lean.abs() < 1e-5);
}

#[test]
fn corner_pose_is_continuous_across_the_cell_boundary() {
    let mut before = LightcycleSim::start((0, 0), Heading::PosX);
    before.queue_turn(Turn::Right);
    before.cell_t = 1.0;

    // The simulation applies the turn at the boundary and starts the next cell.
    let mut after = LightcycleSim::start((1, 0), Heading::PosZ);
    after.trail.push((0, 0));

    let before = cycle_cell_pose(&before);
    let after = cycle_cell_pose(&after);

    assert!((before.position.0 - after.position.0).abs() < 1e-5);
    assert!((before.position.1 - after.position.1).abs() < 1e-5);
    assert!((before.direction - after.direction).length() < 1e-5);
    assert!((before.lean - after.lean).abs() < 1e-5);
}

#[test]
fn the_trail_stops_at_the_cycle_tail_not_the_cell_center() {
    let mut sim = LightcycleSim::start((0, 0), Heading::PosX);
    sim.cell_t = 0.7;
    let pose = cycle_cell_pose(&sim);
    let points = trail_centerline(&sim);
    let tail = *points.last().unwrap();

    let expected = (
        pose.position.0 - pose.direction.x * config::LIGHTCYCLE_TRAIL_TAIL,
        pose.position.1 - pose.direction.y * config::LIGHTCYCLE_TRAIL_TAIL,
    );
    assert!((tail.0 - expected.0).abs() < 1e-4);
    assert!((tail.1 - expected.1).abs() < 1e-4);
    assert!(
        (tail.0 - pose.position.0).abs() > 0.2,
        "trail still ends at the bike origin"
    );
}

#[test]
fn the_live_trail_follows_the_same_arc_as_the_cycle() {
    // Second half of a right turn: the tail has already entered the corner,
    // so the ribbon itself must leave the incoming axis.
    let mut sim = LightcycleSim::start((1, 0), Heading::PosZ);
    sim.trail.push((0, 0));
    sim.cell_t = 0.35;
    let points = trail_centerline(&sim);

    let leaves_axis = points.windows(2).any(|pair| {
        let dx = (pair[1].0 - pair[0].0).abs();
        let dz = (pair[1].1 - pair[0].1).abs();
        dx > 1e-4 && dz > 1e-4
    });
    assert!(
        leaves_axis,
        "trail stayed axis-aligned through a corner: {points:?}"
    );
}

#[test]
fn the_trail_is_a_meniscus_at_the_tail_and_full_height_behind_it() {
    let points = vec![(0.0, 0.0), (2.0, 0.0)];
    let heights = trail_heights(&points);
    assert!(
        heights[0] > heights[1],
        "oldest point should be full height"
    );
    assert!((heights[0] - config::LIGHTCYCLE_TRAIL_HEIGHT).abs() < 1e-4);
    assert!((heights[1] - config::LIGHTCYCLE_TRAIL_SPAWN_HEIGHT).abs() < 1e-4);
}

/// A zero-vertex mesh makes Bevy's allocator skip the allocation but still
/// run the upload, logging a use-after-free every frame.
#[test]
fn the_trail_mesh_always_has_vertices() {
    let mut fresh = LightcycleSim::start((0, 0), Heading::PosX);
    assert!(
        build_trail_mesh(&fresh).count_vertices() > 0,
        "a freshly spawned run has no ribbon yet"
    );

    // The whole stretch where the tail has not yet cleared its spawn cell.
    for step in 0..20 {
        fresh.cell_t = step as f32 / 20.0;
        assert!(
            build_trail_mesh(&fresh).count_vertices() > 0,
            "empty mesh at cell_t {}",
            fresh.cell_t
        );
    }

    let mut riding = LightcycleSim::start((2, 0), Heading::PosX);
    riding.trail.extend([(0, 0), (1, 0)]);
    riding.cell_t = 0.5;
    assert!(build_trail_mesh(&riding).count_vertices() > 0);
}

#[test]
fn trimming_the_polyline_end_shortens_it_by_the_asked_distance() {
    let points = trim_polyline_end(vec![(0.0, 0.0), (1.0, 0.0)], 0.25);
    assert_eq!(points.len(), 2);
    assert!((points[1].0 - 0.75).abs() < 1e-5);
    assert!(trim_polyline_end(vec![(0.0, 0.0), (0.1, 0.0)], 0.4).len() < 2);
}

#[test]
fn a_wall_without_a_gate_is_one_unbroken_rail() {
    assert_eq!(rail_segments(-5.0, 5.0, None), vec![(-5.0, 5.0)]);
}

#[test]
fn a_gate_splits_its_wall_into_the_rails_on_either_side() {
    assert_eq!(
        rail_segments(-5.0, 5.0, Some((-1.0, 2.0))),
        vec![(-5.0, -1.0), (2.0, 5.0)]
    );
}

#[test]
fn a_gate_at_a_wall_corner_drops_the_empty_side() {
    assert_eq!(
        rail_segments(-5.0, 5.0, Some((-5.0, -2.0))),
        vec![(-2.0, 5.0)]
    );
    assert_eq!(
        rail_segments(-5.0, 5.0, Some((2.0, 5.0))),
        vec![(-5.0, 2.0)]
    );
}

#[test]
fn a_gate_spanning_the_whole_wall_leaves_no_rail() {
    assert!(rail_segments(-5.0, 5.0, Some((-5.0, 5.0))).is_empty());
}

#[test]
fn gate_frame_pulses_between_its_trough_and_peak() {
    let samples: Vec<_> = (0..64).map(|step| gate_pulse(step as f32 * 0.05)).collect();

    assert!(samples.iter().all(|value| (0.0..=1.0).contains(value)));
    assert!(samples.iter().any(|value| *value > 0.9), "never brightens");
    assert!(samples.iter().any(|value| *value < 0.1), "never dims");
}

#[test]
fn gate_bars_sweep_up_the_opening_and_wrap_at_the_lintel() {
    let travel = config::LIGHTCYCLE_PORTAL_HEIGHT;
    let bar = GateScanBar {
        offset: 0.0,
        travel,
    };

    // Long enough to cover more than one full sweep of the opening.
    let steps = (2.5 / config::LIGHTCYCLE_PORTAL_BAR_SPEED / 0.05) as usize;
    let heights: Vec<_> = (0..steps)
        .map(|step| gate_bar_height(&bar, step as f32 * 0.05))
        .collect();

    assert!(heights.iter().all(|height| (0.0..=travel).contains(height)));
    assert!(heights[1] > heights[0], "bars should rise, not fall");
    assert!(
        heights.windows(2).any(|pair| pair[1] < pair[0]),
        "bars should wrap back to the ground"
    );
}

#[test]
fn gate_bars_stay_evenly_spaced_up_the_opening() {
    let travel = config::LIGHTCYCLE_PORTAL_HEIGHT;
    let count = config::LIGHTCYCLE_PORTAL_BAR_COUNT;
    let bars: Vec<_> = (0..count)
        .map(|index| GateScanBar {
            offset: index as f32 / count as f32,
            travel,
        })
        .collect();

    let mut heights: Vec<_> = bars.iter().map(|bar| gate_bar_height(bar, 0.37)).collect();
    heights.sort_by(f32::total_cmp);

    let expected = travel / count as f32;
    for pair in heights.windows(2) {
        assert!((pair[1] - pair[0] - expected).abs() < 1e-4);
    }
}

/// The rendered cycle must point along its travel direction and stay
/// upright in every heading, including the one antipodal to the model's
/// reference axis.
#[test]
fn cycle_faces_travel_direction_and_stays_upright_in_every_heading() {
    for heading in [Heading::PosX, Heading::NegX, Heading::PosZ, Heading::NegZ] {
        let (dx, dz) = heading.delta();
        let pose = super::CyclePose {
            position: (0.0, 0.0),
            direction: Vec2::new(dx as f32, dz as f32),
            lean: 0.0,
        };
        let rotation = pose_rotation(&pose);
        let model_yaw = bevy::prelude::Quat::from_rotation_y(config::LIGHTCYCLE_MODEL_YAW);

        let travel = super::camera::pose_forward(&pose);
        let nose = rotation * model_yaw * MODEL_NOSE_AXIS;
        assert!(
            nose.dot(travel) > 0.99,
            "{heading:?}: nose {nose:?} should point along travel {travel:?}"
        );

        let up = rotation * model_yaw * Vec3::Y;
        assert!(
            up.y > 0.99,
            "{heading:?}: cycle should stay upright, got {up:?}"
        );
    }
}

#[test]
fn cycle_banks_toward_the_inside_of_the_corner() {
    let right_up = pose_rotation(&right_corner(0.5)) * Vec3::Y;
    let left_up = pose_rotation(&arc_cell_pose((1, 0), (1, 0), (0, -1), 0.5, RADIUS)) * Vec3::Y;

    assert!(right_up.z > 0.1, "a right turn should bank toward +Z");
    assert!(left_up.z < -0.1, "a left turn should bank toward -Z");
}

fn heading_block(along_x: bool, preview: &str) -> crate::document::layout::PlacedBlock {
    crate::document::layout::PlacedBlock {
        kind: crate::document::parse::DocBlockKind::Heading(1),
        text: preview.to_string(),
        preview: preview.to_string(),
        spine: vec![(0, 0)],
        walls: vec![],
        landmark: (0, 0),
        along_x,
    }
}

#[test]
fn heading_glyph_meshes_are_non_empty() {
    let (mesh, used) = super::document::document_glyph_line_mesh(&heading_block(true, "Title"), 24);
    assert!(used > 0);
    assert!(mesh.count_vertices() > 0);
}

#[test]
fn glyph_budget_caps_characters_per_line_and_overall() {
    let long = "A".repeat(80);
    let heading = heading_block(true, &long);
    let (_, used) =
        super::document::document_glyph_line_mesh(&heading, config::DOCUMENT_MAX_GLYPHS);
    assert_eq!(used, config::DOCUMENT_HEADING_GLYPHS);

    let paragraph = crate::document::layout::PlacedBlock {
        kind: crate::document::parse::DocBlockKind::Paragraph,
        text: long.clone(),
        preview: long,
        spine: vec![(0, 0)],
        walls: vec![],
        landmark: (1, 0),
        along_x: true,
    };
    let (_, used) =
        super::document::document_glyph_line_mesh(&paragraph, config::DOCUMENT_MAX_GLYPHS);
    assert_eq!(used, config::DOCUMENT_PARAGRAPH_GLYPHS);

    let leftover = super::document::document_glyph_line_mesh(&heading, 3).1;
    assert_eq!(leftover, 3);
}

#[test]
fn heading_glyphs_follow_block_orientation() {
    let along_x = super::document::document_glyph_line_mesh(&heading_block(true, "HEADING"), 24)
        .0
        .compute_aabb()
        .unwrap();
    let along_z = super::document::document_glyph_line_mesh(&heading_block(false, "HEADING"), 24)
        .0
        .compute_aabb()
        .unwrap();
    assert!(along_x.half_extents.x > along_x.half_extents.z);
    assert!(along_z.half_extents.z > along_z.half_extents.x);
}

/// A glyph's columns and its line's characters have to run the same way. When
/// they disagreed the text rendered mirrored from one side of the page and in
/// reverse character order from the other.
#[test]
fn glyph_columns_read_the_same_way_as_the_characters() {
    for along_x in [true, false] {
        let advance = document_line_advance(along_x);
        let pixel = 0.09;

        let next_char =
            glyph_char_offset(advance, 1, 3, pixel) - glyph_char_offset(advance, 0, 3, pixel);
        let next_column =
            glyph_pixel_offset(advance, 7, 0, pixel) - glyph_pixel_offset(advance, 0, 0, pixel);
        assert!(next_char.dot(advance) > 0.0, "characters must read forward");
        assert!(next_column.dot(advance) > 0.0, "columns must read forward");
        assert!(
            next_char.dot(advance) > next_column.dot(advance),
            "one character has to advance further than one glyph is wide"
        );

        // Row 0 is the top of the glyph, so it must sit highest.
        let top = glyph_pixel_offset(advance, 0, 0, pixel);
        let bottom = glyph_pixel_offset(advance, 0, 7, pixel);
        assert!(top.y > bottom.y);
    }
}

/// Text reads toward screen right for a reader standing on the open side of a
/// paragraph's ink wall: `+X` seen from `+Z`, and `-Z` seen from `+X`.
#[test]
fn text_reads_from_the_side_its_wall_leaves_open() {
    for (along_x, viewer_forward, expected) in [
        (true, Vec3::NEG_Z, Vec3::X),
        (false, Vec3::NEG_X, Vec3::NEG_Z),
    ] {
        let advance = document_line_advance(along_x);
        assert_eq!(advance, expected);
        assert!(
            viewer_forward.cross(Vec3::Y).abs_diff_eq(advance, 1e-6),
            "reading direction must be screen right for that viewpoint"
        );
    }
}

/// [`glyph_pixel_offset`] maps column 0 to the left of the letter, which only
/// holds while font8x8 packs rows least-significant bit first. 'F' pins the
/// order down: its top bar runs from the left edge and stops short of the
/// right, so bit 0 is set and bit 7 is not. Under the opposite convention
/// both would flip and every glyph would render mirrored.
#[test]
fn font_rows_pack_the_leftmost_pixel_in_the_lowest_bit() {
    let top_bar = glyph_pixels('F')[0];
    assert!(top_bar & 1 != 0, "the top bar must start at bit 0");
    assert!(top_bar & (1 << 7) == 0, "and stop short of bit 7");
}

#[test]
fn page_rules_span_the_document_arena() {
    let (arena, _) = crate::document::layout::build_document_arena(
        std::path::Path::new("/docs/page.md"),
        "# A\n\nB\n",
    );
    let aabb = super::document::document_rule_mesh(&arena)
        .unwrap()
        .compute_aabb()
        .unwrap();
    let spacing = config::GRID_SPACING;
    assert!(aabb.half_extents.x * 2.0 >= (arena.max.0 - arena.min.0) as f32 * spacing * 0.9);
    assert!(aabb.half_extents.z * 2.0 >= (arena.max.1 - arena.min.1) as f32 * spacing * 0.9);
}

#[test]
fn directory_and_document_palettes_and_portals_differ() {
    assert_ne!(
        config::DOCUMENT_FLOOR_COLOR,
        config::LIGHTCYCLE_CITY_FLOOR_COLOR
    );
    assert_ne!(
        config::DOCUMENT_FOLIO_COLOR,
        config::LIGHTCYCLE_PORTAL_COLOR
    );
    assert_ne!(config::MARKDOWN_TOWER_COLOR, config::FILE_COLOR);
    assert_ne!(
        config::DOCUMENT_INK_COLOR,
        config::LIGHTCYCLE_CITY_FLOOR_COLOR
    );
}

#[test]
fn document_arenas_have_no_city_skyline() {
    let run =
        super::run::build_document_run(std::path::Path::new("/tmp/note.md"), b"# Hi\n\nHello\n");
    assert_eq!(
        run.arena.kind,
        crate::lightcycle::logic::ArenaKind::Document
    );
    assert!(run.arena.structures.is_empty());
    assert!(!run.arena.roads.is_empty());
    assert!(run.is_document());
}

#[test]
fn closing_a_document_can_rebuild_the_containing_directory() {
    let path = std::path::PathBuf::from("/tmp");
    let nodes = vec![crate::filesystem::node::FileNode::new(
        "note.md".into(),
        path.join("note.md"),
        false,
        12,
        0,
    )];
    let directory = super::run::build_active_run(&path, nodes.clone());
    let document = super::run::build_document_run(&nodes[0].path, b"# Hi\n");
    assert!(document.is_document());
    assert!(!directory.is_document());
    assert_eq!(
        directory.arena.kind,
        crate::lightcycle::logic::ArenaKind::Directory
    );
    let restored = super::run::build_active_run(&path, nodes);
    assert_eq!(restored.arena, directory.arena);
}

#[test]
fn an_unrotated_chase_rig_sits_behind_and_above_the_cycle() {
    let (offset, view_forward) = chase_camera_rig(Vec3::X, Vec2::ZERO);
    assert!(view_forward.abs_diff_eq(Vec3::X, 1e-5));
    assert!(
        offset.abs_diff_eq(
            Vec3::new(
                -config::LIGHTCYCLE_CAMERA_DISTANCE,
                config::LIGHTCYCLE_CAMERA_HEIGHT,
                0.0
            ),
            1e-4
        ),
        "free look at rest must reproduce the fixed rig, got {offset}"
    );
}

/// Entering the lightcycle flies the camera to a rig that `update_chase_camera`
/// then holds on its own, and dives in along the road the cycle is about to
/// ride. Landing anywhere else would pop on the first frame of the run.
#[test]
fn the_flight_lands_on_the_rig_the_chase_camera_will_hold() {
    let path = std::path::PathBuf::from("/tmp");
    let nodes = vec![crate::filesystem::node::FileNode::new(
        "a.txt".into(),
        path.join("a.txt"),
        false,
        12,
        0,
    )];
    let run = super::run::build_active_run(&path, nodes);
    let (landing, focus, road) = super::camera::chase_landing_pose(&run);
    let cycle = pose_world_position(&cycle_cell_pose(&run.sim));

    assert!((landing.translation.distance(cycle) - chase_rig_radius()).abs() < 1e-4);
    assert!((landing.translation.y - config::LIGHTCYCLE_CAMERA_HEIGHT).abs() < 1e-4);
    assert!(
        (landing.rotation * Vec3::NEG_Z)
            .abs_diff_eq((focus - landing.translation).normalize(), 1e-5)
    );

    // The road is the cycle's own heading, which the overhead shot leans on
    // for its roll, so it has to be a unit vector along the ground.
    assert!(road.abs_diff_eq(pose_forward(&cycle_cell_pose(&run.sim)), 1e-5));
    assert!((road.length() - 1.0).abs() < 1e-5 && road.y.abs() < 1e-5);
    assert!(
        focus.abs_diff_eq(cycle + road * config::LIGHTCYCLE_CAMERA_LOOKAHEAD, 1e-4),
        "the shot has to be aimed down the road ahead of the cycle"
    );
}

#[test]
fn free_look_yaw_orbits_the_cycle_at_a_constant_radius_and_height() {
    let (rest, _) = chase_camera_rig(Vec3::X, Vec2::ZERO);
    for steps in 1..8 {
        let yaw = steps as f32 * 0.7;
        let (offset, view_forward) = chase_camera_rig(Vec3::X, Vec2::new(yaw, 0.0));
        assert!((offset.length() - chase_rig_radius()).abs() < 1e-3);
        assert!(
            (offset.y - rest.y).abs() < 1e-4,
            "yaw must not change height"
        );
        assert!((view_forward.length() - 1.0).abs() < 1e-4);
    }
}

/// A drag that runs past the pitch limits must leave the camera above the
/// arena floor and still looking at the cycle rather than straight down it.
#[test]
fn free_look_pitch_stays_within_its_limits() {
    let mut chase = ChaseCamera {
        forward: Vec3::X,
        look: Vec2::ZERO,
    };

    for _ in 0..200 {
        chase.apply_look_drag(Vec2::new(0.0, -50.0));
    }
    let (up, _) = chase_camera_rig(chase.forward, chase.look);
    assert!(up.y > 0.0 && up.y < chase_rig_radius());

    for _ in 0..400 {
        chase.apply_look_drag(Vec2::new(0.0, 50.0));
    }
    let (down, _) = chase_camera_rig(chase.forward, chase.look);
    assert!(down.y > 0.0, "the camera must not drop below the floor");

    // One frame back the other way has to move the camera immediately, not
    // spend itself unwinding rotation banked up past the limit.
    chase.apply_look_drag(Vec2::new(0.0, -20.0));
    let (recovered, _) = chase_camera_rig(chase.forward, chase.look);
    assert!(recovered.y > down.y);
}

#[test]
fn releasing_the_button_settles_free_look_back_behind_the_cycle() {
    let mut chase = ChaseCamera {
        forward: Vec3::X,
        look: Vec2::ZERO,
    };
    chase.apply_look_drag(Vec2::new(-90.0, -40.0));
    let dragged = chase.look;
    assert_ne!(dragged, Vec2::ZERO);

    // Both axes have to ease toward the default rig, not just shrink overall.
    chase.recenter_look(1.0 / 60.0);
    assert!(chase.look.x.abs() < dragged.x.abs());
    assert!(chase.look.y.abs() < dragged.y.abs());

    // The ease has to land exactly home rather than trail an ever-smaller
    // remainder, and it has to get there in a settling time a rider would
    // read as prompt.
    let mut frames = 1;
    while chase.look != Vec2::ZERO {
        assert!(frames < 60 * 4, "free look never settled");
        chase.recenter_look(1.0 / 60.0);
        frames += 1;
    }
    let (offset, view_forward) = chase_camera_rig(chase.forward, chase.look);
    assert!(view_forward.abs_diff_eq(Vec3::X, 1e-5));
    assert!(offset.abs_diff_eq(
        Vec3::new(
            -config::LIGHTCYCLE_CAMERA_DISTANCE,
            config::LIGHTCYCLE_CAMERA_HEIGHT,
            0.0
        ),
        1e-4
    ));
}

/// Spinning the camera several turns must still unwind the short way, rather
/// than rewinding every revolution the drag wound on.
#[test]
fn free_look_yaw_wraps_instead_of_accumulating_revolutions() {
    let mut chase = ChaseCamera {
        forward: Vec3::X,
        look: Vec2::ZERO,
    };
    for _ in 0..300 {
        chase.apply_look_drag(Vec2::new(-25.0, 0.0));
    }
    assert!(chase.look.x.abs() <= std::f32::consts::PI);

    for angle in [-9.0, -3.5, 0.0, 3.5, 9.0] {
        let wrapped = wrap_angle(angle);
        assert!((-std::f32::consts::PI..std::f32::consts::PI).contains(&wrapped));
        let turns = (angle - wrapped) / std::f32::consts::TAU;
        assert!(
            (turns - turns.round()).abs() < 1e-5,
            "wrapping must only remove whole turns"
        );
    }
}

#[test]
fn entry_beam_rises_brightly_then_collapses() {
    assert_eq!(entry_effect_envelope(0.0), 0.0);
    assert_eq!(entry_effect_envelope(1.0), 0.0);
    assert!(entry_effect_envelope(0.18) > 0.99);
    assert!(entry_effect_envelope(0.08) < entry_effect_envelope(0.18));
    assert!(entry_effect_envelope(0.7) < entry_effect_envelope(0.35));
}

#[test]
fn entry_halos_sweep_up_the_beam_at_staggered_heights() {
    let progress = 0.25;
    let poses: Vec<_> = (0..config::LIGHTCYCLE_ENTRY_HALO_COUNT)
        .map(|index| {
            entry_halo_pose(
                progress,
                index as f32 / config::LIGHTCYCLE_ENTRY_HALO_COUNT as f32,
            )
        })
        .collect();

    assert!(poses.iter().all(|(height, scale)| {
        *height >= 0.35 && *height <= config::LIGHTCYCLE_ENTRY_HALO_HEIGHT + 0.35 && *scale > 0.0
    }));
    assert!(
        poses
            .windows(2)
            .any(|pair| (pair[0].0 - pair[1].0).abs() > 0.5),
        "halos should not collapse into one ring"
    );
}

#[test]
fn directory_request_waits_until_near_the_transport_apex() {
    let mut fx = crate::lightcycle::EntryFx::new(
        std::path::PathBuf::from("/next"),
        config::LIGHTCYCLE_ENTRY_FX_DURATION,
    );
    fx.elapsed = fx.duration * 0.5;
    assert!(fx.progress() < config::LIGHTCYCLE_ENTRY_FX_REQUEST_AT);
    fx.elapsed = fx.duration * 0.9;
    assert!(fx.progress() >= config::LIGHTCYCLE_ENTRY_FX_REQUEST_AT);
}

#[test]
fn the_field_facing_round_trips_through_the_grid_headings() {
    for heading in [Heading::PosX, Heading::PosZ, Heading::NegX, Heading::NegZ] {
        assert_eq!(nearest_heading(heading_facing(heading)), heading);
    }
    // Near a diagonal the nearest cardinal leans toward the dominant axis.
    assert_eq!(
        nearest_heading(std::f32::consts::FRAC_PI_4 * 0.9),
        Heading::PosX
    );
    assert_eq!(
        nearest_heading(std::f32::consts::FRAC_PI_4 * 1.1),
        Heading::PosZ
    );
}

/// Each language must build only its own sim, so the field-only behaviour
/// (parked bike, overhead camera, pivot input) can never leak into another
/// ring. This is the regression test for the overhead camera that once
/// appeared in disc wars.
#[test]
fn each_source_language_builds_only_its_own_game() {
    use crate::disc::language::SourceLanguage;
    let run = |name: &str, language: SourceLanguage, body: &[u8]| {
        super::run::build_source_run(std::path::Path::new(name), language, body)
    };

    let field = run("/tmp/field.c", SourceLanguage::C, b"int main(void) {}\n");
    assert!(field.source_asteroids().is_some());
    assert!(field.source_disc().is_none() && field.source_snake().is_none());
    assert!(field.asteroid_field_active());
    assert!(
        crate::asteroids::plugin::field_camera_focus(&field).is_some(),
        "the field plays from above"
    );

    let ring = run("/tmp/ring.rs", SourceLanguage::Rust, b"fn main() {}\n");
    assert!(ring.source_disc().is_some());
    assert!(ring.source_asteroids().is_none() && ring.source_snake().is_none());
    assert!(!ring.asteroid_field_active());
    assert!(
        crate::asteroids::plugin::field_camera_focus(&ring).is_none(),
        "a disc-wars ring must keep the chase camera"
    );

    let snake = run("/tmp/snake.py", SourceLanguage::Python, b"print('hi')\n");
    assert!(snake.source_snake().is_some());
    assert!(snake.source_disc().is_none() && snake.source_asteroids().is_none());
    assert!(!snake.asteroid_field_active());
    assert!(
        crate::asteroids::plugin::field_camera_focus(&snake).is_none(),
        "snake drives on the grid, so it keeps the chase camera"
    );
    let snake = snake.source_snake().expect("snake state");
    assert_eq!(snake.food.len(), config::SNAKE_FOOD_TARGET);
    assert!(!snake.exit_open, "the exit starts locked");

    // The three games that build their own space off the grid.
    let level = run(
        "/tmp/run.slint",
        SourceLanguage::Slint,
        b"export component App {}\n",
    );
    assert!(level.source_platformer().is_some());
    assert!(!level.asteroid_field_active());
    assert!(
        level.source_platformer().expect("level").platforms.len() > 2,
        "a level should have platforms to run"
    );

    let court = run("/tmp/init.lua", SourceLanguage::Lua, b"local x = 1\n");
    assert!(court.source_breaker().is_some());
    assert!(court.source_breaker().expect("court").remaining() > 0);

    let room = run("/tmp/build.sh", SourceLanguage::Shell, b"set -e\n");
    assert!(room.source_stealth().is_some());
    assert!(!room.source_stealth().expect("room").guards.is_empty());
}

#[test]
fn a_locked_snake_gate_is_a_wall_until_it_opens() {
    let run = super::run::build_source_run(
        std::path::Path::new("/tmp/snake.py"),
        crate::disc::language::SourceLanguage::Python,
        b"print('hi')\n",
    );
    let portal = run.arena.parent_portal.as_ref().expect("a close gate");
    assert!(
        super::decor::is_ring_gate(&run.arena, portal.to),
        "the portal cell is the gate"
    );
    assert!(
        !run.source_snake().expect("snake state").exit_open,
        "so the gate is solid at the start of the run"
    );
}

#[test]
fn heavy_build_directories_are_quarantined() {
    assert!(is_quarantined(Path::new("/home/me/project/node_modules")));
    assert!(is_quarantined(Path::new("/home/me/project/target/debug")));
    assert!(is_quarantined(Path::new("/srv/.git")));
    assert!(!is_quarantined(Path::new("/home/me/project/src")));
    assert!(!is_quarantined(Path::new("/home/me")));
}

#[test]
fn a_fresh_session_has_not_ridden_yet_so_the_opening_room_is_grace() {
    let state = crate::lightcycle::LightcycleState::default();
    assert!(
        !state.rides_started,
        "the opening room is hazard-free until the first ride starts"
    );
}

#[test]
fn the_collectors_sweep_crosses_the_whole_arena() {
    let duration = config::GC_SWEEP_SECONDS;
    assert_eq!(gc_sweep_plane(duration, duration, -4.0, 8.0), -4.0);
    assert_eq!(gc_sweep_plane(0.0, duration, -4.0, 8.0), 8.0);
    let middle = gc_sweep_plane(duration * 0.5, duration, -4.0, 8.0);
    assert!((middle - 2.0).abs() < 0.001, "half way is the middle");
    assert!(
        gc_sweep_plane(duration * 0.25, duration, -4.0, 8.0)
            > gc_sweep_plane(duration * 0.75, duration, -4.0, 8.0),
        "as the countdown falls, the sweep moves on"
    );
}

#[test]
fn a_plunge_starts_and_ends_at_the_resting_height() {
    assert_eq!(stack_plunge(0, 0.0), 0.0, "the dive starts at the ceiling");
    assert_eq!(stack_plunge(0, 1.0), 0.0, "and returns to it");
}

#[test]
fn a_plunge_rests_at_the_bottom_in_the_middle_and_never_overshoots() {
    // Level 0's window starts at the top of the event, so its timeline is
    // the event scaled by the stagger span.
    let span = 1.0 - config::STACK_PLUNGE_STAGGER * (config::STACK_FRAME_MAX - 1) as f32;
    let hold = config::STACK_PLUNGE_HOLD;
    let leg = (1.0 - hold) * 0.5;
    let down_end = leg * span;
    let up_start = (leg + hold) * span;

    assert!(
        (stack_plunge(0, (down_end + up_start) * 0.5) - 1.0).abs() < 1e-6,
        "the middle of the hold is the mirror of the resting height"
    );

    // The descent only ever goes down, and ends at the full drop.
    let mut previous = 0.0;
    for step in 0..=200 {
        let progress = down_end * step as f32 / 200.0;
        let dive = stack_plunge(0, progress);
        assert!(
            (0.0..=1.0).contains(&dive),
            "dive {dive} left its travel at {progress}"
        );
        assert!(
            dive >= previous - 1e-4,
            "the descent came back up early at {progress}"
        );
        previous = dive;
    }
    assert!((previous - 1.0).abs() < 1e-6, "the bottom is the full drop");

    // And the return only comes up, landing home.
    let mut previous = 1.0;
    for step in 0..=200 {
        let progress = up_start + (span - up_start) * step as f32 / 200.0;
        let dive = stack_plunge(0, progress);
        assert!(dive <= previous + 1e-4, "the return dipped at {progress}");
        previous = dive;
    }
    assert!(previous.abs() < 1e-6, "and it lands back at rest");
}

#[test]
fn deeper_frames_lag_the_dive_and_finish_with_it() {
    let early = 0.2;
    assert!(
        stack_plunge(3, early) < stack_plunge(0, early),
        "a deeper frame starts its dive later"
    );
    for level in 0..config::STACK_FRAME_MAX {
        assert_eq!(
            stack_plunge(level, 1.0),
            0.0,
            "every frame is home by the end of the event"
        );
        assert_eq!(stack_plunge(level, 0.0), 0.0, "and none move before it");
    }
}

#[test]
fn the_plunge_clock_runs_between_events() {
    let mut motion = StackMotion::default();
    assert_eq!(motion.plunge, None, "it starts idle");
    // Nothing happens until the interval is up.
    assert_eq!(motion.advance(config::STACK_PLUNGE_INTERVAL * 0.5), None);
    // Then the event runs for its own duration.
    assert_eq!(
        motion.advance(config::STACK_PLUNGE_INTERVAL * 0.5),
        Some(0.0)
    );
    let progress = motion
        .advance(config::STACK_PLUNGE_SECONDS * 0.5)
        .expect("still plunging");
    assert!((progress - 0.5).abs() < 0.01, "progress was {progress}");
    // And it ends, resetting the interval.
    assert_eq!(motion.advance(config::STACK_PLUNGE_SECONDS), None);
    assert_eq!(motion.timer, config::STACK_PLUNGE_INTERVAL);
}

#[test]
fn a_stack_frame_is_an_open_outline() {
    let frame = stack_frame_mesh(20.0, 12.0);
    let single = Mesh::from(Cuboid::default());
    let triangles = |mesh: &Mesh| mesh.indices().map(|i| i.len()).unwrap_or(0);
    assert_eq!(
        frame.count_vertices(),
        single.count_vertices() * 4,
        "four bars make the outline"
    );
    assert_eq!(
        triangles(&frame),
        triangles(&single) * 4,
        "and nothing fills the middle"
    );
}

#[test]
fn stack_frames_glide_within_their_radius_and_ripple() {
    let radius = config::STACK_FRAME_GLIDE;
    let mut moved = false;
    for step in 0..400 {
        let t = step as f32 * 0.05;
        let (x, z) = stack_frame_glide(0, t);
        assert!(
            x.abs() <= radius + 0.001 && z.abs() <= radius + 0.001,
            "glide {x},{z} left its radius"
        );
        moved |= x.abs() > radius * 0.5 || z.abs() > radius * 0.5;
    }
    assert!(moved, "the frame should actually drift");
    assert!(
        stack_frame_glide(0, 1.0) != stack_frame_glide(1, 1.0),
        "neighbouring frames should not glide in lockstep"
    );
}

#[test]
fn stack_frames_hover_within_their_travel_and_ripple() {
    let peak = config::STACK_FRAME_HOVER;
    let mut seen_low = false;
    let mut seen_high = false;
    for step in 0..200 {
        let t = step as f32 * 0.05;
        let hover = stack_frame_hover(0, t);
        assert!(hover.abs() <= peak + 0.001, "hover {hover} left its travel");
        seen_low |= hover < -peak * 0.5;
        seen_high |= hover > peak * 0.5;
    }
    assert!(seen_low && seen_high, "the plate should actually travel");

    // Levels are out of phase, so the stack ripples rather than sliding as
    // one block.
    assert!(
        (stack_frame_hover(0, 0.0) - stack_frame_hover(1, 0.0)).abs() > 0.01,
        "neighbouring plates should not move in lockstep"
    );
}

#[test]
fn stack_plates_rock_within_their_limit() {
    let limit = config::STACK_FRAME_ROCK;
    for step in 0..200 {
        let t = step as f32 * 0.05;
        let (x, z) = stack_frame_rock(2, t);
        assert!(x.abs() <= limit + 0.001, "rock x {x} left its limit");
        assert!(z.abs() <= limit + 0.001, "rock z {z} left its limit");
    }
}
