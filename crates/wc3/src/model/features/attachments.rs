//! Sockets and authored child models, composed independently.
use crate::model::common::{Detached, IntoOwned};
use crate::model::features::{FromFeature, TryFromFeature};
use crate::model::ModelResourceId;
use crate::model::{Construct, PropertyValue, PropertyValues};
use std::borrow::Cow;

use crate::model::profiles::{m2, wmo};
use crate::model::{
    scene, FeatureId, FeatureKind, Link, ModelMount, MountTarget, NodeId, Property, ValidateLinks,
    ValueSample,
};

/// Sockets and authored child model references.
#[derive(Clone, Debug, IntoOwned)]
pub enum Attachment<'a> {
    /// A socket exposed to game content, without an instantiated resource.
    Socket(Socket<'a>),
    /// A resource instantiated at its carrying node.
    Child(ChildModel<'a>),
}

impl<'a> Attachment<'a> {
    /// Exposed socket, when this behavior declares one.
    pub fn socket(&self) -> Option<&Socket<'a>> {
        match self {
            Self::Socket(point) => Some(point),
            Self::Child(value) => value.socket.as_ref(),
        }
    }

    /// Instantiated child resource, when this behavior declares one.
    pub fn child(&self) -> Option<&ChildModel<'a>> {
        match self {
            Self::Socket(_) => None,
            Self::Child(value) => Some(value),
        }
    }

    /// Describe the authored child mount independently of resource resolution.
    /// The supplied identities belong to this behavior and its carrying node.
    pub fn mount(&self, feature: FeatureId, node: NodeId) -> Option<ModelMount> {
        self.child().map(|_| {
            ModelMount::new(if self.socket().is_some() {
                MountTarget::Socket(feature)
            } else {
                MountTarget::Node(node)
            })
        })
    }

    /// Samples mounted content visibility from ordinary, retimed, or evaluated values.
    /// Children without a socket are fully visible.
    pub fn visibility(&self, sample: &impl ValueSample) -> f32 {
        self.socket()
            .map_or(1., |point| sample.animated(&point.visibility).clamp(0., 1.))
    }
}

/// Identity and visibility of an exposed socket.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SocketDefinition<I, A, C> {
    /// Identity in an explicitly distinguished application namespace.
    pub identity: I,
    /// Authored visibility of mounted content.
    pub visibility: A,
    /// Additional authored control whose meaning remains unresolved.
    pub control: C,
}

/// Broad built-in socket vocabulary.
pub type Socket<'a> = SocketDefinition<SocketIdentity<'a>, Property<f32>, Option<u32>>;

/// Attachment namespaces do not imply correspondence between numeric values.
#[derive(Clone, Debug, PartialEq, Eq, IntoOwned)]
pub enum SocketIdentity<'a> {
    /// Named socket addressed by game content.
    Named(Cow<'a, str>),
    /// Application attachment role from an M2 model.
    Role(u32),
    /// Attachment-point identifier from an MDLX model.
    Index(u32),
}

impl Socket<'_> {
    /// Detach a borrowed socket name from its source storage.
    pub fn into_owned(self) -> Socket<'static> {
        Socket {
            identity: self.identity.into_owned(),
            visibility: self.visibility,
            control: self.control,
        }
    }
}
/// Child resource with an optional exposed socket and authored appearance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChildModelDefinition<R, S, A> {
    /// Resource path or external database identity, preserving borrowed storage.
    pub resource: R,
    /// Exposed socket, when admitted by the selected vocabulary.
    pub socket: S,
    /// Authored placement appearance and flags.
    pub appearance: A,
}
owned_components!(SocketDefinition<I, A, C> { identity, visibility, control; });
owned_components!(ChildModelDefinition<R, S, A> { resource, socket, appearance; });

/// Broad built-in external-placement vocabulary.
pub type ChildModel<'a> =
    ChildModelDefinition<ModelResourceId, Option<Socket<'a>>, PlacementAppearance>;
/// Child resource carrying an indexed socket with fractional visibility.
pub type IndexedChildModel<R> =
    ChildModelDefinition<R, SocketDefinition<u32, Property<f32>, ()>, ()>;
/// World placement appearance, independently reusable from its resource identity.
#[derive(Clone, Debug, PartialEq, IntoOwned)]
pub struct WorldPlacementAppearance {
    /// Tint and optional linked illumination.
    pub tint: wmo::DoodadTint,
    /// Optional tint multiplier.
    pub color_multiplier: Option<f32>,
    /// Authored placement flags with unresolved meanings.
    pub flags: wmo::DoodadFlags,
}
/// Appearance of an external resource placement.
#[derive(Clone, Debug, IntoOwned)]
pub enum PlacementAppearance {
    /// No authored appearance override.
    Unmodified,
    /// World placement tint, illumination, and authored flags.
    World(WorldPlacementAppearance),
}

