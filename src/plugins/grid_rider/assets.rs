//! Materials and unit meshes shared by every grid rider arena.

use super::character::vision_cone_mesh;
use super::city::city_palette;
use super::trail::trail_glass_material;
use crate::config;
use crate::filesystem::language::SourceLanguage;
use crate::grid_rider::scene::GridRiderAssets;
use bevy::prelude::*;

pub(crate) fn unlit_material(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        unlit: true,
        ..default()
    }
}

pub(crate) fn neon_material(color: Color, emissive: LinearRgba) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        emissive,
        unlit: true,
        ..default()
    }
}

pub(crate) fn setup_grid_rider_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let palette = city_palette();
    let city_accent_materials = std::array::from_fn(|theme| {
        std::array::from_fn(|accent| {
            let (color, emissive) = palette[theme][accent];
            materials.add(neon_material(color, emissive))
        })
    });
    commands.insert_resource(GridRiderAssets {
        unit_cube: meshes.add(Cuboid::default()),
        entry_beam_mesh: meshes.add(Cylinder::new(
            config::grid_rider::GRID_RIDER_ENTRY_BEAM_RADIUS,
            config::grid_rider::GRID_RIDER_ENTRY_BEAM_HEIGHT,
        )),
        entry_halo_mesh: meshes.add(Torus::new(
            config::grid_rider::GRID_RIDER_ENTRY_HALO_INNER_RADIUS,
            config::grid_rider::GRID_RIDER_ENTRY_HALO_OUTER_RADIUS,
        )),
        disc_mesh: meshes.add(Cylinder::new(
            config::disc::DISC_MESH_RADIUS,
            config::disc::DISC_MESH_THICKNESS,
        )),
        sentinel_mesh: meshes.add(Cylinder::new(
            config::disc::SENTINEL_RADIUS,
            config::disc::SENTINEL_HEIGHT,
        )),
        rock_material: materials.add(StandardMaterial {
            base_color: config::asteroid_field::ASTEROID_FIELD_ROCK_COLOR,
            emissive: LinearRgba::from(config::asteroid_field::ASTEROID_FIELD_ROCK_CORE_COLOR) * 0.3,
            perceptual_roughness: 0.92,
            ..default()
        }),
        beam_material: materials.add(StandardMaterial {
            base_color: config::asteroid_field::ASTEROID_FIELD_BEAM_COLOR,
            emissive: LinearRgba::from(config::asteroid_field::ASTEROID_FIELD_BEAM_COLOR) * 3.4,
            unlit: true,
            alpha_mode: AlphaMode::Add,
            ..default()
        }),
        snake_food_material: materials.add(StandardMaterial {
            base_color: config::snake::SNAKE_FOOD_COLOR,
            emissive: LinearRgba::from(config::snake::SNAKE_FOOD_COLOR) * 2.2,
            unlit: true,
            ..default()
        }),
        snake_lock_material: materials.add(StandardMaterial {
            base_color: config::snake::SNAKE_GATE_COLOR,
            emissive: LinearRgba::from(config::snake::SNAKE_GATE_COLOR) * 1.6,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        runner_scene: asset_server
            .load(GltfAssetLabel::Scene(0).from_asset(config::character::RUNNER_MODEL_ASSET)),
        runner_gltf: asset_server.load(config::character::RUNNER_MODEL_ASSET),
        platform_material: materials.add(unlit_material(
            config::platformer::PLATFORMER_PLATFORM_COLOR,
        )),
        exit_material: materials.add(StandardMaterial {
            base_color: config::platformer::PLATFORMER_EXIT_COLOR,
            emissive: LinearRgba::from(config::platformer::PLATFORMER_EXIT_COLOR) * 2.4,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        brick_material: materials.add(StandardMaterial {
            base_color: config::breaker::BREAKER_BRICK_COLOR,
            emissive: LinearRgba::from(config::breaker::BREAKER_BRICK_COLOR) * 1.5,
            unlit: true,
            ..default()
        }),
        ball_material: materials.add(StandardMaterial {
            base_color: config::breaker::BREAKER_BALL_COLOR,
            emissive: LinearRgba::from(config::breaker::BREAKER_BALL_COLOR) * 3.0,
            unlit: true,
            ..default()
        }),
        court_material: materials.add(unlit_material(config::breaker::BREAKER_WALL_COLOR)),
        stealth_floor_material: materials.add(unlit_material(config::stealth::STEALTH_FLOOR_COLOR)),
        stealth_wall_material: materials.add(StandardMaterial {
            base_color: config::stealth::STEALTH_WALL_COLOR,
            emissive: LinearRgba::from(config::stealth::STEALTH_WALL_COLOR) * 0.6,
            unlit: true,
            ..default()
        }),
        stealth_cone_material: materials.add(StandardMaterial {
            base_color: config::stealth::STEALTH_CONE_COLOR,
            emissive: LinearRgba::from(config::stealth::STEALTH_CONE_COLOR) * 1.4,
            unlit: true,
            alpha_mode: AlphaMode::Add,
            ..default()
        }),
        stealth_exit_material: materials.add(StandardMaterial {
            base_color: config::stealth::STEALTH_EXIT_COLOR,
            emissive: LinearRgba::from(config::stealth::STEALTH_EXIT_COLOR) * 2.4,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        vision_cone: meshes.add(vision_cone_mesh(
            config::stealth::STEALTH_VISION_HALF_ANGLE,
            config::stealth::STEALTH_CONE_SEGMENTS,
        )),
        surfer_water_material: materials.add(StandardMaterial {
            base_color: config::surfer::SURFER_WATER_COLOR,
            emissive: LinearRgba::from(config::surfer::SURFER_WATER_COLOR) * 0.2,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        surfer_rock_material: materials.add(unlit_material(config::surfer::SURFER_ROCK_COLOR)),
        surfer_gate_material: materials.add(StandardMaterial {
            base_color: config::surfer::SURFER_GATE_COLOR,
            emissive: LinearRgba::from(config::surfer::SURFER_GATE_COLOR) * 2.2,
            unlit: true,
            ..default()
        }),
        surfer_finish_material: materials.add(StandardMaterial {
            base_color: config::surfer::SURFER_FINISH_COLOR,
            emissive: LinearRgba::from(config::surfer::SURFER_FINISH_COLOR) * 2.4,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        swarm_bug_material: materials.add(StandardMaterial {
            base_color: config::swarm_shooter::SWARM_SHOOTER_BUG_COLOR,
            emissive: LinearRgba::from(config::swarm_shooter::SWARM_SHOOTER_BUG_COLOR) * 2.0,
            unlit: true,
            ..default()
        }),
        swarm_beam_material: materials.add(StandardMaterial {
            base_color: config::swarm_shooter::SWARM_SHOOTER_BEAM_COLOR,
            emissive: LinearRgba::from(config::swarm_shooter::SWARM_SHOOTER_BEAM_COLOR) * 3.0,
            unlit: true,
            ..default()
        }),
        gem_materials: std::array::from_fn(|index| {
            materials.add(StandardMaterial {
                base_color: config::arcade::FALLING_GEMS_COLORS_LIST[index],
                emissive: LinearRgba::from(config::arcade::FALLING_GEMS_COLORS_LIST[index]) * 1.6,
                unlit: true,
                ..default()
            })
        }),
        block_fall_materials: std::array::from_fn(|index| {
            materials.add(StandardMaterial {
                base_color: config::arcade::BLOCK_FALL_COLORS[index],
                emissive: LinearRgba::from(config::arcade::BLOCK_FALL_COLORS[index]) * 1.4,
                unlit: true,
                ..default()
            })
        }),
        cube_dim: materials.add(unlit_material(config::arcade::CUBE_HOPPER_CUBE_DIM_COLOR)),
        cube_lit: materials.add(StandardMaterial {
            base_color: config::arcade::CUBE_HOPPER_CUBE_LIT_COLOR,
            emissive: LinearRgba::from(config::arcade::CUBE_HOPPER_CUBE_LIT_COLOR) * 1.8,
            unlit: true,
            ..default()
        }),
        cube_enemy_material: materials.add(StandardMaterial {
            base_color: config::arcade::CUBE_HOPPER_ENEMY_COLOR,
            emissive: LinearRgba::from(config::arcade::CUBE_HOPPER_ENEMY_COLOR) * 2.0,
            unlit: true,
            ..default()
        }),
        plinko_pin_material: materials.add(unlit_material(config::arcade::PLINKO_PIN_COLOR)),
        plinko_ball_material: materials.add(StandardMaterial {
            base_color: config::arcade::PLINKO_BALL_COLOR,
            emissive: LinearRgba::from(config::arcade::PLINKO_BALL_COLOR) * 1.8,
            unlit: true,
            ..default()
        }),
        grid_bomber_crate_material: materials.add(unlit_material(config::arcade::GRID_BOMBER_CRATE_COLOR)),
        grid_bomber_bomb_material: materials.add(unlit_material(config::arcade::GRID_BOMBER_BOMB_COLOR)),
        cycle_scene: asset_server
            .load(GltfAssetLabel::Scene(0).from_asset(config::grid_rider::GRID_RIDER_MODEL_ASSET)),
        trail_material: materials.add(trail_glass_material()),
        flood_material: materials.add(StandardMaterial {
            base_color: config::grid_rider::FLOOD_COLOR,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            ..default()
        }),
        flood_crest_material: materials.add(StandardMaterial {
            base_color: config::grid_rider::FLOOD_CREST_COLOR,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            ..default()
        }),
        gc_sweep_material: materials.add(StandardMaterial {
            base_color: config::grid_rider::GC_SWEEP_COLOR,
            emissive: LinearRgba::from(config::grid_rider::GC_SWEEP_COLOR) * 2.6,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            ..default()
        }),
        wall_material: materials.add(unlit_material(config::grid_rider::GRID_RIDER_WALL_COLOR)),
        city_floor_material: materials.add(StandardMaterial {
            base_color: config::grid_rider::GRID_RIDER_CITY_FLOOR_COLOR,
            unlit: true,
            ..default()
        }),
        city_foundation_material: materials.add(StandardMaterial {
            base_color: config::grid_rider::GRID_RIDER_CITY_FOUNDATION_COLOR,
            unlit: true,
            ..default()
        }),
        city_glass_material: materials.add(StandardMaterial {
            base_color: config::grid_rider::GRID_RIDER_CITY_GLASS_COLOR,
            emissive: LinearRgba::rgb(0.02, 0.12, 0.25),
            metallic: 0.18,
            perceptual_roughness: 0.08,
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        city_accent_materials,
        portal_material: materials.add(unlit_material(config::grid_rider::GRID_RIDER_PORTAL_COLOR)),
        portal_bar_material: materials.add(StandardMaterial {
            base_color: config::grid_rider::GRID_RIDER_PORTAL_COLOR
                .with_alpha(config::grid_rider::GRID_RIDER_PORTAL_BAR_ALPHA),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        dir_tower_material: materials.add(unlit_material(config::DIR_COLOR)),
        file_tower_material: materials.add(unlit_material(config::FILE_COLOR)),
        markdown_tower_material: materials
            .add(unlit_material(config::document::MARKDOWN_TOWER_COLOR)),
        source_tower_material: materials.add(neon_material(
            config::SOURCE_TOWER_COLOR,
            LinearRgba::rgb(2.2, 0.9, 0.05),
        )),
        document_floor_material: materials
            .add(unlit_material(config::document::DOCUMENT_FLOOR_COLOR)),
        document_rule_material: materials
            .add(unlit_material(config::document::DOCUMENT_RULE_COLOR)),
        document_margin_material: materials
            .add(unlit_material(config::document::DOCUMENT_MARGIN_COLOR)),
        document_ink_material: materials.add(neon_material(
            config::document::DOCUMENT_INK_COLOR,
            LinearRgba::from(config::document::DOCUMENT_INK_EMISSIVE),
        )),
        document_heading_material: materials.add(neon_material(
            config::document::DOCUMENT_HEADING_COLOR,
            LinearRgba::rgb(0.55, 0.28, 0.08),
        )),
        document_folio_material: materials
            .add(unlit_material(config::document::DOCUMENT_FOLIO_COLOR)),
        document_focus_material: materials.add(neon_material(
            config::document::DOCUMENT_FOCUS_COLOR,
            LinearRgba::rgb(2.4, 0.9, 0.1),
        )),
        crash_material: materials.add(unlit_material(Color::srgb(1.0, 0.45, 0.1))),
        entry_beam_material: materials.add(StandardMaterial {
            base_color: Color::srgba(0.25, 0.92, 1.0, 0.14),
            emissive: LinearRgba::rgb(0.08, 1.8, 2.8),
            alpha_mode: AlphaMode::Add,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        entry_halo_material: materials.add(StandardMaterial {
            base_color: Color::srgba(0.7, 0.98, 1.0, 0.92),
            emissive: LinearRgba::rgb(3.2, 5.0, 6.0),
            alpha_mode: AlphaMode::Add,
            unlit: true,
            ..default()
        }),
        disc_floor_material: materials.add(unlit_material(config::disc::DISC_FLOOR_COLOR)),
        disc_ring_material: materials.add(neon_material(
            config::disc::DISC_RING_COLOR,
            LinearRgba::rgb(0.05, 0.4, 0.62),
        )),
        disc_plinth_material: materials.add(unlit_material(config::disc::DISC_RING_COLOR)),
        disc_hazard_material: materials.add(neon_material(
            config::disc::DISC_HAZARD_COLOR,
            LinearRgba::rgb(2.4, 0.2, 0.1),
        )),
        disc_opponent_material: materials.add(neon_material(
            config::disc::DISC_OPPONENT_COLOR,
            LinearRgba::rgb(2.4, 1.1, 0.1),
        )),
        disc_player_disc_material: materials.add(neon_material(
            config::disc::DISC_PLAYER_DISC_COLOR,
            LinearRgba::rgb(0.6, 2.6, 3.0),
        )),
        disc_pickup_material: materials.add(neon_material(
            config::disc::DISC_PICKUP_COLOR,
            LinearRgba::rgb(2.4, 2.1, 0.3),
        )),
        disc_safe_pad_material: materials.add(neon_material(
            config::disc::DISC_SAFE_PAD_COLOR,
            LinearRgba::rgb(0.1, 1.4, 0.5),
        )),
        disc_accent_materials: std::array::from_fn(|index| {
            let accent = SourceLanguage::ALL[index].accent();
            materials.add(neon_material(accent, LinearRgba::from(accent)))
        }),
    });
}
