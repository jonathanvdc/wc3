//! PHYS versions 0–6, including inline PFDC framing and polytope topology.
use super::super as m2;
use super::super::codec::{ArrayRef, Decoder, Encoder};
use super::super::records::ModelFlags;
use super::super::records::{PhysicsBodyKind, PhysicsKind};
use super::super::{ReadError, WriteError, ZeroVec};
use super::fixed::*;
use super::{encode_records, flat_records, write_chunk};
use crate::model::common::Ownership;
use m2::Write as _;
use std::borrow::Cow;
use std::collections::BTreeSet;
#[derive(Clone, Debug, PartialEq, Ownership)]
/// A convex polytope with native topology and borrowed coordinate arrays.
pub struct Polytope<'data> {
    /// Fixed metadata and preserved header padding; array counts are rebuilt on write.
    pub header: PolytopeHeader,
    /// Convex hull vertex positions in the polytope's local coordinates.
    pub vertices: ZeroVec<'data, [f32; 3]>,
    /// Face planes encoded as (normal X, normal Y, normal Z, distance).
    pub planes: ZeroVec<'data, [f32; 4]>,
    /// Entry half-edge index for each face; following `next_edge` walks its boundary.
    pub face_first_edge: ZeroVec<'data, u8>,
    /// Directed half-edges linking hull vertices, faces, twins, and face boundary cycles.
    pub edges: Vec<PhysicsEdge>,
}
#[derive(Clone, Debug, PartialEq, Ownership)]
/// A typed PHYS chunk, preserving its version-specific record spelling.
pub enum PhysicsChunk<'data> {
    /// PHYS header containing the physics layout version, from zero through six.
    Header(u16),
    /// PHYT object behavior: 0/1 worn ragdoll, 2 vegetation, 3 shared-world ragdoll, 4 private-world ragdoll.
    Kind(PhysicsKind),
    /// PLYT convex polytope geometry.
    Polytopes(Vec<Polytope<'data>>),
    /// PHAO host-skeleton allow-list controlling use of authored kinematic follow factors.
    AllowList {
        /// Header word whose low byte is retained by the client; no active meaning is established.
        header: u32,
        /// Name CRCs of host skeleton key bone 4 permitted to use the authored follow factors.
        keys: ZeroVec<'data, u32>,
    },
    /// BOXS oriented boxes.
    Boxes(Vec<PhysicsBox>),
    /// SPHS spheres.
    Spheres(Vec<PhysicsSphere>),
    /// CAPS capsules.
    Capsules(Vec<PhysicsCapsule>),
    /// SHAP shape references for versions zero and one.
    Shapes(Vec<PhysicsShape>),
    /// SHP2 shape references for versions two through six.
    Shapes2(Vec<PhysicsShape2>),
    /// BODY rigid bodies for versions zero and one.
    Bodies(Vec<PhysicsBody>),
    /// BDY2 rigid bodies for version two.
    Bodies2(Vec<PhysicsBody2>),
    /// BDY3 rigid bodies for version three.
    Bodies3(Vec<PhysicsBody3>),
    /// BDY4 rigid bodies for versions four through six.
    Bodies4(Vec<PhysicsBody4>),
    /// SHOJ shoulder joints for versions three through five.
    Shoulders(Vec<PhysicsShoulder>),
    /// SHJ2 shoulder joints for version six.
    Shoulders2(Vec<PhysicsShoulder2>),
    /// WELJ weld joints for versions zero and one.
    Welds(Vec<PhysicsWeld>),
    /// WLJ2 weld joints for versions two through five.
    Welds2(Vec<PhysicsWeld2>),
    /// WLJ3 weld joints for version six.
    Welds3(Vec<PhysicsWeld3>),
    /// REVJ revolute joints for versions three through five.
    Revolutes(Vec<PhysicsRevolute>),
    /// REV2 revolute joints for version six.
    Revolutes2(Vec<PhysicsRevolute2>),
    /// SPHJ spherical joints.
    Sphericals(Vec<PhysicsSpherical>),
    /// DSTJ distance joints.
    Distances(Vec<PhysicsDistance>),
    /// PRSJ legacy prismatic joints.
    Prismatics(Vec<PhysicsPrismatic>),
    /// PRS2 prismatic joints for version six.
    Prismatics2(Vec<PhysicsPrismatic2>),
    /// JOIN body references and typed joint indices.
    Joints(Vec<PhysicsJoint>),
    /// PHYV vegetation tuning records.
    Vegetation(Vec<PhysicsVegetation>),
    /// Unrecognized chunk payload retained without interpretation.
    Unknown {
        /// Logical tag bytes; PHYS writes tags in reverse order on disk.
        tag: [u8; 4],
        /// Borrowed or edited opaque payload bytes.
        data: Cow<'data, [u8]>,
    },
}
#[derive(Clone, Debug, PartialEq, Ownership)]
/// An ordered PHYS document or inline PFDC payload.
pub struct PhysicsFile<'data> {
    /// Chunks in file order, beginning with exactly one PHYS header.
    pub chunks: Vec<PhysicsChunk<'data>>,
    /// Preserved zero-filled inline alignment bytes.
    pub padding: Cow<'data, [u8]>,
}
impl<'data> PhysicsFile<'data> {
    /// Reads standalone PHYS framing, rejecting trailing bytes.
    pub fn read(data: &'data [u8]) -> Result<Self, ReadError> {
        Self::decode(data, false)
    }
    /// Reads PHYS framing with up to fifteen zero-filled inline alignment bytes.
    pub fn read_inline(data: &'data [u8]) -> Result<Self, ReadError> {
        Self::decode(data, true)
    }
    fn decode(data: &'data [u8], inline: bool) -> Result<Self, ReadError> {
        let mut d = Decoder::new(data, 0, ModelFlags::default());
        let mut chunks = Vec::new();
        let mut version = None;
        let mut padding = &data[data.len()..];
        while d.position() < data.len() {
            let remaining = &data[d.position()..];
            if inline && remaining.len() < 16 && remaining.iter().all(|b| *b == 0) {
                padding = remaining;
                break;
            }
            let mut tag: [u8; 4] = d
                .bytes(4)?
                .try_into()
                .map_err(|_| ReadError::invalid(d.position(), "physics chunk tag"))?;
            tag.reverse();
            let size = d.u32()? as usize;
            let bytes = d.bytes(size)?;
            if tag != *b"PHYS" {
                let selected = version
                    .ok_or_else(|| ReadError::invalid(0, "physics file requires PHYS header"))?;
                if !valid_tag(selected, tag) {
                    return Err(ReadError::invalid(
                        d.position() - size - 8,
                        "physics chunk unavailable in version",
                    ));
                }
            } else if version.is_some() {
                return Err(ReadError::invalid(
                    d.position() - size - 8,
                    "duplicate PHYS header",
                ));
            }
            let mut payload = Decoder::new(bytes, 0, ModelFlags::default());
            let chunk = match &tag {
                b"PHYS" => {
                    if bytes.len() != 2 {
                        return Err(ReadError::invalid(0, "PHYS header length"));
                    }
                    let selected = payload.u16()?;
                    if selected > 6 {
                        return Err(ReadError::invalid(0, "unsupported PHYS version"));
                    }
                    version = Some(selected);
                    PhysicsChunk::Header(selected)
                }
                b"PHYT" => {
                    if bytes.len() != 4 {
                        return Err(ReadError::invalid(0, "PHYT length"));
                    }
                    PhysicsChunk::Kind(PhysicsKind::from_raw(payload.u32()?))
                }
                b"PLYT" => PhysicsChunk::Polytopes(read_polytopes(bytes)?),
                b"PHAO" => {
                    if bytes.len() < 4 || !bytes.len().is_multiple_of(4) {
                        return Err(ReadError::invalid(0, "PHAO length"));
                    }
                    let header = payload.u32()?;
                    let keys = payload.array(ArrayRef {
                        count: ((bytes.len() - 4) / 4) as u32,
                        offset: 4,
                    })?;
                    PhysicsChunk::AllowList { header, keys }
                }
                b"BOXS" => PhysicsChunk::Boxes(flat_records(bytes, 0, ModelFlags::default())?),
                b"SPHS" => PhysicsChunk::Spheres(flat_records(bytes, 0, ModelFlags::default())?),
                b"CAPS" => PhysicsChunk::Capsules(flat_records(bytes, 0, ModelFlags::default())?),
                b"SHAP" => PhysicsChunk::Shapes(flat_records(bytes, 0, ModelFlags::default())?),
                b"SHP2" => PhysicsChunk::Shapes2(flat_records(bytes, 0, ModelFlags::default())?),
                b"BODY" => PhysicsChunk::Bodies(flat_records(bytes, 0, ModelFlags::default())?),
                b"BDY2" => PhysicsChunk::Bodies2(flat_records(bytes, 0, ModelFlags::default())?),
                b"BDY3" => PhysicsChunk::Bodies3(flat_records(bytes, 0, ModelFlags::default())?),
                b"BDY4" => PhysicsChunk::Bodies4(flat_records(bytes, 0, ModelFlags::default())?),
                b"SHOJ" => PhysicsChunk::Shoulders(flat_records(bytes, 0, ModelFlags::default())?),
                b"SHJ2" => PhysicsChunk::Shoulders2(flat_records(bytes, 0, ModelFlags::default())?),
                b"WELJ" => PhysicsChunk::Welds(flat_records(bytes, 0, ModelFlags::default())?),
                b"WLJ2" => PhysicsChunk::Welds2(flat_records(bytes, 0, ModelFlags::default())?),
                b"WLJ3" => PhysicsChunk::Welds3(flat_records(bytes, 0, ModelFlags::default())?),
                b"REVJ" => PhysicsChunk::Revolutes(flat_records(bytes, 0, ModelFlags::default())?),
                b"REV2" => PhysicsChunk::Revolutes2(flat_records(bytes, 0, ModelFlags::default())?),
                b"SPHJ" => PhysicsChunk::Sphericals(flat_records(bytes, 0, ModelFlags::default())?),
                b"DSTJ" => PhysicsChunk::Distances(flat_records(bytes, 0, ModelFlags::default())?),
                b"PRSJ" => PhysicsChunk::Prismatics(flat_records(bytes, 0, ModelFlags::default())?),
                b"PRS2" => {
                    PhysicsChunk::Prismatics2(flat_records(bytes, 0, ModelFlags::default())?)
                }
                b"JOIN" => PhysicsChunk::Joints(flat_records(bytes, 0, ModelFlags::default())?),
                b"PHYV" => PhysicsChunk::Vegetation(flat_records(bytes, 0, ModelFlags::default())?),
                _ => PhysicsChunk::Unknown {
                    tag,
                    data: Cow::Borrowed(bytes),
                },
            };
            chunks.push(chunk);
        }
        if !matches!(chunks.first(), Some(PhysicsChunk::Header(_))) {
            return Err(ReadError::invalid(0, "physics file requires PHYS header"));
        }
        Ok(Self {
            chunks,
            padding: Cow::Borrowed(padding),
        })
    }
    /// Checks understood local shape, body, joint, and polytope references.
    ///
    /// Unknown shape and joint kinds remain uninterpreted. This check is separate
    /// from decoding and writing, which preserve native records without requiring
    /// verified game semantics for every field.
    pub fn validate(&self) -> Result<(), ReadError> {
        self.validate_references(None)
    }

