//! Model playback, shadow passes, geometry sorting, and external category bindings.
use crate::model::values::model::{self as values, ModelFlags, SequenceFlags};
use crate::model::{Bounds, ClipId, Link, NodeId, ValidateLinks};
/// Native playback selection and transition controls owned by a common clip.
#[derive(Clone, Debug)]
pub struct SequenceControls {
    /// External animation-data category selecting this playback program.
    pub animation_id: u16,
    /// Variation identity within the animation-data category.
    pub variation: u16,
    /// Movement speed associated with playback, in model units per second.
    pub movement_speed: f32,
    /// Sequence behavior and storage declarations; alias selection is stored separately.
    pub flags: SequenceFlags,
    /// Relative selection frequency among variations.
    pub frequency: i16,
    /// Minimum and maximum replay-control values declared by the sequence.
    pub replay: [u32; 2],
    /// Incoming and outgoing transition durations in milliseconds.
    pub blend_time: [u32; 2],
    /// Conservative spatial bounds associated with this playback program.
    pub bounds: Bounds,
    /// Next alternative playback program in this variation chain.
    pub next_variation: Option<ClipId>,
    /// Native alias selection; common playback shares the target animation identities directly.
    pub alias: Option<ClipId>,
}
impl Default for SequenceControls {
    fn default() -> Self {
        Self {
            animation_id: 0,
            variation: 0,
            movement_speed: 0.0,
            flags: SequenceFlags::default(),
            frequency: 1,
            replay: [0; 2],
            blend_time: [0; 2],
            bounds: Bounds::default(),
            next_variation: None,
            alias: None,
        }
    }
}
impl ValidateLinks for SequenceControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.next_variation.visit_links(visitor);
        self.alias.visit_links(visitor);
        if !self.movement_speed.is_finite() || self.bounds.validate().is_err() {
            visitor(Link::InvalidValue);
        }
    }
}
/// Native shadow-pass behavior; geometry, range, and surface live on the draw.
#[derive(Clone, Debug)]
pub struct ShadowControls {
    /// Complete shadow behavior mask.
    pub flags: values::ShadowBatchFlags,
    /// Additional shadow and edge-fade behavior.
    pub extra_flags: values::ShadowBatchExtraFlags,
    /// Unresolved shadow parameter.
    pub parameter: u16,
    /// Unresolved native color selection parameter.
    pub color_parameter: u16,
    /// Unresolved native opacity selection parameter.
    pub transparency_parameter: u16,
}
impl ValidateLinks for ShadowControls {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
/// Model-wide behavior and external game-category bindings.
#[derive(Clone, Debug, Default)]
pub struct ModelMetadata {
    /// Model behavior flags.
    pub flags: ModelFlags,
    /// Playback bindings indexed by external animation-data category.
    pub animation_lookup: Vec<Option<ClipId>>,
    /// Skeleton bindings indexed by external key-bone category.
    pub key_bone_lookup: Vec<Option<NodeId>>,
}
impl ValidateLinks for ModelMetadata {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.animation_lookup.visit_links(v);
        self.key_bone_lookup.visit_links(v)
    }
}
/// Sorting and culling behavior attached directly to its logical geometry.
#[derive(Clone, Debug)]
pub struct GeometryControls {
    /// Bone carrying the section's sorting frame.
    pub center_bone: Option<NodeId>,
    /// Section center in model coordinates.
    pub center: [f32; 3],
    /// Optional transparency sorting sphere, as center and radius.
    pub sorting: Option<([f32; 3], f32)>,
}
impl ValidateLinks for GeometryControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.center_bone.visit_links(visitor);
        if self.center.iter().any(|v| !v.is_finite())
            || self.sorting.is_some_and(|(center, radius)| {
                center.iter().any(|v| !v.is_finite()) || !radius.is_finite() || radius < 0.
            })
        {
            visitor(Link::InvalidValue);
        }
    }
}

/// Skeleton controls whose meanings are specific to the game model family.
#[derive(Clone, Debug)]
pub struct BoneControls {
    /// External key-bone category.
    pub key_bone: i32,
    /// Simulation and transform behavior beyond the common hierarchy controls.
    pub flags: values::BoneBehaviorFlags,
    /// Associated mesh-part category.
    pub submesh: u16,
    /// External name checksum used by host-skeleton matching.
    pub name_crc: u32,
}

impl Default for BoneControls {
    fn default() -> Self {
        Self {
            key_bone: -1,
            flags: values::BoneBehaviorFlags::default(),
            submesh: 0,
            name_crc: 0,
        }
    }
}

/// M2 draw-local native metadata and optional shadow behavior.
#[derive(Clone, Debug, Default)]
pub struct DrawControls {
    /// Auxiliary native section/flags word, whose interpretation varies by client version.
    /// This is not a semantic geometry reference.
    pub auxiliary_section: u16,
    /// Optional dedicated shadow-pass behavior.
    pub shadow: Option<ShadowControls>,
}
impl ValidateLinks for DrawControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.shadow.visit_links(visitor);
    }
}
