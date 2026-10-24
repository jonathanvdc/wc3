//! Exact projection constraints imposed by a built-in authoring vocabulary.
use super::geometry::{AttributeValueKind, InfluenceKind, TopologyKind};
use crate::model::{AttributeDomain, AttributeSemantic, GeometryKind, Interpolation};
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

/// The built-in vocabulary whose constraints are being checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileKind {
    /// Warcraft III representation.
    Mdlx,
    /// World of Warcraft model representation.
    M2,
    /// World of Warcraft world representation.
    Wmo,
}
/// The independently projected owner payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PayloadKind {
    /// ImageResource representation.
    ImageResource,
    /// ModelResource representation.
    ModelResource,
    /// EffectResource representation.
    EffectResource,
    /// PhysicsResource representation.
    PhysicsResource,
    /// BoneOverrideResource representation.
    BoneOverrideResource,
    /// Appearance representation.
    Appearance,
    /// Binding representation.
    Binding,
    /// Geometry representation.
    Geometry,
    /// ModelMetadata representation.
    ModelMetadata,
    /// EventSource representation.
    EventSource,
    /// System representation.
    System,
    /// Surface representation.
    Surface,
    /// Feature representation.
    Feature,
    /// NodeProperties representation.
    NodeProperties,
    /// MeshProperties representation.
    MeshProperties,
    /// InstanceProperties representation.
    InstanceProperties,
    /// DrawProperties representation.
    DrawProperties,
    /// ClipProperties representation.
    ClipProperties,
}
/// Vocabulary of a rejected payload, without retaining the payload itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PayloadVocabulary {
    /// A single material assignment.
    SingleAppearance,
    /// Draw-based material alternatives.
    DrawAppearance,
    /// Implicit model-space joint binding.
    IdentityBinding,
    /// Explicit authored joint-space transforms.
    ExplicitBinding,
    /// Portable simulation bodies and constraints.
    PortablePhysics,
    /// Warcraft III representation.
    Mdlx,
    /// World of Warcraft model representation.
    M2,
    /// World of Warcraft world representation.
    Wmo,
    /// Universal representation.
    Universal,
    /// None representation.
    None,
}
/// An authored resource-address form, independent of its display spelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceAddressKind {
    /// Root representation.
    Root,
    /// Relative representation.
    Relative,
    /// Path representation.
    Path,
    /// File-data-ID representation.
    FileDataId,
    /// Replaceable representation.
    Replaceable,
    /// EmbeddedImage representation.
    EmbeddedImage,
    /// Named representation.
    Named,
}
/// An attached behavior family, independent of its potentially borrowed definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeatureFamily {
    /// Light representation.
    Light,
    /// Camera representation.
    Camera,
    /// Attachment representation.
    Attachment,
    /// Particle representation.
    Particle,
    /// Ribbon representation.
    Ribbon,
    /// GeometryAnimation representation.
    GeometryAnimation,
    /// CollisionShape representation.
    CollisionShape,
    /// ParticleProgram representation.
    ParticleProgram,
    /// FaceActor representation.
    FaceActor,
    /// LightProgram representation.
    LightProgram,
    /// Particle2 representation.
    Particle2,
    /// PointLight representation.
    PointLight,
}
/// A concrete authoring rule that cannot be projected exactly.
#[derive(Clone, Debug)]
pub enum ProfileViolation {
    /// The payload belongs to a vocabulary the target cannot retain.
    UnsupportedPayload {
        /// Independently projected owner payload category.
        owner: PayloadKind,
        /// Observed value or representation.
        actual: PayloadVocabulary,
    },
    /// The target requires an authored payload that is absent.
    RequiredPayload {
        /// Independently projected owner payload category.
        owner: PayloadKind,
    },
    /// The target cannot retain this resource-address form.
    UnsupportedResourceAddress {
        /// Independently projected owner payload category.
        owner: PayloadKind,
        /// Observed value or representation.
        actual: ResourceAddressKind,
    },
    /// The feature family has no exact representation in the target.
    UnsupportedFeature {
        /// Rejected attached behavior family.
        family: FeatureFamily,
    },
    /// A scalar shader projection requires a fixed number of steps.
    MaterialStepCount {
        /// Required value or correspondence count.
        expected: usize,
        /// Observed value or representation.
        actual: usize,
    },
    /// A step belongs to another shader dialect.
    MaterialDialect {
        /// Observed value or representation.
        actual: ProfileKind,
    },
    /// The target cannot retain an authored material declaration.
    MaterialDeclaration,
    /// The target requires display geometry.
    GeometryKind {
        /// Observed value or representation.
        actual: GeometryKind,
    },
    /// The topology family is unavailable in the target.
    GeometryTopology {
        /// Observed value or representation.
        actual: TopologyKind,
    },
    /// The joint distribution interpretation is unavailable in the target.
    GeometryInfluences {
        /// Observed value or representation.
        actual: InfluenceKind,
    },
    /// The target has no corresponding vertex stream.
    GeometryStream {
        /// Owned meaning of the rejected attribute.
        semantic: AttributeSemantic<'static>,
    },
    /// The target cannot retain the attribute representation.
    GeometryAttribute {
        /// Owned meaning of the rejected attribute.
        semantic: AttributeSemantic<'static>,
        /// Correspondence domain of the rejected attribute.
        domain: AttributeDomain,
        /// Encoded type of the rejected attribute values.
        values: AttributeValueKind,
    },
    /// The number of additional topology sets exceeds the target limit.
    AdditionalConnectivity {
        /// Upper bound or maximum supported count, when bounded.
        maximum: usize,
        /// Observed value or representation.
        actual: usize,
    },
    /// Color slots are not contiguous from zero.
    ColorSlot {
        /// Required value or correspondence count.
        expected: u32,
        /// Observed value or representation.
        actual: u32,
    },
    /// The camera projection has different field-of-view or application-role semantics.
    CameraProjection,
    /// The target requires a constant authored field.
    AnimatedField,
    /// The target has no independent shadow or falloff settings.
    LightParameters,
    /// A shared-clock animation is selected at clip level.
    SharedClockSelection,
    /// Clips do not select the same ordered ordinary animations.
    OrdinaryAnimationSelection,
    /// An event is selected at a level incompatible with its clock.
    EventSelection,
    /// An alias selects different animation identities from its target.
    AliasAnimationSelection,
    /// Nonlooping playback has no established native mapping.
    NonloopingClip,
    /// A bone reference selects a node without a bone declaration.
    UndeclaredBone,
    /// A placed transform would change model-space joint semantics.
    SkinnedPlacementTransform,
    /// A node visibility state cannot be retained.
    NodeVisibility,
    /// The animation value type has no exact native representation.
    TrackValue,
    /// The target cannot retain this interpolation mode.
    Interpolation {
        /// Observed value or representation.
        actual: Interpolation,
    },
    /// The target image declaration has no transfer-function field.
    ColorSpace,
    /// The target supports only repeat and clamp addressing.
    AddressMode,
    /// The native particle program requires the default sampler.
    ParticleSampler,
    /// The image declaration cannot retain sampler or transfer-function settings.
    ImageSampling,
    /// A static world model contains skeleton, timeline or event data.
    StaticAnimation,
    /// A world model does not contain exactly one world system.
    WorldSystemCount {
        /// Required value or correspondence count.
        expected: usize,
        /// Observed value or representation.
        actual: usize,
    },
    /// A world node has hierarchy or view-dependent constraints.
    AbsoluteNodeTransform,
    /// A region selection group has no native field.
    RegionSelectionGroup {
        /// Observed value or representation.
        actual: u32,
    },
    /// A globally active ambient volume is selected by a region.
    GlobalAmbientSelection,
    /// A world region is placed outside authored world coordinates.
    RegionTransform,
    /// A static light contains independently animated properties.
    LightAnimation,
    /// A light node duplicates definition-owned shape or orientation.
    LightTransform,
    /// A world doodad has nonuniform scale.
    DoodadScale,
    /// A region light contains node rotation or scale.
    RegionLightTransform,
    /// The radius differs from the axis-aligned box half diagonal.
    BoundsRadius {
        /// Required value or correspondence count.
        expected: f32,
        /// Observed value or representation.
        actual: f32,
    },
    /// A guild color component is nonfinite.
    GuildPalette,
}
/// A target vocabulary and the exact rule that prevented lossless projection.
#[derive(Clone, Debug)]
pub struct ProfileError {
    /// Built-in vocabulary being checked.
    pub target: ProfileKind,
    /// Concrete projection constraint and retained evidence.
    pub violation: ProfileViolation,
}
impl ProfileError {
    /// Retains a target and its concrete constraint without formatting either.
    pub fn new(target: ProfileKind, violation: ProfileViolation) -> Self {
        Self { target, violation }
    }
}
impl Display for ProfileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: ", self.target)?;
        match &self.violation {
            ProfileViolation::UnsupportedPayload { owner, actual } => write!(f, "{owner:?} payload in {actual:?} vocabulary cannot be retained"),
            ProfileViolation::RequiredPayload { owner } => write!(f, "{owner:?} payload is required"),
            ProfileViolation::UnsupportedResourceAddress { owner, actual } => write!(f, "{owner:?} cannot use {actual:?} addressing"),
            ProfileViolation::UnsupportedFeature { family } => write!(f, "{family:?} feature cannot be retained exactly"),
            ProfileViolation::MaterialStepCount { expected, actual } => write!(f, "expected {expected} material steps, found {actual}"),
            ProfileViolation::MaterialDialect { actual } => write!(f, "cannot retain a {actual:?} material step"),
            ProfileViolation::MaterialDeclaration => f.write_str("cannot retain the material declaration"),
            ProfileViolation::GeometryKind { actual } => write!(f, "requires display geometry, found {actual:?}"),
            ProfileViolation::GeometryTopology { actual } => write!(f, "cannot retain {actual:?} topology"),
            ProfileViolation::GeometryInfluences { actual } => write!(f, "cannot retain {actual:?} joint distributions"),
            ProfileViolation::GeometryStream { semantic } => write!(f, "cannot retain the {semantic:?} stream"),
            ProfileViolation::GeometryAttribute { semantic, domain, values } => write!(f, "cannot retain {semantic:?} in {domain:?} with {values:?} values"),
            ProfileViolation::AdditionalConnectivity { maximum, actual } => write!(f, "at most {maximum} additional connectivity sets are supported, found {actual}"),
            ProfileViolation::ColorSlot { expected, actual } => write!(f, "expected contiguous color slot {expected}, found {actual}"),
            ProfileViolation::CameraProjection => f.write_str("camera projection has incompatible field-of-view or role semantics"),
            ProfileViolation::AnimatedField => f.write_str("field must be constant"),
            ProfileViolation::LightParameters => f.write_str("independent shadow and falloff parameters cannot be retained"),
            ProfileViolation::SharedClockSelection => f.write_str("shared-clock animations must be selected at model level"),
            ProfileViolation::OrdinaryAnimationSelection => f.write_str("clips must select the same ordered ordinary animations"),
            ProfileViolation::EventSelection => f.write_str("ordinary events require clip selection and shared-clock events require model-level selection"),
            ProfileViolation::AliasAnimationSelection => f.write_str("alias must select the target's animation identities"),
            ProfileViolation::NonloopingClip => f.write_str("nonlooping playback has no native mapping"),
            ProfileViolation::UndeclaredBone => f.write_str("reference must select a declared bone"),
            ProfileViolation::SkinnedPlacementTransform => f.write_str("skinned geometry requires model-space joint transforms without a placed mesh transform"),
            ProfileViolation::NodeVisibility => f.write_str("hierarchy nodes cannot retain a hidden or animated visibility state"),
            ProfileViolation::TrackValue => f.write_str("track value type has no exact native animation mapping"),
            ProfileViolation::Interpolation { actual } => write!(f, "{actual:?} interpolation cannot be retained"),
            ProfileViolation::ColorSpace => f.write_str("image declarations cannot retain a color transfer function"),
            ProfileViolation::AddressMode => f.write_str("only repeat and clamp image addressing can be retained"),
            ProfileViolation::ParticleSampler => f.write_str("age-driven particles require default repeat sampling"),
            ProfileViolation::ImageSampling => f.write_str("image declarations cannot retain sampler or transfer-function settings"),
            ProfileViolation::StaticAnimation => f.write_str("static world models cannot retain skeleton, timeline, or event programs"),
            ProfileViolation::WorldSystemCount { expected, actual } => write!(f, "expected {expected} world-system collection, found {actual}"),
            ProfileViolation::AbsoluteNodeTransform => f.write_str("static world actors require absolute transforms without hierarchy or visibility constraints"),
            ProfileViolation::RegionSelectionGroup { actual } => write!(f, "world regions cannot retain selection group {actual}"),
            ProfileViolation::GlobalAmbientSelection => f.write_str("globally active illumination cannot be selected by individual regions"),
            ProfileViolation::RegionTransform => f.write_str("world geometry must be authored directly in world coordinates"),
            ProfileViolation::LightAnimation => f.write_str("static world lights require constant properties or authored flicker programs"),
            ProfileViolation::LightTransform => f.write_str("light shape scale and modern orientation belong to the light definition"),
            ProfileViolation::DoodadScale => f.write_str("world doodads require uniform scale"),
            ProfileViolation::RegionLightTransform => f.write_str("region lights require absolute translation without node rotation or scale"),
            ProfileViolation::BoundsRadius { expected, actual } => write!(f, "box half-diagonal radius is {expected}, found {actual}"),
            ProfileViolation::GuildPalette => f.write_str("guild colours must be finite"),
        }
    }
}
impl StdError for ProfileError {}