    /// Checks local references and animation-driven body bone indices.
    ///
    /// Supply the bone count of the effective owning skeleton, including an
    /// external skeleton when present. Static bodies do not require a bone.
    pub fn validate_with_bone_count(&self, bone_count: usize) -> Result<(), ReadError> {
        self.validate_references(Some(bone_count))
    }

    fn validate_references(&self, bone_count: Option<usize>) -> Result<(), ReadError> {
        let mut geometry = [0usize; 4];
        let mut joints = [0usize; 6];
        let mut shapes = Vec::new();
        let mut bodies = Vec::new();
        let mut bindings = Vec::new();
        for chunk in &self.chunks {
            match chunk {
                PhysicsChunk::Boxes(values) => geometry[0] += values.len(),
                PhysicsChunk::Capsules(values) => geometry[1] += values.len(),
                PhysicsChunk::Spheres(values) => geometry[2] += values.len(),
                PhysicsChunk::Polytopes(values) => {
                    geometry[3] += values.len();
                    for value in values {
                        value.validate()?;
                    }
                }
                PhysicsChunk::Shapes(values) => shapes.extend(values.iter()),
                PhysicsChunk::Shapes2(values) => {
                    shapes.extend(values.iter().map(|value| &value.shape))
                }
                PhysicsChunk::Bodies(values) => bodies.extend(values.iter().map(|value| {
                    (
                        value.body_type,
                        value.bone_index as u16,
                        value.shape_index,
                        value.shape_count,
                    )
                })),
                PhysicsChunk::Bodies2(values) => bodies.extend(values.iter().map(|value| {
                    (
                        value.body.body_type,
                        value.body.bone_index as u16,
                        value.body.shape_index,
                        value.body.shape_count,
                    )
                })),
                PhysicsChunk::Bodies3(values) => bodies.extend(values.iter().map(|value| {
                    (
                        value.body_type,
                        value.bone_index,
                        value.shape_index,
                        value.shape_count,
                    )
                })),
                PhysicsChunk::Bodies4(values) => bodies.extend(values.iter().map(|value| {
                    (
                        value.body_type,
                        value.bone_index,
                        value.shape_index,
                        value.shape_count,
                    )
                })),
                PhysicsChunk::Sphericals(values) => joints[0] += values.len(),
                PhysicsChunk::Shoulders(values) => joints[1] += values.len(),
                PhysicsChunk::Shoulders2(values) => joints[1] += values.len(),
                PhysicsChunk::Welds(values) => joints[2] += values.len(),
                PhysicsChunk::Welds2(values) => joints[2] += values.len(),
                PhysicsChunk::Welds3(values) => joints[2] += values.len(),
                PhysicsChunk::Revolutes(values) => joints[3] += values.len(),
                PhysicsChunk::Revolutes2(values) => joints[3] += values.len(),
                PhysicsChunk::Prismatics(values) => joints[4] += values.len(),
                PhysicsChunk::Prismatics2(values) => joints[4] += values.len(),
                PhysicsChunk::Distances(values) => joints[5] += values.len(),
                PhysicsChunk::Joints(values) => bindings.extend(values.iter()),
                _ => {}
            }
        }
        for shape in &shapes {
            if let Some(count) = geometry.get(usize::from(shape.shape_type.raw())) {
                if shape.shape_index < 0 || shape.shape_index as usize >= *count {
                    return Err(ReadError::invalid(0, "physics shape geometry index"));
                }
            }
        }
        for &(kind, bone, first, count) in &bodies {
            if count < 0 {
                return Err(ReadError::invalid(0, "negative physics body shape count"));
            }
            // A shapeless body does not dereference its shape base.
            if count != 0
                && (first < 0
                    || (first as usize)
                        .checked_add(count as usize)
                        .is_none_or(|end| end > shapes.len()))
            {
                return Err(ReadError::invalid(0, "physics body shape range"));
            }
            if matches!(kind, PhysicsBodyKind::Kinematic | PhysicsBodyKind::Dynamic)
                && bone_count.is_some_and(|count| bone as usize >= count)
            {
                return Err(ReadError::invalid(0, "physics body bone index"));
            }
        }
        for binding in bindings {
            if binding.body_a as usize >= bodies.len() || binding.body_b as usize >= bodies.len() {
                return Err(ReadError::invalid(0, "physics joint body index"));
            }
            if let Some(count) = joints.get(usize::from(binding.joint_type.raw())) {
                if binding.joint_id < 0 || binding.joint_id as usize >= *count {
                    return Err(ReadError::invalid(0, "physics joint constraint index"));
                }
            }
        }
        Ok(())
    }

