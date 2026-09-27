//! Typed material layers and versioned texture slots.
use std::fmt::Debug;
crate::animation::track_group! {
    pub enum LayerTrack {
        Alpha: LayerAlpha,
        TextureId: LayerTextureId,
        EmissiveGain: LayerEmissiveGain,
        FresnelColor: LayerFresnelColor,
        FresnelOpacity: LayerFresnelOpacity,
        FresnelTeamColor: LayerFresnelTeamColor,
    }
}

use crate::EncodeError;
use crate::Encoder;
use crate::KnownChunk;
use crate::ValueError;
use crate::{Color, LayerTextureId, ModelVersion, Tag, TrackTag, Version};
use std::marker::PhantomData;

use crate::{Cursor, MaterialsChunk};
use crate::{Decodable, Encodable};
use std::borrow::Cow;

use crate::FixedText;
use crate::{AnimationTrack, DecodeError, Model};

const LAYER_TAG: Tag = *b"LAYS";

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
    pub track: Option<AnimationTrack<LayerTextureId>>,
}

/// A material with directly accessible layers and an exact shader field.
#[derive(Clone, Debug, PartialEq)]
pub struct Material<V: ModelVersion> {
    version: PhantomData<V>,
    priority_plane: u32,
    render_mode: u32,
    shader: V::Shader,
    layers: Vec<Layer<V>>,
}

/// A material layer with parsed texture slots and animation tracks.
#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
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

