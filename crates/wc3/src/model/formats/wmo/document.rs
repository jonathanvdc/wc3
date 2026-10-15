//! Ordered wire documents and typed mandatory-header envelopes.
use super::codec::{Decoder, Encoder};
use super::group::{GroupChunk, GroupContext};
use super::records::{GroupHeader, Header};
use super::root::RootChunk;
use super::{Dialect, Read, ReadError, RecordLayout, Validate, Write, WriteError};
use crate::model::common::{binary::Chunks, Ownership};
use std::borrow::Cow;

/// A canonical FourCC and its uninterpreted payload.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct RawChunk<'data> {
    /// Logical tag, such as `MOHD`; encoding reverses it into WMO wire order.
    pub tag: [u8; 4],
    /// Borrowed original bytes or an edited owned payload.
    pub data: Cow<'data, [u8]>,
}
/// Bounds-checked chunks without model validity or dialect assumptions.
#[derive(Clone, Debug, Default, PartialEq, Ownership)]
pub struct RawDocument<'data> {
    /// Every chunk in physical order, including duplicate headers.
    pub chunks: Vec<RawChunk<'data>>,
}
impl<'data> RawDocument<'data> {
    /// Reads reversed WMO FourCCs and checked payload sizes.
    pub fn decode(data: &'data [u8]) -> Result<Self, ReadError> {
        Ok(Self {
            chunks: frames(data, 0)?
                .into_iter()
                .map(|(tag, data, _)| RawChunk {
                    tag,
                    data: Cow::Borrowed(data),
                })
                .collect(),
        })
    }
    /// Writes the ordered frames with recalculated payload lengths.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        let mut output = Encoder::new();
        for chunk in &self.chunks {
            frame(&mut output, chunk.tag, &chunk.data)?;
        }
        Ok(output.finish())
    }
}
pub(crate) type Frame<'data> = ([u8; 4], &'data [u8], usize);

