pub mod bench;
pub mod camera;
pub mod capture;
pub mod effects;
pub mod filesystem;
pub mod grid_rider;
pub mod labels;
pub mod menu;
pub mod music;
pub mod scene;
pub mod selection;
pub mod transition;
pub mod ui;

use bevy::prelude::*;

pub struct RaptorPlugins;

impl Plugin for RaptorPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            camera::CameraPlugin,
            filesystem::FilesystemPlugin,
            scene::ScenePlugin,
            selection::SelectionPlugin,
            labels::LabelsPlugin,
            ui::UiPlugin,
            effects::EffectsPlugin,
            transition::ModeTransitionPlugin,
            grid_rider::GridRiderPlugin,
            music::MusicPlugin,
            menu::MenuPlugin,
        ));
    }
}
