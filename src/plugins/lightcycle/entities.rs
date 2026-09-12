//! Pooled-entity sync systems: show, hide and move a mini-game's entity pool.

use super::camera::character_pose;
use crate::config;
use crate::disc::plugin::disc_entity_position;
use crate::disc::plugin::{DiscPickupEntity, OpponentDiscEntity, OpponentEntity, PlayerDiscEntity};
use crate::lightcycle::LightcycleState;
use crate::lightcycle::scene::CharacterAnim;
use crate::lightcycle::scene::CycleEntity;
use crate::lightcycle::scene::Fighter;
use crate::lightcycle::scene::FreeOf;
use crate::lightcycle::scene::PooledShown;
use crate::state::{DirectorySceneRoot, InteractionMode};
use bevy::prelude::*;

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

/// Keeps the disc, opponent, opponent disc, and pickups glued to the sim.
pub(crate) fn sync_disc_entities(
    state: Res<LightcycleState>,
    mut player_disc: Fighter<PlayerDiscEntity, OpponentEntity, OpponentDiscEntity>,
    mut opponent: Fighter<OpponentEntity, PlayerDiscEntity, OpponentDiscEntity>,
    mut opponent_disc: Fighter<OpponentDiscEntity, PlayerDiscEntity, OpponentEntity>,
    mut pickups: PooledShown<
        DiscPickupEntity,
        FreeOf<PlayerDiscEntity, OpponentEntity, OpponentDiscEntity>,
    >,
) {
    let Some(run) = state.run.as_ref() else {
        return;
    };
    let Some(disc) = run.source_disc() else {
        return;
    };

    if let Ok((mut transform, mut visibility)) = player_disc.single_mut() {
        match disc.player_disc.as_ref() {
            Some(flying) => {
                transform.translation = disc_entity_position(flying.cell);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
    if let Ok((mut transform, mut visibility)) = opponent.single_mut() {
        if disc.opponent.alive {
            // The opponent steps a whole cell at a time. Render it partway to the
            // cell it is walking into so it glides instead of teleporting; while
            // it charges it stands exactly on its cell, so the shot is readable.
            let progress = if disc.opponent.windup > 0.0 {
                0.0
            } else {
                disc.opponent.move_clock.clamp(0.0, 1.0)
            };
            let (dx, dz) = disc.opponent.heading.delta();
            transform.translation =
                config::ground_position(disc.opponent.cell.0, disc.opponent.cell.1)
                    + Vec3::new(dx as f32, 0.0, dz as f32) * (progress * config::GRID_SPACING)
                    + Vec3::Y * (config::disc::RECOGNIZER_HEIGHT * 0.5);
            // Swell while winding up, so its shot is telegraphed.
            let charge =
                (disc.opponent.windup / config::disc::DISC_OPPONENT_WINDUP).clamp(0.0, 1.0);
            transform.scale =
                Vec3::new(1.0 + charge * 0.35, 1.0 - charge * 0.2, 1.0 + charge * 0.35);
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
    if let Ok((mut transform, mut visibility)) = opponent_disc.single_mut() {
        match disc.opponent.disc.as_ref() {
            Some(flying) => {
                transform.translation = disc_entity_position(flying.cell);
                *visibility = Visibility::Visible;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
    for (pickup, mut visibility) in &mut pickups {
        *visibility = if disc.taken.contains(&pickup.index) {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}

/// Poses the on-foot character, whichever game it belongs to.
pub(crate) fn sync_character_entities(
    state: Res<LightcycleState>,
    time: Res<Time>,
    mut character: Query<(&mut Transform, &mut CharacterAnim), Without<CycleEntity>>,
) {
    let Some(run) = state.run.as_ref() else {
        return;
    };
    let Some(pose) = character_pose(run) else {
        return;
    };

    let dt = time.delta_secs();
    for (mut transform, mut anim) in &mut character {
        // The stealth sim moves in whole cells; easing toward the cell turns
        // that into a glide, and the walk clip supplies the limbs. The
        // platformer's physics is already continuous.
        let base = if pose.smooth {
            // Cover the ground at the pace the sim steps, instead of easing to
            // each cell and waiting. The second term only bites once the figure
            // has fallen behind, so a frame hitch does not leave it trailing.
            let to_target = pose.target - anim.base;
            let distance = to_target.length();
            let travel = config::stealth::STEALTH_WALK_SPEED.max(distance * 2.0) * dt;
            if distance <= travel {
                pose.target
            } else {
                anim.base + to_target / distance * travel
            }
        } else {
            pose.target
        };
        anim.base = base;
        transform.translation = base;
        transform.rotation = Quat::from_rotation_y(pose.yaw);
    }
}
