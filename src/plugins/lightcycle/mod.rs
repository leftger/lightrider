use self::character::sync_character_entities;
use crate::disc::load::{SourceLoadFailed, SourceLoaded, SourceRequested, WarpRequested};
use crate::disc::plugin::sync_disc_entities;
use crate::document::load::{DocumentLoadFailed, DocumentLoaded, DocumentRequested};
use crate::lightcycle::LightcycleState;
use crate::lightcycle::scene::SceneEntities;
use crate::state::DirectorySceneRoot;
use crate::state::{
    CacheState, FloodState, HistoryState, InteractionMode, PauseState, StackMotion,
};
use bevy::prelude::*;
pub(crate) mod assets;
pub(crate) mod camera;
pub(crate) mod character;
pub(crate) mod city;
pub(crate) mod decor;
pub(crate) mod entry;
pub(crate) mod input;
pub(crate) mod load;
pub(crate) mod run;
pub(crate) mod space;
pub(crate) mod step;
pub(crate) mod trail;

use self::assets::setup_lightcycle_assets;
use self::camera::update_chase_camera;
use self::character::{
    drive_character_walk, fit_guard_cones, prepare_character_walk, tag_character_model,
};
use self::decor::{
    animate_city_beacons, animate_parent_gate, animate_stack_frames, update_flood, update_gc_sweep,
};
use self::entry::{
    animate_entry_effect, cleanup_orphaned_entry_effect, spawn_crash_effect, spawn_entry_effect,
    update_crash_effects,
};
use self::input::{read_lightcycle_input, update_cycle_transform};
use self::load::{
    apply_document_load_failure, apply_load_failure, apply_source_load_failure,
    handle_warp_requests, poll_document_loads, poll_source_loads, reset_on_directory_loaded,
    reset_on_document_loaded, reset_on_source_loaded, start_document_loads, start_source_loads,
};
use self::run::{
    apply_district_ambience, apply_mode_swap, in_lightcycle_mode, restore_directory_arena,
    toggle_mode,
};
use self::step::step_lightcycle;
use self::trail::update_trail_mesh;
use crate::asteroids::plugin::sync_asteroid_entities;
use crate::bomberman::plugin::sync_bomberman_entities;
use crate::breaker::plugin::sync_breaker_entities;
use crate::columns::plugin::sync_columns_entities;
use crate::disc::plugin::animate_disc_pickups;
use crate::disc::plugin::update_disc_focus;
use crate::document::plugin::update_document_focus;
use crate::frogger::plugin::sync_frogger_entities;
use crate::galaga::plugin::sync_galaga_entities;
use crate::pacman::plugin::sync_pacman_entities;
use crate::plinko::plugin::sync_plinko_entities;
use crate::qbert::plugin::sync_qbert_entities;
use crate::snake::plugin::sync_snake_entities;
use crate::stealth::plugin::sync_stealth_entities;
use crate::tetris::plugin::sync_tetris_entities;

pub struct LightcyclePlugin;

