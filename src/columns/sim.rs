//! Bevy-free Columns: falling gem stacks matched in a vertical well.
//!
//! The well is `COLUMNS_COLS` wide and `COLUMNS_ROWS` tall on the X/Y plane.
//! A stack of three gems falls; slide it, rotate its colours, and drop it.
//! Three or more matching gems in a run clear, the rest fall, and clearing the
//! last gem wins. Landing with a gem above the rim loses.

use crate::config;
use crate::minigame::{GameInput, GameSound, GameTick, SourceGameSim};
use crate::rng::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnsPhase {
    Falling,
    Won,
    Lost,
}

impl ColumnsPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Falling => "FALLING",
            Self::Won => "CLEARED",
            Self::Lost => "BURIED",
        }
    }
}

/// What one frame produced, for sound and labels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ColumnsEvents {
    pub rotated: bool,
    pub landed: bool,
    pub matched: u32,
    pub cleared: bool,
    pub lost: bool,
}

/// One input frame: edge slide, edge rotate, edge hard drop.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ColumnsInput {
    pub slide: i32,
    pub rotate: bool,
    pub drop: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnsSim {
    /// Row-major `cols * rows`; `None` is empty, `Some(colour)` a landed gem.
    pub board: Vec<Option<u8>>,
    /// Column the falling piece is over.
    pub col: i32,
    /// Colour of each of the three falling gems, bottom first.
    pub piece: [u8; 3],
    /// Row of the bottom gem of the falling piece.
    pub bottom: i32,
    pub phase: ColumnsPhase,
    pub score: u32,
    pub input: ColumnsInput,
    fall_clock: f32,
    rng: Rng,
    seed: u64,
    lines: usize,
}

impl ColumnsSim {
    pub fn new(seed: u64, lines: usize) -> Self {
        let mut sim = Self {
            board: vec![None; config::arcade::COLUMNS_COLS * config::arcade::COLUMNS_ROWS],
            col: config::arcade::COLUMNS_COLS as i32 / 2,
            piece: [0, 1, 2],
            bottom: 2,
            phase: ColumnsPhase::Falling,
            score: 0,
            input: ColumnsInput::default(),
            fall_clock: config::arcade::COLUMNS_FALL_SECONDS,
            rng: Rng::from_state(seed | 1),
            seed,
            lines,
        };
        // Seed a few rows so there is something to match right away, shaped by
        // the file's length so longer files start a little busier.
        let rows = config::arcade::COLUMNS_SEED_ROWS + lines.min(400) / 130;
        for _ in 0..rows {
            sim.seed_row();
        }
        sim.piece = sim.next_piece();
        sim
    }

    fn next_piece(&mut self) -> [u8; 3] {
        let colours = config::arcade::COLUMNS_GEM_COLORS as u8;
        [
            (self.rng.unit() * colours as f32) as u8 % colours,
            (self.rng.unit() * colours as f32) as u8 % colours,
            (self.rng.unit() * colours as f32) as u8 % colours,
        ]
    }

    /// Adds one seeded row at the bottom, pushing the rest toward the top.
    fn seed_row(&mut self) {
        let cols = config::arcade::COLUMNS_COLS;
        let rows = config::arcade::COLUMNS_ROWS;
        for row in 1..rows {
            for col in 0..cols {
                self.board[col + (row - 1) * cols] = self.board[col + row * cols];
            }
        }
        for col in 0..cols {
            self.board[col + (rows - 1) * cols] = Some(
                (self.rng.unit() * config::arcade::COLUMNS_GEM_COLORS as f32) as u8
                    % config::arcade::COLUMNS_GEM_COLORS as u8,
            );
        }
    }

    /// Latches a frame's input. Every field is an edge, so each accumulates
    /// until a step consumes it rather than being overwritten by a frame that
    /// ran no step.
    pub fn set_input(&mut self, slide: i32, rotate: bool, drop: bool) {
        if slide != 0 {
            self.input.slide = slide;
        }
        self.input.rotate |= rotate;
        self.input.drop |= drop;
    }

    fn cell(&self, col: i32, row: i32) -> Option<u8> {
        if col < 0
            || col >= config::arcade::COLUMNS_COLS as i32
            || row < 0
            || row >= config::arcade::COLUMNS_ROWS as i32
        {
            return None;
        }
        self.board[col as usize + row as usize * config::arcade::COLUMNS_COLS]
    }

    fn piece_cells(&self) -> [(i32, i32); 3] {
        [
            (self.col, self.bottom),
            (self.col, self.bottom - 1),
            (self.col, self.bottom - 2),
        ]
    }

