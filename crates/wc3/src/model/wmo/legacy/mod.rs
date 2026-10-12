//! Native alpha and early-beta layouts identified by MVER14 and MVER16.
//!
//! Version14 keeps root chunks inside MOMO and embeds group payloads in the same
//! file. Version16 separates group files, but retains four-byte triangle records
//! and thirty-two-byte draw batches. These are independent document types rather
//! than version17 records relabeled with another number.
mod group;
pub use group::*;
mod root;
use crate::model::{common::Ownership, wmo};
pub use root::*;
use wmo::codec::{Decoder, Encoder};
use wmo::{Read, ReadError, Validate, Write, WriteError};
/// A monolithic alpha root or an explicitly supplied standalone alpha group.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum File14<'data> {
    /// Root chunks and embedded groups.
    Root(Box<Root14<'data>>),
    /// Individual group representation.
    Group(Box<Group14<'data>>),
}
impl<'data> File14<'data> {
    /// Detects MOMO root framing versus MOGP group framing.
    pub fn decode(data: &'data [u8]) -> Result<Self, ReadError> {
        let root = wmo::document::frames(data, 0)?
            .iter()
            .any(|(tag, _, _)| tag == b"MOMO");
        if root {
            Ok(Self::Root(Box::new(Root14::decode(data)?)))
        } else {
            Ok(Self::Group(Box::new(Group14::decode(data)?)))
        }
    }
    /// Encodes the native document representation.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        match self {
            Self::Root(root) => root.encode(),
            Self::Group(group) => group.encode(),
        }
    }
    /// Validates known local record relationships.
    pub fn validate(&self) -> Result<(), ReadError> {
        match self {
            Self::Root(root) => root.validate(),
            Self::Group(group) => group.validate(),
        }
    }
}
/// A version16 root or group document.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum File16<'data> {
    /// Root declaring external group files.
    Root(Box<Root16<'data>>),
    /// Group geometry in the early-beta layout.
    Group(Box<Group16<'data>>),
}
impl<'data> File16<'data> {
    /// Detects MOHD root framing versus MOGP group framing.
    pub fn decode(data: &'data [u8]) -> Result<Self, ReadError> {
        let root = wmo::document::frames(data, 0)?
            .iter()
            .any(|(tag, _, _)| tag == b"MOHD");
        if root {
            Ok(Self::Root(Box::new(Root16::decode(data)?)))
        } else {
            Ok(Self::Group(Box::new(Group16::decode(data)?)))
        }
    }
    /// Encodes the native document representation.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        match self {
            Self::Root(root) => root.encode(),
            Self::Group(group) => group.encode(),
        }
    }
    /// Validates known local record relationships.
    pub fn validate(&self) -> Result<(), ReadError> {
        match self {
            Self::Root(root) => root.validate(),
            Self::Group(group) => group.validate(),
        }
    }
}
impl<'data> Read<'data> for File14<'data> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let origin = input.absolute_position();
        let bytes = input.bytes(input.remaining())?;
        Self::decode(bytes).map_err(|mut e| {
            e.offset = origin.saturating_add(e.offset);
            e
        })
    }
}
impl Write for File14<'_> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.encode()?)
    }
}
impl<'data> Read<'data> for File16<'data> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let origin = input.absolute_position();
        let bytes = input.bytes(input.remaining())?;
        Self::decode(bytes).map_err(|mut e| {
            e.offset = origin.saturating_add(e.offset);
            e
        })
    }
}
impl Write for File16<'_> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.encode()?)
    }
}

mod bundle;
pub use bundle::Bundle16;
