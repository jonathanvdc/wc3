//! World of Warcraft model authoring features independent of binary layout.
/// Typed animated destinations for World of Warcraft features.
pub mod properties;
mod texture;
pub use texture::{SupplementalTextureFlags, TextureControls};
mod materials;
pub use materials::*;
mod actors;
pub use actors::*;
mod particles;
pub use particles::*;
mod physics;
pub use physics::*;
/// Logical choices and behavior flags specific to this model family.
pub mod values;
use crate::model::semantic::{
    Animated, Bounds, ClipId, Link, NodeId, Resource, ValidateLinks, Values,
};
use values::{ModelFlags, SequenceFlags};

/// Native playback selection and transition controls owned by a common clip.
#[derive(Clone, Debug)]
pub struct SequenceControls {
    /// External animation-data category selecting this playback program.
    pub animation_id: u16,
    /// Variation identity within the animation-data category.
    pub variation: u16,
    /// Movement speed associated with playback, in model units per second.
    pub movement_speed: f32,
    /// Sequence behavior and storage declarations; playback aliases live on the clip.
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
        }
    }
}
impl ValidateLinks for SequenceControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.next_variation.visit_links(visitor);
        if !self.movement_speed.is_finite() || self.bounds.validate().is_err() {
            visitor(Link::InvalidValue);
        }
    }
}
/// Additional behavior for an actual draw pass.
#[derive(Clone, Debug)]
pub enum DrawControls {
    /// Native shadow pass controls; geometry, range and surface live on the draw.
    Shadow {
        /// Complete shadow behavior mask.
        flags: values::ShadowBatchFlags,
        /// Additional shadow and edge-fade behavior.
        extra_flags: values::ShadowBatchExtraFlags,
        /// Unresolved shadow parameter.
        parameter: u16,
        /// Unresolved native color selection parameter.
        color_parameter: u16,
        /// Unresolved native opacity selection parameter.
        transparency_parameter: u16,
    },
}
impl ValidateLinks for DrawControls {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
/// Sparse per-bone transform override supplied by a companion asset.
#[derive(Clone, Debug)]
pub struct BoneOverride<'a> {
    /// External companion resource identity.
    pub resource: Option<Resource<'a>>,
    /// Bones receiving the matrices, in matching order.
    pub bones: Vec<NodeId>,
    /// Model-space override transforms.
    pub transforms: Values<'a, [[f32; 4]; 4]>,
}
/// Model-wide behavior and external game-category bindings.
#[derive(Clone, Debug, Default)]
pub struct ModelControls {
    /// Model behavior flags.
    pub flags: ModelFlags,
    /// Playback bindings indexed by external animation-data category.
    pub animation_lookup: Vec<Option<ClipId>>,
    /// Skeleton bindings indexed by external key-bone category.
    pub key_bone_lookup: Vec<Option<NodeId>>,
}
/// Independent authoring systems that can coexist in one logical model.
#[derive(Clone, Debug)]
pub enum System<'a> {
    /// Static collision geometry.
    Collision(Collision<'a>),
    /// Sparse customization matrices and their external resources.
    BoneOverrides(Vec<BoneOverride<'a>>),
    /// Animated texture controls not assigned to an active surface.
    TextureAnimations(TextureAnimations),
    /// Physics simulation and constraints.
    Physics(Physics<'a>),
}
/// Independently authored texture animation controls.
#[derive(Clone, Debug, Default)]
pub struct TextureAnimations {
    /// Texture-space transformations.
    pub transforms: Vec<TextureTransform>,
    /// Animated texture opacity weights.
    pub weights: Vec<Animated<f32>>,
}
/// Collision triangles without native offsets or vertex encodings.
#[derive(Clone, Debug)]
pub struct Collision<'a> {
    /// Collision vertices in model coordinates.
    pub positions: Values<'a, [f32; 3]>,
    /// Collision face connectivity.
    pub triangles: Values<'a, [u32; 3]>,
    /// Per-face collision normals.
    pub normals: Values<'a, [f32; 3]>,
    /// Conservative collision bounds.
    pub bounds: Bounds,
}
impl ValidateLinks for BoneOverride<'_> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.bones.visit_links(v)
    }
}
impl ValidateLinks for ModelControls {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.animation_lookup.visit_links(v);
        self.key_bone_lookup.visit_links(v)
    }
}
impl ValidateLinks for TextureAnimations {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.transforms.visit_links(v);
        self.weights.visit_links(v)
    }
}
impl ValidateLinks for System<'_> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        match self {
            Self::Collision(value) => {
                if value.bounds.validate().is_err()
                    || value
                        .triangles
                        .iter()
                        .flatten()
                        .any(|i| i as usize >= value.positions.len())
                    || (!value.normals.is_empty() && value.normals.len() != value.triangles.len())
                    || value
                        .positions
                        .iter()
                        .chain(value.normals.iter())
                        .flatten()
                        .any(|x| !x.is_finite())
                {
                    v(Link::InvalidValue)
                }
            }
            Self::BoneOverrides(values) => {
                for value in values {
                    value.visit_links(v);
                    if value.bones.len() != value.transforms.len()
                        || value
                            .transforms
                            .iter()
                            .flatten()
                            .flatten()
                            .any(|x| !x.is_finite())
                    {
                        v(Link::InvalidValue)
                    }
                }
            }
            Self::TextureAnimations(value) => value.visit_links(v),
            Self::Physics(value) => {
                value.visit_links(v);
                if value.validate().is_err() {
                    v(Link::InvalidValue)
                }
            }
        }
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

mod vocabulary;
pub use vocabulary::M2;

mod geometry;
pub use geometry::*;
