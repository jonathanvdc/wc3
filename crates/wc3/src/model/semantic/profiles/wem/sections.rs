//! Checked mesh-local section identities describe authored polygon grouping.
use super::{Availability, Metadata};
use crate::model::semantic::{Bounds, Link, MaterialSlotId, NodeId, ValidateLinks, Values};
use bitfield::bitfield;
use std::borrow::Cow;
/// A section identity scoped to its owning geometry.
///
/// Only a geometry can create these handles. A handle from another geometry
/// cannot be used as a polygon assignment or repair-history reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SectionId {
    pub(super) scope: u64,
    pub(super) index: u32,
}
bitfield! {
    /// Draw-group behavior, including undeclared option bits.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct SectionFlags(u32);
    /// All authored option bits.
    pub bits,_:31,0;
    /// Suppresses this group from drawing.
    pub hidden,set_hidden:0;
    /// Participates in cloth simulation.
    pub cloth_simulated,set_cloth_simulated:1;
    /// Receives cloth deformation.
    pub cloth_influenced,set_cloth_influenced:2;
    /// Uses billboard behavior.
    pub billboard,set_billboard:3;
    /// Receives projected shadows.
    pub projected_shadow,set_projected_shadow:4;
}
impl SectionFlags {
    /// Retains all named and unnamed option bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
/// An independently authored polygon group; polygons need not be contiguous.
#[derive(Clone, Debug)]
pub struct Section<'a> {
    /// Producer-defined family availability for this polygon group.
    pub availability: Availability,
    /// Display name.
    pub name: Cow<'a, str>,
    /// Appearance slot assigned to this group.
    pub slot: Option<MaterialSlotId>,
    /// Optional node binding all vertices rigidly.
    pub rigid_node: Option<NodeId>,
    /// Native selection-group identity interpreted by external preparation tools.
    pub selection_group: u16,
    /// Group draw and deformation behavior.
    pub flags: SectionFlags,
    /// Conservative bounds of this group.
    pub bounds: Bounds,
    /// Ordered authoring properties associated with this group.
    pub metadata: Metadata<'a>,
}
impl ValidateLinks for Section<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Some(slot) = self.slot {
            visitor(Link::AppearanceSlot(slot));
        }
        self.rigid_node.visit_links(visitor);
        if self.bounds.validate().is_err() {
            visitor(Link::InvalidValue);
        }
    }
}
/// One duplication retained from mesh repair.
#[derive(Clone, Debug)]
pub struct VertexSplit {
    /// Vertex before duplication.
    pub original: u32,
    /// Vertex created by duplication.
    pub created: u32,
}
/// An earlier polygon removed by repair.
#[derive(Clone, Debug)]
pub struct RemovedFace<'a> {
    /// Ordered geometric vertex identities.
    pub corners: Values<'a, u32>,
    /// Original polygon-order position.
    pub original_order: u32,
    /// Original draw group, when one was assigned.
    pub section: Option<SectionId>,
}
/// Independently editable geometric repair history.
#[derive(Clone, Debug, Default)]
pub struct RepairHistory<'a> {
    /// Recorded vertex duplications in application order.
    pub splits: Values<'a, VertexSplit>,
    /// Removed polygon definitions.
    pub removed_faces: Vec<RemovedFace<'a>>,
}
