//! Load complete native models without choosing a filesystem or asynchronous runtime.
//!
//! A [`Reader`] requests the supporting documents required by the native format.
//! Supply each [`Request`] from an archive, filesystem, or network, then poll again.
//! Images and optional scene children remain semantic dependencies rather than
//! required model companions. Completion returns independently owned checked
//! semantics together with the report from the session's explicit import policy.
use super::mdlx as semantic_mdlx;
use crate::model::common::FileKey;
use crate::model::conversion::{Conversion, ImportOptions};
use crate::model::formats::mdlx::{mdl::Read as _, DynamicDocument, NoExtensions};
use crate::model::formats::wmo;
use crate::model::Model;
use crate::visit_document;
use std::collections::BTreeMap;
use std::str::from_utf8;

mod error;
pub use error::{DecodeError, Error, LoadError};
mod m2;
mod world;

/// Native family and any provenance required to interpret its layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    /// Warcraft III MDX or MDL; the `MDLX` header selects binary decoding.
    Mdlx,
    /// World of Warcraft M2; the core version and envelope are detected.
    M2,
    /// World of Warcraft WMO with an explicitly selected client layout.
    Wmo(wmo::Profile),
}

/// The role a required native companion supplies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompanionKind {
    /// An external base or LOD mesh view.
    Skin {
        /// Index within the native base or LOD view list.
        index: usize,
        /// Whether the index selects an additional LOD view.
        lod: bool,
    },
    /// External keys for one native animation and variation.
    Animation {
        /// Native animation ID.
        animation_id: u16,
        /// Native animation variation.
        variation_id: u16,
    },
    /// A shared skeleton or parent skeleton.
    Skeleton,
    /// Sparse native bone transforms.
    Bone,
    /// Authored physics data.
    Physics,
    /// Geometry or an emitter model required by an M2 bundle.
    Model,
    /// WMO group geometry; zero is the base LOD.
    WmoGroup {
        /// Index in the root group-info table.
        index: usize,
        /// Zero for base geometry; larger values select LOD rows.
        lod: usize,
    },
}

/// A native companion identity together with its declaring owner and role.
///
/// Paths, raw filename bytes, file IDs, and generated siblings retain their
/// original meaning. Mapping them to physical storage belongs to the caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request {
    /// Document that declares the dependency.
    pub owner: FileKey,
    /// Identity of the requested document.
    pub key: FileKey,
    /// Interpretation of the supporting bytes.
    pub kind: CompanionKind,
}

