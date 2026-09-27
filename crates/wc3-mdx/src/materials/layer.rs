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
    emissive_gain: V::EmissiveGain,
    fresnel: V::Fresnel,
    shader_type_id: V::ShaderTypeId,
    texture_slots: V::TextureSlots,
    tracks: Vec<LayerTrack>,
}

/// Version-selected storage for the layer's emissive gain.
pub trait EmissiveGainField: Default + Readable + Writable + Clone + Debug + PartialEq {
    fn emissive_gain(&self) -> Option<f32> {
        None
    }
    fn emissive_gain_mut(&mut self) -> Option<&mut f32> {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, Readable, Writable)]
pub struct NoEmissiveGain;
impl EmissiveGainField for NoEmissiveGain {}

#[derive(Clone, Debug, PartialEq, Readable, Writable)]
pub struct EmissiveGain(f32);
impl Default for EmissiveGain {
    fn default() -> Self {
        Self(1.0)
    }
}
impl EmissiveGainField for EmissiveGain {
    fn emissive_gain(&self) -> Option<f32> {
        Some(self.0)
    }
    fn emissive_gain_mut(&mut self) -> Option<&mut f32> {
        Some(&mut self.0)
    }
}

/// Version-selected storage for the layer's fresnel.
pub trait FresnelField: Default + Readable + Writable + Clone + Debug + PartialEq {
    fn fresnel(&self) -> Option<LayerFresnel> {
        None
    }
    fn fresnel_mut(&mut self) -> Option<&mut LayerFresnel> {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, Readable, Writable)]
pub struct NoFresnel;
impl FresnelField for NoFresnel {}

/// The layer's RGB Fresnel color, opacity, and team-color contribution in MDX order.
#[derive(Clone, Copy, Debug, PartialEq, Readable, Writable)]
pub struct LayerFresnel {
    pub color: Color,
    pub opacity: f32,
    pub team_color: f32,
}
impl Default for LayerFresnel {
    fn default() -> Self {
        Self {
            color: [1.0; 3],
            opacity: 0.0,
            team_color: 0.0,
        }
    }
}
impl FresnelField for LayerFresnel {
    fn fresnel(&self) -> Option<LayerFresnel> {
        Some(*self)
    }
    fn fresnel_mut(&mut self) -> Option<&mut LayerFresnel> {
        Some(self)
    }
}

/// Version-selected storage for the layer's shader type id.
pub trait LayerShaderTypeField: Default + Readable + Writable + Clone + Debug + PartialEq {
    fn shader_type_id(&self) -> Option<u32> {
        None
    }
    fn shader_type_id_mut(&mut self) -> Option<&mut u32> {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, Readable, Writable)]
pub struct NoLayerShaderType;
impl LayerShaderTypeField for NoLayerShaderType {}

#[derive(Clone, Debug, Default, PartialEq, Readable, Writable)]
pub struct LayerShaderType(u32);
impl LayerShaderTypeField for LayerShaderType {
    fn shader_type_id(&self) -> Option<u32> {
        Some(self.0)
    }
    fn shader_type_id_mut(&mut self) -> Option<&mut u32> {
        Some(&mut self.0)
    }
}

/// Version-selected storage for Reforged texture slots.
pub trait LayerTextureSlotsField:
    Default + Readable + Writable + Clone + Debug + PartialEq
{
    fn texture_slots(&self) -> Option<&[LayerTextureSlot]> {
        None
    }
    fn texture_slots_mut(&mut self) -> Option<&mut Vec<LayerTextureSlot>> {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, Readable, Writable)]
pub struct NoLayerTextureSlots;
impl LayerTextureSlotsField for NoLayerTextureSlots {}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayerTextureSlots(Vec<LayerTextureSlot>);
impl LayerTextureSlotsField for LayerTextureSlots {
    fn texture_slots(&self) -> Option<&[LayerTextureSlot]> {
        Some(&self.0)
    }
    fn texture_slots_mut(&mut self) -> Option<&mut Vec<LayerTextureSlot>> {
        Some(&mut self.0)
    }
}

impl Readable for LayerTextureSlots {
    fn read_from(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
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
        Ok(Self(texture_slots))
    }
}

