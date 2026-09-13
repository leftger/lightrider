//! Small helpers for the square grids the sims run on.
//!
//! The grid maths that all of them need used to be written out per module; this
//! is where it lives now.

/// Chebyshev distance between two cells: the steps a king would need, which on a
/// grid where diagonal moves are one step is the distance that matters.
pub fn chebyshev(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

#[cfg(test)]
mod tests {
    use super::chebyshev;

    #[test]
    fn distance_is_the_longest_axis() {
        assert_eq!(chebyshev((0, 0), (0, 0)), 0);
        assert_eq!(chebyshev((0, 0), (3, 0)), 3);
        assert_eq!(chebyshev((0, 0), (0, -4)), 4);
        assert_eq!(chebyshev((0, 0), (3, 4)), 4, "the longer axis wins");
        assert_eq!(
            chebyshev((5, 5), (1, 9)),
            4,
            "and the order does not matter"
        );
        assert_eq!(chebyshev((1, 9), (5, 5)), 4);
    }
}
