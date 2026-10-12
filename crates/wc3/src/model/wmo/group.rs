//! Typed group chunk payloads, preserving repeated streams and native record widths.
//!
//! Geometry and collision arrays borrow their source bytes. Liquid union fields
//! require explicit water/magma context; decoding without that context retains
//! raw vertices. Tangent payloads require the owning MOBA batch count.
use crate::model::{common::Ownership, wmo};
use std::borrow::Cow;
use wmo::codec::{Array, Decoder, Encoder};
use wmo::records::{
    Batch, BatchOverride, BspNode, LightSetRange, Plane, PointLight, PointLightAnimation,
    PrepassBatch, ShadowBatch, SpotLight, SpotLightAnimation, Triangle, Triangle2,
};
use wmo::{Dialect, Read as _, ReadError, Validate, Write as _, WriteError};

/// Explicit interpretation of the first four bytes of every liquid vertex.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiquidKind {
    /// Water and ocean flow-byte representation.
    Water,
    /// Magma and slime signed texture-coordinate representation.
    Magma,
}
/// Context whose absence cannot be inferred from an isolated chunk payload.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GroupContext {
    /// Number of MOBA batches, determining the MOTA u16 prefix length.
    pub batch_count: Option<usize>,
    /// Liquid interpretation resolved by the caller from root/group material information.
    pub liquid_kind: Option<LiquidKind>,
}
/// Packed 30-byte MLIQ dimensions, origin and material reference.
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
pub struct LiquidHeader {
    /// Number of grid vertices along X and Y.
    pub vertex_dimensions: [i32; 2],
    /// Number of grid tiles along X and Y.
    pub tile_dimensions: [i32; 2],
    /// Lower grid corner in group coordinates.
    pub corner: [f32; 3],
    /// Native material reference; root flags determine whether it addresses MOMT or LiquidType.
    pub material: u16,
}
/// Eight-byte liquid vertex retaining the uninterpreted union prefix.
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
pub struct LiquidVertex {
    /// Water flow bytes or two little-endian signed magma UV coordinates.
    pub data: [u8; 4],
    /// Surface height in model-space units.
    pub height: f32,
}
/// Eight-byte liquid vertex under the explicit water/ocean interpretation.
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
pub struct WaterVertex {
    /// First native flow velocity byte.
    pub flow_1: u8,
    /// Second native flow velocity byte.
    pub flow_2: u8,
    /// Percentage assigned to the first flow component, retained without normalization.
    pub flow_1_percentage: u8,
    /// Preserved padding byte.
    pub filler: u8,
    /// Surface height in model-space units.
    pub height: f32,
}
/// Eight-byte liquid vertex under the explicit magma/slime interpretation.
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
pub struct MagmaVertex {
    /// Signed native texture coordinates S and T.
    pub texture_coordinates: [i16; 2],
    /// Surface height in model-space units.
    pub height: f32,
}
/// Borrowed liquid vertices retaining the caller-selected union interpretation.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum LiquidVertices<'data> {
    /// Union bytes preserved until an explicit interpretation is supplied.
    Unresolved(Array<'data, LiquidVertex>),
    /// Water/ocean flow bytes and heights.
    Water(Array<'data, WaterVertex>),
    /// Magma/slime texture coordinates and heights.
    Magma(Array<'data, MagmaVertex>),
}
impl LiquidVertices<'_> {
    /// Number of liquid vertices.
    pub fn len(&self) -> usize {
        match self {
            Self::Unresolved(v) => v.len(),
            Self::Water(v) => v.len(),
            Self::Magma(v) => v.len(),
        }
    }
    /// Whether the liquid has no grid vertices.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Exact encoded vertex bytes independent of union interpretation.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Unresolved(v) => v.as_bytes(),
            Self::Water(v) => v.as_bytes(),
            Self::Magma(v) => v.as_bytes(),
        }
    }
}
/// An MLIQ vertex grid and tile flag grid, without implicit material resolution.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct Liquid<'data> {
    /// Native dimensions, corner and material reference.
    pub header: LiquidHeader,
    /// Row-major vertices, with X varying fastest.
    pub vertices: LiquidVertices<'data>,
    /// Row-major tile flags; native type occupies low bits and upper bits carry fishable/shared hints.
    pub tiles: Array<'data, u8>,
}
fn dimensions(dimensions: [i32; 2]) -> Result<usize, ReadError> {
    let x = usize::try_from(dimensions[0])
        .map_err(|_| ReadError::invalid(0, "negative liquid dimension"))?;
    let y = usize::try_from(dimensions[1])
        .map_err(|_| ReadError::invalid(0, "negative liquid dimension"))?;
    x.checked_mul(y)
        .ok_or_else(|| ReadError::invalid(0, "liquid dimensions overflow"))
}
impl<'data> Liquid<'data> {
    /// Reads a complete MLIQ payload using only explicitly provided material context.
    pub fn decode(
        payload: &'data [u8],
        origin: usize,
        kind: Option<LiquidKind>,
    ) -> Result<Self, ReadError> {
        let mut input = Decoder::new(payload).with_origin(origin);
        let header = LiquidHeader::read(&mut input)?;
        let vertex_count = dimensions(header.vertex_dimensions)
            .map_err(|_| ReadError::invalid(origin, "liquid vertex dimensions"))?;
        let tile_count = dimensions(header.tile_dimensions)
            .map_err(|_| ReadError::invalid(origin + 8, "liquid tile dimensions"))?;
        let length = vertex_count
            .checked_mul(8)
            .ok_or_else(|| ReadError::invalid(origin, "liquid vertex dimensions overflow"))?;
        let bytes = input.bytes(length)?;
        let vertices = match kind {
            None => LiquidVertices::Unresolved(Array::decode(bytes)?),
            Some(LiquidKind::Water) => LiquidVertices::Water(Array::decode(bytes)?),
            Some(LiquidKind::Magma) => LiquidVertices::Magma(Array::decode(bytes)?),
        };
        let tiles = Array::decode(input.bytes(tile_count)?)?;
        if input.remaining() != 0 {
            return Err(ReadError::invalid(
                input.absolute_position(),
                "liquid trailing bytes",
            ));
        }
        Ok(Self {
            header,
            vertices,
            tiles,
        })
    }
    /// Writes native dimensions and grids, rejecting inconsistent cardinalities.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        self.validate()
            .map_err(|_| WriteError::invalid("liquid grid dimensions"))?;
        let mut output = Encoder::new();
        self.header.write(&mut output)?;
        output.bytes(self.vertices.as_bytes())?;
        self.tiles.write(&mut output)?;
        Ok(output.finish())
    }
}
impl Validate for Liquid<'_> {
    fn validate(&self) -> Result<(), ReadError> {
        if dimensions(self.header.vertex_dimensions)? != self.vertices.len()
            || dimensions(self.header.tile_dimensions)? != self.tiles.len()
        {
            return Err(ReadError::invalid(0, "liquid grid cardinality"));
        }
        Ok(())
    }
}
/// MOTA's batch-to-tangent mapping followed by four-component tangent vectors.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct Tangents<'data> {
    /// One tangent start per MOBA batch; 0xffff marks a batch with no tangents.
    pub first_tangent: Array<'data, u16>,
    /// Tangent vectors in their original four-component representation.
    pub tangents: Array<'data, [f32; 4]>,
}
impl<'data> Tangents<'data> {
    /// Decodes the payload with the owning draw-batch count.
    pub fn decode(
        payload: &'data [u8],
        origin: usize,
        batch_count: usize,
    ) -> Result<Self, ReadError> {
        let length = batch_count
            .checked_mul(2)
            .ok_or_else(|| ReadError::invalid(origin, "tangent batch count overflow"))?;
        let mut input = Decoder::new(payload).with_origin(origin);
        let first_tangent = Array::decode(input.bytes(length)?)?;
        let tangents = Array::decode(input.bytes(input.remaining())?)
            .map_err(|_| ReadError::invalid(input.absolute_position(), "tangent vector width"))?;
        Ok(Self {
            first_tangent,
            tangents,
        })
    }
    /// Encodes the map and vector arrays in native order.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        let mut output = Encoder::new();
        self.first_tangent.write(&mut output)?;
        self.tangents.write(&mut output)?;
        Ok(output.finish())
    }
}
/// MOGX query base plus later detail-doodad selection bytes.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct QueryGeometry<'data> {
    /// First triangle covered by MOQG ground-type query entries.
    pub first_face: u32,
    /// Later detail-doodad key and any additional bytes, retained in native order.
    pub extension: Cow<'data, [u8]>,
}
impl QueryGeometry<'_> {
    /// Returns the detail-doodad key when the payload contains that extension.
    pub fn detail_doodad_key(&self) -> Option<u8> {
        self.extension.first().copied()
    }
}

