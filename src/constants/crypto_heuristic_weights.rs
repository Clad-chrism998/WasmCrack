// Crypto score weighting.

// Weights for the three main factors.
pub const WEIGHT_ROT_AND_SH: f32 = 100.0;
pub const WEIGHT_XOR: f32 = 100.0;
pub const WEIGHT_COMPUTE: f32 = 1.0;

// Min total operations a func must have. If it does not have this many ops it is auto given a 0.0 score
pub const MIN_TOTAL_OPS: usize = 20;

// Cap for (computation / memory ops) ratio.
pub const COMPUTE_RATIO_CAP: f32 = 20.0;