//! Alpha and early-beta group layouts, before version 17 narrowed triangle records.
use crate::model::formats::wmo::records::{BatchFlags, GroupFlags, GroupFlags2, TriangleFlags};
use crate::model::{common::Ownership, formats::wmo};
use std::borrow::Cow;
use wmo::codec::{Array, Decoder, Encoder};
use wmo::document::{frame, frames};
use wmo::group::{GroupChunk, Liquid, LiquidHeader, LiquidVertex, LiquidVertices, WaterVertex};
use wmo::records::CullingBox;
use wmo::{Classic, Read, ReadError, RecordLayout, Validate, Write, WriteError};

/// Version-14 header window into vertices and draw batches.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct HeaderBatchRange {
    /// First vertex in the MOVT array.
    pub vertex_start: u16,
    /// Number of vertices in this window.
    pub vertex_count: u16,
    /// First draw batch in MOBA.
    pub batch_start: u16,
    /// Number of draw batches in this window.
    pub batch_count: u16,
}
/// The 128-byte version-14 MOGP header.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct GroupHeader14 {
    /// Group-name byte offset in root MOGN.
    pub name_offset: u32,
    /// Descriptive-name byte offset in root MOGN.
    pub descriptive_name_offset: u32,
    /// Native group flags, including the legacy lightmap bit.
    pub flags: GroupFlags,
    /// Axis-aligned minimum corner in model coordinates.
    pub bounds_minimum: [f32; 3],
    /// Axis-aligned maximum corner in model coordinates.
    pub bounds_maximum: [f32; 3],
    /// First portal reference, retained as a signed 32-bit value.
    pub portal_start: i32,
    /// Number of portal references belonging to this group.
    pub portal_count: u32,
    /// Four root fog indices.
    pub fog_indices: [u8; 4],
    /// Native group liquid code.
    pub liquid_type: u32,
    /// Four interior vertex/batch windows.
    pub interior_batches: [HeaderBatchRange; 4],
    /// Four exterior vertex/batch windows.
    pub exterior_batches: [HeaderBatchRange; 4],
    /// Native group identity associated with area metadata.
    pub unique_id: i32,
    /// Eight preserved trailing bytes.
    pub reserved_tail: [u8; 8],
}
/// The 68-byte version-16 MOGP header with its signed portal-start field.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct GroupHeader16 {
    /// Group-name byte offset in root MOGN.
    pub name_offset: u32,
    /// Descriptive-name byte offset in root MOGN.
    pub descriptive_name_offset: u32,
    /// Native group flags.
    pub flags: GroupFlags,
    /// Axis-aligned minimum corner in model coordinates.
    pub bounds_minimum: [f32; 3],
    /// Axis-aligned maximum corner in model coordinates.
    pub bounds_maximum: [f32; 3],
    /// First portal reference as a signed 16-bit value.
    pub portal_start: i16,
    /// Number of portal references belonging to this group.
    pub portal_count: u16,
    /// Number of transition rendering batches.
    pub transition_batch_count: u16,
    /// Number of interior rendering batches.
    pub interior_batch_count: u16,
    /// Number of exterior rendering batches.
    pub exterior_batch_count: u16,
    /// Fourth batch-count slot, retained without assigning semantics.
    pub reserved_batch_count: u16,
    /// Four root fog indices.
    pub fog_indices: [u8; 4],
    /// Native group liquid code.
    pub liquid_type: u32,
    /// Native group identity associated with area metadata.
    pub unique_id: i32,
    /// Preserved secondary flag word.
    pub flags2: GroupFlags2,
    /// Preserved last header word.
    pub reserved_tail: u32,
}
/// The 24-byte version-14 MOBA record with lightmap and texture selectors.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct Batch14 {
    /// Legacy lightmap selector.
    pub lightmap: u8,
    /// Root material index in the leading legacy selector byte.
    pub material: u8,
    /// Rounded model-space culling bounds.
    pub bounds: CullingBox,
    /// First triangle index, stored in sixteen bits in this layout.
    pub start_index: u16,
    /// Number of index elements used by this batch.
    pub index_count: u16,
    /// First referenced vertex.
    pub minimum_vertex: u16,
    /// Last referenced vertex, inclusive.
    pub maximum_vertex: u16,
    /// Native batch flags.
    pub flags: BatchFlags,
    /// Preserved final byte, not a material ID in version 14.
    pub reserved: u8,
}
/// The 32-byte version-16 MOBA record retaining its eight-byte extension.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct Batch16 {
    /// Rounded model-space culling bounds.
    pub bounds: CullingBox,
    /// First triangle index, stored in thirty-two bits.
    pub start_index: u32,
    /// Number of consecutive index elements.
    pub index_count: u16,
    /// First referenced vertex.
    pub minimum_vertex: u16,
    /// Last referenced vertex, inclusive.
    pub maximum_vertex: u16,
    /// Native rendering flags.
    pub flags: BatchFlags,
    /// Root material index.
    pub material: u8,
    /// Eight unresolved trailing bytes, absent from version 17 batches.
    pub extension: [u8; 8],
}
/// Four-byte MOPY record shared by versions 14 and 16.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct LegacyTriangle {
    /// Triangle collision and drawing flags.
    pub flags: TriangleFlags,
    /// Legacy lightmap texture selection.
    pub lightmap: u8,
    /// Root material index; 0xff denotes a collision-only face.
    pub material: u8,
    /// Preserved alignment byte.
    pub reserved: u8,
}
/// Four-byte MOLM rectangle within a legacy lightmap.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct LightMap {
    /// Rectangle X coordinate in native lightmap texels.
    pub x: u8,
    /// Rectangle Y coordinate in native lightmap texels.
    pub y: u8,
    /// Rectangle width in native lightmap texels.
    pub width: u8,
    /// Rectangle height in native lightmap texels.
    pub height: u8,
}
/// One MOLD lightmap image followed by four preserved padding bytes.
#[derive(
    Clone, Debug, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct LightMapData {
    /// 32768 encoded texel bytes; channel interpretation is not inferred.
    pub texels: [u8; 32768],
    /// Four preserved trailing padding bytes.
    pub padding: [u8; 4],
}
/// Four-byte MPBV or MPBP window in its corresponding legacy stream.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct BatchRange {
    /// First element in the corresponding stream.
    pub start: u16,
    /// Number of consecutive elements.
    pub count: u16,
}
/// A chunk whose batch record is selected by the historical document type.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum LegacyGroupChunk<'data, B> {
    /// MLIQ grids with the historical unsigned magma-coordinate interpretation.
    Liquid(LegacyLiquid<'data>),
    /// MOPY four-byte triangle records.
    Triangles(Array<'data, LegacyTriangle>),
    /// MOBA batches in the selected historical layout.
    Batches(Array<'data, B>),
    /// MOIN sixteen-bit vertex indices.
    LegacyIndices(Array<'data, u16>),
    /// MOLV lightmap texture coordinates.
    LightMapCoordinates(Array<'data, [f32; 2]>),
    /// MOLM lightmap rectangles.
    LightMaps(Array<'data, LightMap>),
    /// MOLD fixed-size encoded lightmap images and padding.
    LightMapData(Array<'data, LightMapData>),
    /// MPBV legacy vertex windows.
    BatchVertices(Array<'data, BatchRange>),
    /// MPBP legacy plane windows.
    BatchPlanes(Array<'data, BatchRange>),
    /// MPBI legacy sixteen-bit index elements.
    BatchIndices(Array<'data, u16>),
    /// MPBG legacy three-component geometry values.
    BatchGeometry(Array<'data, [f32; 3]>),
    /// Payload whose fixed layout is unchanged from the classic version-17 group codec.
    Common(GroupChunk<'data, Classic>),
}
/// Version-14 group chunks with the native 24-byte lightmapped batch type.
pub type GroupChunk14<'data> = LegacyGroupChunk<'data, Batch14>;
/// Version-16 group chunks with the native 32-byte batch type.
pub type GroupChunk16<'data> = LegacyGroupChunk<'data, Batch16>;
impl<'data, B: RecordLayout> LegacyGroupChunk<'data, B> {
    /// Decodes one payload using the historical batch width and triangle layout.
    pub fn decode(tag: [u8; 4], bytes: &'data [u8], origin: usize) -> Result<Self, ReadError> {
        if matches!(&tag, b"MVER" | b"MOHD" | b"MOGP" | b"MOMO") {
            return Err(ReadError::invalid(
                origin,
                "legacy chunk requires document framing",
            ));
        }
        Ok(match &tag {
            b"MLIQ" => Self::Liquid(LegacyLiquid::decode(bytes, origin)?),
            b"MOPY" => Self::Triangles(array(bytes, origin)?),
            b"MOBA" => Self::Batches(array(bytes, origin)?),
            b"MOIN" => Self::LegacyIndices(array(bytes, origin)?),
            b"MOLV" => Self::LightMapCoordinates(array(bytes, origin)?),
            b"MOLM" => Self::LightMaps(array(bytes, origin)?),
            b"MOLD" => Self::LightMapData(array(bytes, origin)?),
            b"MPBV" => Self::BatchVertices(array(bytes, origin)?),
            b"MPBP" => Self::BatchPlanes(array(bytes, origin)?),
            b"MPBI" => Self::BatchIndices(array(bytes, origin)?),
            b"MPBG" => Self::BatchGeometry(array(bytes, origin)?),
            _ => Self::Common(GroupChunk::decode(tag, bytes, origin)?),
        })
    }
    /// Logical FourCC before reversal into native wire order.
    pub fn tag(&self) -> [u8; 4] {
        match self {
            Self::Liquid(_) => *b"MLIQ",
            Self::Triangles(_) => *b"MOPY",
            Self::Batches(_) => *b"MOBA",
            Self::LegacyIndices(_) => *b"MOIN",
            Self::LightMapCoordinates(_) => *b"MOLV",
            Self::LightMaps(_) => *b"MOLM",
            Self::LightMapData(_) => *b"MOLD",
            Self::BatchVertices(_) => *b"MPBV",
            Self::BatchPlanes(_) => *b"MPBP",
            Self::BatchIndices(_) => *b"MPBI",
            Self::BatchGeometry(_) => *b"MPBG",
            Self::Common(value) => value.tag(),
        }
    }
    /// Encodes exact historical records without modernizing their fields.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        let bytes = match self {
            Self::Liquid(value) => return value.encode(),
            Self::Triangles(v) => v.as_bytes(),
            Self::Batches(v) => v.as_bytes(),
            Self::LegacyIndices(v) => v.as_bytes(),
            Self::LightMapCoordinates(v) => v.as_bytes(),
            Self::LightMaps(v) => v.as_bytes(),
            Self::LightMapData(v) => v.as_bytes(),
            Self::BatchVertices(v) => v.as_bytes(),
            Self::BatchPlanes(v) => v.as_bytes(),
            Self::BatchIndices(v) => v.as_bytes(),
            Self::BatchGeometry(v) => v.as_bytes(),
            Self::Common(value) => {
                if matches!(
                    &value.tag(),
                    b"MOPY"
                        | b"MOBA"
                        | b"MOLV"
                        | b"MOLM"
                        | b"MOLD"
                        | b"MOIN"
                        | b"MPBV"
                        | b"MPBP"
                        | b"MPBI"
                        | b"MPBG"
                        | b"MLIQ"
                ) {
                    return Err(WriteError::invalid(
                        "historical chunk requires native payload type",
                    ));
                }
                return value.encode();
            }
        };
        let mut output = Encoder::new();
        output.bytes(bytes)?;
        Ok(output.finish())
    }
}
fn array<'data, T: RecordLayout>(
    bytes: &'data [u8],
    origin: usize,
) -> Result<Array<'data, T>, ReadError> {
    Array::decode(bytes).map_err(|_| ReadError::invalid(origin, "legacy group record width"))
}

/// Position of a mandatory legacy header or retained outer payload.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum LegacyGroupEntry<'data> {
    /// The standalone MVER frame.
    Version,
    /// The MOGP frame containing the mandatory header and optionally its body.
    Body,
    /// One body chunk at its original position in header-only standalone framing.
    BodyPayload,
    /// An outer frame not interpreted as group geometry.
    Other {
        /// Logical tag before wire reversal.
        tag: [u8; 4],
        /// Exact native payload bytes.
        data: Cow<'data, [u8]>,
    },
}
/// Native version-14 group, standalone or embedded in an alpha root.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct Group14<'data> {
    /// Required 128-byte group header.
    pub header: GroupHeader14,
    /// Ordered group-body chunks, including repeated liquid and texture layers.
    pub chunks: Vec<GroupChunk14<'data>>,
    /// Original standalone mandatory-header and outer-chunk order.
    entries: Vec<LegacyGroupEntry<'data>>,
    /// Whether the MOGP frame encloses body chunks or only its header.
    pub framing: wmo::GroupFraming,
}
/// Native version-16 group with four-byte triangles and 32-byte draw batches.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct Group16<'data> {
    /// Required 68-byte group header.
    pub header: GroupHeader16,
    /// Ordered group-body chunks in their historical representation.
    pub chunks: Vec<GroupChunk16<'data>>,
    /// Original mandatory-header and outer-chunk order.
    entries: Vec<LegacyGroupEntry<'data>>,
    /// Whether MOGP encloses body chunks or only its header.
    pub framing: wmo::GroupFraming,
}
impl Default for Group14<'_> {
    fn default() -> Self {
        Self::new()
    }
}
impl Default for Group16<'_> {
    fn default() -> Self {
        Self::new()
    }
}
impl<'data> Group14<'data> {
    /// Original immutable standalone header and outer-payload order.
    pub fn entries(&self) -> &[LegacyGroupEntry<'data>] {
        &self.entries
    }

    /// Creates a standalone version-14 group with an empty nested body.
    pub fn new() -> Self {
        Self {
            header: GroupHeader14::default(),
            chunks: Vec::new(),
            entries: vec![LegacyGroupEntry::Version, LegacyGroupEntry::Body],
            framing: wmo::GroupFraming::Nested,
        }
    }
    /// Reads standalone MVER/MOGP framing or an embedded MOGP frame without MVER.
    pub fn decode(bytes: &'data [u8]) -> Result<Self, ReadError> {
        let DecodedGroup {
            header,
            chunks,
            entries,
            framing,
        } = decode_document(bytes, 14, false)?;
        Ok(Self {
            header,
            chunks,
            entries,
            framing,
        })
    }
    /// Reads the MOGP payload directly, as used by alpha roots.
    pub fn decode_payload(bytes: &'data [u8]) -> Result<Self, ReadError> {
        let (header, chunks) = decode_payload(bytes, 0)?;
        Ok(Self {
            header,
            chunks,
            entries: vec![LegacyGroupEntry::Body],
            framing: wmo::GroupFraming::Nested,
        })
    }
    /// Writes the native payload without MOGP or MVER framing.
    pub fn encode_payload(&self) -> Result<Vec<u8>, WriteError> {
        encode_payload(&self.header, &self.chunks)
    }
    /// Writes original standalone framing, retaining unversioned embedded conventions.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        encode_document(&self.header, &self.chunks, &self.entries, self.framing, 14)
    }
}
impl<'data> Group16<'data> {
    /// Original immutable standalone header and outer-payload order.
    pub fn entries(&self) -> &[LegacyGroupEntry<'data>] {
        &self.entries
    }

    /// Creates a standalone version-16 group with an empty nested body.
    pub fn new() -> Self {
        Self {
            header: GroupHeader16::default(),
            chunks: Vec::new(),
            entries: vec![LegacyGroupEntry::Version, LegacyGroupEntry::Body],
            framing: wmo::GroupFraming::Nested,
        }
    }
    /// Reads a standalone group and requires its version-16 MVER.
    pub fn decode(bytes: &'data [u8]) -> Result<Self, ReadError> {
        let DecodedGroup {
            header,
            chunks,
            entries,
            framing,
        } = decode_document(bytes, 16, true)?;
        Ok(Self {
            header,
            chunks,
            entries,
            framing,
        })
    }
    /// Reads the MOGP payload directly without standalone framing.
    pub fn decode_payload(bytes: &'data [u8]) -> Result<Self, ReadError> {
        let (header, chunks) = decode_payload(bytes, 0)?;
        Ok(Self {
            header,
            chunks,
            entries: vec![LegacyGroupEntry::Version, LegacyGroupEntry::Body],
            framing: wmo::GroupFraming::Nested,
        })
    }
    /// Writes the native MOGP payload without framing.
    pub fn encode_payload(&self) -> Result<Vec<u8>, WriteError> {
        encode_payload(&self.header, &self.chunks)
    }
    /// Writes original standalone chunk order and nesting convention.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        validate_entries(&self.entries, true)
            .map_err(|_| WriteError::invalid("legacy mandatory header cardinality"))?;
        encode_document(&self.header, &self.chunks, &self.entries, self.framing, 16)
    }
}
fn decode_payload<'data, H: Read<'data> + RecordLayout, B: RecordLayout>(
    bytes: &'data [u8],
    origin: usize,
) -> Result<(H, Vec<LegacyGroupChunk<'data, B>>), ReadError> {
    let mut input = Decoder::new(bytes).with_origin(origin);
    let header = H::read(&mut input)?;
    if input.position() != H::size() {
        return Err(ReadError::invalid(origin, "legacy header width"));
    }
    let rest = input.bytes(input.remaining())?;
    let chunks = frames(rest, origin + H::size())?
        .into_iter()
        .map(|(tag, payload, at)| LegacyGroupChunk::decode(tag, payload, at))
        .collect::<Result<_, _>>()?;
    Ok((header, chunks))
}
struct DecodedGroup<'data, H, B> {
    header: H,
    chunks: Vec<LegacyGroupChunk<'data, B>>,
    entries: Vec<LegacyGroupEntry<'data>>,
    framing: wmo::GroupFraming,
}
fn decode_document<'data, H: Read<'data> + RecordLayout, B: RecordLayout>(
    bytes: &'data [u8],
    version: u32,
    require_version: bool,
) -> Result<DecodedGroup<'data, H, B>, ReadError> {
    let mut header = None;
    let mut saw_version = false;
    let mut entries = Vec::new();
    let mut body = Vec::new();
    let mut framing = wmo::GroupFraming::Nested;
    for (tag, payload, origin) in frames(bytes, 0)? {
        match &tag {
            b"MVER" => {
                if saw_version {
                    return Err(ReadError::invalid(origin, "duplicate legacy MVER"));
                }
                let mut input = Decoder::new(payload).with_origin(origin);
                let value: u32 = input.read()?;
                if input.remaining() != 0 || value != version {
                    return Err(ReadError::invalid(origin, "legacy group version"));
                }
                saw_version = true;
                entries.push(LegacyGroupEntry::Version);
            }
            b"MOGP" => {
                if header.is_some() {
                    return Err(ReadError::invalid(origin, "duplicate legacy MOGP"));
                }
                let (value, chunks) = decode_payload(payload, origin)?;
                header = Some(value);
                body.extend(chunks);
                entries.push(LegacyGroupEntry::Body);
                if payload.len() == H::size() {
                    framing = wmo::GroupFraming::HeaderOnly;
                }
            }
            b"MOHD" | b"MOMO" => {
                return Err(ReadError::invalid(origin, "root framing in legacy group"));
            }
            _ if header.is_some() && framing == wmo::GroupFraming::HeaderOnly => {
                body.push(LegacyGroupChunk::decode(tag, payload, origin)?);
                entries.push(LegacyGroupEntry::BodyPayload);
            }
            _ => entries.push(LegacyGroupEntry::Other {
                tag,
                data: Cow::Borrowed(payload),
            }),
        }
    }
    if require_version && !saw_version {
        return Err(ReadError::invalid(0, "missing legacy group MVER"));
    }
    Ok(DecodedGroup {
        header: header.ok_or_else(|| ReadError::invalid(0, "missing legacy MOGP"))?,
        chunks: body,
        entries,
        framing,
    })
}
fn encode_payload<H: Write, B: RecordLayout>(
    header: &H,
    chunks: &[LegacyGroupChunk<'_, B>],
) -> Result<Vec<u8>, WriteError> {
    let mut output = Encoder::new();
    header.write(&mut output)?;
    for chunk in chunks {
        if matches!(&chunk.tag(), b"MVER" | b"MOHD" | b"MOGP" | b"MOMO") {
            return Err(WriteError::invalid("reserved legacy group body tag"));
        }
        frame(&mut output, chunk.tag(), &chunk.encode()?)?;
    }
    Ok(output.finish())
}
fn encode_document<H: Write + RecordLayout, B: RecordLayout>(
    header: &H,
    chunks: &[LegacyGroupChunk<'_, B>],
    entries: &[LegacyGroupEntry<'_>],
    framing: wmo::GroupFraming,
    version: u32,
) -> Result<Vec<u8>, WriteError> {
    let mut header_output = Encoder::new();
    header.write(&mut header_output)?;
    let header_bytes = header_output.finish();
    let mut payloads = Vec::new();
    for chunk in chunks {
        if matches!(&chunk.tag(), b"MVER" | b"MOHD" | b"MOGP" | b"MOMO") {
            return Err(WriteError::invalid("reserved legacy group body tag"));
        }
        let mut encoded = Encoder::new();
        frame(&mut encoded, chunk.tag(), &chunk.encode()?)?;
        payloads.push(encoded.finish());
    }
    let slots = entries
        .iter()
        .any(|entry| matches!(entry, LegacyGroupEntry::BodyPayload));
    let mut next = 0;
    let mut output = Encoder::new();
    let mut bodies = 0;
    let mut versions = 0;
    for entry in entries {
        match entry {
            LegacyGroupEntry::Version => {
                versions += 1;
                frame(&mut output, *b"MVER", &version.to_le_bytes())?;
            }
            LegacyGroupEntry::Body => {
                bodies += 1;
                match framing {
                    wmo::GroupFraming::Nested => {
                        let mut payload = header_bytes.clone();
                        for encoded in &payloads {
                            payload.extend_from_slice(encoded);
                        }
                        frame(&mut output, *b"MOGP", &payload)?;
                    }
                    wmo::GroupFraming::HeaderOnly => {
                        frame(&mut output, *b"MOGP", &header_bytes)?;
                        if !slots {
                            for encoded in &payloads {
                                output.bytes(encoded)?;
                            }
                            next = payloads.len();
                        }
                    }
                }
            }
            LegacyGroupEntry::BodyPayload if framing == wmo::GroupFraming::HeaderOnly => {
                if let Some(encoded) = payloads.get(next) {
                    output.bytes(encoded)?;
                    next += 1;
                }
            }
            LegacyGroupEntry::BodyPayload => {}
            LegacyGroupEntry::Other { tag, data } => {
                if matches!(tag, b"MVER" | b"MOHD" | b"MOGP" | b"MOMO") {
                    return Err(WriteError::invalid("reserved legacy outer tag"));
                }
                frame(&mut output, *tag, data)?;
            }
        }
    }
    if framing == wmo::GroupFraming::HeaderOnly {
        for encoded in &payloads[next..] {
            output.bytes(encoded)?;
        }
    }
    if bodies != 1 || versions > 1 {
        return Err(WriteError::invalid("legacy mandatory header cardinality"));
    }
    Ok(output.finish())
}
impl<'data> Read<'data> for Group14<'data> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let origin = input.absolute_position();
        Self::decode(input.bytes(input.remaining())?).map_err(|mut error| {
            error.offset = origin.saturating_add(error.offset);
            error
        })
    }
}
impl Write for Group14<'_> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.encode()?)
    }
}
impl<'data> Read<'data> for Group16<'data> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let origin = input.absolute_position();
        Self::decode(input.bytes(input.remaining())?).map_err(|mut error| {
            error.offset = origin.saturating_add(error.offset);
            error
        })
    }
}
impl Write for Group16<'_> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.encode()?)
    }
}

