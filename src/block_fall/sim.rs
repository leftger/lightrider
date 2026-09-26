//! Bevy-free Block Fall: falling indentation blocks cleared in lines.
//!
//! The board is `BLOCK_FALL_COLS` wide and `BLOCK_FALL_ROWS` tall on the X/Y plane.
//! Seven tetrominoes fall, rotate, and lock; a full line clears and everything
//! above drops. Clear the seeded target number of lines to win; lock a piece
//! above the rim to lose.

use crate::config;
use crate::minigame::{GameInput, GameSound, GameTick, SourceGameSim};
use crate::rng::Rng;

/// The seven tetrominoes as four cell offsets each.
pub const TETROMINOES: [[(i32, i32); 4]; 7] = [
    [(0, 0), (1, 0), (0, 1), (1, 1)], // O
    [(0, 0), (0, 1), (0, 2), (0, 3)], // I
    [(0, 0), (0, 1), (0, 2), (1, 2)], // L
    [(1, 0), (1, 1), (1, 2), (0, 2)], // J
    [(0, 0), (1, 0), (1, 1), (2, 1)], // S
    [(1, 0), (2, 0), (0, 1), (1, 1)], // Z
    [(0, 0), (1, 0), (2, 0), (1, 1)], // T
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockFallPhase {
    Falling,
    Won,
    Lost,
}

impl BlockFallPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Falling => "FALLING",
            Self::Won => "CLEARED",
            Self::Lost => "TOPPED OUT",
        }
    }
}

/// What one frame produced, for sound and labels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BlockFallEvents {
    pub rotated: bool,
    pub locked: bool,
    pub lines: u32,
    pub cleared: bool,
    pub lost: bool,
}

/// One input frame: edge left/right, edge rotate, held soft drop, edge hard drop.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BlockFallInput {
    pub slide: i32,
    pub rotate: bool,
    pub soft: bool,
    pub drop: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockFallSim {
    /// Row-major `cols * rows`; `None` is empty, `Some(colour)` a landed block.
    pub board: Vec<Option<u8>>,
    pub piece: usize,
    pub rotation: usize,
    pub x: i32,
    pub y: i32,
    pub lines: u32,
    pub phase: BlockFallPhase,
    pub input: BlockFallInput,
    fall_clock: f32,
    rng: Rng,
    seed: u64,
    file_lines: usize,
}

impl BlockFallSim {
    pub fn new(seed: u64, lines: usize) -> Self {
        let mut sim = Self {
            board: vec![None; config::arcade::BLOCK_FALL_COLS * config::arcade::BLOCK_FALL_ROWS],
            piece: 0,
            rotation: 0,
            x: config::arcade::BLOCK_FALL_COLS as i32 / 2 - 1,
            y: 0,
            lines: 0,
            phase: BlockFallPhase::Falling,
            input: BlockFallInput::default(),
            fall_clock: config::arcade::BLOCK_FALL_FALL_SECONDS,
            rng: Rng::from_state(seed | 1),
            seed,
            file_lines: lines,
        };
        sim.spawn_piece();
        sim
    }

    fn next_piece(&mut self) -> usize {
        (self.rng.unit() * TETROMINOES.len() as f32) as usize % TETROMINOES.len()
    }

    /// Latches a frame's input. `soft` is held; the rest are edges, so they
    /// accumulate until a step consumes them rather than being overwritten by a
    /// frame that ran no step.
    pub fn set_input(&mut self, slide: i32, rotate: bool, soft: bool, drop: bool) {
        if slide != 0 {
            self.input.slide = slide;
        }
        self.input.rotate |= rotate;
        self.input.soft = soft;
        self.input.drop |= drop;
    }

    fn cells(&self, piece: usize, rotation: usize, x: i32, y: i32) -> [(i32, i32); 4] {
        let mut cells = TETROMINOES[piece];
        for _ in 0..rotation {
            for cell in &mut cells {
                // Rotate 90° clockwise around the piece's origin.
                let (px, py) = *cell;
                *cell = (-py, px);
            }
        }
        cells.map(|(cx, cy)| (x + cx, y - cy))
    }

    fn fits(&self, piece: usize, rotation: usize, x: i32, y: i32) -> bool {
        self.cells(piece, rotation, x, y).iter().all(|&(cx, cy)| {
            cx >= 0
                && cx < config::arcade::BLOCK_FALL_COLS as i32
                && cy >= 0
                && cy < config::arcade::BLOCK_FALL_ROWS as i32
                && self.board[cx as usize + cy as usize * config::arcade::BLOCK_FALL_COLS].is_none()
        })
    }