/// The fixed shader field selected by a model version.
pub trait ShaderField: Clone + Debug + PartialEq {
    fn empty() -> Self;
    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError>;
    fn encode(&self, output: &mut Encoder<'_>);
    fn text(&self) -> Option<Cow<'_, str>>;
    fn set(&mut self, text: &str) -> Result<(), ValueError>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct NoShader;

impl ShaderField for NoShader {
    fn empty() -> Self {
        Self
    }
    fn decode(_: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        Ok(Self)
    }
    fn encode(&self, _: &mut Encoder<'_>) {}
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

#[derive(Clone, Debug, PartialEq)]
pub struct ShaderText(FixedText<80>);

impl ShaderField for ShaderText {
    fn empty() -> Self {
        Self(FixedText::default())
    }
    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        Ok(Self(cursor.read()?))
    }
    fn encode(&self, output: &mut Encoder<'_>) {
        output.write(&self.0);
    }
    fn text(&self) -> Option<Cow<'_, str>> {
        Some(self.0.text())
    }
    fn set(&mut self, text: &str) -> Result<(), ValueError> {
        self.0.set_text(text)
    }
}

/// The fields following a layer's shared header.
pub trait LayerExtra: Clone + Debug + PartialEq {
    fn empty() -> Self;
    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError>;
    fn encode(&self, output: &mut Encoder<'_>) -> Result<(), EncodeError>;
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

#[derive(Clone, Debug, PartialEq)]
pub struct ClassicLayerExtra;

impl LayerExtra for ClassicLayerExtra {
    fn empty() -> Self {
        Self
    }
    fn decode(_: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        Ok(Self)
    }
    fn encode(&self, _: &mut Encoder<'_>) -> Result<(), EncodeError> {
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Layer900Extra {
    emissive_gain: f32,
}

impl LayerExtra for Layer900Extra {
    fn empty() -> Self {
        Self { emissive_gain: 1.0 }
    }
    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            emissive_gain: cursor.read()?,
        })
    }
    fn encode(&self, output: &mut Encoder<'_>) -> Result<(), EncodeError> {
        output.write(self.emissive_gain);
        Ok(())
    }
    fn emissive_gain(&self) -> Option<f32> {
        Some(self.emissive_gain)
    }
    fn emissive_gain_mut(&mut self) -> Option<&mut f32> {
        Some(&mut self.emissive_gain)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Layer1000Extra {
    base: Layer900Extra,
    fresnel: Fresnel,
}

impl LayerExtra for Layer1000Extra {
    fn empty() -> Self {
        Self {
            base: Layer900Extra::empty(),
            fresnel: Fresnel::default(),
        }
    }
    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            base: Layer900Extra::decode(cursor)?,
            fresnel: Fresnel {
                color: cursor.read()?,
                opacity: cursor.read()?,
                team_color: cursor.read()?,
            },
        })
    }
    fn encode(&self, output: &mut Encoder<'_>) -> Result<(), EncodeError> {
        self.base.encode(output)?;
        output.write(self.fresnel.color);
        output.write(self.fresnel.opacity);
        output.write(self.fresnel.team_color);
        Ok(())
    }
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

#[derive(Clone, Debug, PartialEq)]
pub struct Layer1100Extra {
    base: Layer1000Extra,
    shader_type_id: u32,
    texture_slots: Vec<LayerTextureSlot>,
}

impl LayerExtra for Layer1100Extra {
    fn empty() -> Self {
        Self {
            base: Layer1000Extra::empty(),
            shader_type_id: 0,
            texture_slots: Vec::new(),
        }
    }
    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        let base = Layer1000Extra::decode(cursor)?;
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
    fn encode(&self, output: &mut Encoder<'_>) -> Result<(), EncodeError> {
        self.base.encode(output)?;
        output.write(self.shader_type_id);
        write_count(output, self.texture_slots.len(), LAYER_TAG)?;
        for slot in &self.texture_slots {
            output.write(slot.texture_id);
            output.write(slot.texture_type);
            if let Some(track) = &slot.track {
                output.write(track);
            }
        }
        Ok(())
    }
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

/// Chooses the material and layer fields for a version.
pub trait MaterialLayout {
    type Shader: ShaderField;
    type LayerExtra: LayerExtra;
}

use crate::{V1000, V1100, V1200, V1800, V800, V900};

impl MaterialLayout for V800 {
    type Shader = NoShader;
    type LayerExtra = ClassicLayerExtra;
}
impl MaterialLayout for V900 {
    type Shader = ShaderText;
    type LayerExtra = Layer900Extra;
}
impl MaterialLayout for V1000 {
    type Shader = ShaderText;
    type LayerExtra = Layer1000Extra;
}
impl MaterialLayout for V1100 {
    type Shader = NoShader;
    type LayerExtra = Layer1100Extra;
}
impl MaterialLayout for V1200 {
    type Shader = NoShader;
    type LayerExtra = Layer1100Extra;
}
impl MaterialLayout for V1800 {
    type Shader = NoShader;
    type LayerExtra = Layer1100Extra;
}

fn expect_tag(cursor: &mut Cursor<'_>, expected: Tag, record_tag: Tag) -> Result<(), DecodeError> {
    let offset = cursor.absolute_position();
    if cursor.read_exact(4)? == expected {
        Ok(())
    } else {
        Err(DecodeError::MalformedRecord {
            tag: record_tag,
            offset,
        })
    }
}

fn write_count(bytes: &mut Encoder<'_>, count: usize, tag: Tag) -> Result<(), EncodeError> {
    let value =
        u32::try_from(count).map_err(|_| EncodeError::ChunkTooLarge { tag, size: count })?;
    bytes.write(value);
    Ok(())
}

impl<V: ModelVersion> Material<V> {
    /// Creates an empty material using `V`'s fields.
    pub fn new() -> Self {
        Self {
            version: PhantomData,
            priority_plane: 0,
            render_mode: 0,
            shader: V::Shader::empty(),
            layers: Vec::new(),
        }
    }

    /// Returns the MDX version used for this material.
    pub fn version(&self) -> Version {
        V::NUMBER
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
        self.shader.text()
    }
    /// Changes the shader path and clears unused bytes.
    pub fn set_shader(&mut self, shader: &str) -> Result<(), ValueError> {
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
            extensions: V::LayerExtra::empty(),
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
    pub fn emissive_gain(&self) -> Option<f32> {
        self.extensions.emissive_gain()
    }
    pub fn set_emissive_gain(&mut self, value: f32) -> Result<(), ValueError> {
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
    pub fn fresnel_color(&self) -> Option<Color> {
        self.extensions.fresnel().map(|f| f.color)
    }
    pub fn set_fresnel_color(&mut self, value: Color) -> Result<(), ValueError> {
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
    pub fn fresnel_opacity(&self) -> Option<f32> {
        self.extensions.fresnel().map(|f| f.opacity)
    }
    pub fn set_fresnel_opacity(&mut self, value: f32) -> Result<(), ValueError> {
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
    pub fn fresnel_team_color(&self) -> Option<f32> {
        self.extensions.fresnel().map(|f| f.team_color)
    }
    pub fn set_fresnel_team_color(&mut self, value: f32) -> Result<(), ValueError> {
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
    pub fn shader_type_id(&self) -> Option<u32> {
        self.extensions.shader_type_id()
    }
    pub fn set_shader_type_id(&mut self, value: u32) -> Result<(), ValueError> {
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
    pub fn texture_slots(&self) -> &[LayerTextureSlot] {
        self.extensions.texture_slots()
    }
    pub fn set_texture_slots(&mut self, slots: &[LayerTextureSlot]) -> Result<(), ValueError> {
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

impl<V: ModelVersion> Decodable for Material<V> {
    fn decode_one(source: &mut Cursor<'_>, _version: Version) -> Result<Self, DecodeError> {
        let mut cursor = source.slice_u32_sized()?;
        let value = {
            let priority_plane = cursor.read()?;
            let render_mode = cursor.read()?;
            let shader = V::Shader::decode(&mut cursor)?;
            expect_tag(&mut cursor, LAYER_TAG, MaterialsChunk::<V>::TAG)?;
            let count = cursor.read::<u32>()? as usize;
            let mut layers = Vec::new();
            for _ in 0..count {
                let layer = Layer::<V>::decode_one(&mut cursor, V::NUMBER)?;
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

impl<V: ModelVersion> Encodable for Material<V> {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        let marker = bytes.begin_sized();
        bytes.write(self.priority_plane);
        bytes.write(self.render_mode);
        self.shader.encode(bytes);
        bytes.write_bytes(b"LAYS");
        write_count(bytes, self.layers.len(), MaterialsChunk::<V>::TAG)?;
        for layer in &self.layers {
            layer.encode_to(bytes)?;
        }
        bytes.finish_sized(marker, MaterialsChunk::<V>::TAG)?;
        Ok(())
    }
}

impl<V: ModelVersion> Decodable for Layer<V> {
    fn decode_one(source: &mut Cursor<'_>, _version: Version) -> Result<Self, DecodeError> {
        let mut cursor = source.slice_u32_sized()?;
        let value = {
            let filter_mode = cursor.read()?;
            let shading_flags = cursor.read()?;
            let texture_id = cursor.read()?;
            let texture_animation_id = cursor.read()?;
            let coordinate_id = cursor.read()?;
            let alpha = cursor.read()?;
            let extensions = V::LayerExtra::decode(&mut cursor)?;
            let mut tracks = Vec::new();
            while !cursor.remaining().is_empty() {
                tracks.push(cursor.read::<LayerTrack>()?);
            }
            Ok(Self {
                version: PhantomData,
                filter_mode,
                shading_flags,
                texture_id,
                texture_animation_id,
                coordinate_id,
                alpha,
                extensions,
                tracks,
            })
        }?;
        cursor.finish()?;
        Ok(value)
    }
}

impl<V: ModelVersion> Encodable for Layer<V> {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        let marker = bytes.begin_sized();
        for word in [
            self.filter_mode,
            self.shading_flags,
            self.texture_id,
            self.texture_animation_id,
            self.coordinate_id,
        ] {
            bytes.write(word);
        }
        bytes.write(self.alpha);
        self.extensions.encode(bytes)?;
        for track in &self.tracks {
            bytes.write(track);
        }
        bytes.finish_sized(marker, LAYER_TAG)?;
        Ok(())
    }
}
