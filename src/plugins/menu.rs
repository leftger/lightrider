//! Badass TRON-style main menu plugin with 3D lightcycle diorama, glowing trail,
//! dynamic camera motion, and interactive cyber-terminal UI with live configuration.

use crate::config;
use crate::lightcycle::scene::LightcycleAssets;
use crate::load::DirectoryRequested;
use crate::plugins::lightcycle::decor::decorate_directory_run;
use crate::plugins::lightcycle::run::build_active_run;
use crate::plugins::lightcycle::run::spawn_run_entities;
use crate::plugins::music::MusicState;
use crate::state::{
    CacheState, FloodState, HistoryState, InteractionMode, MainMenuSceneRoot, NavigatorResource,
    OrbitCameraResource, RenderSettings, UiSettings,
};
use bevy::app::AppExit;
use bevy::ecs::system::SystemParam;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuScreen>()
            .init_resource::<MenuNavigationState>()
            .add_systems(
                Startup,
                (
                    setup_main_menu_scene
                        .after(crate::plugins::lightcycle::assets::setup_lightcycle_assets),
                    setup_main_menu_ui,
                ),
            )
            .add_systems(
                Update,
                (
                    animate_main_menu_scene,
                    handle_menu_input,
                    update_menu_display,
                    sync_menu_visibility,
                ),
            );
    }
}

/// Active screen within the main menu.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuScreen {
    #[default]
    Main,
    Settings,
    Controls,
}

/// Navigation cursor state and menu timer.
#[derive(Resource, Default)]
pub struct MenuNavigationState {
    pub selected_index: usize,
    pub time_in_menu: f32,
    pub last_screen: MenuScreen,
}

/// Marker for the animated showcase lightcycle in the 3D diorama.
#[derive(Component)]
struct MainMenuBike {
    base_y: f32,
}

/// Marker for the glowing ribbon trail in the 3D diorama.
#[derive(Component)]
struct MainMenuTrail;

/// Root container for all main menu UI elements.
#[derive(Component)]
struct MainMenuUiRoot;

/// Marker for the dynamic menu items container.
#[derive(Component)]
struct MenuItemsList;