    /// Encodes typed chunks and preserves any recorded inline padding.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        if !matches!(self.chunks.first(), Some(PhysicsChunk::Header(_))) {
            return Err(WriteError::invalid("physics file requires PHYS header"));
        }
        let version = match self.chunks.first() {
            Some(PhysicsChunk::Header(version)) => *version,
            _ => unreachable!(),
        };
        let mut e = Encoder::new(0, ModelFlags::default());
        for (index, chunk) in self.chunks.iter().enumerate() {
            if index > 0 && matches!(chunk, PhysicsChunk::Header(_)) {
                return Err(WriteError::invalid("duplicate PHYS header"));
            }
            if matches!(chunk, PhysicsChunk::Unknown { tag, .. } if known_tag(*tag)) {
                return Err(WriteError::invalid("known physics tag in opaque chunk"));
            }
            let (mut tag, data) = match chunk {
                PhysicsChunk::Header(version) => {
                    if *version > 6 {
                        return Err(WriteError::invalid("unsupported PHYS version"));
                    }
                    (*b"PHYS", version.to_le_bytes().to_vec())
                }
                PhysicsChunk::Kind(kind) => (*b"PHYT", kind.raw().to_le_bytes().to_vec()),
                PhysicsChunk::Polytopes(values) => (*b"PLYT", write_polytopes(values)?),
                PhysicsChunk::AllowList { header, keys } => {
                    let mut data = header.to_le_bytes().to_vec();
                    data.extend_from_slice(keys.as_bytes());
                    (*b"PHAO", data)
                }
                PhysicsChunk::Boxes(values) => {
                    (*b"BOXS", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Spheres(values) => {
                    (*b"SPHS", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Capsules(values) => {
                    (*b"CAPS", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Shapes(values) => {
                    (*b"SHAP", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Shapes2(values) => {
                    (*b"SHP2", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Bodies(values) => {
                    (*b"BODY", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Bodies2(values) => {
                    (*b"BDY2", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Bodies3(values) => {
                    (*b"BDY3", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Bodies4(values) => {
                    (*b"BDY4", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Shoulders(values) => {
                    (*b"SHOJ", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Shoulders2(values) => {
                    (*b"SHJ2", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Welds(values) => {
                    (*b"WELJ", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Welds2(values) => {
                    (*b"WLJ2", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Welds3(values) => {
                    (*b"WLJ3", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Revolutes(values) => {
                    (*b"REVJ", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Revolutes2(values) => {
                    (*b"REV2", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Sphericals(values) => {
                    (*b"SPHJ", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Distances(values) => {
                    (*b"DSTJ", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Prismatics(values) => {
                    (*b"PRSJ", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Prismatics2(values) => {
                    (*b"PRS2", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Joints(values) => {
                    (*b"JOIN", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Vegetation(values) => {
                    (*b"PHYV", encode_records(values, 0, ModelFlags::default())?)
                }
                PhysicsChunk::Unknown { tag, data } => (*tag, data.to_vec()),
            };
            if !valid_tag(version, tag) {
                return Err(WriteError::invalid("physics chunk unavailable in version"));
            }
            tag.reverse();
            write_chunk(&mut e, tag, &data)?;
        }
        e.bytes(&self.padding)?;
        Ok(e.finish())
    }
    /// Encodes a PFDC payload and regenerates its sixteen-byte alignment.
    pub fn write_inline(&self) -> Result<Vec<u8>, WriteError> {
        let mut bytes = self.write()?;
        // Inline padding is a container property and must be regenerated after edits.
        bytes.truncate(bytes.len() - self.padding.len());
        let length = bytes
            .len()
            .checked_add(15)
            .ok_or_else(|| WriteError::invalid("physics length overflow"))?
            & !15;
        bytes.resize(length, 0);
        Ok(bytes)
    }
}
fn read_polytopes<'data>(data: &'data [u8]) -> Result<Vec<Polytope<'data>>, ReadError> {
    let mut d = Decoder::new(data, 0, ModelFlags::default());
    let count = d.u32()? as usize;
    let headers = d.records::<PolytopeHeader>(ArrayRef {
        count: u32::try_from(count).map_err(|_| ReadError::invalid(0, "polytope count"))?,
        offset: 4,
    })?;
    let start = 4usize
        .checked_add(
            count
                .checked_mul(80)
                .ok_or_else(|| ReadError::invalid(0, "polytope header overflow"))?,
        )
        .ok_or_else(|| ReadError::invalid(0, "polytope header overflow"))?;
    let mut cursor = start;
    let mut values = Vec::new();
    for header in headers {
        let vertices = read_array(&d, &mut cursor, header.vertex_count, 12)?;
        let planes = read_array(&d, &mut cursor, header.face_count, 16)?;
        let face_first_edge = read_array(&d, &mut cursor, header.face_count, 1)?;
        let edges = d.records(ArrayRef {
            count: header.edge_count,
            offset: u32::try_from(cursor)
                .map_err(|_| ReadError::invalid(cursor, "polytope offset"))?,
        })?;
        cursor = cursor
            .checked_add(
                (header.edge_count as usize)
                    .checked_mul(4)
                    .ok_or_else(|| ReadError::invalid(cursor, "polytope edge length"))?,
            )
            .ok_or_else(|| ReadError::invalid(cursor, "polytope edge length"))?;
        values.push(Polytope {
            header,
            vertices,
            planes,
            face_first_edge,
            edges,
        });
    }
    if cursor != data.len() {
        return Err(ReadError::invalid(cursor, "polytope trailing data"));
    }
    Ok(values)
}
fn read_array<'data, T: super::super::codec::Value>(
    d: &Decoder<'data>,
    cursor: &mut usize,
    count: u32,
    stride: usize,
) -> Result<ZeroVec<'data, T>, ReadError> {
    let a = ArrayRef {
        count,
        offset: u32::try_from(*cursor)
            .map_err(|_| ReadError::invalid(*cursor, "polytope offset"))?,
    };
    let value = d.array(a)?;
    *cursor = cursor
        .checked_add(
            (count as usize)
                .checked_mul(stride)
                .ok_or_else(|| ReadError::invalid(*cursor, "polytope array length"))?,
        )
        .ok_or_else(|| ReadError::invalid(*cursor, "polytope array length"))?;
    Ok(value)
}
fn write_polytopes(values: &[Polytope<'_>]) -> Result<Vec<u8>, WriteError> {
    let mut e = Encoder::new(0, ModelFlags::default());
    e.u32(
        u32::try_from(values.len())
            .map_err(|_| WriteError::invalid("polytope count exceeds u32"))?,
    )?;
    for value in values {
        if value.planes.len() != value.face_first_edge.len() {
            return Err(WriteError::invalid("polytope face count mismatch"));
        }
        let mut header = value.header.clone();
        header.vertex_count = u32::try_from(value.vertices.len())
            .map_err(|_| WriteError::invalid("polytope vertices exceed u32"))?;
        header.face_count = u32::try_from(value.planes.len())
            .map_err(|_| WriteError::invalid("polytope faces exceed u32"))?;
        header.edge_count = u32::try_from(value.edges.len())
            .map_err(|_| WriteError::invalid("polytope edges exceed u32"))?;
        header.write(&mut e)?;
    }
    for value in values {
        e.bytes(value.vertices.as_bytes())?;
        e.bytes(value.planes.as_bytes())?;
        e.bytes(value.face_first_edge.as_bytes())?;
        for edge in &value.edges {
            edge.write(&mut e)?;
        }
    }
    Ok(e.finish())
}

// Version-specific spellings are wire layouts, not interchangeable aliases.
// SHOJ/REVJ payloads are accepted in both version 3 and version 4.
fn valid_tag(version: u16, tag: [u8; 4]) -> bool {
    match &tag {
        b"PHYT" => version >= 1,
        b"BODY" | b"SHAP" | b"WELJ" => version <= 1,
        b"BDY2" => version == 2,
        b"BDY3" => version == 3,
        b"BDY4" => version >= 4,
        b"SHP2" => version >= 2,
        b"WLJ2" => (2..=5).contains(&version),
        b"PLYT" => version >= 3,
        b"SHOJ" | b"REVJ" => (3..=5).contains(&version),
        b"PRSJ" => version <= 5,
        b"SHJ2" | b"WLJ3" | b"REV2" | b"PRS2" => version == 6,
        _ => true,
    }
}

fn known_tag(tag: [u8; 4]) -> bool {
    matches!(
        &tag,
        b"PHYS"
            | b"PHYT"
            | b"PLYT"
            | b"PHAO"
            | b"BOXS"
            | b"SPHS"
            | b"CAPS"
            | b"SHAP"
            | b"SHP2"
            | b"BODY"
            | b"BDY2"
            | b"BDY3"
            | b"BDY4"
            | b"SHOJ"
            | b"SHJ2"
            | b"WELJ"
            | b"WLJ2"
            | b"WLJ3"
            | b"REVJ"
            | b"REV2"
            | b"SPHJ"
            | b"DSTJ"
            | b"PRSJ"
            | b"PRS2"
            | b"JOIN"
            | b"PHYV"
    )
}

impl Polytope<'_> {
    /// Checks the native half-edge topology against the edited geometry arrays.
    pub fn validate(&self) -> Result<(), ReadError> {
        if self.planes.len() != self.face_first_edge.len() {
            return Err(ReadError::invalid(0, "polytope face array count"));
        }
        for (index, edge) in self.edges.iter().enumerate() {
            if edge.origin_vertex as usize >= self.vertices.len()
                || edge.face_index as usize >= self.planes.len()
                || edge.next_edge as usize >= self.edges.len()
            {
                return Err(ReadError::invalid(0, "polytope edge index"));
            }
            let twin = (index as isize)
                .checked_add(edge.twin_offset as isize)
                .and_then(|value| usize::try_from(value).ok())
                .filter(|value| *value < self.edges.len())
                .ok_or_else(|| ReadError::invalid(0, "polytope twin index"))?;
            if (twin as isize) + self.edges[twin].twin_offset as isize != index as isize {
                return Err(ReadError::invalid(0, "polytope reciprocal twin"));
            }
        }
        for (face, start) in self.face_first_edge.iter().enumerate() {
            let start = start as usize;
            let mut edge = start;
            let mut seen = BTreeSet::new();
            loop {
                let value = self
                    .edges
                    .get(edge)
                    .ok_or_else(|| ReadError::invalid(0, "polytope face edge index"))?;
                if value.face_index as usize != face {
                    return Err(ReadError::invalid(0, "polytope face edge ownership"));
                }
                if !seen.insert(edge) {
                    return Err(ReadError::invalid(0, "polytope face edge cycle"));
                }
                edge = value.next_edge as usize;
                if edge == start {
                    break;
                }
            }
        }
        Ok(())
    }
}

impl<'data> m2::Read<'data> for PhysicsFile<'data> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let origin = input.absolute_position();
        let remaining = input.data().len() - input.position();
        Self::read(input.bytes(remaining)?).map_err(|mut error| {
            error.offset = origin.saturating_add(error.offset);
            error
        })
    }
}
impl m2::Write for PhysicsFile<'_> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.write()?)
    }
}