    /// Places the next piece at the top of the board; losing happens here if
    /// the stack already reaches the spawn rows.
    fn spawn_piece(&mut self) -> bool {
        self.piece = self.next_piece();
        self.rotation = 0;
        self.x = config::arcade::BLOCK_FALL_COLS as i32 / 2 - 1;
        // Row 0 is the top; spawn one row of headroom below it so every piece
        // can still rotate without poking above the rim.
        self.y = TETROMINOES[self.piece]
            .iter()
            .map(|&(_, cy)| cy)
            .max()
            .unwrap_or(0)
            + 1;
        if !self.fits(self.piece, self.rotation, self.x, self.y) {
            self.phase = BlockFallPhase::Lost;
            return false;
        }
        true
    }

    /// The board with the falling piece overlaid, for the renderer.
    pub fn render_board(&self) -> Vec<Option<u8>> {
        let mut rendered = self.board.clone();
        for (cx, cy) in self.cells(self.piece, self.rotation, self.x, self.y) {
            if cx >= 0
                && cx < config::arcade::BLOCK_FALL_COLS as i32
                && cy >= 0
                && cy < config::arcade::BLOCK_FALL_ROWS as i32
            {
                rendered[cx as usize + cy as usize * config::arcade::BLOCK_FALL_COLS] =
                    Some(self.piece as u8);
            }
        }
        rendered
    }

    fn lock(&mut self) -> bool {
        for (cx, cy) in self.cells(self.piece, self.rotation, self.x, self.y) {
            if cy < 0 {
                self.phase = BlockFallPhase::Lost;
                return false;
            }
            self.board[cx as usize + cy as usize * config::arcade::BLOCK_FALL_COLS] =
                Some(self.piece as u8);
        }
        self.spawn_piece()
    }

    /// Clears full lines and drops everything above them.
    fn clear_lines(&mut self) -> u32 {
        let cols = config::arcade::BLOCK_FALL_COLS;
        let rows = config::arcade::BLOCK_FALL_ROWS;
        let mut cleared = 0;
        let mut row = rows as i32 - 1;
        while row >= 0 {
            let full = (0..cols).all(|col| self.board[col + row as usize * cols].is_some());
            if full {
                cleared += 1;
                for shift in (0..row).rev() {
                    for col in 0..cols {
                        self.board[col + (shift as usize + 1) * cols] =
                            self.board[col + shift as usize * cols];
                    }
                }
                for col in 0..cols {
                    self.board[col] = None;
                }
            } else {
                row -= 1;
            }
        }
        self.lines += cleared;
        cleared
    }

    pub fn update(&mut self, dt: f32) -> BlockFallEvents {
        let mut events = BlockFallEvents::default();
        let input = std::mem::take(&mut self.input);
        self.input.slide = 0;
        self.input.soft = false;
        if self.phase != BlockFallPhase::Falling {
            return events;
        }

        if input.slide != 0 && self.fits(self.piece, self.rotation, self.x + input.slide, self.y) {
            self.x += input.slide;
        }
        if input.rotate {
            let rotation = (self.rotation + 1) % 4;
            if self.fits(self.piece, rotation, self.x, self.y) {
                self.rotation = rotation;
                events.rotated = true;
            }
        }
        if input.drop {
            while self.fits(self.piece, self.rotation, self.x, self.y + 1) {
                self.y += 1;
            }
            self.fall_clock = 0.0;
        }

        let interval = if input.soft {
            config::arcade::BLOCK_FALL_FALL_SECONDS * 0.2
        } else {
            config::arcade::BLOCK_FALL_FALL_SECONDS
        };
        self.fall_clock -= dt;
        if self.fall_clock <= 0.0 {
            if self.fits(self.piece, self.rotation, self.x, self.y + 1) {
                self.y += 1;
            } else {
                if !self.lock() {
                    events.lost = true;
                    return events;
                }
                events.locked = true;
                let lines = self.clear_lines();
                events.lines = lines;
            }
            self.fall_clock = interval;
        }

        if self.lines >= config::arcade::BLOCK_FALL_TARGET_LINES {
            self.phase = BlockFallPhase::Won;
            events.cleared = true;
        }
        events
    }

