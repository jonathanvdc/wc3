//! Core model headers and offset-addressed record collections.
use super::codec::{ArrayRef, Decoder, Encoder, Value};
use super::companions::SkinProfile;
use super::records::{ArrayVisitor, Validate, VisitArrays};
use super::records::{
    Attachment, Bone, ColorAnimation, Event, Extent, Light, Material, ModelFlags, Sequence,
    Texture, TextureTransform, TextureWeight, Vertex,
};
use super::records::{Camera, ParticleEmitter, RibbonEmitter};
use super::{ModelLayout, Read, ReadError, RecordLayout, Write, WriteError};
use crate::model::common::{Borrowed, Detached};
use crate::model::common::{CommonPartialEq, Ownership};
use std::borrow::Cow;
use std::fmt::Debug;
use zerovec::ZeroVec;

/// An array whose record layout is unknown, preserving its original address space.
#[derive(Clone, Debug, Default, PartialEq, Ownership)]
pub struct OpaqueArray<'data> {
    /// Native count and offset descriptor; no element stride is inferred.
    pub reference: ArrayRef,
    /// Original address space needed to preserve a nonempty array's offsets.
    pub address_space: Option<Cow<'data, [u8]>>,
}

/// Version 256's sixteen uninterpreted trailing header bytes.
#[derive(Clone, Debug, Default, PartialEq, Ownership)]
pub struct HeaderTail256 {
    /// Native reserved words.
    pub reserved: [u8; 16],
}
/// Version 257's additional opaque array and trailing header words.
#[derive(Clone, Debug, Default, PartialEq, Ownership)]
pub struct HeaderTail257<'data> {
    /// Additional array of unknown record width.
    pub array: OpaqueArray<'data>,
    /// Native reserved words.
    pub reserved: [u8; 8],
}

