//! Alpha root records and root envelopes for versions14 and16.
use super::Group14;
use crate::model::{common::Ownership, wmo};
use std::borrow::Cow;
use std::collections::BTreeSet;
use wmo::codec::{Array, Decoder, Encoder};
use wmo::document::{frame, frames};
use wmo::records::Header;
use wmo::resources::{string_at, Dependency, DependencyKind};
use wmo::root::RootChunk;
use wmo::validation::validate_tables;
use wmo::{Classic, FileKey, RawChunk, Read, ReadError, RecordLayout, Validate, Write, WriteError};
/// Alpha MOHD header; the later bounds and flag area is reserved in this layout.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct Header14 {
    /// Number of referenced textures.
    pub texture_count: u32,
    /// Number of embedded groups.
    pub group_count: u32,
    /// Number of portal definitions.
    pub portal_count: u32,
    /// Stored light count.
    pub light_count: u32,
    /// Stored doodad-name count.
    pub doodad_name_count: u32,
    /// Number of placed doodads.
    pub doodad_definition_count: u32,
    /// Number of doodad sets.
    pub doodad_set_count: u32,
    /// Ambient color in native BGRA order.
    pub ambient_color: [u8; 4],
    /// WMOAreaTable identity.
    pub wmo_id: u32,
    /// Reserved bytes where later clients store bounds and flags.
    pub reserved: [u8; 28],
}
/// Forty-four-byte alpha material, before the shader and third texture fields.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct Material14 {
    /// Native per-material version word.
    pub version: u32,
    /// Render-state flags.
    pub flags: u32,
    /// Native blend-mode code.
    pub blend_mode: u32,
    /// Byte offset into MOTX of the diffuse texture.
    pub texture0: u32,
    /// Native self-illumination color in BGRA order.
    pub sidn_color: [u8; 4],
    /// Preserved runtime self-illumination slot.
    pub frame_sidn_color: [u8; 4],
    /// Byte offset into MOTX of the environment texture.
    pub texture1: u32,
    /// Native diffuse color in BGRA order.
    pub diffuse_color: [u8; 4],
    /// TerrainType reference.
    pub ground_type: u32,
    /// Uninterpreted trailing bytes.
    pub reserved: [u8; 8],
}
/// Alpha MOGI entry addressing an embedded group in the complete file.
#[derive(
    Clone, Debug, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct GroupInfo14 {
    /// Absolute byte position of the embedded MOGP chunk header.
    pub group_offset: u32,
    /// Complete MOGP frame length, including its eight-byte chunk header.
    pub group_size: u32,
    /// Group flags corresponding to the MOGP header.
    pub flags: u32,
    /// Group bounds in WMO coordinates.
    pub bounds: [[f32; 3]; 2],
    /// MOGN name offset, or -1 when absent.
    pub name_offset: i32,
}
impl Default for GroupInfo14 {
    fn default() -> Self {
        Self {
            group_offset: 0,
            group_size: 0,
            flags: 0,
            bounds: [[0.0; 3]; 2],
            name_offset: -1,
        }
    }
}
/// Alpha MOLT light without the later orientation quaternion.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct Light14 {
    /// Native light kind.
    pub kind: u8,
    /// Whether attenuation is enabled.
    pub use_attenuation: u8,
    /// Uninterpreted bytes.
    pub reserved: [u8; 2],
    /// Light color in native BGRA order.
    pub color: [u8; 4],
    /// Position in WMO coordinates.
    pub position: [f32; 3],
    /// Authored intensity.
    pub intensity: f32,
    /// Attenuation start distance.
    pub attenuation_start: f32,
    /// Attenuation end distance.
    pub attenuation_end: f32,
}
/// Native alpha root chunks whose record widths differ from version17.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum RootChunk14<'data> {
    /// MOMT alpha materials.
    Materials(Array<'data, Material14>),
    /// MOGI absolute embedded-group locations.
    Groups(Array<'data, GroupInfo14>),
    /// MOLT alpha lights.
    Lights(Array<'data, Light14>),
    /// Shared-layout known chunk or byte-preserving unknown extension.
    Common(RootChunk<'data, Classic>),
}
impl<'data> RootChunk14<'data> {
    /// Native chunk identifier.
    pub fn tag(&self) -> [u8; 4] {
        match self {
            Self::Materials(_) => *b"MOMT",
            Self::Groups(_) => *b"MOGI",
            Self::Lights(_) => *b"MOLT",
            Self::Common(c) => c.tag(),
        }
    }
    /// Parses a bounded alpha root chunk.
    pub fn decode(tag: [u8; 4], data: &'data [u8], origin: usize) -> Result<Self, ReadError> {
        let parse =
            |error: ReadError| ReadError::invalid(origin.saturating_add(error.offset), error.field);
        Ok(match &tag {
            b"MOMT" => Self::Materials(Array::decode(data).map_err(parse)?),
            b"MOGI" => Self::Groups(Array::decode(data).map_err(parse)?),
            b"MOLT" => Self::Lights(Array::decode(data).map_err(parse)?),
            b"MOHD" | b"MVER" | b"MOMO" | b"MOGP" => {
                return Err(ReadError::invalid(
                    origin,
                    "root chunk requires document framing",
                ));
            }
            _ => Self::Common(RootChunk::decode(tag, data, origin)?),
        })
    }
    /// Encodes the unframed chunk payload.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        Ok(match self {
            Self::Materials(a) => a.as_bytes().to_vec(),
            Self::Groups(a) => a.as_bytes().to_vec(),
            Self::Lights(a) => a.as_bytes().to_vec(),
            Self::Common(c) => {
                if matches!(
                    &c.tag(),
                    b"MOHD" | b"MVER" | b"MOMO" | b"MOGP" | b"MOMT" | b"MOGI" | b"MOLT"
                ) {
                    return Err(WriteError::invalid(
                        "alpha-specific chunk requires native storage",
                    ));
                }
                c.encode()?
            }
        })
    }
}
#[derive(Clone, Debug, PartialEq, Ownership)]
enum Data14<'data> {
    Header,
    Chunk(RootChunk14<'data>),
}
#[derive(Clone, Debug, PartialEq, Ownership)]
enum Entry14<'data> {
    Version,
    Momo(Vec<Data14<'data>>),
    Group(Group14<'data>),
    Unknown(RawChunk<'data>),
}
/// Alpha root data and embedded group graph in a single version14 file.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct Root14<'data> {
    header: Header14,
    entries: Vec<Entry14<'data>>,
    group_order: Vec<usize>,
}
impl Default for Root14<'_> {
    fn default() -> Self {
        Self::new()
    }
}
impl<'data> Root14<'data> {
    /// Creates an empty monolithic root with protected required headers.
    pub fn new() -> Self {
        Self {
            header: Header14::default(),
            group_order: Vec::new(),
            entries: vec![Entry14::Version, Entry14::Momo(vec![Data14::Header])],
        }
    }
    /// Borrows the native alpha header.
    pub fn header(&self) -> &Header14 {
        &self.header
    }
    /// Edits native alpha header fields.
    pub fn header_mut(&mut self) -> &mut Header14 {
        &mut self.header
    }
    /// Iterates root payloads in their physical order inside MOMO.
    pub fn chunks(&self) -> impl Iterator<Item = &RootChunk14<'data>> {
        self.entries
            .iter()
            .flat_map(|entry| match entry {
                Entry14::Momo(data) => data.as_slice(),
                _ => &[],
            })
            .filter_map(|e| match e {
                Data14::Chunk(c) => Some(c),
                _ => None,
            })
    }
    /// Iterates uninterpreted top-level payloads outside the MOMO root body.
    pub fn unknown_chunks(&self) -> impl Iterator<Item = &RawChunk<'data>> {
        self.entries.iter().filter_map(|entry| match entry {
            Entry14::Unknown(chunk) => Some(chunk),
            _ => None,
        })
    }
    /// Iterates mutable root payloads without exposing required-header removal.
    pub fn chunks_mut(&mut self) -> impl Iterator<Item = &mut RootChunk14<'data>> {
        self.entries
            .iter_mut()
            .flat_map(|entry| match entry {
                Entry14::Momo(data) => data.as_mut_slice(),
                _ => &mut [],
            })
            .filter_map(|e| match e {
                Data14::Chunk(c) => Some(c),
                _ => None,
            })
    }
    /// Iterates embedded groups in physical file order.
    pub fn groups(&self) -> impl Iterator<Item = &Group14<'data>> {
        self.entries.iter().filter_map(|entry| match entry {
            Entry14::Group(g) => Some(g),
            _ => None,
        })
    }
    /// Iterates editable embedded groups.
    pub fn groups_mut(&mut self) -> impl Iterator<Item = &mut Group14<'data>> {
        self.entries.iter_mut().filter_map(|entry| match entry {
            Entry14::Group(g) => Some(g),
            _ => None,
        })
    }
    /// Borrows an embedded group by its logical MOGI index.
    pub fn group(&self, index: usize) -> Option<&Group14<'data>> {
        self.groups().nth(*self.group_order.get(index)?)
    }
    /// Edits an embedded group by its logical MOGI index.
    pub fn group_mut(&mut self, index: usize) -> Option<&mut Group14<'data>> {
        let physical = *self.group_order.get(index)?;
        self.groups_mut().nth(physical)
    }
    /// Number of embedded group documents.
    pub fn group_count(&self) -> usize {
        self.groups().count()
    }
    /// Appends a root extension while retaining mandatory header framing.
    pub fn push_chunk(&mut self, chunk: RootChunk14<'data>) -> Result<(), WriteError> {
        if matches!(
            chunk.tag(),
            [b'M', b'O', b'H', b'D']
                | [b'M', b'V', b'E', b'R']
                | [b'M', b'O', b'M', b'O']
                | [b'M', b'O', b'G', b'P']
        ) {
            return Err(WriteError::invalid("protected root framing"));
        }
        let data = self
            .entries
            .iter_mut()
            .find_map(|entry| match entry {
                Entry14::Momo(data) => Some(data),
                _ => None,
            })
            .unwrap();
        data.push(Data14::Chunk(chunk));
        Ok(())
    }
    /// Adds an embedded group and its matching native MOGI entry.
    pub fn push_group(
        &mut self,
        group: Group14<'data>,
        info: GroupInfo14,
    ) -> Result<(), WriteError> {
        let mut records = match self.chunks().find_map(|c| match c {
            RootChunk14::Groups(a) => Some(a),
            _ => None,
        }) {
            Some(array) => array
                .iter()
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| WriteError::invalid("group-info records"))?,
            None => Vec::new(),
        };
        records.push(info);
        let mut array = Some(Array::from_records(&records)?);
        for found in self.chunks_mut() {
            if matches!(found, RootChunk14::Groups(_)) {
                *found = RootChunk14::Groups(array.take().unwrap());
                break;
            }
        }
        if let Some(array) = array {
            self.push_chunk(RootChunk14::Groups(array))?;
        }
        self.group_order.push(self.group_count());
        self.entries.push(Entry14::Group(group));
        self.header.group_count =
            u32::try_from(records.len()).map_err(|_| WriteError::invalid("group count"))?;
        Ok(())
    }
    /// Decodes MOMO root framing and all embedded groups with checked bounds.
    pub fn decode(data: &'data [u8]) -> Result<Self, ReadError> {
        let mut header = None;
        let mut version = false;
        let mut momo = false;
        let mut entries = Vec::new();
        let mut positions = Vec::new();
        for (tag, payload, origin) in frames(data, 0)? {
            match &tag {
                b"MVER" => {
                    if version {
                        return Err(ReadError::invalid(origin, "duplicate MVER"));
                    }
                    check_version(payload, origin, 14)?;
                    version = true;
                    entries.push(Entry14::Version);
                }
                b"MOMO" => {
                    if momo {
                        return Err(ReadError::invalid(origin, "duplicate MOMO"));
                    }
                    momo = true;
                    let mut children = Vec::new();
                    for (child, bytes, child_origin) in frames(payload, origin)? {
                        if child == *b"MOHD" {
                            if header.is_some() {
                                return Err(ReadError::invalid(child_origin, "duplicate MOHD"));
                            }
                            header = Some(single::<Header14>(bytes, child_origin)?);
                            children.push(Data14::Header);
                        } else {
                            children.push(Data14::Chunk(RootChunk14::decode(
                                child,
                                bytes,
                                child_origin,
                            )?));
                        }
                    }
                    entries.push(Entry14::Momo(children));
                }
                b"MOGP" => {
                    positions.push((origin - 8, payload.len() + 8));
                    entries.push(Entry14::Group(Group14::decode_payload(payload).map_err(
                        |mut e| {
                            e.offset = origin.saturating_add(e.offset);
                            e
                        },
                    )?));
                }
                b"MOHD" => {
                    return Err(ReadError::invalid(origin, "alpha root header outside MOMO"));
                }
                _ => entries.push(Entry14::Unknown(RawChunk {
                    tag,
                    data: Cow::Borrowed(payload),
                })),
            }
        }
        if !version || !momo {
            return Err(ReadError::invalid(0, "missing MVER/MOMO"));
        }
        let mut result = Self {
            group_order: Vec::new(),
            header: header.ok_or_else(|| ReadError::invalid(0, "missing MOHD"))?,
            entries,
        };
        let order = if let Some(info) = result.chunks().find_map(|c| match c {
            RootChunk14::Groups(a) => Some(a),
            _ => None,
        }) {
            if info.len() != positions.len() {
                return Err(ReadError::invalid(0, "embedded group count"));
            }
            let mut order = Vec::new();
            for record in info.iter() {
                let record = record?;
                let physical = positions
                    .iter()
                    .position(|(offset, size)| {
                        record.group_offset as usize == *offset
                            && record.group_size as usize == *size
                    })
                    .ok_or_else(|| {
                        ReadError::invalid(
                            record.group_offset as usize,
                            "embedded MOGI address/size",
                        )
                    })?;
                if order.contains(&physical) {
                    return Err(ReadError::invalid(
                        record.group_offset as usize,
                        "duplicate embedded group reference",
                    ));
                }
                order.push(physical);
            }
            order
        } else if !positions.is_empty() {
            return Err(ReadError::invalid(0, "embedded groups without MOGI"));
        } else {
            Vec::new()
        };
        result.group_order = order;
        Ok(result)
    }
    fn encoded_header(&self) -> Result<Header14, WriteError> {
        let mut header = self.header.clone();
        header.group_count =
            u32::try_from(self.group_count()).map_err(|_| WriteError::invalid("group count"))?;
        for chunk in self.chunks() {
            if let RootChunk14::Common(chunk) = chunk {
                match chunk {
                    RootChunk::Portals(a) => {
                        header.portal_count = u32::try_from(a.len())
                            .map_err(|_| WriteError::invalid("portal count"))?
                    }
                    RootChunk::DoodadSets(a) => {
                        header.doodad_set_count = u32::try_from(a.len())
                            .map_err(|_| WriteError::invalid("doodad set count"))?
                    }
                    _ => {}
                }
            }
        }
        Ok(header)
    }
    /// Updates counts owned by embedded groups, portals, and doodad-set tables.
    pub fn synchronize_counts(&mut self) -> Result<(), WriteError> {
        self.header = self.encoded_header()?;
        Ok(())
    }
    /// Encodes the monolithic graph, relocating MOGI absolute addresses on edits.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        let groups = self
            .groups()
            .map(Group14::encode_payload)
            .collect::<Result<Vec<_>, _>>()?;
        let mut lengths = Vec::new();
        let mut offset = 0usize;
        let mut group = 0usize;
        for entry in &self.entries {
            let length = match entry {
                Entry14::Version => 12,
                Entry14::Momo(children) => self.momo_payload(children, None)?.len() + 8,
                Entry14::Group(_) => {
                    let length = groups[group].len() + 8;
                    lengths.push((
                        u32::try_from(offset).map_err(|_| WriteError::invalid("group offset"))?,
                        u32::try_from(length).map_err(|_| WriteError::invalid("group size"))?,
                    ));
                    group += 1;
                    length
                }
                Entry14::Unknown(c) => c.data.len() + 8,
            };
            offset = offset
                .checked_add(length)
                .ok_or_else(|| WriteError::invalid("file length"))?;
        }
        let mut output = Encoder::new();
        let mut group = 0;
        for entry in &self.entries {
            match entry {
                Entry14::Version => frame(&mut output, *b"MVER", &14u32.to_le_bytes())?,
                Entry14::Momo(children) => frame(
                    &mut output,
                    *b"MOMO",
                    &self.momo_payload(children, Some(&lengths))?,
                )?,
                Entry14::Group(_) => {
                    frame(&mut output, *b"MOGP", &groups[group])?;
                    group += 1;
                }
                Entry14::Unknown(c) => frame(&mut output, c.tag, &c.data)?,
            }
        }
        Ok(output.finish())
    }
    fn momo_payload(
        &self,
        children: &[Data14<'data>],
        positions: Option<&[(u32, u32)]>,
    ) -> Result<Vec<u8>, WriteError> {
        let mut output = Encoder::new();
        for child in children {
            match child {
                Data14::Header => {
                    let header = self.encoded_header()?;
                    let mut payload = Encoder::new();
                    header.write(&mut payload)?;
                    frame(&mut output, *b"MOHD", &payload.finish())?;
                }
                Data14::Chunk(RootChunk14::Groups(records)) => {
                    let mut values = records
                        .iter()
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|_| WriteError::invalid("group-info records"))?;
                    if values.len() != self.group_count() {
                        return Err(WriteError::invalid("embedded group count"));
                    }
                    if let Some(positions) = positions {
                        for (index, value) in values.iter_mut().enumerate() {
                            let physical = *self
                                .group_order
                                .get(index)
                                .ok_or_else(|| WriteError::invalid("group-info mapping"))?;
                            let (offset, size) = *positions
                                .get(physical)
                                .ok_or_else(|| WriteError::invalid("group-info mapping"))?;
                            value.group_offset = offset;
                            value.group_size = size;
                        }
                    }
                    frame(
                        &mut output,
                        *b"MOGI",
                        Array::from_records(&values)?.as_bytes(),
                    )?;
                }
                Data14::Chunk(chunk) => {
                    if matches!(&chunk.tag(), b"MOHD" | b"MVER" | b"MOMO" | b"MOGP") {
                        return Err(WriteError::invalid("duplicate protected header"));
                    }
                    frame(&mut output, chunk.tag(), &chunk.encode()?)?;
                }
            }
        }
        Ok(output.finish())
    }
    /// Checks local references in the complete embedded group graph.
    pub fn validate(&self) -> Result<(), ReadError> {
        let count = self.group_count();
        let mut seen = BTreeSet::new();
        for chunk in self.chunks() {
            let unknown = matches!(chunk, RootChunk14::Common(RootChunk::Unknown { .. }));
            if !unknown && !seen.insert(chunk.tag()) {
                return Err(ReadError::invalid(0, "duplicate alpha root table"));
            }
            if matches!(chunk, RootChunk14::Common(_))
                && matches!(
                    &chunk.tag(),
                    b"MOHD" | b"MVER" | b"MOMO" | b"MOGP" | b"MOMT" | b"MOGI" | b"MOLT"
                )
            {
                return Err(ReadError::invalid(
                    0,
                    "alpha-specific chunk requires native storage",
                ));
            }
        }
        if self.header.group_count as usize != count {
            return Err(ReadError::invalid(0, "root group count"));
        }
        let infos = self.chunks().find_map(|c| match c {
            RootChunk14::Groups(a) => Some(a),
            _ => None,
        });
        if infos.map_or(0, Array::len) != count {
            return Err(ReadError::invalid(0, "MOGI group count"));
        }
        for group in self.groups() {
            group.validate()?;
            self.validate_group_references(group)?;
        }
        let common = self
            .chunks()
            .filter_map(|c| match c {
                RootChunk14::Common(c) => Some(c),
                _ => None,
            })
            .collect::<Vec<_>>();
        let light_count = self
            .chunks()
            .find_map(|c| match c {
                RootChunk14::Lights(a) => Some(a.len()),
                _ => None,
            })
            .unwrap_or(0);
        validate_tables(
            &common,
            count,
            self.header.portal_count as usize,
            self.header.doodad_set_count as usize,
            Some(light_count),
        )?;
        for c in self.chunks() {
            match c {
                RootChunk14::Materials(a) => a.validate()?,
                RootChunk14::Groups(a) => a.validate()?,
                RootChunk14::Lights(a) => a.validate()?,
                RootChunk14::Common(_) => {}
            }
        }
        if let Some(infos) = infos {
            let names = common.iter().find_map(|c| match c {
                RootChunk::GroupNames(n) => Some(n.as_bytes()),
                _ => None,
            });
            for info in infos.iter() {
                let info = info?;
                if info.name_offset != -1 {
                    let offset = u32::try_from(info.name_offset)
                        .map_err(|_| ReadError::invalid(0, "group name offset"))?;
                    string_at(names.unwrap_or(&[]), offset)?;
                }
            }
        }
        self.dependencies(&FileKey::Root)?;
        Ok(())
    }
    /// Enumerates filename-addressed textures, doodads, and skyboxes; groups are embedded.
    pub fn dependencies(&self, _owner: &FileKey) -> Result<Vec<Dependency>, ReadError> {
        let common = self
            .chunks()
            .filter_map(|c| match c {
                RootChunk14::Common(c) => Some(c),
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut result = common_dependencies(&common)?;
        let names = common.iter().find_map(|c| match c {
            RootChunk::TextureNames(n) => Some(n.as_bytes()),
            _ => None,
        });
        if let Some(materials) = self.chunks().find_map(|c| match c {
            RootChunk14::Materials(a) => Some(a),
            _ => None,
        }) {
            let names = names.ok_or_else(|| ReadError::invalid(0, "missing MOTX"))?;
            for (index, material) in materials.iter().enumerate() {
                let material = material?;
                for (slot, offset) in [material.texture0, material.texture1]
                    .into_iter()
                    .enumerate()
                {
                    push_path(
                        &mut result,
                        DependencyKind::Texture {
                            material: index,
                            slot,
                        },
                        string_at(names, offset)?,
                    );
                }
            }
        }
        Ok(result)
    }
}
#[derive(Clone, Debug, PartialEq, Ownership)]
enum Entry16<'data> {
    Version,
    Header,
    Chunk(RootChunk<'data, Classic>),
}
/// Early-beta root with modern root record widths and external version16 groups.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct Root16<'data> {
    header: Header<Classic>,
    entries: Vec<Entry16<'data>>,
}
impl Default for Root16<'_> {
    fn default() -> Self {
        Self::new()
    }
}
impl<'data> Root16<'data> {
    /// Creates an empty native version16 root.
    pub fn new() -> Self {
        Self {
            header: Header::default(),
            entries: vec![Entry16::Version, Entry16::Header],
        }
    }
    /// Native root header.
    pub fn header(&self) -> &Header<Classic> {
        &self.header
    }
    /// Editable native root header.
    pub fn header_mut(&mut self) -> &mut Header<Classic> {
        &mut self.header
    }
    /// Known and unknown payloads in native order, excluding protected headers.
    pub fn chunks(&self) -> impl Iterator<Item = &RootChunk<'data, Classic>> {
        self.entries.iter().filter_map(|entry| match entry {
            Entry16::Chunk(c) => Some(c),
            _ => None,
        })
    }
    /// Mutable extension payloads without required-header removal.
    pub fn chunks_mut(&mut self) -> impl Iterator<Item = &mut RootChunk<'data, Classic>> {
        self.entries.iter_mut().filter_map(|entry| match entry {
            Entry16::Chunk(c) => Some(c),
            _ => None,
        })
    }
    /// Appends a root payload while protecting required framing.
    pub fn push_chunk(&mut self, chunk: RootChunk<'data, Classic>) -> Result<(), WriteError> {
        if matches!(
            chunk.tag(),
            [b'M', b'O', b'H', b'D']
                | [b'M', b'V', b'E', b'R']
                | [b'M', b'O', b'G', b'P']
                | [b'M', b'O', b'M', b'O']
        ) {
            return Err(WriteError::invalid("protected root framing"));
        }
        self.entries.push(Entry16::Chunk(chunk));
        Ok(())
    }
    /// Number of external base groups declared by the group-info table.
    pub fn group_count(&self) -> usize {
        self.chunks()
            .find_map(|c| match c {
                RootChunk::Groups(a) => Some(a.len()),
                _ => None,
            })
            .unwrap_or(self.header.group_count as usize)
    }
    /// Reads version16 root framing without treating groups as version17 records.
    pub fn decode(data: &'data [u8]) -> Result<Self, ReadError> {
        let mut version = false;
        let mut header = None;
        let mut entries = Vec::new();
        for (tag, payload, origin) in frames(data, 0)? {
            match &tag {
                b"MVER" => {
                    if version {
                        return Err(ReadError::invalid(origin, "duplicate MVER"));
                    }
                    check_version(payload, origin, 16)?;
                    version = true;
                    entries.push(Entry16::Version);
                }
                b"MOHD" => {
                    if header.is_some() {
                        return Err(ReadError::invalid(origin, "duplicate MOHD"));
                    }
                    header = Some(single(payload, origin)?);
                    entries.push(Entry16::Header);
                }
                b"MOMO" | b"MOGP" => {
                    return Err(ReadError::invalid(origin, "root/group framing mismatch"));
                }
                _ => entries.push(Entry16::Chunk(RootChunk::decode(tag, payload, origin)?)),
            }
        }
        if !version {
            return Err(ReadError::invalid(0, "missing MVER"));
        }
        Ok(Self {
            header: header.ok_or_else(|| ReadError::invalid(0, "missing MOHD"))?,
            entries,
        })
    }
    fn encoded_header(&self) -> Result<Header<Classic>, WriteError> {
        let mut header = self.header.clone();
        for chunk in self.chunks() {
            match chunk {
                RootChunk::Groups(a) => {
                    header.group_count =
                        u32::try_from(a.len()).map_err(|_| WriteError::invalid("group count"))?
                }
                RootChunk::Portals(a) => {
                    header.portal_count =
                        u32::try_from(a.len()).map_err(|_| WriteError::invalid("portal count"))?
                }
                RootChunk::DoodadSets(a) => {
                    header.doodad_set_count = u32::try_from(a.len())
                        .map_err(|_| WriteError::invalid("doodad set count"))?
                }
                _ => {}
            }
        }
        Ok(header)
    }
    /// Updates counts owned by group-info, portal, and doodad-set tables.
    pub fn synchronize_counts(&mut self) -> Result<(), WriteError> {
        self.header = self.encoded_header()?;
        Ok(())
    }
    /// Encodes version16 headers and preserved root payload order.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        let mut output = Encoder::new();
        for entry in &self.entries {
            match entry {
                Entry16::Version => frame(&mut output, *b"MVER", &16u32.to_le_bytes())?,
                Entry16::Header => {
                    let mut payload = Encoder::new();
                    self.encoded_header()?.write(&mut payload)?;
                    frame(&mut output, *b"MOHD", &payload.finish())?;
                }
                Entry16::Chunk(c) => {
                    if matches!(&c.tag(), b"MOHD" | b"MVER" | b"MOMO" | b"MOGP") {
                        return Err(WriteError::invalid("duplicate protected header"));
                    }
                    frame(&mut output, c.tag(), &c.encode()?)?;
                }
            }
        }
        Ok(output.finish())
    }
    /// Validates local root references without loading external group files.
    pub fn validate(&self) -> Result<(), ReadError> {
        if self.chunks().any(|c| c.tag() == *b"MOMO") {
            return Err(ReadError::invalid(0, "alpha framing in version16 root"));
        }
        validate_tables(
            &self.chunks().collect::<Vec<_>>(),
            self.header.group_count as usize,
            self.header.portal_count as usize,
            self.header.doodad_set_count as usize,
            None,
        )?;
        if self.header.group_count != 0 && !self.chunks().any(|c| matches!(c, RootChunk::Groups(_)))
        {
            return Err(ReadError::invalid(0, "missing MOGI"));
        }
        self.dependencies(&FileKey::Root)?;
        Ok(())
    }
    /// Enumerates external version16 groups and filename-addressed resources.
    pub fn dependencies(&self, owner: &FileKey) -> Result<Vec<Dependency>, ReadError> {
        let chunks = self.chunks().collect::<Vec<_>>();
        if self.header.group_count != 0 && !chunks.iter().any(|c| matches!(c, RootChunk::Groups(_)))
        {
            return Err(ReadError::invalid(0, "missing MOGI"));
        }
        let mut result = common_dependencies(&chunks)?;
        for index in 0..self.group_count() {
            result.push(Dependency {
                kind: DependencyKind::Group { index, lod: 0 },
                key: owner.sibling(format!("_{index:03}.wmo")),
            });
        }
        if let Some(materials) = chunks.iter().find_map(|c| match c {
            RootChunk::Materials(a) => Some(a),
            _ => None,
        }) {
            let names = chunks
                .iter()
                .find_map(|c| match c {
                    RootChunk::TextureNames(n) => Some(n.as_bytes()),
                    _ => None,
                })
                .ok_or_else(|| ReadError::invalid(0, "missing MOTX"))?;
            for (index, material) in materials.iter().enumerate() {
                let material = material?;
                for (slot, offset) in [material.texture0, material.texture1, material.texture2]
                    .into_iter()
                    .enumerate()
                {
                    push_path(
                        &mut result,
                        DependencyKind::Texture {
                            material: index,
                            slot,
                        },
                        string_at(names, offset)?,
                    );
                }
            }
        }
        Ok(result)
    }
}
fn check_version(bytes: &[u8], origin: usize, expected: u32) -> Result<(), ReadError> {
    if bytes.len() != 4 || u32::read(&mut Decoder::new(bytes).with_origin(origin))? != expected {
        return Err(ReadError::invalid(origin, "legacy MVER"));
    }
    Ok(())
}
fn single<'data, T: Read<'data> + RecordLayout>(
    bytes: &'data [u8],
    origin: usize,
) -> Result<T, ReadError> {
    if bytes.len() != T::size() {
        return Err(ReadError::invalid(origin, "legacy record width"));
    }
    T::read(&mut Decoder::new(bytes).with_origin(origin))
}
fn push_path(result: &mut Vec<Dependency>, kind: DependencyKind, bytes: &[u8]) {
    if bytes.is_empty() {
        return;
    }
    let key = match std::str::from_utf8(bytes) {
        Ok(path) => FileKey::Path(path.to_owned()),
        Err(_) => FileKey::RawPath(bytes.to_vec()),
    };
    result.push(Dependency { kind, key });
}
fn common_dependencies(chunks: &[&RootChunk<'_, Classic>]) -> Result<Vec<Dependency>, ReadError> {
    let mut result = Vec::new();
    let names = chunks.iter().find_map(|c| match c {
        RootChunk::DoodadNames(n) => Some(n.as_bytes()),
        _ => None,
    });
    if let Some(doodads) = chunks.iter().find_map(|c| match c {
        RootChunk::Doodads(a) => Some(a),
        _ => None,
    }) {
        let names = names.ok_or_else(|| ReadError::invalid(0, "missing MODN"))?;
        for (index, doodad) in doodads.iter().enumerate() {
            push_path(
                &mut result,
                DependencyKind::Doodad { index },
                string_at(names, doodad?.name_and_flags & 0xffffff)?,
            );
        }
    }
    for chunk in chunks {
        if let RootChunk::SkyboxName(n) = chunk {
            if !n.is_empty() {
                push_path(
                    &mut result,
                    DependencyKind::Skybox,
                    string_at(n.as_bytes(), 0)?,
                );
            }
        }
    }
    Ok(result)
}
impl<'data> Read<'data> for Root14<'data> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let origin = input.absolute_position();
        Self::decode(input.bytes(input.remaining())?).map_err(|mut e| {
            e.offset = origin.saturating_add(e.offset);
            e
        })
    }
}
impl Write for Root14<'_> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.encode()?)
    }
}
impl<'data> Read<'data> for Root16<'data> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let origin = input.absolute_position();
        Self::decode(input.bytes(input.remaining())?).map_err(|mut e| {
            e.offset = origin.saturating_add(e.offset);
            e
        })
    }
}
impl Write for Root16<'_> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.encode()?)
    }
}
