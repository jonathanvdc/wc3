//! Profile-defined actions and their scheduled occurrences.
use crate::model::common::IntoOwned;
use crate::model::profiles::{m2, mdlx, Unavailable};
use crate::model::{Link, TimeSource, ValidateLinks};
use std::fmt::Debug;

/// Evaluation clock of a complete profile-specific event source.
///
/// Implementations must visit every anchor and shared-clock reference through
/// `ValidateLinks`, including references of sources with no occurrences.
pub trait EventSourceSchema: Clone + Debug + Send + Sync + ValidateLinks {
    /// Clock shared by all occurrences of this source.
    fn clock(&self) -> TimeSource;
}

/// Complete event-source vocabulary retained when widening native models.
#[derive(Clone, Debug, IntoOwned)]
pub enum EventSource<'a> {
    /// Warcraft III named action at a hierarchy anchor.
    Mdlx(mdlx::EventSource<'a>),
    /// M2 action code and external game-data identity at a hierarchy anchor.
    M2(m2::EventSource),
}
impl EventSourceSchema for EventSource<'_> {
    fn clock(&self) -> TimeSource {
        match self {
            Self::Mdlx(value) => value.clock,
            Self::M2(value) => value.clock,
        }
    }
}
impl ValidateLinks for EventSource<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Mdlx(value) => value.visit_links(visitor),
            Self::M2(value) => value.visit_links(visitor),
        }
    }
}
impl EventSourceSchema for Unavailable {
    fn clock(&self) -> TimeSource {
        match *self {}
    }
}
