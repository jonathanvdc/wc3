//! Logical relationship traversal for checked graph construction.
use crate::model::semantic::{
    ClipId, ClockId, MaterialId, MeshId, ModelId, NodeId, SkinId, TextureId,
};
use crate::model::semantic::{
    ControllerId, CurveValue, EventId, FeatureId, FeatureKind, GeometryKind, InstanceId,
    MaterialSlotId, NodeFeature, SystemId, SystemMember, TextureTransformId,
};
use std::any::TypeId;
/// A relationship to another object in the semantic graph.
#[derive(Clone, Debug)]
pub enum Link {
    /// Another model in the containing collection.
    Model(ModelId),
    /// An identified occurrence in a programme.
    Event(EventId),
    /// Named material slot owned by the model's appearance vocabulary.
    AppearanceSlot(MaterialSlotId),
    /// Shared texture-coordinate transformation.
    TextureTransform(TextureTransformId),
    /// Independently authored scene system.
    System(SystemId),
    /// Authored member of an independently scoped scene system.
    SystemMember {
        /// System owning the member's identity domain.
        system: SystemId,
        /// Collection and ordinal within that domain.
        member: SystemMember,
    },
    /// Invalid numeric or logical rest value encountered during traversal.
    InvalidValue,
    /// A relationship requiring a particular feature payload.
    TypedFeature {
        /// Referenced attached behavior.
        id: FeatureId,
        /// Required payload category.
        kind: FeatureKind,
    },
    /// A relationship accepting any of several explicitly named behavior categories.
    FeatureAlternatives {
        /// Actual attached behavior being selected.
        id: FeatureId,
        /// Accepted behavior categories, independent of profile layout.
        kinds: &'static [FeatureKind],
    },
    /// Geometry placement relationship.
    Instance(InstanceId),
    /// Attached feature relationship.
    Feature(FeatureId),
    /// The value type of a logical animated destination.
    Property {
        /// Animated destination whose owner must exist.
        target: PropertyIdentity,
        /// Rust value type required by this destination.
        value_type: TypeId,
        /// Whether this reference declares an authored destination, rather than targeting it.
        definition: bool,
    },
    /// Node relationship.
    Node(NodeId),
    /// Geometry relationship requiring a particular authored purpose.
    TypedMesh {
        /// Referenced mesh definition.
        id: MeshId,
        /// Required geometry category.
        kind: GeometryKind,
    },
    /// Geometry relationship.
    Mesh(MeshId),
    /// Material relationship.
    Material(MaterialId),
    /// Texture relationship.
    Texture(TextureId),
    /// Clip relationship.
    Clip(ClipId),
    /// Skin relationship.
    Skin(SkinId),
    /// Shared clock relationship.
    Clock(ClockId),
    /// Animation controller relationship.
    Controller(ControllerId),
}
/// Visits logical object references without any file or encoding context.
pub trait ValidateLinks {
    /// Enumerates every referenced graph object.
    fn visit_links(&self, visitor: &mut dyn FnMut(Link));
}
impl<T: ValidateLinks> ValidateLinks for Vec<T> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        for value in self {
            value.visit_links(v)
        }
    }
}
impl<T: ValidateLinks, const N: usize> ValidateLinks for [T; N] {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for value in self {
            value.visit_links(visitor);
        }
    }
}
impl<T: ValidateLinks> ValidateLinks for Option<T> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        if let Some(value) = self {
            value.visit_links(v)
        }
    }
}
impl ValidateLinks for ModelId {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::Model(*self));
    }
}
impl ValidateLinks for NodeId {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        v(Link::Node(*self))
    }
}
impl ValidateLinks for TextureId {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        v(Link::Texture(*self))
    }
}
impl ValidateLinks for MaterialId {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        v(Link::Material(*self))
    }
}
impl ValidateLinks for MeshId {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        v(Link::Mesh(*self))
    }
}
impl ValidateLinks for ClipId {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        v(Link::Clip(*self))
    }
}
impl ValidateLinks for SkinId {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        v(Link::Skin(*self))
    }
}
impl ValidateLinks for ClockId {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        v(Link::Clock(*self))
    }
}

