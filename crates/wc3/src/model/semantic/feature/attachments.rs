//! Attachment identities and external placements composed independently.
use std::borrow::Cow;
use std::str::from_utf8;

use crate::model::semantic::profiles::{m2, m3, mdlx, wmo};
use crate::model::semantic::{scene, Animated, FeatureKind, Link, Resource, ValidateLinks};

/// Attachment points and external model placements.
#[derive(Clone, Debug)]
pub enum Attachment<'a> {
    /// A point exposed to game content, without an instantiated resource.
    Point(AttachmentPoint<'a>),
    /// A resource instantiated at its carrying node.
    External(ExternalAttachment<'a>),
}

/// Identity and activation of an exposed attachment point.
#[derive(Clone, Debug)]
pub struct AttachmentPoint<'a> {
    /// Identity in an explicitly distinguished application namespace.
    pub identity: AttachmentIdentity<'a>,
    /// Authored activation behavior.
    pub activation: AttachmentActivation,
    /// Additional authored control whose meaning remains unresolved.
    pub control: Option<u32>,
}

/// Attachment namespaces do not imply correspondence between numeric values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttachmentIdentity<'a> {
    /// Named point addressed by game content.
    Named(Cow<'a, str>),
    /// Application attachment role from an M2 model.
    Role(u32),
    /// Attachment-point identifier from an MDLX model.
    Index(u32),
}

/// Discrete enablement and fractional visibility have distinct meanings.
#[derive(Clone, Debug)]
pub enum AttachmentActivation {
    /// No authored activation control.
    Uncontrolled,
    /// Discrete enable toggle.
    Enabled(Animated<bool>),
    /// Fractional visibility multiplier.
    Visibility(Animated<f32>),
}

/// Instantiated resource with optional attachment-point identity and appearance.
#[derive(Clone, Debug)]
pub struct ExternalAttachment<'a> {
    /// Resource path or external database identity, preserving borrowed storage.
    pub resource: Resource<'a>,
    /// Optional exposed point identifying this instance.
    pub point: Option<AttachmentPoint<'a>>,
    /// Authored placement appearance and controls.
    pub appearance: PlacementAppearance,
}

/// Appearance of an external resource placement.
#[derive(Clone, Debug)]
pub enum PlacementAppearance {
    /// No authored appearance override.
    Unmodified,
    /// World placement tint, illumination, and unresolved controls.
    World {
        /// Tint with optional referenced world-light illumination.
        tint: wmo::DoodadTint,
        /// Optional multiplier applied to the placement tint.
        color_multiplier: Option<f32>,
        /// Authored placement flags with unresolved meanings.
        controls: wmo::DoodadFlags,
    },
}

