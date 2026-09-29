//! Ordered rendering layers and material settings.
use crate::model::conversion::ConversionContext;
use crate::model::{mdl, mdx};
use crate::model::{ConversionError, ConversionIssueKind};
use bitfield::bitfield;
use mdl_codec::zero_priority;
use std::{borrow::Cow, fmt::Debug, marker::PhantomData};

use super::layer::{
    EmissiveGain, EmissiveGainField, FresnelField, Layer, LayerFresnel, LayerShaderTypeField,
    LayerTextureSlot, LayerTextureSlots, LayerTextureSlotsField, NoEmissiveGain, NoFresnel,
    NoLayerShaderType, NoLayerTextureSlots, LAYER_TAG,
};
use super::{write_count, ShaderType};
use crate::model::{
    Cursor, Encoder, FixedText, KnownChunk, MaterialsChunk, Model, ModelVersion, ReadError,
    SupportsMaterialShaderPath, Tag, ValueError, Version, WriteError,
};

bitfield! {
    /// Material rendering bits, preserving unrecognized bits.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, mdx::Read, mdx::Write)]
    pub struct MaterialRenderFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the `CONSTANT_COLOR` bit.
    pub constant_color, set_constant_color: 0;
    /// Returns or changes the historical `TWO_SIDED` bit.
    pub two_sided, set_two_sided: 1;
    /// Returns or changes the historical `SORT_PRIMITIVES_NEAR_Z` bit.
    pub sort_primitives_near_z, set_sort_primitives_near_z: 3;
    /// Returns or changes the `SORT_PRIMITIVES_FAR_Z` bit.
    pub sort_primitives_far_z, set_sort_primitives_far_z: 4;
    /// Returns or changes the `FULL_RESOLUTION` bit.
    pub full_resolution, set_full_resolution: 5;
}

/// Surface appearance composed of ordered rendering layers.
#[derive(Clone, Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(
    block = "Material",
    write_order(priority_plane, render_mode, unfogged, shader, layers),
    virtual_fields(
        #[mdl(flag = "Unfogged", default)]
        #[mdl(get = "Self::mdl_unfogged", set = "Self::set_mdl_unfogged")]
        unfogged: bool
    )
)]
pub struct Material<V: ModelVersion> {
    #[mdl(skip, default)]
    version: PhantomData<V>,
    #[mdl(property = "PriorityPlane", default, skip_if = "zero_priority")]
    /// Signed render-order priority.
    pub priority_plane: i32,
    #[mdl(flags(
        ConstantColor = 1,
        TwoSided = 2,
        SortPrimsNearZ = 8,
        SortPrimsFarZ = 16,
        FullResolution = 32
    ))]
    #[mdl(hive_flags(SortPrimitives = 16), hive_skip_bits = 8)]
    /// Rendering flags.
    pub render_mode: MaterialRenderFlags,
    #[mdl(property = "Shader", delegate)]
    shader: V::Shader,
    #[mdl(repeated = "Layer")]
    /// Rendering layers, in draw order.
    pub layers: Vec<Layer<V>>,
}

/// The fixed shader field selected by a model version.
pub trait ShaderField:
    Default
    + mdx::Read
    + mdx::Write
    + mdl::ReadProperty
    + mdl::WriteProperty
    + Clone
    + Debug
    + PartialEq
{
    fn text(&self) -> Option<Cow<'_, str>>;
    fn fixed_text(&self) -> Option<&FixedText<80>> {
        None
    }
    fn fixed_text_mut(&mut self) -> Option<&mut FixedText<80>> {
        None
    }
    fn set(&mut self, text: &str) -> Result<(), ValueError>;
}

#[derive(Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write)]
pub struct NoShader;

impl ShaderField for NoShader {
    fn text(&self) -> Option<Cow<'_, str>> {
        None
    }
    fn set(&mut self, _: &str) -> Result<(), ValueError> {
        Err(ValueError::UnavailableField {
            tag: *b"MTLS",
            field: "shader",
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write)]
pub struct ShaderText(FixedText<80>);

impl ShaderField for ShaderText {
    fn fixed_text(&self) -> Option<&FixedText<80>> {
        Some(&self.0)
    }
    fn fixed_text_mut(&mut self) -> Option<&mut FixedText<80>> {
        Some(&mut self.0)
    }
    fn text(&self) -> Option<Cow<'_, str>> {
        Some(self.0.text())
    }
    fn set(&mut self, text: &str) -> Result<(), ValueError> {
        self.0.set_text(text)
    }
}

/// Chooses the material and layer fields for a version.
pub trait MaterialLayout {
    type Shader: ShaderField;
    type EmissiveGain: EmissiveGainField;
    type Fresnel: FresnelField;
    type ShaderType: LayerShaderTypeField;
    type TextureSlots: LayerTextureSlotsField;
}

use crate::model::{V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900};

