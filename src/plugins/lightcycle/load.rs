//! Loading a directory, document or source file into a run, and the failures that follow.

use super::decor::decorate_directory_run;
use super::despawn_lightcycle_entities;
use super::run::{build_active_run, build_document_run, build_source_run, spawn_run_entities};
use crate::config;
use crate::disc::language::SourceLanguage;
use crate::disc::load::{
    SourceLoadFailed, SourceLoadState, SourceLoaded, SourceRequested, WarpRequested,
};
use crate::document::load::{
    DocumentLoadFailed, DocumentLoadState, DocumentLoaded, DocumentRequested,
};
use crate::lightcycle::LightcycleState;
use crate::lightcycle::logic::RunPhase;
use crate::lightcycle::scene::EntryTransportEntity;
use crate::lightcycle::scene::LightcycleAssets;
use crate::lightcycle::scene::SceneEntities;
use crate::load::{DirectoryLoadFailed, DirectoryLoaded};
use crate::music::sfx::MusicSfx;
use crate::state::{CacheState, FloodState, HistoryState, InteractionMode, PauseState};
use bevy::prelude::*;
use std::path::PathBuf;

#[allow(clippy::too_many_arguments)]
pub(crate) fn reset_on_directory_loaded(
    mut loaded: MessageReader<DirectoryLoaded>,
    mode: Res<InteractionMode>,
    mut state: ResMut<LightcycleState>,
    mut cache: ResMut<CacheState>,
    mut flood: ResMut<FloodState>,
    mut history: ResMut<HistoryState>,
    assets: Res<LightcycleAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mut effects: MessageWriter<MusicSfx>,
    old_lightcycle_entities: SceneEntities,
) {
    if *mode != InteractionMode::Lightcycle {
        return;
    }

    for event in loaded.read() {
        despawn_lightcycle_entities(&mut commands, &old_lightcycle_entities);

        let run = build_active_run(&event.path, event.contents.nodes.clone());
        spawn_run_entities(&mut commands, &assets, &mut meshes, &run);
        // Opening a document is never the first ride, so its room is armed.
        state.grace_room = false;
        decorate_directory_run(
            &mut commands,
            &assets,
            &mut meshes,
            &mut state,
            &mut flood,
            &event.path,
            &run,
        );
        history.commit(&event.path);

        // A directory already opened is a cache hit: a short speed surge for
        // the rest of the run, plus the disk-head seek on the hop in.
        let hit = !cache.visited.insert(event.path.clone());
        state.cache_boost = if hit {
            config::lightcycle::CACHE_BOOST_SECONDS
        } else {
            0.0
        };
        state.gc_timer = config::lightcycle::GC_INTERVAL_SECONDS;
        state.gc_pause = 0.0;
        state.gc_sweep = 0.0;
        effects.write(MusicSfx::Seek);

        state.clock = 0.0;
        state.crash_fx = None;
        state.entry_fx = None;
        state.restore_directory = false;
        state.run = Some(run);
    }
}

pub(crate) fn start_document_loads(
    mut requests: MessageReader<DocumentRequested>,
    mut documents: ResMut<DocumentLoadState>,
) {
    for request in requests.read() {
        let generation = documents.next_generation();
        documents.begin_load(generation, request.path.clone());
    }
}

pub(crate) fn poll_document_loads(
    mut documents: ResMut<DocumentLoadState>,
    mut loaded: MessageWriter<DocumentLoaded>,
    mut failed: MessageWriter<DocumentLoadFailed>,
) {
    while let Some(result) = documents.poll() {
        if result.generation != documents.generation {
            continue;
        }
        match result.result {
            Ok(bytes) => {
                loaded.write(DocumentLoaded {
                    path: result.path,
                    bytes,
                });
            }
            Err(message) => {
                failed.write(DocumentLoadFailed {
                    path: result.path,
                    message,
                });
            }
        }
    }
}

pub(crate) fn reset_on_document_loaded(
    mut loaded: MessageReader<DocumentLoaded>,
    mode: Res<InteractionMode>,
    mut state: ResMut<LightcycleState>,
    assets: Res<LightcycleAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    old_lightcycle_entities: SceneEntities,
) {
    if *mode != InteractionMode::Lightcycle {
        return;
    }

    for event in loaded.read() {
        despawn_lightcycle_entities(&mut commands, &old_lightcycle_entities);
        let run = build_document_run(&event.path, &event.bytes);
        spawn_run_entities(&mut commands, &assets, &mut meshes, &run);
        state.clock = 0.0;
        state.crash_fx = None;
        state.entry_fx = None;
        state.restore_directory = false;
        state.run = Some(run);
    }
}

