//! Materials, shader fields, and version-specific layouts.
use crate::model::conversion::ConversionContext;
use crate::model::mdx;
use crate::model::ConversionError;
use bitfield::bitfield;
use std::{borrow::Cow, fmt::Debug, marker::PhantomData};

use super::layer::{
    EmissiveGain, EmissiveGainField, FresnelField, Layer, LayerFresnel, LayerShaderType,
    LayerShaderTypeField, LayerTextureSlots, LayerTextureSlotsField, NoEmissiveGain, NoFresnel,
    NoLayerShaderType, NoLayerTextureSlots, LAYER_TAG,
};
use super::write_count;
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

/// A material with directly accessible layers and an exact shader field.
#[derive(Clone, Debug, PartialEq)]
pub struct Material<V: ModelVersion> {
    version: PhantomData<V>,
    priority_plane: i32,
    render_mode: MaterialRenderFlags,
    shader: V::Shader,
    layers: Vec<Layer<V>>,
}

/// The fixed shader field selected by a model version.
pub trait ShaderField: Default + mdx::Read + mdx::Write + Clone + Debug + PartialEq {
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
    type ShaderTypeId: LayerShaderTypeField;
    type TextureSlots: LayerTextureSlotsField;
}

use crate::model::{V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900};

impl MaterialLayout for V800 {
    type Shader = NoShader;
    type EmissiveGain = NoEmissiveGain;
    type Fresnel = NoFresnel;
    type ShaderTypeId = NoLayerShaderType;
    type TextureSlots = NoLayerTextureSlots;
}
impl MaterialLayout for V900 {
    type Shader = ShaderText;
    type EmissiveGain = EmissiveGain;
    type Fresnel = NoFresnel;
    type ShaderTypeId = NoLayerShaderType;
    type TextureSlots = NoLayerTextureSlots;
}
impl MaterialLayout for V1000 {
    type Shader = ShaderText;
    type EmissiveGain = EmissiveGain;
    type Fresnel = LayerFresnel;
    type ShaderTypeId = NoLayerShaderType;
    type TextureSlots = NoLayerTextureSlots;
}
impl MaterialLayout for V1100 {
    type Shader = NoShader;
    type EmissiveGain = EmissiveGain;
    type Fresnel = LayerFresnel;
    type ShaderTypeId = LayerShaderType;
    type TextureSlots = LayerTextureSlots;
}
impl MaterialLayout for V1200 {
    type Shader = NoShader;
    type EmissiveGain = EmissiveGain;
    type Fresnel = LayerFresnel;
    type ShaderTypeId = LayerShaderType;
    type TextureSlots = LayerTextureSlots;
}
impl MaterialLayout for V1300 {
    type Shader = NoShader;
    type EmissiveGain = EmissiveGain;
    type Fresnel = LayerFresnel;
    type ShaderTypeId = LayerShaderType;
    type TextureSlots = LayerTextureSlots;
}
impl MaterialLayout for V1400 {
    type Shader = NoShader;
    type EmissiveGain = EmissiveGain;
    type Fresnel = LayerFresnel;
    type ShaderTypeId = LayerShaderType;
    type TextureSlots = LayerTextureSlots;
}
impl MaterialLayout for V1600 {
    type Shader = NoShader;
    type EmissiveGain = EmissiveGain;
    type Fresnel = LayerFresnel;
    type ShaderTypeId = LayerShaderType;
    type TextureSlots = LayerTextureSlots;
}
impl MaterialLayout for V1800 {
    type Shader = NoShader;
    type EmissiveGain = EmissiveGain;
    type Fresnel = LayerFresnel;
    type ShaderTypeId = LayerShaderType;
    type TextureSlots = LayerTextureSlots;
}

fn expect_tag(cursor: &mut Cursor<'_>, expected: Tag, record_tag: Tag) -> Result<(), ReadError> {
    let offset = cursor.absolute_position();
    if cursor.read_exact(4)? == expected {
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
    /// Returns the material priority plane.
    pub fn priority_plane(&self) -> i32 {
        self.priority_plane
    }
    /// Changes the material priority plane.
    pub fn set_priority_plane(&mut self, value: i32) {
        self.priority_plane = value;
    }
    /// Returns decoded rendering flags.
    pub fn render_mode(&self) -> MaterialRenderFlags {
        self.render_mode
    }
    /// Changes the rendering flags.
    pub fn set_render_mode(&mut self, value: MaterialRenderFlags) {
        self.render_mode = value;
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
    /// Borrows layers without decoding or allocating.
    pub fn layers(&self) -> &[Layer<V>] {
        &self.layers
    }
    /// Mutably borrows layers for bulk edits.
    pub fn layers_mut(&mut self) -> &mut [Layer<V>] {
        &mut self.layers
    }
    /// Replaces all layers.
    pub fn set_layers(&mut self, layers: &[Layer<V>]) {
        self.layers = layers.to_vec();
    }
}

impl<V: ModelVersion> Default for Material<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: ModelVersion> Model<V> {
    /// Decodes all `MTLS` records in file order.
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
        let mut cursor = source.slice_u32_sized()?;
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
