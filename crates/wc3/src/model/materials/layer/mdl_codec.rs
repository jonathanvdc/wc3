//! Derived layer syntax with a focused adapter for slot-qualified textures.
use super::{
    EmissiveGainField, FresnelField, Layer, LayerFresnel, LayerShaderTypeField, LayerTextureSlot,
    LayerTextureSlotsField, LayerTrack, ShaderType,
};
use crate::model::mdl::{
    Field, MdlWriter, Parser, ReadErrorKind, ReadFields, Span, TokenKind, WriteFields,
};
use crate::model::{mdl, FixedText};
use crate::model::{AnimationTrack, Color, LayerTextureId, ModelVersion};
use std::io::Write as IoWrite;
use std::marker::PhantomData;

#[derive(Clone, Copy, mdl::Read, mdl::Write)]
#[mdl(value)]
enum FilterMode {
    None,
    Transparent,
    Blend,
    Additive,
    AddAlpha,
    Modulate,
    Modulate2x,
}
fn read_filter(parser: &mut Parser<'_>) -> Result<u32, mdl::ReadError> {
    Ok(parser.read::<FilterMode>()? as u32)
}
fn write_filter<W: IoWrite>(value: &u32, writer: &mut MdlWriter<W>) -> Result<(), mdl::WriteError> {
    let mode = match value {
        0 => FilterMode::None,
        1 => FilterMode::Transparent,
        2 => FilterMode::Blend,
        3 => FilterMode::Additive,
        4 => FilterMode::AddAlpha,
        5 => FilterMode::Modulate,
        6 => FilterMode::Modulate2x,
        _ => return Err(mdl::WriteError::Unsupported("filter mode")),
    };
    writer.write(&mode)
}
fn one() -> f32 {
    1.0
}
fn white() -> Color {
    [1.0; 3]
}
fn is_no_reference(value: &u32) -> bool {
    *value == u32::MAX
}
fn zero_id(value: &u32) -> bool {
    *value == 0
}
fn full(value: &f32) -> bool {
    value.to_bits() == 1.0f32.to_bits()
}
fn zero(value: &f32) -> bool {
    value.to_bits() == 0
}
fn is_white(value: &Color) -> bool {
    value.iter().all(full)
}
fn no_reference() -> u32 {
    u32::MAX
}

#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "Layer", validate_read = "Self::validate_read")]
struct LayerMdl<V: ModelVersion> {
    #[mdl(skip, default)]
    version: PhantomData<V>,
    #[mdl(
        property = "FilterMode",
        default,
        read_with = "read_filter",
        write_with = "write_filter"
    )]
    filter: u32,
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
    flags: u32,
    #[mdl(property = "Shader", delegate)]
    shader: Option<FixedText<80>>,
    #[mdl(flatten)]
    textures: TextureBindings,
    #[mdl(
        property = "TVertexAnimId",
        default = "no_reference",
        skip_if = "is_no_reference"
    )]
    texture_animation: u32,
    #[mdl(property = "CoordId", default, skip_if = "zero_id")]
    coordinate: u32,
    #[mdl(
        animatable = "Alpha",
        track = "LayerTrack::Alpha",
        default = "one",
        skip_if = "full"
    )]
    alpha: f32,
    #[mdl(
        animatable = "EmissiveGain",
        track = "LayerTrack::EmissiveGain",
        default = "one",
        skip_if = "full",
        enabled_if = "Self::has_emissive",
        enable_with = "Self::mark_emissive"
    )]
    emissive: f32,
    #[mdl(
        animatable = "FresnelColor",
        track = "LayerTrack::FresnelColor",
        default = "white",
        skip_if = "is_white",
        enabled_if = "Self::has_fresnel",
        enable_with = "Self::mark_fresnel"
    )]
    color: Color,
    #[mdl(
        animatable = "FresnelOpacity",
        track = "LayerTrack::FresnelOpacity",
        default,
        skip_if = "zero",
        enabled_if = "Self::has_fresnel",
        enable_with = "Self::mark_fresnel"
    )]
    opacity: f32,
    #[mdl(
        animatable = "FresnelTeamColor",
        track = "LayerTrack::FresnelTeamColor",
        default,
        skip_if = "zero",
        enabled_if = "Self::has_fresnel",
        enable_with = "Self::mark_fresnel"
    )]
    team_color: f32,
    #[mdl(tracks)]
    tracks: Vec<LayerTrack>,
    #[mdl(skip, default)]
    emissive_present: bool,
    #[mdl(skip, default)]
    fresnel_present: bool,
}
impl<V: ModelVersion> LayerMdl<V> {
    fn has_emissive(&self) -> bool {
        V::EmissiveGain::default().emissive_gain().is_some()
    }
    fn has_fresnel(&self) -> bool {
        V::Fresnel::default().fresnel().is_some()
    }
    fn mark_emissive(&mut self) {
        self.emissive_present = true;
    }
    fn mark_fresnel(&mut self) {
        self.fresnel_present = true;
    }
    fn validate_read(&self, span: Span) -> Result<(), mdl::ReadError> {
        if (self.emissive_present && !self.has_emissive())
            || (self.fresnel_present && !self.has_fresnel())
            || (self.shader.is_some() && V::ShaderType::default().shader_type().is_none())
        {
            return Err(mdl::ReadError::new(span, ReadErrorKind::UnsupportedField));
        }
        Ok(())
    }
}