/// One loading step, requiring resource access or returning checked semantics.
#[derive(Debug)]
pub enum Progress {
    /// Supply this required companion before continuing.
    Pending(Request),
    /// Native companions are complete and checked import succeeded.
    Ready(Box<Conversion<Model<'static>>>),
}

/// Incremental loading session independent of storage and executor choices.
///
/// Each successful supply is cached by native identity. Repeated polls while a
/// request is pending return that same request. Native bundle decoding remains
/// authoritative for recursive dependencies, validation, and animation context.
#[derive(Debug)]
pub struct Reader {
    format: Format,
    key: FileKey,
    bytes: Vec<u8>,
    options: ImportOptions,
    buffers: BTreeMap<FileKey, Vec<u8>>,
    m2: Option<m2::Preparation>,
    pending: Option<Request>,
    finished: bool,
}
impl Reader {
    /// Starts a session with a fixed native loss and approximation policy.
    pub fn new(
        format: Format,
        key: FileKey,
        bytes: impl Into<Vec<u8>>,
        options: ImportOptions,
    ) -> Self {
        Self {
            format,
            key,
            bytes: bytes.into(),
            options,
            buffers: BTreeMap::new(),
            m2: None,
            pending: None,
            finished: false,
        }
    }
    /// Decodes and imports until a companion is missing or an owned model is ready.
    /// Successful completion releases source buffers and forbids further polling.
    /// An import failure retains its sources and the session's fixed policy.
    pub fn poll(&mut self) -> Result<Progress, Error> {
        if self.finished {
            return Err(Error::Finished);
        }
        if let Some(request) = &self.pending {
            return Ok(Progress::Pending(request.clone()));
        }
        let mut missing = None;
        let converted = match self.format {
            Format::Mdlx => {
                let document = if self.bytes.starts_with(b"MDLX") {
                    DynamicDocument::<NoExtensions>::decode_mdx(&self.bytes, 800)
                        .map_err(|source| self.decode_error(DecodeError::Mdx(source)))?
                } else {
                    let text = from_utf8(&self.bytes)
                        .map_err(|source| self.decode_error(DecodeError::Utf8(source)))?;
                    DynamicDocument::<NoExtensions>::decode_mdl(text)
                        .map_err(|source| self.decode_error(DecodeError::Mdl(source)))?
                };
                Some(
                    visit_document!(&document, |document| {
                        semantic_mdlx::import(document, &self.options).map(|converted| Conversion {
                            value: converted.value.widen().into_owned(),
                            report: converted.report,
                        })
                    })
                    .map_err(|source| Error::Import {
                        key: self.key.clone(),
                        source,
                    })?,
                )
            }
            Format::M2 => {
                if self.m2.is_none() {
                    self.m2 = Some(
                        m2::prepare(&self.bytes, &self.key)
                            .map_err(|source| self.decode_error(DecodeError::M2(source)))?,
                    );
                }
                m2::decode(
                    self.m2.as_ref().unwrap(),
                    &self.bytes,
                    &self.key,
                    &self.buffers,
                    &mut missing,
                    &self.options,
                )?
            }
            Format::Wmo(profile) => world::decode(
                &self.bytes,
                &self.key,
                profile,
                &self.buffers,
                &mut missing,
                &self.options,
            )?,
        };
        if let Some(request) = missing {
            self.pending = Some(request.clone());
            return Ok(Progress::Pending(request));
        }
        let converted =
            converted.expect("native decoding either completes or requests a companion");
        self.finished = true;
        self.bytes = Vec::new();
        self.buffers.clear();
        self.m2 = None;
        Ok(Progress::Ready(Box::new(converted)))
    }
    /// Supplies bytes for the current request. Stale or unrelated requests fail
    /// without changing the session or replacing already accepted buffers.
    pub fn provide(&mut self, request: &Request, bytes: impl Into<Vec<u8>>) -> Result<(), Error> {
        if self.pending.as_ref() != Some(request) {
            return Err(Error::UnexpectedCompanion {
                expected: self.pending.clone().map(Box::new),
                supplied: Box::new(request.clone()),
            });
        }
        self.buffers.insert(request.key.clone(), bytes.into());
        self.pending = None;
        Ok(())
    }
    fn decode_error(&self, source: DecodeError) -> Error {
        Error::Decode {
            key: self.key.clone(),
            source,
        }
    }
}

/// Loads a complete native model through synchronous caller-owned resource access.
///
/// The resolver receives only required native companions, retaining their full
/// identities and roles. Root bytes and format provenance are supplied directly;
/// filesystem paths, archive lookups, and missing-file policy remain with the
/// caller. Asynchronous callers can drive [`Reader`] one request at a time.
pub fn load<E>(
    format: Format,
    key: FileKey,
    bytes: impl Into<Vec<u8>>,
    options: ImportOptions,
    mut resolve: impl FnMut(&Request) -> Result<Vec<u8>, E>,
) -> Result<Conversion<Model<'static>>, LoadError<E>> {
    let mut reader = Reader::new(format, key, bytes, options);
    loop {
        match reader.poll().map_err(LoadError::Native)? {
            Progress::Pending(request) => {
                let bytes = resolve(&request).map_err(|source| LoadError::Companion {
                    request: request.clone(),
                    source,
                })?;
                reader.provide(&request, bytes).map_err(LoadError::Native)?;
            }
            Progress::Ready(converted) => return Ok(*converted),
        }
    }
}
