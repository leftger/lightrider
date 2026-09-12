//! Moved out of `super` by the modularity pass: spawn_asteroid_field, spawn_bomber_room, spawn_breaker_court, spawn_frogger_highway, spawn_galaga_field, spawn_gem_well, spawn_pac_maze, spawn_platformer_level, spawn_plinko_board, spawn_qbert_pyramid, spawn_snake_field, spawn_stealth_room, spawn_surfer_course, spawn_surfer_gate, spawn_tetris_board, surfer_river_mesh.
//!
//! Nothing about them changed in the move.

use super::*;

/// Spawns the pooled rock and beam bodies for an asteroid field.
///
/// The sim drives visibility and transforms; the pool is fixed because a rock
/// only ever splits into a bounded number of children.
pub(crate) fn spawn_asteroid_field(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    sim: &AsteroidsSim,
) {
    let rock_count = config::ASTEROIDS_MAX_ROCKS.max(sim.rocks.len());
    for index in 0..rock_count {
        let live = sim.rocks.get(index);
        let radius = live.map_or(1.0, |rock| rock.size.radius());
        commands.spawn((
            LightcycleSceneRoot,
            RockEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.rock_material.clone()),
            Transform::from_xyz(0.0, radius, 0.0).with_scale(Vec3::splat(radius * 2.0)),
            if live.is_some() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }
    for index in 0..config::ASTEROIDS_MAX_BEAMS {
        let live = sim.beams.get(index);
        commands.spawn((
            LightcycleSceneRoot,
            BeamEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.beam_material.clone()),
            Transform::from_xyz(0.0, 0.35, 0.0),
            if live.is_some() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }
}

/// Spawns the pooled bug and beam bodies for a Galaga field.
///
/// The formation is a fixed grid, so the bug pool never grows; the sim drives
/// transforms and visibility.
pub(crate) fn spawn_galaga_field(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    sim: &GalagaSim,
) {
    for (index, bug) in sim.bugs.iter().enumerate() {
        commands.spawn((
            LightcycleSceneRoot,
            BugEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.galaga_bug_material.clone()),
            Transform::from_xyz(bug.x, config::GALAGA_BUG_HEIGHT * 0.5, bug.z).with_scale(
                Vec3::new(
                    config::GALAGA_BUG_RADIUS * 2.0,
                    config::GALAGA_BUG_HEIGHT,
                    config::GALAGA_BUG_RADIUS * 2.0,
                ),
            ),
            if bug.alive {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }
    for index in 0..config::GALAGA_MAX_BEAMS {
        let live = sim.beams.get(index);
        commands.spawn((
            LightcycleSceneRoot,
            GalagaBeamEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.galaga_beam_material.clone()),
            Transform::from_xyz(0.0, 0.35, 0.0),
            if live.is_some() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }
}

/// Spawns the Pac-Man maze: static walls plus pooled dots and ghosts.
pub(crate) fn spawn_pac_maze(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    _meshes: &mut Assets<Mesh>,
    sim: &PacSim,
) {
    for row in 0..config::PAC_ROWS {
        for col in 0..config::PAC_COLS {
            if !PacSim::solid((col, row)) {
                continue;
            }
            let (x, z) = PacSim::center((col, row));
            commands.spawn((
                LightcycleSceneRoot,
                Mesh3d(assets.unit_cube.clone()),
                MeshMaterial3d(assets.stealth_wall_material.clone()),
                Transform::from_xyz(x, config::GRID_SPACING * 0.5, z)
                    .with_scale(Vec3::splat(config::GRID_SPACING)),
                Visibility::Visible,
                Pickable::IGNORE,
            ));
        }
    }
    for (index, ghost) in sim.ghosts.iter().enumerate() {
        commands.spawn((
            LightcycleSceneRoot,
            GhostEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.galaga_bug_material.clone()),
            Transform::from_xyz(ghost.x, 0.9, ghost.z).with_scale(Vec3::splat(1.5)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
    for &cell in &sim.dots {
        let (x, z) = PacSim::center(cell);
        commands.spawn((
            LightcycleSceneRoot,
            DotEntity { cell },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.snake_food_material.clone()),
            Transform::from_xyz(x, 0.35, z).with_scale(Vec3::splat(0.55)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
}

/// Spawns the pooled gem cells of a Columns well.
pub(crate) fn spawn_gem_well(commands: &mut Commands, assets: &LightcycleAssets, sim: &ColumnsSim) {
    let rendered = sim.render_board();
    for (index, colour) in rendered.iter().copied().enumerate() {
        let col = index % config::COLUMNS_COLS;
        let row = index / config::COLUMNS_COLS;
        let x = (col as f32 - (config::COLUMNS_COLS - 1) as f32 * 0.5) * 1.6;
        // Row 0 is the top of the well, so higher rows sit lower on screen.
        // The whole well is lifted above the arena floor.
        let y = ((config::COLUMNS_ROWS - 1) - row) as f32 * 1.6 + 0.8;
        commands.spawn((
            LightcycleSceneRoot,
            GemEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(
                assets.gem_materials[colour.unwrap_or(0) as usize % config::COLUMNS_GEM_COLORS]
                    .clone(),
            ),
            Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(1.5)),
            if colour.is_some() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }

    // A visible frame marks the playfield: side walls plus a floor bar.
    let board_half = config::COLUMNS_COLS as f32 * 0.8;
    let wall_x = board_half + 0.55;
    let board_height = config::COLUMNS_ROWS as f32 * 1.6;
    let wall_scale = Vec3::new(0.4, board_height + 0.6, 0.5);
    for x in [-wall_x, wall_x] {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.stealth_wall_material.clone()),
            Transform::from_xyz(x, board_height * 0.5 + 0.3, 0.0).with_scale(wall_scale),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.stealth_wall_material.clone()),
        Transform::from_xyz(0.0, 0.2, 0.0).with_scale(Vec3::new(wall_x * 2.0 + 0.8, 0.4, 0.5)),
        Visibility::Visible,
        Pickable::IGNORE,
    ));
}

/// Spawns the pooled block cells of a Tetris board.
pub(crate) fn spawn_tetris_board(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    sim: &TetrisSim,
) {
    let rendered = sim.render_board();
    for (index, colour) in rendered.iter().copied().enumerate() {
        let col = index % config::TETRIS_COLS;
        let row = index / config::TETRIS_COLS;
        let x = (col as f32 - (config::TETRIS_COLS - 1) as f32 * 0.5) * 1.2;
        // Row 0 is the top of the board, so higher rows sit lower on screen.
        // The whole board is lifted above the arena floor.
        let y = ((config::TETRIS_ROWS - 1) - row) as f32 * 1.2 + 0.6;
        commands.spawn((
            LightcycleSceneRoot,
            BlockEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.tetris_materials[colour.unwrap_or(0) as usize % 7].clone()),
            Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(1.15)),
            if colour.is_some() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }

    // A visible frame marks the playfield: side walls plus a floor bar.
    let board_half = config::TETRIS_COLS as f32 * 0.6;
    let wall_x = board_half + 0.55;
    let board_height = config::TETRIS_ROWS as f32 * 1.2;
    let wall_scale = Vec3::new(0.4, board_height + 0.6, 0.5);
    for x in [-wall_x, wall_x] {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.stealth_wall_material.clone()),
            Transform::from_xyz(x, board_height * 0.5 + 0.3, 0.0).with_scale(wall_scale),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.stealth_wall_material.clone()),
        Transform::from_xyz(0.0, 0.15, 0.0).with_scale(Vec3::new(wall_x * 2.0 + 0.8, 0.3, 0.5)),
        Visibility::Visible,
        Pickable::IGNORE,
    ));
}

/// Spawns the pooled obstacle cubes of a Frogger highway.
pub(crate) fn spawn_frogger_highway(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    sim: &FroggerSim,
) {
    let cells = sim.obstacle_cells();
    for (index, &cell) in cells.iter().enumerate() {
        let (x, z) = FroggerSim::center(cell);
        commands.spawn((
            LightcycleSceneRoot,
            FrogObstacleEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.galaga_bug_material.clone()),
            Transform::from_xyz(x, 0.6, z).with_scale(Vec3::splat(1.9)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
}

/// Spawns the Q*bert pyramid cubes and its pooled enemies.
pub(crate) fn spawn_qbert_pyramid(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    sim: &QbertSim,
) {
    for row in 0..config::QBERT_ROWS {
        for index in 0..=row {
            let (x, z) = QbertSim::cube_position(row, index);
            let lit = sim.lit[QbertSim::cube_index(row, index)];
            let y =
                (config::QBERT_ROWS as f32 - 1.0 - row as f32) * config::QBERT_CUBE_HEIGHT * 0.5;
            commands.spawn((
                LightcycleSceneRoot,
                QbertCubeEntity { row, index },
                Mesh3d(assets.unit_cube.clone()),
                MeshMaterial3d(if lit {
                    assets.qbert_cube_lit.clone()
                } else {
                    assets.qbert_cube_dim.clone()
                }),
                Transform::from_xyz(x, y, z).with_scale(Vec3::new(
                    config::QBERT_CUBE_SPACING * 0.9,
                    config::QBERT_CUBE_HEIGHT,
                    config::QBERT_CUBE_SPACING * 0.9,
                )),
                Visibility::Visible,
                Pickable::IGNORE,
            ));
        }
    }
    for (index, enemy) in sim.enemies.iter().enumerate() {
        let (x, z) = QbertSim::cube_position(enemy.row, enemy.index);
        let y =
            (config::QBERT_ROWS as f32 - 1.0 - enemy.row as f32) * config::QBERT_CUBE_HEIGHT * 0.5
                + config::QBERT_CUBE_HEIGHT * 0.8;
        commands.spawn((
            LightcycleSceneRoot,
            QbertEnemyEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.qbert_enemy_material.clone()),
            Transform::from_xyz(x, y, z).with_scale(Vec3::splat(1.4)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
}

/// Spawns the Bomberman room: crates, bombs, walls and the exit marker.
pub(crate) fn spawn_bomber_room(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    _meshes: &mut Assets<Mesh>,
    sim: &BomberSim,
) {
    for row in 0..config::BOMBER_ROWS {
        for col in 0..config::BOMBER_COLS {
            let cell = (col, row);
            let (x, z) = BomberSim::center(cell);
            let is_border = col == 0
                || col == config::BOMBER_COLS - 1
                || row == 0
                || row == config::BOMBER_ROWS - 1;
            if is_border {
                commands.spawn((
                    LightcycleSceneRoot,
                    Mesh3d(assets.unit_cube.clone()),
                    MeshMaterial3d(assets.stealth_wall_material.clone()),
                    Transform::from_xyz(x, 1.0, z).with_scale(Vec3::splat(2.0)),
                    Visibility::Visible,
                    Pickable::IGNORE,
                ));
            } else if sim.crates.contains(&cell) {
                commands.spawn((
                    LightcycleSceneRoot,
                    BomberCrateEntity { cell },
                    Mesh3d(assets.unit_cube.clone()),
                    MeshMaterial3d(assets.bomber_crate_material.clone()),
                    Transform::from_xyz(x, 0.8, z).with_scale(Vec3::splat(1.8)),
                    Visibility::Visible,
                    Pickable::IGNORE,
                ));
            }
        }
    }
    let (ex, ez) = BomberSim::center(sim.exit);
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.stealth_exit_material.clone()),
        Transform::from_xyz(ex, 0.5, ez).with_scale(Vec3::new(1.9, 0.4, 1.9)),
        Visibility::Visible,
        Pickable::IGNORE,
    ));
    for index in 0..config::BOMBER_MAX_BOMBS {
        commands.spawn((
            LightcycleSceneRoot,
            BomberBombEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.bomber_bomb_material.clone()),
            Transform::from_xyz(0.0, 0.6, 0.0),
            Visibility::Hidden,
            Pickable::IGNORE,
        ));
    }
}

/// Spawns the Plinko board: static pins and buckets plus pooled balls.
pub(crate) fn spawn_plinko_board(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    _meshes: &mut Assets<Mesh>,
    sim: &PlinkoSim,
) {
    // A dark backdrop behind the board makes the pins and balls read clearly.
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.qbert_cube_dim.clone()),
        Transform::from_xyz(0.0, 0.0, -0.35).with_scale(Vec3::new(
            config::PLINKO_WIDTH + 1.5,
            config::PLINKO_HEIGHT + 1.5,
            0.2,
        )),
        Visibility::Visible,
        Pickable::IGNORE,
    ));
    // The drop rail along the top, where the cycle slides to aim.
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.stealth_wall_material.clone()),
        Transform::from_xyz(0.0, config::PLINKO_HEIGHT * 0.5 + 0.35, 0.0).with_scale(Vec3::new(
            config::PLINKO_WIDTH + 1.0,
            0.6,
            0.6,
        )),
        Visibility::Visible,
        Pickable::IGNORE,
    ));
    for pin in &sim.pins {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.plinko_pin_material.clone()),
            Transform::from_xyz(pin.x, pin.y, 0.0).with_scale(Vec3::splat(0.62)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
    for slot in 0..8 {
        let x = (slot as f32 - 3.5) * 2.0;
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.disc_accent_materials[slot].clone()),
            Transform::from_xyz(x, -config::PLINKO_HEIGHT * 0.5 - 1.0, 0.0)
                .with_scale(Vec3::new(1.9, 1.3, 0.9)),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    }
    for index in 0..config::PLINKO_BALLS {
        commands.spawn((
            LightcycleSceneRoot,
            PlinkoBallEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.plinko_ball_material.clone()),
            Transform::from_xyz(0.0, config::PLINKO_HEIGHT * 0.5, 0.0).with_scale(Vec3::splat(1.1)),
            Visibility::Hidden,
            Pickable::IGNORE,
        ));
    }
}

/// Spawns a snake ring's power-ups and the bar sealing its exit.
///
/// The power-ups are a fixed pool keyed by index; the sim marks them eaten.
pub(crate) fn spawn_snake_field(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    arena: &Arena,
    snake: &SnakeSim,
) {
    for (index, food) in snake.food.iter().enumerate() {
        commands.spawn((
            LightcycleSceneRoot,
            SnakeFoodEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.snake_food_material.clone()),
            Transform::from_translation(
                config::ground_position(food.cell.0, food.cell.1) + Vec3::Y * 0.5,
            )
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_4))
            .with_scale(Vec3::splat(config::SNAKE_FOOD_SIZE)),
            if food.eaten {
                Visibility::Hidden
            } else {
                Visibility::Visible
            },
            Pickable::IGNORE,
        ));
    }

    // A solid bar in the gate opening until the exit opens. The physical lock is
    // the classification; this is what the rider sees.
    let Some(portal) = arena.parent_portal.as_ref() else {
        return;
    };
    let gate = portal.to;
    commands.spawn((
        LightcycleSceneRoot,
        SnakeGateLock,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.snake_lock_material.clone()),
        Transform::from_translation(config::ground_position(gate.0, gate.1) + Vec3::Y * 0.9)
            .with_scale(Vec3::new(
                config::GRID_SPACING,
                config::LIGHTCYCLE_WALL_HEIGHT * 0.9,
                config::GRID_SPACING,
            )),
        if snake.exit_open {
            Visibility::Hidden
        } else {
            Visibility::Visible
        },
        Pickable::IGNORE,
    ));
}

