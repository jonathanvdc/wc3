//! Root chunk payloads; document framing and mandatory-header handling live in the root container.
use super as wmo;
use super::codec::{Array, Decoder, Encoder};
use super::records::*;
use crate::model::common::Ownership;
use std::borrow::Cow;
use wmo::{Dialect, Read, ReadError, RecordLayout, Write, WriteError};
/// Native root chunks with bounded typed payloads and byte-preserving unknown extensions.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum RootChunk<'data, D: Dialect> {
    /// MOHD root header, extracted into the owning document by its container.
    Header(Header<D>),
    /// MOSI skybox model FileDataID.
    SkyboxFileId(u32),
    /// MOTX: Filename bytes addressed by material texture slots when MOTX is present.
    TextureNames(Array<'data, u8>),
    /// MOMT: Shared material definitions.
    Materials(Array<'data, Material<D>>),
    /// MOUV: Per-material UV scroll speeds.
    UvAnimations(Array<'data, MaterialUvAnimation>),
    /// MOMX: Per-material color and texture override keys.
    MaterialExtensions(Array<'data, MaterialExtension>),
    /// MOTN: Nine MOTX byte offsets per MOM3 material.
    M3TextureNames(Array<'data, [u32; 9]>),
    /// MOGN: Zero-terminated group-name byte block.
    GroupNames(Array<'data, u8>),
    /// MOGI: Group flags, bounds, and names.
    Groups(Array<'data, GroupInfo>),
    /// MGI2: Secondary group flags and LOD indices.
    Groups2(Array<'data, GroupInfo2>),
    /// MOSB: Zero-terminated skybox model path bytes.
    SkyboxName(Array<'data, u8>),
    /// MOPV: Portal polygon vertices in WMO coordinates.
    PortalVertices(Array<'data, [f32; 3]>),
    /// MOPT: Portal polygons and separating planes.
    Portals(Array<'data, Portal>),
    /// MOPR: Portal-to-group relationships.
    PortalRefs(Array<'data, PortalRef>),
    /// MOPE: Portal culling plane adjustment data.
    PortalExtras(Array<'data, PortalExtra>),
    /// MOVV: Historical visibility-block vertices.
    VisibleVertices(Array<'data, [f32; 3]>),
    /// MOVB: Ranges into MOVV visibility vertices.
    VisibleBlocks(Array<'data, VisibleBlock>),
    /// MOLT: Legacy light definitions.
    Lights(Array<'data, Light>),
    /// MOLV: Directional light terms whose active use remains unresolved.
    LightExtensions(Array<'data, LightExtension>),
    /// MNLD: Modern point and spot lights.
    NewLights(Array<'data, NewLight>),
    /// MODS: Named ranges of doodad placements.
    DoodadSets(Array<'data, DoodadSet>),
    /// MODN: Doodad model name bytes addressed by MODD.
    DoodadNames(Array<'data, u8>),
    /// MODI: Doodad model FileDataIDs addressed by MODD.
    DoodadFileIds(Array<'data, u32>),
    /// MODD: Doodad transforms, flags, and colors.
    Doodads(Array<'data, DoodadDef>),
    /// MDDI: Per-doodad color multipliers.
    DoodadColorMultipliers(Array<'data, f32>),
    /// MFOG: Spherical fog volumes.
    Fogs(Array<'data, Fog>),
    /// MFOB: Six-plane fog volumes.
    FogBoxes(Array<'data, FogBox>),
    /// MFED: Per-fog doodad-set restrictions.
    FogExtras(Array<'data, FogExtra>),
    /// MAVG: Global ambient lighting volumes.
    GlobalAmbients(Array<'data, AmbientVolume>),
    /// MAVD: Ambient lighting volumes.
    AmbientVolumes(Array<'data, AmbientVolume>),
    /// MBVD: Six-plane ambient volumes.
    AmbientBoxes(Array<'data, AmbientBox>),
    /// MPVD: Known 4272-byte volume records; field meanings remain unresolved.
    ParticulateVolumes(Array<'data, [u8; 4272]>),
    /// MCVP: Convex-volume planes (normal XYZ and distance).
    ConvexPlanes(Array<'data, [f32; 4]>),
    /// GFID: Group file FileDataIDs laid out in group-order rows per LOD.
    GroupFileIds(Array<'data, u32>),
    /// MOM3: Embedded M3SI material block; its payload structure remains unresolved.
    M3Materials(Cow<'data, [u8]>),
    /// MDDL: Detail-doodad layer and placement block; its payload structure remains unresolved.
    DetailDoodads(Cow<'data, [u8]>),
    /// Unrecognized chunk preserved without interpreting its data.
    Unknown {
        /// Logical FourCC, before reversal into native wire byte order.
        tag: [u8; 4],
        /// Original or edited bytes excluding the chunk header.
        data: Cow<'data, [u8]>,
    },
}
impl<'data, D: Dialect> RootChunk<'data, D> {
    /// Decodes one logical chunk payload with absolute diagnostic origin.
    pub fn decode(tag: [u8; 4], data: &'data [u8], origin: usize) -> Result<Self, ReadError> {
        let mut input = Decoder::new(data).with_origin(origin);
        Ok(match &tag {
            b"MOHD" => Self::Header(single(&mut input)?),
            b"MOSI" => Self::SkyboxFileId(single(&mut input)?),
            b"MOTX" => Self::TextureNames(array(data, origin)?),
            b"MOMT" => Self::Materials(array(data, origin)?),
            b"MOUV" => Self::UvAnimations(array(data, origin)?),
            b"MOMX" => Self::MaterialExtensions(array(data, origin)?),
            b"MOTN" => Self::M3TextureNames(array(data, origin)?),
            b"MOGN" => Self::GroupNames(array(data, origin)?),
            b"MOGI" => Self::Groups(array(data, origin)?),
            b"MGI2" => Self::Groups2(array(data, origin)?),
            b"MOSB" => Self::SkyboxName(array(data, origin)?),
            b"MOPV" => Self::PortalVertices(array(data, origin)?),
            b"MOPT" => Self::Portals(array(data, origin)?),
            b"MOPR" => Self::PortalRefs(array(data, origin)?),
            b"MOPE" => Self::PortalExtras(array(data, origin)?),
            b"MOVV" => Self::VisibleVertices(array(data, origin)?),
            b"MOVB" => Self::VisibleBlocks(array(data, origin)?),
            b"MOLT" => Self::Lights(array(data, origin)?),
            b"MOLV" => Self::LightExtensions(array(data, origin)?),
            b"MNLD" => Self::NewLights(array(data, origin)?),
            b"MODS" => Self::DoodadSets(array(data, origin)?),
            b"MODN" => Self::DoodadNames(array(data, origin)?),
            b"MODI" => Self::DoodadFileIds(array(data, origin)?),
            b"MODD" => Self::Doodads(array(data, origin)?),
            b"MDDI" => Self::DoodadColorMultipliers(array(data, origin)?),
            b"MFOG" => Self::Fogs(array(data, origin)?),
            b"MFOB" => Self::FogBoxes(array(data, origin)?),
            b"MFED" => Self::FogExtras(array(data, origin)?),
            b"MAVG" => Self::GlobalAmbients(array(data, origin)?),
            b"MAVD" => Self::AmbientVolumes(array(data, origin)?),
            b"MBVD" => Self::AmbientBoxes(array(data, origin)?),
            b"MPVD" => Self::ParticulateVolumes(array(data, origin)?),
            b"MCVP" => Self::ConvexPlanes(array(data, origin)?),
            b"GFID" => Self::GroupFileIds(array(data, origin)?),
            b"MOM3" => Self::M3Materials(Cow::Borrowed(data)),
            b"MDDL" => Self::DetailDoodads(Cow::Borrowed(data)),
            _ => Self::Unknown {
                tag,
                data: Cow::Borrowed(data),
            },
        })
    }
    /// Returns the logical chunk FourCC.
    pub fn tag(&self) -> [u8; 4] {
        match self {
            Self::Header(_) => *b"MOHD",
            Self::SkyboxFileId(_) => *b"MOSI",
            Self::TextureNames(_) => *b"MOTX",
            Self::Materials(_) => *b"MOMT",
            Self::UvAnimations(_) => *b"MOUV",
            Self::MaterialExtensions(_) => *b"MOMX",
            Self::M3TextureNames(_) => *b"MOTN",
            Self::GroupNames(_) => *b"MOGN",
            Self::Groups(_) => *b"MOGI",
            Self::Groups2(_) => *b"MGI2",
            Self::SkyboxName(_) => *b"MOSB",
            Self::PortalVertices(_) => *b"MOPV",
            Self::Portals(_) => *b"MOPT",
            Self::PortalRefs(_) => *b"MOPR",
            Self::PortalExtras(_) => *b"MOPE",
            Self::VisibleVertices(_) => *b"MOVV",
            Self::VisibleBlocks(_) => *b"MOVB",
            Self::Lights(_) => *b"MOLT",
            Self::LightExtensions(_) => *b"MOLV",
            Self::NewLights(_) => *b"MNLD",
            Self::DoodadSets(_) => *b"MODS",
            Self::DoodadNames(_) => *b"MODN",
            Self::DoodadFileIds(_) => *b"MODI",
            Self::Doodads(_) => *b"MODD",
            Self::DoodadColorMultipliers(_) => *b"MDDI",
            Self::Fogs(_) => *b"MFOG",
            Self::FogBoxes(_) => *b"MFOB",
            Self::FogExtras(_) => *b"MFED",
            Self::GlobalAmbients(_) => *b"MAVG",
            Self::AmbientVolumes(_) => *b"MAVD",
            Self::AmbientBoxes(_) => *b"MBVD",
            Self::ParticulateVolumes(_) => *b"MPVD",
            Self::ConvexPlanes(_) => *b"MCVP",
            Self::GroupFileIds(_) => *b"GFID",
            Self::M3Materials(_) => *b"MOM3",
            Self::DetailDoodads(_) => *b"MDDL",
            Self::Unknown { tag, .. } => *tag,
        }
    }
    /// Encodes the bounded payload without document chunk framing.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        let mut output = Encoder::new();
        match self {
            Self::Header(value) => value.write(&mut output)?,
            Self::SkyboxFileId(value) => value.write(&mut output)?,
            Self::TextureNames(values) => output.bytes(values.as_bytes())?,
            Self::Materials(values) => output.bytes(values.as_bytes())?,
            Self::UvAnimations(values) => output.bytes(values.as_bytes())?,
            Self::MaterialExtensions(values) => output.bytes(values.as_bytes())?,
            Self::M3TextureNames(values) => output.bytes(values.as_bytes())?,
            Self::GroupNames(values) => output.bytes(values.as_bytes())?,
            Self::Groups(values) => output.bytes(values.as_bytes())?,
            Self::Groups2(values) => output.bytes(values.as_bytes())?,
            Self::SkyboxName(values) => output.bytes(values.as_bytes())?,
            Self::PortalVertices(values) => output.bytes(values.as_bytes())?,
            Self::Portals(values) => output.bytes(values.as_bytes())?,
            Self::PortalRefs(values) => output.bytes(values.as_bytes())?,
            Self::PortalExtras(values) => output.bytes(values.as_bytes())?,
            Self::VisibleVertices(values) => output.bytes(values.as_bytes())?,
            Self::VisibleBlocks(values) => output.bytes(values.as_bytes())?,
            Self::Lights(values) => output.bytes(values.as_bytes())?,
            Self::LightExtensions(values) => output.bytes(values.as_bytes())?,
            Self::NewLights(values) => output.bytes(values.as_bytes())?,
            Self::DoodadSets(values) => output.bytes(values.as_bytes())?,
            Self::DoodadNames(values) => output.bytes(values.as_bytes())?,
            Self::DoodadFileIds(values) => output.bytes(values.as_bytes())?,
            Self::Doodads(values) => output.bytes(values.as_bytes())?,
            Self::DoodadColorMultipliers(values) => output.bytes(values.as_bytes())?,
            Self::Fogs(values) => output.bytes(values.as_bytes())?,
            Self::FogBoxes(values) => output.bytes(values.as_bytes())?,
            Self::FogExtras(values) => output.bytes(values.as_bytes())?,
            Self::GlobalAmbients(values) => output.bytes(values.as_bytes())?,
            Self::AmbientVolumes(values) => output.bytes(values.as_bytes())?,
            Self::AmbientBoxes(values) => output.bytes(values.as_bytes())?,
            Self::ParticulateVolumes(values) => output.bytes(values.as_bytes())?,
            Self::ConvexPlanes(values) => output.bytes(values.as_bytes())?,
            Self::GroupFileIds(values) => output.bytes(values.as_bytes())?,
            Self::M3Materials(bytes) => output.bytes(bytes)?,
            Self::DetailDoodads(bytes) => output.bytes(bytes)?,
            Self::Unknown { data, .. } => output.bytes(data)?,
        }
        Ok(output.finish())
    }
}
fn single<'data, T: Read<'data> + RecordLayout>(
    input: &mut Decoder<'data>,
) -> Result<T, ReadError> {
    if input.remaining() != T::size() {
        return Err(ReadError::invalid(
            input.absolute_position(),
            "fixed root chunk width",
        ));
    }
    let result = T::read(input)?;
    if input.remaining() != 0 {
        return Err(ReadError::invalid(
            input.absolute_position(),
            "trailing fixed chunk bytes",
        ));
    }
    Ok(result)
}
fn array<T: RecordLayout>(data: &[u8], origin: usize) -> Result<Array<'_, T>, ReadError> {
    Array::decode(data).map_err(|mut error| {
        error.offset = origin.saturating_add(error.offset);
        error
    })
}
