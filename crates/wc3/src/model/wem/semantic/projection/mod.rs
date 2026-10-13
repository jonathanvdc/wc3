//! Export views of polygon domains with explicit routes back to editable source records.
use super::triangulation::triangulate;
use super::*;
mod read;
mod rebuild;
pub(super) use read::read_mesh;
pub(super) use rebuild::rebuild;
fn domain_index(domain: r::Domain, source: VertexSource) -> Result<usize, ReadError> {
    match domain {
        r::Domain::VERTEX => Ok(source.vertex as usize),
        r::Domain::HALFEDGE => Ok(source.corner as usize),
        r::Domain::FACE => Ok(source.face as usize),
        r::Domain::MESH => Ok(0),
        _ => Err(ReadError::unsupported(
            0,
            "edge attribute domain has no explicit edge identity table",
        )),
    }
}
fn scalar_sections(doc: &RawDocument<'_>, layer: &r::AttrLayer) -> Result<Vec<u32>, ReadError> {
    let bytes = rawbytes(doc, layer.data)?;
    match layer.type_ {
        r::AttrType::U32 => {
            if !bytes.len().is_multiple_of(4) {
                return Err(ReadError::invalid(0, "face section byte count"));
            }
            Ok(bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|v| u32::from_le_bytes(*v))
                .collect())
        }
        r::AttrType::U16 => {
            if !bytes.len().is_multiple_of(2) {
                return Err(ReadError::invalid(0, "face section byte count"));
            }
            Ok(bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|v| u32::from(u16::from_le_bytes(*v)))
                .collect())
        }
        _ => Err(ReadError::unsupported(0, "face section attribute type")),
    }
}