/// Component attached to selectable menu button rows.
#[derive(Component)]
struct MenuButtonAction {
    index: usize,
    action: MenuAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum MenuAction {
    // Main screen
    RideTheGrid,
    ExploreDirectory,
    OpenSettings,
    OpenControls,
    ExitProgram,

    // Settings screen
    ToggleMsaa,
    ToggleBloom,
    ToggleScanlines,
    ToggleVignette,
    ToggleMusic,
    VolumeDown,
    VolumeUp,
    ToggleLabels,
    ToggleFps,
    BackToMain,
}

// Neon cyber palette
const NEON_CYAN: Color = Color::srgb(0.0, 0.95, 1.0);
const NEON_MAGENTA: Color = Color::srgb(1.0, 0.08, 0.65);
const NEON_ORANGE: Color = Color::srgb(1.0, 0.45, 0.0);
const NEON_GREEN: Color = Color::srgb(0.05, 1.0, 0.4);
const CYBER_PANEL_BG: Color = Color::srgba(0.02, 0.04, 0.08, 0.88);
const CYBER_BORDER_COLOR: Color = Color::srgba(0.0, 0.95, 1.0, 0.6);
const CYBER_BUTTON_NORMAL: Color = Color::srgba(0.04, 0.08, 0.14, 0.75);
const CYBER_BUTTON_SELECTED: Color = Color::srgba(0.0, 0.35, 0.45, 0.85);

/// Spawns the 3D showcase diorama (the lightcycle, its curved luminous trail,
/// cyber-grid floor, data pillars, and colored stage lighting).
fn setup_main_menu_scene(
    mut commands: Commands,
    assets: Option<Res<LightcycleAssets>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(assets) = assets else {
        return;
    };
    // 1. Showcase Lightcycle
    commands.spawn((
        MainMenuSceneRoot,
        MainMenuBike { base_y: 0.3 },
        Transform::from_xyz(0.0, 0.3, 0.0)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_4)),
        Visibility::Inherited,
        children![(
            WorldAssetRoot(assets.cycle_scene.clone()),
            Transform::from_rotation(Quat::from_rotation_y(
                config::lightcycle::LIGHTCYCLE_MODEL_YAW,
            ))
            .with_scale(Vec3::splat(
                config::lightcycle::LIGHTCYCLE_MODEL_SCALE * 1.35
            )),
        )],
    ));

    // 2. High-intensity twin headlights
    commands.spawn((
        MainMenuSceneRoot,
        PointLight {
            color: NEON_CYAN,
            intensity: 22_000.0,
            range: 24.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(2.2, 0.8, 2.2),
        Visibility::Inherited,
    ));

    // 3. Hot magenta rim spotlight from the rear
    commands.spawn((
        MainMenuSceneRoot,
        PointLight {
            color: NEON_MAGENTA,
            intensity: 18_000.0,
            range: 22.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-3.5, 2.5, -3.5),
        Visibility::Inherited,
    ));

    // 4. Subtle overhead ambient cyber fill
    commands.spawn((
        MainMenuSceneRoot,
        PointLight {
            color: Color::srgb(0.2, 0.4, 0.9),
            intensity: 12_000.0,
            range: 30.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 6.0, 0.0),
        Visibility::Inherited,
    ));

    // 5. Dynamic S-curving neon glass lightcycle trail
    let trail_mesh = generate_curved_trail_mesh();
    commands.spawn((
        MainMenuSceneRoot,
        MainMenuTrail,
        Mesh3d(meshes.add(trail_mesh)),
        MeshMaterial3d(assets.trail_material.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::Inherited,
    ));

    // Also spawn a bright emissive inner core for the trail so it glows with bloom
    let core_material = materials.add(StandardMaterial {
        base_color: NEON_CYAN,
        emissive: LinearRgba::from(NEON_CYAN) * 4.5,
        unlit: true,
        alpha_mode: AlphaMode::Add,
        ..default()
    });
    let core_mesh = generate_curved_trail_core_mesh();
    commands.spawn((
        MainMenuSceneRoot,
        Mesh3d(meshes.add(core_mesh)),
        MeshMaterial3d(core_material),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::Inherited,
    ));

    // 6. Cyber Grid Floor Lines
    let grid_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.9, 0.8, 0.35),
        emissive: LinearRgba::from(Color::srgb(0.0, 0.9, 0.8)) * 0.8,
        unlit: true,
        ..default()
    });

    let grid_extent = 40.0;
    let grid_spacing = 2.5;
    let num_lines = (grid_extent / grid_spacing) as i32;

    for i in -num_lines..=num_lines {
        let pos = i as f32 * grid_spacing;
        // Lines along X
        commands.spawn((
            MainMenuSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(grid_mat.clone()),
            Transform::from_translation(Vec3::new(0.0, 0.01, pos)).with_scale(Vec3::new(
                grid_extent * 2.0,
                0.02,
                0.03,
            )),
            Visibility::Inherited,
        ));
        // Lines along Z
        commands.spawn((
            MainMenuSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(grid_mat.clone()),
            Transform::from_translation(Vec3::new(pos, 0.01, 0.0)).with_scale(Vec3::new(
                0.03,
                0.02,
                grid_extent * 2.0,
            )),
            Visibility::Inherited,
        ));
    }

    // 7. Distant Cyber Monoliths / Towers flanking the grid horizon
    let monolith_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.01, 0.02, 0.05),
        perceptual_roughness: 0.2,
        metallic: 0.8,
        ..default()
    });
    let monolith_glow = materials.add(StandardMaterial {
        base_color: NEON_MAGENTA,
        emissive: LinearRgba::from(NEON_MAGENTA) * 2.5,
        unlit: true,
        ..default()
    });

    let tower_positions = [
        (-18.0, -15.0, 14.0),
        (-22.0, -5.0, 20.0),
        (-16.0, 12.0, 16.0),
        (18.0, -18.0, 22.0),
        (22.0, 2.0, 18.0),
        (15.0, 16.0, 12.0),
        (-10.0, -26.0, 25.0),
        (12.0, -28.0, 28.0),
    ];

    for &(x, z, height) in &tower_positions {
        commands.spawn((
            MainMenuSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(monolith_mat.clone()),
            Transform::from_xyz(x, height * 0.5, z).with_scale(Vec3::new(3.5, height, 3.5)),
            Visibility::Inherited,
        ));
        // Vertical neon accent strip on the face of the monolith
        commands.spawn((
            MainMenuSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(monolith_glow.clone()),
            Transform::from_xyz(x, height * 0.5, z + 1.8).with_scale(Vec3::new(0.15, height, 0.05)),
            Visibility::Inherited,
        ));
    }
}

