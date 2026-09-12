//! Moved out of `super` by the modularity pass: cone_mesh, cone_positions, drive_character_walk, fit_guard_cones, prepare_character_walk, refit_cone, tag_character_model, vision_cone_mesh, walk_players.
//!
//! Nothing about them changed in the move.

use super::*;

/// A floor fan: a centre point, then one rim point per ray, each reaching as far
/// as that ray can see. Radii are in the mesh's own units.
pub(crate) fn cone_positions(half_angle: f32, radii: &[f32]) -> Vec<[f32; 3]> {
    let segments = radii.len().saturating_sub(1).max(1);
    let mut positions = Vec::with_capacity(radii.len() + 1);
    positions.push([0.0, 0.0, 0.0]);
    for (step, reach) in radii.iter().enumerate() {
        let t = step as f32 / segments as f32;
        let angle = -half_angle + t * half_angle * 2.0;
        positions.push([angle.sin() * reach, 0.0, angle.cos() * reach]);
    }
    positions
}

/// Moves an existing cone's rim out to `radii`, leaving its topology alone.
pub(crate) fn refit_cone(mesh: &mut Mesh, half_angle: f32, radii: &[f32]) {
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, cone_positions(half_angle, radii));
}

/// A cone whose rays reach `radii`: the shape of what a guard can actually see.
pub(crate) fn cone_mesh(half_angle: f32, radii: &[f32]) -> Mesh {
    let positions = cone_positions(half_angle, radii);
    let segments = radii.len().saturating_sub(1).max(1);
    let mut indices = Vec::new();
    for step in 0..segments as u32 {
        indices.extend_from_slice(&[0, step + 1, step + 2]);
    }
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

/// Unit-length flat cone with the stealth half-angle, opening along local `+Z`.
///
/// Used as a guard's cone until its real shape has been measured, so the first
/// frame is not a hole in the floor.
pub(crate) fn vision_cone_mesh(half_angle: f32, segments: usize) -> Mesh {
    let radii = vec![1.0; segments + 1];
    cone_mesh(half_angle, &radii)
}

/// Cuts each guard's cone to what it can actually see, rebuilding the mesh on the
/// first frame and refitting it after that.
///
/// Detection samples line of sight, so a cone that ignores cover tells the player
/// a lie about where they are safe.
pub(crate) fn fit_guard_cones(
    state: Res<LightcycleState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cones: Query<(&mut GuardConeEntity, &mut Transform, &mut Mesh3d)>,
) {
    let Some(room) = state.run.as_ref().and_then(|run| run.source_stealth()) else {
        return;
    };
    for (mut cone, mut transform, mut mesh) in &mut cones {
        let radii = room.vision_radii(cone.index, config::STEALTH_CONE_SEGMENTS);
        match cone.mesh.clone() {
            Some(handle) => {
                if let Some(mut geometry) = meshes.get_mut(&handle) {
                    refit_cone(&mut geometry, config::STEALTH_VISION_HALF_ANGLE, &radii);
                }
            }
            None => {
                // The radii are in cells, so the scale drops to one cell from the
                // fixed reach the placeholder fan was drawn at.
                let handle = meshes.add(cone_mesh(config::STEALTH_VISION_HALF_ANGLE, &radii));
                mesh.0 = handle.clone();
                transform.scale = Vec3::splat(config::GRID_SPACING);
                cone.mesh = Some(handle);
            }
        }
    }
}

/// Builds the walk graph for the character's player and attaches it. The glTF
/// loader makes the `AnimationPlayer` but leaves the graph to us.
pub(crate) fn prepare_character_walk(
    mut commands: Commands,
    assets: Res<LightcycleAssets>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    players: Query<Entity, (With<AnimationPlayer>, Without<CharacterWalk>)>,
) {
    if players.is_empty() {
        return;
    }
    let Some(clip) = gltfs
        .get(&assets.tron_gltf)
        .and_then(|gltf| gltf.named_animations.get(config::WALK_CLIP))
        .cloned()
    else {
        return;
    };
    for entity in &players {
        let (graph, nodes) = AnimationGraph::from_clips([clip.clone()]);
        if let Some(index) = nodes.first().copied() {
            commands.entity(entity).insert((
                AnimationGraphHandle(graphs.add(graph)),
                CharacterWalk(index),
            ));
        }
    }
}

/// Plays the walk while the character is moving and rewinds it when it stops,
/// so it never stands mid-stride. The rate is matched to ground speed: a clip
/// run at the wrong speed makes the feet skate.
pub(crate) fn drive_character_walk(
    state: Res<LightcycleState>,
    assets: Res<LightcycleAssets>,
    gltfs: Res<Assets<Gltf>>,
    mut reported: Local<bool>,
    mut character: Query<(&mut AnimationPlayer, &CharacterWalk), With<CharacterModel>>,
    mut guards: Query<(&mut AnimationPlayer, &CharacterWalk), Without<CharacterModel>>,
) {
    let Some(run) = state.run.as_ref() else {
        return;
    };
    // Say once, when the character's player first exists, what the animation
    // plumbing actually found. If the character ever walks without animating,
    // this line is the first thing to look at.
    if !*reported && !(character.is_empty() && guards.is_empty()) {
        *reported = true;
        let clips: Vec<Box<str>> = gltfs
            .get(&assets.tron_gltf)
            .map(|gltf| gltf.named_animations.keys().cloned().collect())
            .unwrap_or_default();
        info!(
            "walk animation: {} character player(s), {} other, {:?} clip, asset carries {clips:?}",
            character.iter().len(),
            guards.iter().len(),
            config::WALK_CLIP,
        );
    }
    // The patrol step rate, and the character's own ground speed in world units
    // per second. The guards walk continuously, so their clip must not stop
    // just because the player is waiting for them to pass.
    let step_speed = config::STEALTH_WALK_SPEED;
    let character_speed = if let Some(room) = run.source_stealth() {
        if room.walking { step_speed } else { 0.0 }
    } else if let Some(level) = run.source_platformer() {
        level.runner.vx.abs()
    } else {
        0.0
    };
    let guard_speed = if run.source_stealth().is_some() {
        step_speed
    } else {
        0.0
    };

    // The character's own player is tagged; every other player in the scene
    // belongs to a guard, which keeps walking while the player waits.
    walk_players(character.iter_mut(), character_speed);
    walk_players(guards.iter_mut(), guard_speed);
}

/// Tags every entity under an on-foot character, however deep.
pub(crate) fn tag_character_model(
    mut commands: Commands,
    characters: Query<Entity, With<CharacterEntity>>,
    children: Query<&Children>,
    tagged: Query<(), With<CharacterModel>>,
) {
    for character in &characters {
        let mut stack = vec![character];
        while let Some(entity) = stack.pop() {
            if tagged.get(entity).is_err() {
                commands.entity(entity).insert(CharacterModel);
            }
            if let Ok(kids) = children.get(entity) {
                stack.extend(kids.iter());
            }
        }
    }
}

/// Drives one set of players at a ground speed in world units per second. Zero
/// stops them, which is what standing still has to look like.
pub(crate) fn walk_players<'a>(
    players: impl Iterator<Item = (Mut<'a, AnimationPlayer>, &'a CharacterWalk)>,
    speed: f32,
) {
    for (mut player, walk) in players {
        if speed <= 0.05 {
            if player.is_playing_animation(walk.0) {
                player.stop(walk.0);
            }
            continue;
        }
        let rate = (speed / config::WALK_CLIP_GROUND).clamp(0.3, 2.5);
        let active = player.play(walk.0);
        // Bevy's default repeat mode is `Never`: the clip plays once and then
        // parks on its last frame, which reads as a character sliding along
        // frozen mid-stride. Loop it, and rewind it if a previous pass already
        // completed, since `play` never restarts an active animation.
        if active.is_finished() {
            active.replay();
        }
        active.set_speed(rate).repeat();
    }
}
