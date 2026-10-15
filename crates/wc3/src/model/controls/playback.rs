//! Playback controls and structured event and extent animation semantics.
use crate::model::animation::programmed::AnimationControl;
use crate::model::values::programmed::SequenceFlags;
use crate::model::{
    Bounds, ControllerId, EventId, FeatureId, Link, Property, PropertyTarget, Time, ValidateLinks,
};
use bitfield::bitfield;
use std::borrow::Cow;

/// Native sequence authoring behavior supplementing a logical clip.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SequenceControls<'a> {
    /// Source authoring identifier, independent of graph clip identity.
    pub source_id: i32,
    /// Producer-defined sequence number, independent of graph identity and controller selection.
    pub sequence_number: i32,
    /// Unresolved authoring control present in legacy sequence declarations.
    pub legacy_control: Option<u32>,
    /// Native playback controls; looping remains authoritative on the clip.
    pub flags: SequenceFlags,
    /// Authored timeline origin used by related event and controller clocks.
    pub timeline_origin: Time,
    /// Expected movement speed during this playback interval.
    pub movement_speed: f32,
    /// Relative probability of choosing this interval.
    pub frequency: u32,
    /// Native replay interval controls.
    pub replay_range: [u32; 2],
    /// Transition blending duration.
    pub blend_duration: Time,
    /// Authored spatial bounds during the sequence.
    pub bounds: Bounds,
    /// Source animation-set identities participating in this playback interval.
    pub animation_sets: Vec<u8>,
    /// Named controller selection associated with this playback interval, when declared.
    pub selection: Option<ControllerSelection<'a>>,
}

/// One independently authored named selection of controller programs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ControllerSelection<'a> {
    /// Authored selection name; it need not match a playback clip's display name.
    pub name: Cow<'a, str>,
    /// Programs evaluated by this selection, in authored order.
    pub controllers: Vec<ControllerId>,
}
impl ValidateLinks for ControllerSelection<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.controllers.visit_links(visitor);
    }
}
/// A layered controller program and its state and block behavior.
#[derive(Clone, Debug, PartialEq)]
pub struct ControllerProgram<B, U> {
    /// Native animation-state property identities selected by this controller.
    pub state_properties: Vec<u32>,
    /// Authored key blocks; typed curves themselves belong to this controller program.
    pub blocks: Vec<B>,
    /// Known typed keys without an available property header or active property selection.
    pub unbound: Vec<U>,
}

bitfield! {
    /// Native key-block controls whose individual meanings are unresolved.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct BlockFlags(u32);
    /// Returns the complete mask without discarding unknown bits.
    pub bits, _:31,0;
}
impl BlockFlags {
    /// Constructs the complete behavior mask, retaining unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}

/// Storage precision of a logically typed native key family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyEncoding {
    /// Structured engine command occurrences.
    Event,
    /// Two-component floating-point values.
    Vector2,
    /// Three-component floating-point values.
    Vector3,
    /// Quaternion rotation values.
    Quaternion,
    /// Four-channel colors encoded with eight-bit channels.
    Color,
    /// Scalar floating-point values.
    Float,
    /// Unsigned eight-bit integer values.
    Byte,
    /// Signed sixteen-bit integer values.
    Signed16,
    /// Unsigned sixteen-bit integer values.
    Unsigned16,
    /// Signed thirty-two-bit integer values.
    Signed32,
    /// Unsigned thirty-two-bit integer values.
    Unsigned32,
    /// Thirty-two-bit behavior masks.
    Flags,
    /// Structured minimum, maximum, and sphere-radius values.
    Extent,
}

/// Behavioral controls of one animated property block.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockControls {
    /// Source animation identity; shared identities can bind several properties.
    pub source_id: u32,
    /// Logical destinations driven by this key block.
    pub targets: Vec<PropertyTarget>,
    /// Exact controller-owned occurrences selected by a structured event block.
    pub events: Vec<EventId>,
    /// Extent feature driven by a structured bounds block.
    pub extents: Vec<FeatureId>,
    /// Integer precision and structured family used by the native animation system.
    pub encoding: KeyEncoding,
    /// Complete key-block behavior mask.
    pub flags: BlockFlags,
    /// Authored end of the key block on the model timeline.
    pub end: Time,
}

