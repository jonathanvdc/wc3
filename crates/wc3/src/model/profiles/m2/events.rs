//! Complete M2 event-source definitions.
use crate::model::common::IntoOwned;
use crate::model::{EventSourceSchema, Link, NodeId, TimeSource, ValidateLinks};

/// An anchored action whose occurrences share one evaluation clock.
#[derive(Clone, Debug, PartialEq, IntoOwned)]
pub struct EventSource {
    /// Hierarchy anchor of the action.
    pub node: NodeId,
    /// Clock on which all occurrence times are authored.
    pub clock: TimeSource,
    /// Four-byte action code.
    pub identifier: [u8; 4],
    /// External game-data identity.
    pub data: u32,
}
impl EventSourceSchema for EventSource {
    fn clock(&self) -> TimeSource {
        self.clock
    }
}
impl ValidateLinks for EventSource {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.node.visit_links(visitor);
        self.clock.visit_links(visitor);
    }
}