pub(crate) fn frames(data: &[u8], origin: usize) -> Result<Vec<Frame<'_>>, ReadError> {
    Chunks::new(data, origin)
        .map(|chunk| {
            let chunk = chunk.map_err(|error| ReadError::invalid(error.offset, "chunk bounds"))?;
            let mut tag = chunk.header.tag;
            tag.reverse();
            Ok((tag, chunk.data, chunk.offset))
        })
        .collect()
}
pub(crate) fn frame(output: &mut Encoder, mut tag: [u8; 4], data: &[u8]) -> Result<(), WriteError> {
    tag.reverse();
    output.bytes(&tag)?;
    output.write(&u32::try_from(data.len()).map_err(|_| WriteError::invalid("chunk length"))?)?;
    output.bytes(data)
}
fn version(data: &[u8], origin: usize, expected: u32) -> Result<(), ReadError> {
    if data.len() != 4 {
        return Err(ReadError::invalid(origin, "MVER width"));
    }
    if u32::read(&mut Decoder::new(data).with_origin(origin))? != expected {
        return Err(ReadError::invalid(origin, "unsupported WMO version"));
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Ownership)]
enum RootEntry<'data, D: Dialect> {
    Version,
    Header,
    Payload(RootChunk<'data, D>),
}
/// A WMO root with exactly one version and root header.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct Root<'data, D: Dialect> {
    header: Header<D>,
    entries: Vec<RootEntry<'data, D>>,
}
impl<'data, D: Dialect> Default for Root<'data, D> {
    fn default() -> Self {
        Self::new()
    }
}
impl<'data, D: Dialect> Root<'data, D> {
    /// Creates a root with an empty header and standard header order.
    pub fn new() -> Self {
        Self {
            header: Header::default(),
            entries: vec![RootEntry::Version, RootEntry::Header],
        }
    }
    /// Required root header, independent of its original chunk position.
    pub fn header(&self) -> &Header<D> {
        &self.header
    }
    /// Edits the required header without allowing removal of its framing.
    pub fn header_mut(&mut self) -> &mut Header<D> {
        &mut self.header
    }
    /// Native payload chunks, excluding MVER and MOHD.
    pub fn chunks(&self) -> impl Iterator<Item = &RootChunk<'data, D>> {
        self.entries.iter().filter_map(|entry| match entry {
            RootEntry::Payload(chunk) => Some(chunk),
            _ => None,
        })
    }
    /// Edits payloads while retaining mandatory header slots.
    pub fn chunks_mut(&mut self) -> impl Iterator<Item = &mut RootChunk<'data, D>> {
        self.entries.iter_mut().filter_map(|entry| match entry {
            RootEntry::Payload(chunk) => Some(chunk),
            _ => None,
        })
    }
    /// Inserts a payload before the given payload index, or appends at the end.
    pub fn insert_chunk(
        &mut self,
        index: usize,
        chunk: RootChunk<'data, D>,
    ) -> Result<(), WriteError> {
        if matches!(chunk, RootChunk::Header(_))
            || matches!(&chunk.tag(), b"MVER" | b"MOHD" | b"MOGP")
        {
            return Err(WriteError::invalid("reserved root header tag"));
        }
        let position = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| matches!(entry, RootEntry::Payload(_)))
            .nth(index)
            .map(|(position, _)| position);
        let count = self.chunks().count();
        if index > count {
            return Err(WriteError::invalid("chunk insertion position"));
        }
        self.entries.insert(
            position.unwrap_or(self.entries.len()),
            RootEntry::Payload(chunk),
        );
        Ok(())
    }
    /// Removes matching payload chunks without disturbing mandatory headers.
    pub fn remove_chunks(&mut self, tag: [u8; 4]) {
        self.entries
            .retain(|entry| !matches!(entry,RootEntry::Payload(chunk) if chunk.tag()==tag));
    }
    /// Decodes a valid root envelope; semantic reference checks are separate.
    pub fn decode(data: &'data [u8]) -> Result<Self, ReadError> {
        let mut header = None;
        let mut saw_version = false;
        let mut entries = Vec::new();
        for (tag, bytes, origin) in frames(data, 0)? {
            if tag == *b"MVER" {
                if saw_version {
                    return Err(ReadError::invalid(origin, "duplicate MVER"));
                }
                version(bytes, origin, D::VERSION)?;
                saw_version = true;
                entries.push(RootEntry::Version);
            } else if tag == *b"MOHD" {
                if header.is_some() {
                    return Err(ReadError::invalid(origin, "duplicate MOHD"));
                }
                let RootChunk::Header(value) = RootChunk::<D>::decode(tag, bytes, origin)? else {
                    return Err(ReadError::invalid(origin, "MOHD payload"));
                };
                header = Some(value);
                entries.push(RootEntry::Header);
            } else if tag == *b"MOGP" {
                return Err(ReadError::invalid(origin, "group header in root"));
            } else {
                entries.push(RootEntry::Payload(RootChunk::<D>::decode(
                    tag, bytes, origin,
                )?));
            }
        }
        if !saw_version {
            return Err(ReadError::invalid(0, "missing MVER"));
        }
        Ok(Self {
            header: header.ok_or_else(|| ReadError::invalid(0, "missing MOHD"))?,
            entries,
        })
    }
    /// Updates counts whose owning tables are unambiguous, retaining legacy exporter counts.
    pub fn synchronize_counts(&mut self) -> Result<(), WriteError> {
        self.header = self.encoded_header()?;
        Ok(())
    }
    fn encoded_header(&self) -> Result<Header<D>, WriteError> {
        let mut header = self.header.clone();
        let group_count = self
            .chunks()
            .find_map(|chunk| match chunk {
                RootChunk::Groups(values) => Some(values.len()),
                _ => None,
            })
            .or_else(|| {
                self.chunks().find_map(|chunk| match chunk {
                    RootChunk::Groups2(values) => Some(values.len()),
                    _ => None,
                })
            });
        if let Some(count) = group_count {
            header.group_count =
                u32::try_from(count).map_err(|_| WriteError::invalid("group count"))?;
        }
        for chunk in self.chunks() {
            match chunk {
                RootChunk::Portals(values) => {
                    header.portal_count = u32::try_from(values.len())
                        .map_err(|_| WriteError::invalid("portal count"))?
                }
                RootChunk::DoodadSets(values) => {
                    header.doodad_set_count = u32::try_from(values.len())
                        .map_err(|_| WriteError::invalid("doodad set count"))?
                }
                _ => {}
            }
        }
        Ok(header)
    }
    /// Writes original chunk order, recalculating chunk lengths.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        let normalized = self.encoded_header()?;
        let mut output = Encoder::new();
        for entry in &self.entries {
            match entry {
                RootEntry::Version => frame(&mut output, *b"MVER", &D::VERSION.to_le_bytes())?,
                RootEntry::Header => {
                    let mut data = Encoder::new();
                    normalized.write(&mut data)?;
                    frame(&mut output, *b"MOHD", &data.finish())?;
                }
                RootEntry::Payload(chunk) => {
                    if matches!(&chunk.tag(), b"MVER" | b"MOHD" | b"MOGP") {
                        return Err(WriteError::invalid("reserved root header tag"));
                    }
                    frame(&mut output, chunk.tag(), &chunk.encode()?)?;
                }
            }
        }
        Ok(output.finish())
    }
}
/// Physical placement of the group body relative to its MOGP frame.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GroupFraming {
    /// MOGP encloses both its fixed header and nested payload chunks.
    #[default]
    Nested,
    /// MOGP contains only its header; body chunks follow it in the file.
    HeaderOnly,
}
#[derive(Clone, Debug, PartialEq, Ownership)]
enum GroupEntry<'data> {
    Version,
    Body,
    BodyPayload,
    Other(RawChunk<'data>),
}
/// A group document with a required MOGP header and ordered geometry payloads.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct Group<'data, D: Dialect> {
    header: GroupHeader<D>,
    chunks: Vec<GroupChunk<'data, D>>,
    entries: Vec<GroupEntry<'data>>,
    framing: GroupFraming,
}
impl<'data, D: Dialect> Default for Group<'data, D> {
    fn default() -> Self {
        Self::new()
    }
}
impl<'data, D: Dialect> Group<'data, D> {
    /// Creates an empty group with nested MOGP framing.
    pub fn new() -> Self {
        Self {
            header: GroupHeader::default(),
            chunks: Vec::new(),
            entries: vec![GroupEntry::Version, GroupEntry::Body],
            framing: GroupFraming::Nested,
        }
    }
    /// Required MOGP header.
    pub fn header(&self) -> &GroupHeader<D> {
        &self.header
    }
    /// Edits the required header.
    pub fn header_mut(&mut self) -> &mut GroupHeader<D> {
        &mut self.header
    }
    /// Body payloads in native order, including repeated vertex layers.
    pub fn chunks(&self) -> &[GroupChunk<'data, D>] {
        &self.chunks
    }
    /// Edits the group body without removing its required header.
    pub fn chunks_mut(&mut self) -> &mut Vec<GroupChunk<'data, D>> {
        &mut self.chunks
    }
    /// Returns the decoded nesting convention.
    pub fn framing(&self) -> GroupFraming {
        self.framing
    }
    /// Selects nested or header-only MOGP framing for the next encoding.
    pub fn set_framing(&mut self, framing: GroupFraming) {
        self.framing = framing;
    }
    /// Decodes the group envelope and bounds every body payload independently.
    pub fn decode(data: &'data [u8]) -> Result<Self, ReadError> {
        Self::decode_with_context(data, &GroupContext::default())
    }
    /// Decodes using caller-supplied liquid interpretation when game metadata is available.
    /// The document's actual batch count supplies tangent-map context when present.
    pub fn decode_with_context(
        data: &'data [u8],
        supplied: &GroupContext,
    ) -> Result<Self, ReadError> {
        let raw = frames(data, 0)?;
        let mut header = None;
        let mut entries = Vec::new();
        let mut saw_version = false;
        let mut body = Vec::new();
        let mut framing = GroupFraming::Nested;
        for (tag, bytes, origin) in raw {
            match &tag {
                b"MVER" => {
                    if saw_version {
                        return Err(ReadError::invalid(origin, "duplicate MVER"));
                    }
                    version(bytes, origin, D::VERSION)?;
                    saw_version = true;
                    entries.push(GroupEntry::Version);
                }
                b"MOGP" => {
                    if header.is_some() {
                        return Err(ReadError::invalid(origin, "duplicate MOGP"));
                    }
                    let size = GroupHeader::<D>::size();
                    if bytes.len() < size {
                        return Err(ReadError::invalid(origin, "MOGP header width"));
                    }
                    header = Some(GroupHeader::read(
                        &mut Decoder::new(&bytes[..size]).with_origin(origin),
                    )?);
                    entries.push(GroupEntry::Body);
                    if bytes.len() == size {
                        framing = GroupFraming::HeaderOnly;
                    } else {
                        body.extend(frames(&bytes[size..], origin + size)?);
                    }
                }
                b"MOHD" => return Err(ReadError::invalid(origin, "root header in group")),
                _ if header.is_some() && framing == GroupFraming::HeaderOnly => {
                    body.push((tag, bytes, origin));
                    entries.push(GroupEntry::BodyPayload);
                }
                _ => entries.push(GroupEntry::Other(RawChunk {
                    tag,
                    data: Cow::Borrowed(bytes),
                })),
            }
        }
        if !saw_version {
            return Err(ReadError::invalid(0, "missing MVER"));
        }
        let header = header.ok_or_else(|| ReadError::invalid(0, "missing MOGP"))?;
        let batch_count = body
            .iter()
            .find(|(tag, _, _)| tag == b"MOBA")
            .map(|(_, bytes, _)| bytes.len() / 24);
        let context = GroupContext {
            batch_count: batch_count.or(supplied.batch_count),
            liquid_kind: supplied.liquid_kind,
        };
        let chunks = body
            .into_iter()
            .map(|(tag, bytes, origin)| {
                if matches!(&tag, b"MVER" | b"MOHD" | b"MOGP") {
                    Err(ReadError::invalid(origin, "reserved group header in body"))
                } else {
                    GroupChunk::<D>::decode_with_context(tag, bytes, origin, &context)
                }
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            header,
            chunks,
            entries,
            framing,
        })
    }
    /// Writes group framing and independently encoded body chunks.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        let mut header = Encoder::new();
        self.header.write(&mut header)?;
        let header = header.finish();
        let mut payloads = Vec::new();
        for chunk in &self.chunks {
            if matches!(&chunk.tag(), b"MVER" | b"MOGP" | b"MOHD") {
                return Err(WriteError::invalid("reserved group header tag"));
            }
            let mut output = Encoder::new();
            frame(&mut output, chunk.tag(), &chunk.encode()?)?;
            payloads.push(output.finish());
        }
        let has_slots = self
            .entries
            .iter()
            .any(|entry| matches!(entry, GroupEntry::BodyPayload));
        let mut next = 0;
        let mut output = Encoder::new();
        for entry in &self.entries {
            match entry {
                GroupEntry::Version => frame(&mut output, *b"MVER", &D::VERSION.to_le_bytes())?,
                GroupEntry::Body => match self.framing {
                    GroupFraming::Nested => {
                        let mut body = header.clone();
                        for bytes in &payloads {
                            body.extend_from_slice(bytes);
                        }
                        frame(&mut output, *b"MOGP", &body)?;
                    }
                    GroupFraming::HeaderOnly => {
                        frame(&mut output, *b"MOGP", &header)?;
                        if !has_slots {
                            for bytes in &payloads {
                                output.bytes(bytes)?;
                            }
                            next = payloads.len();
                        }
                    }
                },
                GroupEntry::BodyPayload if self.framing == GroupFraming::HeaderOnly => {
                    if let Some(bytes) = payloads.get(next) {
                        output.bytes(bytes)?;
                        next += 1;
                    }
                }
                GroupEntry::BodyPayload => {}
                GroupEntry::Other(chunk) => {
                    if matches!(&chunk.tag, b"MVER" | b"MOGP" | b"MOHD") {
                        return Err(WriteError::invalid("reserved group header tag"));
                    }
                    frame(&mut output, chunk.tag, &chunk.data)?;
                }
            }
        }
        if self.framing == GroupFraming::HeaderOnly {
            for bytes in &payloads[next..] {
                output.bytes(bytes)?;
            }
        }
        Ok(output.finish())
    }
}
/// A root or group document selected from its required header tag.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum File<'data, D: Dialect> {
    /// Root document with shared records and group identities.
    Root(Root<'data, D>),
    /// Geometry group document.
    Group(Group<'data, D>),
}
impl<'data, D: Dialect> File<'data, D> {
    /// Detects document role without guessing its client dialect.
    pub fn decode(data: &'data [u8]) -> Result<Self, ReadError> {
        let raw = frames(data, 0)?;
        let root = raw.iter().any(|(tag, _, _)| tag == b"MOHD");
        let group = raw.iter().any(|(tag, _, _)| tag == b"MOGP");
        match (root, group) {
            (true, false) => Ok(Self::Root(Root::decode(data)?)),
            (false, true) => Ok(Self::Group(Group::decode(data)?)),
            _ => Err(ReadError::invalid(
                0,
                "ambiguous or missing document header",
            )),
        }
    }
    /// Encodes the selected role.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        match self {
            Self::Root(root) => root.encode(),
            Self::Group(group) => group.encode(),
        }
    }
}
macro_rules! document_codec {($($ty:ident),*)=>{$(
impl<'data,D:Dialect> Read<'data> for $ty<'data,D>{fn read(input:&mut Decoder<'data>)->Result<Self,ReadError>{let origin=input.absolute_position();let length=input.remaining();Self::decode(input.bytes(length)?).map_err(|mut error|{error.offset=error.offset.saturating_add(origin);error})}}
impl<D:Dialect> Write for $ty<'_,D>{fn write(&self,output:&mut Encoder)->Result<(),WriteError>{output.bytes(&self.encode()?)}}
)*};}
document_codec!(Root, Group, File);

impl<'data> Read<'data> for RawDocument<'data> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let origin = input.absolute_position();
        let count = input.remaining();
        Self::decode(input.bytes(count)?).map_err(|mut error| {
            error.offset = error.offset.saturating_add(origin);
            error
        })
    }
}
impl Write for RawDocument<'_> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.encode()?)
    }
}

impl<D: Dialect> Validate for File<'_, D> {
    fn validate(&self) -> Result<(), ReadError> {
        match self {
            Self::Root(root) => root.validate(),
            Self::Group(group) => group.validate(),
        }
    }
}
