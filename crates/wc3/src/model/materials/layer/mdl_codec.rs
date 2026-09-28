//! MDL accessors and slot-qualified texture bindings.
use super::{
    EmissiveGainField, FresnelField, Layer, LayerShaderTypeField, LayerTextureSlot,
    LayerTextureSlotsField, LayerTrack, ShaderType,
};
use crate::model::mdl::{
    Field, MdlWriter, Parser, ReadErrorKind, ReadFields, Span, TokenKind, WriteFields,
};
use crate::model::{mdl, FixedText};
use crate::model::{AnimationTrack, Color, LayerTextureId, ModelVersion};
use std::io::Write as IoWrite;

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
pub(super) fn read_filter(parser: &mut Parser<'_>) -> Result<u32, mdl::ReadError> {
    Ok(parser.read::<FilterMode>()? as u32)
}
pub(super) fn write_filter<W: IoWrite>(
    value: &u32,
    writer: &mut MdlWriter<W>,
) -> Result<(), mdl::WriteError> {
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
pub(super) fn one() -> f32 {
    1.0
}
pub(super) fn white() -> Color {
    [1.0; 3]
}
pub(super) fn is_no_reference(value: &u32) -> bool {
    *value == u32::MAX
}
pub(super) fn zero_id(value: &u32) -> bool {
    *value == 0
}
pub(super) fn full(value: &f32) -> bool {
    value.to_bits() == 1.0f32.to_bits()
}
pub(super) fn zero(value: &f32) -> bool {
    value.to_bits() == 0
}
pub(super) fn is_white(value: &Color) -> bool {
    value.iter().all(full)
}
pub(super) fn no_reference() -> u32 {
    u32::MAX
}

#[derive(Default)]
pub(super) struct TextureBindings {
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
        validate_slots(&self.slots)
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
        write_slots(&self.slots, writer)
    }
}

