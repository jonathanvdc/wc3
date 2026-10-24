//! Typed owners and structured semantic field locations.
use crate::model::scene::{BodyId, JointId, ShapeId};
use crate::model::{
    AnimationId, BoneOverrideResourceId, ClipId, ClockId, ConnectivityId, DrawId, DrawSubsetId,
    EffectResourceId, EventId, EventSourceId, FeatureId, ImageResourceId, InstanceId, LookId,
    MaterialId, MeshId, ModelResourceId, NodeId, PhysicsResourceId, SkinId, SystemId,
};
use crate::model::{AttributeDomain, AttributeSemantic, SystemMember, ValueSlot};
use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};

/// An actual semantic identity, retained independently of its display spelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    /// A node definition or reference.
    Node(NodeId),
    /// A mesh definition or reference.
    Mesh(MeshId),
    /// A material definition or reference.
    Material(MaterialId),
    /// A clip definition or reference.
    Clip(ClipId),
    /// A instance definition or reference.
    Instance(InstanceId),
    /// A skin definition or reference.
    Skin(SkinId),
    /// A clock definition or reference.
    Clock(ClockId),
    /// A eventsource definition or reference.
    EventSource(EventSourceId),
    /// A event definition or reference.
    Event(EventId),
    /// A look definition or reference.
    Look(LookId),
    /// A feature definition or reference.
    Feature(FeatureId),
    /// A system definition or reference.
    System(SystemId),
    /// A draw definition or reference.
    Draw(DrawId),
    /// A connectivity definition or reference.
    Connectivity(ConnectivityId),
    /// A drawsubset definition or reference.
    DrawSubset(DrawSubsetId),
    /// A animation definition or reference.
    Animation(AnimationId),
    /// A imageresource definition or reference.
    ImageResource(ImageResourceId),
    /// A modelresource definition or reference.
    ModelResource(ModelResourceId),
    /// A effectresource definition or reference.
    EffectResource(EffectResourceId),
    /// A physicsresource definition or reference.
    PhysicsResource(PhysicsResourceId),
    /// A boneoverrideresource definition or reference.
    BoneOverrideResource(BoneOverrideResourceId),
    /// A typed authored property slot.
    Value(ValueSlot),
    /// A shape within a portable simulation system.
    Shape(ShapeId),
    /// A body within a portable simulation system.
    Body(BodyId),
    /// A joint within a portable simulation system.
    Joint(JointId),
    /// A member in an explicitly owned scene-system domain.
    SystemMember {
        /// Owning scene system.
        system: SystemId,
        /// Selected local member.
        member: SystemMember,
    },
}
macro_rules! owners {
    ($($variant:ident($ty:ty)),* $(,)?) => {$ (
        impl From<$ty> for Owner {
            fn from(id: $ty) -> Self { Self::$variant(id) }
        }
    )*};
}
owners! {
    Node(NodeId),
    Mesh(MeshId),
    Material(MaterialId),
    Clip(ClipId),
    Instance(InstanceId),
    Skin(SkinId),
    Clock(ClockId),
    EventSource(EventSourceId),
    Event(EventId),
    Look(LookId),
    Feature(FeatureId),
    System(SystemId),
    Draw(DrawId),
    Connectivity(ConnectivityId),
    DrawSubset(DrawSubsetId),
    Animation(AnimationId),
    ImageResource(ImageResourceId),
    ModelResource(ModelResourceId),
    EffectResource(EffectResourceId),
    PhysicsResource(PhysicsResourceId),
    BoneOverrideResource(BoneOverrideResourceId),
    Value(ValueSlot), Shape(ShapeId), Body(BodyId), Joint(JointId),
}
/// One literal component of a semantic location; labels are never parsed as paths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Segment {
    /// A scoped owner or nested definition.
    Owner(Owner),
    /// One local field identifier or actual custom-schema data label.
    /// Nesting, owners and indices occupy separate segments.
    Field(Cow<'static, str>),
    /// An element within an ordered field.
    Index(usize),
    /// An attribute identified by its semantic meaning and correspondence domain.
    Attribute {
        /// Owned attribute meaning, including a custom name when authored.
        semantic: AttributeSemantic<'static>,
        /// Domain carrying the attribute's values.
        domain: AttributeDomain,
    },
}
/// A semantic location constructed from owners, fields, indices and attributes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Location {
    segments: Vec<Segment>,
}
impl Location {
    /// Begins a local location without an enclosing model owner.
    pub fn new() -> Self {
        Self::default()
    }
    /// Begins at an actual semantic owner or value slot.
    pub fn for_owner(owner: impl Into<Owner>) -> Self {
        Self::new().with_owner(owner)
    }
    /// Appends one local identifier, with nesting and indices in separate segments.
    /// Punctuation is literal only when it belongs to an actual authored or
    /// custom-schema field label; this does not accept an encoded field path.
    pub fn field(mut self, field: impl Into<Cow<'static, str>>) -> Self {
        self.segments.push(Segment::Field(field.into()));
        self
    }
    /// Appends an index in an ordered field.
    pub fn index(mut self, index: usize) -> Self {
        self.segments.push(Segment::Index(index));
        self
    }
    /// Appends an owned attribute identity without retaining its source model.
    pub fn attribute(mut self, semantic: AttributeSemantic<'_>, domain: AttributeDomain) -> Self {
        self.segments.push(Segment::Attribute {
            semantic: semantic.into_owned(),
            domain,
        });
        self
    }
    /// Prepends an enclosing owner while retaining any more specific location.
    pub fn with_owner(self, owner: impl Into<Owner>) -> Self {
        self.prefixed_by(Self {
            segments: vec![Segment::Owner(owner.into())],
        })
    }
    /// Prepends context while retaining every existing location component.
    pub fn prefixed_by(self, mut context: Self) -> Self {
        context.segments.extend(self.segments);
        context
    }
    /// Inspects the actual components without parsing presentation text.
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }
}
impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (index, segment) in self.segments.iter().enumerate() {
            match segment {
                Segment::Index(value) => write!(f, "[{value}]")?,
                Segment::Field(value) => {
                    if index != 0 {
                        f.write_str(".")?;
                    }
                    f.write_str(value)?;
                }
                Segment::Owner(value) => {
                    if index != 0 {
                        f.write_str(".")?;
                    }
                    write!(f, "{value:?}")?;
                }
                Segment::Attribute { semantic, domain } => {
                    if index != 0 {
                        f.write_str(".")?;
                    }
                    write!(f, "attribute({semantic:?}, {domain:?})")?;
                }
            }
        }
        Ok(())
    }
}
