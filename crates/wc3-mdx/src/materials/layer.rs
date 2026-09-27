//! Material layers, animation tracks, and version-specific texture slots.
use std::{fmt::Debug, marker::PhantomData};

use super::write_count;
use crate::animation::track_group;
use crate::{
    AnimationTrack, Color, Cursor, DecodeError, EncodeError, Encoder, LayerTextureId, ModelVersion,
    Readable, SupportsEmissiveGain, SupportsFresnel, SupportsLayerShaderTypeId,
    SupportsLayerTextureSlots, Tag, TrackTag, ValueError, Version, Writable,
};

pub(super) const LAYER_TAG: Tag = *b"LAYS";

track_group! {
    pub enum LayerTrack {
        Alpha: LayerAlpha,
        TextureId: LayerTextureId,
        EmissiveGain: LayerEmissiveGain,
        FresnelColor: LayerFresnelColor,
        FresnelOpacity: LayerFresnelOpacity,
        FresnelTeamColor: LayerFresnelTeamColor,
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
    pub track: Option<AnimationTrack<LayerTextureId>>,
}

/// A material layer with parsed texture slots and animation tracks.
#[derive(Clone, Debug, PartialEq, Readable, Writable)]
#[mdx(sized(tag = LAYER_TAG))]
pub struct Layer<V: ModelVersion> {
    version: PhantomData<V>,
    filter_mode: u32,
    shading_flags: u32,
    texture_id: u32,
    texture_animation_id: u32,
    coordinate_id: u32,
    alpha: f32,
    extensions: V::LayerExtra,
    tracks: Vec<LayerTrack>,
}

#[derive(Clone, Debug, PartialEq, Readable, Writable)]
pub struct Fresnel {
    color: Color,
    opacity: f32,
    team_color: f32,
}

impl Default for Fresnel {
    fn default() -> Self {
        Self {
            color: [1.0; 3],
            opacity: 0.0,
            team_color: 0.0,
        }
    }
}

/// The fields following a layer's shared header.
pub trait LayerExtra: Default + Readable + Writable + Clone + Debug + PartialEq {
    fn emissive_gain(&self) -> Option<f32> {
        None
    }
    fn emissive_gain_mut(&mut self) -> Option<&mut f32> {
        None
    }
    fn fresnel(&self) -> Option<&Fresnel> {
        None
    }
    fn fresnel_mut(&mut self) -> Option<&mut Fresnel> {
        None
    }
    fn shader_type_id(&self) -> Option<u32> {
        None
    }
    fn shader_type_id_mut(&mut self) -> Option<&mut u32> {
        None
    }
    fn texture_slots(&self) -> &[LayerTextureSlot] {
        &[]
    }
    fn texture_slots_mut(&mut self) -> Option<&mut Vec<LayerTextureSlot>> {
        None
    }
}

#[derive(Clone, Debug, PartialEq, Default, Readable, Writable)]
pub struct ClassicLayerExtra;

impl LayerExtra for ClassicLayerExtra {}

#[derive(Clone, Debug, PartialEq, Default, Readable, Writable)]
pub struct Layer900Extra {
    emissive_gain: f32,
}

impl LayerExtra for Layer900Extra {
    fn emissive_gain(&self) -> Option<f32> {
        Some(self.emissive_gain)
    }
    fn emissive_gain_mut(&mut self) -> Option<&mut f32> {
        Some(&mut self.emissive_gain)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Readable, Writable)]
pub struct Layer1000Extra {
    base: Layer900Extra,
    fresnel: Fresnel,
}

impl LayerExtra for Layer1000Extra {
    fn emissive_gain(&self) -> Option<f32> {
        self.base.emissive_gain()
    }
    fn emissive_gain_mut(&mut self) -> Option<&mut f32> {
        self.base.emissive_gain_mut()
    }
    fn fresnel(&self) -> Option<&Fresnel> {
        Some(&self.fresnel)
    }
    fn fresnel_mut(&mut self) -> Option<&mut Fresnel> {
        Some(&mut self.fresnel)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layer1100Extra {
    base: Layer1000Extra,
    shader_type_id: u32,
    texture_slots: Vec<LayerTextureSlot>,
}

impl Readable for Layer1100Extra {
    fn read_from(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        let base = cursor.read::<Layer1000Extra>()?;
        let shader_type_id = cursor.read()?;
        let count = cursor.read::<u32>()? as usize;
        let mut texture_slots = Vec::new();
        for _ in 0..count {
            let texture_id = cursor.read()?;
            let texture_type = cursor.read()?;
            let track = if cursor
                .remaining()
                .starts_with(&TrackTag::LayerTextureId.bytes())
            {
                Some(cursor.read::<AnimationTrack<LayerTextureId>>()?)
            } else {
                None
            };
            texture_slots.push(LayerTextureSlot {
                texture_id,
                texture_type,
                track,
            });
        }
        Ok(Self {
            base,
            shader_type_id,
            texture_slots,
        })
    }
}

impl Writable for Layer1100Extra {
    fn write_to(&self, output: &mut Encoder<'_>) -> Result<(), EncodeError> {
        output.write(&self.base)?;
        output.write(&self.shader_type_id)?;
        write_count(output, self.texture_slots.len(), LAYER_TAG)?;
        for slot in &self.texture_slots {
            output.write(&slot.texture_id)?;
            output.write(&slot.texture_type)?;
            if let Some(track) = &slot.track {
                output.write(track)?;
            }
        }
        Ok(())
    }
}

impl LayerExtra for Layer1100Extra {
    fn emissive_gain(&self) -> Option<f32> {
        self.base.emissive_gain()
    }
    fn emissive_gain_mut(&mut self) -> Option<&mut f32> {
        self.base.emissive_gain_mut()
    }
    fn fresnel(&self) -> Option<&Fresnel> {
        self.base.fresnel()
    }
    fn fresnel_mut(&mut self) -> Option<&mut Fresnel> {
        self.base.fresnel_mut()
    }
    fn shader_type_id(&self) -> Option<u32> {
        Some(self.shader_type_id)
    }
    fn shader_type_id_mut(&mut self) -> Option<&mut u32> {
        Some(&mut self.shader_type_id)
    }
    fn texture_slots(&self) -> &[LayerTextureSlot] {
        &self.texture_slots
    }
    fn texture_slots_mut(&mut self) -> Option<&mut Vec<LayerTextureSlot>> {
        Some(&mut self.texture_slots)
    }
}

impl<V: ModelVersion> Default for Layer<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: ModelVersion> Layer<V> {
    /// Creates an empty layer with version-appropriate fields.
    pub fn new() -> Self {
        Self {
            version: PhantomData,
            filter_mode: 0,
            shading_flags: 0,
            texture_id: 0,
            texture_animation_id: u32::MAX,
            coordinate_id: 0,
            alpha: 1.0,
            extensions: V::LayerExtra::default(),
            tracks: Vec::new(),
        }
    }

    /// Returns the MDX version used for this layer.
    pub fn version(&self) -> Version {
        V::NUMBER
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
    /// Returns emissive gain when this layout includes it.
    pub fn try_emissive_gain(&self) -> Result<f32, ValueError> {
        if !(V::NUMBER >= 900) {
            return Err(ValueError::UnsupportedVersion {
                tag: *b"LAYS",
                minimum: 900,
                actual: V::NUMBER,
            });
        }
        Ok(self.extensions.emissive_gain().expect("supported version"))
    }
    pub fn try_set_emissive_gain(&mut self, value: f32) -> Result<(), ValueError> {
        *self
            .extensions
            .emissive_gain_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 900,
                actual: V::NUMBER,
            })? = value;
        Ok(())
    }
    pub fn try_fresnel_color(&self) -> Result<Color, ValueError> {
        if !(V::NUMBER >= 1000) {
            return Err(ValueError::UnsupportedVersion {
                tag: *b"LAYS",
                minimum: 1000,
                actual: V::NUMBER,
            });
        }
        Ok(self
            .extensions
            .fresnel()
            .map(|f| f.color)
            .expect("supported version"))
    }
    pub fn try_set_fresnel_color(&mut self, value: Color) -> Result<(), ValueError> {
        self.extensions
            .fresnel_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 1000,
                actual: V::NUMBER,
            })?
            .color = value;
        Ok(())
    }
    pub fn try_fresnel_opacity(&self) -> Result<f32, ValueError> {
        if !(V::NUMBER >= 1000) {
            return Err(ValueError::UnsupportedVersion {
                tag: *b"LAYS",
                minimum: 1000,
                actual: V::NUMBER,
            });
        }
        Ok(self
            .extensions
            .fresnel()
            .map(|f| f.opacity)
            .expect("supported version"))
    }
    pub fn try_set_fresnel_opacity(&mut self, value: f32) -> Result<(), ValueError> {
        self.extensions
            .fresnel_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 1000,
                actual: V::NUMBER,
            })?
            .opacity = value;
        Ok(())
    }
    pub fn try_fresnel_team_color(&self) -> Result<f32, ValueError> {
        if !(V::NUMBER >= 1000) {
            return Err(ValueError::UnsupportedVersion {
                tag: *b"LAYS",
                minimum: 1000,
                actual: V::NUMBER,
            });
        }
        Ok(self
            .extensions
            .fresnel()
            .map(|f| f.team_color)
            .expect("supported version"))
    }
    pub fn try_set_fresnel_team_color(&mut self, value: f32) -> Result<(), ValueError> {
        self.extensions
            .fresnel_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 1000,
                actual: V::NUMBER,
            })?
            .team_color = value;
        Ok(())
    }
    pub fn try_shader_type_id(&self) -> Result<u32, ValueError> {
        if !(V::NUMBER >= 1100) {
            return Err(ValueError::UnsupportedVersion {
                tag: *b"LAYS",
                minimum: 1100,
                actual: V::NUMBER,
            });
        }
        Ok(self.extensions.shader_type_id().expect("supported version"))
    }
    pub fn try_set_shader_type_id(&mut self, value: u32) -> Result<(), ValueError> {
        *self
            .extensions
            .shader_type_id_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 1100,
                actual: V::NUMBER,
            })? = value;
        Ok(())
    }
    pub fn try_texture_slots(&self) -> Result<&[LayerTextureSlot], ValueError> {
        if !(V::NUMBER >= 1100) {
            return Err(ValueError::UnsupportedVersion {
                tag: *b"LAYS",
                minimum: 1100,
                actual: V::NUMBER,
            });
        }
        Ok(self.extensions.texture_slots())
    }
    pub fn try_set_texture_slots(&mut self, slots: &[LayerTextureSlot]) -> Result<(), ValueError> {
        *self
            .extensions
            .texture_slots_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 1100,
                actual: V::NUMBER,
            })? = slots.to_vec();
        Ok(())
    }
    /// Borrows layer animation tracks after any texture slots.
    pub fn tracks(&self) -> &[LayerTrack] {
        &self.tracks
    }
    /// Replaces layer animation tracks.
    pub fn set_tracks(&mut self, tracks: &[LayerTrack]) {
        self.tracks = tracks.to_vec();
    }
}

