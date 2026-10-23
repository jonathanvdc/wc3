//! Complete MDLX event-source definitions.
use crate::model::common::IntoOwned;
use crate::model::{EventSourceSchema, Link, NodeId, TimeSource, ValidateLinks};
use std::borrow::Cow;

/// An anchored action whose occurrences share one evaluation clock.
///
/// Source definitions belong to their profile's vocabulary:
/// ```compile_fail
/// use wc3::model::{Model, TimeSource};
/// use wc3::model::profiles::{m2, Mdlx};
/// let mut draft = Model::<Mdlx>::builder("");
/// let node = draft.add_named_node("anchor").unwrap();
/// draft.add_event_source(m2::EventSource {
///     node, clock: TimeSource::Animation, identifier: *b"$SND", data: 1,
/// });
/// ```
#[derive(Clone, Debug, PartialEq, IntoOwned)]
pub struct EventSource<'a> {
    /// Hierarchy anchor of the action.
    pub node: NodeId,
    /// Clock on which all occurrence times are authored.
    pub clock: TimeSource,
    /// Authored action name.
    pub name: Cow<'a, str>,
}
impl EventSourceSchema for EventSource<'_> {
    fn clock(&self) -> TimeSource {
        self.clock
    }
}
impl ValidateLinks for EventSource<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.node.visit_links(visitor);
        self.clock.visit_links(visitor);
    }
}
