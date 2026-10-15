//! Typed MD21 extension payloads. Unknown chunks retain their original bytes.
//!
//! Extension arrays follow their owning core's particle, light, texture, or
//! sequence order where documented. [`ChunkContext`] supplies that owning
//! context when a payload cannot establish its record count independently.
//! Unresolved meanings are identified explicitly rather than assigned units or
//! rendering behavior.
use super as m2;
use super::codec::{ArrayRef, Decoder, Encoder, Value};
use super::companions::{encode_records, flat_records, write_chunk, PhysicsFile};
use super::records::{
    CollisionFlags, DetailLightFlags, Extent, LodFlags, ModelFlags, ParticleTrack, PivotFlags,
    TextureWeight, TrackBase, WaterfallFlags,
};
use super::{ModelLayout, ReadError, WriteError, ZeroVec};
use crate::model::common::Ownership;
use m2::Write as _;
use std::borrow::Cow;
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// AFID mapping from an animation ID and variation to an external file.
pub struct AnimationFileId {
    /// Animation ID matched against a sequence ID, not its array index.
    pub animation_id: u16,
    /// Variation ID matched against the sequence variation index.
    pub variation_id: u16,
    /// File-data ID of the external animation; zero means no file.
    pub file_id: u32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// EXPT particle extensions, aligned with the core particle-emitter array.
pub struct ParticleExtensionV1 {
    /// Native z-source parameter; its coordinate space and application remain unresolved.
    pub z_source: f32,
    /// Multiplier applied to the particle diffuse color.
    pub color_multiplier: f32,
    /// Multiplier applied to particle opacity.
    pub alpha_multiplier: f32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// LDV1 configuration for mesh and particle levels of detail.
pub struct LodConfiguration {
    /// LOD configuration bits; 0x08 enables the fixed-point scale. Unknown bits are preserved.
    pub flags: LodFlags,
    /// Number of available LOD levels; the client clamps this count to eight.
    pub lod_count: u16,
    /// Native distance parameter used in the client scale clamp `clamp(740 / distance, 0.5, 5)`; its physical unit is not established.
    pub lod_distance: f32,
    /// Bone-flag bit shifts controlling particle suppression at the first four LOD levels; the fourth entry also supplies levels four through seven.
    pub particle_bone_lod: [u8; 4],
    /// Unsigned fixed-point scale with a divisor of 2048, enabled by flag 0x08.
    pub lod_scale_raw: u16,
    /// Native LOD batch count; its relation to the skin batch tables remains unresolved.
    pub lod_batch_count: u8,
    /// Uninterpreted trailing byte, preserved without assigning semantics.
    pub reserved: u8,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// EDGF edge-fade parameters for a native submesh or batch reference.
pub struct EdgeFade {
    /// Start factor for edge-distance fading, documented in the normalized range zero to one.
    pub start: f32,
    /// End factor for edge-distance fading, documented in the normalized range zero to one.
    pub end: f32,
    /// Native fade-distance multiplier or falloff parameter; the precise formula and units remain unresolved.
    pub distance: f32,
    /// Native submesh or batch reference; the exact target table remains unresolved.
    pub submesh_index: u32,
    /// Uninterpreted trailing words.
    pub reserved: [u32; 2],
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// NERF squared-distance thresholds controlling model opacity.
pub struct DistanceAlpha {
    /// Squared distance where the model alpha fades to zero, in squared model-distance units.
    pub squared_far_distance: f32,
    /// Squared distance where the model alpha reaches one, in squared model-distance units.
    pub squared_near_distance: f32,
    /// Uninterpreted trailing words.
    pub reserved: [u32; 2],
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// DETL parameters aligned with the core light array.
pub struct DetailLight {
    /// Native detail-light flags; bit meanings remain unresolved.
    pub flags: DetailLightFlags,
    /// Binary16 bits for the shadow render-target matrix scale.
    pub scale_half: u16,
    /// Binary16 bits for the multiplier applied to the corresponding light diffuse color.
    pub diffuse_multiplier_half: u16,
    /// Uninterpreted 16-bit field.
    pub unknown0: u16,
    /// Uninterpreted 32-bit field.
    pub unknown1: u32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// DBOC opacity parameters for a native submesh or batch reference.
pub struct OpacityControl {
    /// Native distance or scale parameter; its exact interpretation and units remain unresolved.
    pub distance_or_scale: f32,
    /// Native opacity-control multiplier; its precise application remains unresolved.
    pub multiplier: f32,
    /// Native submesh or batch reference; the exact target table remains unresolved.
    pub submesh_index: u32,
    /// Uninterpreted trailing word.
    pub reserved: u32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// DPIV pivot-displacement record with partially understood parameters.
pub struct PivotDisplacement {
    /// Three-component pivot displacement; its coordinate space and units remain unresolved.
    pub offset: [f32; 3],
    /// Native displacement flags; bit meanings remain unresolved.
    pub flags: PivotFlags,
    /// Uninterpreted trailing words.
    pub reserved: [u32; 4],
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// TEXL cookie-texture parameters aligned with the core light array.
pub struct TexturedLight {
    /// Scale applied along the light-cookie texture U coordinate.
    pub scale_u: f32,
    /// Scale applied along the light-cookie texture V coordinate.
    pub scale_v: f32,
    /// Index in TXID identifying the light-cookie texture, or -1 when absent.
    pub texture_lookup: i32,
    /// Uninterpreted trailing signed word.
    pub unknown: i32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// WFV3 parameters for the separate waterfall rendering path.
pub struct WaterfallV3 {
    /// Bump scale forwarded to the vertex shader; its exact application remains unresolved.
    pub bump_scale: f32,
    /// First ten native fragment-shader parameters, in wire order; individual meanings remain unresolved.
    pub values0: [f32; 10],
    /// Base color stored as four unsigned RGBA bytes.
    pub base_color: [u8; 4],
    /// Native waterfall-path flags; bit meanings remain unresolved.
    pub flags: WaterfallFlags,
    /// Uninterpreted 16-bit word between flags and the remaining shader parameters.
    pub unknown0: u16,
    /// Remaining seven native shader parameters, in wire order; individual meanings remain unresolved.
    pub values1: [f32; 7],
}

#[derive(Clone, Debug, PartialEq, Ownership, m2::Read, m2::Write, m2::RecordLayout)]
/// EXP2 particle extensions, aligned with the core particle-emitter array.
pub struct ParticleExtension<'data> {
    /// Native z-source parameter; its coordinate space and application remain unresolved.
    pub z_source: f32,
    /// Multiplier applied to the particle diffuse color.
    pub color_multiplier: f32,
    /// Multiplier applied to particle opacity.
    pub alpha_multiplier: f32,
    /// Alpha-test threshold over normalized particle lifetime; timestamps and values use unsigned normalized 16-bit encoding.
    pub alpha_cutoff: ParticleTrack<'data, u16>,
}

#[derive(Clone, Debug, Default, PartialEq, Ownership)]
/// PCOL collision arrays addressed relative to their chunk payload.
pub struct CollisionMesh<'data> {
    /// Collision vertex positions; the coordinate space and units remain unresolved.
    pub positions: ZeroVec<'data, [f32; 3]>,
    /// Collision face normals, stored separately from the vertex positions.
    pub normals: ZeroVec<'data, [f32; 3]>,
    /// Collision index entries referencing the position array.
    pub indices: ZeroVec<'data, u16>,
    /// Native collision flags; element associations and bit meanings remain unresolved.
    pub flags: ZeroVec<'data, CollisionFlags>,
}
#[derive(Clone, Debug, Default)]
/// Owning-model context needed to decode native extension records.
pub struct ChunkContext<V: ModelLayout> {
    /// Core model flags used when decoding flag-selected record fields.
    pub flags: ModelFlags,
    /// Number of core lights, needed to distinguish DETL records from alignment padding.
    pub light_count: Option<usize>,
    /// One entry per owning sequence: true when its animation leaves belong to an external file.
    pub external_sequences: Arc<[bool]>,
    layout: PhantomData<V>,
}
impl<V: ModelLayout> ChunkContext<V> {
    /// Supplies native flags, optional light count, and external-sequence identity.
    pub fn new(
        flags: ModelFlags,
        light_count: Option<usize>,
        external_sequences: Arc<[bool]>,
    ) -> Self {
        Self {
            flags,
            light_count,
            external_sequences,
            layout: PhantomData,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Ownership)]
/// An MD21 extension payload with its native record layout retained.
pub enum Chunk<'data, V: ModelLayout> {
    /// Core payload is decoded by the model container.
    Core(Cow<'data, [u8]>),
    /// PFID file-data ID of an external physics file.
    PhysicsId(u32),
    /// SFID base skin file-data IDs followed by optional LOD skin IDs.
    SkinIds(ZeroVec<'data, u32>),
    /// AFID animation ID, variation, and external file mappings.
    AnimationIds(Vec<AnimationFileId>),
    /// BFID file-data IDs of sparse bone-transform override files.
    BoneIds(ZeroVec<'data, u32>),
    /// TXAC two-byte combiner hints for materials followed by particle emitters; individual byte meanings remain unresolved.
    TextureCombinerHints(ZeroVec<'data, [u8; 2]>),
    /// EXPT per-emitter diffuse-color and opacity multipliers.
    ParticleExtensionsV1(Vec<ParticleExtensionV1>),
    /// EXP2 per-emitter extensions and normalized-lifetime alpha cutoffs.
    ParticleExtensions {
        /// Records in native array order.
        entries: Vec<ParticleExtension<'data>>,
        /// Additional extension bytes with an unresolved interpretation.
        extra_data: ZeroVec<'data, u8>,
    },
    /// PABC animation IDs for which parent sequence lookup is bypassed.
    ParentAnimationBlacklist(ZeroVec<'data, u16>),
    /// PADC replacement texture-weight tracks for parent animation data.
    ParentTextureWeights(Vec<TextureWeight<'data, V>>),
    /// PSBC bounds aligned with the parent sequence array.
    ParentSequenceBounds(Vec<Extent>),
    /// PEDC timestamp tracks supplementing parent event data.
    ParentEventTracks(Vec<TrackBase<'data, V>>),
    /// SKID file-data ID of the shared skeleton.
    SkeletonId(u32),
    /// TXID texture file-data IDs aligned with the core texture array.
    TextureIds(ZeroVec<'data, u32>),
    /// LDV1 mesh and particle LOD configuration.
    Lod(LodConfiguration),
    /// RPID child-emitter model file-data IDs aligned with particle emitters.
    RecursiveParticleIds(ZeroVec<'data, u32>),
    /// GPID particle-geometry model file-data IDs aligned with particle emitters.
    GeometryParticleIds(ZeroVec<'data, u32>),
    /// WFV1 empty marker selecting a waterfall rendering path.
    Waterfall1,
    /// WFV2 empty marker selecting a revised waterfall rendering path.
    Waterfall2,
    /// WFV3 parameters for the waterfall rendering path.
    Waterfall3(WaterfallV3),
    /// PGD1 particle-emitter geoset assignments.
    ParticleGeosets {
        /// Geoset visibility ID per particle emitter; zero means always visible.
        geosets: ZeroVec<'data, u16>,
        /// Uninterpreted header words.
        reserved: [u32; 2],
    },
    /// PFDC inline physics document.
    Physics(PhysicsFile<'data>),
    /// EDGF edge-fade records and their trailing alignment bytes.
    EdgeFades {
        /// Records in native array order.
        entries: Vec<EdgeFade>,
        /// Original trailing alignment bytes, preserved verbatim.
        padding: Cow<'data, [u8]>,
    },
    /// NERF squared-distance model-alpha thresholds.
    DistanceAlpha(Vec<DistanceAlpha>),
    /// DETL parameters aligned with core lights and trailing alignment bytes.
    DetailLights {
        /// Records in native array order.
        entries: Vec<DetailLight>,
        /// Original trailing alignment bytes, preserved verbatim.
        padding: Cow<'data, [u8]>,
    },
    /// DBOC opacity controls associated with submesh or batch references.
    OpacityControls(Vec<OpacityControl>),
    /// AFRA's payload layout remains undocumented; retain it without interpretation.
    AnimationFrameRate(Cow<'data, [u8]>),
    /// PCOL collision positions, face normals, indices, and flags.
    Collision(CollisionMesh<'data>),
    /// DPIV pivot-displacement records in native order.
    PivotDisplacements(Vec<PivotDisplacement>),
    /// TEXL light-cookie parameters aligned with core lights.
    TexturedLights(Vec<TexturedLight>),
    /// Unrecognized chunk tag and its uninterpreted payload bytes.
    Unknown {
        /// Four-byte native chunk identifier.
        tag: [u8; 4],
        /// Original payload bytes, preserved without interpretation.
        data: Cow<'data, [u8]>,
    },
}
impl<'data, V: ModelLayout> Chunk<'data, V> {
    /// Decodes one tagged payload using its owning-model context.
    pub fn read(
        tag: [u8; 4],
        data: &'data [u8],
        context: &ChunkContext<V>,
    ) -> Result<Self, ReadError> {
        let mut d = Decoder::new(data, V::VERSION, context.flags);
        d.external_sequences = context.external_sequences.clone();
        Ok(match &tag {
            b"MD21" => Self::Core(Cow::Borrowed(data)),
            b"PFID" => Self::PhysicsId(read_id(&mut d, data.len())?),
            b"SKID" => Self::SkeletonId(read_id(&mut d, data.len())?),
            b"SFID" => Self::SkinIds(leaf(&d, data, 4)?),
            b"BFID" => Self::BoneIds(leaf(&d, data, 4)?),
            b"TXID" => Self::TextureIds(leaf(&d, data, 4)?),
            b"RPID" => Self::RecursiveParticleIds(leaf(&d, data, 4)?),
            b"GPID" => Self::GeometryParticleIds(leaf(&d, data, 4)?),
            b"TXAC" => Self::TextureCombinerHints(leaf(&d, data, 2)?),
            b"PABC" => {
                let a = d.descriptor()?;
                Self::ParentAnimationBlacklist(d.array(a)?)
            }
            b"AFID" => Self::AnimationIds(flat_records(data, V::VERSION, context.flags)?),
            b"EXPT" => Self::ParticleExtensionsV1(flat_records(data, V::VERSION, context.flags)?),
            b"EXP2" => {
                let a = d.descriptor()?;
                let extra = d.descriptor()?;
                Self::ParticleExtensions {
                    entries: d.records(a)?,
                    extra_data: d.array(extra)?,
                }
            }
            b"PADC" => {
                let a = d.descriptor()?;
                Self::ParentTextureWeights(d.records(a)?)
            }
            b"PSBC" => {
                let a = d.descriptor()?;
                Self::ParentSequenceBounds(d.records(a)?)
            }
            b"PEDC" => {
                let a = d.descriptor()?;
                Self::ParentEventTracks(d.records(a)?)
            }
            b"LDV1" => Self::Lod(read_single(data, context)?),
            b"NERF" => Self::DistanceAlpha(flat_records(data, V::VERSION, context.flags)?),
            b"WFV1" => {
                if !data.is_empty() {
                    return Err(ReadError::invalid(0, "WFV1 marker payload"));
                }
                Self::Waterfall1
            }
            b"WFV2" => {
                if !data.is_empty() {
                    return Err(ReadError::invalid(0, "WFV2 marker payload"));
                }
                Self::Waterfall2
            }
            b"WFV3" => Self::Waterfall3(read_single(data, context)?),
            b"PGD1" => {
                let a = d.descriptor()?;
                let reserved = [d.u32()?, d.u32()?];
                Self::ParticleGeosets {
                    geosets: d.array(a)?,
                    reserved,
                }
            }
            b"PFDC" => Self::Physics(PhysicsFile::read_inline(data)?),
            b"EDGF" => {
                let count = data.len() / 24;
                let length = count * 24;
                Self::EdgeFades {
                    entries: flat_records(&data[..length], V::VERSION, context.flags)?,
                    padding: Cow::Borrowed(&data[length..]),
                }
            }
            b"DETL" => {
                let count = context
                    .light_count
                    .ok_or_else(|| ReadError::invalid(0, "DETL requires core light count"))?;
                let length = count
                    .checked_mul(12)
                    .ok_or_else(|| ReadError::invalid(0, "DETL size overflow"))?;
                if length > data.len() {
                    return Err(ReadError::invalid(0, "DETL too short"));
                }
                Self::DetailLights {
                    entries: flat_records(&data[..length], V::VERSION, context.flags)?,
                    padding: Cow::Borrowed(&data[length..]),
                }
            }
            b"DBOC" => Self::OpacityControls(flat_records(data, V::VERSION, context.flags)?),
            b"AFRA" => Self::AnimationFrameRate(Cow::Borrowed(data)),
            b"PCOL" => {
                let a = d.descriptor()?;
                let b = d.descriptor()?;
                let c = d.descriptor()?;
                let f = d.descriptor()?;
                Self::Collision(CollisionMesh {
                    positions: d.array(a)?,
                    normals: d.array(b)?,
                    indices: d.array(c)?,
                    flags: d.array(f)?,
                })
            }
            b"DPIV" => Self::PivotDisplacements(flat_records(data, V::VERSION, context.flags)?),
            b"TEXL" => Self::TexturedLights(flat_records(data, V::VERSION, context.flags)?),
            _ => Self::Unknown {
                tag,
                data: Cow::Borrowed(data),
            },
        })
    }
    /// Returns the on-disk chunk tag.
    pub fn tag(&self) -> [u8; 4] {
        match self {
            Self::Core(..) => *b"MD21",
            Self::PhysicsId(..) => *b"PFID",
            Self::SkinIds(..) => *b"SFID",
            Self::AnimationIds(..) => *b"AFID",
            Self::BoneIds(..) => *b"BFID",
            Self::TextureCombinerHints(..) => *b"TXAC",
            Self::ParticleExtensionsV1(..) => *b"EXPT",
            Self::ParticleExtensions { .. } => *b"EXP2",
            Self::ParentAnimationBlacklist(..) => *b"PABC",
            Self::ParentTextureWeights(..) => *b"PADC",
            Self::ParentSequenceBounds(..) => *b"PSBC",
            Self::ParentEventTracks(..) => *b"PEDC",
            Self::SkeletonId(..) => *b"SKID",
            Self::TextureIds(..) => *b"TXID",
            Self::Lod(..) => *b"LDV1",
            Self::RecursiveParticleIds(..) => *b"RPID",
            Self::GeometryParticleIds(..) => *b"GPID",
            Self::Waterfall1 => *b"WFV1",
            Self::Waterfall2 => *b"WFV2",
            Self::Waterfall3(..) => *b"WFV3",
            Self::ParticleGeosets { .. } => *b"PGD1",
            Self::Physics(..) => *b"PFDC",
            Self::EdgeFades { .. } => *b"EDGF",
            Self::DistanceAlpha(..) => *b"NERF",
            Self::DetailLights { .. } => *b"DETL",
            Self::OpacityControls(..) => *b"DBOC",
            Self::AnimationFrameRate(..) => *b"AFRA",
            Self::Collision(..) => *b"PCOL",
            Self::PivotDisplacements(..) => *b"DPIV",
            Self::TexturedLights(..) => *b"TEXL",
            Self::Unknown { tag, .. } => *tag,
        }
    }
    /// Encodes the typed chunk payload with freshly assigned local offsets.
    pub fn payload(&self, flags: ModelFlags) -> Result<Vec<u8>, WriteError> {
        let version = V::VERSION;
        let mut e = Encoder::new(version, flags);
        match self {
            Self::Core(data) | Self::AnimationFrameRate(data) | Self::Unknown { data, .. } => {
                e.bytes(data)?
            }
            Self::PhysicsId(id) | Self::SkeletonId(id) => e.u32(*id)?,
            Self::SkinIds(a)
            | Self::BoneIds(a)
            | Self::TextureIds(a)
            | Self::RecursiveParticleIds(a)
            | Self::GeometryParticleIds(a) => e.bytes(a.as_bytes())?,
            Self::ParentAnimationBlacklist(a) => {
                e.reserve(8)?;
                let a = e.array(a)?;
                e.write_descriptor(a)?;
            }
            Self::TextureCombinerHints(a) => e.bytes(a.as_bytes())?,
            Self::AnimationIds(values) => e.bytes(&encode_records(values, version, flags)?)?,
            Self::ParticleExtensionsV1(values) => {
                e.bytes(&encode_records(values, version, flags)?)?
            }
            Self::ParentSequenceBounds(values) => {
                e.reserve(8)?;
                let a = e.write_records(values)?;
                e.write_descriptor(a)?;
            }
            Self::OpacityControls(values) => e.bytes(&encode_records(values, version, flags)?)?,
            Self::PivotDisplacements(values) => {
                e.bytes(&encode_records(values, version, flags)?)?
            }
            Self::TexturedLights(values) => e.bytes(&encode_records(values, version, flags)?)?,
            Self::ParticleExtensions {
                entries,
                extra_data,
            } => {
                e.reserve(16)?;
                let a = e.write_records(entries)?;
                let extra = e.array(extra_data)?;
                e.write_descriptor(a)?;
                e.write_descriptor(extra)?;
            }
            Self::ParentTextureWeights(values) => {
                e.reserve(8)?;
                let a = e.write_records(values)?;
                e.write_descriptor(a)?;
            }
            Self::ParentEventTracks(values) => {
                e.reserve(8)?;
                let a = e.write_records(values)?;
                e.write_descriptor(a)?;
            }
            Self::Lod(v) => v.write(&mut e)?,
            Self::DistanceAlpha(v) => e.bytes(&encode_records(v, version, flags)?)?,
            Self::Waterfall3(v) => v.write(&mut e)?,
            Self::Waterfall1 | Self::Waterfall2 => {}
            Self::ParticleGeosets { geosets, reserved } => {
                e.reserve(16)?;
                let a = e.array(geosets)?;
                e.write_descriptor(a)?;
                for v in reserved {
                    e.u32(*v)?;
                }
                e.align(16)?;
            }
            Self::Physics(physics) => e.bytes(&physics.write_inline()?)?,
            Self::EdgeFades { entries, padding } => {
                e.bytes(&encode_records(entries, version, flags)?)?;
                e.bytes(padding)?;
            }
            Self::DetailLights { entries, padding } => {
                e.bytes(&encode_records(entries, version, flags)?)?;
                e.bytes(padding)?;
            }
            Self::Collision(v) => {
                e.reserve(32)?;
                let a = e.array(&v.positions)?;
                let b = e.array(&v.normals)?;
                let c = e.array(&v.indices)?;
                let f = e.array(&v.flags)?;
                for a in [a, b, c, f] {
                    e.write_descriptor(a)?;
                }
            }
        }
        Ok(e.finish())
    }
    /// Encodes the document with freshly assigned local offsets.
    pub fn write(&self, e: &mut Encoder) -> Result<(), WriteError> {
        write_chunk(e, self.tag(), &self.payload(e.flags)?)
    }
}
fn read_id(d: &mut Decoder<'_>, len: usize) -> Result<u32, ReadError> {
    if len != 4 {
        return Err(ReadError::invalid(0, "file ID chunk length"));
    }
    d.u32()
}
fn leaf<'data, T: Value>(
    d: &Decoder<'data>,
    data: &'data [u8],
    stride: usize,
) -> Result<ZeroVec<'data, T>, ReadError> {
    if !data.len().is_multiple_of(stride) {
        return Err(ReadError::invalid(0, "chunk array length"));
    }
    let count = u32::try_from(data.len() / stride)
        .map_err(|_| ReadError::invalid(0, "chunk array count"))?;
    d.array(ArrayRef { count, offset: 0 })
}
fn read_single<'data, T: m2::Read<'data> + m2::RecordLayout, V: ModelLayout>(
    data: &'data [u8],
    c: &ChunkContext<V>,
) -> Result<T, ReadError> {
    if data.len() != T::size(V::VERSION, c.flags.bits()) {
        return Err(ReadError::invalid(0, "chunk record size"));
    }
    T::read(&mut Decoder::new(data, V::VERSION, c.flags))
}

impl<V: ModelLayout> m2::Write for Chunk<'_, V> {
    fn write(&self, encoder: &mut Encoder) -> Result<(), WriteError> {
        Self::write(self, encoder)
    }
}