#[derive(Default)]
struct TextureBindings {
    slots: Vec<LayerTextureSlot>,
}
impl ReadFields for TextureBindings {
    type State = Self;
    fn visit_mdl_names(visitor: &mut dyn FnMut(&'static str, bool)) {
        visitor("TextureID", true);
    }
    fn accepts_mdl_field(name: &str, _: bool) -> bool {
        name == "TextureID"
    }
    fn begin_mdl_fields(_: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        Ok(Self::default())
    }
    fn read_mdl_field<'a>(
        mut state: Self,
        parser: &mut Parser<'a>,
        field: Field<'a>,
        mut checkpoint: Parser<'a>,
    ) -> Result<Self, mdl::ReadError> {
        let slot = if field.name == "static" {
            parser.expect_ident("TextureID")?;
            let texture_id = parser.read()?;
            let texture_type = if parser
                .peek()?
                .is_some_and(|token| token.kind == TokenKind::Slot)
            {
                parser.next_token()?;
                parser.read::<u32>()?
            } else {
                0
            };
            parser.expect(TokenKind::Comma)?;
            LayerTextureSlot {
                texture_id,
                texture_type,
                track: None,
            }
        } else {
            let track = checkpoint.read::<AnimationTrack<LayerTextureId>>()?;
            *parser = checkpoint;
            LayerTextureSlot {
                texture_id: 0,
                texture_type: 0,
                track: Some(track),
            }
        };
        if slot.texture_type > 5 {
            return Err(mdl::ReadError::new(
                field.span,
                ReadErrorKind::UnsupportedField,
            ));
        }
        if state
            .slots
            .iter()
            .any(|old| old.texture_type == slot.texture_type)
        {
            return Err(mdl::ReadError::new(
                field.span,
                ReadErrorKind::DuplicateField,
            ));
        }
        state.slots.push(slot);
        Ok(state)
    }
    fn finish_mdl_fields(state: Self, _: Span, _: Span) -> Result<Self, mdl::ReadError> {
        Ok(state)
    }
}
impl WriteFields for TextureBindings {
    type State = ();
    fn visit_mdl_names(visitor: &mut dyn FnMut(&'static str, bool)) {
        visitor("TextureID", true);
    }
    fn prepare_mdl_fields(&self) -> Result<(), mdl::WriteError> {
        for (index, slot) in self.slots.iter().enumerate() {
            if slot.texture_type > 5
                || self.slots[..index]
                    .iter()
                    .any(|old| old.texture_type == slot.texture_type)
            {
                return Err(mdl::WriteError::Unsupported("texture slot"));
            }
            if slot.track.is_some() && (slot.texture_type != 0 || slot.texture_id != 0) {
                return Err(mdl::WriteError::Unsupported(
                    "non-diffuse texture animation or hidden texture base",
                ));
            }
        }
        Ok(())
    }
    fn write_mdl_headers<W: IoWrite>(&self, _: &mut MdlWriter<W>) -> Result<(), mdl::WriteError> {
        Ok(())
    }
    fn write_mdl_fields<W: IoWrite>(
        &self,
        _: (),
        writer: &mut MdlWriter<W>,
    ) -> Result<(), mdl::WriteError> {
        self.prepare_mdl_fields()?;
        for slot in &self.slots {
            if let Some(track) = &slot.track {
                writer.write(track)?;
            } else {
                writer.indent()?;
                writer.raw("static TextureID ")?;
                writer.write(&slot.texture_id)?;
                writer.raw(" <= ")?;
                writer.write(&slot.texture_type)?;
                writer.raw(",\n")?;
            }
        }
        Ok(())
    }
}

impl<V: ModelVersion> mdl::Read for Layer<V> {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        let start = parser.peek()?.map_or(0, |token| token.span.start);
        let value = parser.read::<LayerMdl<V>>()?;
        let error =
            || mdl::ReadError::new(Span::new(start, start), ReadErrorKind::UnsupportedField);
        let mut layer = Self::new();
        layer.filter_mode = value.filter;
        layer.shading_flags = value.flags;
        layer.texture_animation_id = value.texture_animation;
        layer.coordinate_id = value.coordinate;
        layer.alpha = value.alpha;
        if let Some(gain) = layer.emissive_gain.emissive_gain_mut() {
            *gain = value.emissive;
        }
        if let Some(fresnel) = layer.fresnel.fresnel_mut() {
            *fresnel = LayerFresnel {
                color: value.color,
                opacity: value.opacity,
                team_color: value.team_color,
            };
        }
        if let Some(shader) = value.shader {
            let shader = ShaderType::from_name(&shader.text()).ok_or_else(error)?;
            *layer.shader_type.shader_type_mut().ok_or_else(error)? = shader;
        }
        layer.tracks = value.tracks;
        let hd = layer
            .shader_type
            .shader_type()
            .is_some_and(|shader| matches!(shader.id(), 1 | 24));
        if hd {
            *layer.texture_slots.texture_slots_mut().ok_or_else(error)? = value.textures.slots;
        } else {
            for slot in value.textures.slots {
                if slot.texture_type != 0 {
                    return Err(error());
                }
                layer.texture_id = slot.texture_id;
                if let Some(track) = slot.track {
                    layer.tracks.insert(0, LayerTrack::TextureId(track));
                }
            }
        }
        Ok(layer)
    }
}
impl<V: ModelVersion> mdl::Write for Layer<V> {
    fn write_mdl<W: IoWrite>(&self, writer: &mut MdlWriter<W>) -> Result<(), mdl::WriteError> {
        if self.filter_mode > 6 {
            return Err(mdl::WriteError::Unsupported("filter mode"));
        }
        let shader = self.shader_type.shader_type();
        let hd = shader.is_some_and(|shader| matches!(shader.id(), 1 | 24));
        let mut textures = TextureBindings::default();
        let mut tracks = Vec::new();
        if hd {
            if self.texture_id != 0 {
                return Err(mdl::WriteError::Unsupported("HD legacy texture ID"));
            }
            textures.slots = self
                .texture_slots
                .texture_slots()
                .unwrap_or_default()
                .to_vec();
        } else {
            if self
                .texture_slots
                .texture_slots()
                .is_some_and(|slots| !slots.is_empty())
            {
                return Err(mdl::WriteError::Unsupported("SD texture slot storage"));
            }
            textures.slots.push(LayerTextureSlot {
                texture_id: self.texture_id,
                texture_type: 0,
                track: None,
            });
        }
        for (index, track) in self.tracks.iter().enumerate() {
            if let LayerTrack::TextureId(track) = track {
                if index != 0 {
                    return Err(mdl::WriteError::Unsupported(
                        "noncanonical texture-track order",
                    ));
                }
                if hd || textures.slots[0].track.is_some() {
                    return Err(mdl::WriteError::Unsupported("texture animation storage"));
                }
                textures.slots[0].track = Some(track.clone());
            } else {
                tracks.push(track.clone());
            }
        }
        let shader = shader
            .map(|shader| {
                let name = shader
                    .name()
                    .ok_or(mdl::WriteError::Unsupported("shader type"))?;
                let mut text = FixedText::default();
                text.set_text(name).expect("shader registry names fit");
                Ok::<_, mdl::WriteError>(text)
            })
            .transpose()?;
        let fresnel = self.fresnel.fresnel().unwrap_or_default();
        writer.write(&LayerMdl::<V> {
            version: PhantomData,
            filter: self.filter_mode,
            flags: self.shading_flags,
            shader,
            textures,
            texture_animation: self.texture_animation_id,
            coordinate: self.coordinate_id,
            alpha: self.alpha,
            emissive: self.emissive_gain.emissive_gain().unwrap_or(1.0),
            color: fresnel.color,
            opacity: fresnel.opacity,
            team_color: fresnel.team_color,
            tracks,
            emissive_present: false,
            fresnel_present: false,
        })
    }
}
