//! Diablo IV TEX metadata and explicit external pixel payloads.
//!
//! The supported layout is hash F9CD83E7 (120-byte record). Other
//! hashes are rejected, including F9CD83E6 whose layout is not established here.
//! Metadata parsing neither opens files nor requires pixel payloads. Callers
//! select high or low payload storage explicitly when resolving a serialized
//! range; no heuristic guesses the streaming split.
use super::{narrow, put_u32, range, u32_at, Error, MAGIC};
use std::marker::PhantomData;
use zerovec::{ule::AsULE, ZeroVec};
mod sealed {
    pub trait Sealed {}
}
/// Established metadata layout.
pub trait Revision: sealed::Sealed + Clone + std::fmt::Debug + PartialEq {
    /// Native revision discriminator required by this layout.
    const HASH: u32;
}
/// TextureDefinition format hash F9CD83E7, 120-byte record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HashF9cd83e7;
impl sealed::Sealed for HashF9cd83e7 {}
impl Revision for HashF9cd83e7 {
    /// Native revision discriminator required by this layout.
    const HASH: u32 = 0xf9cd83e7;
}
/// External serialized pixel reference. The high size bit is a native flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PayloadRange {
    /// Offset relative to header-stripped external payload storage.
    pub offset: u32,
    /// Native size word; the high bit is a flag.
    pub size_and_flags: u32,
}
impl PayloadRange {
    /// Return the byte size with native flag bits removed.
    pub fn size(self) -> u32 {
        self.size_and_flags & 0x7fffffff
    }
    /// Resolve against a header-stripped payload. Header handling is explicit.
    pub fn resolve(self, payload: &[u8]) -> Result<&[u8], Error> {
        range(payload, self.offset as usize, self.size() as usize)
    }
}
/// Established TextureDefinition fields. Pointer slots are owned by the codec.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    /// Native SNO asset identifier.
    pub sno_id: i32,
    /// Native flag bits, including uninterpreted bits.
    pub flags: u32,
    /// Preset SNO identifier.
    pub ui_style_preset: u32,
    /// Native pixel format identifier; unknown values are retained.
    pub format: u32,
    /// Native X and Y volume slice counts.
    pub volume_slices: [u16; 2],
    /// Base image width in pixels.
    pub width: u16,
    /// Base image height in pixels.
    pub height: u16,
    /// Native texture depth.
    pub depth: u32,
    /// Native face count (one for 2D, six for cubemaps).
    pub faces: u8,
    /// Native minimum stored mip level.
    pub mip_min: u8,
    /// Native maximum stored mip level.
    pub mip_max: u8,
    /// Native import and streaming flags.
    pub import_flags: u32,
    /// Native resource kind: 0 for 2D, 1 for cubemap, 2 for volume.
    pub resource_type: u32,
    /// Average linear RGBA color.
    pub average_color: [f32; 4],
    /// Native image anchor coordinates.
    pub hotspot: [i32; 2],
    /// Unclassified native byte between mip selectors and import flags.
    pub reserved_31: u8,
    /// Unclassified trailing native bytes.
    pub reserved_112: [u8; 8],
}
impl Record {
    /// Construct a 2D record with one mip level and no external range attached.
    pub fn new(width: u16, height: u16, format: u32) -> Self {
        let mut raw = [0; 120];
        put_u32(&mut raw, 12, format);
        raw[16..18].copy_from_slice(&1u16.to_le_bytes());
        raw[18..20].copy_from_slice(&1u16.to_le_bytes());
        raw[20..22].copy_from_slice(&width.to_le_bytes());
        raw[22..24].copy_from_slice(&height.to_le_bytes());
        put_u32(&mut raw, 24, 1);
        raw[28] = 1;
        Self::read(raw).unwrap()
    }

