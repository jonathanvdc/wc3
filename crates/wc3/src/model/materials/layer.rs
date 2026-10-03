//! Texture bindings, blend modes, and animated surface properties.
use super::{write_count, ShaderType};
use crate::model::conversion::ConversionContext;
use crate::model::Animatable;
use crate::model::{mdl, mdx};
use crate::model::{
    Color, Cursor, Encoder, ModelVersion, SupportsEmissiveGain, SupportsFresnel,
    SupportsLayerShaderTypeId, SupportsLayerTextureSlots, Tag, ValueError, Version,
};
use crate::model::{ConversionError, ConversionIssueKind};
use bitfield::bitfield;
use mdl_codec::{
    full, is_no_reference, is_white, no_reference, one, white, zero, zero_id, ShaderMarker,
    TextureBindings,
};
use std::{fmt::Debug, marker::PhantomData};

pub(super) const LAYER_TAG: Tag = *b"LAYS";

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
    /// Opaque rendering without alpha blending.
    None,
    #[mdx(value = 1)]
    /// Alpha-tested rendering.
    Transparent,
    #[mdx(value = 2)]
    /// Source-alpha blending.
    Blend,
    #[mdx(value = 3)]
    /// Additive blending.
    Additive,
    #[mdx(value = 4)]
    /// Additive blending weighted by source alpha.
    AddAlpha,
    #[mdx(value = 5)]
    /// Multiplicative blending.
    Modulate,
    #[mdx(value = 6)]
    /// Multiplicative blending with doubled contribution.
    Modulate2x,
    #[mdx(unknown)]
    #[mdl(unknown)]
    /// An unrecognized wire value preserved by MDX; unsupported in MDL.
    Unknown(u32),
}

bitfield! {
    /// Material layer shading bits, preserving unrecognized bits.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, mdl::Read, mdl::Write)]
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
    ), hive_skip_bits = 0x70c)]
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
    /// Animated index into the model texture collection.
    pub texture_id: Animatable<u32>,
    /// Authored texture-role identifier for this slot.
    pub texture_type: u32,
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
    write_order(filter_mode, shading_flags, shader, textures, texture_animation_id,
        coordinate_id, alpha, emissive, color, opacity, team_color),
    virtual_fields(

        #[mdl(property = "Shader", hive_name = "ShaderTypeId", delegate)]
        #[mdl(get = "Self::mdl_shader", set = "Self::set_mdl_shader")]
        shader: ShaderMarker,
        #[mdl(flatten)]
        #[mdl(get = "Self::mdl_textures", set = "Self::set_mdl_textures")]
        textures: TextureBindings,
        #[mdl(
            property = "EmissiveGain",
            default = "one",
            skip_if = "full"
        )]
        #[mdl(get = "Self::mdl_emissive", slot = "Self::mdl_emissive_mut")]
        emissive: Animatable<f32>,
        #[mdl(
            property = "FresnelColor",
            default = "white",
            skip_if = "is_white"
        )]
        #[mdl(get = "Self::mdl_color", slot = "Self::mdl_color_mut")]
        color: Animatable<Color>,
        #[mdl(
            property = "FresnelOpacity",
            default,
            skip_if = "zero"
        )]
        #[mdl(get = "Self::mdl_opacity", slot = "Self::mdl_opacity_mut")]
        opacity: Animatable<f32>,
        #[mdl(
            property = "FresnelTeamColor",
            default,
            skip_if = "zero"
        )]
        #[mdl(get = "Self::mdl_team_color", slot = "Self::mdl_team_color_mut")]
        team_color: Animatable<f32>,
    )
)]
pub struct Layer<V: ModelVersion> {
    #[mdl(skip, default)]
    version: PhantomData<V>,
    #[mdl(property = "FilterMode", default)]
    /// Blend filter mode.
    pub filter_mode: LayerFilterMode,
    #[mdl(flatten)]
    /// Layer shading bits.
    pub shading_flags: LayerShadingFlags,
    #[mdl(skip, default)]
    /// Index into the model texture collection when no texture-ID track is active.
    #[mdx(tag = *b"KMTF")]
    pub texture_id: Animatable<u32>,
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
    #[mdx(tag = *b"KMTA")]
    #[mdl(property = "Alpha", default = "one", skip_if = "full")]
    /// Opacity when no alpha track is active; 1.0 is fully opaque.
    pub alpha: Animatable<f32>,
    #[mdl(skip, default)]
    #[mdx(flatten)]
    emissive_gain: V::EmissiveGain,
    #[mdl(skip, default)]
    #[mdx(flatten)]
    fresnel: V::Fresnel,
    #[mdl(skip, default)]
    shader_type: V::ShaderType,
    #[mdl(skip, default)]
    texture_slots: V::TextureSlots,
}

