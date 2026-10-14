//! Playback state and recorded simulation settings attached to actual animation owners.
mod physics;
pub(crate) mod validation;
mod values;
use super::Metadata;
use crate::model::semantic::{Bounds, ControllerId, Link, ValidateLinks};
use bitfield::bitfield;
pub use physics::*;
pub use values::*;

bitfield! {
    /// Playback state independent of the clip's looping behavior.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct ClipFlags(u32);
    /// Returns the retained bit pattern.
    pub bits, _:31, 0;
    /// Starts playback automatically.
    pub auto_play, set_auto_play:0;
    /// Keeps the playback program active persistently.
    pub persistent, set_persistent:1;
    /// Uses the world playback clock.
    pub world_clocked, set_world_clocked:2;
}
impl ClipFlags {
    /// Retains unnamed playback bits alongside known behavior.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}

/// One named selection applied with its independently authored combination priority.
#[derive(Clone, Copy, Debug)]
pub struct PlaybackSelection {
    /// Actual model-owned channel selection.
    pub selection: SelectionId,
    /// Ordering priority for this selection within the clip.
    pub priority: i32,
}
/// Additional WEM playback declarations belonging to one clip.
#[derive(Clone, Debug, Default)]
pub struct ClipControls<'a> {
    /// Actual ordered animation programmes owned by this model.
    pub programs: Vec<ControllerId>,
    /// Named channel selections used by this playback clip.
    pub selections: Vec<PlaybackSelection>,
    /// Timed occurrences with exact unsigned application payloads.
    pub events: Vec<Event<'a>>,
    /// Automatic, persistent, and world-clock playback state.
    pub flags: ClipFlags,
    /// Combination convention used by the program.
    pub read_rule: Option<ReadRule>,
    /// Authored animated bounding volume, when declared.
    pub bounds: Option<Bounds>,
    /// Optional recorded simulation settings and editable pre-bake programmes.
    pub physics: Option<RecordedPhysics<'a>>,
    /// Ordered application-specific playback properties.
    pub metadata: Metadata<'a>,
}
impl ValidateLinks for ClipControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.programs.visit_links(visitor);
        self.events.visit_links(visitor);
        self.physics.visit_links(visitor);
        if self.bounds.is_some_and(|bounds| bounds.validate().is_err()) {
            visitor(Link::InvalidValue);
        }
    }
}

/// Additional authoring declarations belonging to one actual animation layer.
#[derive(Clone, Debug, Default)]
pub struct ControllerControls<'a> {
    /// Actual typed program selecting model-owned channel declarations.
    pub tracks: Vec<AnyTrack<'a>>,
    /// Ordered layer properties without an imposed unique-name rule.
    pub metadata: Metadata<'a>,
}
impl ValidateLinks for ControllerControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.tracks.visit_links(visitor);
    }
}

/// Typed declaration identities, initialization, and controller program curves.
pub mod channels;
pub use channels::{
    AnyChannelId, AnyDeclaration, AnyTrack, ChannelId, ChannelObject, ChannelValue, Channels,
    Declaration, FrameComponent, MaterialMount, NodeOperation, Operation, PhysicsControl,
    Selection, SelectionId, SelectionObject, Selections, StageSourceControl, Target, Track,
};

mod events;
pub use events::Event;
