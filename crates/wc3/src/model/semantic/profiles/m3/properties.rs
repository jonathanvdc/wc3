//! Typed StarCraft II destinations for hierarchy and animated bounds.
use crate::model::semantic::profiles::M3;
use crate::model::semantic::{FeatureId, NodeId, OwnerId, PropertyKey, PropertyPayload};
use crate::model::semantic::{FeatureProperty, NodeFeature, PropertyIdentityKey, PropertyKeyRef};
use std::any::TypeId;
use std::sync::Arc;
/// Animated hierarchy batching selection.
pub const BATCHING: PropertyKey<NodeId, u32, M3> = PropertyKey::new(Target::Batching);
/// Animated minimum of a bounding declaration.
pub const EXTENT_MINIMUM: PropertyKey<FeatureId, [f32; 3], M3> =
    PropertyKey::new(Target::ExtentMinimum);
/// Animated maximum of a bounding declaration.
pub const EXTENT_MAXIMUM: PropertyKey<FeatureId, [f32; 3], M3> =
    PropertyKey::new(Target::ExtentMaximum);
/// Animated radius of a bounding declaration.
pub const EXTENT_RADIUS: PropertyKey<FeatureId, f32, M3> = PropertyKey::new(Target::ExtentRadius);

/// Structural destinations owned by this property vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    /// A nested owner field, addressed through explicit field and ordinal segments.
    Field(FieldPath),
    /// Animated batching.
    Batching,
    /// Animated extent minimum.
    ExtentMinimum,
    /// Animated extent maximum.
    ExtentMaximum,
    /// Animated extent radius.
    ExtentRadius,
}

impl PropertyPayload for Target {
    fn identity(&self) -> PropertyKeyRef<'_> {
        if let Self::Field(path) = self {
            match path.segments() {
                [FieldSegment::Field(Field::Light), FieldSegment::Field(Field::DiffuseColor)] => {
                    return FeatureProperty::LightDiffuseColor.identity()
                }
                [FieldSegment::Field(Field::Light), FieldSegment::Field(Field::IntensityMultiplier)] => {
                    return FeatureProperty::LightDiffuseIntensity.identity()
                }
                [FieldSegment::Field(Field::Light), FieldSegment::Field(Field::AttenuationStart)] => {
                    return FeatureProperty::LightAttenuationStart.identity()
                }
                _ => {}
            }
        }
        PropertyKeyRef::new(self)
    }
    fn from_identity(key: &PropertyIdentityKey) -> Option<Self> {
        if let Some(value) = key.downcast_ref::<FeatureProperty>() {
            let field = match value {
                FeatureProperty::LightDiffuseColor => Field::DiffuseColor,
                FeatureProperty::LightDiffuseIntensity => Field::IntensityMultiplier,
                FeatureProperty::LightAttenuationStart => Field::AttenuationStart,
                _ => return None,
            };
            return Some(Self::Field(FieldPath::root(Field::Light).child(field)));
        }
        key.downcast_ref::<Self>().cloned()
    }

    fn value_type(&self) -> Option<TypeId> {
        Some(match self {
            Target::Field(_) => return None,
            Target::Batching => TypeId::of::<u32>(),
            Target::ExtentMinimum => TypeId::of::<[f32; 3]>(),
            Target::ExtentMaximum => TypeId::of::<[f32; 3]>(),
            Target::ExtentRadius => TypeId::of::<f32>(),
        })
    }
    fn accepts_owner(&self, owner: OwnerId) -> bool {
        match self {
            Target::Field(path) => match path.segments().first() {
                Some(FieldSegment::Field(Field::Material)) => matches!(owner, OwnerId::Material(_)),
                Some(FieldSegment::Field(Field::Physics)) => matches!(owner, OwnerId::System(_)),
                Some(FieldSegment::Field(
                    Field::Camera
                    | Field::Force
                    | Field::Light
                    | Field::Particle
                    | Field::ParticleCopy
                    | Field::Projector
                    | Field::Ribbon
                    | Field::Shadow
                    | Field::ViewVolume
                    | Field::Warp,
                )) => matches!(owner, OwnerId::Feature(_)),
                _ => false,
            },
            Target::Batching => matches!(owner, OwnerId::Node(_)),
            Target::ExtentMinimum => matches!(owner, OwnerId::Feature(_)),
            Target::ExtentMaximum => matches!(owner, OwnerId::Feature(_)),
            Target::ExtentRadius => matches!(owner, OwnerId::Feature(_)),
        }
    }
}