impl Plugin for LightcyclePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InteractionMode>()
            .init_resource::<LightcycleState>()
            .init_resource::<PauseState>()
            .init_resource::<CacheState>()
            .init_resource::<FloodState>()
            .init_resource::<HistoryState>()
            .init_resource::<StackMotion>()
            .add_message::<DocumentRequested>()
            .add_message::<DocumentLoaded>()
            .add_message::<DocumentLoadFailed>()
            .add_message::<SourceRequested>()
            .add_message::<SourceLoaded>()
            .add_message::<SourceLoadFailed>()
            .add_message::<WarpRequested>()
            .add_systems(Startup, setup_lightcycle_assets)
            .add_systems(
                Update,
                (
                    (
                        toggle_mode,
                        apply_mode_swap,
                        reset_on_directory_loaded,
                        start_document_loads,
                        poll_document_loads,
                        reset_on_document_loaded,
                        start_source_loads,
                        poll_source_loads,
                        reset_on_source_loaded,
                        handle_warp_requests,
                        apply_load_failure,
                        apply_document_load_failure,
                        apply_source_load_failure,
                        sync_directory_scene_visibility,
                    ),
                    (
                        read_lightcycle_input.run_if(in_lightcycle_mode),
                        step_lightcycle.run_if(in_lightcycle_mode),
                        update_flood.run_if(in_lightcycle_mode),
                        update_gc_sweep.run_if(in_lightcycle_mode),
                        restore_directory_arena.run_if(in_lightcycle_mode),
                        spawn_crash_effect.run_if(in_lightcycle_mode),
                        update_crash_effects.run_if(in_lightcycle_mode),
                        update_trail_mesh.run_if(in_lightcycle_mode),
                        animate_parent_gate.run_if(in_lightcycle_mode),
                        animate_city_beacons.run_if(in_lightcycle_mode),
                        animate_stack_frames.run_if(in_lightcycle_mode),
                        update_document_focus.run_if(in_lightcycle_mode),
                        update_disc_focus.run_if(in_lightcycle_mode),
                        sync_disc_entities.run_if(in_lightcycle_mode),
                    ),
                    (
                        sync_asteroid_entities.run_if(in_lightcycle_mode),
                        sync_galaga_entities.run_if(in_lightcycle_mode),
                        sync_pacman_entities.run_if(in_lightcycle_mode),
                        sync_columns_entities.run_if(in_lightcycle_mode),
                        sync_tetris_entities.run_if(in_lightcycle_mode),
                        sync_frogger_entities.run_if(in_lightcycle_mode),
                        sync_qbert_entities.run_if(in_lightcycle_mode),
                        sync_bomberman_entities.run_if(in_lightcycle_mode),
                        sync_plinko_entities.run_if(in_lightcycle_mode),
                        sync_snake_entities.run_if(in_lightcycle_mode),
                        sync_character_entities.run_if(in_lightcycle_mode),
                        tag_character_model.run_if(in_lightcycle_mode),
                        prepare_character_walk.run_if(in_lightcycle_mode),
                        drive_character_walk.run_if(in_lightcycle_mode),
                        sync_breaker_entities.run_if(in_lightcycle_mode),
                        sync_stealth_entities.run_if(in_lightcycle_mode),
                        fit_guard_cones.run_if(in_lightcycle_mode),
                        animate_disc_pickups.run_if(in_lightcycle_mode),
                        update_cycle_transform.run_if(in_lightcycle_mode),
                        update_chase_camera.run_if(in_lightcycle_mode),
                    ),
                )
                    .chain()
                    .after(crate::plugins::filesystem::apply_loaded),
            )
            .add_systems(
                Update,
                (
                    cleanup_orphaned_entry_effect
                        .after(read_lightcycle_input)
                        .before(spawn_entry_effect),
                    spawn_entry_effect.after(step_lightcycle),
                    animate_entry_effect
                        .after(spawn_entry_effect)
                        .after(update_cycle_transform)
                        .before(update_chase_camera),
                )
                    .run_if(in_lightcycle_mode),
            )
            // Outside the chained group: that tuple is already at Bevy's arity
            // limit, and the tint only has to land after the run it describes.
            .add_systems(Update, apply_district_ambience.after(apply_mode_swap));
    }
}

// The query types the sync systems are written in. Every minigame keeps a pool of
// entities that are spawned once and then shown, hidden and moved to match its
// sim, and writing that query out per system made the signatures unreadable: the
// alias says which marker keys the pool and what the sim drives on it.
//
// `F` is the filter that keeps sibling pools apart, since two pools of the same
// shape would otherwise match each other's entities.

/// Lane markings use a district's primary accent, so ground seams use the
/// secondary one to stay readable against them.
const CITY_TRIM_ACCENT: usize = 1;

fn despawn_lightcycle_entities(commands: &mut Commands, old_lightcycle_entities: &SceneEntities) {
    for entity in old_lightcycle_entities {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests;

pub(crate) fn sync_directory_scene_visibility(
    mode: Res<InteractionMode>,
    mut directory_scene: Query<&mut Visibility, With<DirectorySceneRoot>>,
) {
    let wanted = if *mode == InteractionMode::Explorer {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    // Writing unconditionally marks every entity in the scene changed each
    // frame, which makes Bevy redo visibility propagation for all of them.
    for mut visibility in &mut directory_scene {
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
}
