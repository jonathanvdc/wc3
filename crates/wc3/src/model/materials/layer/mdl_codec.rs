//! MDL accessors and slot-qualified texture bindings.
use super::{
    EmissiveGainField, FresnelField, Layer, LayerShaderTypeField, LayerShadingFlags,
    LayerTextureSlot, LayerTextureSlotsField, LayerTrack, ShaderType,
};
use crate::model::mdl::{
    dispatch_name, Dialect, Field, Parser, ReadErrorKind, ReadFields, Span, TokenKind, WriteFields,
    Writer,
};
use crate::model::{mdl, FixedText};
use crate::model::{AnimationTrack, Color, LayerTextureId, ModelVersion};
use std::io::Write as IoWrite;

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

const SLOT_NAMES: [&str; 6] = [
    "TextureID",
    "NormalTextureID",
    "ORMTextureID",
    "EmissiveTextureID",
    "TeamColorTextureID",
    "ReflectionsTextureID",
];

#[derive(Default)]
pub(super) struct TextureBindings {
    slots: Vec<LayerTextureSlot>,
}
impl ReadFields for TextureBindings {
    type State = Self;
    fn visit_mdl_names(visitor: &mut dyn FnMut(&'static str, bool)) {
        for name in SLOT_NAMES {
            visitor(name, true);
        }
    }
    fn accepts_mdl_field(name: &str, _: bool) -> bool {
        SLOT_NAMES.contains(&name)
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
        let (name, static_form) = dispatch_name(field, checkpoint)?;
        let named_slot = SLOT_NAMES
            .iter()
            .position(|candidate| *candidate == name)
            .expect("dispatched texture name") as u32;
        let name = SLOT_NAMES[named_slot as usize];
        let slot = if static_form {
            parser.expect_ident(name)?;
            let texture_id = parser.read()?;
            let texture_type = if parser
                .peek()?
                .is_some_and(|token| token.kind == TokenKind::Slot)
            {
                if named_slot != 0 {
                    return Err(mdl::ReadError::new(
                        field.span,
                        ReadErrorKind::UnsupportedField,
                    ));
                }
                parser.next_token()?;
                parser.read::<u32>()?
            } else {
                named_slot
            };
            parser.expect(TokenKind::Comma)?;
            LayerTextureSlot {
                texture_id,
                texture_type,
                track: None,
            }
        } else {
            let track = AnimationTrack::<LayerTextureId>::read_mdl_named(&mut checkpoint, name)?;
            *parser = checkpoint;
            LayerTextureSlot {
                texture_id: 0,
                texture_type: named_slot,
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
        for name in SLOT_NAMES {
            visitor(name, true);
        }
    }
    fn prepare_mdl_fields(&self, dialect: Dialect) -> Result<(), mdl::WriteError> {
        validate_slots(&self.slots, dialect)
    }
    fn write_mdl_headers<W: IoWrite>(&self, _: &mut Writer<W>) -> Result<(), mdl::WriteError> {
        Ok(())
    }
    fn write_mdl_fields<W: IoWrite>(
        &self,
        _: (),
        writer: &mut Writer<W>,
    ) -> Result<(), mdl::WriteError> {
        self.prepare_mdl_fields(writer.dialect())?;
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
    fn prepare_mdl_fields(&self, dialect: Dialect) -> Result<(), mdl::WriteError> {
        if self.0.mdl_hd() {
            validate_slots(
                self.0.texture_slots.texture_slots().unwrap_or_default(),
                dialect,
            )
        } else {
            if self.0.mdl_texture_track().is_some() && self.0.texture_id != 0 {
                return Err(mdl::WriteError::Unsupported("hidden texture base"));
            }
            Ok(())
        }
    }
    fn write_mdl_headers<W: IoWrite>(&self, _: &mut Writer<W>) -> Result<(), mdl::WriteError> {
        Ok(())
    }
    fn write_mdl_fields<W: IoWrite>(
        &self,
        _: (),
        writer: &mut Writer<W>,
    ) -> Result<(), mdl::WriteError> {
        self.prepare_mdl_fields(writer.dialect())?;
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
    pub(super) fn mdl_shader(&self) -> ShaderMarker {
        ShaderMarker(self.shader_type.shader_type())
    }
    pub(super) fn set_mdl_shader(
        &mut self,
        value: ShaderMarker,
        _: bool,
        span: Span,
    ) -> Result<(), mdl::ReadError> {
        if let Some(value) = value.0 {
            let target = self
                .shader_type
                .shader_type_mut()
                .ok_or_else(|| mdl::ReadError::new(span, ReadErrorKind::UnsupportedField))?;
            *target = value;
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
        if self.filter_mode.raw() > 6 {
            return Err(mdl::WriteError::Unsupported("filter mode"));
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
fn validate_slots(slots: &[LayerTextureSlot], dialect: Dialect) -> Result<(), mdl::WriteError> {
    for (index, slot) in slots.iter().enumerate() {
        if slot.texture_type > 5
            || slots[..index]
                .iter()
                .any(|old| old.texture_type == slot.texture_type)
        {
            return Err(mdl::WriteError::Unsupported("texture slot"));
        }
        if slot.track.is_some()
            && (slot.texture_id != 0 || (dialect == Dialect::Warcraft3 && slot.texture_type != 0))
        {
            return Err(mdl::WriteError::Unsupported(
                "non-diffuse texture animation or hidden texture base",
            ));
        }
    }
    Ok(())
}
fn write_slots<W: IoWrite>(
    slots: &[LayerTextureSlot],
    writer: &mut Writer<W>,
) -> Result<(), mdl::WriteError> {
    validate_slots(slots, writer.dialect())?;
    for slot in slots {
        if let Some(track) = &slot.track {
            let name = SLOT_NAMES[slot.texture_type as usize];
            track.write_mdl_named(writer, name)?;
        } else {
            write_texture_id(slot.texture_id, slot.texture_type, writer)?;
        }
    }
    Ok(())
}
fn write_texture_id<W: IoWrite>(
    id: u32,
    slot: u32,
    writer: &mut Writer<W>,
) -> Result<(), mdl::WriteError> {
    if slot > 5 {
        return Err(mdl::WriteError::Unsupported("texture slot"));
    }
    if writer.dialect() == Dialect::HiveWorkshop {
        writer.indent()?;
        writer.raw("static ")?;
        writer.identifier(SLOT_NAMES[slot as usize])?;
        writer.raw(" ")?;
        writer.write(&id)?;
        return writer.raw(",\n");
    }
    writer.indent()?;
    writer.raw("static TextureID ")?;
    writer.write(&id)?;
    writer.raw(" <= ")?;
    writer.write(&slot)?;
    writer.raw(",\n")
}

/// One semantic shader assignment shared by both property spellings.
pub(super) struct ShaderMarker(Option<ShaderType>);
impl mdl::ReadProperty for ShaderMarker {
    fn read_mdl_property(
        parser: &mut Parser<'_>,
        field: Field<'_>,
    ) -> Result<Self, mdl::ReadError> {
        let value = if field.name == "ShaderTypeId" {
            ShaderType::new(parser.read_property()?)
        } else {
            let name = parser.read_property::<FixedText<80>>()?;
            ShaderType::from_name(&name.text())
                .ok_or_else(|| mdl::ReadError::new(field.span, ReadErrorKind::UnsupportedField))?
        };
        Ok(Self(Some(value)))
    }
    fn missing_mdl_property(_: &'static str, _: Span) -> Result<Self, mdl::ReadError> {
        Ok(Self(None))
    }
}
impl mdl::WriteProperty for ShaderMarker {
    fn validate_mdl_property(
        &self,
        _: &'static str,
        dialect: Dialect,
    ) -> Result<(), mdl::WriteError> {
        if dialect == Dialect::Warcraft3 && self.0.is_some_and(|shader| shader.name().is_none()) {
            return Err(mdl::WriteError::Unsupported(
                "unnamed shader in Warcraft III dialect",
            ));
        }
        Ok(())
    }
    fn write_mdl_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut Writer<W>,
    ) -> Result<(), mdl::WriteError> {
        self.validate_mdl_property(name, writer.dialect())?;
        if let Some(shader) = self.0 {
            if writer.dialect() == Dialect::HiveWorkshop {
                writer.property(name, &shader.id())?;
            } else {
                writer.property(name, shader.name().expect("validated shader"))?;
            }
        }
        Ok(())
    }
}

impl<V: ModelVersion> Layer<V> {
    pub(super) fn mdl_shading_flags(&self) -> u32 {
        self.shading_flags.bits()
    }
    pub(super) fn set_mdl_shading_flags(
        &mut self,
        bits: u32,
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.shading_flags = LayerShadingFlags::from_bits_retain(bits);
        Ok(())
    }
}
