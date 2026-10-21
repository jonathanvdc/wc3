//! Immutable effect programmes and renderer-independent storage bindings.
use super::payload::Layout as PayloadLayout;
use super::shapes::Shape;
use super::turbulence::Turbulence;
use super::{Program, Value, ValueType};
use glam::{Mat4, Vec3};
use std::sync::Arc;

/// Zero-based layer identity within one compiled effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EffectLayerId(#[doc = "Layer position in the owning effect."] pub u32);
/// Canonical field identity within one compiled layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EffectFieldId(#[doc = "Field position in the owning layer."] pub u32);
/// Authored effect attribute identity, resolved once at compilation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EffectAttributeId(#[doc = "Attribute position in the owning effect."] pub u32);
/// Stable live-particle identity, including the pool-slot generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EffectParticleId {
    /// Reusable pool slot.
    pub slot: u32,
    /// Slot incarnation; stale parents cannot alias a recycled particle.
    pub generation: u32,
}
/// A checked, immutable host scene snapshot for one simulation observation.
///
/// Positions use the same world coordinate system and numeric units as
/// `SceneInputs::world_from_effect`. This value contains no native pointer or VM
/// register identity. The checked executor supplies its own opaque capability.
#[derive(Clone, Copy, Debug, Default)]
pub struct SceneContext {
    camera: Option<Vec3>,
}
impl SceneContext {
    /// Bind camera index zero. Nonfinite positions are unavailable.
    pub fn with_camera(position: Vec3) -> Option<Self> {
        position.is_finite().then_some(Self {
            camera: Some(position),
        })
    }
    /// Return the supported simulation camera, or no position for absent and
    /// unsupported indices. Rendering views never change this selection.
    pub fn camera_position(self, index: i32) -> Option<Vec3> {
        (index == 0).then_some(self.camera).flatten()
    }
}
/// Effect-local external inputs; simulation time belongs to the instance.
#[derive(Clone, Copy, Debug)]
pub struct SceneInputs {
    /// Transforms native effect coordinates into application world coordinates once.
    pub world_from_effect: Mat4,
    /// Explicit host scene capabilities; absent cameras remain unavailable.
    pub scene: SceneContext,
}
impl Default for SceneInputs {
    fn default() -> Self {
        Self {
            world_from_effect: Mat4::IDENTITY,
            scene: SceneContext::default(),
        }
    }
}
/// Canonical particle field, separate from temporary VM register storage.
#[derive(Clone, Debug)]
pub struct EffectField {
    /// Authored field name.
    pub name: Box<[u8]>,
    /// Native type identifier, including metadata with no evaluator meaning.
    pub native_type: u32,
    /// Supported evaluator layout, absent for uninterpreted metadata.
    pub value_type: Option<ValueType>,
    pub(crate) initial: [u32; 4],
}
/// Authored external effect control with a checked value layout.
#[derive(Clone, Debug)]
pub struct EffectAttribute {
    /// Public authored control name.
    pub name: Box<[u8]>,
    /// Default typed value.
    pub default: Value,
}
/// One renderer stream resolved to canonical field storage.
#[derive(Clone, Debug)]
pub struct EffectRendererInput {
    /// Native semantic selector; custom names retain their separate meaning.
    pub semantic: u32,
    /// Canonical field consumed by this stream.
    pub field: EffectFieldId,
    /// Optional custom semantic name.
    pub additional_name: Option<Box<[u8]>>,
    /// Optional native custom type selector.
    pub additional_type: Option<i32>,
}
/// Immutable renderer property retaining exact native component bits.
#[derive(Clone, Debug)]
pub struct EffectRendererProperty {
    /// Authored property name.
    pub name: Box<[u8]>,
    /// Native property layout selector.
    pub property_type: i32,
    /// Bit-preserving numeric components.
    pub numeric: [u32; 4],
    /// Optional authored resource path or text.
    pub text: Option<Box<[u8]>>,
}
/// Renderer description independent of graphics APIs and native object lifetimes.
#[derive(Clone, Debug)]
pub struct EffectRenderer {
    /// Owning compiled layer.
    pub layer: EffectLayerId,
    /// Native renderer family; zero denotes the observed billboard family.
    pub class: i32,
    /// Authored material feature declaration path.
    pub feature_set: Box<[u8]>,
    /// Signed authored priority.
    pub draw_order: i32,
    /// Resolved stream bindings.
    pub inputs: Box<[EffectRendererInput]>,
    /// Authored properties, preserving absent versus present false controls.
    pub properties: Box<[EffectRendererProperty]>,
}
impl EffectRenderer {
    /// Find an authored property during renderer preparation, outside particle loops.
    pub fn property(&self, name: &[u8]) -> Option<&EffectRendererProperty> {
        self.properties
            .iter()
            .find(|property| property.name.as_ref() == name)
    }
    /// Primary diffuse resource when rendering and diffuse sampling are enabled.
    /// Disabled atlas motion-vector fallback resources do not gate this image.
    pub fn primary_image(&self) -> Option<&[u8]> {
        let enabled = |name: &[u8], default| {
            self.property(name)
                .map_or(default, |value| value.numeric[0] != 0)
        };
        if !enabled(b"EnableRendering", true) || !enabled(b"Diffuse", false) {
            return None;
        }
        self.property(b"Diffuse.DiffuseMap")?.text.as_deref()
    }
}
/// A named native graph entry and its selected event route.
#[derive(Clone, Debug)]
pub struct EffectEntry {
    /// Authored name; empty entries remain distinct from a named Root entry.
    pub name: Box<[u8]>,
    pub(crate) event: usize,
}
/// Established implementation policies whose native fidelity remains unverified.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectLimitation {
    /// An unrecognized extra native programme exactly duplicates a supported
    /// alternative; only the explicit known-stage selection is executed.
    RedundantProgramme,
    /// Spatial reads use the previous executed tick and deterministic insertion-order ties.
    SpatialSnapshots,
    /// Original counter-based random stream rather than a verified native RNG.
    RandomSequence,
    /// Births use a deterministic remaining-subframe interval; native fidelity is unverified.
    BirthScheduling,
    /// Explicit stream duration interpretation.
    StreamDuration,
    /// Native coordinate metadata is retained without an inferred second conversion.
    CoordinateMetadata,
    /// Supported T/Q transform filters use the caller's frame without independently verified scale filtering.
    TransformMask,
    /// Supported inverse masks use a checked full affine inverse; native partial scale filtering is unverified.
    InverseTransform,
    /// Constant-capable canonical fields initialize from native constants before the authored initializer.
    FieldInitialization,
    /// Authored death-step controls select whether evolution runs on the terminal interval.
    DeathEvolution,
    /// Authored positive-infinite inverse lifetimes retire on the first positive interval, preserving zero-delta warmup.
    InstantLifetime,
    /// Sparse primitive parameters use explicit defaults whose client fidelity remains unverified.
    ShapeDefaults,
    /// Shape authoring coordinates convert once into the simulation frame by an explicit convention.
    ShapeCoordinates,
    /// Primitive distributions and fixed random consumption are original policies without client trace verification.
    ShapeSampling,
    /// Pure orientation kernels retain explicit axis, normalization and degenerate-vector policies.
    OrientationPolicies,
    /// Varying endpoint payloads use linear values and shortest-arc normalized quaternion interpolation.
    PayloadInterpolation,
    /// Emitted-count payloads use zero-based cumulative occurrence indices.
    PayloadIndex,
    /// Nonzero native draft controls retain independent ownership without a verified additional routing meaning.
    PayloadDraftControl,
    /// Declared root channels omitted by a birth read as typed zero; native defaults are unverified.
    MissingPayloadZero,
    /// Present position/orientation channels replace the complete payload frame, with unit scale.
    PayloadFrame,
    /// Native interpolation selector is restricted to the observed zero mode.
    CurveInterpolation,
    /// Original analytic scalar-potential curl field; native stochastic samples are not reproduced.
    TurbulenceField,
    /// At most 24 octaves with explicit finite spatial/temporal precision domains.
    TurbulenceDomain,
}
/// Explicit interpretation of EventStream.duration, whose native loop contract is unresolved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventStreamDuration {
    /// Last authored occurrence time, or zero for an empty stream.
    LastOccurrence,
    /// Application-selected finite duration in seconds.
    Explicit(f32),
}
/// Original deterministic random policy used by the checked host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectRandom {
    /// Counter-based mixed random words with deterministic child stream splitting.
    Counter,
    /// Interval midpoints for independent numerical-oracle tests.
    Midpoint,
}
/// Extraction behavior for a declared root channel omitted by a particular birth.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MissingPayloadPolicy {
    /// Reject an absent value; never invent payload values.
    #[default]
    Error,
    /// Return zero words with the declared type, without marking the channel present.
    /// Native default-value semantics remain unverified.
    Zero,
}
/// Shared immutable effect graph; instances own no borrowed native data.
#[derive(Clone, Debug)]
pub struct EffectProgram {
    pub(crate) inner: Arc<EffectGraph>,
}
impl EffectProgram {
    /// Whether two handles refer to the same immutable compiled programme.
    /// Clones share identity; independently compiled equal graphs do not.
    pub fn same_program(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner.identity, &other.inner.identity)
    }

    /// Checked extraction policy retained from graph compilation.
    pub fn missing_payload_policy(&self) -> MissingPayloadPolicy {
        self.inner.missing_payload
    }
    /// Authored default entry index, independent of entry names.
    pub fn default_entry(&self) -> u32 {
        self.inner.default_entry
    }
    /// Named and anonymous graph entries.
    pub fn entries(&self) -> &[EffectEntry] {
        &self.inner.entries
    }
    /// Compiled layers in native order.
    pub fn layers(&self) -> &[EffectLayer] {
        &self.inner.layers
    }
    /// Immutable renderer descriptions in layer/native order.
    pub fn renderers(&self) -> &[EffectRenderer] {
        &self.inner.renderers
    }
    /// Exposed typed controls and their defaults.
    pub fn attributes(&self) -> &[EffectAttribute] {
        &self.inner.attributes
    }
    /// Explicit implementation policies with unverified native fidelity.
    pub fn limitations(&self) -> &[EffectLimitation] {
        &self.inner.limitations
    }
}
/// Canonical layer storage and immutable selected programmes.
#[derive(Clone, Debug)]
pub struct EffectLayer {
    /// Canonical particle fields in native storage-index order.
    pub fields: Box<[EffectField]>,
    pub(crate) init: BoundProgram,
    pub(crate) evolve: Option<BoundProgram>,
    pub(crate) life: usize,
    pub(crate) inverse_life: usize,
    pub(crate) standalone: bool,
    pub(crate) spawner: bool,
    pub(crate) rendered: bool,
    pub(crate) evolve_on_death: bool,
    pub(crate) payload: PayloadLayout,
}
#[derive(Clone, Debug)]
pub(crate) struct SpatialResource {
    pub name: Box<[u8]>,
    pub payload_name: Box<[u8]>,
    pub cell_size: f32,
}
#[derive(Debug)]
pub(crate) struct EffectGraph {
    pub identity: Arc<()>,
    pub default_entry: u32,
    pub entries: Box<[EffectEntry]>,
    pub layers: Box<[EffectLayer]>,
    pub events: Box<[EventRoute]>,
    pub renderers: Box<[EffectRenderer]>,
    pub attributes: Box<[EffectAttribute]>,
    pub samplers: Box<[EffectSampler]>,
    pub spatial: Box<[SpatialResource]>,
    pub limitations: Box<[EffectLimitation]>,
    pub duration: EventStreamDuration,
    pub random: EffectRandom,
    pub missing_payload: MissingPayloadPolicy,
}
#[derive(Clone, Debug)]
pub(crate) struct BoundProgram {
    pub program: Program,
    pub externals: Box<[External]>,
    pub callbacks: Box<[Callback]>,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum External {
    Field(usize),
    Time,
    Delta,
    ParticleContext,
    EffectContext,
    RandomContext,
    SceneContext,
    Sampler(usize),
    Spatial(usize),
    Event(usize),
    Attribute(usize),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Callback {
    SpatialAllocate,
    SpatialAppend,
    SpatialInsert,
    SpatialClosest,
    ViewPosition,
    Age,
    Running,
    StreamDuration,
    Generate,
    Trigger,
    BuildElement,
    Append,
    Extract(ValueType),
    InitPayload,
    Kick,
    Kill,
    Rand,
    VRand,
    Transform,
    TransformDirection,
    InverseTransform,
    InverseDirection,
    Sample,
    HsvToRgb,
    RgbToHsv,
    SamplePosition,
    TurbulenceSample,
    Orientation,
    Rotate,
    OrientationSide,
    RotateAbout,
    EffectSide,
    EffectUp,
}
#[derive(Debug)]
pub(crate) struct EventRoute {
    pub targets: Box<[EffectLayerId]>,
    pub payload: PayloadLayout,
    pub maps: Box<[Box<[usize]>]>,
}
#[derive(Debug)]
pub(crate) enum EffectSampler {
    Curve {
        kind: ValueType,
        times: Box<[f32]>,
        values: Box<[[f32; 4]]>,
        tangents: Box<[[f32; 4]]>,
    },
    Events(Box<[f32]>),
    Shape(Shape),
    Turbulence(Turbulence),
}