// The MDL texture grammar selects storage using the shader, so its writer borrows
// a view of the whole layer instead of constructing an owned texture-slot list.
pub(super) struct TextureBindingsView<'a, V: ModelVersion>(&'a Layer<V>);
impl<V: ModelVersion> WriteFields for TextureBindingsView<'_, V> {
    type State = ();
    fn visit_mdl_names(visitor: &mut dyn FnMut(&'static str, bool)) {
        <TextureBindings as WriteFields>::visit_mdl_names(visitor);
    }
    fn prepare_mdl_fields(&self) -> Result<(), mdl::WriteError> {
        if self.0.mdl_hd() {
            validate_slots(self.0.texture_slots.texture_slots().unwrap_or_default())
        } else {
            if self.0.mdl_texture_track().is_some() && self.0.texture_id != 0 {
                return Err(mdl::WriteError::Unsupported("hidden texture base"));
            }
            Ok(())
        }
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
        if self.0.mdl_hd() {
            write_slots(
                self.0.texture_slots.texture_slots().unwrap_or_default(),
                writer,
            )
        } else if let Some(track) = self.0.mdl_texture_track() {
            writer.write(track)
        } else {
            write_texture_id(self.0.texture_id, 0, writer)
        }
    }
}
impl<V: ModelVersion> Layer<V> {
    fn mdl_hd(&self) -> bool {
        self.shader_type
            .shader_type()
            .is_some_and(|shader| matches!(shader.id(), 1 | 24))
    }
    fn mdl_texture_track(&self) -> Option<&AnimationTrack<LayerTextureId>> {
        match self.tracks.first() {
            Some(LayerTrack::TextureId(track)) => Some(track),
            _ => None,
        }
    }
    pub(super) fn mdl_shader(&self) -> Option<FixedText<80>> {
        self.shader_type.shader_type().map(|shader| {
            let mut text = FixedText::default();
            text.set_text(shader.name().expect("validated shader type"))
                .expect("shader names fit");
            text
        })
    }
    pub(super) fn set_mdl_shader(
        &mut self,
        value: Option<FixedText<80>>,
        _: bool,
        span: Span,
    ) -> Result<(), mdl::ReadError> {
        if let Some(value) = value {
            let shader = ShaderType::from_name(&value.text())
                .ok_or_else(|| mdl::ReadError::new(span, ReadErrorKind::UnsupportedField))?;
            let target = self
                .shader_type
                .shader_type_mut()
                .ok_or_else(|| mdl::ReadError::new(span, ReadErrorKind::UnsupportedField))?;
            *target = shader;
        }
        Ok(())
    }
    pub(super) fn mdl_textures(&self) -> TextureBindingsView<'_, V> {
        TextureBindingsView(self)
    }
    pub(super) fn set_mdl_textures(
        &mut self,
        value: TextureBindings,
        _: bool,
        span: Span,
    ) -> Result<(), mdl::ReadError> {
        if self.mdl_hd() {
            *self
                .texture_slots
                .texture_slots_mut()
                .ok_or_else(|| mdl::ReadError::new(span, ReadErrorKind::UnsupportedField))? =
                value.slots;
        } else {
            for slot in value.slots {
                if slot.texture_type != 0 {
                    return Err(mdl::ReadError::new(span, ReadErrorKind::UnsupportedField));
                }
                self.texture_id = slot.texture_id;
                if let Some(track) = slot.track {
                    self.tracks.push(LayerTrack::TextureId(track));
                }
            }
        }
        Ok(())
    }
    pub(super) fn mdl_tracks(&self) -> &[LayerTrack] {
        if self.mdl_texture_track().is_some() {
            &self.tracks[1..]
        } else {
            &self.tracks
        }
    }
    pub(super) fn set_mdl_tracks(
        &mut self,
        tracks: Vec<LayerTrack>,
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.tracks.extend(tracks);
        Ok(())
    }
    pub(super) fn validate_mdl(&self) -> Result<(), mdl::WriteError> {
        if self.filter_mode > 6 {
            return Err(mdl::WriteError::Unsupported("filter mode"));
        }
        if self
            .shader_type
            .shader_type()
            .is_some_and(|shader| shader.name().is_none())
        {
            return Err(mdl::WriteError::Unsupported("shader type"));
        }
        if self.mdl_hd() {
            if self.texture_id != 0 {
                return Err(mdl::WriteError::Unsupported("HD legacy texture ID"));
            }
            if self
                .tracks
                .iter()
                .any(|track| matches!(track, LayerTrack::TextureId(_)))
            {
                return Err(mdl::WriteError::Unsupported("texture animation storage"));
            }
        } else {
            if self
                .texture_slots
                .texture_slots()
                .is_some_and(|slots| !slots.is_empty())
            {
                return Err(mdl::WriteError::Unsupported("SD texture slot storage"));
            }
            if self
                .tracks
                .iter()
                .enumerate()
                .any(|(index, track)| index != 0 && matches!(track, LayerTrack::TextureId(_)))
            {
                return Err(mdl::WriteError::Unsupported(
                    "noncanonical texture-track order",
                ));
            }
        }
        Ok(())
    }
    pub(super) fn mdl_emissive(&self) -> Option<f32> {
        self.emissive_gain.emissive_gain()
    }
    pub(super) fn mdl_emissive_mut(&mut self) -> Option<&mut f32> {
        self.emissive_gain.emissive_gain_mut()
    }
    pub(super) fn mdl_color(&self) -> Option<Color> {
        self.fresnel.fresnel().map(|value| value.color)
    }
    pub(super) fn mdl_color_mut(&mut self) -> Option<&mut Color> {
        self.fresnel.fresnel_mut().map(|value| &mut value.color)
    }
    pub(super) fn mdl_opacity(&self) -> Option<f32> {
        self.fresnel.fresnel().map(|value| value.opacity)
    }
    pub(super) fn mdl_opacity_mut(&mut self) -> Option<&mut f32> {
        self.fresnel.fresnel_mut().map(|value| &mut value.opacity)
    }
    pub(super) fn mdl_team_color(&self) -> Option<f32> {
        self.fresnel.fresnel().map(|value| value.team_color)
    }
    pub(super) fn mdl_team_color_mut(&mut self) -> Option<&mut f32> {
        self.fresnel
            .fresnel_mut()
            .map(|value| &mut value.team_color)
    }
}
fn validate_slots(slots: &[LayerTextureSlot]) -> Result<(), mdl::WriteError> {
    for (index, slot) in slots.iter().enumerate() {
        if slot.texture_type > 5
            || slots[..index]
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
fn write_slots<W: IoWrite>(
    slots: &[LayerTextureSlot],
    writer: &mut MdlWriter<W>,
) -> Result<(), mdl::WriteError> {
    for slot in slots {
        if let Some(track) = &slot.track {
            writer.write(track)?;
        } else {
            write_texture_id(slot.texture_id, slot.texture_type, writer)?;
        }
    }
    Ok(())
}
fn write_texture_id<W: IoWrite>(
    id: u32,
    slot: u32,
    writer: &mut MdlWriter<W>,
) -> Result<(), mdl::WriteError> {
    writer.indent()?;
    writer.raw("static TextureID ")?;
    writer.write(&id)?;
    writer.raw(" <= ")?;
    writer.write(&slot)?;
    writer.raw(",\n")
}