impl MaterialLayout for V800 {
    type Shader = NoShader;
    type EmissiveGain = NoEmissiveGain;
    type Fresnel = NoFresnel;
    type ShaderType = NoLayerShaderType;
    type TextureSlots = NoLayerTextureSlots;
}
impl MaterialLayout for V900 {
    type Shader = ShaderText;
    type EmissiveGain = EmissiveGain;
    type Fresnel = NoFresnel;
    type ShaderType = NoLayerShaderType;
    type TextureSlots = NoLayerTextureSlots;
}
impl MaterialLayout for V1000 {
    type Shader = ShaderText;
    type EmissiveGain = EmissiveGain;
    type Fresnel = LayerFresnel;
    type ShaderType = NoLayerShaderType;
    type TextureSlots = NoLayerTextureSlots;
}
impl MaterialLayout for V1100 {
    type Shader = NoShader;
    type EmissiveGain = EmissiveGain;
    type Fresnel = LayerFresnel;
    type ShaderType = ShaderType;
    type TextureSlots = LayerTextureSlots;
}
impl MaterialLayout for V1200 {
    type Shader = NoShader;
    type EmissiveGain = EmissiveGain;
    type Fresnel = LayerFresnel;
    type ShaderType = ShaderType;
    type TextureSlots = LayerTextureSlots;
}
impl MaterialLayout for V1300 {
    type Shader = NoShader;
    type EmissiveGain = EmissiveGain;
    type Fresnel = LayerFresnel;
    type ShaderType = ShaderType;
    type TextureSlots = LayerTextureSlots;
}
impl MaterialLayout for V1400 {
    type Shader = NoShader;
    type EmissiveGain = EmissiveGain;
    type Fresnel = LayerFresnel;
    type ShaderType = ShaderType;
    type TextureSlots = LayerTextureSlots;
}
impl MaterialLayout for V1600 {
    type Shader = NoShader;
    type EmissiveGain = EmissiveGain;
    type Fresnel = LayerFresnel;
    type ShaderType = ShaderType;
    type TextureSlots = LayerTextureSlots;
}
impl MaterialLayout for V1800 {
    type Shader = NoShader;
    type EmissiveGain = EmissiveGain;
    type Fresnel = LayerFresnel;
    type ShaderType = ShaderType;
    type TextureSlots = LayerTextureSlots;
}

fn expect_tag(cursor: &mut Cursor<'_>, expected: Tag, record_tag: Tag) -> Result<(), ReadError> {
    let offset = cursor.absolute_position();
    if cursor.read_bytes(4)? == expected {
        Ok(())
    } else {
        Err(ReadError::MalformedRecord {
            tag: record_tag,
            offset,
        })
    }
}

impl<V: ModelVersion> Material<V> {
    /// Creates an empty material using `V`'s fields.
    pub fn new() -> Self {
        Self {
            version: PhantomData,
            priority_plane: 0,
            render_mode: MaterialRenderFlags::default(),
            shader: V::Shader::default(),
            layers: Vec::new(),
        }
    }

    /// Returns the MDX version used for this material.
    pub fn version(&self) -> Version {
        V::NUMBER
    }

    /// Returns the shader path in versions 900 through 1099.
    pub fn try_shader(&self) -> Result<Cow<'_, str>, ValueError> {
        if !(matches!(V::NUMBER, 900 | 1000)) {
            return Err(ValueError::UnsupportedField {
                tag: *b"MTLS",
                field: "shader",
                actual: V::NUMBER,
            });
        }
        Ok(self.shader.text().expect("supported version"))
    }
    /// Changes the shader path and clears unused bytes.
    pub fn try_set_shader(&mut self, shader: &str) -> Result<(), ValueError> {
        if !matches!(V::NUMBER, 900 | 1000) {
            return Err(ValueError::UnsupportedField {
                tag: *b"MTLS",
                field: "shader",
                actual: V::NUMBER,
            });
        }
        self.shader.set(shader)
    }
}

impl<V: ModelVersion> Default for Material<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: ModelVersion> Model<V> {
    /// Returns owned copies of `MTLS` records in file order.
    pub fn materials(&self) -> Vec<Material<V>> {
        self.collect_chunk_records::<MaterialsChunk<V>>()
    }

    /// Replaces all material records in the first `MTLS` chunk.
    pub fn set_materials(&mut self, materials: &[Material<V>]) {
        self.replace_chunk(MaterialsChunk::new(materials.to_vec()));
    }
}

impl<V: ModelVersion> mdx::Read for Material<V> {
    fn read_mdx(source: &mut Cursor<'_>) -> Result<Self, ReadError> {
        let mut cursor = source.subcursor_u32_sized()?;
        let value = {
            let priority_plane = cursor.read()?;
            let render_mode = cursor.read()?;
            let shader = cursor.read::<V::Shader>()?;
            expect_tag(&mut cursor, LAYER_TAG, MaterialsChunk::<V>::TAG)?;
            let count = cursor.read::<u32>()? as usize;
            let mut layers = Vec::new();
            for _ in 0..count {
                let layer = cursor.read()?;
                layers.push(layer);
            }
            Ok(Self {
                version: PhantomData,
                priority_plane,
                render_mode,
                shader,
                layers,
            })
        }?;
        cursor.finish()?;
        Ok(value)
    }
}