/// Spawns a platformer level: a backdrop, the platforms with neon lips, the
/// exit door and the runner.
pub(crate) fn spawn_platformer_level(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    language: SourceLanguage,
    level: &PlatformerSim,
) {
    let _ = meshes;
    let depth = config::PLATFORMER_DEPTH;
    let lip = assets.disc_accent_materials[disc_language_index(language)].clone();

    // A dark slab behind the level so the platforms read against something.
    let span = level.length + 60.0;
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.platform_material.clone()),
        Transform::from_translation(Vec3::new(level.length * 0.5, 8.0, -depth))
            .with_scale(Vec3::new(span, 46.0, 1.0)),
        Pickable::IGNORE,
    ));

    for platform in &level.platforms {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.platform_material.clone()),
            Transform::from_translation(Vec3::new(
                platform.x + platform.w * 0.5,
                platform.y - platform.h * 0.5,
                0.0,
            ))
            .with_scale(Vec3::new(platform.w, platform.h, depth)),
            Pickable::IGNORE,
        ));
        // The lip marks the surface the runner lands on.
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(lip.clone()),
            Transform::from_translation(Vec3::new(platform.x + platform.w * 0.5, platform.y, 0.0))
                .with_scale(Vec3::new(platform.w, 0.14, depth + 0.2)),
            Pickable::IGNORE,
        ));
    }

    let door = level.exit_box();
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.exit_material.clone()),
        Transform::from_translation(Vec3::new(door.x + door.w * 0.5, door.y - door.h * 0.5, 0.0))
            .with_scale(Vec3::new(door.w, door.h, depth * 0.4)),
        Pickable::IGNORE,
    ));

    commands.spawn((
        LightcycleSceneRoot,
        CharacterEntity,
        CharacterAnim::at(Vec3::new(level.runner.x, level.runner.y, 0.0)),
        Transform::from_translation(Vec3::new(level.runner.x, level.runner.y, 0.0)),
        Visibility::default(),
        Pickable::IGNORE,
        children![(
            WorldAssetRoot(assets.tron_scene.clone()),
            Transform::from_rotation(Quat::from_rotation_y(config::TRON_MODEL_YAW))
                .with_scale(Vec3::splat(config::TRON_MODEL_SCALE)),
        )],
    ));
}