pub(crate) fn apply_load_failure(
    mut failed: MessageReader<DirectoryLoadFailed>,
    mut state: ResMut<LightcycleState>,
    mut commands: Commands,
    effect_entities: Query<Entity, With<EntryTransportEntity>>,
) {
    if failed.read().next().is_none() {
        return;
    }

    if let Some(run) = state.run.as_mut()
        && run.sim.phase == RunPhase::EnteringDir
    {
        run.sim.pending_request = None;
        run.entering_label = None;
        run.sim.phase = RunPhase::Running;
    }
    state.entry_fx = None;
    for entity in &effect_entities {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn apply_document_load_failure(
    mut failed: MessageReader<DocumentLoadFailed>,
    mut documents: ResMut<DocumentLoadState>,
    mut state: ResMut<LightcycleState>,
) {
    let Some(event) = failed.read().next() else {
        return;
    };
    documents.last_error = Some(format!("{}: {}", event.path.display(), event.message));
    if let Some(run) = state.run.as_mut() {
        run.sim.pending_request = None;
        run.entering_label = None;
        run.sim.phase = RunPhase::Running;
        run.crash_label = Some(format!("could not open {}", event.path.display()));
    }
}

pub(crate) fn start_source_loads(
    mut requests: MessageReader<SourceRequested>,
    mut sources: ResMut<SourceLoadState>,
) {
    for request in requests.read() {
        let generation = sources.next_generation();
        sources.begin_load(generation, request.path.clone());
    }
}

pub(crate) fn poll_source_loads(
    mut sources: ResMut<SourceLoadState>,
    mut loaded: MessageWriter<SourceLoaded>,
    mut failed: MessageWriter<SourceLoadFailed>,
) {
    while let Some(result) = sources.poll() {
        if result.generation != sources.generation {
            continue;
        }
        match result.result {
            Ok(bytes) => {
                loaded.write(SourceLoaded {
                    path: result.path,
                    bytes,
                });
            }
            Err(message) => {
                failed.write(SourceLoadFailed {
                    path: result.path,
                    message,
                });
            }
        }
    }
}

pub(crate) fn reset_on_source_loaded(
    mut loaded: MessageReader<SourceLoaded>,
    mode: Res<InteractionMode>,
    mut state: ResMut<LightcycleState>,
    assets: Res<LightcycleAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    old_lightcycle_entities: SceneEntities,
) {
    if *mode != InteractionMode::Lightcycle {
        return;
    }

    for event in loaded.read() {
        let Some(language) = SourceLanguage::from_path(&event.path) else {
            continue;
        };
        despawn_lightcycle_entities(&mut commands, &old_lightcycle_entities);
        let run = build_source_run(&event.path, language, &event.bytes);
        spawn_run_entities(&mut commands, &assets, &mut meshes, &run);
        state.clock = 0.0;
        state.crash_fx = None;
        state.entry_fx = None;
        state.restore_directory = false;
        state.run = Some(run);
    }
}

/// Warps into a game from the pause menu, using a synthetic file of the
/// game's representative language. Mirrors `reset_on_source_loaded` so the
/// swap looks identical to entering a real file.
pub(crate) fn handle_warp_requests(
    mut requests: MessageReader<WarpRequested>,
    mut state: ResMut<LightcycleState>,
    assets: Res<LightcycleAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mut pause: ResMut<PauseState>,
    old_lightcycle_entities: SceneEntities,
) {
    let Some(request) = requests.read().next() else {
        return;
    };
    let language = SourceLanguage::for_game(request.game);
    let bytes = warp_bytes(language);
    let path = PathBuf::from(format!("warp/level.{}", language.name()));
    despawn_lightcycle_entities(&mut commands, &old_lightcycle_entities);
    let run = build_source_run(&path, language, &bytes);
    spawn_run_entities(&mut commands, &assets, &mut meshes, &run);
    state.clock = 0.0;
    state.crash_fx = None;
    state.entry_fx = None;
    state.restore_directory = false;
    state.run = Some(run);
    pause.paused = false;
}

/// A short synthetic source file so the warp menu can build any game without
/// hunting for a real file of that language.
pub(crate) fn warp_bytes(language: SourceLanguage) -> Vec<u8> {
    let text = match language {
        SourceLanguage::Rust => "fn warp() {\n    let x = 1;\n}\nfn chase() {}\n",
        SourceLanguage::C => "int warp(void) {\n    return 1;\n}\nint chase(void) { return 0; }\n",
        SourceLanguage::Cpp => "int warp() {\n    return 1;\n}\nint chase() { return 0; }\n",
        SourceLanguage::Python => "def warp():\n    return 1\n\ndef chase():\n    return 0\n",
        SourceLanguage::Slint => "export component Warp {\n    in property <int> x: 1;\n}\n",
        SourceLanguage::Lua => {
            "local function warp()\n    return 1\nend\nlocal function chase() return 0 end\n"
        }
        SourceLanguage::Shell => "warp() {\n    echo 1\n}\nchase() { echo 0; }\n",
        SourceLanguage::Toml => "[warp]\nvalue = 1\n[chase]\nvalue = 0\n",
        SourceLanguage::Json => "{\n  \"warp\": 1,\n  \"chase\": 0\n}\n",
        SourceLanguage::Go => {
            "package warp\n\nfunc warp() int {\n    return 1\n}\nfunc chase() int { return 0 }\n"
        }
        SourceLanguage::Ruby => "def warp\n  1\nend\n\ndef chase\n  0\nend\n",
        SourceLanguage::Yaml => "warp: 1\nchase: 0\n",
        SourceLanguage::JavaScript => {
            "function warp() {\n    return 1;\n}\nconst chase = () => 0;\n"
        }
        SourceLanguage::Zig => "fn warp() i32 {\n    return 1;\n}\nfn chase() i32 { return 0; }\n",
        SourceLanguage::Php => {
            "<?php\nfunction warp() {\n    return 1;\n}\nfunction chase() { return 0; }\n"
        }
        SourceLanguage::R => "warp <- function() {\n    1\n}\nchase <- function() { 0 }\n",
    };
    text.as_bytes().to_vec()
}

pub(crate) fn apply_source_load_failure(
    mut failed: MessageReader<SourceLoadFailed>,
    mut sources: ResMut<SourceLoadState>,
    mut state: ResMut<LightcycleState>,
) {
    let Some(event) = failed.read().next() else {
        return;
    };
    sources.last_error = Some(format!("{}: {}", event.path.display(), event.message));
    if let Some(run) = state.run.as_mut() {
        run.sim.pending_request = None;
        run.entering_label = None;
        run.sim.phase = RunPhase::Running;
        run.crash_label = Some(format!("could not open {}", event.path.display()));
    }
}