/// A timed game event command; the common event occurrence references its feature.
#[derive(Clone, Debug, PartialEq)]
pub struct EventCommand<'a> {
    /// Native command category.
    pub event_type: u32,
    /// Authored event-space transformation.
    pub transform: [[f32; 4]; 4],
    /// Command options interpreted by the consuming engine.
    pub options: Cow<'a, str>,
    /// Render-to-texture channel selected by the command, when supplied.
    pub render_target_channel: Option<u32>,
    /// Additional game-defined command parameter, when supplied.
    pub parameter: Option<u32>,
    /// Unresolved native event control word.
    pub control: u32,
}

/// Three typed properties jointly animate one authored extent.
#[derive(Clone, Debug, PartialEq)]
pub struct ExtentBinding {
    /// Detail level owning a declared mesh extent; absence denotes an unbound controller property.
    pub detail_level: Option<u32>,
    /// Static authored extent before animation evaluates.
    pub initial: Bounds,
    /// Extent used when the native animation controller resets.
    pub reset: Bounds,
    /// Source property identity and controller behavior.
    pub control: AnimationControl,
    /// Animated minimum corner of the spatial bounds.
    pub minimum: Property<[f32; 3]>,
    /// Animated maximum corner of the spatial bounds.
    pub maximum: Property<[f32; 3]>,
    /// Animated sphere radius associated with these bounds.
    pub radius: Property<f32>,
}

impl ValidateLinks for EventCommand<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self
            .transform
            .iter()
            .flatten()
            .any(|value| !value.is_finite())
        {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for ExtentBinding {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self.initial.validate().is_err() || self.reset.validate().is_err() {
            visitor(Link::InvalidValue);
        }
        self.minimum.visit_links(visitor);
        self.maximum.visit_links(visitor);
        self.radius.visit_links(visitor);
    }
}
impl ValidateLinks for SequenceControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if !self.movement_speed.is_finite()
            || self
                .bounds
                .minimum
                .iter()
                .chain(self.bounds.maximum.iter())
                .any(|v| !v.is_finite())
            || !self.bounds.sphere_radius.is_finite()
            || self.bounds.sphere_radius < 0.
            || (0..3).any(|i| self.bounds.minimum[i] > self.bounds.maximum[i])
        {
            visitor(Link::InvalidValue);
        }
        self.selection.visit_links(visitor);
    }
}
impl<B: ValidateLinks, U: ValidateLinks> ValidateLinks for ControllerProgram<B, U> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.unbound.visit_links(visitor);
        self.blocks.visit_links(visitor);
    }
}
impl ValidateLinks for BlockControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.events.visit_links(visitor);
        self.extents.visit_links(visitor);
        for target in &self.targets {
            match target {
                PropertyTarget::Translation(id)
                | PropertyTarget::Rotation(id)
                | PropertyTarget::Scale(id)
                | PropertyTarget::Visibility(id) => id.visit_links(visitor),
                PropertyTarget::MaterialOpacity(id) | PropertyTarget::MaterialTexture(id, _) => {
                    id.visit_links(visitor)
                }
                PropertyTarget::MeshColor(id) | PropertyTarget::MeshOpacity(id) => {
                    id.visit_links(visitor)
                }
                PropertyTarget::TextureTranslation(id)
                | PropertyTarget::TextureRotation(id)
                | PropertyTarget::TextureScale(id) => id.visit_links(visitor),
                PropertyTarget::Profile { owner, .. } => owner.visit_links(visitor),
            }
        }
    }
}

impl<B, U> Default for ControllerProgram<B, U> {
    fn default() -> Self {
        Self {
            state_properties: Vec::new(),
            blocks: Vec::new(),
            unbound: Vec::new(),
        }
    }
}