/// All fields of an MD20 core payload, retaining native record layouts.
#[derive(Clone, Debug, Default, CommonPartialEq, Ownership)]
pub struct Core<'data, V: ModelLayout> {
    /// Model name bytes, including any encoded terminator.
    pub name: ZeroVec<'data, u8>,
    /// Global flags, including unknown bits.
    pub flags: ModelFlags,
    /// Classic header tail present only in versions 256 and 257.
    pub classic_header_tail: V::ClassicTail<'data>,
    /// Global animation loop durations in milliseconds.
    pub global_sequences: ZeroVec<'data, u32>,
    /// Animation sequences in native file order.
    pub sequences: Vec<Sequence<V>>,
    /// Animation ID hash/lookup table.
    pub sequence_lookup: ZeroVec<'data, u16>,
    /// Legacy animation lookup entries, retaining both packed u16 words.
    pub playable_animation_lookup: V::PlayableLookup<'data>,
    /// Skeleton bones.
    pub bones: Vec<Bone<'data, V>>,
    /// Key-bone index lookup table.
    pub key_bone_lookup: ZeroVec<'data, u16>,
    /// Document vertices.
    pub vertices: ZeroVec<'data, Vertex>,
    /// Embedded skin profiles or an external profile count.
    pub skins: V::Skins<'data>,
    /// Animated material color records.
    pub colors: Vec<ColorAnimation<'data, V>>,
    /// Texture filenames and resource types.
    pub textures: Vec<Texture<'data>>,
    /// Animated texture weights.
    pub texture_weights: Vec<TextureWeight<'data, V>>,
    /// Legacy texture flipbook entries.
    pub texture_flipbooks: V::Flipbooks<'data>,
    /// Unnamed classic array at header offset 0xac, with an unverified element layout.
    pub classic_lookup: V::ClassicLookup<'data>,
    /// Animated texture transforms.
    pub texture_transforms: Vec<TextureTransform<'data, V>>,
    /// Texture resource-type lookup table.
    pub texture_lookup: ZeroVec<'data, u16>,
    /// Material rendering flags and blend modes.
    pub materials: ZeroVec<'data, Material>,
    /// Bone combination table.
    pub bone_combos: ZeroVec<'data, u16>,
    /// Texture combination table.
    pub texture_combos: ZeroVec<'data, u16>,
    /// Texture coordinate/bone mapping table.
    pub texture_transform_bone_map: ZeroVec<'data, u16>,
    /// Texture weight combination table.
    pub texture_weight_combos: ZeroVec<'data, u16>,
    /// Texture transform combination table.
    pub texture_transform_combos: ZeroVec<'data, u16>,
    /// Visual model bounds.
    pub bounding: Extent,
    /// Collision bounds.
    pub collision: Extent,
    /// Collision triangle indices.
    pub collision_indices: ZeroVec<'data, u16>,
    /// Collision vertex positions.
    pub collision_vertices: ZeroVec<'data, [f32; 3]>,
    /// Collision face normals.
    pub collision_normals: ZeroVec<'data, [f32; 3]>,
    /// Attachment records.
    pub attachments: Vec<Attachment<'data, V>>,
    /// Attachment ID lookup table.
    pub attachment_lookup: ZeroVec<'data, u16>,
    /// Timestamp-only animation events.
    pub events: Vec<Event<'data, V>>,
    /// Light records.
    pub lights: Vec<Light<'data, V>>,
    /// Camera records.
    pub cameras: Vec<Camera<'data, V>>,
    /// Camera ID lookup table.
    pub camera_lookup: ZeroVec<'data, u16>,
    /// Ribbon emitter records.
    pub ribbons: Vec<RibbonEmitter<'data, V>>,
    /// Particle emitter records.
    pub particles: Vec<ParticleEmitter<'data, V>>,
    /// Flag-selected legacy texture combiner table.
    pub texture_combiner_combos: V::Combiner<'data>,
}
impl<'data, V: ModelLayout> Core<'data, V> {
    /// Creates an empty core using the selected native layout.
    pub fn new() -> Self {
        Self::default()
    }
    /// Returns embedded profiles, empty for externally stored profiles.
    pub fn embedded_skins(&self) -> &[SkinProfile<'data, V>] {
        self.skins.embedded()
    }
    /// Returns the external base profile count, zero for embedded profiles.
    pub fn external_skin_count(&self) -> u32 {
        self.skins.external_count()
    }
    /// Decodes a core address space, including its MD20 magic and version.
    pub fn decode(data: &'data [u8], origin: usize) -> Result<Self, ReadError> {
        let expected_version = V::VERSION;
        let mut input =
            Decoder::new(data, expected_version, ModelFlags::default()).with_origin(origin);
        if input.bytes(4)? != b"MD20" {
            return Err(ReadError::invalid(origin, "MD20 magic"));
        }
        let version = input.u32()?;
        if version != expected_version {
            return Err(ReadError::invalid(origin + 4, "dialect version"));
        }
        let name = input.descriptor()?;
        let flags = input.u32()?;
        input.flags = ModelFlags::from_bits_retain(flags);
        let global_sequences = input.descriptor()?;
        let sequences = input.descriptor()?;
        let sequence_lookup = input.descriptor()?;
        let playable_animation_lookup = V::PlayableLookup::read_header(&mut input)?;
        let bones = input.descriptor()?;
        let key_bone_lookup = input.descriptor()?;
        let vertices = input.descriptor()?;
        let skins = V::Skins::read_header(&mut input)?;
        let colors = input.descriptor()?;
        let textures = input.descriptor()?;
        let weights = input.descriptor()?;
        let texture_flipbooks = V::Flipbooks::read_header(&mut input)?;
        let transforms = input.descriptor()?;
        let texture_lookup = input.descriptor()?;
        let materials = input.descriptor()?;
        let bone_combos = input.descriptor()?;
        let texture_combos = input.descriptor()?;
        let transform_bones = input.descriptor()?;
        let weight_combos = input.descriptor()?;
        let transform_combos = input.descriptor()?;
        let classic_lookup = V::ClassicLookup::read_header(&mut input)?;
        let bounding = Extent::read(&mut input)?;
        let collision = Extent::read(&mut input)?;
        let collision_indices = input.descriptor()?;
        let collision_vertices = input.descriptor()?;
        let collision_normals = input.descriptor()?;
        let attachments = input.descriptor()?;
        let attachment_lookup = input.descriptor()?;
        let events = input.descriptor()?;
        let lights = input.descriptor()?;
        let cameras = input.descriptor()?;
        let camera_lookup = input.descriptor()?;
        let ribbons = input.descriptor()?;
        let particles = input.descriptor()?;
        let classic_header_tail = V::ClassicTail::read_header(&mut input)?;
        let texture_combiner_combos = V::Combiner::read_header(&mut input)?;
        let sequences: Vec<Sequence<V>> = input.records(sequences)?;
        input.external_sequences = sequences
            .iter()
            .map(|sequence| sequence.flags.external_animation())
            .collect();
        Ok(Self {
            name: input.array(name)?,
            flags: ModelFlags::from_bits_retain(flags),
            classic_header_tail,
            classic_lookup,
            global_sequences: input.array(global_sequences)?,
            sequences,
            sequence_lookup: input.array(sequence_lookup)?,
            playable_animation_lookup,
            bones: input.records(bones)?,
            key_bone_lookup: input.array(key_bone_lookup)?,
            vertices: input.array(vertices)?,
            skins,
            colors: input.records(colors)?,
            textures: input.records(textures)?,
            texture_weights: input.records(weights)?,
            texture_flipbooks,
            texture_transforms: input.records(transforms)?,
            texture_lookup: input.array(texture_lookup)?,
            materials: input.array(materials)?,
            bone_combos: input.array(bone_combos)?,
            texture_combos: input.array(texture_combos)?,
            texture_transform_bone_map: input.array(transform_bones)?,
            texture_weight_combos: input.array(weight_combos)?,
            texture_transform_combos: input.array(transform_combos)?,
            bounding,
            collision,
            collision_indices: input.array(collision_indices)?,
            collision_vertices: input.array(collision_vertices)?,
            collision_normals: input.array(collision_normals)?,
            attachments: input.records(attachments)?,
            attachment_lookup: input.array(attachment_lookup)?,
            events: input.records(events)?,
            lights: input.records(lights)?,
            cameras: input.records(cameras)?,
            camera_lookup: input.array(camera_lookup)?,
            ribbons: input.records(ribbons)?,
            particles: input.records(particles)?,
            texture_combiner_combos,
        })
    }
    /// Rebuilds a core payload with relocated arrays and nested record data.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        let version = V::VERSION;
        let mut source = None;
        for bytes in [
            self.classic_lookup.original_bytes()?,
            self.classic_header_tail.original_bytes()?,
        ]
        .into_iter()
        .flatten()
        {
            if source.is_some_and(|other: &[u8]| other != bytes) {
                return Err(WriteError::invalid(
                    "incompatible classic opaque address spaces",
                ));
            }
            source = Some(bytes);
        }
        let mut output = match source {
            Some(bytes) => Encoder::from_bytes(version, self.flags, bytes.to_vec())?,
            None => Encoder::new(version, self.flags),
        };
        let header_size = V::HEADER_SIZE + self.texture_combiner_combos.header_size(self.flags);
        if output.len() < header_size {
            output.reserve(header_size - output.len())?;
        }
        output.bytes(b"MD20")?;
        output.u32(version)?;
        macro_rules! array {
            ($value:expr) => {{
                let reference = output.array($value)?;
                output.write_descriptor(reference)?;
            }};
        }
        macro_rules! records {
            ($value:expr) => {{
                let reference = output.write_records($value)?;
                output.write_descriptor(reference)?;
            }};
        }
        array!(&self.name);
        output.u32(self.flags.bits())?;
        array!(&self.global_sequences);
        records!(&self.sequences);
        array!(&self.sequence_lookup);
        self.playable_animation_lookup.write_header(&mut output)?;
        records!(&self.bones);
        array!(&self.key_bone_lookup);
        array!(&self.vertices);
        self.skins.write_header(&mut output)?;
        records!(&self.colors);
        records!(&self.textures);
        records!(&self.texture_weights);
        self.texture_flipbooks.write_header(&mut output)?;
        records!(&self.texture_transforms);
        array!(&self.texture_lookup);
        array!(&self.materials);
        array!(&self.bone_combos);
        array!(&self.texture_combos);
        array!(&self.texture_transform_bone_map);
        array!(&self.texture_weight_combos);
        array!(&self.texture_transform_combos);
        self.classic_lookup.write_header(&mut output)?;
        self.bounding.write(&mut output)?;
        self.collision.write(&mut output)?;
        array!(&self.collision_indices);
        array!(&self.collision_vertices);
        array!(&self.collision_normals);
        records!(&self.attachments);
        array!(&self.attachment_lookup);
        records!(&self.events);
        records!(&self.lights);
        records!(&self.cameras);
        array!(&self.camera_lookup);
        records!(&self.ribbons);
        records!(&self.particles);
        self.classic_header_tail.write_header(&mut output)?;
        self.texture_combiner_combos.write_header(&mut output)?;
        Ok(output.finish())
    }
}

