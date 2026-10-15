//! Shared illumination components with explicit activation semantics.
use crate::model::semantic::profiles::{m2, m3, mdlx, wmo};
use crate::model::semantic::{scene, Animated};
use m2::values::LightKind as ActorLightKind;
/// Complete illumination models.
#[derive(Clone, Debug)]
pub enum Light {
    /// RGB lighting with cone and shadow controls.
    Portable(Box<scene::Light>),
    /// Independently animated ambient and direct illumination.
    AmbientDiffuse(Box<AmbientDiffuseLight>),
    /// Independent diffuse and specular response with property metadata.
    Specular(Box<m3::Light>),
    /// World lighting with RGBA color and cookie extensions.
    World(Box<wmo::Light>),
    /// Region-local point illumination.
    Point(wmo::PointLight),
}
/// Directional interpretation shared by ambient/diffuse light programs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Debug)]
pub struct LightEmission {
    /// RGB emission color.
    pub color: Animated<[f32; 3]>,
    /// Emission intensity multiplier.
    pub intensity: Animated<f32>,
}
/// Shared ambient/diffuse light fields.
#[derive(Clone, Debug)]
pub struct AmbientDiffuseLight {
    /// Spatial emission interpretation.
    pub kind: IlluminationKind,
    /// Direct illumination.
    pub diffuse: LightEmission,
    /// Ambient illumination.
    pub ambient: LightEmission,
    /// Beginning and ending distance attenuation.
    pub attenuation: [Animated<f32>; 2],
    /// Fractional visibility or discrete activation, including supported shadow controls.
    pub activation: LightActivation,
}
/// Distinct authored activation programs.
#[derive(Clone, Debug)]
pub enum LightActivation {
    /// Fractional visibility with independently authored shadow and falloff controls.
    Visibility(Box<VisibleLight>),
    /// Boolean activation.
    Enabled(Animated<bool>),
}
/// Fractional light visibility and optional shadow/falloff controls.
#[derive(Clone, Debug)]
pub struct VisibleLight {
    /// Visibility multiplier.
    pub visibility: Animated<f32>,
    /// Explicit shadow casting.
    pub shadow_casting: Option<bool>,
    /// Explicit shadow intensity.
    pub shadow_intensity: Option<f32>,
    /// Animated shadow near/far distances.
    pub shadow_range: Option<[Animated<f32>; 2]>,
    /// Quadratic, linear, and damping attenuation coefficients.
    pub falloff: Option<[Animated<f32>; 3]>,
}
impl From<mdlx::Light> for AmbientDiffuseLight {
    fn from(v: mdlx::Light) -> Self {
        Self {
            kind: match v.kind {
                mdlx::LightKind::Omni => IlluminationKind::Point,
                mdlx::LightKind::Directional => IlluminationKind::Directional,
                mdlx::LightKind::Ambient => IlluminationKind::Ambient,
                mdlx::LightKind::Unknown(v) => {
                    IlluminationKind::Unknown(UnknownIlluminationKind::Mdlx(v))
                }
            },
            diffuse: LightEmission {
                color: v.color,
                intensity: v.intensity,
            },
            ambient: LightEmission {
                color: v.ambient_color,
                intensity: v.ambient_intensity,
            },
            attenuation: v.attenuation,
            activation: LightActivation::Visibility(Box::new(VisibleLight {
                visibility: v.visibility,
                shadow_casting: v.shadow_casting,
                shadow_intensity: v.shadow_intensity,
                shadow_range: v.shadow_range,
                falloff: v.falloff,
            })),
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
            diffuse: LightEmission {
                color: v.diffuse_color,
                intensity: v.diffuse_intensity,
            },
            ambient: LightEmission {
                color: v.ambient_color,
                intensity: v.ambient_intensity,
            },
            attenuation: [v.attenuation_start, v.attenuation_end],
            activation: LightActivation::Enabled(v.enabled),
        }
    }
}
impl TryFrom<AmbientDiffuseLight> for mdlx::Light {
    type Error = Box<AmbientDiffuseLight>;
    fn try_from(v: AmbientDiffuseLight) -> Result<Self, Self::Error> {
        let kind = match v.kind {
            IlluminationKind::Point => mdlx::LightKind::Omni,
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
            color: v.diffuse.color,
            intensity: v.diffuse.intensity,
            ambient_color: v.ambient.color,
            ambient_intensity: v.ambient.intensity,
            attenuation: v.attenuation,
            visibility: control.visibility,
            shadow_casting: control.shadow_casting,
            shadow_intensity: control.shadow_intensity,
            shadow_range: control.shadow_range,
            falloff: control.falloff,
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
        let LightActivation::Enabled(enabled) = v.activation else {
            return Err(Box::new(v));
        };
        let [attenuation_start, attenuation_end] = v.attenuation;
        Ok(Self {
            kind,
            diffuse_color: v.diffuse.color,
            diffuse_intensity: v.diffuse.intensity,
            ambient_color: v.ambient.color,
            ambient_intensity: v.ambient.intensity,
            attenuation_start,
            attenuation_end,
            enabled,
        })
    }
}