impl<V: SupportsEmissiveGain> Layer<V> {
    pub fn emissive_gain(&self) -> f32 {
        self.extensions.emissive_gain().expect("supported version")
    }
    pub fn set_emissive_gain(&mut self, value: f32) {
        self.try_set_emissive_gain(value)
            .expect("supported version")
    }
}

impl<V: SupportsFresnel> Layer<V> {
    pub fn fresnel_color(&self) -> Color {
        self.extensions
            .fresnel()
            .map(|f| f.color)
            .expect("supported version")
    }
    pub fn set_fresnel_color(&mut self, value: Color) {
        self.try_set_fresnel_color(value)
            .expect("supported version")
    }
}

impl<V: SupportsFresnel> Layer<V> {
    pub fn fresnel_opacity(&self) -> f32 {
        self.extensions
            .fresnel()
            .map(|f| f.opacity)
            .expect("supported version")
    }
    pub fn set_fresnel_opacity(&mut self, value: f32) {
        self.try_set_fresnel_opacity(value)
            .expect("supported version")
    }
}

impl<V: SupportsFresnel> Layer<V> {
    pub fn fresnel_team_color(&self) -> f32 {
        self.extensions
            .fresnel()
            .map(|f| f.team_color)
            .expect("supported version")
    }
    pub fn set_fresnel_team_color(&mut self, value: f32) {
        self.try_set_fresnel_team_color(value)
            .expect("supported version")
    }
}

impl<V: SupportsLayerShaderTypeId> Layer<V> {
    pub fn shader_type_id(&self) -> u32 {
        self.extensions.shader_type_id().expect("supported version")
    }
    pub fn set_shader_type_id(&mut self, value: u32) {
        self.try_set_shader_type_id(value)
            .expect("supported version")
    }
}

impl<V: SupportsLayerTextureSlots> Layer<V> {
    pub fn texture_slots(&self) -> &[LayerTextureSlot] {
        self.extensions.texture_slots()
    }
    pub fn set_texture_slots(&mut self, slots: &[LayerTextureSlot]) {
        self.try_set_texture_slots(slots)
            .expect("supported version")
    }
}