impl<V: ModelVersion> mdx::Write for Material<V> {
    fn write_mdx(&self, bytes: &mut Encoder<'_>) -> Result<(), WriteError> {
        let marker = bytes.begin_sized();
        bytes.write(&(self.priority_plane))?;
        bytes.write(&(self.render_mode))?;
        bytes.write(&self.shader)?;
        bytes.write_bytes(b"LAYS");
        write_count(bytes, self.layers.len(), MaterialsChunk::<V>::TAG)?;
        for layer in &self.layers {
            bytes.write(layer)?;
        }
        bytes.finish_sized(marker, MaterialsChunk::<V>::TAG)?;
        Ok(())
    }
}

impl<V: SupportsMaterialShaderPath> Material<V> {
    pub fn shader(&self) -> Cow<'_, str> {
        self.shader.text().expect("supported version")
    }
    pub fn set_shader(&mut self, shader: &str) -> Result<(), ValueError> {
        self.try_set_shader(shader)
    }
}

impl<V: ModelVersion> Material<V> {
    pub(crate) fn convert_with<T: ModelVersion>(
        &self,
        context: &mut ConversionContext<'_>,
        path: &str,
    ) -> Result<Material<T>, ConversionError> {
        let mut target = Material::<T>::new();
        target.priority_plane = self.priority_plane;
        target.render_mode = self.render_mode;
        if matches!(V::NUMBER, 900 | 1000) && T::NUMBER >= 1100 {
            let shader = self
                .shader
                .fixed_text()
                .and_then(|name| ShaderType::from_name(&name.text()));
            if let Some(shader) = shader {
                let hd = shader == ShaderType::HD_DEFAULT_UNIT || shader == ShaderType::HD_CRYSTAL;
                if hd && self.layers.len() != 6 {
                    context.drop(
                        &format!("{path}.shader"),
                        "HD shader upgrade requires six texture-role layers",
                    )?;
                } else {
                    if hd {
                        for (index, layer) in self.layers.iter().enumerate().skip(1) {
                            if layer.alpha.track().is_some()
                                || layer
                                    .try_emissive_gain()
                                    .is_ok_and(|value| value.track().is_some())
                                || layer.try_fresnel().is_ok_and(|value| {
                                    value.color.track().is_some()
                                        || value.opacity.track().is_some()
                                        || value.team_color.track().is_some()
                                })
                                || layer.coordinate_id != self.layers[0].coordinate_id
                                || layer.texture_animation_id != self.layers[0].texture_animation_id
                            {
                                return Err(context.error(&format!("{path}.layers[{index}]"),
                                    "HD texture-role layer has independent animation or UV settings"));
                            }
                        }
                        let mut slots = Vec::new();
                        for (index, layer) in self.layers.iter().enumerate() {
                            let converted = layer
                                .convert_with::<T>(context, &format!("{path}.layers[{index}]"))?;
                            let mut slot: LayerTextureSlot =
                                converted.try_texture_slots().expect("target slots")[0].clone();
                            slot.texture_type = index as u32;
                            slots.push(slot);
                            if index == 0 {
                                target.layers.push(converted);
                            }
                        }
                        let layer = &mut target.layers[0];
                        layer.try_set_texture_slots(&slots).expect("target slots");
                        layer.try_set_shader_type(shader).expect("target shader");
                        if self.render_mode.two_sided() {
                            layer.shading_flags.set_two_sided(true);
                        }
                    } else {
                        for (index, layer) in self.layers.iter().enumerate() {
                            let mut layer = layer
                                .convert_with::<T>(context, &format!("{path}.layers[{index}]"))?;
                            layer.try_set_shader_type(shader).expect("target shader");
                            target.layers.push(layer);
                        }
                    }
                    context.issue(&format!("{path}.shader"), ConversionIssueKind::Normalized,
                        if hd { "upgraded six texture-role layers into one HD layer using the first layer's rendering properties" }
                        else { "moved material shader assignment into layer shader fields" });
                    return Ok(target);
                }
            }
        }
        context.field(
            self.shader.fixed_text().copied(),
            target.shader.fixed_text_mut(),
            FixedText::default(),
            &format!("{path}.shader"),
        )?;
        target.layers = self
            .layers
            .iter()
            .enumerate()
            .map(|(i, layer)| layer.convert_with::<T>(context, &format!("{path}.layers[{i}]")))
            .collect::<Result<_, _>>()?;
        Ok(target)
    }
}

mod mdl_codec;
