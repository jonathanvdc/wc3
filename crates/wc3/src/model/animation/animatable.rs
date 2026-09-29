//! A property's stored value and optional animation.
use super::{Track, TrackValue};
use crate::model::mdl::{Parser, TokenKind, Writer};
use crate::model::{mdl, mdx};
use std::io::Write as IoWrite;
use std::mem::replace;

/// A static value, animation, or animation with a stored base value.
///
/// Animation takes precedence over the base value. MDX retains both when the
/// layout stores a base; MDL writes the animation in place of that base.
/// When `Animated` has no base, MDX supplies `T::default()` for its fixed field.
/// MDL decoding retains the enclosing field's declared default as a base.
#[derive(Clone, Debug, PartialEq)]
pub enum Animatable<T: TrackValue> {
    Static(T),
    Animated(Track<T>),
    Both { value: T, track: Track<T> },
}
impl<T: TrackValue> Default for Animatable<T> {
    fn default() -> Self {
        Self::Static(T::default())
    }
}
impl<T: TrackValue> From<T> for Animatable<T> {
    fn from(value: T) -> Self {
        Self::Static(value)
    }
}
impl<T: TrackValue> Animatable<T> {
    /// The stored base value, if any.
    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Static(value) | Self::Both { value, .. } => Some(value),
            Self::Animated(_) => None,
        }
    }
    /// The animation, if any.
    pub fn track(&self) -> Option<&Track<T>> {
        match self {
            Self::Animated(track) | Self::Both { track, .. } => Some(track),
            Self::Static(_) => None,
        }
    }
    /// Replaces the animation while retaining the stored base value.
    pub fn set_track(&mut self, track: Track<T>) {
        *self = match self.value().copied() {
            Some(value) => Self::Both { value, track },
            None => Self::Animated(track),
        };
    }
    /// Replaces the base value while retaining any animation.
    pub fn set_value(&mut self, value: T) {
        if let Self::Animated(track) | Self::Both { track, .. } = replace(self, Self::Static(value))
        {
            *self = Self::Both { value, track };
        }
    }
}
impl<T: TrackValue> mdx::Read for Animatable<T> {
    fn read_mdx(cursor: &mut mdx::Cursor<'_>) -> Result<Self, mdx::ReadError> {
        cursor.read().map(Self::Static)
    }
}
impl<T: TrackValue> mdx::Write for Animatable<T> {
    fn write_mdx(&self, encoder: &mut mdx::Encoder<'_>) -> Result<(), mdx::WriteError> {
        encoder.write(&self.value().copied().unwrap_or_default())
    }
}
impl<T: TrackValue + mdl::Write> Animatable<T> {
    pub fn write_mdl_property<W: IoWrite>(
        &self,
        name: &str,
        writer: &mut Writer<W>,
    ) -> Result<(), mdl::WriteError> {
        if let Some(track) = self.track() {
            track.write_mdl_named(writer, name)
        } else {
            writer.indent()?;
            writer.raw("static ")?;
            writer.identifier(name)?;
            writer.raw(" ")?;
            writer.write(self.value().expect("static value"))?;
            writer.raw(",\n")
        }
    }
}
impl<T: TrackValue + mdl::ValueEq> mdl::ValueEq for Animatable<T> {
    fn eq_mdl(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Static(a), Self::Static(b)) => a.eq_mdl(b),
            _ => self == other,
        }
    }
}

impl<T: TrackValue + mdl::Read> mdl::ReadAnimationProperty for Animatable<T> {
    fn read_mdl_animation_property(
        &mut self,
        parser: &mut Parser<'_>,
        static_form: bool,
        bare_static: bool,
    ) -> Result<(), mdl::ReadError> {
        if static_form {
            *self = Self::Static(parser.read_property()?);
            return Ok(());
        }
        if bare_static {
            let mut probe = *parser;
            if let Ok(value) = probe.read::<T>() {
                if probe
                    .peek()?
                    .is_some_and(|token| token.kind == TokenKind::Comma)
                {
                    probe.next_token()?;
                    *parser = probe;
                    *self = Self::Static(value);
                    return Ok(());
                }
            }
        }
        self.set_track(Track::read_mdl_payload(parser)?);
        Ok(())
    }
}
impl<T: TrackValue + mdl::Write> mdl::WriteAnimationProperty for Animatable<T> {
    fn has_animation(&self) -> bool {
        self.track().is_some()
    }
    fn write_mdl_animation_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut Writer<W>,
    ) -> Result<(), mdl::WriteError> {
        self.write_mdl_property(name, writer)
    }
}
impl<T: TrackValue + mdl::Read> mdl::ReadAnimationProperty for Option<Track<T>> {
    fn read_mdl_animation_property(
        &mut self,
        parser: &mut Parser<'_>,
        static_form: bool,
        _: bool,
    ) -> Result<(), mdl::ReadError> {
        if static_form {
            return Err(parser.error(mdl::ReadErrorKind::UnsupportedField));
        }
        *self = Some(Track::read_mdl_payload(parser)?);
        Ok(())
    }
}
impl<T: TrackValue + mdl::Write> mdl::WriteAnimationProperty for Option<Track<T>> {
    fn has_animation(&self) -> bool {
        self.is_some()
    }
    fn write_mdl_animation_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut Writer<W>,
    ) -> Result<(), mdl::WriteError> {
        if let Some(track) = self {
            track.write_mdl_named(writer, name)?;
        }
        Ok(())
    }
}
impl<T: TrackValue> mdl::ValueEq for Option<Track<T>> {
    fn eq_mdl(&self, other: &Self) -> bool {
        self == other
    }
}
