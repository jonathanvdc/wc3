//! Schema-specific views of baked compiler-cache objects.
use super::{Cursor, Document, Object, ReadError, Script};

/// Borrowed compiled storage with native linkage to external declarations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerBlob<'a> {
    /// Optional native class-name string reference.
    pub class_name: Option<u32>,
    /// Native stage identifier; absence denotes initialization zero.
    pub identifier: u32,
    /// Compiled-script bytes, excluding the native word-array count.
    pub bytes: &'a [u8],
    /// One-based links to ordered external declarations.
    pub externals: Vec<u32>,
    /// One-based links to ordered function declarations.
    pub functions: Vec<u32>,
    /// One-based entrypoint object link, or zero when absent.
    pub entrypoint: u32,
}

impl<'a> CompilerBlob<'a> {
    /// Decode this blob's declared instruction and constant storage.
    pub fn script(&self) -> Result<Script<'a>, ReadError> {
        Script::read(self.bytes)
    }
}

impl<'a> Document<'a> {
    /// Discover compiled-cache objects in native object order.
    /// Unrecognized fields fail their typed view while the document remains intact.
    pub fn compiler_blobs(&self) -> impl Iterator<Item = Result<CompilerBlob<'a>, ReadError>> + '_ {
        self.objects
            .iter()
            .filter(|object| {
                self.strings[self.types[object.type_index as usize].name as usize]
                    == b"CCompilerBlobCache"
            })
            .map(|object| self.compiler_blob(object))
    }

    /// Read one compiler cache by its checked one-based native object link.
    pub fn compiler_blob_at(&self, link: u32) -> Result<CompilerBlob<'a>, ReadError> {
        self.compiler_blob(self.declaration(link, b"CCompilerBlobCache")?)
    }

    fn compiler_blob(&self, object: &Object<'a>) -> Result<CompilerBlob<'a>, ReadError> {
        let origin = object.fields.as_ptr() as usize - self.bytes.as_ptr() as usize;
        let mut fields = Cursor::new(object.fields, origin);
        let mut result = CompilerBlob {
            class_name: None,
            identifier: 0,
            bytes: &[],
            externals: Vec::new(),
            functions: Vec::new(),
            entrypoint: 0,
        };
        let mut seen = 0u8;
        for _ in 0..object.field_count {
            let field = fields.short()?;
            if field > 5 {
                return Err(fields.error("unsupported compiled-cache field"));
            }
            let bit = 1 << field;
            if seen & bit != 0 {
                return Err(fields.error("duplicate compiled-cache field"));
            }
            seen |= bit;
            match field {
                0 => result.class_name = Some(fields.word()?),
                1 => result.identifier = fields.word()?,
                2 => {
                    let count = fields.count(4)?;
                    result.bytes = fields.take(count * 4)?;
                }
                3 | 4 => {
                    let count = fields.count(4)?;
                    let mut links = Vec::with_capacity(count);
                    for _ in 0..count {
                        links.push(fields.word()?);
                    }
                    if field == 3 {
                        result.externals = links;
                    } else {
                        result.functions = links;
                    }
                }
                5 => result.entrypoint = fields.word()?,
                _ => unreachable!(),
            }
        }
        if fields.remaining() != 0 {
            return Err(fields.error("unaccounted compiled-cache field bytes"));
        }
        if result
            .class_name
            .is_some_and(|index| index != u32::MAX && index as usize >= self.strings.len())
        {
            return Err(fields.error("compiled-cache class name is outside the string table"));
        }
        if result
            .externals
            .iter()
            .chain(&result.functions)
            .chain([&result.entrypoint])
            .any(|&link| link as usize > self.objects.len())
        {
            return Err(fields.error("compiled-cache object link is outside the object table"));
        }
        Ok(result)
    }
}