/// Version-selected storage for the layer's emissive gain.
pub trait EmissiveGainField:
    Default + mdx::Read + mdx::Write + mdx::ReadTracks + mdx::WriteTracks + Clone + Debug + PartialEq
{
    fn emissive_gain(&self) -> Option<&Animatable<f32>> {
        None
    }
    fn emissive_gain_mut(&mut self) -> Option<&mut Animatable<f32>> {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write)]
pub struct NoEmissiveGain;
impl EmissiveGainField for NoEmissiveGain {}

#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write)]
pub struct EmissiveGain(#[mdx(tag = *b"KMTE")] Animatable<f32>);
impl Default for EmissiveGain {
    fn default() -> Self {
        Self(Animatable::Static(1.0))
    }
}
impl EmissiveGainField for EmissiveGain {
    fn emissive_gain(&self) -> Option<&Animatable<f32>> {
        Some(&self.0)
    }
    fn emissive_gain_mut(&mut self) -> Option<&mut Animatable<f32>> {
        Some(&mut self.0)
    }
}

/// Version-selected storage for the layer's fresnel.
pub trait FresnelField:
    Default + mdx::Read + mdx::Write + mdx::ReadTracks + mdx::WriteTracks + Clone + Debug + PartialEq
{
    fn fresnel(&self) -> Option<&LayerFresnel> {
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
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write)]
pub struct LayerFresnel {
    #[mdx(tag = *b"KFC3")]
    /// Animated Fresnel color.
    pub color: Animatable<Color>,
    #[mdx(tag = *b"KFCA")]
    /// Animated Fresnel opacity.
    pub opacity: Animatable<f32>,
    #[mdx(tag = *b"KFTC")]
    /// Animated team-color contribution to the Fresnel effect.
    pub team_color: Animatable<f32>,
}
impl Default for LayerFresnel {
    fn default() -> Self {
        Self {
            color: Animatable::Static([1.0; 3]),
            opacity: Animatable::Static(0.0),
            team_color: Animatable::Static(0.0),
        }
    }
}
impl FresnelField for LayerFresnel {
    fn fresnel(&self) -> Option<&LayerFresnel> {
        Some(self)
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
    fn read_mdx(cursor: &mut Cursor<'_>) -> Result<Self, mdx::ReadError> {
        let count = cursor.read::<u32>()? as usize;
        let mut texture_slots = Vec::new();
        for _ in 0..count {
            let mut texture_id: Animatable<u32> = cursor.read()?;
            let texture_type = cursor.read()?;
            while cursor.remaining().starts_with(b"KMTF") {
                cursor.read_bytes(4)?;
                texture_id.set_track(cursor.read()?);
            }
            texture_slots.push(LayerTextureSlot {
                texture_id,
                texture_type,
            });
        }
        Ok(Self(texture_slots))
    }
}

impl mdx::Write for LayerTextureSlots {
    fn write_mdx(&self, output: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        write_count(output, self.0.len(), LAYER_TAG)?;
        for slot in &self.0 {
            output.write(&slot.texture_id)?;
            output.write(&slot.texture_type)?;
            mdx::WriteTrackProperty::write_mdx_track_property(&slot.texture_id, *b"KMTF", output)?;
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
            texture_id: Animatable::Static(0),
            texture_animation_id: u32::MAX,
            coordinate_id: 0,
            alpha: Animatable::Static(1.0),
            emissive_gain: V::EmissiveGain::default(),
            fresnel: V::Fresnel::default(),
            shader_type: V::ShaderType::default(),
            texture_slots: V::TextureSlots::default(),
        }
    }

    /// Returns the MDX version used for this layer.
    pub fn version(&self) -> Version {
        V::NUMBER
    }

    /// Returns the layer's emissive gain, available from version 900.
    pub fn try_emissive_gain(&self) -> Result<Animatable<f32>, ValueError> {
        self.emissive_gain
            .emissive_gain()
            .cloned()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LAYER_TAG,
                minimum: 900,
                actual: V::NUMBER,
            })
    }
    /// Sets the layer's emissive gain, available from version 900.
    pub fn try_set_emissive_gain(&mut self, value: Animatable<f32>) -> Result<(), ValueError> {
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
            .cloned()
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
    pub fn try_fresnel_color(&self) -> Result<Animatable<Color>, ValueError> {
        Ok(self.try_fresnel()?.color)
    }
    /// Sets the Fresnel color, available from version 1000.
    pub fn try_set_fresnel_color(&mut self, value: Animatable<Color>) -> Result<(), ValueError> {
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
    pub fn try_fresnel_opacity(&self) -> Result<Animatable<f32>, ValueError> {
        Ok(self.try_fresnel()?.opacity)
    }
    /// Sets the Fresnel opacity, available from version 1000.
    pub fn try_set_fresnel_opacity(&mut self, value: Animatable<f32>) -> Result<(), ValueError> {
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
    pub fn try_fresnel_team_color(&self) -> Result<Animatable<f32>, ValueError> {
        Ok(self.try_fresnel()?.team_color)
    }
    /// Sets the Fresnel team color, available from version 1000.
    pub fn try_set_fresnel_team_color(&mut self, value: Animatable<f32>) -> Result<(), ValueError> {
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
    pub fn emissive_gain(&self) -> Animatable<f32> {
        self.try_emissive_gain().expect("supported version")
    }
    /// Sets the layer's emissive gain.
    pub fn set_emissive_gain(&mut self, value: Animatable<f32>) {
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
    pub fn fresnel_color(&self) -> Animatable<Color> {
        self.try_fresnel_color().expect("supported version")
    }
    /// Sets the layer's fresnel color.
    pub fn set_fresnel_color(&mut self, value: Animatable<Color>) {
        self.try_set_fresnel_color(value)
            .expect("supported version");
    }
}

impl<V: SupportsFresnel> Layer<V> {
    /// Returns the layer's fresnel opacity.
    pub fn fresnel_opacity(&self) -> Animatable<f32> {
        self.try_fresnel_opacity().expect("supported version")
    }
    /// Sets the layer's fresnel opacity.
    pub fn set_fresnel_opacity(&mut self, value: Animatable<f32>) {
        self.try_set_fresnel_opacity(value)
            .expect("supported version");
    }
}

impl<V: SupportsFresnel> Layer<V> {
    /// Returns the layer's fresnel team color.
    pub fn fresnel_team_color(&self) -> Animatable<f32> {
        self.try_fresnel_team_color().expect("supported version")
    }
    /// Sets the layer's fresnel team color.
    pub fn set_fresnel_team_color(&mut self, value: Animatable<f32>) {
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
        target.texture_id = self.texture_id.clone();
        target.texture_animation_id = self.texture_animation_id;
        target.coordinate_id = self.coordinate_id;
        target.alpha = self.alpha.clone();

        context.field(
            self.emissive_gain.emissive_gain().cloned(),
            target.emissive_gain.emissive_gain_mut(),
            Animatable::Static(1.0),
            &format!("{path}.emissive_gain"),
        )?;
        context.field(
            self.fresnel.fresnel().cloned(),
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
        let downgrade_diffuse = V::NUMBER >= 1100
            && T::NUMBER < 1100
            && self.texture_id == Animatable::Static(0)
            && self
                .texture_slots
                .texture_slots()
                .is_some_and(|slots| slots.len() == 1 && slots[0].texture_type == 0);
        if V::NUMBER < 1100 && T::NUMBER >= 1100 {
            *target
                .texture_slots
                .texture_slots_mut()
                .expect("target texture slots") = vec![LayerTextureSlot {
                texture_id: self.texture_id.clone(),
                texture_type: 0,
            }];
            target.texture_id = Animatable::Static(0);
            context.issue(
                &format!("{path}.texture_slots"),
                ConversionIssueKind::Normalized,
                "moved legacy texture binding and animation into the diffuse slot",
            );
        } else if downgrade_diffuse {
            target.texture_id = self.texture_slots.texture_slots().expect("source slots")[0]
                .texture_id
                .clone();
            context.issue(
                &format!("{path}.texture_slots"),
                ConversionIssueKind::Normalized,
                "moved diffuse slot binding and animation into legacy texture storage",
            );
        } else {
            context.field(
                self.texture_slots.texture_slots().map(<[_]>::to_vec),
                target.texture_slots.texture_slots_mut(),
                Vec::new(),
                &format!("{path}.texture_slots"),
            )?;
        }
        Ok(target)
    }
}

mod mdl_codec;