/// Spawns a breaker court: the side and ceiling walls, the bricks, and the ball.
///
/// The bike itself is the paddle, so it is left to `update_cycle_transform`.
pub(crate) fn spawn_breaker_court(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    level: &BreakerSim,
) {
    let (width, height) = level.court;
    let thickness = 0.8;
    let depth = config::PLATFORMER_DEPTH;
    // Side walls and ceiling.
    for (x, y, w, h) in [
        (
            -width * 0.5 - thickness * 0.5,
            height * 0.5,
            thickness,
            height,
        ),
        (
            width * 0.5 + thickness * 0.5,
            height * 0.5,
            thickness,
            height,
        ),
        (
            0.0,
            height + thickness * 0.5,
            width + thickness * 2.0,
            thickness,
        ),
    ] {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.court_material.clone()),
            Transform::from_translation(Vec3::new(x, y, -depth * 0.5))
                .with_scale(Vec3::new(w, h, depth)),
            Pickable::IGNORE,
        ));
    }

    for (index, brick) in level.bricks.iter().enumerate() {
        let (bx, by, bw, bh) = level.brick_box(brick);
        commands.spawn((
            LightcycleSceneRoot,
            BrickEntity { index },
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.brick_material.clone()),
            Transform::from_translation(Vec3::new(bx + bw * 0.5, by + bh * 0.5, 0.0))
                .with_scale(Vec3::new(bw, bh, depth * 0.6)),
            if brick.alive {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Pickable::IGNORE,
        ));
    }

    commands.spawn((
        LightcycleSceneRoot,
        BallEntity,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.ball_material.clone()),
        Transform::from_translation(Vec3::new(level.ball.x, level.ball.y, 0.0))
            .with_scale(Vec3::splat(config::BREAKER_BALL_RADIUS * 2.0)),
        Pickable::IGNORE,
    ));
}