use crate::model::semantic::profiles::Profile;
use crate::model::semantic::{
    Animated, Property, PropertyIdentity, PropertyPayload, PropertyTarget,
};
impl<T: 'static, P: Profile> ValidateLinks for Property<T, P> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        v(Link::Property {
            target: self.target().identity(),
            value_type: TypeId::of::<T>(),
            definition: true,
        });
        if self
            .target()
            .value_type()
            .is_some_and(|expected| expected != TypeId::of::<T>())
        {
            v(Link::InvalidValue);
        }
        match self.target() {
            PropertyTarget::Translation(id)
            | PropertyTarget::Rotation(id)
            | PropertyTarget::Scale(id)
            | PropertyTarget::Visibility(id) => id.visit_links(v),
            PropertyTarget::MaterialOpacity(id) | PropertyTarget::MaterialTexture(id, _) => {
                id.visit_links(v)
            }
            PropertyTarget::MeshColor(id) | PropertyTarget::MeshOpacity(id) => id.visit_links(v),
            PropertyTarget::TextureTranslation(id)
            | PropertyTarget::TextureRotation(id)
            | PropertyTarget::TextureScale(id) => id.visit_links(v),
            PropertyTarget::Profile { owner, key } => {
                if !key.accepts_owner(*owner) {
                    v(Link::InvalidValue);
                }
                owner.visit_links(v);
            }
        }
    }
}
impl<T: CurveValue + 'static, P: Profile> ValidateLinks for Animated<T, P> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        if !self.default.is_valid() {
            v(Link::InvalidValue);
        }
        self.default.visit_value_links(v);
        self.property.visit_links(v);
    }
}
use crate::model::semantic::{Channel, Clock, Track};
impl<T: CurveValue + 'static, P: Profile> ValidateLinks for Track<'_, T, P> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        for value in self.curve.values().iter() {
            value.visit_value_links(v);
        }
        self.curve.visit_control_links(v);
        self.target.visit_links(&mut |link| match link {
            Link::Property {
                target, value_type, ..
            } => v(Link::Property {
                target,
                value_type,
                definition: false,
            }),
            other => v(other),
        });
        if let Clock::Shared(id) = self.clock {
            id.visit_links(v);
        }
    }
}
impl<P: Profile> ValidateLinks for Channel<'_, P> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        match self {
            Self::Unsigned4(t) => t.visit_links(v),
            Self::Bytes4(t) => t.visit_links(v),
            Self::Rotation(t) => t.visit_links(v),
            Self::Scalar(t) => t.visit_links(v),
            Self::Vector2(t) => t.visit_links(v),
            Self::Vector3(t) => t.visit_links(v),
            Self::Vector4(t) => t.visit_links(v),
            Self::Unsigned(t) => t.visit_links(v),
            Self::Signed(t) => t.visit_links(v),
            Self::Unsigned16(t) => t.visit_links(v),
            Self::Byte(t) => t.visit_links(v),
            Self::Boolean(t) => t.visit_links(v),
            Self::Texture(t) => t.visit_links(v),
            Self::OptionalTexture(t) => t.visit_links(v),
        }
    }
}

impl ValidateLinks for FeatureId {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        v(Link::Feature(*self));
    }
}
impl ValidateLinks for NodeFeature<'_> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        match self {
            Self::Wem(value) => value.visit_links(v),
            Self::Generic(value) => value.visit_links(v),
            Self::Mdlx(value) => value.visit_links(v),
            Self::M2(value) => value.visit_links(v),
            Self::M3(value) => value.visit_links(v),
            Self::Wmo(value) => value.visit_links(v),
        }
    }
}

impl ValidateLinks for InstanceId {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        v(Link::Instance(*self));
    }
}
impl ValidateLinks for ControllerId {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        v(Link::Controller(*self));
    }
}

impl ValidateLinks for TextureTransformId {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::TextureTransform(*self));
    }
}
impl ValidateLinks for SystemId {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::System(*self));
    }
}

impl ValidateLinks for MaterialSlotId {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::AppearanceSlot(*self));
    }
}

impl ValidateLinks for EventId {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::Event(*self));
    }
}
