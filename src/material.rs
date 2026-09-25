//! Size-bounded material and layer records.

use crate::{AnimationTrack, Error, Model};

const TAG: [u8; 4] = *b"MTLS";

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

/// A material record, including all version-specific bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Material {
    bytes: Vec<u8>,
}

/// A material layer record, including animation tracks and extensions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Layer {
    bytes: Vec<u8>,
}

/// A Reforged layer texture slot, optionally animated by `KMTF`.
#[derive(Clone, Debug, PartialEq)]
pub struct LayerTextureSlot {
    /// Static texture index when no track is present.
    pub texture_id: u32,
    /// Texture purpose identifier stored alongside the index.
    pub texture_type: u32,
    /// Optional integer texture-index animation.
    pub track: Option<AnimationTrack>,
}

fn fixed_size(version: u32) -> usize {
    if version >= 1100 {
        60
    } else if version >= 1000 {
        52
    } else if version >= 900 {
        32
    } else {
        28
    }
}

fn is_layer_track(tag: [u8; 4]) -> bool {
    matches!(
        &tag,
        b"KMTA" | b"KMTF" | b"KMTE" | b"KFC3" | b"KFCA" | b"KFTC"
    )
}

fn sized_records(data: &[u8], tag: [u8; 4]) -> Result<Vec<&[u8]>, Error> {
    let mut records = Vec::new();
    let mut offset = 0;
    while offset < data.len() {
        let size_bytes = data
            .get(offset..offset.saturating_add(4))
            .ok_or(Error::MalformedRecord { tag, offset })?;
        let size = u32::from_le_bytes(size_bytes.try_into().expect("four-byte size")) as usize;
        if size < 4 {
            return Err(Error::MalformedRecord { tag, offset });
        }
        let end = offset
            .checked_add(size)
            .filter(|&end| end <= data.len())
            .ok_or(Error::MalformedRecord { tag, offset })?;
        records.push(&data[offset..end]);
        offset = end;
    }
    Ok(records)
}

impl Material {
    /// Creates an empty material for the selected format version.
    pub fn new(version: u32) -> Self {
        let header = if (900..1100).contains(&version) {
            92
        } else {
            12
        };
        let mut bytes = vec![0; header + 8];
        bytes[..4].copy_from_slice(&((header + 8) as u32).to_le_bytes());
        bytes[header..header + 4].copy_from_slice(b"LAYS");
        Self { bytes }
    }

    /// Wraps one inclusive-size material record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if sized_records(bytes, TAG)?.len() != 1 {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Returns the complete record, including its size field.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the material priority plane.
    pub fn priority_plane(&self) -> Result<u32, Error> {
        self.u32_at(4)
    }

    /// Sets the material priority plane.
    pub fn set_priority_plane(&mut self, value: u32) -> Result<(), Error> {
        self.set_u32_at(4, value)
    }

    /// Returns decoded material render flags.
    pub fn render_mode(&self) -> Result<MaterialRenderFlags, Error> {
        self.raw_render_mode().map(MaterialRenderFlags::from_bits)
    }

    /// Returns exact raw render mode bits.
    pub fn raw_render_mode(&self) -> Result<u32, Error> {
        self.u32_at(8)
    }

    /// Sets decoded material render flags.
    pub fn set_render_mode(&mut self, value: MaterialRenderFlags) -> Result<(), Error> {
        self.set_raw_render_mode(value.bits())
    }

    /// Sets exact raw render mode bits.
    pub fn set_raw_render_mode(&mut self, value: u32) -> Result<(), Error> {
        self.set_u32_at(8, value)
    }

    /// Returns the bounded layer records. Versions 900 through 1099 have an
    /// additional 80-byte shader field before `LAYS`.
    pub fn layers(&self, version: u32) -> Result<Vec<Layer>, Error> {
        let offset = if (900..1100).contains(&version) {
            92
        } else {
            12
        };
        let header = self
            .bytes
            .get(offset..offset + 8)
            .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
        if &header[..4] != b"LAYS" {
            return Err(Error::MalformedRecord { tag: TAG, offset });
        }
        let count = u32::from_le_bytes(header[4..8].try_into().expect("four-byte count")) as usize;
        let records = sized_records(&self.bytes[offset + 8..], *b"LAYS")?;
        if records.len() != count {
            return Err(Error::MalformedRecord { tag: TAG, offset });
        }
        Ok(records
            .into_iter()
            .map(|bytes| Layer {
                bytes: bytes.to_vec(),
            })
            .collect())
    }