    fn piece_landed(&self) -> bool {
        self.bottom + 1 >= config::arcade::COLUMNS_ROWS as i32
            || self.cell(self.col, self.bottom + 1).is_some()
    }

    fn land(&mut self) -> bool {
        for (index, (col, row)) in self.piece_cells().into_iter().enumerate() {
            self.board[col as usize + row as usize * config::arcade::COLUMNS_COLS] =
                Some(self.piece[index]);
        }
        self.col = config::arcade::COLUMNS_COLS as i32 / 2;
        self.bottom = 2;
        self.piece = self.next_piece();
        // The stack reached the spawn rows: the next piece has nowhere to go.
        if self
            .piece_cells()
            .iter()
            .any(|&(col, row)| self.cell(col, row).is_some())
        {
            self.phase = ColumnsPhase::Lost;
            return false;
        }
        true
    }

    /// The well with the falling stack overlaid, for the renderer.
    pub fn render_board(&self) -> Vec<Option<u8>> {
        let mut rendered = self.board.clone();
        for (index, &(col, row)) in self.piece_cells().iter().enumerate() {
            if row >= 0 && row < config::arcade::COLUMNS_ROWS as i32 {
                rendered[col as usize + row as usize * config::arcade::COLUMNS_COLS] =
                    Some(self.piece[index]);
            }
        }
        rendered
    }

    /// Removes every run of three or more, lets the rest fall, and repeats.
    fn resolve_matches(&mut self) -> u32 {
        let mut total = 0;
        loop {
            let cleared = self.clear_matches();
            if cleared == 0 {
                return total;
            }
            total += cleared;
            self.score += cleared * 10;
            self.apply_gravity();
        }
    }

    fn clear_matches(&mut self) -> u32 {
        let cols = config::arcade::COLUMNS_COLS;
        let rows = config::arcade::COLUMNS_ROWS;
        let mut remove = vec![false; cols * rows];
        for row in 0..rows {
            for col in 0..cols {
                let Some(colour) = self.board[col + row * cols] else {
                    continue;
                };
                for (dx, dy) in [(1, 0), (0, 1), (1, 1), (1, -1)] {
                    let mut run = vec![(col, row)];
                    let (mut x, mut y) = (col as i32 + dx, row as i32 + dy);
                    while x >= 0
                        && y >= 0
                        && (x as usize) < cols
                        && (y as usize) < rows
                        && self.cell(x, y) == Some(colour)
                    {
                        run.push((x as usize, y as usize));
                        x += dx;
                        y += dy;
                    }
                    if run.len() >= 3 {
                        for (x, y) in run {
                            remove[x + y * cols] = true;
                        }
                    }
                }
            }
        }
        for (index, gone) in remove.iter().enumerate() {
            if *gone {
                self.board[index] = None;
            }
        }
        // Count gems removed, not runs.
        remove.iter().filter(|gone| **gone).count() as u32
    }

    fn apply_gravity(&mut self) {
        let cols = config::arcade::COLUMNS_COLS;
        let rows = config::arcade::COLUMNS_ROWS;
        for col in 0..cols {
            let mut write = rows as i32 - 1;
            for row in (0..rows as i32).rev() {
                if let Some(colour) = self.board[col + row as usize * cols] {
                    self.board[col + row as usize * cols] = None;
                    self.board[col + write as usize * cols] = Some(colour);
                    write -= 1;
                }
            }
        }
    }

    /// Advances one frame.
    pub fn update(&mut self, dt: f32) -> ColumnsEvents {
        let mut events = ColumnsEvents::default();
        let input = std::mem::take(&mut self.input);
        self.input.slide = 0;
        if self.phase != ColumnsPhase::Falling {
            return events;
        }

        if input.slide != 0 {
            let next = self.col + input.slide;
            if next >= 0 && next < config::arcade::COLUMNS_COLS as i32 {
                self.col = next;
            }
        }
        if input.rotate {
            self.piece.rotate_right(1);
            events.rotated = true;
        }
        if input.drop {
            while !self.piece_landed() {
                self.bottom += 1;
            }
            self.fall_clock = 0.0;
        }

        self.fall_clock -= dt;
        if self.fall_clock <= 0.0 {
            if self.piece_landed() {
                if !self.land() {
                    events.lost = true;
                    return events;
                }
                events.landed = true;
                let matched = self.resolve_matches();
                events.matched = matched;
                self.fall_clock = config::arcade::COLUMNS_FALL_SECONDS;
            } else {
                self.bottom += 1;
                self.fall_clock = config::arcade::COLUMNS_FALL_SECONDS;
            }
        }

        if self.board.iter().all(|cell| cell.is_none()) {
            self.phase = ColumnsPhase::Won;
            events.cleared = true;
        }
        events
    }

