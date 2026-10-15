//! Runtime-selected profiles without treating MVER as a client-version detector.
use super::codec::{Decoder, Encoder};
use super::legacy::{File14, File16};
use super::{
    Bfa, Classic, Dialect, Dragonflight, File, Legion, ReadError, Shadowlands, ShadowlandsPre92,
    WarWithin, WriteError,
};
use super::{Read, Validate, Write};
use crate::model::common::Ownership;

/// Native layout profile chosen from client provenance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Profile {
    /// Alpha monolithic version-14 root and 128-byte group header family.
    Alpha14,
    /// Early-beta version-16 split documents.
    Legacy16,
    /// Classic through the pre-Legion record family.
    Classic,
    /// Legion group batch family.
    Legion,
    /// Battle for Azeroth group batch family.
    Bfa,
    /// Shadowlands before split-group links were introduced.
    ShadowlandsPre92,
    /// Shadowlands 9.2 split-group family.
    Shadowlands,
    /// Dragonflight family.
    Dragonflight,
    /// The War Within family.
    WarWithin,
}
/// A root or group retaining its selected native profile.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum DynamicFile<'data> {
    /// Alpha monolithic or standalone group document.
    Alpha14(Box<File14<'data>>),
    /// Early-beta root or group document.
    Legacy16(Box<File16<'data>>),
    /// Classic document.
    Classic(Box<File<'data, Classic>>),
    /// Legion document.
    Legion(Box<File<'data, Legion>>),
    /// Battle for Azeroth document.
    Bfa(Box<File<'data, Bfa>>),
    /// Early Shadowlands document.
    ShadowlandsPre92(Box<File<'data, ShadowlandsPre92>>),
    /// Shadowlands 9.2 document.
    Shadowlands(Box<File<'data, Shadowlands>>),
    /// Dragonflight document.
    Dragonflight(Box<File<'data, Dragonflight>>),
    /// The War Within document.
    WarWithin(Box<File<'data, WarWithin>>),
}
impl<'data> DynamicFile<'data> {
    /// Decodes using an explicit profile; the shared version number is not sufficient.
    pub fn decode(data: &'data [u8], profile: Profile) -> Result<Self, ReadError> {
        Ok(match profile {
            Profile::Alpha14 => Self::Alpha14(Box::new(File14::decode(data)?)),
            Profile::Legacy16 => Self::Legacy16(Box::new(File16::decode(data)?)),
            Profile::Classic => Self::Classic(Box::new(File::decode(data)?)),
            Profile::Legion => Self::Legion(Box::new(File::decode(data)?)),
            Profile::Bfa => Self::Bfa(Box::new(File::decode(data)?)),
            Profile::ShadowlandsPre92 => Self::ShadowlandsPre92(Box::new(File::decode(data)?)),
            Profile::Shadowlands => Self::Shadowlands(Box::new(File::decode(data)?)),
            Profile::Dragonflight => Self::Dragonflight(Box::new(File::decode(data)?)),
            Profile::WarWithin => Self::WarWithin(Box::new(File::decode(data)?)),
        })
    }
    /// Decodes only when structural evidence identifies one profile.
    /// Ambiguous version-17 layouts require an explicit caller-selected profile.
    pub fn decode_detected(data: &'data [u8]) -> Result<Self, ReadError> {
        let profiles = Self::candidates(data);
        match profiles.as_slice() {
            [profile] => Self::decode(data, *profile),
            [] => Err(ReadError::invalid(0, "unsupported WMO dialect")),
            _ => Err(ReadError::invalid(
                0,
                "ambiguous WMO dialect; explicit profile required",
            )),
        }
    }
    /// Returns every profile that can decode the framing and record widths.
    ///
    /// Successful decoding does not prove a profile's semantic interpretation.
    /// Multiple candidates require caller provenance rather than a guessed choice.
    pub fn candidates(data: &'data [u8]) -> Vec<Profile> {
        [
            Profile::Alpha14,
            Profile::Legacy16,
            Profile::Classic,
            Profile::Legion,
            Profile::Bfa,
            Profile::ShadowlandsPre92,
            Profile::Shadowlands,
            Profile::Dragonflight,
            Profile::WarWithin,
        ]
        .into_iter()
        .filter(|profile| Self::decode(data, *profile).is_ok())
        .collect()
    }
    /// Returns the selected profile.
    pub fn profile(&self) -> Profile {
        match self {
            Self::Alpha14(_) => Profile::Alpha14,
            Self::Legacy16(_) => Profile::Legacy16,
            Self::Classic(_) => Profile::Classic,
            Self::Legion(_) => Profile::Legion,
            Self::Bfa(_) => Profile::Bfa,
            Self::ShadowlandsPre92(_) => Profile::ShadowlandsPre92,
            Self::Shadowlands(_) => Profile::Shadowlands,
            Self::Dragonflight(_) => Profile::Dragonflight,
            Self::WarWithin(_) => Profile::WarWithin,
        }
    }
    /// Encodes the document without translating its dialect.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        crate::visit_wmo_file!(self, |file| file.encode())
    }
    /// Borrows the document only when its native profile matches.
    pub fn as_typed<D: DynamicDialect>(&self) -> Option<&File<'data, D>> {
        D::get(self)
    }
    /// Edits the document only when its native profile matches.
    pub fn as_typed_mut<D: DynamicDialect>(&mut self) -> Option<&mut File<'data, D>> {
        D::get_mut(self)
    }
}
/// Conversion between a concrete profile and its runtime wrapper.
pub trait DynamicDialect: Dialect {
    /// Extracts a matching borrowed document.
    fn get<'view, 'data>(file: &'view DynamicFile<'data>) -> Option<&'view File<'data, Self>>;
    /// Extracts a matching editable document.
    fn get_mut<'view, 'data>(
        file: &'view mut DynamicFile<'data>,
    ) -> Option<&'view mut File<'data, Self>>;
}
macro_rules! typed {($($profile:ident),*)=>{$(
impl DynamicDialect for $profile{
fn get<'view,'data>(file:&'view DynamicFile<'data>)->Option<&'view File<'data,Self>>{if let DynamicFile::$profile(file)=file{Some(file)}else{None}}
fn get_mut<'view,'data>(file:&'view mut DynamicFile<'data>)->Option<&'view mut File<'data,Self>>{if let DynamicFile::$profile(file)=file{Some(file)}else{None}}
}
)*};}
typed!(
    Classic,
    Legion,
    Bfa,
    ShadowlandsPre92,
    Shadowlands,
    Dragonflight,
    WarWithin
);
/// Dispatches an operation over a runtime-selected WMO document.
#[macro_export]
macro_rules! visit_wmo_file {
    ($file:expr, |$typed:ident| $body:expr) => {
        match $file {
            $crate::model::formats::wmo::DynamicFile::Alpha14($typed) => $body,
            $crate::model::formats::wmo::DynamicFile::Legacy16($typed) => $body,
            $crate::model::formats::wmo::DynamicFile::Classic($typed) => $body,
            $crate::model::formats::wmo::DynamicFile::Legion($typed) => $body,
            $crate::model::formats::wmo::DynamicFile::Bfa($typed) => $body,
            $crate::model::formats::wmo::DynamicFile::ShadowlandsPre92($typed) => $body,
            $crate::model::formats::wmo::DynamicFile::Shadowlands($typed) => $body,
            $crate::model::formats::wmo::DynamicFile::Dragonflight($typed) => $body,
            $crate::model::formats::wmo::DynamicFile::WarWithin($typed) => $body,
        }
    };
}

impl<'data> Read<'data> for DynamicFile<'data> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let origin = input.absolute_position();
        let length = input.remaining();
        Self::decode_detected(input.bytes(length)?).map_err(|mut error| {
            error.offset = error.offset.saturating_add(origin);
            error
        })
    }
}
impl Write for DynamicFile<'_> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.encode()?)
    }
}
impl Validate for DynamicFile<'_> {
    fn validate(&self) -> Result<(), ReadError> {
        crate::visit_wmo_file!(self, |file| file.validate())
    }
}