/// One field in the StarCraft II authoring vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Field {
    /// active.
    Active,
    /// alpha amplitude.
    AlphaAmplitude,
    /// alpha frequency.
    AlphaFrequency,
    /// alpha layer1.
    AlphaLayer1,
    /// alpha layer2.
    AlphaLayer2,
    /// alpha threshold.
    AlphaThreshold,
    /// ambient occlusion layer.
    AmbientOcclusionLayer,
    /// angular.
    Angular,
    /// aspect ratio.
    AspectRatio,
    /// attenuation start.
    AttenuationStart,
    /// avi play.
    AviPlay,
    /// avi restart.
    AviRestart,
    /// axial.
    Axial,
    /// blur angle.
    BlurAngle,
    /// blur distance max.
    BlurDistanceMax,
    /// blur map.
    BlurMap,
    /// body.
    Body,
    /// bokeh f stop.
    BokehFStop,
    /// bokeh max co c diameter.
    BokehMaxCoCDiameter,
    /// box offset x left.
    BoxOffsetXLeft,
    /// box offset x right.
    BoxOffsetXRight,
    /// box offset y back.
    BoxOffsetYBack,
    /// box offset y front.
    BoxOffsetYFront,
    /// box offset z bottom.
    BoxOffsetZBottom,
    /// box offset z top.
    BoxOffsetZTop,
    /// camera.
    Camera,
    /// cloth.
    Cloth,
    /// color.
    Color,
    /// color amplitude.
    ColorAmplitude,
    /// color diffuse.
    ColorDiffuse,
    /// color end.
    ColorEnd,
    /// color end random.
    ColorEndRandom,
    /// color frequency.
    ColorFrequency,
    /// color map.
    ColorMap,
    /// color mid.
    ColorMid,
    /// color mid random.
    ColorMidRandom,
    /// color specular.
    ColorSpecular,
    /// color start.
    ColorStart,
    /// color start random.
    ColorStartRandom,
    /// current frame.
    CurrentFrame,
    /// decal layer.
    DecalLayer,
    /// decay.
    Decay,
    /// density.
    Density,
    /// diffuse color.
    DiffuseColor,
    /// diffuse layer.
    DiffuseLayer,
    /// diffuse map.
    DiffuseMap,
    /// displacement map.
    DisplacementMap,
    /// displacement strength.
    DisplacementStrength,
    /// dof amount.
    DofAmount,
    /// emission rate.
    EmissionRate,
    /// emissive layer1.
    EmissiveLayer1,
    /// emissive layer2.
    EmissiveLayer2,
    /// environment layer.
    EnvironmentLayer,
    /// environment mask layer.
    EnvironmentMaskLayer,
    /// falloff.
    Falloff,
    /// far.
    Far,
    /// far clip.
    FarClip,
    /// far focus range.
    FarFocusRange,
    /// field of view.
    FieldOfView,
    /// flare map.
    FlareMap,
    /// focus distance.
    FocusDistance,
    /// force.
    Force,
    /// gloss layer.
    GlossLayer,
    /// groups.
    Groups,
    /// hdr.
    Hdr,
    /// height.
    Height,
    /// height layer.
    HeightLayer,
    /// horizontal amplitude.
    HorizontalAmplitude,
    /// horizontal frequency.
    HorizontalFrequency,
    /// hot spot.
    HotSpot,
    /// initial horizontal.
    InitialHorizontal,
    /// initial pitch.
    InitialPitch,
    /// initial speed.
    InitialSpeed,
    /// initial speed random.
    InitialSpeedRandom,
    /// initial vertical.
    InitialVertical,
    /// initial yaw.
    InitialYaw,
    /// inner radius.
    InnerRadius,
    /// intensity.
    Intensity,
    /// intensity multiplier.
    IntensityMultiplier,
    /// layer ambient occlusion.
    LayerAmbientOcclusion,
    /// layer base.
    LayerBase,
    /// layer spec noise.
    LayerSpecNoise,
    /// layer spec shift.
    LayerSpecShift,
    /// length.
    Length,
    /// lifetime.
    Lifetime,
    /// lifetime random.
    LifetimeRandom,
    /// light.
    Light,
    /// light map layer.
    LightMapLayer,
    /// lower bound.
    LowerBound,
    /// map alpha.
    MapAlpha,
    /// map multiplier.
    MapMultiplier,
    /// mask map.
    MaskMap,
    /// material.
    Material,
    /// max length.
    MaxLength,
    /// motion blur amount.
    MotionBlurAmount,
    /// near.
    Near,
    /// near clip.
    NearClip,
    /// near falloff end.
    NearFalloffEnd,
    /// near falloff start.
    NearFalloffStart,
    /// near focus range.
    NearFocusRange,
    /// near plane.
    NearPlane,
    /// noise map1.
    NoiseMap1,
    /// noise map2.
    NoiseMap2,
    /// normal blend1 layer.
    NormalBlend1Layer,
    /// normal blend1 mask layer.
    NormalBlend1MaskLayer,
    /// normal blend2 layer.
    NormalBlend2Layer,
    /// normal blend2 mask layer.
    NormalBlend2MaskLayer,
    /// normal blend factors.
    NormalBlendFactors,
    /// normal layer.
    NormalLayer,
    /// normal map.
    NormalMap,
    /// offset.
    Offset,
    /// outer radius.
    OuterRadius,
    /// overlay.
    Overlay,
    /// parallax height.
    ParallaxHeight,
    /// particle.
    Particle,
    /// particle velocity.
    ParticleVelocity,
    /// phase shift.
    PhaseShift,
    /// physics.
    Physics,
    /// pitch.
    Pitch,
    /// pitch amplitude.
    PitchAmplitude,
    /// pitch frequency.
    PitchFrequency,
    /// position.
    Position,
    /// projector.
    Projector,
    /// properties.
    Properties,
    /// radial.
    Radial,
    /// radius.
    Radius,
    /// reflection map.
    ReflectionMap,
    /// reflection offset.
    ReflectionOffset,
    /// reflection strength.
    ReflectionStrength,
    /// rgb add.
    RgbAdd,
    /// rgb multiply.
    RgbMultiply,
    /// ribbon.
    Ribbon,
    /// roll.
    Roll,
    /// rotation.
    Rotation,
    /// rotation amplitude.
    RotationAmplitude,
    /// rotation animation.
    RotationAnimation,
    /// rotation frequency.
    RotationFrequency,
    /// rotation random animation.
    RotationRandomAnimation,
    /// scale.
    Scale,
    /// scroll rate.
    ScrollRate,
    /// sections.
    Sections,
    /// shadow.
    Shadow,
    /// shadow clip distance.
    ShadowClipDistance,
    /// shape inner.
    ShapeInner,
    /// shape outer.
    ShapeOuter,
    /// size.
    Size,
    /// size amplitude.
    SizeAmplitude,
    /// size animation.
    SizeAnimation,
    /// size frequency.
    SizeFrequency,
    /// specular color.
    SpecularColor,
    /// specular layer.
    SpecularLayer,
    /// specular map.
    SpecularMap,
    /// specular multiplier.
    SpecularMultiplier,
    /// speed amplitude.
    SpeedAmplitude,
    /// speed frequency.
    SpeedFrequency,
    /// spline points.
    SplinePoints,
    /// squirt amount.
    SquirtAmount,
    /// state.
    State,
    /// strength.
    Strength,
    /// strength map.
    StrengthMap,
    /// sub flares.
    SubFlares,
    /// terrain map.
    TerrainMap,
    /// trail emission rate.
    TrailEmissionRate,
    /// triplanar offset.
    TriplanarOffset,
    /// triplanar scale.
    TriplanarScale,
    /// upper bound.
    UpperBound,
    /// uv angle.
    UvAngle,
    /// uv offset.
    UvOffset,
    /// uv tiling.
    UvTiling,
    /// velocity.
    Velocity,
    /// velocity amplitude.
    VelocityAmplitude,
    /// velocity base factor.
    VelocityBaseFactor,
    /// velocity end factor.
    VelocityEndFactor,
    /// velocity frequency.
    VelocityFrequency,
    /// vertical amplitude.
    VerticalAmplitude,
    /// vertical frequency.
    VerticalFrequency,
    /// view volume.
    ViewVolume,
    /// w offset.
    WOffset,
    /// w tiling.
    WTiling,
    /// warp.
    Warp,
    /// width.
    Width,
    /// yaw.
    Yaw,
    /// yaw amplitude.
    YawAmplitude,
    /// yaw frequency.
    YawFrequency,
    /// particle copy.
    ParticleCopy,
    /// size random animation.
    SizeRandomAnimation,
}
/// One structural segment in an owner-local field address.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FieldSegment {
    /// A semantic field selector.
    Field(Field),
    /// An element ordinal within the preceding collection.
    Index(usize),
}
/// A structured path through nested owner fields and indexed collections.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FieldPath(Arc<[FieldSegment]>);
impl FieldPath {
    /// Starts at an owner-local semantic field.
    pub fn root(field: Field) -> Self {
        Self(Arc::from([FieldSegment::Field(field)]))
    }
    /// Selects a child field without formatting or parsing text.
    pub fn child(&self, field: Field) -> Self {
        let mut segments = self.0.to_vec();
        segments.push(FieldSegment::Field(field));
        Self(segments.into())
    }
    /// Selects an element of the preceding collection.
    pub fn indexed(&self, index: usize) -> Self {
        let mut segments = self.0.to_vec();
        segments.push(FieldSegment::Index(index));
        Self(segments.into())
    }
    /// Structural address segments in traversal order.
    pub fn segments(&self) -> &[FieldSegment] {
        &self.0
    }
}

