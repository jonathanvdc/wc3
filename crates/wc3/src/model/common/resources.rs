//! File identities shared by model bundles and caller-provided resolvers.

/// A native or caller-supplied document identity.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum FileKey {
    /// The caller's root document, before a filename or archive ID is assigned.
    Root,
    /// A filename whose bytes are valid UTF-8; separators remain unmodified.
    Path(String),
    /// A nonzero game file-data ID, usually resolved through a CASC provider.
    FileId(u32),
    /// A generated name relative to an owner whose concrete filename is unknown.
    Sibling {
        /// Identity of the document whose filename supplies the stem.
        owner: Box<FileKey>,
        /// Suffix appended to the owner's stem, including its extension.
        suffix: String,
    },
    /// A native filename that cannot be represented as UTF-8 without losing bytes.
    RawPath(Vec<u8>),
}
impl FileKey {
    /// Builds a companion filename from this identity's final filename stem.
    ///
    /// Concrete paths keep their directory and native separator spelling. Other
    /// identity forms produce a logical sibling for the caller's resolver.
    pub fn sibling(&self, suffix: impl Into<String>) -> Self {
        let suffix = suffix.into();
        match self {
            Self::Path(path) => {
                let separator = path.rfind(['/', '\\']).map_or(0, |n| n + 1);
                let end = path[separator..]
                    .rfind('.')
                    .map_or(path.len(), |n| separator + n);
                Self::Path(format!("{}{}", &path[..end], suffix))
            }
            Self::RawPath(path) => {
                let separator = path
                    .iter()
                    .rposition(|b| matches!(*b, b'/' | b'\\'))
                    .map_or(0, |n| n + 1);
                let end = path[separator..]
                    .iter()
                    .rposition(|b| *b == b'.')
                    .map_or(path.len(), |n| separator + n);
                let mut bytes = path[..end].to_vec();
                bytes.extend_from_slice(suffix.as_bytes());
                Self::RawPath(bytes)
            }
            _ => Self::Sibling {
                owner: Box::new(self.clone()),
                suffix,
            },
        }
    }
}
