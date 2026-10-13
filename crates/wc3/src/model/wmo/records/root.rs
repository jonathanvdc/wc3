//! Root-file material, visibility, doodad, lighting, and environmental records.
use super::super as wmo;
use crate::model::wmo::records::{
    AmbientBoxFlags, AmbientFlags, BlendMode, DoodadFlags, DoodadReference, FlickerMode,
    FogBoxFlags, FogFlags, GroupFlags, GroupFlags2, LightFlags, LightKind, MaterialFlags,
    MaterialFlags2, NewLightKind, PortalFlags, RootFlags,
};
use std::fmt::Debug;
use wmo::{Dialect, Read, RecordLayout, Validate, Write, WriteError};

/// Fixed native field types selected by a root dialect.
pub trait RootField:
    for<'data> Read<'data> + Write + RecordLayout + Validate + Clone + Debug + Default + PartialEq
{
}
impl<
        T: for<'data> Read<'data>
            + Write
            + RecordLayout
            + Validate
            + Clone
            + Debug
            + Default
            + PartialEq,
    > RootField for T
{
}
/// Statically selected root-header and material field families.
pub trait RootLayout {
    /// Interpretation of the final four MOHD bytes.
    type HeaderTail: RootField + HeaderTailField;
    /// Interpretation of the final twenty-four MOMT bytes.
    type MaterialTail: RootField + MaterialTailField;
}
/// Common access to native header flags and supported LOD counts.
pub trait HeaderTailField {
    /// Root behavior flags without changing their native encoding.
    fn flags(&self) -> RootFlags;
    /// Number of extra group LOD levels authored in this layout.
    fn lod_count(&self) -> u16;
}
/// MOHD final flags and reserved word before group LOD metadata.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct LegacyHeaderTail {
    /// Root behavior flags.
    pub flags: RootFlags,
    /// Reserved trailing header word before group LOD metadata was introduced.
    pub reserved: u16,
}

/// MOHD final flags and native LOD metadata word.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct ModernHeaderTail {
    /// Root lighting, liquid-type, and group LOD flags.
    pub flags: RootFlags,
    /// Number of group LOD levels including the base; newer clients use the low byte.
    pub lod_count: u16,
}

/// Historical MOMT additional color, flags, and runtime words reused as textures by later shaders.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct LegacyMaterialTail {
    /// Historical additional BGRA material color; modern shaders reuse this word as texture slot 3.
    pub color2: [u8; 4],
    /// Historical secondary material flags; modern shaders reuse this word as texture slot 4.
    pub flags2: MaterialFlags2,
    /// Historical runtime material words; modern shaders reuse them as texture slots 5–8.
    pub runtime: [u32; 4],
}

/// Modern interpretation of the six trailing MOMT texture words.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct ModernMaterialTail {
    /// Additional texture slots 3–8, used by modern shaders 22–24.
    pub textures: [u32; 6],
}

/// MOUV scroll speeds for two material texture layers.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct MaterialUvAnimation {
    /// Texture layer zero UV scroll velocity.
    pub speed0: [f32; 2],
    /// Texture layer one UV scroll velocity.
    pub speed1: [f32; 2],
}

/// MOMX per-material color and texture override key.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct MaterialExtension {
    /// BGRA color used by the shader-23 branch.
    pub color: [u8; 4],
    /// Low/high words of the texture-override-set key, unaligned in the native record.
    pub override_key: [u32; 2],
    /// Trailing word whose meaning is not established.
    pub unknown: u32,
}

/// MOGI group visibility flags, bounds, and name reference.
#[derive(Clone, Debug, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
pub struct GroupInfo {
    /// Group flags corresponding to the group file MOGP flags.
    pub flags: GroupFlags,
    /// Minimum and maximum group bounds in WMO coordinates.
    pub bounds: [[f32; 3]; 2],
    /// Byte offset into MOGN group-name bytes, or -1 for no name.
    pub name_offset: i32,
}

