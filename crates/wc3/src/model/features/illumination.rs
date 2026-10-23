//! Shared illumination components with explicit activation semantics.
use crate::model::common::{Detached, IntoOwned};
use crate::model::profiles::{m2, mdlx, wmo};
use crate::model::Construct;
use crate::model::{Link, Property, ValidateLinks};
use glam::Vec3;
use m2::values::LightKind as ActorLightKind;
/// Complete illumination models.
#[derive(Clone, Debug, IntoOwned)]
pub enum Light {
    /// RGB lighting with cone and shadow controls.
    SingleEmission(Box<SingleEmissionLight>),
    /// Independently animated ambient and direct illumination.
    AmbientDiffuse(Box<AmbientDiffuseLight>),

    /// World lighting with RGBA color and cookie extensions.
    World(Box<wmo::Light>),
    /// Region-local point illumination.
    Point(wmo::PointLight),
}
/// Directional interpretation shared by ambient/diffuse light programs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, IntoOwned)]
pub enum IlluminationKind {
    /// Radiates from a position.
    Point,
    /// Illuminates along one direction.
    Directional,
    /// Supplies ambient illumination.
    Ambient,
    /// Unresolved selector retaining the vocabulary that gives its code meaning.
    Unknown(UnknownIlluminationKind),
}
/// An unresolved light-kind code cannot be reinterpreted in another vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnknownIlluminationKind {
    /// Warcraft III light selector.
    Mdlx(u32),
    /// World of Warcraft animated-model light selector.
    M2(u16),
}
/// Independently animated RGB emission and multiplier.
#[derive(Clone, Debug, Construct, PartialEq, IntoOwned)]
pub struct LightEmission {
    /// RGB emission color.
    pub color: Property<Vec3>,
    /// Emission intensity multiplier.
    pub intensity: Property<f32>,
}
/// Shared ambient/diffuse light fields.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct AmbientDiffuseLight<K = IlluminationKind, A = LightActivation> {
    /// Spatial emission interpretation.
    pub kind: K,
    /// Direct illumination.
    pub diffuse: LightEmission,
    /// Ambient illumination.
    pub ambient: LightEmission,
    /// Beginning and ending distance attenuation.
    pub attenuation: [Property<f32>; 2],
    /// Fractional visibility or discrete activation.
    pub activation: A,
    /// Independently authored shadow parameters.
    pub shadows: LightShadows,
    /// Quadratic, linear, and damping attenuation coefficients.
    pub falloff: Option<[Property<f32>; 3]>,
}
owned_components!(AmbientDiffuseLight<K, A> { kind, activation; diffuse, ambient, attenuation, shadows, falloff });