impl<'a> From<m2::Attachment> for AttachmentPoint<'a> {
    fn from(value: m2::Attachment) -> Self {
        Self {
            identity: AttachmentIdentity::Role(value.role),
            activation: AttachmentActivation::Enabled(value.enabled),
            control: None,
        }
    }
}
impl<'a> TryFrom<AttachmentPoint<'a>> for m2::Attachment {
    type Error = Box<AttachmentPoint<'a>>;
    fn try_from(value: AttachmentPoint<'a>) -> Result<Self, Self::Error> {
        match value {
            AttachmentPoint {
                identity: AttachmentIdentity::Role(role),
                activation: AttachmentActivation::Enabled(enabled),
                control: None,
            } => Ok(Self { role, enabled }),
            value => Err(Box::new(value)),
        }
    }
}
impl<'a> From<m3::Attachment<'a>> for AttachmentPoint<'a> {
    fn from(value: m3::Attachment<'a>) -> Self {
        Self {
            identity: AttachmentIdentity::Named(value.name),
            activation: AttachmentActivation::Uncontrolled,
            control: Some(value.control),
        }
    }
}
impl<'a> TryFrom<AttachmentPoint<'a>> for m3::Attachment<'a> {
    type Error = Box<AttachmentPoint<'a>>;
    fn try_from(value: AttachmentPoint<'a>) -> Result<Self, Self::Error> {
        match value {
            AttachmentPoint {
                identity: AttachmentIdentity::Named(name),
                activation: AttachmentActivation::Uncontrolled,
                control: Some(control),
            } => Ok(Self { name, control }),
            value => Err(Box::new(value)),
        }
    }
}
impl<'a> From<scene::Attachment<'a>> for ExternalAttachment<'a> {
    fn from(value: scene::Attachment<'a>) -> Self {
        Self {
            resource: value.resource,
            point: Some(AttachmentPoint {
                identity: AttachmentIdentity::Index(value.id),
                activation: AttachmentActivation::Visibility(value.visibility),
                control: None,
            }),
            appearance: PlacementAppearance::Unmodified,
        }
    }
}
impl<'a> TryFrom<ExternalAttachment<'a>> for scene::Attachment<'a> {
    type Error = Box<ExternalAttachment<'a>>;
    fn try_from(value: ExternalAttachment<'a>) -> Result<Self, Self::Error> {
        match value {
            ExternalAttachment {
                resource,
                point:
                    Some(AttachmentPoint {
                        identity: AttachmentIdentity::Index(id),
                        activation: AttachmentActivation::Visibility(visibility),
                        control: None,
                    }),
                appearance: PlacementAppearance::Unmodified,
            } => Ok(Self {
                resource,
                id,
                visibility,
            }),
            value => Err(Box::new(value)),
        }
    }
}
impl<'a> From<mdlx::Attachment<'a>> for ExternalAttachment<'a> {
    fn from(value: mdlx::Attachment<'a>) -> Self {
        scene::Attachment {
            resource: Resource::Path(match value.resource {
                Cow::Borrowed(path) => Cow::Borrowed(path.as_bytes()),
                Cow::Owned(path) => Cow::Owned(path.into_bytes()),
            }),
            id: value.id,
            visibility: value.visibility,
        }
        .into()
    }
}
impl<'a> TryFrom<ExternalAttachment<'a>> for mdlx::Attachment<'a> {
    type Error = Box<ExternalAttachment<'a>>;
    fn try_from(value: ExternalAttachment<'a>) -> Result<Self, Self::Error> {
        if !matches!(&value.resource, Resource::Path(path) if from_utf8(path).is_ok()) {
            return Err(Box::new(value));
        }
        let value = scene::Attachment::try_from(value)?;
        let Resource::Path(path) = value.resource else {
            unreachable!()
        };
        let resource = match path {
            Cow::Borrowed(path) => Cow::Borrowed(from_utf8(path).expect("validated UTF-8")),
            Cow::Owned(path) => Cow::Owned(String::from_utf8(path).expect("validated UTF-8")),
        };
        Ok(Self {
            resource,
            id: value.id,
            visibility: value.visibility,
        })
    }
}
impl<'a> From<wmo::Doodad<'a>> for ExternalAttachment<'a> {
    fn from(value: wmo::Doodad<'a>) -> Self {
        Self {
            resource: value.resource,
            point: None,
            appearance: PlacementAppearance::World {
                tint: value.tint,
                color_multiplier: value.color_multiplier,
                controls: value.controls,
            },
        }
    }
}
impl<'a> TryFrom<ExternalAttachment<'a>> for wmo::Doodad<'a> {
    type Error = Box<ExternalAttachment<'a>>;
    fn try_from(value: ExternalAttachment<'a>) -> Result<Self, Self::Error> {
        match value {
            ExternalAttachment {
                resource,
                point: None,
                appearance:
                    PlacementAppearance::World {
                        tint,
                        color_multiplier,
                        controls,
                    },
            } => Ok(Self {
                resource,
                tint,
                color_multiplier,
                controls,
            }),
            value => Err(Box::new(value)),
        }
    }
}
impl ValidateLinks for AttachmentPoint<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match &self.activation {
            AttachmentActivation::Uncontrolled => {}
            AttachmentActivation::Enabled(value) => value.visit_links(visitor),
            AttachmentActivation::Visibility(value) => value.visit_links(visitor),
        }
    }
}
impl ValidateLinks for ExternalAttachment<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.point.visit_links(visitor);
        if let PlacementAppearance::World {
            tint,
            color_multiplier,
            ..
        } = &self.appearance
        {
            if color_multiplier.is_some_and(|value| !value.is_finite()) {
                visitor(Link::InvalidValue);
            }
            if let wmo::DoodadTint::Lit { light: id, .. } = tint {
                visitor(Link::TypedFeature {
                    id: *id,
                    kind: FeatureKind::WmoLight,
                });
            }
        }
    }
}
impl ValidateLinks for Attachment<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Point(value) => value.visit_links(visitor),
            Self::External(value) => value.visit_links(visitor),
        }
    }
}
