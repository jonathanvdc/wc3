//! Fixed-width texture records in `TEXS` chunks.
use crate::mdl::{
    ReadError as MdlError, ReadErrorKind, Fields, MdlRead, MdlWrite, MdlWriter, Parser, TokenKind,
    WriteError,
};
use crate::ModelVersion;
use crate::Tag;
use crate::ValueError;
use bitfield::bitfield;
use std::io::Write;

use crate::TexturesChunk;
use crate::{Readable, Writable};
use std::borrow::Cow;

use crate::FixedText;
use crate::Model;

bitfield! {
    /// Texture wrapping flags; unknown bits remain available through `bits`.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct TextureFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the `WRAP_WIDTH` bit.
    pub wrap_width, set_wrap_width: 0;
    /// Returns or changes the `WRAP_HEIGHT` bit.
    pub wrap_height, set_wrap_height: 1;
}

const PATH_SIZE: usize = 256;

/// A texture reference with its original reserved bytes intact.
#[derive(Clone, Debug, Eq, PartialEq, Readable, Writable)]
pub struct Texture {
    replaceable_id: u32,
    path: FixedText<PATH_SIZE>,
    reserved: Tag,
    flags: u32,
}

impl Texture {
    /// Creates a texture with a path and no flags or replacement ID.
    pub fn new(path: &str) -> Result<Self, ValueError> {
        let mut texture = Self {
            replaceable_id: 0,
            path: FixedText::default(),
            reserved: [0; 4],
            flags: 0,
        };
        texture.set_path(path)?;
        Ok(texture)
    }

    /// Returns the replaceable texture ID.
    pub fn replaceable_id(&self) -> u32 {
        self.replaceable_id
    }

    /// Sets the replaceable texture ID.
    pub fn set_replaceable_id(&mut self, id: u32) {
        self.replaceable_id = id;
    }

    /// Returns the path up to the first NUL, replacing invalid UTF-8.
    pub fn path(&self) -> Cow<'_, str> {
        self.path.text()
    }

    /// Sets the path, clearing the unused part of the fixed-width field.
    pub fn set_path(&mut self, path: &str) -> Result<(), ValueError> {
        self.path.set_text(path)
    }

    /// Returns decoded texture wrapping flags.
    pub fn flags(&self) -> TextureFlags {
        TextureFlags(self.flags)
    }

    /// Sets decoded texture wrapping flags.
    pub fn set_flags(&mut self, flags: TextureFlags) {
        self.flags = flags.bits();
    }
}

impl<V: ModelVersion> Model<V> {
    /// Decodes all `TEXS` chunks in file order.
    pub fn textures(&self) -> Vec<Texture> {
        self.collect_chunk_records::<TexturesChunk>()
    }

    /// Writes the texture list to the first `TEXS` chunk. Additional `TEXS`
    /// chunks are removed after their records are replaced.
    pub fn set_textures(&mut self, textures: &[Texture]) {
        self.replace_chunk(TexturesChunk::new(textures.to_vec()));
    }
}

impl MdlRead for Texture {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, MdlError> {
        parser.expect_ident("Bitmap")?;
        let mut body = parser.begin_block()?;
        let mut texture = Self {
            replaceable_id: 0,
            path: FixedText::default(),
            reserved: [0; 4],
            flags: 0,
        };
        let mut fields = Fields::default();
        while let Some(field) = body.next_field()? {
            match field.name {
                "Image" => {
                    fields.mark(0, field)?;
                    texture.path = body.read_property()?;
                }
                "ReplaceableId" => {
                    fields.mark(1, field)?;
                    texture.replaceable_id = body.read_property()?;
                }
                "WrapWidth" => {
                    fields.mark(2, field)?;
                    body.expect(TokenKind::Comma)?;
                    texture.flags |= 1;
                }
                "WrapHeight" => {
                    fields.mark(3, field)?;
                    body.expect(TokenKind::Comma)?;
                    texture.flags |= 2;
                }
                _ => return Err(MdlError::new(field.span, ReadErrorKind::UnknownField)),
            }
        }
        body.finish()?;
        Ok(texture)
    }
}

impl MdlWrite for Texture {
    fn write_mdl<W: Write>(&self, writer: &mut MdlWriter<W>) -> Result<(), WriteError> {
        if self.reserved != [0; 4] {
            return Err(WriteError::Unsupported("texture reserved bytes"));
        }
        if self.flags & !3 != 0 {
            return Err(WriteError::Unsupported("unknown texture flags"));
        }
        writer.begin_block("Bitmap")?;
        writer.property("Image", &self.path)?;
        if self.replaceable_id != 0 {
            writer.property("ReplaceableId", &self.replaceable_id)?;
        }
        if self.flags & 1 != 0 {
            writer.flag("WrapWidth")?;
        }
        if self.flags & 2 != 0 {
            writer.flag("WrapHeight")?;
        }
        writer.end_block()
    }
}