    /// Restarts from the same seed, as `R` does.
    pub fn restart(&mut self) {
        *self = Self::new(self.seed, self.lines);
    }
}

impl SourceGameSim for ColumnsSim {
    fn tick(&mut self, dt: f32) -> GameTick {
        let events = self.update(dt);
        let mut tick = GameTick::default();
        if events.matched > 0 {
            tick.sound(GameSound::Portal);
        }
        if events.landed {
            tick.sound(GameSound::Beam);
        }
        if events.cleared {
            tick.sound(GameSound::Victory);
        }
        tick.cleared = events.cleared;
        if self.phase == ColumnsPhase::Lost {
            tick.lost = true;
            tick.label = Some("the gem well".to_string());
        }
        tick
    }

    fn status_line(&self, ring: &str, language: &str, inner: &str) -> String {
        let mut status = format!(
            "COLUMNS {} | RING: {ring} | {language} | {inner}",
            self.score
        );
        status = format!("{status} | {}", self.phase.label());
        status
    }

    fn input(&mut self, input: &GameInput) {
        // A/D slides the piece, W rotates, Space hard-drops.
        self.set_input(input.right_x(), input.hop_z > 0, input.action);
    }
}

#[cfg(test)]
mod tests {
    use super::{ColumnsPhase, ColumnsSim};
    use crate::config;

    fn sim() -> ColumnsSim {
        ColumnsSim::new(5, 200)
    }

    #[test]
    fn edge_presses_survive_a_frame_that_runs_no_step() {
        // Input is latched once per rendered frame but consumed on a 1/60
        // accumulator, so a press has to outlive the release that follows it
        // on a frame that ran no step.
        let mut well = sim();
        let start = well.col;
        let colours = well.piece;
        well.set_input(1, true, false);
        well.set_input(0, false, false);
        let events = well.update(1.0 / 60.0);
        assert_eq!(well.col, start + 1, "the slide was swallowed");
        assert!(events.rotated, "the rotate was swallowed");
        assert_ne!(well.piece, colours);
    }

    #[test]
    fn the_same_seed_builds_the_same_well() {
        assert_eq!(sim(), sim());
    }

    #[test]
    fn sliding_moves_the_piece_and_rotating_cycles_colours() {
        let mut well = sim();
        let start = well.col;
        let colours = well.piece;
        well.set_input(1, false, false);
        well.update(1.0 / 60.0);
        assert_eq!(well.col, start + 1);
        well.set_input(0, true, false);
        let events = well.update(1.0 / 60.0);
        assert!(events.rotated);
        assert_ne!(well.piece, colours);
    }

    #[test]
    fn a_hard_drop_lands_the_piece() {
        let mut well = sim();
        well.set_input(0, false, true);
        let events = well.update(1.0 / 60.0);
        assert!(events.landed, "hard drop should land the piece");
        assert!(well.board.iter().any(|cell| cell.is_some()));
    }

    #[test]
    fn matching_clears_gems_and_gravity_fills_in() {
        let mut well = sim();
        well.board = vec![None; config::arcade::COLUMNS_COLS * config::arcade::COLUMNS_ROWS];
        // Three reds in a column.
        well.board[0] = Some(0);
        well.board[config::arcade::COLUMNS_COLS] = Some(0);
        well.board[2 * config::arcade::COLUMNS_COLS] = Some(0);
        well.board[3 * config::arcade::COLUMNS_COLS] = Some(1); // above them, must fall
        let cleared = well.clear_matches();
        assert_eq!(cleared, 3);
        well.apply_gravity();
        assert_eq!(
            well.board[(config::arcade::COLUMNS_ROWS - 1) * config::arcade::COLUMNS_COLS],
            Some(1),
            "the gem above should fall to the floor"
        );
    }

    #[test]
    fn clearing_the_well_wins() {
        let mut well = sim();
        well.board = vec![None; config::arcade::COLUMNS_COLS * config::arcade::COLUMNS_ROWS];
        let events = well.update(1.0 / 60.0);
        assert!(events.cleared);
        assert_eq!(well.phase, ColumnsPhase::Won);
    }

    #[test]
    fn restarting_relays_the_well() {
        let mut well = sim();
        well.board.clear();
        well.restart();
        assert_eq!(well.board, sim().board);
        assert_eq!(well.phase, ColumnsPhase::Falling);
    }
}