/// Spawns a stealth room: floor, cover, the door, the character and the patrols
/// with their vision cones.
pub(crate) fn spawn_stealth_room(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    room: &StealthSim,
) {
    let _ = meshes;
    let (half_w, half_h) = (
        config::STEALTH_WIDTH as f32 * 0.5,
        config::STEALTH_HEIGHT as f32 * 0.5,
    );
    let span = config::GRID_SPACING;
    let width = half_w * 2.0 * span;
    let height = half_h * 2.0 * span;

    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.stealth_floor_material.clone()),
        Transform::from_translation(Vec3::new(0.0, -0.1, 0.0)).with_scale(Vec3::new(
            width + span * 2.0,
            0.2,
            height + span * 2.0,
        )),
        Pickable::IGNORE,
    ));

    for cell in &room.cover {
        let position = config::ground_position(cell.0, cell.1);
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.stealth_wall_material.clone()),
            Transform::from_translation(position + Vec3::Y * config::STEALTH_WALL_HEIGHT * 0.5)
                .with_scale(Vec3::new(
                    span * 0.96,
                    config::STEALTH_WALL_HEIGHT,
                    span * 0.96,
                )),
            Pickable::IGNORE,
        ));
    }

    let exit = config::ground_position(room.exit.0, room.exit.1);
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(assets.stealth_exit_material.clone()),
        Transform::from_translation(exit + Vec3::Y * 1.4).with_scale(Vec3::new(
            span * 0.9,
            2.8,
            span * 0.9,
        )),
        Pickable::IGNORE,
    ));

    let facing = |angle: f32| Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 - angle);
    let body = |assets: &LightcycleAssets, scale: f32| {
        children![(
            WorldAssetRoot(assets.tron_scene.clone()),
            Transform::from_rotation(Quat::from_rotation_y(config::TRON_MODEL_YAW))
                .with_scale(Vec3::splat(scale)),
        )]
    };

    let start = config::ground_position(room.character.0, room.character.1);
    commands.spawn((
        LightcycleSceneRoot,
        CharacterEntity,
        CharacterAnim::at(start),
        Transform::from_translation(start).with_rotation(facing(room.heading.angle())),
        Visibility::default(),
        Pickable::IGNORE,
        body(assets, config::STEALTH_CHARACTER_SCALE),
    ));

    for (index, guard) in room.guards.iter().enumerate() {
        let cell = guard.cell();
        let position = config::ground_position(cell.0, cell.1);
        commands.spawn((
            LightcycleSceneRoot,
            GuardEntity { index },
            Transform::from_translation(position)
                .with_rotation(facing(guard.patrol.heading().angle())),
            Visibility::default(),
            Pickable::IGNORE,
            body(
                assets,
                config::STEALTH_CHARACTER_SCALE * config::STEALTH_GUARD_SCALE,
            ),
        ));
        commands.spawn((
            LightcycleSceneRoot,
            GuardConeEntity { index, mesh: None },
            Mesh3d(assets.vision_cone.clone()),
            MeshMaterial3d(assets.stealth_cone_material.clone()),
            Transform::from_translation(position + Vec3::Y * 0.08)
                .with_rotation(facing(guard.vision_angle()))
                .with_scale(Vec3::splat(config::STEALTH_CONE_REACH)),
            Pickable::IGNORE,
        ));
    }
}