impl Target {
    pub(crate) fn supports_feature(&self, feature: &super::Feature<'_>) -> bool {
        match self {
            Self::ExtentMinimum | Self::ExtentMaximum | Self::ExtentRadius => {
                matches!(feature, super::Feature::Extent(_))
            }
            Self::Field(path) => {
                let root = path.segments().first();
                let matches = matches!(
                    (root, feature),
                    (
                        Some(FieldSegment::Field(Field::Camera)),
                        super::Feature::Camera(_)
                    ) | (
                        Some(FieldSegment::Field(Field::Light)),
                        super::Feature::Light(_)
                    ) | (
                        Some(FieldSegment::Field(Field::Force)),
                        super::Feature::Force(_)
                    ) | (
                        Some(FieldSegment::Field(Field::Warp)),
                        super::Feature::Warp(_)
                    ) | (
                        Some(FieldSegment::Field(Field::Projector)),
                        super::Feature::Projector(_)
                    ) | (
                        Some(FieldSegment::Field(Field::Particle)),
                        super::Feature::Particle(_)
                    ) | (
                        Some(FieldSegment::Field(Field::ParticleCopy)),
                        super::Feature::ParticleCopy(_)
                    ) | (
                        Some(FieldSegment::Field(Field::Ribbon)),
                        super::Feature::Ribbon(_)
                    ) | (
                        Some(FieldSegment::Field(Field::Shadow)),
                        super::Feature::ShadowBox(_)
                    ) | (
                        Some(FieldSegment::Field(Field::ViewVolume)),
                        super::Feature::ViewVolume(_)
                    )
                );
                if !matches {
                    return false;
                }
                if let [_, FieldSegment::Field(Field::SplinePoints), FieldSegment::Index(index), ..] =
                    path.segments()
                {
                    return match feature {
                        super::Feature::Particle(v) => *index < v.spline_points.len(),
                        super::Feature::Ribbon(v) => *index < v.spline_points.len(),
                        _ => false,
                    };
                }
                true
            }
            _ => false,
        }
    }
    pub(crate) fn supports_material(&self) -> bool {
        matches!(self, Self::Field(path) if matches!(path.segments().first(), Some(FieldSegment::Field(Field::Material))))
    }
}

