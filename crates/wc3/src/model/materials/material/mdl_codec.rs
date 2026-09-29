//! Material directives, including historical flags and version-selected shaders.
use super::{Material, NoShader, ShaderText};
use crate::model::mdl;
use crate::model::mdl::{Dialect, Field, Parser, ReadProperty, Span, WriteProperty, Writer};
use crate::model::IoError;
use crate::model::{FixedText, ModelVersion};
use std::io::Write as IoWrite;

impl ReadProperty for NoShader {
    fn read_mdl_property(_: &mut Parser<'_>, field: Field<'_>) -> Result<Self, mdl::ReadError> {
        Err(mdl::ReadError::new(
            field.span,
            mdl::ReadErrorKind::UnsupportedField,
        ))
    }
    fn missing_mdl_property(_: &'static str, _: Span) -> Result<Self, mdl::ReadError> {
        Ok(Self)
    }
}
impl WriteProperty for NoShader {
    fn write_mdl_property<W: IoWrite>(
        &self,
        _: &'static str,
        _: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        Ok(())
    }
}
impl ReadProperty for ShaderText {
    fn read_mdl_property(parser: &mut Parser<'_>, _: Field<'_>) -> Result<Self, mdl::ReadError> {
        parser.read_property().map(Self)
    }
    fn missing_mdl_property(_: &'static str, _: Span) -> Result<Self, mdl::ReadError> {
        Ok(Self::default())
    }
}
impl WriteProperty for ShaderText {
    fn validate_mdl_property(&self, _: &'static str, _: Dialect) -> Result<(), mdl::WriteError> {
        mdl::validate_fixed_text(&self.0)
    }
    fn write_mdl_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        self.validate_mdl_property(name, writer.dialect())?;
        if self.0 != FixedText::default() {
            writer.property(name, &self.0)?;
        }
        Ok(())
    }
}

pub(super) fn zero_priority(value: &i32) -> bool {
    *value == 0
}

impl<V: ModelVersion> Material<V> {
    // Historical material-level Unfogged is accepted but has no stored effect.
    pub(super) fn mdl_unfogged(&self) -> bool {
        false
    }

    pub(super) fn set_mdl_unfogged(
        &mut self,
        _: bool,
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        Ok(())
    }
}
