//! ANCHOR core: event log, safety kernel A, numerics, personal model, detection.
//! Design: DESIGN_PAPER.md; decisions: docs/adr/.
#![forbid(unsafe_code)]

pub mod baseline;
pub mod detect;
pub mod engine;
pub mod kernel;
pub mod log;
pub mod outbox;
pub mod numerics;
pub mod rng;
pub mod session;
pub mod synth;