impl Default for GroupInfo {
    fn default() -> Self {
        Self {
            flags: Default::default(),
            bounds: [[0.0; 3]; 2],
            name_offset: -1,
        }
    }
}

/// MGI2 secondary group flags and LOD metadata.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct GroupInfo2 {
    /// Secondary group flags corresponding to the MOGP flags2 field.
    pub flags2: GroupFlags2,
    /// Group LOD index; the latest client reads only its low byte.
    pub lod_index: u32,
}

/// MOPT portal polygon range and separating plane.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct Portal {
    /// Index of the first vertex in the root MOPV array.
    pub start_vertex: u16,
    /// Number of consecutive portal boundary vertices.
    pub vertex_count: u16,
    /// Plane coefficients (nx, ny, nz, distance): dot(normal, point) + distance = 0.
    pub plane: [f32; 4],
}

/// MOPR connection from a portal to the neighboring group.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct PortalRef {
    /// Index into root MOPT portal records.
    pub portal_index: u16,
    /// Group index on the other side of this portal.
    pub group_index: u16,
    /// Signed portal-plane side for this group relationship.
    pub side: i16,
    /// Reference flags: 0x1 skips viewer queries; 0x20 applies the MOPE plane adjustment.
    pub flags: PortalFlags,
}

/// MOPE supplemental portal culling adjustment.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct PortalExtra {
    /// Index of the portal receiving this culling-plane adjustment.
    pub portal_index: u32,
    /// Plane adjustment data whose individual word encodings remain unresolved.
    pub unknown: [u32; 3],
}

/// MOVB run of historical visibility vertices.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct VisibleBlock {
    /// First vertex index into MOVV visibility vertices.
    pub first_vertex: u16,
    /// Number of consecutive MOVV vertices belonging to this block.
    pub count: u16,
}

/// MOLT legacy light with color, transform, intensity, and attenuation.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct Light {
    /// Legacy light kind: 0 omni, 1 spot, 2 directional, 3 ambient.
    pub kind: LightKind,
    /// Whether distance attenuation is enabled.
    pub use_attenuation: u8,
    /// Reserved bytes with no established meaning.
    pub padding: [u8; 2],
    /// Light color in BGRA byte order.
    pub color: [u8; 4],
    /// Light position in WMO coordinates.
    pub position: [f32; 3],
    /// Authored light intensity.
    pub intensity: f32,
    /// Light orientation quaternion.
    pub rotation: [f32; 4],
    /// Distance at which attenuation begins.
    pub attenuation_start: f32,
    /// Distance at which attenuation ends.
    pub attenuation_end: f32,
}

/// MODS named consecutive range of doodad placements.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct DoodadSet {
    /// Fixed-size, zero-terminated doodad-set name, including unused tail bytes.
    pub name: [u8; 20],
    /// Index of the first MODD doodad definition belonging to this set.
    pub start_index: u32,
    /// Number of consecutive doodads in this set.
    pub count: u32,
    /// Reserved word with no established meaning.
    pub padding: u32,
}

/// MODD doodad model reference, transform, flags, and tint.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct DoodadDef {
    /// Low 24 bits are a MODN byte offset or MODI index; high eight bits are doodad flags.
    pub name_and_flags: DoodadReference,
    /// Doodad position in WMO coordinates.
    pub position: [f32; 3],
    /// Doodad orientation quaternion (x, y, z, w).
    pub rotation: [f32; 4],
    /// Uniform doodad scale.
    pub scale: f32,
    /// BGRA doodad tint; with LightFromMolt, the alpha byte indexes MOLT.
    pub color: [u8; 4],
}

/// Above-water or underwater fog color and distance parameters.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct FogParams {
    /// Distance where fog reaches its full effect.
    pub end: f32,
    /// Multiplier of end defining the fog start distance.
    pub start_scalar: f32,
    /// Fog color in BGRA order.
    pub color: [u8; 4],
}

