use super::super::common::FileKey;
use super::super::wem::{self, Document, V3};
use super::{Error, ExportOptions, Format};
use std::borrow::Cow;

/// One uninterpreted native payload retained only when explicitly requested.
#[derive(Clone, Debug)]
pub struct OpaquePayload {
    /// Source format.
    pub format: Format,
    /// Native file identity, including companion ownership.
    pub file: FileKey,
    /// Native payload tag.
    pub tag: [u8; 4],
    /// Independently versioned record layout when supplied by the source.
    pub revision: Option<u32>,
    /// Declared row count when supplied by the source.
    pub count: Option<u32>,
    /// Payload bytes alone, without encoding the original file.
    pub bytes: Cow<'static, [u8]>,
}
/// An editable WEM document with explicitly retained unknown payloads.
#[derive(Clone, Debug)]
pub struct Asset {
    /// Editable native WEM document.
    pub document: Document<'static, V3>,
    /// Inactive unknown payloads; never inserted into rebuilt native output.
    pub opaque: Vec<OpaquePayload>,
}
impl Asset {
    /// Creates an asset without retained unknown payloads.
    pub fn new(document: Document<'static, V3>) -> Self {
        Self {
            document,
            opaque: Vec::new(),
        }
    }
    /// Reads the selected semantic model under the requested or declared profile.
    pub fn model(&self, options: &ExportOptions) -> Result<wem::semantic::Model<'_>, Error> {
        let root = self
            .document
            .root::<wem::records::DocumentRecord<wem::records::V1>>()?;
        Ok(wem::semantic::read_with_look(
            &self.document,
            options.model,
            options.profile.unwrap_or(root.default_profile),
            options.look,
        )?)
    }
}
