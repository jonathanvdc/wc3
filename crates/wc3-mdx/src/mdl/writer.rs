use super::{MdlWrite, WriteError};
use crate::FixedText;
use std::fmt::Arguments;
use std::io::{Cursor as IoCursor, Write};
use std::str::from_utf8;

/// Sequential MDL output to a caller-owned sink, with no intermediate String.
/// Canonical output uses tabs, LF, and round-tripping f32 decimal literals.
/// An error can leave partial output; discard it or roll back your sink.
pub struct MdlWriter<W> {
    output: W,
    depth: usize,
}
impl<W: Write> MdlWriter<W> {
    pub fn new(output: W) -> Self {
        Self { output, depth: 0 }
    }
    /// Access the sink without checking block balance; prefer finish on success.
    pub fn into_inner(self) -> W {
        self.output
    }
    pub fn finish(self) -> Result<W, WriteError> {
        if self.depth != 0 {
            Err(WriteError::UnbalancedBlocks)
        } else {
            Ok(self.output)
        }
    }
    pub fn write<T: MdlWrite + ?Sized>(&mut self, value: &T) -> Result<(), WriteError> {
        value.write_mdl(self)
    }
    /// Writes literal punctuation or formatting; does not change indentation.
    pub fn raw(&mut self, value: &str) -> Result<(), WriteError> {
        self.output.write_all(value.as_bytes())?;
        Ok(())
    }
    pub fn identifier(&mut self, value: &str) -> Result<(), WriteError> {
        let mut bytes = value.bytes();
        if !bytes
            .next()
            .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
            || !bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(WriteError::InvalidIdentifier);
        }
        self.raw(value)
    }
    /// Strings have no escaping. Quotes and NUL cannot be represented; CR/LF
    /// and backslashes are written literally, per the Warcraft III dialect.
    pub fn quoted(&mut self, value: &str) -> Result<(), WriteError> {
        if value.contains(['"', '\0']) {
            return Err(WriteError::InvalidString);
        }
        self.raw("\"")?;
        self.raw(value)?;
        self.raw("\"")
    }
    pub fn indent(&mut self) -> Result<(), WriteError> {
        for _ in 0..self.depth {
            self.raw("\t")?;
        }
        Ok(())
    }
    /// Begins a block without parameters.
    pub fn begin_block(&mut self, name: &str) -> Result<(), WriteError> {
        self.indent()?;
        self.identifier(name)?;
        self.open_body()
    }
    pub fn begin_named_block(&mut self, name: &str, value: &str) -> Result<(), WriteError> {
        self.indent()?;
        self.identifier(name)?;
        self.raw(" ")?;
        self.quoted(value)?;
        self.open_body()
    }
    pub fn begin_counted_block(&mut self, name: &str, count: usize) -> Result<(), WriteError> {
        if count > u32::MAX as usize {
            return Err(WriteError::Unsupported("list count above u32::MAX"));
        }
        self.indent()?;
        self.identifier(name)?;
        write!(self.output, " {count}")?;
        self.open_body()
    }
    /// Opens a body after a header written with the lower-level methods.
    pub fn open_body(&mut self) -> Result<(), WriteError> {
        let depth = self
            .depth
            .checked_add(1)
            .ok_or(WriteError::UnbalancedBlocks)?;
        self.raw(" {\n")?;
        self.depth = depth;
        Ok(())
    }
    pub fn end_block(&mut self) -> Result<(), WriteError> {
        self.depth = self
            .depth
            .checked_sub(1)
            .ok_or(WriteError::UnbalancedBlocks)?;
        self.indent()?;
        self.raw("}\n")
    }
    pub fn property<T: MdlWrite + ?Sized>(
        &mut self,
        name: &str,
        value: &T,
    ) -> Result<(), WriteError> {
        self.indent()?;
        self.identifier(name)?;
        self.raw(" ")?;
        self.write(value)?;
        self.raw(",\n")
    }
    pub fn flag(&mut self, name: &str) -> Result<(), WriteError> {
        self.indent()?;
        self.identifier(name)?;
        self.raw(",\n")
    }
    /// Writes an anonymous value and its entry separator.
    pub fn entry<T: MdlWrite + ?Sized>(&mut self, value: &T) -> Result<(), WriteError> {
        self.indent()?;
        self.write(value)?;
        self.raw(",\n")
    }
    /// Writes a count-prefixed list of records. Each record owns its indentation
    /// and separator, just as it does in the corresponding parser list.
    pub fn counted<'a, T: MdlWrite + 'a>(
        &mut self,
        name: &str,
        items: impl ExactSizeIterator<Item = &'a T>,
    ) -> Result<(), WriteError> {
        self.begin_counted_block(name, items.len())?;
        for item in items {
            self.write(item)?;
        }
        self.end_block()
    }
    pub(crate) fn formatted(&mut self, args: Arguments<'_>) -> Result<(), WriteError> {
        self.output.write_fmt(args)?;
        Ok(())
    }
}