/// Spawns a river surfer course: the water ribbon that follows the sim's
/// centreline, the rocks, the boost gates and the finish gate. The bike itself
/// is the shared cycle, posed from the sim every frame.
pub(crate) fn spawn_surfer_course(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    meshes: &mut Assets<Mesh>,
    surfer: &SurferSim,
) {
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(meshes.add(surfer_river_mesh(surfer))),
        MeshMaterial3d(assets.surfer_water_material.clone()),
        Pickable::IGNORE,
    ));

    for rock in &surfer.rocks {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(assets.surfer_rock_material.clone()),
            Transform::from_translation(Vec3::new(
                rock.x,
                config::SURFER_ROCK_HEIGHT * 0.5,
                rock.z,
            ))
            .with_scale(Vec3::new(
                rock.radius * 2.0,
                config::SURFER_ROCK_HEIGHT,
                rock.radius * 2.0,
            )),
            Pickable::IGNORE,
        ));
    }

    for gate in &surfer.gates {
        spawn_surfer_gate(
            commands,
            assets,
            gate.x,
            gate.z,
            config::SURFER_GATE_SPAN,
            config::SURFER_GATE_HEIGHT,
            &assets.surfer_gate_material,
        );
    }

    let finish_x = surfer.centerline(surfer.length);
    spawn_surfer_gate(
        commands,
        assets,
        finish_x,
        surfer.length,
        surfer.width * 0.9,
        config::SURFER_FINISH_HEIGHT,
        &assets.surfer_finish_material,
    );
}

