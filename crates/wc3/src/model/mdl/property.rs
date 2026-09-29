//! Delegated named properties: field types own framing and presence policies.
use super::{Dialect, Field, Parser, Read, Span, Write, Writer};
use crate::model::mdl;
use crate::model::IoError;
use std::io::Write as IoWrite;

/// Reads a whole property's payload after the enclosing record consumes its name.
///
/// Used by `#[mdl(property = "Name", delegate)]`. The record owns name dispatch,
/// duplicate detection, and unknown-field rejection. Implementations own the
/// payload's punctuation (including the comma for scalar properties), and may
/// reject presence using the supplied field's span. No value-level `Read` or
/// `Default` implementation is required.
pub trait ReadProperty: Sized {
    fn read_mdl_property(parser: &mut Parser<'_>, field: Field<'_>)
        -> Result<Self, mdl::ReadError>;

    /// Resolves an omitted property. The default makes the property required;
    /// optional or unavailable field types override this to reconstruct storage.
    /// Called once at the closing brace, only when the property was absent.
    fn missing_mdl_property(name: &'static str, span: Span) -> Result<Self, mdl::ReadError> {
        Err(mdl::ReadError::new(
            span,
            mdl::ReadErrorKind::MissingField(name),
        ))
    }
}

/// Writes a complete named property, or omits it without emitting any framing.
///
/// Implementations own omission and must reject values that omission would lose.
/// `write_mdl_property` includes indentation, the name, value, and punctuation.
/// The record's `write_order` still determines where it is called.
pub trait WriteProperty {
    /// Checks representability before the enclosing record emits any output.
    /// Defaults to no additional checks. Implementations must repeat necessary
    /// checks in their writer if it is also callable directly.
    fn validate_mdl_property(
        &self,
        _name: &'static str,
        _dialect: Dialect,
    ) -> Result<(), mdl::WriteError> {
        Ok(())
    }

    fn write_mdl_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>>;
}

/// An ordinary optional scalar/vector/string property. Missing means None;
/// Some(value) always emits its complete property, even if value is a default.
impl<T: Read> ReadProperty for Option<T> {
    fn read_mdl_property(
        parser: &mut Parser<'_>,
        _field: Field<'_>,
    ) -> Result<Self, mdl::ReadError> {
        parser.read_property().map(Some)
    }

    fn missing_mdl_property(_name: &'static str, _span: Span) -> Result<Self, mdl::ReadError> {
        Ok(None)
    }
}

impl<T: Write> WriteProperty for Option<T> {
    fn write_mdl_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        if let Some(value) = self {
            writer.property(name, value)?;
        }
        Ok(())
    }
}

/// Reads a static or animated property into its field, retaining any base value
/// when the input supplies animation. The record owns dispatch and duplicates.
pub trait ReadAnimationProperty {
    fn read_mdl_animation_property(
        &mut self,
        parser: &mut Parser<'_>,
        static_form: bool,
        bare_static: bool,
    ) -> Result<(), mdl::ReadError>;
}

/// Writes the animation when present, otherwise the static value or nothing.
pub trait WriteAnimationProperty {
    fn has_animation(&self) -> bool;
    fn write_mdl_animation_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>>;
}
