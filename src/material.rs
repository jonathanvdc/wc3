//! Typed material layers and versioned texture slots.

use crate::Record;
use std::borrow::Cow;

use crate::utils::field;
use crate::{AnimationTrack, Error, Model};

const LAYER_TAG: [u8; 4] = *b"LAYS";

/// Material rendering bits, preserving unrecognized bits.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MaterialRenderFlags(u32);

impl MaterialRenderFlags {
    pub const CONSTANT_COLOR: Self = Self(1);
    pub const SORT_PRIMITIVES_FAR_Z: Self = Self(16);
    pub const FULL_RESOLUTION: Self = Self(32);
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }
    pub const fn bits(self) -> u32 {
        self.0
    }
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
    pub fn set(&mut self, other: Self, enabled: bool) {
        if enabled {
            self.0 |= other.0;
        } else {
            self.0 &= !other.0;
        }
    }
}

/// Material layer shading bits, preserving unrecognized bits.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LayerShadingFlags(u32);

impl LayerShadingFlags {
    pub const UNSHADED: Self = Self(1);
    pub const SPHERE_ENV_MAP: Self = Self(2);
    pub const TWO_SIDED: Self = Self(16);
    pub const UNFOGGED: Self = Self(32);
    pub const NO_DEPTH_TEST: Self = Self(64);
    pub const NO_DEPTH_SET: Self = Self(128);
    pub const UNLIT: Self = Self(256);
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }
    pub const fn bits(self) -> u32 {
        self.0
    }
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
    pub fn set(&mut self, other: Self, enabled: bool) {
        if enabled {
            self.0 |= other.0;
        } else {
            self.0 &= !other.0;
        }
    }
}

/// A Reforged layer texture slot, optionally animated by `KMTF`.
#[derive(Clone, Debug, PartialEq)]
pub struct LayerTextureSlot {
    pub texture_id: u32,
    pub texture_type: u32,
    pub track: Option<AnimationTrack>,
}

/// A material with directly accessible layers and an exact shader field.
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    version: u32,
    priority_plane: u32,
    render_mode: u32,
    shader: Option<[u8; 80]>,
    layers: Vec<Layer>,
}

/// A material layer with parsed texture slots and animation tracks.
#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    version: u32,
    filter_mode: u32,
    shading_flags: u32,
    texture_id: u32,
    texture_animation_id: u32,
    coordinate_id: u32,
    alpha: f32,
    emissive_gain: Option<f32>,
    fresnel_color: Option<[f32; 3]>,
    fresnel_opacity: Option<f32>,
    fresnel_team_color: Option<f32>,
    shader_type_id: Option<u32>,
    texture_slots: Vec<LayerTextureSlot>,
    tracks: Vec<AnimationTrack>,
}

fn has_shader(version: u32) -> bool {
    (900..1100).contains(&version)
}
fn is_layer_track(tag: [u8; 4]) -> bool {
    matches!(
        &tag,
        b"KMTA" | b"KMTF" | b"KMTE" | b"KFC3" | b"KFCA" | b"KFTC"
    )
}

fn expect_tag(
    cursor: &mut crate::cursor::Cursor<'_>,
    expected: [u8; 4],
    record_tag: [u8; 4],
) -> Result<(), Error> {
    let offset = cursor.absolute_position();
    if cursor.read_exact(4)? == expected {
        Ok(())
    } else {
        Err(Error::MalformedRecord {
            tag: record_tag,
            offset,
        })
    }
}

fn write_count(bytes: &mut Vec<u8>, count: usize, tag: [u8; 4]) -> Result<(), Error> {
    let value = u32::try_from(count).map_err(|_| Error::ChunkTooLarge { tag, size: count })?;
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}
fn finish_record(bytes: &mut [u8], tag: [u8; 4]) -> Result<(), Error> {
    let size = u32::try_from(bytes.len()).map_err(|_| Error::ChunkTooLarge {
        tag,
        size: bytes.len(),
    })?;
    bytes[..4].copy_from_slice(&size.to_le_bytes());
    Ok(())
}