/// Two posts and a lintel framing a gate opening across the river.
pub(crate) fn spawn_surfer_gate(
    commands: &mut Commands,
    assets: &LightcycleAssets,
    x: f32,
    z: f32,
    span: f32,
    height: f32,
    material: &Handle<StandardMaterial>,
) {
    let post = Vec3::new(0.18, height, 0.18);
    for side in [-1.0_f32, 1.0] {
        commands.spawn((
            LightcycleSceneRoot,
            Mesh3d(assets.unit_cube.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(Vec3::new(x + side * span, height * 0.5, z))
                .with_scale(post),
            Pickable::IGNORE,
        ));
    }
    commands.spawn((
        LightcycleSceneRoot,
        Mesh3d(assets.unit_cube.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(Vec3::new(x, height, z)).with_scale(Vec3::new(
            span * 2.0 + 0.36,
            0.18,
            0.18,
        )),
        Pickable::IGNORE,
    ));
}

/// The water ribbon, tessellated along the sim's centreline so the visual
/// banks match the gameplay banks exactly.
pub(crate) fn surfer_river_mesh(surfer: &SurferSim) -> Mesh {
    let start = -config::SURFER_RIVER_MARGIN;
    let end = surfer.length + config::SURFER_RIVER_MARGIN;
    let steps = ((end - start) / config::SURFER_RIVER_SAMPLE).ceil() as usize;
    let mut positions = Vec::with_capacity((steps + 1) * 2);
    let mut normals = Vec::with_capacity((steps + 1) * 2);
    for step in 0..=steps {
        let z = start + (end - start) * step as f32 / steps as f32;
        let center = surfer.centerline(z);
        positions.push([center - surfer.width, 0.0, z]);
        positions.push([center + surfer.width, 0.0, z]);
        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
    }
    let mut indices = Vec::with_capacity(steps * 6);
    for step in 0..steps as u32 {
        let a = step * 2;
        let b = step * 2 + 1;
        let c = (step + 1) * 2;
        let d = (step + 1) * 2 + 1;
        indices.extend_from_slice(&[a, c, b, b, c, d]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}