/// MFOG spherical fog region.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct Fog {
    /// Fog behavior flags; 0x1 denotes infinite radius and is skipped by the client query.
    pub flags: FogFlags,
    /// Fog-volume center in WMO coordinates.
    pub position: [f32; 3],
    /// Radius where the fog volume begins blending.
    pub radius_start: f32,
    /// Outer radius of the fog volume.
    pub radius_end: f32,
    /// Above-water fog parameters.
    pub fog: FogParams,
    /// Underwater fog parameters.
    pub underwater: FogParams,
}

/// MFOB six-plane fog region.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct FogBox {
    /// Bounding planes; a point is inside when dot(normal, point) + distance is nonnegative for all six.
    pub planes: [[f32; 4]; 6],
    /// Distance over which the fog volume fades.
    pub fade_distance: f32,
    /// Above-water fog parameters.
    pub fog: FogParams,
    /// Underwater fog parameters.
    pub underwater: FogParams,
    /// Volume flags whose bit meanings remain unresolved.
    pub flags: FogBoxFlags,
    /// Zero applies always; other values restrict the volume to the active doodad set.
    pub doodad_set_id: u16,
    /// Reserved bytes with no established meaning.
    pub padding: [u8; 10],
}

/// MFED activation metadata corresponding to a spherical fog record.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct FogExtra {
    /// Doodad-set restriction for the corresponding MFOG record.
    pub doodad_set_id: u16,
    /// Reserved bytes with no established meaning.
    pub padding: [u8; 14],
}

/// MAVG/MAVD radial ambient-light region.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct AmbientVolume {
    /// Ambient-volume center in WMO coordinates.
    pub position: [f32; 3],
    /// Inner blending distance.
    pub start: f32,
    /// Outer blending distance.
    pub end: f32,
    /// Three BGRA ambient colors; flag 0x1 selects distinct colors rather than repeating the first.
    pub colors: [[u8; 4]; 3],
    /// Ambient-volume flags; 0x1 selects three distinct colors.
    pub flags: AmbientFlags,
    /// Doodad-set restriction for this ambient volume.
    pub doodad_set_id: u16,
    /// Reserved bytes with no established meaning.
    pub padding: [u8; 10],
}

/// MBVD six-plane ambient-light region.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct AmbientBox {
    /// Six bounding-plane equations in WMO coordinates.
    pub planes: [[f32; 4]; 6],
    /// Ambient-volume blending distance.
    pub end: f32,
    /// Three BGRA ambient colors.
    pub colors: [[u8; 4]; 3],
    /// Ambient-volume flags; other meanings remain unresolved.
    pub flags: AmbientBoxFlags,
    /// Doodad-set restriction for this volume.
    pub doodad_set_id: u16,
    /// Reserved bytes with no established meaning.
    pub padding: [u8; 10],
}

/// One directional term in a preserved MOLV light extension.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct LightTerm {
    /// Direction vector stored by the client without an established active use.
    pub direction: [f32; 3],
    /// Term scalar stored by the client without an established active use.
    pub value: f32,
}

/// MOLV extension linked to a legacy light, with unresolved active semantics.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct LightExtension {
    /// Six stored directional terms whose active interpretation remains unresolved.
    pub terms: [LightTerm; 6],
    /// Bytes with no established meaning.
    pub unknown: [u8; 3],
    /// Index into the MOLT light array.
    pub light_index: u8,
}

