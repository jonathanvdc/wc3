//! Borrowed behavior discovery across semantic feature vocabularies.
//!
//! Views group features by their purpose. Typed alternatives retain authored simulation,
//! projection, illumination, and resource semantics without allocating or converting values.
use crate::model::profiles::{m2, mdlx, wmo, Either, Unavailable};
use crate::model::{
    scene, AmbientDiffuseLight, Attachment, Camera, CameraLens, IlluminationKind, Light,
    LightActivation, LightShadows, LookAtCamera, NodeFeature, ParticleEmitter, Property,
    RibbonEmitter, UnknownIlluminationKind,
};
use crate::model::{Axis, ChildModel, Socket};
use crate::model::{EffectResourceId, PerspectiveProjection};
use crate::model::{NodeCamera, SingleEmissionLight};
use glam::Vec3;
use std::fmt::Debug;

/// A vocabulary exposing the behavior of an attached feature without conversion.
pub trait FeatureSchema<'data> {
    /// Inspects the authored behavior, borrowing all payload storage.
    fn feature(&self) -> FeatureView<'_, 'data>;
}
/// Camera pose interpretations, independent of source profile.
#[derive(Clone, Copy, Debug)]
pub enum CameraView<'view> {
    /// Camera carried by a hierarchy node.
    Node(&'view NodeCamera),
    /// Model-space eye, target, and roll.
    LookAt(LookAtCameraRef<'view>),
}
/// Illumination interpretations, independent of source profile where meanings agree.
#[derive(Clone, Copy, Debug)]
pub enum LightView<'view> {
    /// Independently authored ambient and direct illumination.
    AmbientDiffuse(AmbientDiffuseLightRef<'view>),
    /// RGB lighting with cone and shadow parameters.
    SingleEmission(&'view SingleEmissionLight),
    /// World lighting with RGBA color and cookie extensions.
    World(&'view wmo::Light),
    /// Region-local point illumination.
    RegionPoint(&'view wmo::PointLight),
    /// Region-local lighting program.
    RegionProgram(&'view wmo::PointProgram),
}
/// Authored alternatives for attachment behavior.
#[derive(Clone, Copy, Debug)]
pub enum AttachmentView<'view, 'data> {
    /// Shared attachment point identity and activation.
    Socket(&'view Socket<'data>),
    /// Shared external resource placement definition.
    ChildDefinition(&'view ChildModel<'data>),
    /// Indexed socket and external child-model dependency.
    External(&'view scene::Attachment),
    /// Complete Role definition, retaining its authored semantics.
    Role(&'view m2::Attachment),
}
/// Authored alternatives for particles behavior.
#[derive(Clone, Copy, Debug)]
pub enum ParticlesView<'view, 'data> {
    /// Indexed socket and external child-model dependency.
    External(&'view EffectResourceId),
    /// Complete Model definition, retaining its authored semantics.
    MdlxEmitter1(&'view mdlx::ParticleEmitter1),
    /// Complete Sprite definition, retaining its authored semantics.
    MdlxEmitter2(&'view mdlx::ParticleEmitter2),
    /// Complete Popcorn definition, retaining its authored semantics.
    Popcorn(&'view mdlx::Popcorn<'data>),
    /// Complete M2 definition, retaining its authored semantics.
    M2(&'view m2::ParticleEmitter<'data>),
}
/// Authored alternatives for ribbon behavior.
#[derive(Clone, Copy, Debug)]
pub enum RibbonView<'view> {
    /// Indexed socket and external child-model dependency.
    External(&'view EffectResourceId),
    /// Complete Mdlx definition, retaining its authored semantics.
    Mdlx(&'view mdlx::RibbonEmitter),
    /// Complete M2 definition, retaining its authored semantics.
    M2(&'view m2::RibbonEmitter),
}
/// Authored alternatives for collision behavior.
#[derive(Clone, Copy, Debug)]
pub enum CollisionView<'view> {
    /// Complete Mdlx definition, retaining its authored semantics.
    Primitive(&'view mdlx::Collision),
}
/// Complete behavior-oriented borrowed inspection of an attached feature.
#[derive(Clone, Copy, Debug)]
pub enum FeatureView<'view, 'data> {
    /// Application-defined behavior outside the built-in inspection vocabulary.
    /// Its concrete profile payload remains available to application consumers.
    Custom(&'view (dyn Debug + Send + Sync)),
    /// Camera behavior.
    Camera(CameraView<'view>),
    /// Light behavior.
    Light(LightView<'view>),
    /// Attachment behavior.
    Attachment(AttachmentView<'view, 'data>),
    /// Particles behavior.
    Particles(ParticlesView<'view, 'data>),
    /// Ribbon behavior.
    Ribbon(RibbonView<'view>),
    /// Collision behavior.
    Collision(CollisionView<'view>),
    /// Complete GeometryAnimation behavior.
    GeometryAnimation(&'view mdlx::GeometryAnimation),
    /// Complete Doodad behavior.
    Doodad(&'view wmo::Doodad),
    /// Complete FaceActor behavior.
    FaceActor(&'view EffectResourceId),
}
impl<'data> mdlx::Node<'data> {
    pub(crate) fn profile_feature(&self) -> FeatureView<'_, 'data> {
        match self {
            Self::GeometryAnimation(value) => FeatureView::GeometryAnimation(value),
            Self::Particle(value) => FeatureView::Particles(ParticlesView::MdlxEmitter1(value)),
            Self::Particle2(value) => FeatureView::Particles(ParticlesView::MdlxEmitter2(value)),
            Self::Ribbon(value) => FeatureView::Ribbon(RibbonView::Mdlx(value)),
            Self::ParticleProgram(value) => FeatureView::Particles(ParticlesView::Popcorn(value)),
            Self::Light(value) => FeatureView::Light(LightView::AmbientDiffuse(value.into())),
            Self::Camera(value) => FeatureView::Camera(CameraView::LookAt(value.into())),
            Self::Attachment(value) => FeatureView::Attachment(AttachmentView::External(value)),
            Self::CollisionShape(value) => FeatureView::Collision(CollisionView::Primitive(value)),

            Self::FaceActor(unavailable)
            | Self::LightProgram(unavailable)
            | Self::PointLight(unavailable) => match *unavailable {},
        }
    }
}
impl<'data> m2::ActorKind<'data> {
    pub(crate) fn profile_feature(&self) -> FeatureView<'_, 'data> {
        match self {
            Self::Attachment(value) => FeatureView::Attachment(AttachmentView::Role(value)),
            Self::Light(value) => FeatureView::Light(LightView::AmbientDiffuse(value.into())),
            Self::Camera(value) => FeatureView::Camera(CameraView::LookAt(value.into())),
            Self::Ribbon(value) => FeatureView::Ribbon(RibbonView::M2(value)),
            Self::Particle(value) => FeatureView::Particles(ParticlesView::M2(value)),

            Self::GeometryAnimation(unavailable)
            | Self::CollisionShape(unavailable)
            | Self::ParticleProgram(unavailable)
            | Self::FaceActor(unavailable)
            | Self::LightProgram(unavailable)
            | Self::Particle2(unavailable)
            | Self::PointLight(unavailable) => match *unavailable {},
        }
    }
}
impl<'data> wmo::Node<'data> {
    pub(crate) fn profile_feature(&self) -> FeatureView<'_, 'data> {
        match self {
            Self::Attachment(value) => FeatureView::Doodad(value),
            Self::Light(value) => FeatureView::Light(LightView::World(value)),
            Self::PointLight(value) => FeatureView::Light(LightView::RegionPoint(value)),
            Self::LightProgram(value) => FeatureView::Light(LightView::RegionProgram(value)),

            Self::Camera(unavailable)
            | Self::Particle(unavailable)
            | Self::Ribbon(unavailable)
            | Self::GeometryAnimation(unavailable)
            | Self::CollisionShape(unavailable)
            | Self::ParticleProgram(unavailable)
            | Self::FaceActor(unavailable)
            | Self::Particle2(unavailable) => match *unavailable {},
        }
    }
}

impl<'data> FeatureSchema<'data> for scene::Feature {
    fn feature(&self) -> FeatureView<'_, 'data> {
        match self {
            Self::Camera(value) => FeatureView::Camera(CameraView::Node(value)),
            Self::Light(value) => FeatureView::Light(LightView::SingleEmission(value)),
            Self::Attachment(value) => FeatureView::Attachment(AttachmentView::External(value)),
            Self::Particles(value) => FeatureView::Particles(ParticlesView::External(value)),
            Self::Ribbon(value) => FeatureView::Ribbon(RibbonView::External(value)),
            Self::FaceActor(value) => FeatureView::FaceActor(value),
        }
    }
}
impl<'data, A: FeatureSchema<'data>, B: FeatureSchema<'data>> FeatureSchema<'data>
    for Either<A, B>
{
    fn feature(&self) -> FeatureView<'_, 'data> {
        match self {
            Self::Left(value) => value.feature(),
            Self::Right(value) => value.feature(),
        }
    }
}
impl<'data> FeatureSchema<'data> for Unavailable {
    fn feature(&self) -> FeatureView<'_, 'data> {
        match *self {}
    }
}

#[cfg(test)]
#[path = "feature_schema/tests.rs"]
mod tests;

/// Borrowed color and intensity of one illumination component.
#[derive(Clone, Copy, Debug)]
pub struct IlluminationRef<'view> {
    /// RGB color animation.
    pub color: &'view Property<Vec3>,
    /// Intensity animation.
    pub intensity: &'view Property<f32>,
}
/// Borrowed authored activation without changing its numeric domain.
#[derive(Clone, Copy, Debug)]
pub enum LightActivationRef<'view> {
    /// Fractional visibility.
    Visibility(&'view Property<f32>),
    /// Boolean activation.
    Enabled(&'view Property<bool>),
}
/// Shared ambient/diffuse illumination, including its complete authored parameters.
#[derive(Clone, Copy, Debug)]
pub struct AmbientDiffuseLightRef<'view> {
    /// Spatial interpretation.
    pub kind: IlluminationKind,
    /// Ambient illumination.
    pub ambient: IlluminationRef<'view>,
    /// Direct illumination.
    pub diffuse: IlluminationRef<'view>,
    /// Attenuation start and end.
    pub attenuation: [&'view Property<f32>; 2],
    /// Authored activation.
    pub activation: LightActivationRef<'view>,
    /// Independent shadow parameters.
    pub shadows: &'view LightShadows,
    /// Optional attenuation coefficients.
    pub falloff: Option<&'view [Property<f32>; 3]>,
}
macro_rules! light_ref {
    ($source:ty, $kind:expr, $activation:expr) => {
        impl<'a> From<&'a $source> for AmbientDiffuseLightRef<'a> {
            fn from(value: &'a $source) -> Self {
                Self {
                    kind: ($kind)(value.kind),
                    ambient: IlluminationRef {
                        color: &value.ambient.color,
                        intensity: &value.ambient.intensity,
                    },
                    diffuse: IlluminationRef {
                        color: &value.diffuse.color,
                        intensity: &value.diffuse.intensity,
                    },
                    attenuation: value.attenuation.each_ref(),
                    activation: ($activation)(&value.activation),
                    shadows: &value.shadows,
                    falloff: value.falloff.as_ref(),
                }
            }
        }
    };
}
fn activation_ref(activation: &LightActivation) -> LightActivationRef<'_> {
    match activation {
        LightActivation::Visibility(value) => LightActivationRef::Visibility(value),
        LightActivation::Enabled(value) => LightActivationRef::Enabled(value),
    }
}
light_ref!(AmbientDiffuseLight, |kind| kind, activation_ref);
light_ref!(
    mdlx::Light,
    |kind| match kind {
        mdlx::LightKind::Point => IlluminationKind::Point,
        mdlx::LightKind::Directional => IlluminationKind::Directional,
        mdlx::LightKind::Ambient => IlluminationKind::Ambient,
        mdlx::LightKind::Unknown(value) =>
            IlluminationKind::Unknown(UnknownIlluminationKind::Mdlx(value)),
    },
    LightActivationRef::Visibility
);
light_ref!(
    m2::Light,
    |kind| match kind {
        m2::values::LightKind::Point => IlluminationKind::Point,
        m2::values::LightKind::Directional => IlluminationKind::Directional,
        m2::values::LightKind::Unknown(value) =>
            IlluminationKind::Unknown(UnknownIlluminationKind::M2(value)),
    },
    LightActivationRef::Enabled
);
impl<'view> LightView<'view> {
    /// Borrows ambient/diffuse illumination when this light uses that interpretation.
    pub fn ambient_diffuse(self) -> Option<AmbientDiffuseLightRef<'view>> {
        match self {
            Self::AmbientDiffuse(value) => Some(value),
            _ => None,
        }
    }
}
/// Static or animated angle, independent of the projection axis.
#[derive(Clone, Copy, Debug)]
pub enum CameraAngleRef<'view> {
    /// Static angle in radians.
    Constant(&'view f32),
    /// Animated angle in radians.
    Animated(&'view Property<f32>),
}
/// Borrowed look-at camera, independent of its native profile.
#[derive(Clone, Copy, Debug)]
pub struct LookAtCameraRef<'view> {
    /// Eye displacement from the carrying node's rest translation.
    pub eye_offset: &'view Property<Vec3>,
    /// Model-space base target.
    pub target: &'view Vec3,
    /// Additive target displacement.
    pub target_offset: &'view Property<Vec3>,
    /// Roll angle.
    pub roll: &'view Property<f32>,
    /// Shared perspective projection with a borrowed static or animated angle.
    pub projection: PerspectiveProjection<CameraAngleRef<'view>>,
    /// Optional application role.
    pub role: Option<m2::values::CameraKind>,
    /// Optional authored visibility.
    pub visibility: Option<&'view Property<f32>>,
    /// Optical parameters.
    pub lens: &'view CameraLens,
}
macro_rules! camera_ref {
    ($angle:ty, $variant:ident) => {
        impl<'a> From<&'a LookAtCamera<$angle>> for LookAtCameraRef<'a> {
            fn from(value: &'a LookAtCamera<$angle>) -> Self {
                Self {
                    eye_offset: &value.eye_offset,
                    target: &value.target,
                    target_offset: &value.target_offset,
                    roll: &value.roll,
                    projection: PerspectiveProjection {
                        field_of_view: CameraAngleRef::$variant(&value.projection.field_of_view),
                        axis: value.projection.axis,
                        clipping: value.projection.clipping,
                    },
                    role: value.role,
                    visibility: value.visibility.as_ref(),
                    lens: &value.lens,
                }
            }
        }
    };
}
camera_ref!(f32, Constant);
camera_ref!(Property<f32>, Animated);
impl<'view> CameraView<'view> {
    /// Borrows a look-at camera when this camera uses that pose interpretation.
    pub fn look_at(self) -> Option<LookAtCameraRef<'view>> {
        match self {
            Self::LookAt(value) => Some(value),
            _ => None,
        }
    }
}

impl<'data> NodeFeature<'data> {
    pub(crate) fn profile_feature(&self) -> FeatureView<'_, 'data> {
        match self {
            Self::Camera(v) => FeatureView::Camera(match v {
                Camera::Node(v) => CameraView::Node(v),
                Camera::LookAt(v) => CameraView::LookAt((&**v).into()),
            }),
            Self::Light(v) => FeatureView::Light(match v {
                Light::SingleEmission(v) => LightView::SingleEmission(v),
                Light::AmbientDiffuse(v) => LightView::AmbientDiffuse((&**v).into()),

                Light::World(v) => LightView::World(v),
                Light::Point(v) => LightView::RegionPoint(v),
            }),
            Self::Attachment(v) => match v {
                Attachment::Child(v) => FeatureView::Attachment(AttachmentView::ChildDefinition(v)),
                Attachment::Socket(v) => FeatureView::Attachment(AttachmentView::Socket(v)),
            },
            Self::Particle(v) => FeatureView::Particles(match v {
                ParticleEmitter::Resource(v) => ParticlesView::External(v),
                ParticleEmitter::MdlxEmitter1(v) => ParticlesView::MdlxEmitter1(v),
                ParticleEmitter::MdlxEmitter2(v) => ParticlesView::MdlxEmitter2(v),
                ParticleEmitter::M2(v) => ParticlesView::M2(v),
            }),
            Self::Ribbon(v) => FeatureView::Ribbon(match v {
                RibbonEmitter::Resource(v) => RibbonView::External(v),
                RibbonEmitter::Mdlx(v) => RibbonView::Mdlx(v),
                RibbonEmitter::M2(v) => RibbonView::M2(v),
            }),
            Self::CollisionShape(v) => FeatureView::Collision(CollisionView::Primitive(v)),

            Self::ParticleProgram(v) => FeatureView::Particles(ParticlesView::Popcorn(v)),

            Self::LightProgram(v) => FeatureView::Light(LightView::RegionProgram(v)),
            Self::GeometryAnimation(v) => FeatureView::GeometryAnimation(v),
            Self::FaceActor(v) => FeatureView::FaceActor(v),

            Self::Particle2(unavailable) | Self::PointLight(unavailable) => match *unavailable {},
        }
    }
}

/// Authored emission timing; a burst count is distinct from particles per second.
#[derive(Clone, Copy, Debug)]
pub enum EmissionRef<'view> {
    /// Particles per second evaluated on the model clock.
    Continuous(&'view Property<f32>),
    /// Particle count emitted at emission-property keys.
    BurstAtKeys(&'view Property<f32>),
    /// Emission-rate input interpreted by an external effect program.
    ProgramInput(&'view Property<f32>),
}
/// Nominal lifetime control, preserving animation metadata and native program semantics.
#[derive(Clone, Copy, Debug)]
pub enum LifetimeRef<'view> {
    /// Fixed persistence in seconds.
    Seconds(&'view f32),
    /// Model-clock animation of persistence in seconds.
    AnimatedSeconds(&'view Property<f32>),
    /// Lifetime input interpreted by an external effect program.
    ProgramInput(&'view Property<f32>),
}
impl<'view, 'data> ParticlesView<'view, 'data> {
    /// Inspects authored emission controls without conflating continuous rates and bursts.
    /// External resources have no local definition; copies expose their local overrides.
    pub fn emission(self) -> Option<EmissionRef<'view>> {
        match self {
            Self::External(_) => None,
            Self::MdlxEmitter1(v) => Some(EmissionRef::Continuous(&v.emission.rate)),
            Self::MdlxEmitter2(v) if v.emission.trigger => {
                Some(EmissionRef::BurstAtKeys(&v.emission.rate))
            }
            Self::MdlxEmitter2(v) => Some(EmissionRef::Continuous(&v.emission.rate)),
            Self::Popcorn(v) => Some(EmissionRef::ProgramInput(&v.emission_rate)),
            Self::M2(v) => Some(EmissionRef::Continuous(&v.emission.rate)),
        }
    }
    /// Inspects nominal particle persistence. Random variation remains on the full definition.
    /// Copies inherit lifetime from their referenced emitter, so require graph resolution.
    pub fn lifetime(self) -> Option<LifetimeRef<'view>> {
        match self {
            Self::External(_) => None,
            Self::MdlxEmitter1(v) => Some(LifetimeRef::AnimatedSeconds(&v.lifetime.duration)),
            Self::MdlxEmitter2(v) => Some(LifetimeRef::Seconds(&v.lifetime.duration)),
            Self::Popcorn(v) => Some(LifetimeRef::ProgramInput(&v.life_span)),
            Self::M2(v) => Some(LifetimeRef::AnimatedSeconds(&v.lifetime.duration)),
        }
    }
}
/// Trail sections per second, retaining the authored numeric domain.
#[derive(Clone, Copy, Debug)]
pub enum TrailRateRef<'view> {
    /// Integral section frequency.
    Integral(&'view u32),
    /// Fractional edge frequency.
    Fractional(&'view f32),
}
/// Animated atlas-cell selection with its complete authored integer domain.
#[derive(Clone, Copy, Debug)]
pub enum AtlasCellRef<'view> {
    /// Full-width cell selection.
    U32(&'view Property<u32>),
    /// Sixteen-bit cell selection.
    U16(&'view Property<u16>),
}
/// Shared section-based ribbon controls, borrowing complete model-clock animations.
#[derive(Clone, Copy, Debug)]
pub struct EdgeStripRef<'view> {
    /// Emitter-local direction of positive cross-section extent.
    pub axis: &'view Axis,
    /// RGB tint of emitted trail geometry.
    pub color: &'view Property<Vec3>,
    /// Fractional opacity.
    pub opacity: &'view Property<f32>,
    /// Extent above and below the center line.
    pub height: [&'view Property<f32>; 2],
    /// Persistence of each emitted section in seconds.
    pub lifetime_seconds: &'view f32,
    /// Section generation frequency.
    pub emission_rate: TrailRateRef<'view>,
    /// Atlas cell selected on the model clock.
    pub atlas_cell: AtlasCellRef<'view>,
}
impl<'view> RibbonView<'view> {
    /// Inspects common section-based trail controls.
    pub fn edge_strip(self) -> Option<EdgeStripRef<'view>> {
        match self {
            Self::Mdlx(v) => Some(EdgeStripRef {
                axis: &v.geometry.axis,
                color: &v.appearance.color,
                opacity: &v.appearance.opacity,
                height: [&v.geometry.height_above, &v.geometry.height_below],
                lifetime_seconds: &v.lifetime,
                emission_rate: TrailRateRef::Integral(&v.emission_rate),
                atlas_cell: AtlasCellRef::U32(&v.appearance.texture_slot),
            }),
            Self::M2(v) => Some(EdgeStripRef {
                axis: &v.geometry.axis,
                color: &v.appearance.color,
                opacity: &v.appearance.opacity,
                height: [&v.geometry.height_above, &v.geometry.height_below],
                lifetime_seconds: &v.lifetime,
                emission_rate: TrailRateRef::Fractional(&v.emission_rate),
                atlas_cell: AtlasCellRef::U16(&v.appearance.texture_slot),
            }),

            Self::External(_) => None,
        }
    }
    /// Inspects nominal ribbon persistence in seconds.
    pub fn lifetime(self) -> Option<LifetimeRef<'view>> {
        match self {
            Self::External(_) => None,
            Self::Mdlx(v) => Some(LifetimeRef::Seconds(&v.lifetime)),
            Self::M2(v) => Some(LifetimeRef::Seconds(&v.lifetime)),
        }
    }
}
