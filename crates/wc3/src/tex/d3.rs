//! Diablo III TEX revision 47. Unknown header fields and atlas names are retained.
use super::{narrow, put_u32, range, u32_at, Error, MAGIC};
use std::{borrow::Cow, marker::PhantomData};

mod sealed {
    pub trait Sealed {}
}
/// Supported native TEX layout.
pub trait Revision: sealed::Sealed + Clone + std::fmt::Debug + PartialEq {
    /// Native revision discriminator required by this layout.
    const VERSION: u32;
}
/// The revision 47 layout with 60 embedded mip slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct V47;
impl sealed::Sealed for V47 {}
impl Revision for V47 {
    /// Native revision discriminator required by this layout.
    const VERSION: u32 = 47;
}
/// A mip's faces in native disk order, including BC prefixes and shuffled blocks.
#[derive(Clone, Debug, PartialEq)]
pub struct Mip<'a> {
    /// Byte size per face, excluding its 16-byte BC prefix.
    pub face_size: u32,
    /// Contiguous native face payloads.
    pub data: Cow<'a, [u8]>,
}
/// Native atlas frame. Names are fixed bytes, without a UTF-8 requirement.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    /// Atlas bounds in u-min, v-min, u-max, v-max order.
    pub uv: [f32; 4],
    /// Fixed native frame name bytes, including terminator or padding.
    pub name: [u8; 64],
}
/// Revision 47 metadata, with structural magic, revision and pointers managed by the codec.
#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    /// Native SNO asset identifier.
    pub sno_id: u32,
    /// Native pixel format identifier; unknown values are retained.
    pub format: u32,
    /// Base image width in pixels.
    pub width: u32,
    /// Base image height in pixels.
    pub height: u32,
    /// Native face count (one for 2D, six for cubemaps).
    pub faces: u32,
    /// Native flag bits, including uninterpreted bits.
    pub flags: u32,
    /// Native filtering and sampler hints.
    pub sampler_hints: [u32; 2],
    /// Uninterpreted bytes retained from the original header.
    reserved: [u8; 624],
}
impl Header {
    fn read(raw: [u8; 624]) -> Result<Self, Error> {
        Ok(Self {
            sno_id: u32_at(&raw, 16)?,
            format: u32_at(&raw, 32)?,
            width: u32_at(&raw, 36)?,
            height: u32_at(&raw, 40)?,
            faces: u32_at(&raw, 44)?,
            flags: u32_at(&raw, 48)?,
            sampler_hints: [u32_at(&raw, 572)?, u32_at(&raw, 576)?],
            reserved: raw,
        })
    }
    fn bytes<V: Revision>(&self) -> [u8; 624] {
        let mut raw = self.reserved;
        for (at, value) in [
            (0, MAGIC),
            (4, V::VERSION),
            (16, self.sno_id),
            (32, self.format),
            (36, self.width),
            (40, self.height),
            (44, self.faces),
            (48, self.flags),
            (572, self.sampler_hints[0]),
            (576, self.sampler_hints[1]),
        ] {
            put_u32(&mut raw, at, value);
        }
        raw
    }
}
/// Editable native texture; revision is selected by the type parameter.
#[derive(Clone, Debug, PartialEq)]
pub struct Texture<'a, V: Revision = V47> {
    /// Native header, retaining reserved words; structural offsets are rebuilt.
    pub header: Header,
    /// Mip levels in native table order.
    pub mipmaps: Vec<Mip<'a>>,
    /// Native atlas frame records.
    pub frames: Vec<Frame>,
    /// Native frame hash array bytes.
    pub frame_hashes: Cow<'a, [u8]>,
    revision: PhantomData<V>,
}
impl<'a, V: Revision> Texture<'a, V> {
    /// Parse the selected native revision, borrowing leaf payload bytes.
    pub fn read(bytes: &'a [u8]) -> Result<Self, Error> {
        let header: [u8; 624] = range(bytes, 0, 624)?.try_into().unwrap();
        if u32_at(bytes, 0)? != MAGIC {
            return Err(Error::InvalidMagic);
        }
        let version = u32_at(bytes, 4)?;
        if version != V::VERSION {
            return Err(Error::UnsupportedRevision(version));
        }
        let mut result = Self {
            header: Header::read(header)?,
            mipmaps: Vec::new(),
            frames: Vec::new(),
            frame_hashes: Cow::Borrowed(&[]),
            revision: PhantomData,
        };
        result.validate_dimensions()?;
        let mut ended = false;
        for i in 0..60 {
            let at = 56 + i * 8;
            let offset = u32_at(bytes, at)? as usize;
            let size = u32_at(bytes, at + 4)?;
            if size == 0 {
                ended = true;
                continue;
            }
            if ended {
                return Err(Error::Invalid("noncontiguous mip table"));
            }
            let len = (size as usize)
                .checked_add(result.prefix_size())
                .and_then(|n| n.checked_mul(result.faces() as usize))
                .ok_or(Error::LimitExceeded)?;
            result.mipmaps.push(Mip {
                face_size: size,
                data: Cow::Borrowed(range(bytes, offset, len)?),
            });
        }
        let count = u32_at(bytes, 536)? as usize;
        let offset = u32_at(bytes, 540)? as usize;
        let size = u32_at(bytes, 544)? as usize;
        if count.checked_mul(80).ok_or(Error::LimitExceeded)? != size {
            return Err(Error::Invalid("atlas frame size"));
        }
        for frame in range(bytes, offset, size)?.as_chunks::<80>().0 {
            let mut uv = [0.; 4];
            for (i, value) in uv.iter_mut().enumerate() {
                *value = f32::from_bits(u32_at(frame, i * 4)?);
            }
            result.frames.push(Frame {
                uv,
                name: frame[16..].try_into().unwrap(),
            });
        }
        result.frame_hashes = Cow::Borrowed(range(
            bytes,
            u32_at(bytes, 616)? as usize,
            u32_at(bytes, 620)? as usize,
        )?);
        Ok(result)
    }
    /// Build an empty native container, to which encoded mipmaps can be attached.
    pub fn new(width: u32, height: u32, faces: u32, format: u32) -> Result<Self, Error> {
        let mut header = [0; 624];
        for (at, val) in [
            (0, MAGIC),
            (4, V::VERSION),
            (32, format),
            (36, width),
            (40, height),
            (44, faces),
        ] {
            put_u32(&mut header, at, val);
        }
        let result = Self {
            header: Header::read(header)?,
            mipmaps: Vec::new(),
            frames: Vec::new(),
            frame_hashes: Cow::Borrowed(&[]),
            revision: PhantomData,
        };
        result.validate_dimensions()?;
        Ok(result)
    }
    /// Return the base image width.
    pub fn width(&self) -> u32 {
        self.header.width
    }
    /// Return the base image height.
    pub fn height(&self) -> u32 {
        self.header.height
    }
    /// Return the native face count.
    pub fn faces(&self) -> u32 {
        self.header.faces
    }
    /// Return the native pixel format identifier.
    pub fn format(&self) -> u32 {
        self.header.format
    }
    fn prefix_size(&self) -> usize {
        if matches!(self.format(), 9..=12 | 43) {
            16
        } else {
            0
        }
    }
    fn validate_dimensions(&self) -> Result<(), Error> {
        if self.width() == 0 || self.height() == 0 {
            return Err(Error::Invalid("zero texture dimensions"));
        }
        if !matches!(self.faces(), 1 | 6) || (self.faces() == 6 && self.width() != self.height()) {
            return Err(Error::Invalid("texture faces"));
        }
        Ok(())
    }
    /// Serialize the selected revision with relocated metadata and payload offsets.
    pub fn write(&self) -> Result<Vec<u8>, Error> {
        self.validate_dimensions()?;
        if self.mipmaps.len() > 60 {
            return Err(Error::Invalid("more than 60 mipmaps"));
        }
        let mut out = self.header.bytes::<V>().to_vec();
        put_u32(&mut out, 0, MAGIC);
        put_u32(&mut out, 4, V::VERSION);
        out[56..536].fill(0);
        put_u32(&mut out, 52, self.mipmaps.len().saturating_sub(1) as u32);
        put_u32(&mut out, 616, 624);
        put_u32(&mut out, 620, narrow(self.frame_hashes.len())?);
        out.extend_from_slice(&self.frame_hashes);
        while !out.len().is_multiple_of(8) {
            out.push(0);
        }
        let frames_offset = narrow(out.len())?;
        put_u32(&mut out, 536, narrow(self.frames.len())?);
        put_u32(&mut out, 540, frames_offset);
        put_u32(
            &mut out,
            544,
            narrow(
                self.frames
                    .len()
                    .checked_mul(80)
                    .ok_or(Error::LimitExceeded)?,
            )?,
        );
        for frame in &self.frames {
            for v in frame.uv {
                out.extend_from_slice(&v.to_le_bytes());
            }
            out.extend_from_slice(&frame.name);
        }
        for (i, mip) in self.mipmaps.iter().enumerate() {
            let expected = (mip.face_size as usize)
                .checked_add(self.prefix_size())
                .and_then(|n| n.checked_mul(self.faces() as usize))
                .ok_or(Error::LimitExceeded)?;
            if mip.face_size == 0 || mip.data.len() != expected {
                return Err(Error::Invalid("mip payload size"));
            }
            let offset = narrow(out.len())?;
            put_u32(&mut out, 56 + i * 8, offset);
            put_u32(&mut out, 60 + i * 8, mip.face_size);
            out.extend_from_slice(&mip.data);
        }
        narrow(out.len())?;
        Ok(out)
    }
    /// Borrow encoded buffers without copying their contents.
    pub fn as_ref(&self) -> Texture<'_, V> {
        Texture {
            header: self.header.clone(),
            mipmaps: self
                .mipmaps
                .iter()
                .map(|m| Mip {
                    face_size: m.face_size,
                    data: Cow::Borrowed(&m.data),
                })
                .collect(),
            frames: self.frames.clone(),
            frame_hashes: Cow::Borrowed(&self.frame_hashes),
            revision: PhantomData,
        }
    }
    /// Move owned buffers and copy borrowed buffers into independent storage.
    pub fn into_owned(self) -> Texture<'static, V> {
        Texture {
            header: self.header.clone(),
            mipmaps: self
                .mipmaps
                .into_iter()
                .map(|m| Mip {
                    face_size: m.face_size,
                    data: Cow::Owned(m.data.into_owned()),
                })
                .collect(),
            frames: self.frames,
            frame_hashes: Cow::Owned(self.frame_hashes.into_owned()),
            revision: PhantomData,
        }
    }
    #[cfg(feature = "tex-decode")]
    /// Decode the selected mip and face to RGBA8 pixels.
    pub fn decode_mip(&self, level: usize, face: usize) -> Result<image::RgbaImage, Error> {
        self.validate_dimensions()?;
        let mip = self.mipmaps.get(level).ok_or(Error::MissingMip)?;
        if face >= self.faces() as usize {
            return Err(Error::MissingMip);
        }
        let stride = (mip.face_size as usize)
            .checked_add(self.prefix_size())
            .ok_or(Error::LimitExceeded)?;
        let start = face
            .checked_mul(stride)
            .and_then(|n| n.checked_add(self.prefix_size()))
            .ok_or(Error::LimitExceeded)?;
        let data = range(&mip.data, start, mip.face_size as usize)?;
        let width = self.width().checked_shr(level as u32).unwrap_or(0).max(1);
        let height = self.height().checked_shr(level as u32).unwrap_or(0).max(1);
        super::pixels::decode_d3(self.format(), data, width, height)
    }
    #[cfg(feature = "tex-encode")]
    /// Encode RGBA8 pixels into a native mip payload.
    pub fn encode_mip(&self, image: &image::RgbaImage) -> Result<Mip<'static>, Error> {
        if self.faces() != 1 {
            return Err(Error::Invalid("encode_mip requires a 2D texture"));
        }
        let data = super::pixels::encode_d3(self.format(), image)?;
        Ok(Mip {
            face_size: narrow(data.len().saturating_sub(self.prefix_size()))?,
            data: Cow::Owned(data),
        })
    }
}
