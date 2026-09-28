use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::io;

/// Half-open byte range in the original MDL input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// Allocation-free diagnostic details. Field names borrow the source via spans.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadErrorKind {
    InvalidCharacter,
    UnterminatedString,
    Expected(&'static str),
    InvalidNumber(&'static str),
    InvalidString {
        max_bytes: usize,
    },
    UnknownField,
    /// A recognized property is unavailable for this field type.
    UnsupportedField,
    DuplicateField,
    MissingField(&'static str),
    CountMismatch {
        expected: usize,
        actual: usize,
    },
    NoProgress,
    TrailingInput,
}

/// An MDL syntax or value error. No source text is copied into the error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadError {
    pub span: Span,
    pub kind: ReadErrorKind,
}
impl ReadError {
    pub const fn new(span: Span, kind: ReadErrorKind) -> Self {
        Self { span, kind }
    }

    /// One-based line and character column. Computed only when requested.
    pub fn line_column(&self, source: &str) -> (usize, usize) {
        let mut offset = self.span.start.min(source.len());
        while !source.is_char_boundary(offset) {
            offset -= 1;
        }
        let prefix = &source[..offset];
        let (mut line, mut column) = (1, 1);
        let mut previous_cr = false;
        for character in prefix.chars() {
            match character {
                '\r' => {
                    line += 1;
                    column = 1;
                }
                '\n' => {
                    if !previous_cr {
                        line += 1;
                    }
                    column = 1;
                }
                _ => column += 1,
            }
            previous_cr = character == '\r';
        }
        (line, column)
    }

    /// Formats a diagnostic against the original source without allocating.
    pub fn diagnostic<'a>(&'a self, source: &'a str) -> Diagnostic<'a> {
        Diagnostic {
            error: self,
            source,
        }
    }
}
impl Display for ReadErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCharacter => f.write_str("invalid character"),
            Self::UnterminatedString => f.write_str("unterminated quoted string"),
            Self::Expected(value) => write!(f, "expected {value}"),
            Self::InvalidNumber(ty) => write!(f, "invalid or out-of-range {ty}"),
            Self::InvalidString { max_bytes } => write!(
                f,
                "string must contain no NUL and at most {max_bytes} bytes"
            ),
            Self::UnsupportedField => f.write_str("property is unavailable for this field type"),
            Self::UnknownField => f.write_str("unknown field"),
            Self::DuplicateField => f.write_str("duplicate field"),
            Self::MissingField(field) => write!(f, "missing required field {field}"),
            Self::CountMismatch { expected, actual } => {
                write!(f, "expected {expected} items, found {actual}")
            }
            Self::NoProgress => f.write_str("list item reader consumed no input"),
            Self::TrailingInput => f.write_str("unexpected trailing input"),
        }
    }
}
impl Display for ReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.kind, self.span.start)
    }
}
impl StdError for ReadError {}

pub struct Diagnostic<'a> {
    error: &'a ReadError,
    source: &'a str,
}
impl Display for Diagnostic<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (line, column) = self.error.line_column(self.source);
        write!(f, "{} at {line}:{column}", self.error.kind)?;
        if let Some(text) = self.source.get(self.error.span.start..self.error.span.end) {
            if !text.is_empty() {
                write!(f, " near {text:?}")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum WriteError {
    Io(io::Error),
    InvalidString,
    InvalidIdentifier,
    UnbalancedBlocks,
    Unsupported(&'static str),
}
impl From<io::Error> for WriteError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
impl Display for WriteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => Display::fmt(error, f),
            Self::InvalidString => f.write_str("MDL strings cannot contain NUL or double quotes"),
            Self::InvalidIdentifier => f.write_str("invalid MDL identifier"),
            Self::UnbalancedBlocks => f.write_str("unbalanced MDL writer blocks"),
            Self::Unsupported(field) => write!(f, "MDL cannot represent {field}"),
        }
    }
}
impl StdError for WriteError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}