/// Generates a smooth sweeping S-curve ribbon mesh for the lightcycle's glowing trail.
fn generate_curved_trail_mesh() -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let segments = 50;
    let height = 0.95;

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let angle = t * std::f32::consts::PI * 1.8;
        let dist = t * 24.0;
        let x = -(dist * 0.707) + (angle).sin() * 2.8;
        let z = -(dist * 0.707) - (angle).cos() * 1.2;

        positions.push([x, 0.02, z]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([t, 0.0]);

        positions.push([x, height, z]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([t, 1.0]);

        if i < segments {
            let base = (i * 2) as u32;
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);

            indices.push(base + 2);
            indices.push(base + 1);
            indices.push(base + 3);

            // Double sided
            indices.push(base + 2);
            indices.push(base + 1);
            indices.push(base);

            indices.push(base + 3);
            indices.push(base + 1);
            indices.push(base + 2);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Thin high-intensity top ridge for the trail that catches the camera with intense bloom.
fn generate_curved_trail_core_mesh() -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let segments = 50;
    let top_y = 0.95;
    let thickness = 0.06;

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let angle = t * std::f32::consts::PI * 1.8;
        let dist = t * 24.0;
        let x = -(dist * 0.707) + (angle).sin() * 2.8;
        let z = -(dist * 0.707) - (angle).cos() * 1.2;

        positions.push([x, top_y - thickness, z]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([t, 0.0]);

        positions.push([x, top_y, z]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([t, 1.0]);

        if i < segments {
            let base = (i * 2) as u32;
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);

            indices.push(base + 2);
            indices.push(base + 1);
            indices.push(base + 3);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Animates the lightcycle's floating bob, roll tilt, slow rotation, and the cinematic camera.
fn animate_main_menu_scene(
    time: Res<Time>,
    mode: Res<InteractionMode>,
    mut nav_state: ResMut<MenuNavigationState>,
    mut bike_query: Query<(&MainMenuBike, &mut Transform)>,
    mut orbit: ResMut<OrbitCameraResource>,
) {
    if *mode != InteractionMode::MainMenu {
        return;
    }

    let dt = time.delta_secs();
    nav_state.time_in_menu += dt;
    let t = nav_state.time_in_menu;

    // Hover bob & tilt
    for (bike, mut transform) in &mut bike_query {
        let hover_y = bike.base_y + (t * 2.0).sin() * 0.06;
        let roll = (t * 1.5).cos() * 0.035;
        let pitch = (t * 1.2).sin() * 0.015;
        let yaw = std::f32::consts::FRAC_PI_4 + (t * 0.15);

        transform.translation.y = hover_y;
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }

    // Cinematic orbiting camera
    orbit.target = Vec3::new(0.0, 0.85, 0.0);
    orbit.distance = 9.5;
    orbit.yaw = config::DEFAULT_CAMERA_YAW + (t * 0.08);
    orbit.pitch = 0.32 + (t * 0.35).sin() * 0.05;
}

/// Builds the static outer shell for the Cyberpunk UI.
fn setup_main_menu_ui(mut commands: Commands, navigator: Res<NavigatorResource>) {
    let mount_label = navigator
        .0
        .current_path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "/".to_string());

    commands
        .spawn((
            MainMenuUiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(40.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            // Header: Big stylized cyberpunk title + telemetry
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|header| {
                header.spawn((
                    Text::new(">>> DIGITAL SYSTEM READY // CYBERNETIC GRID PROTOCOL <<<"),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(12.0),
                        ..default()
                    },
                    TextColor(NEON_ORANGE),
                ));

                header.spawn((
                    Text::new("LIGHTRIDER"),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(46.0),
                        ..default()
                    },
                    TextColor(NEON_CYAN),
                ));

                header.spawn((
                    Text::new("3D RECONNAISSANCE FILE EXPLORER & HIGH-VELOCITY LIGHTCYCLE"),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(14.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.7, 0.85, 0.95)),
                ));

                header.spawn((
                    Text::new(format!("MOUNT: [ {} ]", mount_label)),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(NEON_GREEN),
                ));
            });

            // Center: Dynamic items container (rebuilt on screen change)
            root.spawn((
                MenuItemsList,
                Node {
                    flex_direction: FlexDirection::Column,
                    min_width: Val::Px(460.0),
                    max_width: Val::Px(640.0),
                    padding: UiRect::all(Val::Px(24.0)),
                    border: UiRect::all(Val::Px(1.5)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(CYBER_PANEL_BG),
                BorderColor::all(CYBER_BORDER_COLOR),
            ));

            // Footer: Control hints
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|footer| {
                footer.spawn((
                    Text::new("NAV: ↑/↓/W/S  |  SELECT: ENTER/SPACE/CLICK  |  ADJUST: ←/→/A/D  |  ESC: BACK"),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(12.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.6, 0.7, 0.8)),
                ));
            });
        });
}