impl Writable for LayerTextureSlots {
    fn write_to(&self, output: &mut Encoder<'_>) -> Result<(), EncodeError> {
        write_count(output, self.0.len(), LAYER_TAG)?;
        for slot in &self.0 {
            output.write(&slot.texture_id)?;
            output.write(&slot.texture_type)?;
            if let Some(track) = &slot.track {
                output.write(track)?;
            }
        }
        Ok(())
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
            emissive_gain: V::EmissiveGain::default(),
            fresnel: V::Fresnel::default(),
            shader_type_id: V::ShaderTypeId::default(),
            texture_slots: V::TextureSlots::default(),
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
    /// Returns the layer's emissive gain, available from version 900.
    pub fn try_emissive_gain(&self) -> Result<f32, ValueError> {
        self.emissive_gain
            .emissive_gain()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 900,
                actual: V::NUMBER,
            })
    }
    /// Sets the layer's emissive gain, available from version 900.
    pub fn try_set_emissive_gain(&mut self, value: f32) -> Result<(), ValueError> {
        *self
            .emissive_gain
            .emissive_gain_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 900,
                actual: V::NUMBER,
            })? = value;
        Ok(())
    }
    /// Returns the layer's fresnel, available from version 1000.
    pub fn try_fresnel(&self) -> Result<LayerFresnel, ValueError> {
        self.fresnel
            .fresnel()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 1000,
                actual: V::NUMBER,
            })
    }
    /// Sets the layer's fresnel, available from version 1000.
    pub fn try_set_fresnel(&mut self, value: LayerFresnel) -> Result<(), ValueError> {
        *self
            .fresnel
            .fresnel_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 1000,
                actual: V::NUMBER,
            })? = value;
        Ok(())
    }
    /// Returns the layer's shader type id, available from version 1100.
    pub fn try_shader_type_id(&self) -> Result<u32, ValueError> {
        self.shader_type_id
            .shader_type_id()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 1100,
                actual: V::NUMBER,
            })
    }
    /// Sets the layer's shader type id, available from version 1100.
    pub fn try_set_shader_type_id(&mut self, value: u32) -> Result<(), ValueError> {
        *self
            .shader_type_id
            .shader_type_id_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 1100,
                actual: V::NUMBER,
            })? = value;
        Ok(())
    }
    /// Returns the layer's texture slots, available from version 1100.
    pub fn try_texture_slots(&self) -> Result<&[LayerTextureSlot], ValueError> {
        self.texture_slots
            .texture_slots()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 1100,
                actual: V::NUMBER,
            })
    }
    /// Sets the layer's texture slots, available from version 1100.
    pub fn try_set_texture_slots(&mut self, value: &[LayerTextureSlot]) -> Result<(), ValueError> {
        *self
            .texture_slots
            .texture_slots_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 1100,
                actual: V::NUMBER,
            })? = value.to_vec();
        Ok(())
    }
    /// Returns the Fresnel color, available from version 1000.
    pub fn try_fresnel_color(&self) -> Result<Color, ValueError> {
        Ok(self.try_fresnel()?.color)
    }
    /// Sets the Fresnel color, available from version 1000.
    pub fn try_set_fresnel_color(&mut self, value: Color) -> Result<(), ValueError> {
        self.fresnel
            .fresnel_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 1000,
                actual: V::NUMBER,
            })?
            .color = value;
        Ok(())
    }
    /// Returns the Fresnel opacity, available from version 1000.
    pub fn try_fresnel_opacity(&self) -> Result<f32, ValueError> {
        Ok(self.try_fresnel()?.opacity)
    }
    /// Sets the Fresnel opacity, available from version 1000.
    pub fn try_set_fresnel_opacity(&mut self, value: f32) -> Result<(), ValueError> {
        self.fresnel
            .fresnel_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 1000,
                actual: V::NUMBER,
            })?
            .opacity = value;
        Ok(())
    }
    /// Returns the Fresnel team color, available from version 1000.
    pub fn try_fresnel_team_color(&self) -> Result<f32, ValueError> {
        Ok(self.try_fresnel()?.team_color)
    }
    /// Sets the Fresnel team color, available from version 1000.
    pub fn try_set_fresnel_team_color(&mut self, value: f32) -> Result<(), ValueError> {
        self.fresnel
            .fresnel_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 1000,
                actual: V::NUMBER,
            })?
            .team_color = value;
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
    /// Returns the layer's emissive gain.
    pub fn emissive_gain(&self) -> f32 {
        self.try_emissive_gain().expect("supported version")
    }
    /// Sets the layer's emissive gain.
    pub fn set_emissive_gain(&mut self, value: f32) {
        self.try_set_emissive_gain(value)
            .expect("supported version");
    }
}

impl<V: SupportsFresnel> Layer<V> {
    /// Returns the layer's fresnel.
    pub fn fresnel(&self) -> LayerFresnel {
        self.try_fresnel().expect("supported version")
    }
    /// Sets the layer's fresnel.
    pub fn set_fresnel(&mut self, value: LayerFresnel) {
        self.try_set_fresnel(value).expect("supported version");
    }
}

impl<V: SupportsFresnel> Layer<V> {
    /// Returns the layer's fresnel color.
    pub fn fresnel_color(&self) -> Color {
        self.try_fresnel_color().expect("supported version")
    }
    /// Sets the layer's fresnel color.
    pub fn set_fresnel_color(&mut self, value: Color) {
        self.try_set_fresnel_color(value)
            .expect("supported version");
    }
}

impl<V: SupportsFresnel> Layer<V> {
    /// Returns the layer's fresnel opacity.
    pub fn fresnel_opacity(&self) -> f32 {
        self.try_fresnel_opacity().expect("supported version")
    }
    /// Sets the layer's fresnel opacity.
    pub fn set_fresnel_opacity(&mut self, value: f32) {
        self.try_set_fresnel_opacity(value)
            .expect("supported version");
    }
}

impl<V: SupportsFresnel> Layer<V> {
    /// Returns the layer's fresnel team color.
    pub fn fresnel_team_color(&self) -> f32 {
        self.try_fresnel_team_color().expect("supported version")
    }
    /// Sets the layer's fresnel team color.
    pub fn set_fresnel_team_color(&mut self, value: f32) {
        self.try_set_fresnel_team_color(value)
            .expect("supported version");
    }
}

impl<V: SupportsLayerShaderTypeId> Layer<V> {
    /// Returns the layer's shader type id.
    pub fn shader_type_id(&self) -> u32 {
        self.try_shader_type_id().expect("supported version")
    }
    /// Sets the layer's shader type id.
    pub fn set_shader_type_id(&mut self, value: u32) {
        self.try_set_shader_type_id(value)
            .expect("supported version");
    }
}

impl<V: SupportsLayerTextureSlots> Layer<V> {
    /// Returns the layer's texture slots.
    pub fn texture_slots(&self) -> &[LayerTextureSlot] {
        self.try_texture_slots().expect("supported version")
    }
    /// Sets the layer's texture slots.
    pub fn set_texture_slots(&mut self, value: &[LayerTextureSlot]) {
        self.try_set_texture_slots(value)
            .expect("supported version");
    }
}