impl Target {
    pub(crate) fn supports_universal_feature(&self, feature: &NodeFeature<'_>) -> bool {
        use crate::model::semantic::{
            Camera, Light, ParticleDetails, ParticleEmitter, RibbonEmitter,
        };
        match self {
            Self::ExtentMinimum | Self::ExtentMaximum | Self::ExtentRadius => {
                matches!(feature, NodeFeature::Extent(_))
            }
            Self::Field(path) => {
                let matches = matches!(
                    (path.segments().first(), feature),
                    (
                        Some(FieldSegment::Field(Field::Camera)),
                        NodeFeature::Camera(Camera::Optical(_))
                    ) | (
                        Some(FieldSegment::Field(Field::Light)),
                        NodeFeature::Light(Light::Specular(_))
                    ) | (
                        Some(FieldSegment::Field(Field::Force)),
                        NodeFeature::Force(_)
                    ) | (Some(FieldSegment::Field(Field::Warp)), NodeFeature::Warp(_))
                        | (
                            Some(FieldSegment::Field(Field::Projector)),
                            NodeFeature::Projector(_)
                        )
                        | (
                            Some(FieldSegment::Field(Field::ParticleCopy)),
                            NodeFeature::ParticleCopy(_)
                        )
                        | (
                            Some(FieldSegment::Field(Field::Ribbon)),
                            NodeFeature::RibbonEmitter(RibbonEmitter::Programmed(_))
                        )
                        | (
                            Some(FieldSegment::Field(Field::Shadow)),
                            NodeFeature::ShadowBox(_)
                        )
                        | (
                            Some(FieldSegment::Field(Field::ViewVolume)),
                            NodeFeature::ViewVolume(_)
                        )
                );
                let matches = matches
                    || (matches!(
                        path.segments().first(),
                        Some(FieldSegment::Field(Field::Particle))
                    ) && feature.is_programmed_particle());
                if !matches {
                    return false;
                }
                if let Some(FieldSegment::Field(field)) = path.segments().get(1) {
                    let supported = match feature {
                        NodeFeature::ParticleEmitter(ParticleEmitter::Simulated(value)) => {
                            value.supports_programmed_property(field)
                        }
                        NodeFeature::RibbonEmitter(RibbonEmitter::Programmed(value)) => {
                            value.supports_programmed_property(field)
                        }
                        _ => true,
                    };
                    if !supported {
                        return false;
                    }
                }
                if let [_, FieldSegment::Field(Field::SplinePoints), FieldSegment::Index(index), ..] =
                    path.segments()
                {
                    return match feature {
                        NodeFeature::ParticleEmitter(ParticleEmitter::Simulated(v)) => {
                            match &v.details {
                                ParticleDetails::Programmed(v) => *index < v.spline_points.len(),
                                _ => false,
                            }
                        }
                        NodeFeature::RibbonEmitter(RibbonEmitter::Programmed(v)) => {
                            *index < v.spline_points.len()
                        }
                        _ => false,
                    };
                }
                true
            }
            _ => false,
        }
    }
}
