//! Git-time-machine constants: how much history the run keeps.

/// Directories remembered for rewind / fast-forward.
pub const HISTORY_LIMIT: usize = 32;

/// How long a rewind / fast-forward notice stays on the status line.
pub const HISTORY_NOTICE_SECONDS: f32 = 2.5;