impl Material {
    /// Creates an empty material for the given MDX version.
    pub fn new(version: u32) -> Self {
        Self {
            version,
            priority_plane: 0,
            render_mode: 0,
            shader: has_shader(version).then_some([0; 80]),
            layers: Vec::new(),
        }
    }

    /// Returns the MDX version used for this material.
    pub fn version(&self) -> u32 {
        self.version
    }
    /// Returns the material priority plane.
    pub fn priority_plane(&self) -> u32 {
        self.priority_plane
    }
    /// Changes the material priority plane.
    pub fn set_priority_plane(&mut self, value: u32) {
        self.priority_plane = value;
    }
    /// Returns decoded rendering flags.
    pub fn render_mode(&self) -> MaterialRenderFlags {
        MaterialRenderFlags::from_bits(self.render_mode)
    }
    /// Returns exact rendering bits.
    pub fn raw_render_mode(&self) -> u32 {
        self.render_mode
    }
    /// Changes the rendering flags.
    pub fn set_render_mode(&mut self, value: MaterialRenderFlags) {
        self.render_mode = value.bits();
    }
    /// Changes exact rendering bits.
    pub fn set_raw_render_mode(&mut self, value: u32) {
        self.render_mode = value;
    }
    /// Returns the shader path in versions 900 through 1099.
    pub fn shader(&self) -> Option<Cow<'_, str>> {
        self.shader.as_ref().map(|field| field::text(field))
    }
    /// Changes the shader path and clears unused bytes.
    pub fn set_shader(&mut self, shader: &str) -> Result<(), Error> {
        let field = self.shader.as_mut().ok_or(Error::MalformedRecord {
            tag: Material::TAG,
            offset: 12,
        })?;
        field::set_text(field, shader)
    }
    /// Borrows layers without decoding or allocating.
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }
    /// Mutably borrows layers for bulk edits.
    pub fn layers_mut(&mut self) -> &mut [Layer] {
        &mut self.layers
    }
    /// Replaces all layers, rejecting a different MDX version.
    pub fn set_layers(&mut self, layers: &[Layer]) -> Result<(), Error> {
        if let Some(layer) = layers.iter().find(|layer| layer.version != self.version) {
            return Err(Error::VersionMismatch {
                expected: self.version,
                actual: layer.version,
            });
        }
        if layers.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: Material::TAG,
                size: layers.len(),
            });
        }
        self.layers = layers.to_vec();
        Ok(())
    }
}

impl Layer {
    /// Creates an empty layer with version-appropriate fields.
    pub fn new(version: u32) -> Self {
        Self {
            version,
            filter_mode: 0,
            shading_flags: 0,
            texture_id: 0,
            texture_animation_id: u32::MAX,
            coordinate_id: 0,
            alpha: 1.0,
            emissive_gain: (version >= 900).then_some(1.0),
            fresnel_color: (version >= 1000).then_some([1.0; 3]),
            fresnel_opacity: (version >= 1000).then_some(0.0),
            fresnel_team_color: (version >= 1000).then_some(0.0),
            shader_type_id: (version >= 1100).then_some(0),
            texture_slots: Vec::new(),
            tracks: Vec::new(),
        }
    }