/// MNLD modern point or spot light, including flicker and cookie texture metadata.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct NewLight {
    /// Modern light kind: 0 point, 1 spot.
    pub kind: NewLightKind,
    /// Authored light index.
    pub light_index: i32,
    /// 0x1 blends inner to outer color; 0x2 enables shadow casting.
    pub flags: LightFlags,
    /// Doodad-set selection for this light.
    pub doodad_set: i32,
    /// Inner light color in BGRA order.
    pub inner_color: [u8; 4],
    /// Light position in WMO coordinates.
    pub position: [f32; 3],
    /// Euler light rotation in radians.
    pub rotation: [f32; 3],
    /// Distance where attenuation begins.
    pub attenuation_start: f32,
    /// Distance where attenuation ends.
    pub attenuation_end: f32,
    /// Authored light intensity.
    pub intensity: f32,
    /// Outer light color in BGRA order.
    pub outer_color: [u8; 4],
    /// Start of the inner-to-outer color blend.
    pub blend_start: f32,
    /// End of the inner-to-outer color blend.
    pub blend_end: f32,
    /// Word with no established meaning.
    pub gap0: u32,
    /// Intensity of light flicker.
    pub flicker_intensity: f32,
    /// Authored flicker speed; time units are not established.
    pub flicker_speed: f32,
    /// Flicker selector: 0 off, 1 sine, 2 noise, 3 stepped noise.
    pub flicker_mode: FlickerMode,
    /// Vector at offset 0x54 whose purpose remains unresolved.
    pub unknown_vector: [f32; 3],
    /// Word with no established meaning.
    pub gap1: u32,
    /// Cookie texture FileDataID; point lights use a cube map.
    pub cookie_file_id: u32,
    /// Five words with no established meanings.
    pub gap2: [u32; 5],
    /// Authored light falloff parameter.
    pub falloff: f32,
    /// Inner spotlight angle; encoded angle units are not established.
    pub inner_angle: f32,
    /// Outer spotlight angle; encoded angle units are not established.
    pub outer_angle: f32,
    /// Light scale stored as IEEE half-float bits.
    pub scale_half: u16,
    /// Intensity multiplier as half-float bits; zero is interpreted as one by the client.
    pub intensity_multiplier_half: u16,
    /// Trailing words with no established active use.
    pub unused: [u32; 11],
}

impl HeaderTailField for LegacyHeaderTail {
    fn flags(&self) -> RootFlags {
        self.flags
    }
    fn lod_count(&self) -> u16 {
        0
    }
}
impl HeaderTailField for ModernHeaderTail {
    fn flags(&self) -> RootFlags {
        self.flags
    }
    fn lod_count(&self) -> u16 {
        self.lod_count
    }
}
/// MOHD root counts, WMO identity, bounds, and dialect-selected tail.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct Header<D: Dialect> {
    /// Number of materials/textures declared by the exporter.
    pub texture_count: u32,
    /// Number of groups; modern GFID uses this as the stride of each LOD row.
    pub group_count: u32,
    /// Number of portal records.
    pub portal_count: u32,
    /// Number of legacy lights.
    pub light_count: u32,
    /// Number of doodad names declared by the exporter.
    pub doodad_name_count: u32,
    /// Number of placed doodad definitions.
    pub doodad_def_count: u32,
    /// Number of doodad sets.
    pub doodad_set_count: u32,
    /// Ambient color in BGRA order.
    pub ambient_color: [u8; 4],
    /// WMOAreaTable WMOID associated with this model.
    pub wmo_id: u32,
    /// Minimum and maximum bounds in WMO coordinates.
    pub bounds: [[f32; 3]; 2],
    /// Native header flags and any LOD bytes present in this dialect.
    pub tail: <D as RootLayout>::HeaderTail,
}
impl<D: Dialect> Header<D> {
    /// Root flags independent of the native field width.
    pub fn flags(&self) -> RootFlags {
        self.tail.flags()
    }
    /// Authored LOD count, or zero in layouts without LOD metadata.
    pub fn lod_count(&self) -> u16 {
        self.tail.lod_count()
    }
}
/// MOMT material definition with a dialect-selected trailing field family.
#[derive(
    Clone, Debug, Default, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate,
)]
pub struct Material<D: Dialect> {
    /// Material behavior flags such as unlit, unfogged, two-sided, and UV clamp.
    pub flags: MaterialFlags,
    /// Client shader-table index.
    pub shader: u32,
    /// Blend mode: zero opaque, one alpha key, larger values blended.
    pub blend_mode: BlendMode,
    /// First texture, as a MOTX byte offset or FileDataID depending on name-table presence.
    pub texture0: u32,
    /// Self-illumination color in BGRA order, used with the Sidn flag.
    pub sidn_color: [u8; 4],
    /// Serialized runtime self-illumination color with no established active use.
    pub frame_sidn_color: [u8; 4],
    /// Second texture, as a MOTX byte offset or FileDataID.
    pub texture1: u32,
    /// Diffuse BGRA color; also supplies the liquid color for interior groups.
    pub diffuse_color: [u8; 4],
    /// TerrainType value associated with this material.
    pub ground_type: u32,
    /// Third texture, as a MOTX byte offset or FileDataID.
    pub texture2: u32,
    /// Historical additional color/flags or modern texture slots 3–8.
    pub tail: D::MaterialTail,
}

