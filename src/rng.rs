//! The small pseudo-random generator the minigames share.
//!
//! Every sim used to carry its own copy of the same linear congruential step, in
//! one of two shapes: a `unit(&mut self)` method over its own `u64`, or a
//! `unit(rng: &mut u64)` free function. Fourteen copies of one idea, all
//! identical down to the constants. This is the one copy.
//!
//! The sequence is unchanged from those copies, so a file that already fielded a
//! particular asteroid layout still fields exactly the same one.

/// A 64-bit linear congruential generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Seeds the generator. Any state is valid; the sequence depends only on it.
    pub fn from_state(state: u64) -> Self {
        Self { state }
    }

    /// The next value in the sequence, in `0.0..1.0`.
    pub fn unit(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 40) as f32 / (1_u32 << 24) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::Rng;

    #[test]
    fn the_same_state_gives_the_same_sequence() {
        let mut first = Rng::from_state(1234);
        let mut second = Rng::from_state(1234);
        for _ in 0..64 {
            assert_eq!(first.unit(), second.unit());
        }
    }

    #[test]
    fn different_states_diverge() {
        let mut first = Rng::from_state(1);
        let mut second = Rng::from_state(2);
        assert_ne!(first.unit(), second.unit());
    }

    #[test]
    fn unit_stays_inside_its_range() {
        let mut rng = Rng::from_state(0xdead_beef);
        for _ in 0..10_000 {
            let value = rng.unit();
            assert!((0.0..1.0).contains(&value), "unit out of range: {value}");
        }
    }

    /// The sequence is part of the game: a folder that already fielded one
    /// asteroid layout must keep fielding exactly that one.
    #[test]
    fn the_sequence_is_pinned() {
        let mut rng = Rng::from_state(1);
        let first_six: Vec<u32> = (0..6).map(|_| (rng.unit() * 1_000_000.0) as u32).collect();
        assert_eq!(
            first_six,
            vec![423_209, 509_407, 648_359, 382_863, 795_447, 500_511]
        );
    }
}
