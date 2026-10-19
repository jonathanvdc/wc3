//! Borrowed behavior discovery across semantic feature vocabularies.
//!
//! Views group features by their purpose. Typed alternatives retain authored simulation,
//! projection, illumination, and resource semantics without allocating or converting values.
use crate::model::profiles::{m2, mdlx, wmo, Either, Unavailable};
use crate::model::EffectResourceId;
use crate::model::{
    scene, AimedCamera, AmbientDiffuseLight, Attachment, Camera, CameraConvention, Light,
    NodeFeature, ParticleEmitter, Property, RibbonEmitter,
};
use crate::model::{Axis, ChildModel, Socket};
use glam::{Vec2, Vec3};
use std::fmt::Debug;

/// A vocabulary exposing the behavior of an attached feature without conversion.
pub trait FeatureSchema<'data> {
    /// Inspects the authored behavior, borrowing all payload storage.
    fn feature(&self) -> FeatureView<'_, 'data>;
}
/// Authored alternatives for camera behavior.
#[derive(Clone, Copy, Debug)]
pub enum CameraView<'view> {
    /// Shared aim-point camera definition.
    Aimed(&'view AimedCamera),
    /// Complete Scene definition, retaining its authored semantics.
    Scene(&'view scene::Camera),
    /// Complete Mdlx definition, retaining its authored semantics.
    VerticalAim(&'view mdlx::Camera),
    /// Complete M2 definition, retaining its authored semantics.
    DiagonalAim(&'view m2::Camera),
}
/// Authored alternatives for light behavior.
#[derive(Clone, Copy, Debug)]
pub enum LightView<'view> {
    /// Shared independently authored ambient and diffuse illumination.
    AmbientDiffuse(&'view AmbientDiffuseLight),
    /// Complete Scene definition, retaining its authored semantics.
    Scene(&'view scene::Light),
    /// Complete Mdlx definition, retaining its authored semantics.
    VisibleAmbientDiffuse(&'view mdlx::Light),
    /// Complete M2 definition, retaining its authored semantics.
    EnabledAmbientDiffuse(&'view m2::Light),

    /// Complete World definition, retaining its authored semantics.
    World(&'view wmo::Light),
    /// Complete RegionPoint definition, retaining its authored semantics.
    RegionPoint(&'view wmo::PointLight),
    /// Complete RegionProgram definition, retaining its authored semantics.
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
            Self::Light(value) => FeatureView::Light(LightView::VisibleAmbientDiffuse(value)),
            Self::Camera(value) => FeatureView::Camera(CameraView::VerticalAim(value)),
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
            Self::Light(value) => FeatureView::Light(LightView::EnabledAmbientDiffuse(value)),
            Self::Camera(value) => FeatureView::Camera(CameraView::DiagonalAim(value)),
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
            Self::Camera(value) => FeatureView::Camera(CameraView::Scene(value)),
            Self::Light(value) => FeatureView::Light(LightView::Scene(value)),
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

/// Borrowed color and intensity of one independently authored illumination component.
#[derive(Clone, Copy, Debug)]
pub struct IlluminationRef<'view> {
    /// RGB color animation.
    pub color: &'view Property<Vec3>,
    /// Intensity animation.
    pub intensity: &'view Property<f32>,
}
/// Shared ambient/diffuse authoring semantics across native and Universal lights.
#[derive(Clone, Copy, Debug)]
pub struct AmbientDiffuseLightRef<'view> {
    /// Ambient illumination independent of surface-facing direction.
    pub ambient: IlluminationRef<'view>,
    /// Direct illumination component.
    pub diffuse: IlluminationRef<'view>,
    /// Attenuation start and end animations.
    pub attenuation: [&'view Property<f32>; 2],
}
impl<'view> LightView<'view> {
    /// Inspects shared ambient/diffuse components without knowing their source vocabulary.
    pub fn ambient_diffuse(self) -> Option<AmbientDiffuseLightRef<'view>> {
        match self {
            Self::AmbientDiffuse(value) => Some(AmbientDiffuseLightRef {
                ambient: IlluminationRef {
                    color: &value.ambient.color,
                    intensity: &value.ambient.intensity,
                },
                diffuse: IlluminationRef {
                    color: &value.diffuse.color,
                    intensity: &value.diffuse.intensity,
                },
                attenuation: [&value.attenuation[0], &value.attenuation[1]],
            }),
            Self::VisibleAmbientDiffuse(value) => Some(AmbientDiffuseLightRef {
                ambient: IlluminationRef {
                    color: &value.ambient.color,
                    intensity: &value.ambient.intensity,
                },
                diffuse: IlluminationRef {
                    color: &value.diffuse.color,
                    intensity: &value.diffuse.intensity,
                },
                attenuation: [&value.attenuation[0], &value.attenuation[1]],
            }),
            Self::EnabledAmbientDiffuse(value) => Some(AmbientDiffuseLightRef {
                ambient: IlluminationRef {
                    color: &value.ambient.color,
                    intensity: &value.ambient.intensity,
                },
                diffuse: IlluminationRef {
                    color: &value.diffuse.color,
                    intensity: &value.diffuse.intensity,
                },
                attenuation: [&value.attenuation[0], &value.attenuation[1]],
            }),
            _ => None,
        }
    }
}
/// A field of view preserving static versus animated authoring and axis convention.
#[derive(Clone, Copy, Debug)]
pub enum AimedFieldOfViewRef<'view> {
    /// Animated vertical angle in radians.
    AnimatedVertical(&'view Property<f32>),
    /// Static vertical angle in radians.
    Vertical(&'view f32),
    /// Animated diagonal angle in radians.
    Diagonal(&'view Property<f32>),
}
/// Shared additive eye/target and roll semantics of authored aim-point cameras.
#[derive(Clone, Copy, Debug)]
pub struct AimedCameraRef<'view> {
    /// Offset from the carrying node's rest translation.
    pub eye_offset: &'view Property<Vec3>,
    /// Model-space base target.
    pub target: &'view Vec3,
    /// Additive offset from the base target.
    pub target_offset: &'view Property<Vec3>,
    /// Roll angle in radians.
    pub roll: &'view Property<f32>,
    /// Near and far clipping distances.
    pub clipping: &'view Vec2,
    /// Angle and its explicitly authored axis convention.
    pub field_of_view: AimedFieldOfViewRef<'view>,
}
impl<'view> CameraView<'view> {
    /// Inspects shared aimed-camera components while preserving field-of-view conventions.
    pub fn aimed(self) -> Option<AimedCameraRef<'view>> {
        match self {
            Self::Aimed(value) => Some(AimedCameraRef {
                eye_offset: &value.eye_offset,
                target: &value.target,
                target_offset: &value.target_offset,
                roll: &value.roll,
                clipping: &value.clipping,
                field_of_view: match &value.convention {
                    CameraConvention::Vertical(_) => {
                        AimedFieldOfViewRef::AnimatedVertical(&value.field_of_view)
                    }
                    CameraConvention::Diagonal(_) => {
                        AimedFieldOfViewRef::Diagonal(&value.field_of_view)
                    }
                },
            }),
            Self::VerticalAim(value) => Some(AimedCameraRef {
                eye_offset: &value.eye_offset,
                target: &value.target,
                target_offset: &value.target_offset,
                roll: &value.roll,
                clipping: &value.clipping,
                field_of_view: AimedFieldOfViewRef::Vertical(&value.field_of_view),
            }),
            Self::DiagonalAim(value) => Some(AimedCameraRef {
                eye_offset: &value.eye_offset,
                target: &value.target,
                target_offset: &value.target_offset,
                roll: &value.roll,
                clipping: &value.clipping,
                field_of_view: AimedFieldOfViewRef::Diagonal(&value.field_of_view),
            }),
            _ => None,
        }
    }
}

impl<'data> NodeFeature<'data> {
    pub(crate) fn profile_feature(&self) -> FeatureView<'_, 'data> {
        match self {
            Self::Camera(v) => FeatureView::Camera(match v {
                Camera::Portable(v) => CameraView::Scene(v),
                Camera::Aimed(v) => CameraView::Aimed(v),
            }),
            Self::Light(v) => FeatureView::Light(match v {
                Light::Portable(v) => LightView::Scene(v),
                Light::AmbientDiffuse(v) => LightView::AmbientDiffuse(v),

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