macro_rules! layouts{($tail:ty;$($dialect:ty),*$(,)?)=>{$(impl RootLayout for $dialect{type HeaderTail=$tail;type MaterialTail=LegacyMaterialTail;})*};}
layouts!(LegacyHeaderTail;wmo::Classic);
layouts!(ModernHeaderTail;wmo::Legion,wmo::Bfa,wmo::ShadowlandsPre92,wmo::Shadowlands,wmo::Dragonflight,wmo::WarWithin);

/// Common texture-slot access without changing native material fields.
pub trait MaterialTailField {
    /// Texture slots 3–8 in the modern shader interpretation.
    fn texture_slots(&self) -> [u32; 6];
    /// Replaces one of the six trailing material words without changing its native encoding.
    fn set_texture(&mut self, index: usize, value: u32) -> Result<(), WriteError>;
}
impl MaterialTailField for LegacyMaterialTail {
    fn texture_slots(&self) -> [u32; 6] {
        [
            u32::from_le_bytes(self.color2),
            self.flags2.bits(),
            self.runtime[0],
            self.runtime[1],
            self.runtime[2],
            self.runtime[3],
        ]
    }
    fn set_texture(&mut self, index: usize, value: u32) -> Result<(), WriteError> {
        match index {
            0 => self.color2 = value.to_le_bytes(),
            1 => self.flags2 = value.into(),
            2..=5 => self.runtime[index - 2] = value,
            _ => return Err(WriteError::invalid("material texture slot")),
        }
        Ok(())
    }
}
impl MaterialTailField for ModernMaterialTail {
    fn texture_slots(&self) -> [u32; 6] {
        self.textures
    }
    fn set_texture(&mut self, index: usize, value: u32) -> Result<(), WriteError> {
        *self
            .textures
            .get_mut(index)
            .ok_or_else(|| WriteError::invalid("material texture slot"))? = value;
        Ok(())
    }
}
impl<D: Dialect> Material<D> {
    /// Number of texture words used by the selected shader: nine for shaders 22–24, three otherwise.
    pub fn texture_slots(&self) -> usize {
        if matches!(self.shader, 22..=24) {
            9
        } else {
            3
        }
    }
    /// Replaces a native texture word, including the trailing words reused by newer shaders.
    pub fn set_texture(&mut self, slot: usize, value: u32) -> Result<(), WriteError> {
        match slot {
            0 => self.texture0 = value,
            1 => self.texture1 = value,
            2 => self.texture2 = value,
            3..=8 => self.tail.set_texture(slot - 3, value)?,
            _ => return Err(WriteError::invalid("material texture slot")),
        }
        Ok(())
    }

    /// All nine material texture words in client slot order.
    ///
    /// MOTX presence selects name offsets. Later shaders can interpret the six
    /// trailing historical color/runtime words as extra texture slots.
    pub fn textures(&self) -> [u32; 9] {
        let slots = self.tail.texture_slots();
        [
            self.texture0,
            self.texture1,
            self.texture2,
            slots[0],
            slots[1],
            slots[2],
            slots[3],
            slots[4],
            slots[5],
        ]
    }
}
impl DoodadDef {
    /// MODN byte offset or MODI index encoded in the low 24 bits.
    pub fn name_index(&self) -> u32 {
        self.name_and_flags.name_index()
    }
    /// Doodad behavior flags encoded in the highest byte.
    pub fn flags(&self) -> DoodadFlags {
        self.name_and_flags.flags()
    }
}
