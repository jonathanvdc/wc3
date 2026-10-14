//! Explicit, checked interpretation of native vertex element declarations.
use crate::model::conversion::{Context, Error};
use crate::model::diablo::d4::records::{GeoChunkVertexBuffer, VertexElem};
use half::f16 as Half;
use std::collections::BTreeSet;
use zerovec::ZeroVec;

/// Logical meaning explicitly assigned to a native vertex element by the caller.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ElementSemantic {
    /// Model-space position, requiring three components.
    Position,
    /// Surface normal, requiring three components.
    Normal,
    /// Tangent with authored handedness, requiring four components.
    Tangent,
    /// Texture coordinates in an independently numbered set.
    TexCoord(u32),
    /// Primary RGBA vertex color.
    Color,
    /// Four unsigned palette-local joint indices.
    JointIndices,
    /// Four independently authored normalized skin weights.
    JointWeights,
}
/// Established scalar encodings; numeric game-specific format codes remain explicit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElementEncoding {
    /// Two little-endian IEEE binary32 components.
    F32x2,
    /// Three little-endian IEEE binary32 components.
    F32x3,
    /// Four little-endian IEEE binary32 components.
    F32x4,
    /// Two little-endian IEEE binary16 components.
    F16x2,
    /// Four little-endian IEEE binary16 components.
    F16x4,
    /// Four unsigned bytes normalized by 255.
    Unorm8x4,
    /// Four signed bytes normalized by 127, with -128 clamped to -1.
    Snorm8x4,
    /// Four exact unsigned bytes.
    Uint8x4,
    /// Four little-endian unsigned 16-bit integers.
    Uint16x4,
    /// Four little-endian unsigned 32-bit integers.
    Uint32x4,
}
impl ElementEncoding {
    /// Complete storage width, including every component.
    pub const fn width(self) -> u32 {
        match self {
            Self::F32x2 | Self::F16x4 | Self::Uint16x4 => 8,
            Self::F32x3 => 12,
            Self::F32x4 | Self::Uint32x4 => 16,
            _ => 4,
        }
    }
    /// Number of logical components represented by this encoding.
    pub const fn components(self) -> usize {
        match self {
            Self::F32x2 | Self::F16x2 => 2,
            Self::F32x3 => 3,
            _ => 4,
        }
    }
    fn unsigned(self) -> bool {
        matches!(self, Self::Uint8x4 | Self::Uint16x4 | Self::Uint32x4)
    }
}
/// A caller-established correspondence between native declaration codes and logical meaning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VertexElement {
    /// Native `VertexElem.eSemantic` code; no global game mapping is assumed.
    pub native_semantic: i32,
    /// Native `VertexElem.eFormat` code interpreted by `encoding`.
    pub native_format: i32,
    /// Byte offset from the start of each native vertex.
    pub offset: u32,
    /// Logical meaning in the semantic geometry vocabulary.
    pub semantic: ElementSemantic,
    /// Exact scalar storage encoding selected for this declaration.
    pub encoding: ElementEncoding,
}
/// Explicit native vertex interpretation, checked against every declared element.
///
/// This is a caller assertion about the producing game's vertex codes. The codec
/// verifies structure and values, but does not establish that assertion's game fidelity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VertexLayout {
    /// Native buffer-format selector.
    pub native_format: i32,
    /// Full vertex record width, including unassigned bytes.
    pub stride: u32,
    /// Complete logical interpretation of the native element declaration list.
    pub elements: Vec<VertexElement>,
}
impl VertexLayout {
    /// Rejects overlapping fields, duplicate meanings, incomplete skin pairs, and invalid widths.
    pub fn validate(&self) -> Result<(), Error> {
        if self.stride == 0 {
            return Err(error("vertex stride is zero"));
        }
        let mut meanings = BTreeSet::new();
        let mut codes = BTreeSet::new();
        let mut ranges = Vec::new();
        for element in &self.elements {
            if !meanings.insert(element.semantic) || !codes.insert(element.native_semantic) {
                return Err(error("duplicate logical meaning or native semantic code"));
            }
            let end = element
                .offset
                .checked_add(element.encoding.width())
                .ok_or_else(|| error("element offset overflows"))?;
            if end > self.stride {
                return Err(error("vertex element exceeds the declared stride"));
            }
            if ranges
                .iter()
                .any(|&(start, old_end)| element.offset < old_end && start < end)
            {
                return Err(error("vertex element byte ranges overlap"));
            }
            ranges.push((element.offset, end));
            let required = match element.semantic {
                ElementSemantic::Position | ElementSemantic::Normal => 3,
                ElementSemantic::TexCoord(_) => 2,
                _ => 4,
            };
            if element.encoding.components() != required {
                return Err(error(
                    "element component count differs from its logical meaning",
                ));
            }
            if matches!(element.semantic, ElementSemantic::JointIndices)
                != element.encoding.unsigned()
            {
                return Err(error(
                    "unsigned integer encodings are reserved for joint indices",
                ));
            }
        }
        if !meanings.contains(&ElementSemantic::Position) {
            return Err(error("vertex layout has no position element"));
        }
        if meanings.contains(&ElementSemantic::JointIndices)
            != meanings.contains(&ElementSemantic::JointWeights)
        {
            return Err(error(
                "joint index and weight elements must be declared together",
            ));
        }
        let uv: Vec<_> = meanings
            .iter()
            .filter_map(|s| {
                if let ElementSemantic::TexCoord(i) = s {
                    Some(*i)
                } else {
                    None
                }
            })
            .collect();
        if uv.iter().enumerate().any(|(i, v)| i as u32 != *v) {
            return Err(error(
                "texture coordinate sets must be consecutive from zero",
            ));
        }
        Ok(())
    }
    /// Checks native codes, byte offsets, stride, and complete-record payload length.
    pub fn validate_buffer(&self, buffer: &GeoChunkVertexBuffer<'_>) -> Result<usize, Error> {
        self.validate()?;
        if buffer.vb_format != self.native_format
            || buffer.vert_stride != self.stride
            || buffer.vertex_elems.len() != self.elements.len()
        {
            return Err(error(
                "native buffer differs from the supplied layout declaration",
            ));
        }
        let mut native_semantics = BTreeSet::new();
        for native in &buffer.vertex_elems {
            if !native_semantics.insert(native.semantic) {
                return Err(error("duplicate native vertex semantic declaration"));
            }
            let matched = self.elements.iter().any(|e| {
                e.native_semantic == native.semantic
                    && e.native_format == native.format
                    && i32::try_from(e.offset).ok() == Some(native.offset)
            });
            if !matched {
                return Err(error(
                    "native vertex element has no matching explicit interpretation",
                ));
            }
        }
        if !buffer
            .chunk_vertices
            .len()
            .is_multiple_of(self.stride as usize)
        {
            return Err(error("native vertex payload has an incomplete record"));
        }
        Ok(buffer.chunk_vertices.len() / self.stride as usize)
    }
    /// Finds the explicit element corresponding to one logical meaning.
    pub fn element(&self, semantic: ElementSemantic) -> Option<VertexElement> {
        self.elements
            .iter()
            .find(|e| e.semantic == semantic)
            .copied()
    }
    /// Produces a fresh native declaration without retaining runtime buffer IDs.
    pub(crate) fn buffer(&self, bytes: &[u8]) -> Result<GeoChunkVertexBuffer<'static>, Error> {
        self.validate()?;
        Ok(GeoChunkVertexBuffer {
            vb_format: self.native_format,
            vert_stride: self.stride,
            vertex_elems: self
                .elements
                .iter()
                .map(|e| {
                    Ok(VertexElem {
                        semantic: e.native_semantic,
                        format: e.native_format,
                        offset: i32::try_from(e.offset)
                            .map_err(|_| error("element offset exceeds signed native width"))?,
                    })
                })
                .collect::<Result<_, Error>>()?,
            chunk_vertices: ZeroVec::alloc_from_slice(bytes),
            vfid: -1,
            vbid: -1,
            baid: -1,
            ..Default::default()
        })
    }
}
/// An interpretation attached to a native chunk-local source vertex buffer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BufferLayout {
    /// Native structure chunk index.
    pub chunk: u32,
    /// Vertex buffer index within that chunk.
    pub buffer: u32,
    /// Explicit correspondence for that buffer's declarations.
    pub layout: VertexLayout,
}
pub(crate) fn read(
    bytes: &[u8],
    vertex: usize,
    stride: u32,
    element: VertexElement,
) -> Result<[f32; 4], Error> {
    let at = vertex
        .checked_mul(stride as usize)
        .and_then(|v| v.checked_add(element.offset as usize))
        .ok_or_else(|| error("vertex range overflows"))?;
    let end = at
        .checked_add(element.encoding.width() as usize)
        .ok_or_else(|| error("vertex range overflows"))?;
    let data = bytes
        .get(at..end)
        .ok_or_else(|| error("vertex range exceeds payload"))?;
    let mut values = [0.; 4];
    for (i, value) in values
        .iter_mut()
        .enumerate()
        .take(element.encoding.components())
    {
        *value = match element.encoding {
            ElementEncoding::F32x2 | ElementEncoding::F32x3 | ElementEncoding::F32x4 => {
                f32::from_le_bytes(data[i * 4..i * 4 + 4].try_into().expect("four bytes"))
            }
            ElementEncoding::F16x2 | ElementEncoding::F16x4 => {
                Half::from_bits(u16::from_le_bytes([data[i * 2], data[i * 2 + 1]])).to_f32()
            }
            ElementEncoding::Unorm8x4 => f32::from(data[i]) / 255.,
            ElementEncoding::Snorm8x4 => (f32::from(data[i] as i8) / 127.).max(-1.),
            ElementEncoding::Uint8x4 => f32::from(data[i]),
            ElementEncoding::Uint16x4 => {
                f32::from(u16::from_le_bytes([data[i * 2], data[i * 2 + 1]]))
            }
            ElementEncoding::Uint32x4 => {
                u32::from_le_bytes(data[i * 4..i * 4 + 4].try_into().expect("four bytes")) as f32
            }
        };
        if !value.is_finite() {
            return Err(error("vertex component is nonfinite"));
        }
        if element.encoding == ElementEncoding::Uint32x4 && *value > 65535. {
            return Err(error(
                "native joint index exceeds the supported skeleton width",
            ));
        }
    }
    Ok(values)
}
pub(crate) fn write(
    bytes: &mut [u8],
    vertex: usize,
    stride: u32,
    element: VertexElement,
    values: [f32; 4],
    context: &mut Context,
) -> Result<(), Error> {
    let at = vertex
        .checked_mul(stride as usize)
        .and_then(|v| v.checked_add(element.offset as usize))
        .ok_or_else(|| error("vertex range overflows"))?;
    let end = at
        .checked_add(element.encoding.width() as usize)
        .ok_or_else(|| error("vertex range overflows"))?;
    let data = bytes
        .get_mut(at..end)
        .ok_or_else(|| error("vertex range exceeds output"))?;
    for (i, &value) in values
        .iter()
        .enumerate()
        .take(element.encoding.components())
    {
        if !value.is_finite() {
            return Err(error("vertex component is nonfinite"));
        }
        let decoded = match element.encoding {
            ElementEncoding::F32x2 | ElementEncoding::F32x3 | ElementEncoding::F32x4 => {
                data[i * 4..i * 4 + 4].copy_from_slice(&value.to_le_bytes());
                value
            }
            ElementEncoding::F16x2 | ElementEncoding::F16x4 => {
                let half = Half::from_f32(value);
                if !half.is_finite() {
                    return Err(error("vertex component exceeds binary16 range"));
                }
                data[i * 2..i * 2 + 2].copy_from_slice(&half.to_bits().to_le_bytes());
                half.to_f32()
            }
            ElementEncoding::Unorm8x4 => {
                if !(0. ..=1.).contains(&value) {
                    return Err(error("unsigned normalized component is outside [0,1]"));
                }
                let v = (value * 255.).round() as u8;
                data[i] = v;
                f32::from(v) / 255.
            }
            ElementEncoding::Snorm8x4 => {
                if !(-1. ..=1.).contains(&value) {
                    return Err(error("signed normalized component is outside [-1,1]"));
                }
                let v = (value * 127.).round() as i8;
                data[i] = v as u8;
                f32::from(v) / 127.
            }
            ElementEncoding::Uint8x4 => {
                if value.fract() != 0. || !(0. ..=255.).contains(&value) {
                    return Err(error("joint index exceeds u8"));
                }
                data[i] = value as u8;
                value
            }
            ElementEncoding::Uint16x4 => {
                if value.fract() != 0. || !(0. ..=65535.).contains(&value) {
                    return Err(error("joint index exceeds u16"));
                }
                data[i * 2..i * 2 + 2].copy_from_slice(&(value as u16).to_le_bytes());
                value
            }
            ElementEncoding::Uint32x4 => {
                if value.fract() != 0. || !(0. ..=65535.).contains(&value) {
                    return Err(error("joint index exceeds supported native skeleton width"));
                }
                data[i * 4..i * 4 + 4].copy_from_slice(&(value as u32).to_le_bytes());
                value
            }
        };
        context.approximate(
            "d4.geometry.vertex",
            (value - decoded).abs(),
            "selected native scalar encoding requires quantization",
        )?;
    }
    Ok(())
}
fn error(message: &str) -> Error {
    Error::new("d4.vertex_layout", message)
}
