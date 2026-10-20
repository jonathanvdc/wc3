//! Borrowed native Popcorn baked-effect documents.
//!
//! The envelope reader preserves object payloads without guessing unknown field
//! layouts. Parsing a document does not execute its particle programmes.
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

mod bindings;
mod blob;
mod compile;
mod graph;
mod layer;
mod rendering;
mod samplers;
mod script;
pub use bindings::{
    EntryPoint, ExternalDeclaration, FunctionArgument, FunctionDeclaration, NativeType,
};
pub use blob::CompilerBlob;
pub use compile::{CompileError, CompileLimits};
pub use graph::{Effect, EventIndex, Graph, GraphEntry, GraphEvent, GraphLayer, LayerIndex};
pub use layer::{LayerCache, LayerEvent, LayerField, PayloadField};
pub use rendering::{Renderer, RendererInput, RendererProperty};
pub use samplers::{CurveSampler, EventStreamSampler, NamedSampler};
pub use script::{Argument, Instruction, LocatedInstruction, Register, Script};

const MAGIC: [u8; 4] = [0x11, 0x0b, 0x00, 0xca];
const HEADER_SIZE: usize = 28;

/// A checked native envelope whose strings and object bodies borrow the source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document<'a> {
    bytes: &'a [u8],
    header: Header,
    types: Vec<Type>,
    objects: Vec<Object<'a>>,
    strings: Vec<&'a [u8]>,
}

/// Native version and producer information, preserved without normalization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    /// Major, minor, and patch version.
    pub version: [u8; 3],
    /// Native producer identifier; one denotes baked content.
    pub generator: u8,
    /// Native source revision.
    pub revision: u32,
    /// Uninterpreted final header word.
    pub reserved: u32,
}

/// One native type-table entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Type {
    /// Zero-based name in the document's string table.
    pub name: u32,
    /// Uninterpreted second word in the type entry.
    pub reserved: u32,
}

/// An isolated object body with its native identity and payload intact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Object<'a> {
    /// Native object flags.
    pub flags: u8,
    /// Zero-based entry in the document's type table.
    pub type_index: u32,
    /// Declared number of fields; their widths require the object's schema.
    pub field_count: u16,
    /// Exact field bytes following the seven-byte object header.
    pub fields: &'a [u8],
}

/// A malformed or unsupported native document, including its byte location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadError {
    /// Byte offset within the input passed to the failing reader.
    pub offset: usize,
    /// Explanation suitable for a native asset diagnostic.
    pub message: &'static str,
}
impl Display for ReadError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "PKB at byte {}: {}", self.offset, self.message)
    }
}
impl Error for ReadError {}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
    origin: usize,
}
impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8], origin: usize) -> Self {
        Self {
            bytes,
            offset: 0,
            origin,
        }
    }
    fn error(&self, message: &'static str) -> ReadError {
        ReadError {
            offset: self.origin + self.offset,
            message,
        }
    }
    fn take(&mut self, count: usize) -> Result<&'a [u8], ReadError> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or_else(|| self.error("byte range overflows"))?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| self.error("truncated byte range"))?;
        self.offset = end;
        Ok(value)
    }
    fn byte(&mut self) -> Result<u8, ReadError> {
        Ok(self.take(1)?[0])
    }
    fn short(&mut self) -> Result<u16, ReadError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn word(&mut self) -> Result<u32, ReadError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }
    fn count(&mut self, minimum_bytes: usize) -> Result<usize, ReadError> {
        let count = self.word()? as usize;
        if count > self.remaining() / minimum_bytes {
            return Err(self.error("declared count exceeds its enclosing byte range"));
        }
        Ok(count)
    }
}

impl<'a> Document<'a> {
    /// Decode a version 2.5.1 envelope with checked object and table boundaries.
    /// Unknown object types retain their complete field payloads.
    pub fn read(bytes: &'a [u8]) -> Result<Self, ReadError> {
        let mut input = Cursor::new(bytes, 0);
        if input.take(4)? != MAGIC {
            return Err(input.error("invalid signature"));
        }
        let version = input.take(3)?.try_into().unwrap();
        if version != [2, 5, 1] {
            return Err(input.error("unsupported envelope version"));
        }
        let generator = input.byte()?;
        let revision = input.word()?;
        let object_count = input.word()? as usize;
        let type_count = input.word()? as usize;
        let string_offset = input.word()? as usize;
        let reserved = input.word()?;
        let object_bytes = bytes
            .get(HEADER_SIZE..string_offset)
            .ok_or_else(|| input.error("string table overlaps the header or exceeds the source"))?;
        let mut records = Cursor::new(object_bytes, HEADER_SIZE);
        if type_count > records.remaining() / 8 {
            return Err(records.error("type table exceeds the object region"));
        }
        let mut types = Vec::with_capacity(type_count);
        for _ in 0..type_count {
            types.push(Type {
                name: records.word()?,
                reserved: records.word()?,
            });
        }
        if object_count > records.remaining() / 11 {
            return Err(records.error("object count exceeds the object region"));
        }
        let mut objects = Vec::with_capacity(object_count);
        for _ in 0..object_count {
            let length = records.word()? as usize;
            let origin = records.origin + records.offset;
            let mut body = Cursor::new(records.take(length)?, origin);
            let flags = body.byte()?;
            let type_index = body.word()?;
            if type_index as usize >= types.len() {
                return Err(body.error("object type index is outside the type table"));
            }
            let field_count = body.short()?;
            if usize::from(field_count) > body.remaining() / 2 {
                return Err(body.error("field count exceeds the object payload"));
            }
            objects.push(Object {
                flags,
                type_index,
                field_count,
                fields: body.take(body.remaining())?,
            });
        }
        if records.remaining() != 0 {
            return Err(records.error("unaccounted bytes before the string table"));
        }
        let mut table = Cursor::new(&bytes[string_offset..], string_offset);
        let string_count = table.count(1)?;
        let mut strings = Vec::with_capacity(string_count);
        for _ in 0..string_count {
            let first = table.byte()?;
            let length = if first & 0x80 == 0 {
                usize::from(first)
            } else {
                (usize::from(first & 0x7f) << 8) | usize::from(table.byte()?)
            };
            strings.push(table.take(length)?);
        }
        if table.remaining() != 0 {
            return Err(table.error("unaccounted bytes after the string table"));
        }
        if let Some(index) = types
            .iter()
            .position(|entry| entry.name as usize >= strings.len())
        {
            return Err(ReadError {
                offset: HEADER_SIZE + index * 8,
                message: "type name index is outside the string table",
            });
        }
        Ok(Self {
            bytes,
            header: Header {
                version,
                generator,
                revision,
                reserved,
            },
            types,
            objects,
            strings,
        })
    }
    /// Original native bytes, including unrecognized fields and metadata.
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }
    /// Native producer and version information.
    pub fn header(&self) -> Header {
        self.header
    }
    /// Native type entries in their original order.
    pub fn types(&self) -> &[Type] {
        &self.types
    }
    /// Native objects; object links use one-based indices into this array.
    pub fn objects(&self) -> &[Object<'a>] {
        &self.objects
    }
    /// Unmodified string bytes, including empty and non-UTF-8 strings.
    pub fn strings(&self) -> &[&'a [u8]] {
        &self.strings
    }
    /// Follow a nullable, one-based native object link with bounds checking.
    pub fn object(&self, link: u32) -> Option<&Object<'a>> {
        self.objects.get(link.checked_sub(1)? as usize)
    }
}