    fn read(raw: [u8; 120]) -> Result<Self, Error> {
        let word = |at| -> u16 { u16::from_le_bytes(raw[at..at + 2].try_into().unwrap()) };
        let mut average_color = [0.; 4];
        for (i, v) in average_color.iter_mut().enumerate() {
            *v = f32::from_bits(u32_at(&raw, 40 + i * 4)?);
        }
        Ok(Self {
            sno_id: u32_at(&raw, 0)? as i32,
            flags: u32_at(&raw, 4)?,
            ui_style_preset: u32_at(&raw, 8)?,
            format: u32_at(&raw, 12)?,
            volume_slices: [word(16), word(18)],
            width: word(20),
            height: word(22),
            depth: u32_at(&raw, 24)?,
            faces: raw[28],
            mip_min: raw[29],
            mip_max: raw[30],
            import_flags: u32_at(&raw, 32)?,
            resource_type: u32_at(&raw, 36)?,
            average_color,
            hotspot: [u32_at(&raw, 56)? as i32, u32_at(&raw, 60)? as i32],
            reserved_31: raw[31],
            reserved_112: raw[112..120].try_into().unwrap(),
        })
    }
    fn bytes(&self) -> [u8; 120] {
        let mut raw = [0; 120];
        raw[31] = self.reserved_31;
        raw[112..120].copy_from_slice(&self.reserved_112);
        for (at, value) in [
            (0, self.sno_id as u32),
            (4, self.flags),
            (8, self.ui_style_preset),
            (12, self.format),
            (24, self.depth),
            (32, self.import_flags),
            (36, self.resource_type),
            (56, self.hotspot[0] as u32),
            (60, self.hotspot[1] as u32),
        ] {
            put_u32(&mut raw, at, value);
        }
        for (at, value) in [
            (16, self.volume_slices[0]),
            (18, self.volume_slices[1]),
            (20, self.width),
            (22, self.height),
        ] {
            raw[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        raw[28] = self.faces;
        raw[29] = self.mip_min;
        raw[30] = self.mip_max;
        for (i, value) in self.average_color.iter().enumerate() {
            put_u32(&mut raw, 40 + i * 4, value.to_bits());
        }
        raw
    }
}
/// Native atlas frame with independent original and trimmed UV bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    /// Frame image-handle hash.
    pub image_handle: u32,
    /// Original u-min, v-min, u-max, v-max bounds.
    pub uv: [f32; 4],
    /// Trimmed u-min, v-min, u-max, v-max bounds.
    pub trimmed_uv: [f32; 4],
}
impl AsULE for Frame {
    type ULE = <[u8; 36] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0; 36];
        put_u32(&mut bytes, 0, self.image_handle);
        for (i, value) in self.uv.into_iter().chain(self.trimmed_uv).enumerate() {
            put_u32(&mut bytes, 4 + i * 4, value.to_bits());
        }
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes: &[u8] = &value;
        let mut bounds = [0.; 8];
        for (i, value) in bounds.iter_mut().enumerate() {
            *value = f32::from_bits(u32_at(bytes, 4 + i * 4).unwrap());
        }
        Self {
            image_handle: u32_at(bytes, 0).unwrap(),
            uv: bounds[..4].try_into().unwrap(),
            trimmed_uv: bounds[4..].try_into().unwrap(),
        }
    }
}
/// Native metadata. Raw array values preserve presently uninterpreted fields.
#[derive(Clone, Debug, PartialEq)]
pub struct Texture<'a, V: Revision = HashF9cd83e7> {
    /// Reserved SNO header bytes, after magic and format hash.
    pub reserved: [u8; 8],
    /// Fixed native record; array offsets and lengths are rewritten on output.
    pub record: Record,
    /// Serialized external pixel references in native slot order.
    pub pixel_ranges: Vec<PayloadRange>,
    /// Native TextureFrame records.
    pub frames: ZeroVec<'a, Frame>,
    /// Native spherical harmonic coefficients.
    pub spherical_harmonics: ZeroVec<'a, [[f32; 4]; 3]>,
    revision: PhantomData<V>,
}
impl<'a, V: Revision> Texture<'a, V> {
    /// Construct metadata independently from external payload storage.
    pub fn new(record: Record) -> Self {
        Self {
            reserved: [0; 8],
            record,
            pixel_ranges: Vec::new(),
            frames: ZeroVec::new(),
            spherical_harmonics: ZeroVec::new(),
            revision: PhantomData,
        }
    }
    /// Encode one mip into header-stripped external payload storage, including
    /// native 256-byte row padding. Attaching its range and selecting a payload
    /// tier remain explicit caller operations.
    #[cfg(feature = "tex-encode")]
    /// Encode RGBA8 pixels into a native mip payload.
    pub fn encode_mip(&self, image: &image::RgbaImage) -> Result<Vec<u8>, Error> {
        super::pixels::encode_d4(self.format(), image)
    }

