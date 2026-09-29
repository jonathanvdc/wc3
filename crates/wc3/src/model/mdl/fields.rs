//! Streaming codecs for field groups embedded in another record's header/body.
use super::{Dialect, Field, Parser, Span, TokenKind, Writer};
use crate::model::mdl;
use crate::model::IoError;
use std::io::Write as IoWrite;

/// A derived `#[mdl(fields)]` group, or the fields of a derived block.
/// State stores presence markers independently of the final record, so required
/// fields and duplicates remain checked when fields from several groups interleave.
pub trait ReadFields: Sized {
    type State;
    /// Visits each body name once; the boolean indicates support for `static`.
    fn visit_mdl_names(visitor: &mut dyn FnMut(&'static str, bool));
    fn accepts_mdl_field(name: &str, static_form: bool) -> bool;
    /// Reads header values and initializes body storage.
    fn begin_mdl_fields(parser: &mut Parser<'_>) -> Result<Self::State, mdl::ReadError>;
    /// The name has been consumed; checkpoint points immediately before it.
    fn read_mdl_field<'a>(
        state: Self::State,
        parser: &mut Parser<'a>,
        field: Field<'a>,
        checkpoint: Parser<'a>,
    ) -> Result<Self::State, mdl::ReadError>;
    /// Resolves omissions at the closing-brace span and validates the record span.
    fn finish_mdl_fields(
        state: Self::State,
        span: Span,
        record_span: Span,
    ) -> Result<Self, mdl::ReadError>;
}

/// Writes a field group's headers and body without a containing block.
pub trait WriteFields {
    type State;
    /// Validates and caches omission decisions before any output. Pass the returned
    /// state to write_mdl_fields for this same unchanged value and dialect.
    fn prepare_mdl_fields(&self, dialect: Dialect) -> Result<Self::State, mdl::WriteError>;
    fn visit_mdl_names(visitor: &mut dyn FnMut(&'static str, bool));
    fn write_mdl_headers<W: IoWrite>(
        &self,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>>;
    fn write_mdl_fields<W: IoWrite>(
        &self,
        state: Self::State,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>>;
}

/// Checks schema composition, including collisions between flattened groups.
#[doc(hidden)]
pub fn field_names_unique(visit: impl Fn(&mut dyn FnMut(&'static str, bool))) -> bool {
    let mut unique = true;
    let mut has_static = false;
    let mut literal_static = false;
    // Composition is small and checked once per record. A second metadata walk
    // avoids allocating a name set on the otherwise allocation-free codec path.
    visit(&mut |name, static_form| {
        has_static |= static_form;
        literal_static |= name == "static";
        let mut occurrences = 0;
        visit(&mut |other, _| {
            occurrences += usize::from(other == name);
        });
        unique &= occurrences == 1;
    });
    unique && !(has_static && literal_static)
}

/// Resolves the dispatch name without changing the caller's parser.
#[doc(hidden)]
pub fn dispatch_name<'a>(
    field: Field<'a>,
    mut checkpoint: Parser<'a>,
) -> Result<(&'a str, bool), mdl::ReadError> {
    if field.name != "static" {
        return Ok((field.name, false));
    }
    checkpoint.next_token()?;
    let token = checkpoint.next_token()?;
    match token.kind {
        TokenKind::Ident(name) => Ok((name, true)),
        _ => Err(mdl::ReadError::new(
            token.span,
            mdl::ReadErrorKind::Expected("a static property name"),
        )),
    }
}

/// Reads headers and a body after its block name has been consumed.
#[doc(hidden)]
pub fn read_mdl_body<T: ReadFields>(
    parser: &mut Parser<'_>,
    start: usize,
) -> Result<T, mdl::ReadError> {
    let mut state = T::begin_mdl_fields(parser)?;
    let mut body = parser.begin_block()?;
    loop {
        let checkpoint = *body;
        let Some(field) = body.next_field()? else {
            break;
        };
        state = T::read_mdl_field(state, &mut body, field, checkpoint)?;
    }
    let span = body.error(mdl::ReadErrorKind::Expected("'}'")).span;
    let value = T::finish_mdl_fields(state, span, Span::new(start, span.end))?;
    body.finish()?;
    Ok(value)
}
