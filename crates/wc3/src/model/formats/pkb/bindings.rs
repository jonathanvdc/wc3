//! Native declaration views used to resolve compiled programme bindings.
use super::{Cursor, Document, Object, ReadError};
use crate::model::scene::popcorn::ValueType;

/// Lossless native declaration type, independently of packed register-bank bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeType(#[doc = "Unmodified declaration type identifier."] pub u32);
impl NativeType {
    /// Known scalar evaluator layout; unrecognized identifiers remain native data.
    pub fn value_type(self) -> Option<ValueType> {
        Some(match self.0 {
            1 => ValueType::Bool,
            25 => ValueType::Int,
            26 => ValueType::Int2,
            27 => ValueType::Int3,
            31 => ValueType::Float,
            32 => ValueType::Float2,
            33 => ValueType::Float3,
            34 => ValueType::Float4,
            36 => ValueType::Quaternion,
            u32::MAX => ValueType::Context,
            _ => return None,
        })
    }
}
/// A checked external declaration whose names borrow the native string table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalDeclaration<'a> {
    /// Field or capability name within the owning script.
    pub name: &'a [u8],
    /// Native type name retained independently of its numeric identifier.
    pub type_name: &'a [u8],
    /// Native declaration layout identifier.
    pub native_type: NativeType,
    /// Authored native storage size, distinct from evaluator storage.
    pub storage_bytes: u32,
    /// Native ownership/category selector.
    pub meta_type: u32,
    /// Native attribute controls, with sparse zero default.
    pub attributes: u32,
    /// Native read/write controls.
    pub access: u32,
}
/// Named function binding and its native signature links.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionDeclaration<'a> {
    /// Function symbol bytes.
    pub name: &'a [u8],
    /// Bound external slot; native sparse default zero, MAX means unbound.
    pub symbol_slot: u32,
    /// Uninterpreted function traits.
    pub traits: u32,
    /// One-based links to ordered argument declarations.
    pub arguments: Vec<u32>,
}
/// One native function argument; controls do not imply generic writeback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionArgument<'a> {
    /// Native declaration layout identifier.
    pub native_type: NativeType,
    /// Native type name bytes.
    pub type_name: &'a [u8],
    /// Uninterpreted argument attributes, with sparse zero default.
    pub attributes: u32,
}
/// Named range within one script's instruction stream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryPoint<'a> {
    /// Native symbol bytes.
    pub name: &'a [u8],
    /// First instruction byte; omitted fields have native default zero.
    pub begin: u32,
    /// Exclusive instruction-stream end byte.
    pub end: u32,
}