    /// Returns the MDX version used for this layer.
    pub fn version(&self) -> u32 {
        self.version
    }
    /// Returns the blend filter mode.
    pub fn filter_mode(&self) -> u32 {
        self.filter_mode
    }
    /// Changes the blend filter mode.
    pub fn set_filter_mode(&mut self, mode: u32) {
        self.filter_mode = mode;
    }
    /// Returns decoded layer shading bits.
    pub fn shading_flags(&self) -> LayerShadingFlags {
        LayerShadingFlags::from_bits(self.shading_flags)
    }
    /// Returns exact layer shading bits.
    pub fn raw_shading_flags(&self) -> u32 {
        self.shading_flags
    }
    /// Changes decoded layer shading bits.
    pub fn set_shading_flags(&mut self, flags: LayerShadingFlags) {
        self.shading_flags = flags.bits();
    }
    /// Changes exact layer shading bits.
    pub fn set_raw_shading_flags(&mut self, flags: u32) {
        self.shading_flags = flags;
    }
    /// Returns the base texture index.
    pub fn texture_id(&self) -> u32 {
        self.texture_id
    }
    /// Changes the base texture index.
    pub fn set_texture_id(&mut self, id: u32) {
        self.texture_id = id;
    }
    /// Returns the texture animation reference.
    pub fn texture_animation_id(&self) -> u32 {
        self.texture_animation_id
    }
    /// Changes the texture animation reference.
    pub fn set_texture_animation_id(&mut self, id: u32) {
        self.texture_animation_id = id;
    }
    /// Returns the texture coordinate set index.
    pub fn coordinate_id(&self) -> u32 {
        self.coordinate_id
    }
    /// Changes the texture coordinate set index.
    pub fn set_coordinate_id(&mut self, id: u32) {
        self.coordinate_id = id;
    }
    /// Returns the base alpha value.
    pub fn alpha(&self) -> f32 {
        self.alpha
    }
    /// Changes the base alpha value.
    pub fn set_alpha(&mut self, value: f32) {
        self.alpha = value;
    }
    /// Returns emissive gain in versions 900 and later.
    pub fn emissive_gain(&self) -> Option<f32> {
        self.emissive_gain
    }
    /// Changes emissive gain in versions 900 and later.
    pub fn set_emissive_gain(&mut self, value: f32) -> Result<(), Error> {
        self.require_version(900, 28)?;
        self.emissive_gain = Some(value);
        Ok(())
    }
    /// Returns the Fresnel color in versions 1000 and later.
    pub fn fresnel_color(&self) -> Option<[f32; 3]> {
        self.fresnel_color
    }
    /// Changes the Fresnel color in versions 1000 and later.
    pub fn set_fresnel_color(&mut self, value: [f32; 3]) -> Result<(), Error> {
        self.require_version(1000, 32)?;
        self.fresnel_color = Some(value);
        Ok(())
    }
    /// Returns Fresnel opacity in versions 1000 and later.
    pub fn fresnel_opacity(&self) -> Option<f32> {
        self.fresnel_opacity
    }
    /// Changes Fresnel opacity in versions 1000 and later.
    pub fn set_fresnel_opacity(&mut self, value: f32) -> Result<(), Error> {
        self.require_version(1000, 44)?;
        self.fresnel_opacity = Some(value);
        Ok(())
    }
    /// Returns Fresnel team-color strength in versions 1000 and later.
    pub fn fresnel_team_color(&self) -> Option<f32> {
        self.fresnel_team_color
    }
    /// Changes Fresnel team-color strength in versions 1000 and later.
    pub fn set_fresnel_team_color(&mut self, value: f32) -> Result<(), Error> {
        self.require_version(1000, 48)?;
        self.fresnel_team_color = Some(value);
        Ok(())
    }
    /// Returns shader type ID in versions 1100 and later.
    pub fn shader_type_id(&self) -> Option<u32> {
        self.shader_type_id
    }
    /// Changes shader type ID in versions 1100 and later.
    pub fn set_shader_type_id(&mut self, value: u32) -> Result<(), Error> {
        self.require_version(1100, 52)?;
        self.shader_type_id = Some(value);
        Ok(())
    }
    /// Borrows Reforged texture slots and their optional tracks.
    pub fn texture_slots(&self) -> &[LayerTextureSlot] {
        &self.texture_slots
    }
    /// Replaces Reforged texture slots after validating their tracks.
    pub fn set_texture_slots(&mut self, slots: &[LayerTextureSlot]) -> Result<(), Error> {
        self.require_version(1100, 56)?;
        if slots.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: LAYER_TAG,
                size: slots.len(),
            });
        }
        for slot in slots {
            if let Some(track) = &slot.track {
                if track.tag != *b"KMTF" {
                    return Err(Error::MalformedRecord {
                        tag: LAYER_TAG,
                        offset: 0,
                    });
                }
                track.encode()?;
            }
        }
        self.texture_slots = slots.to_vec();
        Ok(())
    }
    /// Borrows layer animation tracks after any texture slots.
    pub fn tracks(&self) -> &[AnimationTrack] {
        &self.tracks
    }
    /// Replaces layer animation tracks.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), Error> {
        for track in tracks {
            if !is_layer_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: LAYER_TAG,
                    offset: 0,
                });
            }
            track.encode()?;
        }
        self.tracks = tracks.to_vec();
        Ok(())
    }
    fn require_version(&self, minimum: u32, offset: usize) -> Result<(), Error> {
        if self.version < minimum {
            Err(Error::MalformedRecord {
                tag: LAYER_TAG,
                offset,
            })
        } else {
            Ok(())
        }
    }
}