/// Synchronizes the visibility of the main menu UI and 3D diorama entities.
fn sync_menu_visibility(
    mode: Res<InteractionMode>,
    mut ui_roots: Query<&mut Node, With<MainMenuUiRoot>>,
    mut scene_roots: Query<&mut Visibility, With<MainMenuSceneRoot>>,
) {
    if !mode.is_changed() {
        return;
    }
    let in_menu = *mode == InteractionMode::MainMenu;

    for mut node in &mut ui_roots {
        node.display = if in_menu {
            Display::Flex
        } else {
            Display::None
        };
    }

    let scene_vis = if in_menu {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut vis in &mut scene_roots {
        if *vis != scene_vis {
            *vis = scene_vis;
        }
    }
}

/// Rebuilds or updates the menu items when screen or state changes.
#[allow(clippy::too_many_arguments)]
fn update_menu_display(
    mut commands: Commands,
    screen: Res<MenuScreen>,
    mut nav_state: ResMut<MenuNavigationState>,
    container: Query<Entity, With<MenuItemsList>>,
    old_buttons: Query<Entity, With<MenuButtonAction>>,
    mut buttons: Query<(
        Entity,
        &MenuButtonAction,
        &mut BackgroundColor,
        &mut BorderColor,
        &Children,
    )>,
    mut texts: Query<(&mut Text, &mut TextColor)>,
    render_settings: Res<RenderSettings>,
    ui_settings: Res<UiSettings>,
    music_state: Res<MusicState>,
) {
    let Ok(container_entity) = container.single() else {
        return;
    };

    // If screen changed, clear old buttons and spawn new items
    if *screen != nav_state.last_screen {
        for entity in &old_buttons {
            commands.entity(entity).despawn();
        }
        nav_state.last_screen = *screen;
        nav_state.selected_index = 0;

        match *screen {
            MenuScreen::Main => spawn_main_screen_items(&mut commands, container_entity),
            MenuScreen::Settings => spawn_settings_screen_items(
                &mut commands,
                container_entity,
                &render_settings,
                &ui_settings,
                &music_state,
            ),
            MenuScreen::Controls => spawn_controls_screen_items(&mut commands, container_entity),
        }
        return;
    }

    // Refresh styling and labels based on selection and current settings
    for (_entity, action, mut bg, mut border, children) in &mut buttons {
        let is_selected = action.index == nav_state.selected_index;
        *bg = if is_selected {
            CYBER_BUTTON_SELECTED.into()
        } else {
            CYBER_BUTTON_NORMAL.into()
        };
        *border = if is_selected {
            BorderColor::all(NEON_CYAN)
        } else {
            BorderColor::all(Color::srgba(0.0, 0.8, 0.9, 0.2))
        };

        if let Some(&text_child) = children.first()
            && let Ok((mut text, mut color)) = texts.get_mut(text_child)
        {
            let label = match action.action {
                MenuAction::RideTheGrid => "RIDE THE GRID (LIGHTCYCLE)".to_string(),
                MenuAction::ExploreDirectory => "EXPLORE DIRECTORY (3D TREE)".to_string(),
                MenuAction::OpenSettings => "SYSTEM CONFIGURATION".to_string(),
                MenuAction::OpenControls => "FLIGHT MANUAL & CONTROLS".to_string(),
                MenuAction::ExitProgram => "TERMINATE PROGRAM".to_string(),

                MenuAction::ToggleMsaa => format!(
                    "MSAA ANTI-ALIASING: [ {} ]",
                    match render_settings.msaa {
                        1 => "OFF (1x)",
                        2 => "2x",
                        4 => "4x",
                        8 => "8x",
                        n =>
                            if n == 0 {
                                "OFF"
                            } else {
                                "CUSTOM"
                            },
                    }
                ),
                MenuAction::ToggleBloom => format!(
                    "HDR BLOOM GLOW: [ {} ]",
                    if render_settings.bloom {
                        "ENABLED"
                    } else {
                        "DISABLED"
                    }
                ),
                MenuAction::ToggleScanlines => format!(
                    "CRT SCANLINES: [ {} ]",
                    if render_settings.scanlines {
                        "ENABLED"
                    } else {
                        "DISABLED"
                    }
                ),
                MenuAction::ToggleVignette => format!(
                    "VIGNETTE POST-PROCESS: [ {} ]",
                    if render_settings.vignette {
                        "ENABLED"
                    } else {
                        "DISABLED"
                    }
                ),
                MenuAction::ToggleMusic => format!(
                    "PROCEDURAL SYNTH MUSIC: [ {} ]",
                    if music_state.enabled {
                        "ENABLED"
                    } else {
                        "DISABLED"
                    }
                ),
                MenuAction::VolumeDown | MenuAction::VolumeUp => {
                    format!("AUDIO VOLUME: [ < {:.0}% > ]", music_state.volume * 100.0)
                }
                MenuAction::ToggleLabels => format!(
                    "FILE LABELS: [ {} ]",
                    if ui_settings.show_labels {
                        "ENABLED"
                    } else {
                        "DISABLED"
                    }
                ),
                MenuAction::ToggleFps => format!(
                    "FPS COUNTER: [ {} ]",
                    if ui_settings.show_fps {
                        "ENABLED"
                    } else {
                        "DISABLED"
                    }
                ),
                MenuAction::BackToMain => "< BACK TO MAIN MENU".to_string(),
            };

            let prefix = if is_selected { "> " } else { "  " };
            let suffix = if is_selected { " <" } else { "" };
            let line = format!("{prefix}{label}{suffix}");
            if **text != line {
                **text = line;
            }
            color.0 = if is_selected {
                NEON_CYAN
            } else {
                Color::srgb(0.85, 0.9, 0.95)
            };
        }
    }
}

fn spawn_main_screen_items(commands: &mut Commands, container: Entity) {
    let items = [
        ("RIDE THE GRID (LIGHTCYCLE)", MenuAction::RideTheGrid),
        ("EXPLORE DIRECTORY (3D TREE)", MenuAction::ExploreDirectory),
        ("SYSTEM CONFIGURATION", MenuAction::OpenSettings),
        ("FLIGHT MANUAL & CONTROLS", MenuAction::OpenControls),
        ("TERMINATE PROGRAM", MenuAction::ExitProgram),
    ];

    commands.entity(container).with_children(|parent| {
        for (index, (label, action)) in items.iter().enumerate() {
            spawn_menu_button(parent, index, *action, label);
        }
    });
}

fn spawn_settings_screen_items(
    commands: &mut Commands,
    container: Entity,
    _render: &RenderSettings,
    _ui: &UiSettings,
    _music: &MusicState,
) {
    let items = [
        ("MSAA ANTI-ALIASING", MenuAction::ToggleMsaa),
        ("HDR BLOOM GLOW", MenuAction::ToggleBloom),
        ("CRT SCANLINES", MenuAction::ToggleScanlines),
        ("VIGNETTE POST-PROCESS", MenuAction::ToggleVignette),
        ("PROCEDURAL SYNTH MUSIC", MenuAction::ToggleMusic),
        ("AUDIO VOLUME", MenuAction::VolumeUp),
        ("FILE LABELS", MenuAction::ToggleLabels),
        ("FPS COUNTER", MenuAction::ToggleFps),
        ("< BACK TO MAIN MENU", MenuAction::BackToMain),
    ];

    commands.entity(container).with_children(|parent| {
        for (index, (label, action)) in items.iter().enumerate() {
            spawn_menu_button(parent, index, *action, label);
        }
    });
}

fn spawn_controls_screen_items(commands: &mut Commands, container: Entity) {
    commands.entity(container).with_children(|parent| {
        parent.spawn((
            Text::new(
                "--- LIGHTCYCLE FLIGHT MANUAL ---\n\
                 STEERING      : WASD / ARROW KEYS\n\
                 TURBO BOOST   : SPACEBAR\n\
                 DRIFT / LEAN  : Q / E\n\
                 BRAKE/REVERSE : S / DOWN\n\
                 PAUSE / WARP  : ESC / P\n\
                 MODE SWAP     : M (Switch to 3D Explorer)\n\n\
                 --- 3D FILE EXPLORER CONTROLS ---\n\
                 NAVIGATE      : HJKL / ARROWS / MOUSE DRAG\n\
                 ENTER DIR     : ENTER / O / DOUBLE-CLICK\n\
                 PARENT DIR    : U / - / CLICK BREADCRUMB\n\
                 FILE LABELS   : TAB\n\
                 MAIN MENU     : ESCAPE\n\n\
                 --- CYBER ARCADE MINIGAMES ---\n\
                 Source code files manifest interactive battlefields:\n\
                 Disc Wars, Asteroids, Snake, Platformer, Breaker,\n\
                 Stealth, Surfer, Galaga, Pacman, Columns, Tetris,\n\
                 Frogger, Qbert, Bomberman, Plinko.",
            ),
            TextFont {
                font_size: bevy::text::FontSize::Px(13.0),
                ..default()
            },
            TextColor(Color::srgb(0.75, 0.9, 1.0)),
            Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
        ));

        spawn_menu_button(parent, 0, MenuAction::BackToMain, "< BACK TO MAIN MENU");
    });
}

fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    action: MenuAction,
    label: &str,
) {
    parent
        .spawn((
            MenuButtonAction { index, action },
            Button,
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                margin: UiRect::axes(Val::Px(0.0), Val::Px(3.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(CYBER_BUTTON_NORMAL),
            BorderColor::all(Color::srgba(0.0, 0.8, 0.9, 0.2)),
        ))
        .with_child((
            Text::new(format!("  {label}")),
            TextFont {
                font_size: bevy::text::FontSize::Px(15.0),
                ..default()
            },
            TextColor(Color::srgb(0.85, 0.9, 0.95)),
        ))
        .observe(
            move |_click: On<Pointer<Click>>, mut nav_state: ResMut<MenuNavigationState>| {
                nav_state.selected_index = index;
            },
        );
}

#[derive(SystemParam)]
struct MenuRunParams<'w, 's> {
    commands: Commands<'w, 's>,
    assets: Res<'w, LightcycleAssets>,
    meshes: ResMut<'w, Assets<Mesh>>,
    lightcycle_state: ResMut<'w, crate::lightcycle::LightcycleState>,
    flood_state: ResMut<'w, FloodState>,
    cache_state: ResMut<'w, CacheState>,
    history_state: ResMut<'w, HistoryState>,
}

/// Handles keyboard navigation and execution of menu actions.
#[allow(clippy::too_many_arguments)]
fn handle_menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<InteractionMode>,
    mut screen: ResMut<MenuScreen>,
    mut nav_state: ResMut<MenuNavigationState>,
    mut render_settings: ResMut<RenderSettings>,
    mut ui_settings: ResMut<UiSettings>,
    mut music_state: ResMut<MusicState>,
    mut app_exit: MessageWriter<AppExit>,
    navigator: Res<NavigatorResource>,
    mut directory_requests: MessageWriter<DirectoryRequested>,
    mut orbit: ResMut<OrbitCameraResource>,
    mut run_params: MenuRunParams,
) {
    if *mode != InteractionMode::MainMenu {
        return;
    }

    let item_count = match *screen {
        MenuScreen::Main => 5,
        MenuScreen::Settings => 9,
        MenuScreen::Controls => 1,
    };

    // Up / W
    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
        if nav_state.selected_index > 0 {
            nav_state.selected_index -= 1;
        } else {
            nav_state.selected_index = item_count - 1;
        }
    }

    // Down / S
    if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
        if nav_state.selected_index + 1 < item_count {
            nav_state.selected_index += 1;
        } else {
            nav_state.selected_index = 0;
        }
    }

    // Escape handling
    if keys.just_pressed(KeyCode::Escape) {
        if *screen != MenuScreen::Main {
            *screen = MenuScreen::Main;
            nav_state.selected_index = 0;
            return;
        } else {
            app_exit.write(AppExit::Success);
            return;
        }
    }

    // Left / Right in settings
    let adjust_left = keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA);
    let adjust_right = keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD);
    let activate = keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space);

    if *screen == MenuScreen::Settings && (adjust_left || adjust_right) {
        match nav_state.selected_index {
            0 => {
                render_settings.msaa = match render_settings.msaa {
                    1 => {
                        if adjust_right {
                            2
                        } else {
                            8
                        }
                    }
                    2 => {
                        if adjust_right {
                            4
                        } else {
                            1
                        }
                    }
                    4 => {
                        if adjust_right {
                            8
                        } else {
                            2
                        }
                    }
                    8 => {
                        if adjust_right {
                            1
                        } else {
                            4
                        }
                    }
                    _ => 1,
                };
            }
            1 => render_settings.bloom = !render_settings.bloom,
            2 => render_settings.scanlines = !render_settings.scanlines,
            3 => render_settings.vignette = !render_settings.vignette,
            4 => {
                let enabled = !music_state.enabled;
                music_state.enabled = enabled;
                music_state.handle.set_enabled(enabled);
            }
            5 => {
                let vol = if adjust_left {
                    (music_state.volume - 0.1).max(0.0)
                } else {
                    (music_state.volume + 0.1).min(1.0)
                };
                music_state.volume = vol;
                music_state.handle.set_volume(vol);
            }
            6 => ui_settings.show_labels = !ui_settings.show_labels,
            7 => ui_settings.show_fps = !ui_settings.show_fps,
            _ => {}
        }
    }

    // Activate item
    if activate {
        match *screen {
            MenuScreen::Main => {
                match nav_state.selected_index {
                    0 => {
                        // Ride the grid!
                        let path = navigator.0.current_path.clone();
                        let run = build_active_run(&path, navigator.0.entries.clone());
                        run_params.lightcycle_state.grace_room = true;
                        run_params.lightcycle_state.rides_started = true;
                        spawn_run_entities(
                            &mut run_params.commands,
                            &run_params.assets,
                            &mut run_params.meshes,
                            &run,
                        );
                        decorate_directory_run(
                            &mut run_params.commands,
                            &run_params.assets,
                            &mut run_params.meshes,
                            &mut run_params.lightcycle_state,
                            &mut run_params.flood_state,
                            &path,
                            &run,
                        );
                        run_params.history_state.commit(&path);
                        run_params.cache_state.visited.insert(path);
                        run_params.lightcycle_state.run = Some(run);
                        *mode = InteractionMode::Lightcycle;
                    }
                    1 => {
                        // Explore directory!
                        orbit.reset_target();
                        orbit.target = Vec3::ZERO;
                        directory_requests.write(DirectoryRequested {
                            path: navigator.0.current_path.clone(),
                        });
                        *mode = InteractionMode::Explorer;
                    }
                    2 => {
                        *screen = MenuScreen::Settings;
                        nav_state.selected_index = 0;
                    }
                    3 => {
                        *screen = MenuScreen::Controls;
                        nav_state.selected_index = 0;
                    }
                    4 => {
                        app_exit.write(AppExit::Success);
                    }
                    _ => {}
                }
            }
            MenuScreen::Settings => match nav_state.selected_index {
                0 => {
                    render_settings.msaa = match render_settings.msaa {
                        1 => 2,
                        2 => 4,
                        4 => 8,
                        _ => 1,
                    };
                }
                1 => render_settings.bloom = !render_settings.bloom,
                2 => render_settings.scanlines = !render_settings.scanlines,
                3 => render_settings.vignette = !render_settings.vignette,
                4 => {
                    let enabled = !music_state.enabled;
                    music_state.enabled = enabled;
                    music_state.handle.set_enabled(enabled);
                }
                5 => {
                    let mut vol = music_state.volume + 0.2;
                    if vol > 1.05 {
                        vol = 0.0;
                    }
                    music_state.volume = vol;
                    music_state.handle.set_volume(vol);
                }
                6 => ui_settings.show_labels = !ui_settings.show_labels,
                7 => ui_settings.show_fps = !ui_settings.show_fps,
                8 => {
                    *screen = MenuScreen::Main;
                    nav_state.selected_index = 2;
                }
                _ => {}
            },
            MenuScreen::Controls => {
                *screen = MenuScreen::Main;
                nav_state.selected_index = 3;
            }
        }
    }
}
