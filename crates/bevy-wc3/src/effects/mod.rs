//! Shared mechanics for model particles, quad particles, and ribbon trails.
//! Emitter modules retain emission, expiry, capacity, and connectivity policy.
//!
//! Simulation helpers work with both entity-backed PREM particles and immutable
//! PRE2/ribbon records. GPU helpers serve only PRE2 and ribbons: each renderer
//! still owns shader layouts, pipeline specialization, sorting, and pass order.
//! Transform query adapters remain at their original schedule positions, using
//! local poses before propagation and propagated world poses afterward.
pub(crate) mod render;
pub(crate) mod simulation;

pub(crate) mod particle_emitter;
pub(crate) mod particle_emitter2;
pub(crate) mod records;
pub(crate) mod ribbon_emitter;