impl Model {
    /// Decodes all `MTLS` records in file order.
    pub fn materials(&self) -> Result<Vec<Material>, Error> {
        self.collect_chunk_records::<crate::MaterialsChunk>(|chunk| match chunk {
            crate::ModelChunk::Materials(decoded) => Some(decoded),
            _ => None,
        })
    }

    /// Replaces all material records in the first `MTLS` chunk.
    pub fn set_materials(&mut self, materials: &[Material]) -> Result<(), Error> {
        let expected = self.version();
        for material in materials {
            if material.version != expected {
                return Err(Error::VersionMismatch {
                    expected,
                    actual: material.version,
                });
            }
        }
        self.replace_chunk(crate::ModelChunk::Materials(crate::MaterialsChunk::new(
            materials.to_vec(),
        )))
    }
}

impl Record for Material {
    fn decode_one(source: &mut crate::Cursor<'_>, version: u32) -> Result<Self, Error> {
        let mut cursor = source.slice_u32_sized()?;
        let value = {
            let priority_plane = cursor.read_u32()?;
            let render_mode = cursor.read_u32()?;
            let shader = if has_shader(version) {
                Some(cursor.read_exact(80)?.try_into().expect("shader field"))
            } else {
                None
            };
            expect_tag(&mut cursor, LAYER_TAG, Material::TAG)?;
            let count = cursor.read_u32()? as usize;
            let mut layers = Vec::new();
            for _ in 0..count {
                let layer = Layer::decode_one(&mut cursor, version)?;
                layers.push(layer);
            }
            Ok(Self {
                version,
                priority_plane,
                render_mode,
                shader,
                layers,
            })
        }?;
        cursor.finish()?;
        Ok(value)
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![0; 4];
        bytes.extend_from_slice(&self.priority_plane.to_le_bytes());
        bytes.extend_from_slice(&self.render_mode.to_le_bytes());
        if has_shader(self.version) {
            bytes.extend_from_slice(self.shader.as_ref().unwrap_or(&[0; 80]));
        }
        bytes.extend_from_slice(b"LAYS");
        write_count(&mut bytes, self.layers.len(), Material::TAG)?;
        for layer in &self.layers {
            if layer.version != self.version {
                return Err(Error::VersionMismatch {
                    expected: self.version,
                    actual: layer.version,
                });
            }
            bytes.extend_from_slice(&layer.encode()?);
        }
        finish_record(&mut bytes, Material::TAG)?;
        Ok(bytes)
    }
}

