//! Event occurrence payloads retain the editable document's exact unsigned value.
use crate::model::semantic::{Link, NodeId, Time, ValidateLinks};
use std::borrow::Cow;
/// One timed occurrence; the node need not have an event actor attached.
#[derive(Clone, Debug)]
pub struct Event<'a> {
    /// Time relative to the owning clip's beginning.
    pub time: Time,
    /// Actual hierarchy owner at which the event occurs.
    pub node: Option<NodeId>,
    /// Event name interpreted by the consuming application.
    pub name: Cow<'a, str>,
    /// Source-specific unsigned event payload.
    pub value: u32,
}
impl ValidateLinks for Event<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.node.visit_links(visitor);
    }
}