    /// Parse the selected native revision, borrowing leaf payload bytes.
    pub fn read(bytes: &'a [u8]) -> Result<Self, Error> {
        if u32_at(bytes, 0)? != MAGIC {
            return Err(Error::InvalidMagic);
        }
        let hash = u32_at(bytes, 4)?;
        if hash != V::HASH {
            return Err(Error::UnsupportedRevision(hash));
        }
        let record: [u8; 120] = range(bytes, 16, 120)?.try_into().unwrap();
        // Field offsets are relative to the record image, after the SNO header.
        let array = |at: usize| -> Result<&'a [u8], Error> {
            if range(&record, at, 8)?.iter().any(|byte| *byte != 0) {
                return Err(Error::Invalid(
                    "nonzero runtime pointer in serialized array",
                ));
            }
            let offset_word = u32_at(&record, at + 8)?;
            let size_word = u32_at(&record, at + 12)?;
            if offset_word > i32::MAX as u32 || size_word > i32::MAX as u32 {
                return Err(Error::Invalid("negative serialized array descriptor"));
            }
            let size = size_word as usize;
            if size == 0 {
                return Ok(&[]);
            }
            let offset = (u32_at(&record, at + 8)? as usize)
                .checked_add(16)
                .ok_or(Error::LimitExceeded)?;
            range(bytes, offset, size)
        };
        let ranges = array(64)?;
        if ranges.len() % 8 != 0 {
            return Err(Error::Invalid("serialized pixel range array"));
        }
        let pixel_ranges = ranges
            .chunks_exact(8)
            .map(|b| {
                Ok(PayloadRange {
                    offset: u32_at(b, 0)?,
                    size_and_flags: u32_at(b, 4)?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Self {
            reserved: range(bytes, 8, 8)?.try_into().unwrap(),
            record: Record::read(record)?,
            pixel_ranges,
            frames: ZeroVec::parse_bytes(array(80)?)
                .map_err(|_| Error::Invalid("atlas frame array"))?,
            spherical_harmonics: ZeroVec::parse_bytes(array(96)?)
                .map_err(|_| Error::Invalid("spherical harmonic array"))?,
            revision: PhantomData,
        })
    }
    /// Return the base image width.
    pub fn width(&self) -> u16 {
        self.record.width
    }
    /// Return the base image height.
    pub fn height(&self) -> u16 {
        self.record.height
    }
    /// Return the native pixel format identifier.
    pub fn format(&self) -> u32 {
        self.record.format
    }
    /// Return the native face count.
    pub fn faces(&self) -> u8 {
        self.record.faces
    }
    /// Serialize metadata while retaining external pixel references verbatim.
    /// Payload offsets refer to independent supplied resources, not this file.
    pub fn write(&self) -> Result<Vec<u8>, Error> {
        let mut out = Vec::with_capacity(136);
        out.extend_from_slice(&MAGIC.to_le_bytes());
        out.extend_from_slice(&V::HASH.to_le_bytes());
        out.extend_from_slice(&self.reserved);
        out.extend_from_slice(&self.record.bytes());
        let mut ranges = Vec::new();
        for r in &self.pixel_ranges {
            ranges.extend_from_slice(&r.offset.to_le_bytes());
            ranges.extend_from_slice(&r.size_and_flags.to_le_bytes());
        }
        for (at, bytes) in [
            (64, ranges.as_slice()),
            (80, self.frames.as_bytes()),
            (96, self.spherical_harmonics.as_bytes()),
        ] {
            while out.len() % 8 != 0 {
                out.push(0);
            }
            let offset = narrow(out.len() - 16)?;
            put_u32(
                &mut out,
                16 + at + 8,
                if bytes.is_empty() { 0 } else { offset },
            );
            put_u32(&mut out, 16 + at + 12, narrow(bytes.len())?);
            out.extend_from_slice(bytes);
        }
        narrow(out.len())?;
        Ok(out)
    }
    /// Borrow encoded buffers without copying their contents.
    pub fn as_ref(&self) -> Texture<'_, V> {
        Texture {
            reserved: self.reserved,
            record: self.record.clone(),
            pixel_ranges: self.pixel_ranges.clone(),
            frames: ZeroVec::parse_bytes(self.frames.as_bytes()).unwrap(),
            spherical_harmonics: ZeroVec::parse_bytes(self.spherical_harmonics.as_bytes()).unwrap(),
            revision: PhantomData,
        }
    }
    /// Move owned buffers and copy borrowed buffers into independent storage.
    pub fn into_owned(self) -> Texture<'static, V> {
        Texture {
            reserved: self.reserved,
            record: self.record.clone(),
            pixel_ranges: self.pixel_ranges,
            frames: self.frames.into_owned(),
            spherical_harmonics: self.spherical_harmonics.into_owned(),
            revision: PhantomData,
        }
    }
    /// Decode an explicitly selected range and mip index from a header-stripped
    /// external payload. Row pitches are aligned to 256 bytes on disk.
    #[cfg(feature = "tex-decode")]
    pub fn decode_range(
        &self,
        index: usize,
        mip: u32,
        payload: &[u8],
    ) -> Result<image::RgbaImage, Error> {
        if self.record.depth != 1 || self.record.resource_type > 1 || !matches!(self.faces(), 1 | 6)
        {
            return Err(Error::Invalid("unsupported texture dimensionality"));
        }
        if self.width() == 0 || self.height() == 0 {
            return Err(Error::Invalid("zero texture dimensions"));
        }
        let data = self
            .pixel_ranges
            .get(index)
            .ok_or(Error::MissingMip)?
            .resolve(payload)?;
        super::pixels::decode_d4(
            self.format(),
            data,
            u32::from(self.width()).checked_shr(mip).unwrap_or(0).max(1),
            u32::from(self.height())
                .checked_shr(mip)
                .unwrap_or(0)
                .max(1),
        )
    }
}
/// Validate and remove an external payload's SNO header. Use raw payload bytes
/// directly with [`PayloadRange::resolve`] when an extractor already removed it.
pub fn payload_body<V: Revision>(bytes: &[u8]) -> Result<&[u8], Error> {
    if u32_at(bytes, 0)? != MAGIC {
        return Err(Error::InvalidMagic);
    }
    let hash = u32_at(bytes, 4)?;
    if hash != V::HASH {
        return Err(Error::UnsupportedRevision(hash));
    }
    range(
        bytes,
        16,
        bytes.len().checked_sub(16).ok_or(Error::Truncated)?,
    )
}
