//! Chunked shared skeleton records; parent references stay explicit.
use super::super as m2;
use super::super::chunks::AnimationFileId;
use super::super::codec::{ArrayRef, Decoder, Encoder};
use super::super::records::{Attachment, Bone, ModelFlags, Sequence, SkeletonFlags};
use super::super::{ModelLayout, ReadError, WriteError, ZeroVec};
use super::{encode_records, flat_records, write_chunk};
use crate::model::common::binary::Chunks;
use crate::model::common::Ownership;
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Ownership)]
/// Shared skeleton arrays and dependency references identified by native chunk tags.
pub enum SkeletonChunk<'data, L: ModelLayout> {
    /// SKL1 skeleton name and header metadata.
    Header {
        /// SKL1 header flags; individual bit meanings are not established.
        flags: SkeletonFlags,
        /// Skeleton name bytes, including any stored string terminator.
        name: ZeroVec<'data, u8>,
        /// Reserved bytes with no established semantic meaning; preserved verbatim.
        reserved: [u8; 4],
    },
    /// SKA1 attachment records and attachment-ID lookup table.
    Attachments {
        /// Attachment positions and visibility animation relative to skeleton bones.
        attachments: Vec<Attachment<'data, L>>,
        /// Maps attachment IDs to indices in `attachments`.
        lookup: ZeroVec<'data, u16>,
    },
    /// SKB1 bone hierarchy and key-bone lookup table.
    Bones {
        /// Bone hierarchy, pivots, and transform tracks using the selected owner layout.
        bones: Vec<Bone<'data, L>>,
        /// Maps semantic key-bone IDs to indices in `bones`; 0xffff marks a missing bone.
        lookup: ZeroVec<'data, u16>,
    },
    /// SKS1 sequence metadata and animation lookup tables.
    Sequences {
        /// Repeating global-sequence durations in milliseconds, indexed by track global-sequence IDs.
        global_loops: ZeroVec<'data, u32>,
        /// Animation sequence metadata, including flags that select inline or external track storage.
        sequences: Vec<Sequence<L>>,
        /// Animation-ID hash buckets containing indices in `sequences`; 0xffff marks an empty bucket.
        lookup: ZeroVec<'data, u16>,
        /// Reserved bytes with no established semantic meaning; preserved verbatim.
        reserved: [u8; 8],
    },
    /// SKPD reference to a parent skeleton supplying inherited records.
    Parent {
        /// Reserved prefix bytes with no established semantic meaning; preserved verbatim.
        reserved0: [u8; 8],
        /// FileDataID of the parent .skel document; zero means no parent.
        file_id: u32,
        /// Reserved trailing bytes with no established semantic meaning; preserved verbatim.
        reserved1: [u8; 4],
    },
    /// AFID mappings from animation/variation IDs to external .anim FileDataIDs.
    AnimationIds(Vec<AnimationFileId>),
    /// BFID FileDataIDs of supporting .bone customization files.
    BoneIds(ZeroVec<'data, u32>),
    /// Unrecognized chunk preserved without interpreting its payload.
    Unknown {
        /// Four-byte native chunk identifier.
        tag: [u8; 4],
        /// Uninterpreted payload bytes, excluding chunk framing.
        data: Cow<'data, [u8]>,
    },
}
#[derive(Clone, Debug, PartialEq, Ownership)]
/// Shared skeleton document interpreted using its owning model's native layout.
pub struct Skeleton<'data, L: ModelLayout> {
    /// Owning model core flags used to select conditional record fields; this context is not a standalone skeleton header.
    pub flags: ModelFlags,
    /// Skeleton chunks in source order, including parent and supporting-file references.
    pub chunks: Vec<SkeletonChunk<'data, L>>,
}
impl<'data, L: ModelLayout> Skeleton<'data, L> {
    /// Decodes a standalone skeleton using the selected native layout.
    pub fn read(data: &'data [u8], flags: ModelFlags) -> Result<Self, ReadError> {
        Self::read_with_sequences(data, flags, &[])
    }
    /// Uses inherited sequence-storage flags when this document has no SKS1.
    ///
    /// An SKS1 chunk supplies its own context. Without either context, ordinary
    /// animation leaves retain unresolved external descriptors.
    pub fn read_with_sequences(
        data: &'data [u8],
        flags: ModelFlags,
        external_sequences: &[bool],
    ) -> Result<Self, ReadError> {
        // Validate framing and locate the last local context without building an
        // index. Local sequence flags apply even to chunks preceding SKS1.
        let mut sequence_chunk = None;
        for chunk in Chunks::new(data, 0) {
            let chunk = chunk
                .map_err(|error| ReadError::invalid(error.offset, "skeleton chunk framing"))?;
            if chunk.header.tag == *b"SKS1" {
                sequence_chunk = Some(chunk);
            }
        }
        let has_sequence_context = sequence_chunk.is_some() || !external_sequences.is_empty();
        let mut cached_sequences = sequence_chunk
            .map(|chunk| {
                let mut d = Decoder::new(chunk.data, L::VERSION, flags).with_origin(chunk.offset);
                d.descriptor()?;
                let reference = d.descriptor()?;
                d.records::<Sequence<L>>(reference)
            })
            .transpose()?;
        let external: Arc<[bool]> = match &cached_sequences {
            Some(sequences) => sequences
                .iter()
                .map(|sequence| sequence.flags.external_animation())
                .collect(),
            None => external_sequences.into(),
        };
        let mut chunks = Vec::new();
        for chunk in Chunks::new(data, 0) {
            let chunk = chunk
                .map_err(|error| ReadError::invalid(error.offset, "skeleton chunk framing"))?;
            let tag = chunk.header.tag;
            let data = chunk.data;
            let mut d = Decoder::new(data, L::VERSION, flags);
            d.external_sequences = external.clone();
            d.unresolved_sequences = !has_sequence_context;
            let chunk = (|| {
                Ok(match &tag {
                    b"SKL1" => {
                        let flags = SkeletonFlags::from_bits_retain(d.u32()?);
                        let name = d.descriptor()?;
                        let reserved = d
                            .bytes(4)?
                            .try_into()
                            .map_err(|_| ReadError::invalid(12, "skeleton reserved"))?;
                        SkeletonChunk::Header {
                            flags,
                            name: d.array(name)?,
                            reserved,
                        }
                    }
                    b"SKA1" => {
                        let attachments = d.descriptor()?;
                        let lookup = d.descriptor()?;
                        SkeletonChunk::Attachments {
                            attachments: d.records(attachments)?,
                            lookup: d.array(lookup)?,
                        }
                    }
                    b"SKB1" => {
                        let bones = d.descriptor()?;
                        let lookup = d.descriptor()?;
                        SkeletonChunk::Bones {
                            bones: d.records(bones)?,
                            lookup: d.array(lookup)?,
                        }
                    }
                    b"SKS1" => {
                        let global = d.descriptor()?;
                        let sequences = d.descriptor()?;
                        let lookup = d.descriptor()?;
                        let reserved = d
                            .bytes(8)?
                            .try_into()
                            .map_err(|_| ReadError::invalid(24, "skeleton reserved"))?;
                        SkeletonChunk::Sequences {
                            global_loops: d.array(global)?,
                            sequences: if sequence_chunk
                                .is_some_and(|last| last.offset == chunk.offset)
                            {
                                cached_sequences.take().expect("last SKS1 was decoded")
                            } else {
                                d.records(sequences)?
                            },
                            lookup: d.array(lookup)?,
                            reserved,
                        }
                    }
                    b"SKPD" => {
                        let reserved0 = d
                            .bytes(8)?
                            .try_into()
                            .map_err(|_| ReadError::invalid(0, "skeleton parent reserved"))?;
                        let file_id = d.u32()?;
                        let reserved1 = d
                            .bytes(4)?
                            .try_into()
                            .map_err(|_| ReadError::invalid(12, "skeleton parent reserved"))?;
                        SkeletonChunk::Parent {
                            reserved0,
                            file_id,
                            reserved1,
                        }
                    }
                    b"AFID" => SkeletonChunk::AnimationIds(flat_records(data, L::VERSION, flags)?),
                    b"BFID" => {
                        if !data.len().is_multiple_of(4) {
                            return Err(ReadError::invalid(0, "bone IDs length"));
                        }
                        SkeletonChunk::BoneIds(d.array(ArrayRef {
                            count: (data.len() / 4) as u32,
                            offset: 0,
                        })?)
                    }
                    _ => SkeletonChunk::Unknown {
                        tag,
                        data: Cow::Borrowed(data),
                    },
                })
            })()
            .map_err(|mut error: ReadError| {
                error.offset = chunk.offset.saturating_add(error.offset);
                error
            })?;
            chunks.push(chunk);
        }
        Ok(Self { flags, chunks })
    }
    /// Encodes the document with freshly assigned local offsets.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        let mut file = Encoder::new(L::VERSION, self.flags);
        for chunk in &self.chunks {
            let mut e = Encoder::new(L::VERSION, self.flags);
            let tag = match chunk {
                SkeletonChunk::Header {
                    flags,
                    name,
                    reserved,
                } => {
                    e.reserve(16)?;
                    e.u32(flags.bits())?;
                    let a = e.array(name)?;
                    e.write_descriptor(a)?;
                    e.bytes(reserved)?;
                    *b"SKL1"
                }
                SkeletonChunk::Attachments {
                    attachments,
                    lookup,
                } => {
                    e.reserve(16)?;
                    let a = e.write_records(attachments)?;
                    let b = e.array(lookup)?;
                    e.write_descriptor(a)?;
                    e.write_descriptor(b)?;
                    *b"SKA1"
                }
                SkeletonChunk::Bones { bones, lookup } => {
                    e.reserve(16)?;
                    let a = e.write_records(bones)?;
                    let b = e.array(lookup)?;
                    e.write_descriptor(a)?;
                    e.write_descriptor(b)?;
                    *b"SKB1"
                }
                SkeletonChunk::Sequences {
                    global_loops,
                    sequences,
                    lookup,
                    reserved,
                } => {
                    e.reserve(32)?;
                    let a = e.array(global_loops)?;
                    let b = e.write_records(sequences)?;
                    let c = e.array(lookup)?;
                    for a in [a, b, c] {
                        e.write_descriptor(a)?;
                    }
                    e.bytes(reserved)?;
                    *b"SKS1"
                }
                SkeletonChunk::Parent {
                    reserved0,
                    file_id,
                    reserved1,
                } => {
                    e.bytes(reserved0)?;
                    e.u32(*file_id)?;
                    e.bytes(reserved1)?;
                    *b"SKPD"
                }
                SkeletonChunk::AnimationIds(records) => {
                    e.bytes(&encode_records(records, L::VERSION, self.flags)?)?;
                    *b"AFID"
                }
                SkeletonChunk::BoneIds(ids) => {
                    e.bytes(ids.as_bytes())?;
                    *b"BFID"
                }
                SkeletonChunk::Unknown { tag, data } => {
                    e.bytes(data)?;
                    *tag
                }
            };
            write_chunk(&mut file, tag, &e.finish())?;
        }
        Ok(file.finish())
    }
}

impl<'data, V: ModelLayout> m2::Read<'data> for Skeleton<'data, V> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let origin = input.absolute_position();
        let flags = input.flags;
        let sequences = input.external_sequences.clone();
        let remaining = input.data().len() - input.position();
        Self::read_with_sequences(input.bytes(remaining)?, flags, &sequences).map_err(
            |mut error| {
                error.offset = origin.saturating_add(error.offset);
                error
            },
        )
    }
}
impl<V: ModelLayout> m2::Write for Skeleton<'_, V> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.write()?)
    }
}
