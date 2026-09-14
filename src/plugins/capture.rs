//! Automated screenshot and GIF capture harness.

use crate::disc::language::SourceGame;
use crate::disc::load::WarpRequested;
use crate::state::InteractionMode;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use std::fs;
use std::path::PathBuf;

pub struct CapturePlugin {
    pub enabled: bool,
}

#[derive(Resource)]
struct CaptureState {
    elapsed: f32,
    phase: u32,
    frame_index: u32,
    next_frame_time: f32,
}

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        if !self.enabled {
            return;
        }

        let _ = fs::create_dir_all("assets/screenshots");
        let _ = fs::create_dir_all("target/frames");

        app.insert_resource(CaptureState {
            elapsed: 0.0,
            phase: 0,
            frame_index: 0,
            next_frame_time: 0.0,
        })
        .add_systems(Update, run_capture);
    }
}

#[allow(clippy::too_many_arguments)]
fn run_capture(
    time: Res<Time>,
    mut commands: Commands,
    mut state: ResMut<CaptureState>,
    mut mode: ResMut<InteractionMode>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut warps: MessageWriter<WarpRequested>,
    mut exit: MessageWriter<AppExit>,
) {
    let dt = time.delta_secs();
    state.elapsed += dt;
    let t = state.elapsed;

    match state.phase {
        0 => {
            // Wait 0.5s for initial scene to load, then press Enter to start riding the grid
            if t >= 0.5 {
                keys.press(KeyCode::Enter);
                state.phase = 1;
            }
        }
        1 => {
            // Wait for camera transition to complete and land right behind the cycle (t >= 2.8s)
            if (2.8..7.5).contains(&t) {
                // Steer at strategic intervals to show turns and trail curvature
                if (3.8..3.9).contains(&t) {
                    keys.press(KeyCode::KeyA);
                }
                if (5.2..5.3).contains(&t) {
                    keys.press(KeyCode::KeyD);
                }
                if (6.4..6.5).contains(&t) {
                    keys.press(KeyCode::KeyA);
                }

                // Capture frames for smooth animated GIF
                if t >= state.next_frame_time {
                    state.next_frame_time = t + 0.12;
                    let path =
                        PathBuf::from(format!("target/frames/frame_{:04}.png", state.frame_index));
                    if state.frame_index == 10 {
                        // Also save a dedicated high-res screenshot of lightcycle gameplay
                        commands
                            .spawn(Screenshot::primary_window())
                            .observe(save_to_disk(PathBuf::from(
                                "assets/screenshots/lightcycle.png",
                            )));
                    }
                    state.frame_index += 1;
                    commands
                        .spawn(Screenshot::primary_window())
                        .observe(save_to_disk(path));
                }
            } else if t >= 7.5 {
                state.phase = 2;
            }
        }
        2 => {
            // Warp to Disc Wars
            println!("[capture] warping to Disc Wars");
            *mode = InteractionMode::Lightcycle;
            warps.write(WarpRequested {
                game: SourceGame::DiscWars,
            });
            state.phase = 3;
        }
        3 => {
            // Let Disc Wars arena settle and take screenshot
            if t >= 9.0 {
                println!("[capture] capturing disc_wars.png");
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(PathBuf::from(
                        "assets/screenshots/disc_wars.png",
                    )));
                state.phase = 4;
            }
        }
        4 => {
            // Warp to Stealth
            if t >= 9.6 {
                println!("[capture] warping to Stealth");
                *mode = InteractionMode::Lightcycle;
                warps.write(WarpRequested {
                    game: SourceGame::Stealth,
                });
                state.phase = 5;
            }
        }
        5 => {
            // Let Stealth arena settle and take screenshot
            if t >= 11.2 {
                println!("[capture] capturing stealth.png");
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(PathBuf::from(
                        "assets/screenshots/stealth.png",
                    )));
                state.phase = 6;
            }
        }
        6 => {
            // Warp to Asteroids
            if t >= 11.8 {
                println!("[capture] warping to Asteroids");
                *mode = InteractionMode::Lightcycle;
                warps.write(WarpRequested {
                    game: SourceGame::Asteroids,
                });
                state.phase = 7;
            }
        }
        7 => {
            // Let Asteroids arena settle and take screenshot
            if t >= 13.2 {
                println!("[capture] capturing asteroid_field.png");
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(PathBuf::from(
                        "assets/screenshots/asteroid_field.png",
                    )));
                state.phase = 8;
            }
        }
        8 if t >= 14.2 => {
            // Finish and exit
            println!("[capture] capture complete, exiting");
            exit.write(AppExit::Success);
            state.phase = 9;
        }
        _ => {}
    }
}