/// Geometry inspection shared by the two historical draw-batch widths.
pub trait LegacyBatchLayout: RecordLayout + for<'data> Read<'data> + Validate {
    /// First index, index count, and inclusive first/last vertex.
    fn geometry_range(&self) -> (usize, usize, usize, usize);
}
impl LegacyBatchLayout for Batch14 {
    fn geometry_range(&self) -> (usize, usize, usize, usize) {
        (
            self.start_index.into(),
            self.index_count.into(),
            self.minimum_vertex.into(),
            self.maximum_vertex.into(),
        )
    }
}
impl LegacyBatchLayout for Batch16 {
    fn geometry_range(&self) -> (usize, usize, usize, usize) {
        (
            self.start_index as usize,
            self.index_count.into(),
            self.minimum_vertex.into(),
            self.maximum_vertex.into(),
        )
    }
}
fn validate_chunks<B: LegacyBatchLayout>(
    chunks: &[LegacyGroupChunk<'_, B>],
) -> Result<(), ReadError> {
    let mut ordinary = wmo::Group::<Classic>::new();
    let mut triangles = None;
    let mut batches = None;
    for chunk in chunks {
        match chunk {
            LegacyGroupChunk::Liquid(liquid) => liquid.validate()?,
            LegacyGroupChunk::Common(GroupChunk::Liquid(liquid)) => liquid.validate()?,
            LegacyGroupChunk::Common(value) => ordinary.chunks_mut().push(value.as_borrowed()),
            LegacyGroupChunk::Triangles(values) => {
                values.validate()?;
                if triangles.replace(values.len()).is_some() {
                    return Err(ReadError::invalid(0, "duplicate legacy triangles"));
                }
            }
            LegacyGroupChunk::Batches(values) => {
                values.validate()?;
                if batches.replace(values).is_some() {
                    return Err(ReadError::invalid(0, "duplicate legacy batches"));
                }
            }
            LegacyGroupChunk::LegacyIndices(values) => ordinary
                .chunks_mut()
                .push(GroupChunk::Indices(values.as_borrowed())),
            LegacyGroupChunk::LightMapCoordinates(values) => values.validate()?,
            LegacyGroupChunk::LightMaps(values) => values.validate()?,
            LegacyGroupChunk::LightMapData(values) => values.validate()?,
            LegacyGroupChunk::BatchVertices(values) | LegacyGroupChunk::BatchPlanes(values) => {
                values.validate()?
            }
            LegacyGroupChunk::BatchIndices(values) => values.validate()?,
            LegacyGroupChunk::BatchGeometry(values) => values.validate()?,
        }
    }
    // Collision links, ordinary vertex streams and light windows retain their common layouts.
    ordinary.validate()?;
    let vertices = ordinary
        .chunks()
        .iter()
        .find_map(|chunk| match chunk {
            GroupChunk::Positions(v) => Some(v.len()),
            _ => None,
        })
        .unwrap_or(0);
    let indices = ordinary
        .chunks()
        .iter()
        .find_map(|chunk| match chunk {
            GroupChunk::Indices(v) => Some(v.len()),
            _ => None,
        })
        .unwrap_or(0);
    if triangles.is_some_and(|count| count != indices / 3) {
        return Err(ReadError::invalid(0, "legacy triangle cardinality"));
    }
    if let Some(batches) = batches {
        for value in batches.iter() {
            let (first, count, minimum, maximum) = value?.geometry_range();
            if first.checked_add(count).is_none_or(|end| end > indices) {
                return Err(ReadError::invalid(0, "legacy batch index range"));
            }
            if count != 0 && (minimum > maximum || maximum >= vertices) {
                return Err(ReadError::invalid(0, "legacy batch vertex range"));
            }
        }
    }
    Ok(())
}
fn validate_entries(
    entries: &[LegacyGroupEntry<'_>],
    require_version: bool,
) -> Result<(), ReadError> {
    let bodies = entries
        .iter()
        .filter(|entry| matches!(entry, LegacyGroupEntry::Body))
        .count();
    let versions = entries
        .iter()
        .filter(|entry| matches!(entry, LegacyGroupEntry::Version))
        .count();
    if bodies != 1 || versions > 1 || require_version && versions != 1 {
        return Err(ReadError::invalid(0, "legacy mandatory header cardinality"));
    }
    Ok(())
}
impl Validate for Group14<'_> {
    fn validate(&self) -> Result<(), ReadError> {
        self.header.validate()?;
        validate_entries(&self.entries, false)?;
        validate_chunks(&self.chunks)?;
        let vertices = self
            .chunks
            .iter()
            .find_map(|chunk| match chunk {
                LegacyGroupChunk::Common(GroupChunk::Positions(v)) => Some(v.len()),
                _ => None,
            })
            .unwrap_or(0);
        let batches = self
            .chunks
            .iter()
            .find_map(|chunk| match chunk {
                LegacyGroupChunk::Batches(v) => Some(v.len()),
                _ => None,
            })
            .unwrap_or(0);
        for range in self
            .header
            .interior_batches
            .iter()
            .chain(&self.header.exterior_batches)
        {
            if range.vertex_count != 0
                && usize::from(range.vertex_start) + usize::from(range.vertex_count) > vertices
                || range.batch_count != 0
                    && usize::from(range.batch_start) + usize::from(range.batch_count) > batches
            {
                return Err(ReadError::invalid(0, "legacy header batch window"));
            }
        }
        Ok(())
    }
}
impl Validate for Group16<'_> {
    fn validate(&self) -> Result<(), ReadError> {
        self.header.validate()?;
        validate_entries(&self.entries, true)?;
        validate_chunks(&self.chunks)?;
        let batches = self
            .chunks
            .iter()
            .find_map(|chunk| match chunk {
                LegacyGroupChunk::Batches(v) => Some(v.len()),
                _ => None,
            })
            .unwrap_or(0);
        if usize::from(self.header.transition_batch_count)
            + usize::from(self.header.interior_batch_count)
            + usize::from(self.header.exterior_batch_count)
            > batches
        {
            return Err(ReadError::invalid(0, "legacy header batch counts"));
        }
        Ok(())
    }
}