    /// Replaces the material's layer list while retaining priority, flags, and shader bytes.
    pub fn set_layers(&mut self, version: u32, layers: &[Layer]) -> Result<(), Error> {
        let offset = if (900..1100).contains(&version) {
            92
        } else {
            12
        };
        if self.bytes.get(offset..offset + 4) != Some(b"LAYS") {
            return Err(Error::MalformedRecord { tag: TAG, offset });
        }
        if layers.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: TAG,
                size: layers.len(),
            });
        }
        let mut bytes = self.bytes[..offset + 4].to_vec();
        bytes.extend_from_slice(&(layers.len() as u32).to_le_bytes());
        for layer in layers {
            bytes.extend_from_slice(layer.as_bytes());
        }
        if bytes.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: TAG,
                size: bytes.len(),
            });
        }
        let size = bytes.len() as u32;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        self.bytes = bytes;
        Ok(())
    }

    fn u32_at(&self, offset: usize) -> Result<u32, Error> {
        let bytes = self
            .bytes
            .get(offset..offset + 4)
            .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
        Ok(u32::from_le_bytes(
            bytes.try_into().expect("four-byte field"),
        ))
    }

    fn set_u32_at(&mut self, offset: usize, value: u32) -> Result<(), Error> {
        let bytes = self
            .bytes
            .get_mut(offset..offset + 4)
            .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
        bytes.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }
}

impl Layer {
    /// Creates an empty layer with version-appropriate fixed fields.
    pub fn new(version: u32) -> Self {
        let size = fixed_size(version);
        let mut bytes = vec![0; size];
        bytes[..4].copy_from_slice(&(size as u32).to_le_bytes());
        bytes[16..20].copy_from_slice(&u32::MAX.to_le_bytes());
        bytes[24..28].copy_from_slice(&1.0f32.to_le_bytes());
        if version >= 900 {
            bytes[28..32].copy_from_slice(&1.0f32.to_le_bytes());
        }
        if version >= 1000 {
            for offset in [32, 36, 40] {
                bytes[offset..offset + 4].copy_from_slice(&1.0f32.to_le_bytes());
            }
        }
        Self { bytes }
    }