    pub fn restart(&mut self) {
        *self = Self::new(self.seed, self.file_lines);
    }
}

impl SourceGameSim for BlockFallSim {
    fn tick(&mut self, dt: f32) -> GameTick {
        let events = self.update(dt);
        let mut tick = GameTick::default();
        if events.lines > 0 {
            tick.sound(GameSound::Portal);
        }
        if events.locked {
            tick.sound(GameSound::Beam);
        }
        if events.cleared {
            tick.sound(GameSound::Victory);
        }
        tick.cleared = events.cleared;
        if self.phase == BlockFallPhase::Lost {
            tick.lost = true;
            tick.label = Some("the stack of indentation".to_string());
        }
        tick
    }

    fn status_line(&self, ring: &str, language: &str, inner: &str) -> String {
        let mut status = format!(
            "BLOCK FALL {} / {} LINES | RING: {ring} | {language} | {inner}",
            self.lines,
            config::arcade::BLOCK_FALL_TARGET_LINES
        );
        status = format!("{status} | {}", self.phase.label());
        status
    }

    fn input(&mut self, input: &GameInput) {
        // A/D slides, W rotates, S soft-drops, Space hard-drops.
        self.set_input(
            input.right_x(),
            input.hop_z > 0,
            input.move_z > 0,
            input.action,
        );
    }

    fn restart(&mut self) {
        BlockFallSim::restart(self);
    }
}

#[cfg(test)]
mod tests {
    use super::{TETROMINOES, BlockFallPhase, BlockFallSim};
    use crate::config;

    fn sim() -> BlockFallSim {
        BlockFallSim::new(5, 200)
    }

    #[test]
    fn edge_presses_survive_a_frame_that_runs_no_step() {
        // Input is latched once per rendered frame but consumed on a 1/60
        // accumulator, so a press has to outlive the release that follows it
        // on a frame that ran no step.
        let mut game = sim();
        let x = game.x;
        game.set_input(1, true, false, false);
        game.set_input(0, false, false, false);
        let events = game.update(1.0 / 60.0);
        assert_eq!(game.x, x + 1, "the slide was swallowed");
        assert!(events.rotated, "the rotate was swallowed");
        assert_eq!(game.rotation, 1);
    }

    #[test]
    fn the_same_seed_builds_the_same_board() {
        assert_eq!(sim(), sim());
    }

    #[test]
    fn all_seven_pieces_fit_at_spawn() {
        for (piece, offsets) in TETROMINOES.iter().enumerate() {
            let game = sim();
            let y = offsets.iter().map(|&(_, cy)| cy).max().unwrap_or(0) + 1;
            assert!(
                game.fits(piece, 0, game.x, y),
                "piece {piece} should fit at its spawn row"
            );
        }
    }

    #[test]
    fn sliding_and_rotating_move_the_piece() {
        let mut game = sim();
        let x = game.x;
        game.set_input(1, false, false, false);
        game.update(1.0 / 60.0);
        assert_eq!(game.x, x + 1);
        game.set_input(0, true, false, false);
        let events = game.update(1.0 / 60.0);
        assert!(events.rotated);
        assert_eq!(game.rotation, 1);
    }

    #[test]
    fn a_full_line_clears() {
        let mut game = sim();
        let cols = config::arcade::BLOCK_FALL_COLS;
        for col in 0..cols {
            game.board[col + (config::arcade::BLOCK_FALL_ROWS - 1) * cols] = Some(1);
        }
        assert_eq!(game.clear_lines(), 1);
        assert_eq!(game.lines, 1);
        assert!((0..cols).all(|col| game.board[col].is_none()));
    }

    #[test]
    fn hard_drop_locks_the_piece() {
        let mut game = sim();
        game.set_input(0, false, false, true);
        let events = game.update(1.0 / 60.0);
        assert!(events.locked);
        assert!(game.board.iter().any(|cell| cell.is_some()));
    }

    #[test]
    fn reaching_the_target_wins() {
        let mut game = sim();
        game.lines = config::arcade::BLOCK_FALL_TARGET_LINES;
        let events = game.update(1.0 / 60.0);
        assert!(events.cleared);
        assert_eq!(game.phase, BlockFallPhase::Won);
    }

    #[test]
    fn restarting_relays_the_board() {
        let mut game = sim();
        game.board.clear();
        game.restart();
        assert_eq!(game.board, sim().board);
        assert_eq!(game.phase, BlockFallPhase::Falling);
    }
}
