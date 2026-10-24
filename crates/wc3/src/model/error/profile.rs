//! Exact projection constraints imposed by a built-in authoring vocabulary.
//!
//! [`ProfileError`] pairs the target vocabulary with a concrete constraint. Rejected
//! payloads are described by their structural form, feature family or address kind,
//! so errors remain owned without retaining the original model or borrowed feature.
use super::geometry::{AttributeValueKind, InfluenceKind, TopologyKind};
use crate::model::{
    AddressMode, AnimationId, AttributeDomain, AttributeSemantic, ColorSpace, GeometryKind,
    Interpolation, Sampler, ValueSlot,
};
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

/// The built-in vocabulary whose constraints are being checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileKind {
    /// The Warcraft III authoring vocabulary.
    Mdlx,
    /// The World of Warcraft animated-model vocabulary.
    M2,
    /// The World of Warcraft static-world vocabulary.
    Wmo,
}
/// The independently projected owner payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PayloadKind {
    /// An image dependency declaration.
    ImageResource,
    /// A model dependency declaration.
    ModelResource,
    /// An effect dependency declaration.
    EffectResource,
    /// A physics dependency declaration.
    PhysicsResource,
    /// An external bone-override declaration.
    BoneOverrideResource,
    /// The model appearance and material alternatives.
    Appearance,
    /// An authored joint binding interpretation.
    Binding,
    /// Mesh geometry and its authored correspondence.
    Geometry,
    /// Model-wide authoring declarations.
    ModelMetadata,
    /// An event source and its clock.
    EventSource,
    /// An independently owned scene system.
    System,
    /// An ordered material program.
    Surface,
    /// An attached behavior definition.
    Feature,
    /// Additional hierarchy-node behavior.
    NodeProperties,
    /// Additional mesh behavior.
    MeshProperties,
    /// Additional placed-instance behavior.
    InstanceProperties,
    /// Additional draw-pass behavior.
    DrawProperties,
    /// Additional playback-clip behavior.
    ClipProperties,
}
/// The vocabulary or structural form of a rejected payload, without retaining its data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PayloadForm {
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
    /// A Warcraft III declaration.
    Mdlx,
    /// A World of Warcraft animated-model declaration.
    M2,
    /// A World of Warcraft static-world declaration.
    Wmo,
    /// A general declaration not restricted to one built-in profile.
    Universal,
    /// No authored payload.
    None,
}
/// An authored resource-address form, independent of its display spelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceAddressKind {
    /// The resource currently being imported.
    Root,
    /// A file addressed relative to its importing resource.
    Relative,
    /// An explicitly authored native path.
    Path,
    /// A numeric file-data identity.
    FileDataId,
    /// A caller-supplied replaceable texture role.
    Replaceable,
    /// An image stored inside the model resource.
    EmbeddedImage,
    /// An application-defined named resource address.
    Named,
}
/// An attached behavior family, independent of its potentially borrowed definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeatureFamily {
    /// An attached light definition.
    Light,
    /// An attached camera definition.
    Camera,
    /// A socket or child-model attachment.
    Attachment,
    /// A particle-emitter definition.
    Particle,
    /// A ribbon-emitter definition.
    Ribbon,
    /// Geometry color and opacity animation.
    GeometryAnimation,
    /// A model collision shape.
    CollisionShape,
    /// An authored particle program.
    ParticleProgram,
    /// An authored face actor.
    FaceActor,
    /// An authored light or flicker program.
    LightProgram,
    /// A second-generation Warcraft III particle emitter.
    Particle2,
    /// A region-local point-light definition.
    PointLight,
}
/// A timeline or skeleton collection unavailable in a static world model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaticModelComponent {
    /// Clip-level animation selection.
    Clips,
    /// Shared animation clocks.
    Clocks,
    /// Joint palettes and skin bindings.
    Skins,
    /// Ordinary model-level animation tracks.
    OrdinaryTracks,
    /// Shared-clock model-level animation tracks.
    SharedTracks,
    /// Authored event occurrences.
    Events,
}
/// The attribute constraint preventing exact profile projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttributeProjectionRule {
    /// The target has no corresponding attribute semantic.
    Semantic,
    /// The attribute is authored in an unavailable correspondence domain.
    Domain,
    /// The target requires a different encoded value type.
    ValueType,
    /// Another attribute already occupies this authored slot.
    Duplicate,
    /// An explicitly selected attribute may not be empty.
    Nonempty,
}
/// A concrete authoring rule that cannot be projected exactly.
#[derive(Clone, Debug)]
pub enum ProfileViolation {
    /// The payload belongs to a vocabulary the target cannot retain.
    UnsupportedPayload {
        /// Independently projected owner payload category.
        owner: PayloadKind,
        /// Authored payload form that the target cannot retain.
        actual: PayloadForm,
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
        /// Authored resource-address form.
        actual: ResourceAddressKind,
    },
    /// The feature family has no exact representation in the target.
    UnsupportedFeature {
        /// Rejected attached behavior family.
        family: FeatureFamily,
    },
    /// A single-step shader projection requires a fixed number of steps.
    MaterialStepCount {
        /// Number of shader steps required by the target projection.
        expected: usize,
        /// Number of authored shader steps.
        actual: usize,
    },
    /// A step belongs to another shader dialect.
    MaterialDialect {
        /// Vocabulary of the rejected shader step.
        actual: ProfileKind,
    },
    /// The target cannot retain an authored material declaration.
    MaterialDeclaration,
    /// The target requires display geometry.
    GeometryKind {
        /// Authored display or collision purpose.
        actual: GeometryKind,
    },
    /// The topology family is unavailable in the target.
    GeometryTopology {
        /// Rejected connectivity family.
        actual: TopologyKind,
    },
    /// The joint distribution interpretation is unavailable in the target.
    GeometryInfluences {
        /// Authored joint distribution interpretation.
        actual: InfluenceKind,
    },
    /// The target has no corresponding vertex stream.
    GeometryStream {
        /// Owned meaning of the rejected attribute.
        semantic: AttributeSemantic<'static>,
    },
    /// The target cannot retain the attribute representation.
    GeometryAttribute {
        /// Specific correspondence or encoding rule that failed.
        rule: AttributeProjectionRule,
        /// Owned meaning of the rejected attribute.
        semantic: AttributeSemantic<'static>,
        /// Correspondence domain of the rejected attribute.
        domain: AttributeDomain,
        /// Encoded type of the rejected attribute values.
        values: AttributeValueKind,
    },
    /// The number of additional topology sets exceeds the target limit.
    AdditionalConnectivity {
        /// Maximum number of additional topology sets retained by the target.
        maximum: usize,
        /// Number of authored additional topology sets.
        actual: usize,
    },
    /// Color slots are not contiguous from zero.
    ColorSlot {
        /// Next color slot required for a contiguous sequence from zero.
        expected: u32,
        /// Next authored color slot.
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
    OrdinaryAnimationSelection {
        /// Ordered ordinary animation identities selected by the first clip.
        expected: Vec<AnimationId>,
        /// Ordered ordinary animation identities selected by the rejected clip.
        actual: Vec<AnimationId>,
    },
    /// An event is selected at a level incompatible with its clock.
    EventSelection,
    /// An alias selects different animation identities from its target.
    AliasAnimationSelection {
        /// Animation identities selected by the alias target.
        expected: Vec<AnimationId>,
        /// Animation identities selected by the alias.
        actual: Vec<AnimationId>,
    },
    /// Nonlooping playback has no established native mapping.
    NonloopingClip,
    /// A bone reference selects a node without a bone declaration.
    UndeclaredBone,
    /// A placed transform would change model-space joint semantics.
    SkinnedPlacementTransform,
    /// A node visibility state cannot be retained.
    NodeVisibility,
    /// The animation value type has no exact native representation.
    TrackValue {
        /// Typed property slot targeted by the unsupported channel.
        target: ValueSlot,
    },
    /// The target cannot retain this interpolation mode.
    Interpolation {
        /// Authored interpolation mode.
        actual: Interpolation,
    },
    /// The target image declaration has no transfer-function field.
    ColorSpace {
        /// Authored color transfer function.
        actual: ColorSpace,
    },
    /// The target supports only repeat and clamp addressing.
    AddressMode {
        /// Authored horizontal addressing mode.
        u: AddressMode,
        /// Authored vertical addressing mode.
        v: AddressMode,
    },
    /// The native particle program requires the default sampler.
    ParticleSampler {
        /// Authored sampler that differs from the native repeat sampler.
        actual: Sampler,
    },
    /// The image declaration cannot retain sampler or transfer-function settings.
    ImageSampling {
        /// Authored transfer function.
        color_space: ColorSpace,
        /// Authored sampler.
        sampler: Sampler,
    },
    /// A static world model contains skeleton, timeline or event data.
    StaticAnimation {
        /// Unsupported timeline or skeleton collection.
        component: StaticModelComponent,
        /// Number of entries in that collection.
        actual: usize,
    },
    /// A plane-bounded environmental volume has a negative fade distance.
    NegativeVolumeFade {
        /// Authored boundary fade distance.
        actual: f32,
    },
    /// A world model does not contain exactly one world system.
    WorldSystemCount {
        /// Number of world-system collections required by one world model.
        expected: usize,
        /// Number of authored scene systems.
        actual: usize,
    },
    /// A world node has hierarchy or view-dependent constraints.
    AbsoluteNodeTransform,
    /// A region selection group has no native field.
    RegionSelectionGroup {
        /// Authored nonzero geometry selection-group value.
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
        /// Radius derived from the authored box half diagonal.
        expected: f32,
        /// Authored sphere radius.
        actual: f32,
    },
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
            ProfileViolation::UnsupportedPayload { owner, actual } => write!(f, "{owner:?} payload in {actual:?} form cannot be retained"),
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
            ProfileViolation::GeometryAttribute { rule, semantic, domain, values } => write!(f, "cannot retain {semantic:?} in {domain:?} with {values:?} values: {rule:?}"),
            ProfileViolation::AdditionalConnectivity { maximum, actual } => write!(f, "at most {maximum} additional connectivity sets are supported, found {actual}"),
            ProfileViolation::ColorSlot { expected, actual } => write!(f, "expected contiguous color slot {expected}, found {actual}"),
            ProfileViolation::CameraProjection => f.write_str("camera projection has incompatible field-of-view or role semantics"),
            ProfileViolation::AnimatedField => f.write_str("field must be constant"),
            ProfileViolation::LightParameters => f.write_str("independent shadow and falloff parameters cannot be retained"),
            ProfileViolation::SharedClockSelection => f.write_str("shared-clock animations must be selected at model level"),
            ProfileViolation::OrdinaryAnimationSelection { expected, actual } => write!(f, "expected ordinary animation selection {expected:?}, found {actual:?}"),
            ProfileViolation::EventSelection => f.write_str("ordinary events require clip selection and shared-clock events require model-level selection"),
            ProfileViolation::AliasAnimationSelection { expected, actual } => write!(f, "alias must select {expected:?}, found {actual:?}"),
            ProfileViolation::NonloopingClip => f.write_str("nonlooping playback has no native mapping"),
            ProfileViolation::UndeclaredBone => f.write_str("reference must select a declared bone"),
            ProfileViolation::SkinnedPlacementTransform => f.write_str("skinned geometry requires model-space joint transforms without a placed mesh transform"),
            ProfileViolation::NodeVisibility => f.write_str("hierarchy nodes cannot retain a hidden or animated visibility state"),
            ProfileViolation::TrackValue { target } => write!(f, "track target {target:?} has no exact native animation mapping"),
            ProfileViolation::Interpolation { actual } => write!(f, "{actual:?} interpolation cannot be retained"),
            ProfileViolation::ColorSpace { actual } => write!(f, "image declarations cannot retain {actual:?} color space"),
            ProfileViolation::AddressMode { u, v } => write!(f, "only repeat and clamp addressing can be retained, found {u:?}/{v:?}"),
            ProfileViolation::ParticleSampler { actual } => write!(f, "age-driven particles require default repeat sampling, found {actual:?}"),
            ProfileViolation::ImageSampling { color_space, sampler } => write!(f, "image declarations cannot retain {color_space:?} transfer and {sampler:?} sampling"),
            ProfileViolation::StaticAnimation { component, actual } => write!(f, "static world models cannot retain {actual} {component:?} entries"),
            ProfileViolation::NegativeVolumeFade { actual } => write!(f, "volume fade must be nonnegative, found {actual}"),
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
        }
    }
}
impl StdError for ProfileError {}