/// Historical magma/slime liquid vertex with two unsigned texture coordinates.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct LegacyMagmaVertex {
    /// Native unsigned texture coordinates S and T.
    pub texture_coordinates: [u16; 2],
    /// Liquid surface height in model-space units.
    pub height: f32,
}
/// Version-14/16 MLIQ grids, retaining the liquid union until explicitly inspected.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct LegacyLiquid<'data> {
    /// Common packed thirty-byte dimensions, origin and material reference.
    pub header: LiquidHeader,
    /// Raw eight-byte vertices shared by water and the historical unsigned magma representation.
    pub vertices: Array<'data, LiquidVertex>,
    /// Native tile-type and fishable/shared flag bytes.
    pub tiles: Array<'data, u8>,
}
impl<'data> LegacyLiquid<'data> {
    /// Decodes the common grid framing without imposing a modern magma interpretation.
    pub fn decode(bytes: &'data [u8], origin: usize) -> Result<Self, ReadError> {
        let liquid = Liquid::decode(bytes, origin, None)?;
        let LiquidVertices::Unresolved(vertices) = liquid.vertices else {
            return Err(ReadError::invalid(origin, "legacy liquid union context"));
        };
        Ok(Self {
            header: liquid.header,
            vertices,
            tiles: liquid.tiles,
        })
    }
    /// Borrows the explicit water/ocean reading without copying encoded vertices.
    pub fn water_vertices(&self) -> Result<Array<'_, WaterVertex>, ReadError> {
        Array::decode(self.vertices.as_bytes())
    }
    /// Borrows the historical unsigned magma/slime reading without copying vertices.
    pub fn magma_vertices(&self) -> Result<Array<'_, LegacyMagmaVertex>, ReadError> {
        Array::decode(self.vertices.as_bytes())
    }
    /// Writes the native header and grids without converting the union data.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        self.validate()
            .map_err(|_| WriteError::invalid("legacy liquid grid dimensions"))?;
        let mut output = Encoder::new();
        self.header.write(&mut output)?;
        self.vertices.write(&mut output)?;
        self.tiles.write(&mut output)?;
        Ok(output.finish())
    }
}
impl Validate for LegacyLiquid<'_> {
    fn validate(&self) -> Result<(), ReadError> {
        let liquid = Liquid {
            header: self.header,
            vertices: LiquidVertices::Unresolved(Array::decode(self.vertices.as_bytes())?),
            tiles: Array::decode(self.tiles.as_bytes())?,
        };
        liquid.validate()
    }
}