macro_rules! integers {
    ($($ty:ty),*) => { $(impl MdlWrite for $ty {
        fn write_mdl<W: Write>(&self, writer: &mut MdlWriter<W>) -> Result<(), WriteError> {
            writer.formatted(format_args!("{self}"))
        }
    })* };
}
integers!(u8, u16, u32, i32);
impl MdlWrite for f32 {
    fn write_mdl<W: Write>(&self, writer: &mut MdlWriter<W>) -> Result<(), WriteError> {
        if self.is_nan() {
            writer.raw("nan")
        } else if *self == f32::INFINITY {
            writer.raw("inf")
        } else if *self == f32::NEG_INFINITY {
            writer.raw("-inf")
        } else {
            // f32 Debug provides shortest-roundtrip digits. Use bounded stack
            // storage so integral exponent forms can retain a decimal point.
            let mut bytes = [0u8; 48];
            let mut buffer = IoCursor::new(&mut bytes[..]);
            write!(buffer, "{self:?}")?;
            let length = buffer.position() as usize;
            let text = from_utf8(&bytes[..length]).expect("f32 formatting is ASCII");
            if self.fract() == 0.0 && !text.contains('.') {
                let (mantissa, exponent) = text
                    .split_once('e')
                    .expect("integral f32 formatting has a decimal point or exponent");
                writer.raw(mantissa)?;
                writer.raw(".0e")?;
                writer.raw(exponent)
            } else {
                writer.raw(text)
            }
        }
    }
}
impl MdlWrite for str {
    fn write_mdl<W: Write>(&self, writer: &mut MdlWriter<W>) -> Result<(), WriteError> {
        writer.quoted(self)
    }
}
impl<T: MdlWrite, const N: usize> MdlWrite for [T; N] {
    fn write_mdl<W: Write>(&self, writer: &mut MdlWriter<W>) -> Result<(), WriteError> {
        writer.raw("{ ")?;
        for (i, value) in self.iter().enumerate() {
            if i != 0 {
                writer.raw(", ")?;
            }
            writer.write(value)?;
        }
        writer.raw(" }")
    }
}

/// Checks that fixed text has a faithful text representation, including padding.
/// Do not use the existing lossy text() accessor for MDL conversion.
pub(crate) fn fixed_text<const N: usize>(text: &FixedText<N>) -> Result<&str, WriteError> {
    let bytes = text.as_bytes();
    let end = bytes
        .iter()
        .position(|&b| b == 0)
        .ok_or(WriteError::Unsupported("unterminated fixed text"))?;
    if bytes[end..].iter().any(|&b| b != 0) {
        return Err(WriteError::Unsupported("nonzero fixed-text padding"));
    }
    from_utf8(&bytes[..end]).map_err(|_| WriteError::Unsupported("non-UTF-8 fixed text"))
}
impl<const N: usize> MdlWrite for FixedText<N> {
    fn write_mdl<W: Write>(&self, writer: &mut MdlWriter<W>) -> Result<(), WriteError> {
        writer.quoted(fixed_text(self)?)
    }
}