impl<'owned> Detached<'owned> for ActorLightKind {
    type Detached = Self;
    fn into_detached(self) -> Self {
        self
    }
}
/// Distinct authored activation programs.
#[derive(Clone, Debug, IntoOwned)]
pub enum LightActivation {
    /// Fractional visibility.
    Visibility(Property<f32>),
    /// Boolean activation.
    Enabled(Property<bool>),
}
/// Independently authored shadow parameters.
#[derive(Clone, Debug, Default, PartialEq, IntoOwned)]
pub struct LightShadows {
    /// Explicit shadow casting.
    pub casting: Option<bool>,
    /// Explicit shadow intensity.
    pub intensity: Option<f32>,
    /// Animated shadow near/far distances.
    pub range: Option<[Property<f32>; 2]>,
}
impl From<mdlx::Light> for AmbientDiffuseLight {
    fn from(v: mdlx::Light) -> Self {
        Self {
            kind: match v.kind {
                mdlx::LightKind::Point => IlluminationKind::Point,
                mdlx::LightKind::Directional => IlluminationKind::Directional,
                mdlx::LightKind::Ambient => IlluminationKind::Ambient,
                mdlx::LightKind::Unknown(v) => {
                    IlluminationKind::Unknown(UnknownIlluminationKind::Mdlx(v))
                }
            },
            diffuse: v.diffuse,
            ambient: v.ambient,
            attenuation: v.attenuation,
            activation: LightActivation::Visibility(v.activation),
            shadows: v.shadows,
            falloff: v.falloff,
        }
    }
}
impl From<m2::Light> for AmbientDiffuseLight {
    fn from(v: m2::Light) -> Self {
        Self {
            kind: match v.kind {
                ActorLightKind::Point => IlluminationKind::Point,
                ActorLightKind::Directional => IlluminationKind::Directional,
                ActorLightKind::Unknown(v) => {
                    IlluminationKind::Unknown(UnknownIlluminationKind::M2(v))
                }
            },
            diffuse: v.diffuse,
            ambient: v.ambient,
            attenuation: v.attenuation,
            activation: LightActivation::Enabled(v.activation),
            shadows: v.shadows,
            falloff: v.falloff,
        }
    }
}
impl TryFrom<AmbientDiffuseLight> for mdlx::Light {
    type Error = Box<AmbientDiffuseLight>;
    fn try_from(v: AmbientDiffuseLight) -> Result<Self, Self::Error> {
        let kind = match v.kind {
            IlluminationKind::Point => mdlx::LightKind::Point,
            IlluminationKind::Directional => mdlx::LightKind::Directional,
            IlluminationKind::Ambient => mdlx::LightKind::Ambient,
            IlluminationKind::Unknown(UnknownIlluminationKind::Mdlx(raw)) => {
                mdlx::LightKind::Unknown(raw)
            }
            _ => return Err(Box::new(v)),
        };
        let LightActivation::Visibility(control) = v.activation else {
            return Err(Box::new(v));
        };
        Ok(Self {
            kind,
            diffuse: v.diffuse,
            ambient: v.ambient,
            attenuation: v.attenuation,
            activation: control,
            shadows: v.shadows,
            falloff: v.falloff,
        })
    }
}
impl TryFrom<AmbientDiffuseLight> for m2::Light {
    type Error = Box<AmbientDiffuseLight>;
    fn try_from(v: AmbientDiffuseLight) -> Result<Self, Self::Error> {
        let kind = match v.kind {
            IlluminationKind::Point => ActorLightKind::Point,
            IlluminationKind::Directional => ActorLightKind::Directional,
            IlluminationKind::Unknown(UnknownIlluminationKind::M2(raw)) => {
                ActorLightKind::Unknown(raw)
            }
            _ => return Err(Box::new(v)),
        };
        if v.shadows != LightShadows::default() || v.falloff.is_some() {
            return Err(Box::new(v));
        }
        let LightActivation::Enabled(enabled) = v.activation else {
            return Err(Box::new(v));
        };
        Ok(Self {
            kind,
            diffuse: v.diffuse,
            ambient: v.ambient,
            attenuation: v.attenuation,
            activation: enabled,
            shadows: v.shadows,
            falloff: v.falloff,
        })
    }
}

impl<K: ValidateLinks, A: ValidateLinks> ValidateLinks for AmbientDiffuseLight<K, A> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.kind.visit_links(visitor);
        self.diffuse.visit_links(visitor);
        self.ambient.visit_links(visitor);
        self.attenuation.visit_links(visitor);
        self.activation.visit_links(visitor);
        self.shadows.visit_links(visitor);
        self.falloff.visit_links(visitor);
    }
}
impl ValidateLinks for LightEmission {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.color.visit_links(visitor);
        self.intensity.visit_links(visitor);
    }
}
impl ValidateLinks for LightShadows {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.range.visit_links(visitor);
        if self.intensity.is_some_and(|value| !value.is_finite()) {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for LightActivation {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Visibility(value) => value.visit_links(visitor),
            Self::Enabled(value) => value.visit_links(visitor),
        }
    }
}

impl ValidateLinks for IlluminationKind {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
impl ValidateLinks for mdlx::LightKind {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
impl ValidateLinks for ActorLightKind {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}

/// Spatial interpretation of a light with one emission component.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EmissionKind {
    /// Radiates from a position.
    #[default]
    Point,
    /// Illuminates along one direction.
    Directional,
    /// Radiates inside a cone.
    Spot,
    /// Supplies ambient illumination.
    Ambient,
    /// Unresolved authored interpretation.
    Unknown(u32),
}
/// One RGB emission component with optional cone and shadow parameters.
#[derive(Clone, Debug, Construct, IntoOwned)]
pub struct SingleEmissionLight {
    /// Spatial interpretation.
    pub kind: EmissionKind,
    /// Animated RGB color and intensity.
    pub emission: LightEmission,
    /// Range attenuation start and end.
    pub attenuation: [Property<f32>; 2],
    /// Inner and outer cone angles.
    pub cone: [Property<f32>; 2],
    /// Independently authored shadow parameters.
    pub shadows: LightShadows,
    /// Quadratic, linear, and exponential attenuation coefficients.
    pub attenuation_coefficients: [Option<f32>; 3],
}
impl ValidateLinks for SingleEmissionLight {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.emission.visit_links(visitor);
        self.attenuation.visit_links(visitor);
        self.cone.visit_links(visitor);
        self.shadows.visit_links(visitor);
    }
}