impl Record for Layer {
    fn decode_one(source: &mut crate::Cursor<'_>, version: u32) -> Result<Self, Error> {
        let mut cursor = source.slice_u32_sized()?;
        let value = {
            let filter_mode = cursor.read_u32()?;
            let shading_flags = cursor.read_u32()?;
            let texture_id = cursor.read_u32()?;
            let texture_animation_id = cursor.read_u32()?;
            let coordinate_id = cursor.read_u32()?;
            let alpha = cursor.read_f32()?;
            let emissive_gain = if version >= 900 {
                Some(cursor.read_f32()?)
            } else {
                None
            };
            let (fresnel_color, fresnel_opacity, fresnel_team_color) = if version >= 1000 {
                (
                    Some(cursor.read_vec3()?),
                    Some(cursor.read_f32()?),
                    Some(cursor.read_f32()?),
                )
            } else {
                (None, None, None)
            };
            let shader_type_id = if version >= 1100 {
                Some(cursor.read_u32()?)
            } else {
                None
            };
            let mut texture_slots = Vec::new();
            if version >= 1100 {
                let count = cursor.read_u32()? as usize;
                for _ in 0..count {
                    let texture_id = cursor.read_u32()?;
                    let texture_type = cursor.read_u32()?;
                    let track = if cursor.remaining().get(..4) == Some(b"KMTF") {
                        Some(AnimationTrack::decode_one(&mut cursor, version)?)
                    } else {
                        None
                    };
                    texture_slots.push(LayerTextureSlot {
                        texture_id,
                        texture_type,
                        track,
                    });
                }
            }
            let mut tracks = Vec::new();
            while !cursor.remaining().is_empty() {
                let offset = cursor.absolute_position();
                let track = AnimationTrack::decode_one(&mut cursor, version)?;
                if !is_layer_track(track.tag) {
                    return Err(Error::MalformedRecord {
                        tag: LAYER_TAG,
                        offset,
                    });
                }
                tracks.push(track);
            }
            Ok(Self {
                version,
                filter_mode,
                shading_flags,
                texture_id,
                texture_animation_id,
                coordinate_id,
                alpha,
                emissive_gain,
                fresnel_color,
                fresnel_opacity,
                fresnel_team_color,
                shader_type_id,
                texture_slots,
                tracks,
            })
        }?;
        cursor.finish()?;
        Ok(value)
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![0; 4];
        for word in [
            self.filter_mode,
            self.shading_flags,
            self.texture_id,
            self.texture_animation_id,
            self.coordinate_id,
        ] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes.extend_from_slice(&self.alpha.to_le_bytes());
        if self.version >= 900 {
            bytes.extend_from_slice(&self.emissive_gain.unwrap_or_default().to_le_bytes());
        }
        if self.version >= 1000 {
            for value in self.fresnel_color.unwrap_or_default() {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            bytes.extend_from_slice(&self.fresnel_opacity.unwrap_or_default().to_le_bytes());
            bytes.extend_from_slice(&self.fresnel_team_color.unwrap_or_default().to_le_bytes());
        }
        if self.version >= 1100 {
            bytes.extend_from_slice(&self.shader_type_id.unwrap_or_default().to_le_bytes());
            write_count(&mut bytes, self.texture_slots.len(), LAYER_TAG)?;
            for slot in &self.texture_slots {
                bytes.extend_from_slice(&slot.texture_id.to_le_bytes());
                bytes.extend_from_slice(&slot.texture_type.to_le_bytes());
                if let Some(track) = &slot.track {
                    if track.tag != *b"KMTF" {
                        return Err(Error::MalformedRecord {
                            tag: LAYER_TAG,
                            offset: bytes.len(),
                        });
                    }
                    bytes.extend_from_slice(&track.encode()?);
                }
            }
        }
        for track in &self.tracks {
            if !is_layer_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: LAYER_TAG,
                    offset: bytes.len(),
                });
            }
            bytes.extend_from_slice(&track.encode()?);
        }
        finish_record(&mut bytes, LAYER_TAG)?;
        Ok(bytes)
    }
}

impl Material {
    /// The tag of the chunk containing this record.
    pub const TAG: [u8; 4] = *b"MTLS";
}