impl<'a> Document<'a> {
    pub(super) fn declaration(&self, link: u32, name: &[u8]) -> Result<&Object<'a>, ReadError> {
        let object = self.object(link).ok_or(ReadError {
            offset: 0,
            message: "declaration link is null or outside the object table",
        })?;
        if self.strings[self.types[object.type_index as usize].name as usize] != name {
            return Err(ReadError {
                offset: self.object_origin(object),
                message: "object type does not match the requested declaration",
            });
        }
        Ok(object)
    }
    pub(super) fn object_origin(&self, object: &Object<'_>) -> usize {
        object.fields.as_ptr() as usize - self.bytes.as_ptr() as usize
    }
    pub(super) fn string(&self, input: &mut Cursor<'_>) -> Result<&'a [u8], ReadError> {
        let index = input.word()?;
        self.strings
            .get(index as usize)
            .copied()
            .ok_or_else(|| input.error("declaration string is null or outside the string table"))
    }
    pub(super) fn links(&self, input: &mut Cursor<'_>) -> Result<Vec<u32>, ReadError> {
        let count = input.count(4)?;
        let mut links = Vec::with_capacity(count);
        for _ in 0..count {
            let link = input.word()?;
            if link == 0 || link as usize > self.objects.len() {
                return Err(
                    input.error("required declaration link is null or outside the object table")
                );
            }
            links.push(link);
        }
        Ok(links)
    }
    pub(super) fn fields(
        &self,
        object: &Object<'a>,
        mut decode: impl FnMut(u16, &mut Cursor<'a>) -> Result<(), ReadError>,
    ) -> Result<(), ReadError> {
        let mut input = Cursor::new(object.fields, self.object_origin(object));
        let mut seen = 0u64;
        for _ in 0..object.field_count {
            let field = input.short()?;
            if field >= 64 {
                return Err(input.error("unsupported declaration field"));
            }
            let bit = 1u64 << field;
            if seen & bit != 0 {
                return Err(input.error("duplicate declaration field"));
            }
            seen |= bit;
            decode(field, &mut input)?;
        }
        if input.remaining() != 0 {
            return Err(input.error("unaccounted declaration field bytes"));
        }
        Ok(())
    }
    pub(super) fn required<T>(
        &self,
        object: &Object<'_>,
        value: Option<T>,
    ) -> Result<T, ReadError> {
        value.ok_or(ReadError {
            offset: self.object_origin(object),
            message: "required declaration field is absent",
        })
    }
    /// Read an external declaration, requiring the fields needed for checked binding.
    pub fn external_declaration(&self, link: u32) -> Result<ExternalDeclaration<'a>, ReadError> {
        let object = self.declaration(link, b"CCompilerBlobCacheExternal")?;
        let (
            mut name,
            mut type_name,
            mut native_type,
            mut storage_bytes,
            mut meta_type,
            mut access,
        ) = (None, None, None, None, None, None);
        let mut attributes = 0;
        self.fields(object, |field, input| {
            match field {
                0 => name = Some(self.string(input)?),
                1 => type_name = Some(self.string(input)?),
                2 => native_type = Some(NativeType(input.word()?)),
                3 => storage_bytes = Some(input.word()?),
                4 => meta_type = Some(input.word()?),
                5 => attributes = input.word()?,
                6 => access = Some(input.word()?),
                _ => return Err(input.error("unsupported external declaration field")),
            }
            Ok(())
        })?;
        Ok(ExternalDeclaration {
            name: self.required(object, name)?,
            type_name: self.required(object, type_name)?,
            native_type: self.required(object, native_type)?,
            storage_bytes: self.required(object, storage_bytes)?,
            meta_type: self.required(object, meta_type)?,
            attributes,
            access: self.required(object, access)?,
        })
    }
    /// Read a function binding, retaining native traits and argument links.
    pub fn function_declaration(&self, link: u32) -> Result<FunctionDeclaration<'a>, ReadError> {
        let object = self.declaration(link, b"CCompilerBlobCacheFunctionDef")?;
        let (mut name, mut traits, mut arguments) = (None, None, None);
        let mut symbol_slot = 0;
        self.fields(object, |field, input| {
            match field {
                0 => name = Some(self.string(input)?),
                1 => symbol_slot = input.word()?,
                2 => traits = Some(input.word()?),
                3 => arguments = Some(self.links(input)?),
                _ => return Err(input.error("unsupported function declaration field")),
            }
            Ok(())
        })?;
        Ok(FunctionDeclaration {
            name: self.required(object, name)?,
            symbol_slot,
            traits: self.required(object, traits)?,
            arguments: self.required(object, arguments)?,
        })
    }
    /// Read an argument's declared type without guessing argument flag semantics.
    pub fn function_argument(&self, link: u32) -> Result<FunctionArgument<'a>, ReadError> {
        let object = self.declaration(link, b"CCompilerBlobCacheFunctionArg")?;
        let (mut native_type, mut type_name) = (None, None);
        let mut attributes = 0;
        self.fields(object, |field, input| {
            match field {
                0 => native_type = Some(NativeType(input.word()?)),
                1 => type_name = Some(self.string(input)?),
                2 => attributes = input.word()?,
                _ => return Err(input.error("unsupported function argument field")),
            }
            Ok(())
        })?;
        Ok(FunctionArgument {
            native_type: self.required(object, native_type)?,
            type_name: self.required(object, type_name)?,
            attributes,
        })
    }
    /// Read a named entrypoint; callers must check its range against its script.
    pub fn entrypoint(&self, link: u32) -> Result<EntryPoint<'a>, ReadError> {
        let object = self.declaration(link, b"CCompilerBlobCacheEntryPoint")?;
        let (mut name, mut end) = (None, None);
        let mut begin = 0;
        self.fields(object, |field, input| {
            match field {
                0 => name = Some(self.string(input)?),
                1 => begin = input.word()?,
                2 => end = Some(input.word()?),
                _ => return Err(input.error("unsupported entrypoint field")),
            }
            Ok(())
        })?;
        let entry = EntryPoint {
            name: self.required(object, name)?,
            begin,
            end: self.required(object, end)?,
        };
        if entry.end < entry.begin {
            return Err(ReadError {
                offset: self.object_origin(object),
                message: "entrypoint ends before its first instruction",
            });
        }
        Ok(entry)
    }
}