impl<'a> FromFeature<m2::Attachment> for Socket<'a> {
    fn from_feature(value: m2::Attachment, values: &mut PropertyValues) -> Self {
        Self {
            identity: SocketIdentity::Role(value.identity),
            visibility: values
                .scalar_from_boolean(value.visibility)
                .expect("validated source slot"),
            control: None,
        }
    }
}
impl<'a> TryFromFeature<Socket<'a>> for m2::Attachment {
    type Error = Box<Socket<'a>>;
    fn try_from_feature(value: Socket<'a>, values: &PropertyValues) -> Result<Self, Self::Error> {
        if let Socket {
            identity: SocketIdentity::Role(role),
            visibility,
            control: None,
        } = &value
        {
            if let Some(source) = values.boolean_source(*visibility) {
                return Ok(Self {
                    identity: *role,
                    visibility: source,
                    control: (),
                });
            }
        }
        Err(Box::new(value))
    }
}

impl<'a> From<scene::Attachment> for ChildModel<'a> {
    fn from(value: scene::Attachment) -> Self {
        Self {
            resource: value.resource,
            socket: Some(Socket {
                identity: SocketIdentity::Index(value.socket.identity),
                visibility: value.socket.visibility,
                control: None,
            }),
            appearance: PlacementAppearance::Unmodified,
        }
    }
}
impl<'a> TryFrom<ChildModel<'a>> for scene::Attachment {
    type Error = Box<ChildModel<'a>>;
    fn try_from(value: ChildModel<'a>) -> Result<Self, Self::Error> {
        match value {
            ChildModel {
                resource,
                socket:
                    Some(Socket {
                        identity: SocketIdentity::Index(id),
                        visibility,
                        control: None,
                    }),
                appearance: PlacementAppearance::Unmodified,
            } => Ok(Self {
                resource,
                socket: SocketDefinition {
                    identity: id,
                    visibility,
                    control: (),
                },
                appearance: (),
            }),
            value => Err(Box::new(value)),
        }
    }
}
impl<'a> From<wmo::Doodad> for ChildModel<'a> {
    fn from(value: wmo::Doodad) -> Self {
        Self {
            resource: value.resource,
            socket: None,
            appearance: PlacementAppearance::World(value.appearance),
        }
    }
}
impl<'a> TryFrom<ChildModel<'a>> for wmo::Doodad {
    type Error = Box<ChildModel<'a>>;
    fn try_from(value: ChildModel<'a>) -> Result<Self, Self::Error> {
        match value {
            ChildModel {
                resource,
                socket: None,
                appearance: PlacementAppearance::World(appearance),
            } => Ok(Self {
                resource,
                socket: (),
                appearance,
            }),
            value => Err(Box::new(value)),
        }
    }
}
impl<I: ValidateLinks, A: ValidateLinks, C: ValidateLinks> ValidateLinks
    for SocketDefinition<I, A, C>
{
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.identity.visit_links(visitor);
        self.visibility.visit_links(visitor);
        self.control.visit_links(visitor);
    }
}
impl ValidateLinks for SocketIdentity<'_> {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
impl<R: ValidateLinks, S: ValidateLinks, A: ValidateLinks> ValidateLinks
    for ChildModelDefinition<R, S, A>
{
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.resource.visit_links(visitor);
        self.socket.visit_links(visitor);
        self.appearance.visit_links(visitor);
    }
}
impl ValidateLinks for PlacementAppearance {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Self::World(appearance) = self {
            appearance.visit_links(visitor);
        }
    }
}
impl ValidateLinks for WorldPlacementAppearance {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self
            .color_multiplier
            .is_some_and(|value| !value.is_finite())
        {
            visitor(Link::InvalidValue);
        }
        if let wmo::DoodadTint::Lit { light: id, .. } = &self.tint {
            visitor(Link::TypedFeature {
                id: *id,
                kind: FeatureKind::WmoLight,
            });
        }
    }
}
impl Default for WorldPlacementAppearance {
    fn default() -> Self {
        Self {
            tint: wmo::DoodadTint::Color([255; 4]),
            color_multiplier: None,
            flags: Default::default(),
        }
    }
}
impl ValidateLinks for Attachment<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Socket(value) => value.visit_links(visitor),
            Self::Child(value) => value.visit_links(visitor),
        }
    }
}

impl<I: Default, T: PropertyValue + Default, C: Default> Construct
    for SocketDefinition<I, Property<T>, C>
{
    fn construct(values: &mut PropertyValues) -> Self {
        Self {
            identity: I::default(),
            visibility: values.add(T::default()),
            control: C::default(),
        }
    }
}