/// Native header field storage, including its descriptor framing.
pub trait HeaderField<'data>:
    Clone + Debug + Default + PartialEq + Validate + VisitArrays<'data>
{
    /// Reads the native header field and resolves its offset-addressed contents.
    fn read_header(input: &mut Decoder<'data>) -> Result<Self, ReadError>;
    /// Writes native header metadata and relocates understood contents.
    fn write_header(&self, output: &mut Encoder) -> Result<(), WriteError>;
    /// Number of inline header bytes enabled by these flags.
    fn header_size(&self, _flags: ModelFlags) -> usize {
        0
    }
    /// Original address space needed by opaque descriptors.
    fn original_bytes(&self) -> Result<Option<&[u8]>, WriteError> {
        Ok(None)
    }
}
impl<'data> HeaderField<'data> for () {
    fn read_header(_: &mut Decoder<'data>) -> Result<Self, ReadError> {
        Ok(())
    }
    fn write_header(&self, _: &mut Encoder) -> Result<(), WriteError> {
        Ok(())
    }
}
impl<'data> HeaderField<'data> for u32 {
    fn read_header(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        input.u32()
    }
    fn write_header(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.u32(*self)
    }
}
impl<'data, T: Value> HeaderField<'data> for ZeroVec<'data, T> {
    fn read_header(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let r = input.descriptor()?;
        input.array(r)
    }
    fn write_header(&self, output: &mut Encoder) -> Result<(), WriteError> {
        let r = output.array(self)?;
        output.write_descriptor(r)
    }
}
impl<'data, T> HeaderField<'data> for Vec<T>
where
    T: Read<'data>
        + Write
        + RecordLayout
        + Clone
        + Debug
        + Default
        + PartialEq
        + Validate
        + VisitArrays<'data>,
{
    fn read_header(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let r = input.descriptor()?;
        input.records(r)
    }
    fn write_header(&self, output: &mut Encoder) -> Result<(), WriteError> {
        let r = output.write_records(self)?;
        output.write_descriptor(r)
    }
}
impl Validate for OpaqueArray<'_> {
    fn validate(&self) -> Result<(), ReadError> {
        Ok(())
    }
}
impl<'data> VisitArrays<'data> for OpaqueArray<'data> {
    fn visit_arrays<A: ArrayVisitor<'data>>(&mut self, _: &mut A) -> Result<(), A::Error> {
        Ok(())
    }
}
impl<'data> HeaderField<'data> for OpaqueArray<'data> {
    fn read_header(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let reference = input.descriptor()?;
        if reference.count != 0 && reference.offset as usize >= input.data().len() {
            return Err(ReadError::invalid(
                input.absolute_position().saturating_sub(4),
                "opaque array offset",
            ));
        }
        Ok(Self {
            reference,
            address_space: (reference.count != 0).then_some(Cow::Borrowed(input.data())),
        })
    }
    fn write_header(&self, output: &mut Encoder) -> Result<(), WriteError> {
        self.original_bytes()?;
        output.write_descriptor(self.reference)
    }
    fn original_bytes(&self) -> Result<Option<&[u8]>, WriteError> {
        if self.reference.count == 0 {
            return Ok(None);
        }
        let bytes = self
            .address_space
            .as_deref()
            .ok_or_else(|| WriteError::invalid("opaque array requires original address space"))?;
        if self.reference.offset as usize >= bytes.len() {
            return Err(WriteError::invalid("opaque array offset"));
        }
        Ok(Some(bytes))
    }
}
impl Validate for HeaderTail256 {
    fn validate(&self) -> Result<(), ReadError> {
        Ok(())
    }
}
impl<'data> VisitArrays<'data> for HeaderTail256 {
    fn visit_arrays<A: ArrayVisitor<'data>>(&mut self, _: &mut A) -> Result<(), A::Error> {
        Ok(())
    }
}
impl<'data> HeaderField<'data> for HeaderTail256 {
    fn read_header(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        Ok(Self {
            reserved: input.bytes(16)?.try_into().unwrap(),
        })
    }
    fn write_header(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.reserved)
    }
}
impl Validate for HeaderTail257<'_> {
    fn validate(&self) -> Result<(), ReadError> {
        Ok(())
    }
}
impl<'data> VisitArrays<'data> for HeaderTail257<'data> {
    fn visit_arrays<A: ArrayVisitor<'data>>(&mut self, _: &mut A) -> Result<(), A::Error> {
        Ok(())
    }
}
impl<'data> HeaderField<'data> for HeaderTail257<'data> {
    fn read_header(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        Ok(Self {
            array: OpaqueArray::read_header(input)?,
            reserved: input.bytes(8)?.try_into().unwrap(),
        })
    }
    fn write_header(&self, output: &mut Encoder) -> Result<(), WriteError> {
        self.array.write_header(output)?;
        output.bytes(&self.reserved)
    }
    fn original_bytes(&self) -> Result<Option<&[u8]>, WriteError> {
        self.array.original_bytes()
    }
}
/// Texture combiner descriptor whose presence is selected by core flag 0x8.
#[derive(Clone, Debug, Default, PartialEq, Ownership)]
pub struct FlaggedCombiner<'data> {
    /// Native combination entries.
    pub values: ZeroVec<'data, u16>,
}
impl Validate for FlaggedCombiner<'_> {
    fn validate(&self) -> Result<(), ReadError> {
        Ok(())
    }
}
impl<'data> VisitArrays<'data> for FlaggedCombiner<'data> {
    fn visit_arrays<A: ArrayVisitor<'data>>(&mut self, _: &mut A) -> Result<(), A::Error> {
        Ok(())
    }
}
impl<'data> HeaderField<'data> for FlaggedCombiner<'data> {
    fn read_header(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        Ok(Self {
            values: if input.flags.texture_combiners() {
                ZeroVec::read_header(input)?
            } else {
                ZeroVec::new()
            },
        })
    }
    fn write_header(&self, output: &mut Encoder) -> Result<(), WriteError> {
        if output.flags.texture_combiners() {
            self.values.write_header(output)
        } else if self.values.is_empty() {
            Ok(())
        } else {
            Err(WriteError::invalid("texture combiner flag"))
        }
    }
    fn header_size(&self, flags: ModelFlags) -> usize {
        if flags.texture_combiners() {
            8
        } else {
            0
        }
    }
}
/// Header fields whose native presence and storage depend on the selected layout.
pub trait HeaderLayout: Sized {
    /// Inline header width before the flag-selected combiner descriptor.
    const HEADER_SIZE: usize;
    /// Embedded profiles or an external profile count.
    type Skins<'data>: SkinStorage<'data, Self>
        + for<'view> Borrowed<'view, View = Self::Skins<'view>>
        + for<'owned> Detached<'owned, Detached = Self::Skins<'owned>>
    where
        Self: ModelLayout;
    /// Legacy playable-animation lookup or an absent unit field.
    type PlayableLookup<'data>: HeaderField<'data>
        + for<'view> Borrowed<'view, View = Self::PlayableLookup<'view>>
        + for<'owned> Detached<'owned, Detached = Self::PlayableLookup<'owned>>;
    /// BC texture flipbook records or an absent unit field.
    type Flipbooks<'data>: HeaderField<'data>
        + for<'view> Borrowed<'view, View = Self::Flipbooks<'view>>
        + for<'owned> Detached<'owned, Detached = Self::Flipbooks<'owned>>;
    /// Classic opaque lookup or an absent unit field.
    type ClassicLookup<'data>: HeaderField<'data>
        + for<'view> Borrowed<'view, View = Self::ClassicLookup<'view>>
        + for<'owned> Detached<'owned, Detached = Self::ClassicLookup<'owned>>;
    /// Classic reserved tail or an absent unit field.
    type ClassicTail<'data>: HeaderField<'data>
        + for<'view> Borrowed<'view, View = Self::ClassicTail<'view>>
        + for<'owned> Detached<'owned, Detached = Self::ClassicTail<'owned>>;
    /// Flag-selected combiner table or an absent unit field.
    type Combiner<'data>: HeaderField<'data>
        + for<'view> Borrowed<'view, View = Self::Combiner<'view>>
        + for<'owned> Detached<'owned, Detached = Self::Combiner<'owned>>;
}

/// Native embedded-profile storage or external profile count.
pub trait SkinStorage<'data, V: ModelLayout>: HeaderField<'data> {
    /// Returns embedded profiles, empty for external storage.
    fn embedded(&self) -> &[SkinProfile<'data, V>];
    /// Returns the external profile count, zero for embedded storage.
    fn external_count(&self) -> u32;
}
impl<'data, V: ModelLayout> SkinStorage<'data, V> for Vec<SkinProfile<'data, V>> {
    fn embedded(&self) -> &[SkinProfile<'data, V>] {
        self
    }
    fn external_count(&self) -> u32 {
        0
    }
}
impl<'data, V: ModelLayout> SkinStorage<'data, V> for u32 {
    fn embedded(&self) -> &[SkinProfile<'data, V>] {
        &[]
    }
    fn external_count(&self) -> u32 {
        *self
    }
}
