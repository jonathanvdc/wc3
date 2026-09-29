//! Texture bindings, blend modes, and animated surface properties.
use crate::model::conversion::ConversionContext;
use crate::model::ConversionError;
use crate::model::{mdl, mdx};
use bitfield::bitfield;
use mdl_codec::{
    full, is_no_reference, is_white, no_reference, one, white, zero, zero_id, ShaderMarker,
    TextureBindings,
};
use std::{fmt::Debug, marker::PhantomData};

use super::{write_count, ShaderType};
use crate::model::animation::track_group;
use crate::model::{
    AnimationTrack, Color, Cursor, Encoder, LayerTextureId, ModelVersion, ReadError,
    SupportsEmissiveGain, SupportsFresnel, SupportsLayerShaderTypeId, SupportsLayerTextureSlots,
    Tag, TrackTag, ValueError, Version, WriteError,
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

/// How the layer blends with the surfaces behind it. Unknown values round-trip in MDX.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    mdx::Read,
    mdx::Write,
    mdl::Read,
    mdl::Write,
    mdx::Value,
)]
#[mdx(value = u32)]
#[mdl(value)]
pub enum LayerFilterMode {
    #[default]
    #[mdx(value = 0)]
    None,
    #[mdx(value = 1)]
    Transparent,
    #[mdx(value = 2)]
    Blend,
    #[mdx(value = 3)]
    Additive,
    #[mdx(value = 4)]
    AddAlpha,
    #[mdx(value = 5)]
    Modulate,
    #[mdx(value = 6)]
    Modulate2x,
    #[mdx(unknown)]
    #[mdl(unknown)]
    Unknown(u32),
}

bitfield! {
    /// Material layer shading bits, preserving unrecognized bits.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct LayerShadingFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Disables shading for the layer.
    pub unshaded, set_unshaded: 0;
    /// Returns or changes the `SPHERE_ENV_MAP` bit.
    pub sphere_env_map, set_sphere_env_map: 1;
    /// Returns or changes the layer U wrap bit (separate from texture flags).
    pub wrap_width, set_wrap_width: 2;
    /// Returns or changes the layer V wrap bit (separate from texture flags).
    pub wrap_height, set_wrap_height: 3;
    /// Renders both sides of the surface.
    pub two_sided, set_two_sided: 4;
    /// Excludes the layer from fog.
    pub unfogged, set_unfogged: 5;
    /// Disables depth testing for this rendering pass.
    pub no_depth_test, set_no_depth_test: 6;
    /// Prevents this rendering pass from writing depth.
    pub no_depth_set, set_no_depth_set: 7;
    /// Returns or changes the `UNLIT` bit.
    pub unlit, set_unlit: 8;
    /// Returns or changes back-face shadow casting.
    pub back_faces_for_shadows, set_back_faces_for_shadows: 9;
    /// Returns or changes ambient occlusion participation.
    pub ambient_occlusion, set_ambient_occlusion: 10;
}

impl LayerShadingFlags {
    /// Retains every bit, including flags without known names.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl mdx::Read for LayerShadingFlags {
    fn read_mdx(cursor: &mut Cursor<'_>) -> Result<Self, mdx::ReadError> {
        Ok(Self(cursor.read()?))
    }
}
impl mdx::Write for LayerShadingFlags {
    fn write_mdx(&self, encoder: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        encoder.write(&self.bits())
    }
}

/// A Reforged texture binding with an optional animated texture index.
#[derive(Clone, Debug, PartialEq)]
pub struct LayerTextureSlot {
    pub texture_id: u32,
    pub texture_type: u32,
    pub track: Option<AnimationTrack<LayerTextureId>>,
}