/// One group subchunk, keeping its native tag and occurrence in file order.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum GroupChunk<'data, D: Dialect> {
    /// MOPY: Per-triangle flags and eight-bit root material indices.
    Triangles(Array<'data, Triangle>),
    /// MPY2: Extended triangle flags and sixteen-bit root material indices.
    Triangles2(Array<'data, Triangle2>),
    /// MOVI: Sixteen-bit triangle vertex indices.
    Indices(Array<'data, u16>),
    /// MOVX: Thirty-two-bit triangle vertex indices for large meshes.
    Indices32(Array<'data, u32>),
    /// MOVT: Model-space vertex positions.
    Positions(Array<'data, [f32; 3]>),
    /// MONR: Vertex normals in native component order.
    Normals(Array<'data, [f32; 3]>),
    /// MOTV: One texture-coordinate layer; repeated chunks retain separate layers.
    TextureCoordinates(Array<'data, [f32; 2]>),
    /// MOCV: One blue/green/red/alpha color layer; repeated chunks remain separate.
    VertexColors(Array<'data, [u8; 4]>),
    /// MOC2: Alternate modern blue/green/red/alpha vertex colors.
    VertexColors2(Array<'data, [u8; 4]>),
    /// MOBA: Rendering batches with dialect-selected prefix metadata.
    Batches(Array<'data, Batch<D>>),
    /// MOBS: Shadow-pass draw batches.
    ShadowBatches(Array<'data, ShadowBatch>),
    /// MOPB: Fixed-width prepass records with unresolved field semantics.
    PrepassBatches(Array<'data, PrepassBatch>),
    /// MORB: Triangle-strip range overrides for corresponding draw batches.
    BatchOverrides(Array<'data, BatchOverride>),
    /// MORI: Triangle-strip vertex indices.
    StripIndices(Array<'data, u16>),
    /// MOLR: Indices into the root light table.
    LightReferences(Array<'data, u16>),
    /// MNLR: Indices into the root modern light table.
    NewLightReferences(Array<'data, u16>),
    /// MODR: Indices into the root doodad definition table.
    DoodadReferences(Array<'data, u16>),
    /// MPVR: Indices into the root particulate volume table.
    ParticulateReferences(Array<'data, u16>),
    /// MAVR: Indices into the root ambient volume table.
    AmbientVolumeReferences(Array<'data, u16>),
    /// MBVR: Indices into the root ambient box table.
    AmbientBoxReferences(Array<'data, u16>),
    /// MFVR: Root fog indices replacing the four header fog slots.
    FogReferences(Array<'data, u16>),
    /// MFBR: Indices into the root fog box table.
    FogBoxReferences(Array<'data, u16>),
    /// MOLP: Group-local point lights.
    PointLights(Array<'data, PointLight>),
    /// MOLS: Fixed-width spot-light records with unresolved field semantics.
    SpotLights(Array<'data, SpotLight>),
    /// MOP2: Animated point lights with flicker and cookie texture references.
    PointLightAnimations(Array<'data, PointLightAnimation>),
    /// MOS2: Fixed-width animated spot-light records with unresolved fields.
    SpotLightAnimations(Array<'data, SpotLightAnimation>),
    /// MLSP: Windows into MOLP point lights.
    PointLightSets(Array<'data, LightSetRange>),
    /// MLSS: Windows into MOLS spot lights.
    SpotLightSets(Array<'data, LightSetRange>),
    /// MLSK: Windows into MOP2 point-light animations.
    PointAnimationSets(Array<'data, LightSetRange>),
    /// MLSO: Windows into MOS2 spot-light animations.
    SpotAnimationSets(Array<'data, LightSetRange>),
    /// MOPL: Model-space terrain-cutting planes.
    TerrainCutPlanes(Array<'data, Plane>),
    /// MOQG: Ground types for triangles relative to the MOGX query base.
    GroundTypes(Array<'data, u32>),
    /// MOBN: Collision BSP nodes.
    BspNodes(Array<'data, BspNode>),
    /// MOBR: Triangle indices referenced by BSP leaf windows.
    BspFaces(Array<'data, u16>),
    /// MOLM: Legacy lightmap material payload with an unresolved layout.
    LightMapMaterials(Cow<'data, [u8]>),
    /// MOLD: Legacy lightmap data with an unresolved layout.
    LightMapData(Cow<'data, [u8]>),
    /// MOIN: Legacy index payload with an unresolved layout.
    LegacyIndices(Cow<'data, [u8]>),
    /// MOLV: Legacy lightmap coordinate payload with an unresolved layout.
    LightMapCoordinates(Cow<'data, [u8]>),
    /// MPBV: Legacy batch vertex payload with an unresolved layout.
    LegacyBatchVertices(Cow<'data, [u8]>),
    /// MPBI: Legacy batch index payload with an unresolved layout.
    LegacyBatchIndices(Cow<'data, [u8]>),
    /// MPBG: Legacy batch geometry payload with an unresolved layout.
    LegacyBatchGeometry(Cow<'data, [u8]>),
    /// MPBP: Legacy batch plane payload with an unresolved layout.
    LegacyBatchPlanes(Cow<'data, [u8]>),
    /// MLIQ liquid grids with explicit or unresolved vertex interpretation.
    Liquid(Liquid<'data>),
    /// MOTA tangent arrays decoded with a supplied batch count.
    Tangents(Tangents<'data>),
    /// MOTA bytes retained because no owning draw-batch count was supplied.
    TangentsUnresolved(Cow<'data, [u8]>),
    /// MOGX ground-query base and detail-doodad extension bytes.
    QueryGeometry(QueryGeometry<'data>),
    /// MDAL ambient override in blue/green/red/alpha byte order.
    AmbientOverride([u8; 4]),
    /// Unrecognized group chunk retaining logical tag spelling and exact payload.
    Unknown {
        /// Logical FourCC, before the WMO wire reversal.
        tag: [u8; 4],
        /// Exact native payload bytes.
        data: Cow<'data, [u8]>,
    },
}
impl<'data, D: Dialect> GroupChunk<'data, D> {
    /// Decodes a standalone payload without guessing tangent or liquid context.
    pub fn decode(tag: [u8; 4], payload: &'data [u8], origin: usize) -> Result<Self, ReadError> {
        Self::decode_with_context(tag, payload, origin, &GroupContext::default())
    }
    /// Decodes a payload using explicit group-wide context for dependent layouts.
    pub fn decode_with_context(
        tag: [u8; 4],
        payload: &'data [u8],
        origin: usize,
        context: &GroupContext,
    ) -> Result<Self, ReadError> {
        let chunk = match &tag {
            b"MOPY" => Self::Triangles(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOPY record width"))?,
            ),
            b"MPY2" => Self::Triangles2(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MPY2 record width"))?,
            ),
            b"MOVI" => Self::Indices(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOVI record width"))?,
            ),
            b"MOVX" => Self::Indices32(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOVX record width"))?,
            ),
            b"MOVT" => Self::Positions(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOVT record width"))?,
            ),
            b"MONR" => Self::Normals(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MONR record width"))?,
            ),
            b"MOTV" => Self::TextureCoordinates(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOTV record width"))?,
            ),
            b"MOCV" => Self::VertexColors(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOCV record width"))?,
            ),
            b"MOC2" => Self::VertexColors2(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOC2 record width"))?,
            ),
            b"MOBA" => Self::Batches(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOBA record width"))?,
            ),
            b"MOBS" => Self::ShadowBatches(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOBS record width"))?,
            ),
            b"MOPB" => Self::PrepassBatches(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOPB record width"))?,
            ),
            b"MORB" => Self::BatchOverrides(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MORB record width"))?,
            ),
            b"MORI" => Self::StripIndices(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MORI record width"))?,
            ),
            b"MOLR" => Self::LightReferences(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOLR record width"))?,
            ),
            b"MNLR" => Self::NewLightReferences(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MNLR record width"))?,
            ),
            b"MODR" => Self::DoodadReferences(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MODR record width"))?,
            ),
            b"MPVR" => Self::ParticulateReferences(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MPVR record width"))?,
            ),
            b"MAVR" => Self::AmbientVolumeReferences(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MAVR record width"))?,
            ),
            b"MBVR" => Self::AmbientBoxReferences(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MBVR record width"))?,
            ),
            b"MFVR" => Self::FogReferences(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MFVR record width"))?,
            ),
            b"MFBR" => Self::FogBoxReferences(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MFBR record width"))?,
            ),
            b"MOLP" => Self::PointLights(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOLP record width"))?,
            ),
            b"MOLS" => Self::SpotLights(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOLS record width"))?,
            ),
            b"MOP2" => Self::PointLightAnimations(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOP2 record width"))?,
            ),
            b"MOS2" => Self::SpotLightAnimations(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOS2 record width"))?,
            ),
            b"MLSP" => Self::PointLightSets(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MLSP record width"))?,
            ),
            b"MLSS" => Self::SpotLightSets(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MLSS record width"))?,
            ),
            b"MLSK" => Self::PointAnimationSets(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MLSK record width"))?,
            ),
            b"MLSO" => Self::SpotAnimationSets(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MLSO record width"))?,
            ),
            b"MOPL" => Self::TerrainCutPlanes(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOPL record width"))?,
            ),
            b"MOQG" => Self::GroundTypes(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOQG record width"))?,
            ),
            b"MOBN" => Self::BspNodes(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOBN record width"))?,
            ),
            b"MOBR" => Self::BspFaces(
                Array::decode(payload)
                    .map_err(|_| ReadError::invalid(origin, "MOBR record width"))?,
            ),
            b"MOLM" => Self::LightMapMaterials(Cow::Borrowed(payload)),
            b"MOLD" => Self::LightMapData(Cow::Borrowed(payload)),
            b"MOIN" => Self::LegacyIndices(Cow::Borrowed(payload)),
            b"MOLV" => Self::LightMapCoordinates(Cow::Borrowed(payload)),
            b"MPBV" => Self::LegacyBatchVertices(Cow::Borrowed(payload)),
            b"MPBI" => Self::LegacyBatchIndices(Cow::Borrowed(payload)),
            b"MPBG" => Self::LegacyBatchGeometry(Cow::Borrowed(payload)),
            b"MPBP" => Self::LegacyBatchPlanes(Cow::Borrowed(payload)),
            b"MLIQ" => Self::Liquid(Liquid::decode(payload, origin, context.liquid_kind)?),
            b"MOTA" => match context.batch_count {
                Some(count) => Self::Tangents(Tangents::decode(payload, origin, count)?),
                None => Self::TangentsUnresolved(Cow::Borrowed(payload)),
            },
            b"MOGX" => {
                let mut input = Decoder::new(payload).with_origin(origin);
                let first_face = input.read()?;
                let extension = Cow::Borrowed(input.bytes(input.remaining())?);
                Self::QueryGeometry(QueryGeometry {
                    first_face,
                    extension,
                })
            }
            b"MDAL" => {
                let mut input = Decoder::new(payload).with_origin(origin);
                let color = input.read()?;
                if input.remaining() != 0 {
                    return Err(ReadError::invalid(
                        input.absolute_position(),
                        "ambient override width",
                    ));
                }
                Self::AmbientOverride(color)
            }
            _ => Self::Unknown {
                tag,
                data: Cow::Borrowed(payload),
            },
        };
        Ok(chunk)
    }
    /// Logical FourCC for this payload, before reversal into WMO wire spelling.
    pub fn tag(&self) -> [u8; 4] {
        match self {
            Self::Triangles(..) => *b"MOPY",
            Self::Triangles2(..) => *b"MPY2",
            Self::Indices(..) => *b"MOVI",
            Self::Indices32(..) => *b"MOVX",
            Self::Positions(..) => *b"MOVT",
            Self::Normals(..) => *b"MONR",
            Self::TextureCoordinates(..) => *b"MOTV",
            Self::VertexColors(..) => *b"MOCV",
            Self::VertexColors2(..) => *b"MOC2",
            Self::Batches(..) => *b"MOBA",
            Self::ShadowBatches(..) => *b"MOBS",
            Self::PrepassBatches(..) => *b"MOPB",
            Self::BatchOverrides(..) => *b"MORB",
            Self::StripIndices(..) => *b"MORI",
            Self::LightReferences(..) => *b"MOLR",
            Self::NewLightReferences(..) => *b"MNLR",
            Self::DoodadReferences(..) => *b"MODR",
            Self::ParticulateReferences(..) => *b"MPVR",
            Self::AmbientVolumeReferences(..) => *b"MAVR",
            Self::AmbientBoxReferences(..) => *b"MBVR",
            Self::FogReferences(..) => *b"MFVR",
            Self::FogBoxReferences(..) => *b"MFBR",
            Self::PointLights(..) => *b"MOLP",
            Self::SpotLights(..) => *b"MOLS",
            Self::PointLightAnimations(..) => *b"MOP2",
            Self::SpotLightAnimations(..) => *b"MOS2",
            Self::PointLightSets(..) => *b"MLSP",
            Self::SpotLightSets(..) => *b"MLSS",
            Self::PointAnimationSets(..) => *b"MLSK",
            Self::SpotAnimationSets(..) => *b"MLSO",
            Self::TerrainCutPlanes(..) => *b"MOPL",
            Self::GroundTypes(..) => *b"MOQG",
            Self::BspNodes(..) => *b"MOBN",
            Self::BspFaces(..) => *b"MOBR",
            Self::LightMapMaterials(..) => *b"MOLM",
            Self::LightMapData(..) => *b"MOLD",
            Self::LegacyIndices(..) => *b"MOIN",
            Self::LightMapCoordinates(..) => *b"MOLV",
            Self::LegacyBatchVertices(..) => *b"MPBV",
            Self::LegacyBatchIndices(..) => *b"MPBI",
            Self::LegacyBatchGeometry(..) => *b"MPBG",
            Self::LegacyBatchPlanes(..) => *b"MPBP",
            Self::Liquid(..) => *b"MLIQ",
            Self::Tangents(..) | Self::TangentsUnresolved(..) => *b"MOTA",
            Self::QueryGeometry(..) => *b"MOGX",
            Self::AmbientOverride(..) => *b"MDAL",
            Self::Unknown { tag, .. } => *tag,
        }
    }
    /// Encodes a payload without chunk framing or implicit material conversion.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        let mut output = Encoder::new();
        match self {
            Self::Triangles(values) => values.write(&mut output)?,
            Self::Triangles2(values) => values.write(&mut output)?,
            Self::Indices(values) => values.write(&mut output)?,
            Self::Indices32(values) => values.write(&mut output)?,
            Self::Positions(values) => values.write(&mut output)?,
            Self::Normals(values) => values.write(&mut output)?,
            Self::TextureCoordinates(values) => values.write(&mut output)?,
            Self::VertexColors(values) => values.write(&mut output)?,
            Self::VertexColors2(values) => values.write(&mut output)?,
            Self::Batches(values) => values.write(&mut output)?,
            Self::ShadowBatches(values) => values.write(&mut output)?,
            Self::PrepassBatches(values) => values.write(&mut output)?,
            Self::BatchOverrides(values) => values.write(&mut output)?,
            Self::StripIndices(values) => values.write(&mut output)?,
            Self::LightReferences(values) => values.write(&mut output)?,
            Self::NewLightReferences(values) => values.write(&mut output)?,
            Self::DoodadReferences(values) => values.write(&mut output)?,
            Self::ParticulateReferences(values) => values.write(&mut output)?,
            Self::AmbientVolumeReferences(values) => values.write(&mut output)?,
            Self::AmbientBoxReferences(values) => values.write(&mut output)?,
            Self::FogReferences(values) => values.write(&mut output)?,
            Self::FogBoxReferences(values) => values.write(&mut output)?,
            Self::PointLights(values) => values.write(&mut output)?,
            Self::SpotLights(values) => values.write(&mut output)?,
            Self::PointLightAnimations(values) => values.write(&mut output)?,
            Self::SpotLightAnimations(values) => values.write(&mut output)?,
            Self::PointLightSets(values) => values.write(&mut output)?,
            Self::SpotLightSets(values) => values.write(&mut output)?,
            Self::PointAnimationSets(values) => values.write(&mut output)?,
            Self::SpotAnimationSets(values) => values.write(&mut output)?,
            Self::TerrainCutPlanes(values) => values.write(&mut output)?,
            Self::GroundTypes(values) => values.write(&mut output)?,
            Self::BspNodes(values) => values.write(&mut output)?,
            Self::BspFaces(values) => values.write(&mut output)?,
            Self::LightMapMaterials(bytes) => output.bytes(bytes)?,
            Self::LightMapData(bytes) => output.bytes(bytes)?,
            Self::LegacyIndices(bytes) => output.bytes(bytes)?,
            Self::LightMapCoordinates(bytes) => output.bytes(bytes)?,
            Self::LegacyBatchVertices(bytes) => output.bytes(bytes)?,
            Self::LegacyBatchIndices(bytes) => output.bytes(bytes)?,
            Self::LegacyBatchGeometry(bytes) => output.bytes(bytes)?,
            Self::LegacyBatchPlanes(bytes) => output.bytes(bytes)?,
            Self::Liquid(value) => return value.encode(),
            Self::Tangents(value) => return value.encode(),
            Self::TangentsUnresolved(bytes) => output.bytes(bytes)?,
            Self::QueryGeometry(value) => {
                value.first_face.write(&mut output)?;
                output.bytes(&value.extension)?;
            }
            Self::AmbientOverride(value) => value.write(&mut output)?,
            Self::Unknown { data, .. } => output.bytes(data)?,
        }
        Ok(output.finish())
    }
}
