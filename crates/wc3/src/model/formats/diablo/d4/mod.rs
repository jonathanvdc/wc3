//! Diablo IV SNO model documents with independently selected format hashes.
//!
//! Rust record declarations define the four supported root layouts. A format hash
//! identifies a root layout, rather than a game-wide
//! revision. Older hashes that merely share a root name are rejected.
//!
//! Array offsets are relative to the body after the 16-byte SNO envelope.
//! External arrays use a separate supplied payload body. Reading never performs
//! external I/O; a nonempty external array requires `read_with_payload`.
//! Writing relocates all supported descriptors and returns both address spaces.
//! Unknown polymorphic record hashes fail explicitly instead of being flattened.

pub mod codec;
pub mod records;
use crate::model::common::{Borrowed, Detached, Ownership};
use codec::{Decoder, Encoder, Space};
use records::{ActorRecord, AnimationRecord, AppearanceRecord, MaterialRecord};
use std::{fmt, marker::PhantomData};
use zerovec::ule::AsULE;

pub use wc3_derive::{D4Read as Read, D4RecordLayout as RecordLayout, D4Write as Write};

/// A malformed or unsupported native input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadError {
    /// Header signature is not the SNO signature.
    InvalidMagic,
    /// No established layout is registered for this hash.
    UnsupportedFormat(u32),
    /// A polymorphic record has no established concrete codec.
    UnsupportedType(u32),
    /// An external array has no supplied external payload.
    MissingPayload,
    /// A byte range falls outside its address space.
    Truncated {
        /// Requested start within the active address space.
        offset: usize,
        /// Requested byte count.
        size: usize,
    },
    /// A descriptor or native invariant is invalid.
    Invalid(&'static str),
    /// Address arithmetic overflowed.
    Overflow,
}
impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Diablo IV read error: {self:?}")
    }
}
impl std::error::Error for ReadError {}
/// A native output cannot represent the authored value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WriteError {
    /// Address arithmetic exceeds native signed offsets.
    Overflow,
    /// An authored native invariant is invalid.
    Invalid(&'static str),
}
impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Diablo IV write error: {self:?}")
    }
}
impl std::error::Error for WriteError {}
/// Fixed byte width of a native record before its referenced arrays.
pub trait RecordLayout {
    /// Fixed byte width before referenced data.
    const SIZE: usize;
}
/// Checked native record parsing within the selected address space.
pub trait Read<'data>: RecordLayout + Sized {
    /// Reads one record at a checked body-relative offset.
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError>;
}
/// Native record serialization with relocated array descriptors.
pub trait Write: RecordLayout {
    /// Writes one record into already reserved fixed storage.
    fn write(&self, encoder: &mut Encoder, offset: usize, space: Space) -> Result<(), WriteError>;
}
macro_rules! scalar {($($t:ty),*)=>{$(impl RecordLayout for $t{const SIZE:usize=size_of::<<Self as AsULE>::ULE>();}impl<'a> Read<'a> for $t{fn read(d:&Decoder<'a>,o:usize)->Result<Self,ReadError>{d.value(o)}}impl Write for $t{fn write(&self,e:&mut Encoder,o:usize,s:Space)->Result<(),WriteError>{e.value(o,self,s)}})*}}
scalar!(u8, u16, u32, i32, u64, f32, [u8; 4], [i32; 2], [f32; 2], [f32; 3], [f32; 4]);
mod sealed {
    pub trait Layout {}
}
/// A supported independent native root layout.
pub trait Layout: sealed::Layout + Clone + Copy + fmt::Debug + Default + Eq + 'static {
    /// Exact native format hash.
    const FORMAT_HASH: u32;
    /// Native resource group.
    const GROUP: i32;
    /// Concrete records selected by this layout.
    type Record<'a>: Read<'a> + Write + Clone + fmt::Debug + Default;
    /// Borrows the root's encoded leaves.
    fn borrow_record<'v, 'a>(record: &'v Self::Record<'a>) -> Self::Record<'v>;
    /// Detaches the root's encoded leaves.
    fn own_record(record: Self::Record<'_>) -> Self::Record<'static>;
}
macro_rules! layout {
    ($name:ident,$hash:literal,$group:literal,$record:ident,$doc:literal) => {
        #[doc=$doc]
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        pub struct $name;
        impl sealed::Layout for $name {}
        impl Layout for $name {
            const FORMAT_HASH: u32 = $hash;
            const GROUP: i32 = $group;
            type Record<'a> = $record<'a>;
            fn borrow_record<'v, 'a>(record: &'v Self::Record<'a>) -> Self::Record<'v> {
                record.as_borrowed()
            }
            fn own_record(record: Self::Record<'_>) -> Self::Record<'static> {
                record.into_owned()
            }
        }
    };
}
layout!(
    ActorC23b892a,
    0xc23b892a,
    1,
    ActorRecord,
    "Actor root with format hash 0xC23B892A."
);
layout!(
    Animation2088d14a,
    0x2088d14a,
    6,
    AnimationRecord,
    "Animation root with format hash 0x2088D14A."
);
layout!(
    Appearance297828cc,
    0x297828cc,
    9,
    AppearanceRecord,
    "Appearance root with format hash 0x297828CC."
);
layout!(
    MaterialBf40b801,
    0xbf40b801,
    57,
    MaterialRecord,
    "Material root with format hash 0xBF40B801."
);
/// SNO envelope signature, stored as a little-endian word.
pub const MAGIC: u32 = 0xdeadbeef;
/// Native file whose root layout is selected at compile time.
#[derive(Clone, Debug)]
pub struct Document<'a, L: Layout> {
    /// Header words whose meanings remain unresolved.
    pub reserved: [u32; 2],
    /// The concrete editable root record.
    pub root: L::Record<'a>,
    layout: PhantomData<L>,
}
impl<'a, L: Layout> Default for Document<'a, L> {
    fn default() -> Self {
        Self {
            reserved: [0; 2],
            root: Default::default(),
            layout: PhantomData,
        }
    }
}
/// Encoded root file and external payload body; storage naming remains caller-owned.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EncodedModel {
    /// Complete root SNO file, including its envelope.
    pub root: Vec<u8>,
    /// External payload body without a storage envelope.
    pub payload: Vec<u8>,
}
impl<'a, L: Layout> Document<'a, L> {
    /// Constructs a native document with the root layout selected by `L`.
    pub fn new(root: L::Record<'a>) -> Self {
        Self {
            reserved: [0; 2],
            root,
            layout: PhantomData,
        }
    }

    /// Borrows all encoded leaves without copying their buffers.
    pub fn as_ref(&self) -> Document<'_, L> {
        Document {
            reserved: self.reserved,
            root: L::borrow_record(&self.root),
            layout: PhantomData,
        }
    }
    /// Detaches borrowed buffers and moves owned buffers.
    pub fn into_owned(self) -> Document<'static, L> {
        Document {
            reserved: self.reserved,
            root: L::own_record(self.root),
            layout: PhantomData,
        }
    }
    /// Reads a document without external payload data.
    pub fn read(bytes: &'a [u8]) -> Result<Self, ReadError> {
        Self::decode(bytes, None)
    }
    /// Reads a document and a separately supplied payload body.
    /// The payload argument excludes any SNO envelope belonging to its storage.
    pub fn read_with_payload(bytes: &'a [u8], payload: &'a [u8]) -> Result<Self, ReadError> {
        Self::decode(bytes, Some(payload))
    }
    fn decode(bytes: &'a [u8], payload: Option<&'a [u8]>) -> Result<Self, ReadError> {
        let header = Decoder::new(bytes, None);
        if header.value::<u32>(0)? != MAGIC {
            return Err(ReadError::InvalidMagic);
        }
        let format = header.value::<u32>(4)?;
        if format != L::FORMAT_HASH {
            return Err(ReadError::UnsupportedFormat(format));
        }
        let reserved = [header.value(8)?, header.value(12)?];
        let body = bytes.get(16..).ok_or(ReadError::Truncated {
            offset: 16,
            size: 0,
        })?;
        let root = L::Record::read(&Decoder::new(body, payload), 0)?;
        Ok(Self {
            reserved,
            root,
            layout: PhantomData,
        })
    }
    /// Encodes fresh root-file and external-payload layouts.
    pub fn write_files(&self) -> Result<EncodedModel, WriteError> {
        let mut encoder = Encoder::new(L::Record::SIZE);
        self.root.write(&mut encoder, 0, Space::Main)?;
        let (body, payload) = encoder.finish();
        let mut root = Vec::with_capacity(16 + body.len());
        for word in [MAGIC, L::FORMAT_HASH, self.reserved[0], self.reserved[1]] {
            root.extend_from_slice(&word.to_le_bytes());
        }
        root.extend_from_slice(&body);
        Ok(EncodedModel { root, payload })
    }
    /// Encodes a document requiring no external payload.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        let out = self.write_files()?;
        if !out.payload.is_empty() {
            return Err(WriteError::Invalid("external payload requires write_files"));
        }
        Ok(out.root)
    }
}
/// Typed appearance document.
pub type Appearance<'a> = Document<'a, Appearance297828cc>;
/// Typed actor document.
pub type Actor<'a> = Document<'a, ActorC23b892a>;
/// Typed animation document.
pub type Animation<'a> = Document<'a, Animation2088d14a>;
/// Typed material document.
pub type Material<'a> = Document<'a, MaterialBf40b801>;
/// Supported runtime-detected root documents.
#[derive(Clone, Debug)]
pub enum DynamicDocument<'a> {
    /// Native actor document.
    Actor(Box<Actor<'a>>),
    /// Native animation document.
    Animation(Box<Animation<'a>>),
    /// Native appearance document.
    Appearance(Box<Appearance<'a>>),
    /// Native material document.
    Material(Box<Material<'a>>),
}
impl<'a> DynamicDocument<'a> {
    /// Dispatches exact supported native format hashes without external payload data.
    pub fn read(bytes: &'a [u8]) -> Result<Self, ReadError> {
        Self::decode(bytes, None)
    }
    /// Dispatches exact supported hashes with a separately supplied external payload body.
    pub fn read_with_payload(bytes: &'a [u8], payload: &'a [u8]) -> Result<Self, ReadError> {
        Self::decode(bytes, Some(payload))
    }
    fn decode(bytes: &'a [u8], payload: Option<&'a [u8]>) -> Result<Self, ReadError> {
        let decoder = Decoder::new(bytes, None);
        if decoder.value::<u32>(0)? != MAGIC {
            return Err(ReadError::InvalidMagic);
        }
        let hash = decoder.value::<u32>(4)?;
        macro_rules! parse {
            ($l:ty,$v:ident) => {
                Ok(Self::$v(Box::new(Document::<$l>::decode(bytes, payload)?)))
            };
        }
        match hash {
            ActorC23b892a::FORMAT_HASH => parse!(ActorC23b892a, Actor),
            Animation2088d14a::FORMAT_HASH => parse!(Animation2088d14a, Animation),
            Appearance297828cc::FORMAT_HASH => parse!(Appearance297828cc, Appearance),
            MaterialBf40b801::FORMAT_HASH => parse!(MaterialBf40b801, Material),
            other => Err(ReadError::UnsupportedFormat(other)),
        }
    }
}
// Keep the encoded-leaf ownership vocabulary available beside native records.
pub use crate::model::common::{AsULE as NativeValue, IntoZeroVec};
#[cfg(test)]
mod tests;

/// An empty native tag map. Nonempty dynamic tag maps have no established codec.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership)]
pub struct EmptyTagMap;
impl RecordLayout for EmptyTagMap {
    const SIZE: usize = 16;
}
impl<'a> Read<'a> for EmptyTagMap {
    fn read(d: &Decoder<'a>, o: usize) -> Result<Self, ReadError> {
        d.range(o, Self::SIZE)?;
        let size: i32 = d.value(o + 12)?;
        if size != 0 {
            return Err(ReadError::UnsupportedType(3493213809));
        }
        let _ = d.array::<u8>(o, false)?;
        Ok(Self)
    }
}
impl Write for EmptyTagMap {
    fn write(&self, e: &mut Encoder, o: usize, s: Space) -> Result<(), WriteError> {
        e.range(o, Self::SIZE, s)?;
        e.value(o, &0u64, s)?;
        e.value(o + 8, &0u64, s)
    }
}

impl<'v, 'a, L: Layout> Borrowed<'v> for Document<'a, L> {
    type View = Document<'v, L>;
    fn as_borrowed(&'v self) -> Self::View {
        self.as_ref()
    }
}
impl<'a, L: Layout> Detached<'static> for Document<'a, L> {
    type Detached = Document<'static, L>;
    fn into_detached(self) -> Self::Detached {
        self.into_owned()
    }
}