/// One rendering pass within a material.
///
/// Texture indices refer to the model texture collection. Reforged layers can
/// bind multiple HD texture slots. Use HiveWorkshop MDL output when animating
/// slots other than diffuse; Warcraft III syntax cannot identify those channels.
/// MDL output exports animation tracks in place of their base values.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdx(sized(tag = LAYER_TAG))]
#[mdl(block = "Layer", validate_write = "Self::validate_mdl",
    write_order(filter_mode, shading, shader, textures, texture_animation_id,
        coordinate_id, alpha, emissive, color, opacity, team_color, channels),
    virtual_fields(
    #[mdl(flags(
        Unshaded = 1,
        SphereEnvMap = 2,
        WrapWidth = 4,
        WrapHeight = 8,
        TwoSided = 16,
        Unfogged = 32,
        NoDepthTest = 64,
        NoDepthSet = 128,
        Unlit = 256,
        BackFacesForShadows = 512,
        AmbientOcclusion = 1024
    ))]
    #[mdl(get = "Self::mdl_shading_flags", set = "Self::set_mdl_shading_flags",
        hive_skip_bits = 0x70c)]
    shading: u32,
        #[mdl(property = "Shader", hive_name = "ShaderTypeId", delegate)]
        #[mdl(get = "Self::mdl_shader", set = "Self::set_mdl_shader")]
        shader: ShaderMarker,
        #[mdl(flatten)]
        #[mdl(get = "Self::mdl_textures", set = "Self::set_mdl_textures")]
        textures: TextureBindings,
        #[mdl(
            animatable = "EmissiveGain",
            track = "LayerTrack::EmissiveGain",
            default = "one",
            skip_if = "full"
        )]
        #[mdl(get = "Self::mdl_emissive", slot = "Self::mdl_emissive_mut")]
        emissive: f32,
        #[mdl(
            animatable = "FresnelColor",
            track = "LayerTrack::FresnelColor",
            default = "white",
            skip_if = "is_white"
        )]
        #[mdl(get = "Self::mdl_color", slot = "Self::mdl_color_mut")]
        color: Color,
        #[mdl(
            animatable = "FresnelOpacity",
            track = "LayerTrack::FresnelOpacity",
            default,
            skip_if = "zero"
        )]
        #[mdl(get = "Self::mdl_opacity", slot = "Self::mdl_opacity_mut")]
        opacity: f32,
        #[mdl(
            animatable = "FresnelTeamColor",
            track = "LayerTrack::FresnelTeamColor",
            default,
            skip_if = "zero"
        )]
        #[mdl(get = "Self::mdl_team_color", slot = "Self::mdl_team_color_mut")]
        team_color: f32,
        #[mdl(tracks)]
        #[mdl(get = "Self::mdl_tracks", set = "Self::set_mdl_tracks")]
        channels: Vec<LayerTrack>
    )
)]
pub struct Layer<V: ModelVersion> {
    #[mdl(skip, default)]
    version: PhantomData<V>,
    #[mdl(property = "FilterMode", default)]
    /// Blend filter mode.
    pub filter_mode: LayerFilterMode,
    #[mdl(skip, default)]
    /// Layer shading bits.
    pub shading_flags: LayerShadingFlags,
    #[mdl(skip, default)]
    /// Index into the model texture collection when no texture-ID track is active.
    pub texture_id: u32,
    #[mdl(
        property = "TVertexAnimId",
        default = "no_reference",
        skip_if = "is_no_reference"
    )]
    /// Texture-animation index, or `u32::MAX` for no texture animation.
    pub texture_animation_id: u32,
    #[mdl(property = "CoordId", default, skip_if = "zero_id")]
    /// Index of the UV coordinate set to use on the geoset.
    pub coordinate_id: u32,
    #[mdl(
        animatable = "Alpha",
        track = "LayerTrack::Alpha",
        default = "one",
        skip_if = "full"
    )]
    /// Opacity when no alpha track is active; 1.0 is fully opaque.
    pub alpha: f32,
    #[mdl(skip, default)]
    emissive_gain: V::EmissiveGain,
    #[mdl(skip, default)]
    fresnel: V::Fresnel,
    #[mdl(skip, default)]
    shader_type: V::ShaderType,
    #[mdl(skip, default)]
    texture_slots: V::TextureSlots,
    #[mdl(skip, default)]
    /// Layer animation tracks after any texture slots.
    pub tracks: Vec<LayerTrack>,
}

/// Version-selected storage for the layer's emissive gain.
pub trait EmissiveGainField: Default + mdx::Read + mdx::Write + Clone + Debug + PartialEq {
    fn emissive_gain(&self) -> Option<f32> {
        None
    }
    fn emissive_gain_mut(&mut self) -> Option<&mut f32> {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write)]
pub struct NoEmissiveGain;
impl EmissiveGainField for NoEmissiveGain {}

#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write)]
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
pub trait FresnelField: Default + mdx::Read + mdx::Write + Clone + Debug + PartialEq {
    fn fresnel(&self) -> Option<LayerFresnel> {
        None
    }
    fn fresnel_mut(&mut self) -> Option<&mut LayerFresnel> {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write)]
pub struct NoFresnel;
impl FresnelField for NoFresnel {}

/// Fresnel color, opacity, and team-color contribution for a Reforged layer.
#[derive(Clone, Copy, Debug, PartialEq, mdx::Read, mdx::Write)]
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

/// Version-selected storage for the layer's shader type.
pub trait LayerShaderTypeField:
    Default + mdx::Read + mdx::Write + Clone + Debug + PartialEq
{
    fn shader_type(&self) -> Option<ShaderType> {
        None
    }
    fn shader_type_mut(&mut self) -> Option<&mut ShaderType> {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write)]
