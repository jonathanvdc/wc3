//! Load complete native models without choosing a filesystem or asynchronous runtime.
//!
//! A [`Reader`](crate::model::io::asset::Reader) requests the supporting documents
//! required by the native format. Supply each [`Request`](crate::model::io::asset::Request)
//! from an archive, filesystem, or network, then poll again.
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
use std::ops::Deref;
use std::str::from_utf8;

mod error;
pub use error::{DecodeError, Error, LoadError};
mod m2;
mod world;

fn source_bytes<B>(source: &B) -> &[u8]
where
    B: Deref,
    B::Target: AsRef<[u8]>,
{
    source.deref().as_ref()
}

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

// Accepted companion buffers remain present until completion, so the fulfilled
// prefix never needs to be visited again. Aliased native identities share bytes.
#[derive(Debug)]
struct CompanionPlan {
    requests: Vec<Request>,
    next: usize,
    #[cfg(test)]
    examined: usize,
}
impl CompanionPlan {
    fn new(requests: Vec<Request>) -> Self {
        Self {
            requests,
            next: 0,
            #[cfg(test)]
            examined: 0,
        }
    }
    fn next_missing<B>(&mut self, buffers: &BTreeMap<FileKey, B>) -> Option<Request> {
        while let Some(request) = self.requests.get(self.next) {
            #[cfg(test)]
            {
                self.examined += 1;
            }
            if !buffers.contains_key(&request.key) {
                return Some(request.clone());
            }
            self.next += 1;
        }
        None
    }
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
/// The caller chooses one source-handle type for the root and companions, such
/// as `Vec<u8>`, `&[u8]`, or `Arc<Vec<u8>>`. The reader retains those handles
/// directly without cloning encoded buffers; completion releases its handles
/// after owning the checked graph.
/// Each handle must expose unchanged bytes until the reader releases it.
#[derive(Debug)]
pub struct Reader<B = Vec<u8>> {
    format: Format,
    key: FileKey,
    bytes: Option<B>,
    options: ImportOptions,
    buffers: BTreeMap<FileKey, B>,
    m2: Option<m2::Preparation>,
    world: Option<CompanionPlan>,
    pending: Option<Request>,
    finished: bool,
}
impl<B> Reader<B>
where
    B: Deref,
    B::Target: AsRef<[u8]>,
{
    /// Starts a session with a fixed native loss and approximation policy.
    pub fn new(format: Format, key: FileKey, bytes: B, options: ImportOptions) -> Self {
        Self {
            format,
            key,
            bytes: Some(bytes),
            options,
            buffers: BTreeMap::new(),
            m2: None,
            world: None,
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
        let bytes = source_bytes(
            self.bytes
                .as_ref()
                .expect("unfinished reader retains its root"),
        );
        let mut missing = None;
        let converted = match self.format {
            Format::Mdlx => {
                let document = if bytes.starts_with(b"MDLX") {
                    DynamicDocument::<NoExtensions>::decode_mdx(bytes, 800)
                        .map_err(|source| self.decode_error(DecodeError::Mdx(source)))?
                } else {
                    let text = from_utf8(bytes)
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
                        m2::prepare(bytes, &self.key)
                            .map_err(|source| self.decode_error(DecodeError::M2(source)))?,
                    );
                }
                m2::decode(
                    self.m2.as_mut().unwrap(),
                    bytes,
                    &self.key,
                    &self.buffers,
                    &mut missing,
                    &self.options,
                )?
            }
            Format::Wmo(profile) => {
                if self.world.is_none() {
                    self.world = Some(world::prepare(bytes, &self.key, profile)?);
                }
                world::decode(
                    self.world.as_mut().unwrap(),
                    bytes,
                    &self.key,
                    profile,
                    &self.buffers,
                    &mut missing,
                    &self.options,
                )?
            }
        };
        if let Some(request) = missing {
            self.pending = Some(request.clone());
            return Ok(Progress::Pending(request));
        }
        let converted =
            converted.expect("native decoding either completes or requests a companion");
        self.finished = true;
        self.bytes = None;
        self.buffers.clear();
        self.m2 = None;
        self.world = None;
        Ok(Progress::Ready(Box::new(converted)))
    }
    /// Supplies bytes for the current request. Stale or unrelated requests fail
    /// without changing the session or replacing already accepted buffers.
    pub fn provide(&mut self, request: &Request, bytes: B) -> Result<(), Error> {
        if self.pending.as_ref() != Some(request) {
            return Err(Error::UnexpectedCompanion {
                expected: self.pending.clone().map(Box::new),
                supplied: Box::new(request.clone()),
            });
        }
        self.buffers.insert(request.key.clone(), bytes);
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
/// Root and resolver results use the same caller-selected source-handle type.
pub fn load<E, B>(
    format: Format,
    key: FileKey,
    bytes: B,
    options: ImportOptions,
    mut resolve: impl FnMut(&Request) -> Result<B, E>,
) -> Result<Conversion<Model<'static>>, LoadError<E>>
where
    B: Deref,
    B::Target: AsRef<[u8]>,
{
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supplied_prefix_and_aliases_are_examined_only_once() {
        let count = 1_024;
        let requests = (0..count)
            .map(|index| Request {
                owner: FileKey::Root,
                key: FileKey::FileId((index / 2) as u32),
                kind: CompanionKind::Skin { index, lod: false },
            })
            .collect();
        let mut plan = CompanionPlan::new(requests);
        let mut buffers = BTreeMap::<FileKey, Vec<u8>>::new();
        for index in 0..count / 2 {
            let request = plan.next_missing(&buffers).unwrap();
            assert_eq!(request.key, FileKey::FileId(index as u32));
            assert_eq!(
                request.kind,
                CompanionKind::Skin {
                    index: index * 2,
                    lod: false
                }
            );
            buffers.insert(request.key, Vec::new());
        }
        assert!(plan.next_missing(&buffers).is_none());
        assert_eq!(plan.examined, count + count / 2);
        assert_eq!(plan.next, count);
        assert!(plan.next_missing(&buffers).is_none());
        assert_eq!(plan.examined, count + count / 2);
    }
}