    /// Wraps one inclusive-size layer record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if sized_records(bytes, *b"LAYS")?.len() != 1 {
            return Err(Error::MalformedRecord {
                tag: *b"LAYS",
                offset: 0,
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Returns the complete layer record.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the filter mode.
    pub fn filter_mode(&self) -> Result<u32, Error> {
        self.u32_at(4)
    }

    /// Returns decoded layer shading flags.
    pub fn shading_flags(&self) -> Result<LayerShadingFlags, Error> {
        self.raw_shading_flags().map(LayerShadingFlags::from_bits)
    }

    /// Returns exact raw layer shading bits.
    pub fn raw_shading_flags(&self) -> Result<u32, Error> {
        self.u32_at(8)
    }

    /// Sets decoded layer shading flags.
    pub fn set_shading_flags(&mut self, flags: LayerShadingFlags) -> Result<(), Error> {
        self.set_raw_shading_flags(flags.bits())
    }

    /// Sets exact raw layer shading bits.
    pub fn set_raw_shading_flags(&mut self, flags: u32) -> Result<(), Error> {
        self.set_u32_at(8, flags)
    }

    /// Returns the texture index.
    pub fn texture_id(&self) -> Result<u32, Error> {
        self.u32_at(12)
    }

    /// Sets the texture index.
    pub fn set_texture_id(&mut self, id: u32) -> Result<(), Error> {
        self.set_u32_at(12, id)
    }

    /// Returns the base alpha value.
    pub fn alpha(&self) -> Result<f32, Error> {
        Ok(f32::from_bits(self.u32_at(24)?))
    }

    /// Returns the texture animation reference, or `u32::MAX` when absent.
    pub fn texture_animation_id(&self) -> Result<u32, Error> {
        self.u32_at(16)
    }

    /// Sets the texture animation reference.
    pub fn set_texture_animation_id(&mut self, id: u32) -> Result<(), Error> {
        self.set_u32_at(16, id)
    }

    /// Returns the texture coordinate set index.
    pub fn coordinate_id(&self) -> Result<u32, Error> {
        self.u32_at(20)
    }

    /// Sets the texture coordinate set index.
    pub fn set_coordinate_id(&mut self, id: u32) -> Result<(), Error> {
        self.set_u32_at(20, id)
    }

    /// Returns the emissive gain present since version 900.
    pub fn emissive_gain(&self) -> Result<f32, Error> {
        Ok(f32::from_bits(self.u32_at(28)?))
    }

    /// Sets emissive gain in a version 900 or later layer.
    pub fn set_emissive_gain(&mut self, gain: f32) -> Result<(), Error> {
        self.set_u32_at(28, gain.to_bits())
    }

    /// Returns the Fresnel color present since version 1000.
    pub fn fresnel_color(&self) -> Result<[f32; 3], Error> {
        Ok([
            f32::from_bits(self.u32_at(32)?),
            f32::from_bits(self.u32_at(36)?),
            f32::from_bits(self.u32_at(40)?),
        ])
    }

    /// Sets the Fresnel color in a version 1000 or later layer.
    pub fn set_fresnel_color(&mut self, color: [f32; 3]) -> Result<(), Error> {
        for (i, value) in color.into_iter().enumerate() {
            self.set_u32_at(32 + i * 4, value.to_bits())?;
        }
        Ok(())
    }

    /// Returns Fresnel opacity present since version 1000.
    pub fn fresnel_opacity(&self) -> Result<f32, Error> {
        Ok(f32::from_bits(self.u32_at(44)?))
    }

    /// Sets Fresnel opacity.
    pub fn set_fresnel_opacity(&mut self, value: f32) -> Result<(), Error> {
        self.set_u32_at(44, value.to_bits())
    }

    /// Returns Fresnel team-color strength present since version 1000.
    pub fn fresnel_team_color(&self) -> Result<f32, Error> {
        Ok(f32::from_bits(self.u32_at(48)?))
    }

    /// Sets Fresnel team-color strength.
    pub fn set_fresnel_team_color(&mut self, value: f32) -> Result<(), Error> {
        self.set_u32_at(48, value.to_bits())
    }

    /// Returns shader type present since version 1100.
    pub fn shader_type_id(&self) -> Result<u32, Error> {
        self.u32_at(52)
    }

    /// Sets shader type in a version 1100 or later layer.
    pub fn set_shader_type_id(&mut self, id: u32) -> Result<(), Error> {
        self.set_u32_at(52, id)
    }

    /// Decodes version 1100 or later texture slots, including optional index tracks.
    pub fn texture_slots(&self, version: u32) -> Result<Vec<LayerTextureSlot>, Error> {
        if version < 1100 {
            return Ok(Vec::new());
        }
        let (slots, _) = self.scan_slots()?;
        Ok(slots)
    }

    /// Replaces Reforged texture slots and keeps animation tracks that follow them.
    pub fn set_texture_slots(&mut self, slots: &[LayerTextureSlot]) -> Result<(), Error> {
        let (_, tail_start) = self.scan_slots()?;
        if slots.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: *b"LAYS",
                size: slots.len(),
            });
        }
        let mut bytes = self.bytes[..60].to_vec();
        bytes[56..60].copy_from_slice(&(slots.len() as u32).to_le_bytes());
        for slot in slots {
            bytes.extend_from_slice(&slot.texture_id.to_le_bytes());
            bytes.extend_from_slice(&slot.texture_type.to_le_bytes());
            if let Some(track) = &slot.track {
                if track.tag != *b"KMTF" {
                    return Err(Error::MalformedRecord {
                        tag: *b"LAYS",
                        offset: bytes.len(),
                    });
                }
                bytes.extend_from_slice(&track.to_bytes()?);
            }
        }
        bytes.extend_from_slice(&self.bytes[tail_start..]);
        self.replace_bytes(bytes)
    }

    /// Returns animation tracks after the fixed header and texture slots.
    pub fn tracks(&self, version: u32) -> Result<Vec<AnimationTrack>, Error> {
        let mut offset = if version >= 1100 {
            self.scan_slots()?.1
        } else {
            fixed_size(version)
        };
        let mut tracks = Vec::new();
        while offset < self.bytes.len() {
            let (track, used) = AnimationTrack::parse(&self.bytes, offset)?;
            if !is_layer_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: *b"LAYS",
                    offset,
                });
            }
            tracks.push(track);
            offset += used;
        }
        Ok(tracks)
    }

    /// Replaces layer animation tracks after any Reforged texture slots.
    pub fn set_tracks(&mut self, version: u32, tracks: &[AnimationTrack]) -> Result<(), Error> {
        let start = if version >= 1100 {
            self.scan_slots()?.1
        } else {
            fixed_size(version)
        };
        let mut bytes = self
            .bytes
            .get(..start)
            .ok_or(Error::MalformedRecord {
                tag: *b"LAYS",
                offset: start,
            })?
            .to_vec();
        for track in tracks {
            if !is_layer_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: *b"LAYS",
                    offset: bytes.len(),
                });
            }
            bytes.extend_from_slice(&track.to_bytes()?);
        }
        self.replace_bytes(bytes)
    }

    fn replace_bytes(&mut self, mut bytes: Vec<u8>) -> Result<(), Error> {
        if bytes.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: *b"LAYS",
                size: bytes.len(),
            });
        }
        let size = bytes.len() as u32;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        self.bytes = bytes;
        Ok(())
    }

    fn scan_slots(&self) -> Result<(Vec<LayerTextureSlot>, usize), Error> {
        let count = self.u32_at(56)? as usize;
        let mut offset = 60usize;
        let mut slots = Vec::new();
        for _ in 0..count {
            let texture_id = self.u32_at(offset)?;
            let texture_type = self.u32_at(offset + 4)?;
            offset += 8;
            let track = if self.bytes.get(offset..offset + 4) == Some(b"KMTF") {
                let (track, used) = AnimationTrack::parse(&self.bytes, offset)?;
                offset += used;
                Some(track)
            } else {
                None
            };
            slots.push(LayerTextureSlot {
                texture_id,
                texture_type,
                track,
            });
        }
        Ok((slots, offset))
    }

    /// Sets the base alpha value.
    pub fn set_alpha(&mut self, alpha: f32) -> Result<(), Error> {
        self.set_u32_at(24, alpha.to_bits())
    }

    fn u32_at(&self, offset: usize) -> Result<u32, Error> {
        let bytes = self
            .bytes
            .get(offset..offset + 4)
            .ok_or(Error::MalformedRecord {
                tag: *b"LAYS",
                offset,
            })?;
        Ok(u32::from_le_bytes(
            bytes.try_into().expect("four-byte field"),
        ))
    }

    fn set_u32_at(&mut self, offset: usize, value: u32) -> Result<(), Error> {
        let bytes = self
            .bytes
            .get_mut(offset..offset + 4)
            .ok_or(Error::MalformedRecord {
                tag: *b"LAYS",
                offset,
            })?;
        bytes.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }
}

impl Model {
    /// Decodes all `MTLS` records in file order.
    pub fn materials(&self) -> Result<Vec<Material>, Error> {
        let mut materials = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == TAG) {
            materials.extend(
                sized_records(&chunk.data, TAG)?
                    .into_iter()
                    .map(|bytes| Material {
                        bytes: bytes.to_vec(),
                    }),
            );
        }
        Ok(materials)
    }

    /// Replaces all material records in the first `MTLS` chunk.
    pub fn set_materials(&mut self, materials: &[Material]) -> Result<(), Error> {
        let size = materials.iter().try_fold(0usize, |sum, material| {
            sum.checked_add(material.bytes.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for material in materials {
            data.extend_from_slice(material.as_bytes());
        }
        self.replace_chunks(TAG, data);
        Ok(())
    }
}