pub struct NoLayerShaderType;
impl LayerShaderTypeField for NoLayerShaderType {}

impl LayerShaderTypeField for ShaderType {
    fn shader_type(&self) -> Option<ShaderType> {
        Some(*self)
    }
    fn shader_type_mut(&mut self) -> Option<&mut ShaderType> {
        Some(self)
    }
}

/// Version-selected storage for Reforged texture slots.
pub trait LayerTextureSlotsField:
    Default + mdx::Read + mdx::Write + Clone + Debug + PartialEq
{
    fn texture_slots(&self) -> Option<&[LayerTextureSlot]> {
        None
    }
    fn texture_slots_mut(&mut self) -> Option<&mut Vec<LayerTextureSlot>> {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write)]
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

impl mdx::Read for LayerTextureSlots {
    fn read_mdx(cursor: &mut Cursor<'_>) -> Result<Self, ReadError> {
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

impl mdx::Write for LayerTextureSlots {
    fn write_mdx(&self, output: &mut Encoder<'_>) -> Result<(), WriteError> {
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
            filter_mode: LayerFilterMode::default(),
            shading_flags: LayerShadingFlags::default(),
            texture_id: 0,
            texture_animation_id: u32::MAX,
            coordinate_id: 0,
            alpha: 1.0,
            emissive_gain: V::EmissiveGain::default(),
            fresnel: V::Fresnel::default(),
            shader_type: V::ShaderType::default(),
            texture_slots: V::TextureSlots::default(),
            tracks: Vec::new(),
        }
    }

    /// Returns the MDX version used for this layer.
    pub fn version(&self) -> Version {
        V::NUMBER
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
    /// Returns the layer's shader type, available from version 1100.
    pub fn try_shader_type(&self) -> Result<ShaderType, ValueError> {
        self.shader_type
            .shader_type()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 1100,
                actual: V::NUMBER,
            })
    }
    /// Sets the layer's shader type, available from version 1100.
    pub fn try_set_shader_type(&mut self, value: ShaderType) -> Result<(), ValueError> {
        *self
            .shader_type
            .shader_type_mut()
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
    /// Returns the layer's shader type.
    pub fn shader_type(&self) -> ShaderType {
        self.try_shader_type().expect("supported version")
    }
    /// Sets the layer's shader type.
    pub fn set_shader_type(&mut self, value: ShaderType) {
        self.try_set_shader_type(value).expect("supported version");
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

impl<V: ModelVersion> Layer<V> {
    pub(crate) fn convert_with<T: ModelVersion>(
        &self,
        context: &mut ConversionContext<'_>,
        path: &str,
    ) -> Result<Layer<T>, ConversionError> {
        let mut target = Layer::<T>::new();
        target.filter_mode = self.filter_mode;
        target.shading_flags = self.shading_flags;
        target.texture_id = self.texture_id;
        target.texture_animation_id = self.texture_animation_id;
        target.coordinate_id = self.coordinate_id;
        target.alpha = self.alpha;

        context.field(
            self.emissive_gain.emissive_gain(),
            target.emissive_gain.emissive_gain_mut(),
            1.0,
            &format!("{path}.emissive_gain"),
        )?;
        context.field(
            self.fresnel.fresnel(),
            target.fresnel.fresnel_mut(),
            LayerFresnel::default(),
            &format!("{path}.fresnel"),
        )?;
        context.field(
            self.shader_type.shader_type(),
            target.shader_type.shader_type_mut(),
            ShaderType::SD_LEGACY,
            &format!("{path}.shader_type"),
        )?;
        context.field(
            self.texture_slots.texture_slots().map(<[_]>::to_vec),
            target.texture_slots.texture_slots_mut(),
            Vec::new(),
            &format!("{path}.texture_slots"),
        )?;
        for (index, track) in self.tracks.iter().enumerate() {
            let supported = V::NUMBER == T::NUMBER
                || match track {
                    LayerTrack::EmissiveGain(_) => T::NUMBER >= 900,
                    LayerTrack::FresnelColor(_)
                    | LayerTrack::FresnelOpacity(_)
                    | LayerTrack::FresnelTeamColor(_) => T::NUMBER >= 1000,
                    _ => true,
                };
            if supported {
                target.tracks.push(track.clone());
            } else {
                context.drop(
                    &format!("{path}.tracks[{index}]"),
                    "animation track is not supported by the target",
                )?;
            }
        }
        Ok(target)
    }
}

mod mdl_codec;
