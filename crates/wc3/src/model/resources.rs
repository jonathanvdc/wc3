//! Direct authored resource references and transactional edits.
//!
//! Enumeration preserves chunk and record order, repeated references, empty path
//! fields, and literal spelling. A bitmap with both a path and a nonzero
//! replaceable ID contributes two independent references. Internal indices such
//! as material texture IDs and PRE2 TextureID are not external resources. Unknown
//! chunks are retained but not searched for strings. File lookup, normalization,
//! recursive dependency discovery, and renderer support policy belong to callers.
//!
//! ```
//! use std::convert::Infallible;
//! use wc3::model::{Model, V800};
//! use wc3::model::materials::Texture;
//! use wc3::model::resources::{ResourceEdit, ResourceValue};
//!
//! let mut model = Model::<V800>::new();
//! model.set_textures(&[Texture::new("Textures\\Body.blp")?]);
//! let report = model.rewrite_resources(|reference| {
//!     let edit = match reference.value {
//!         ResourceValue::Path(path) => match path.text().strip_prefix("Textures\\") {
//!             Some(suffix) => ResourceEdit::SetPath(format!("Custom\\{suffix}")),
//!             None => ResourceEdit::Keep,
//!         },
//!         ResourceValue::ReplaceableId(_) => ResourceEdit::Keep,
//!     };
//!     Ok::<_, Infallible>(edit)
//! })?;
//! assert_eq!(report.changes.len(), 1);
//! assert_eq!(model.textures()[0].path.text(), "Custom\\Body.blp");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::model::chunks::ModelChunk;
use crate::model::{visit_model, DynamicModel, FixedText, Model, ModelVersion, ValueError};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::iter::FusedIterator;

/// A record containing an authored resource field. Indices are local to its chunk.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ResourceSite {
    /// The external animation filename in model metadata.
    ModelInfo,
    /// A bitmap record, including literal and replaceable bindings.
    Bitmap(usize),
    /// An attachment's child-model path.
    Attachment(usize),
    /// A Classic particle emitter's authored model or image path.
    ParticleEmitter(usize),
    /// A quad particle emitter's replaceable binding.
    ParticleEmitter2(usize),
    /// A PopcornFX emitter's effect path or replaceable binding.
    PopcornEmitter(usize),
    /// A FaceFX resource path.
    FaceFx(usize),
}

impl ResourceSite {
    fn record(self) -> usize {
        match self {
            Self::ModelInfo => 0,
            Self::Bitmap(index)
            | Self::Attachment(index)
            | Self::ParticleEmitter(index)
            | Self::ParticleEmitter2(index)
            | Self::PopcornEmitter(index)
            | Self::FaceFx(index) => index,
        }
    }
}

/// The independent authored field within a resource-bearing record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ResourceField {
    /// A fixed-width external filename or path, including empty paths.
    Path,
    /// A nonzero game-provided resource binding. Zero IDs are not enumerated.
    ReplaceableId,
}

/// Exact position of a resource field in the model's current chunk layout.
/// Structural edits to chunks or records invalidate these positions.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ResourceLocation {
    /// Index in the model's ordered chunk list, including duplicate chunks.
    pub chunk: usize,
    /// Record kind and chunk-local index.
    pub site: ResourceSite,
    /// Field within that record.
    pub field: ResourceField,
}

impl Display for ResourceLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "chunk[{}].", self.chunk)?;
        match self.site {
            ResourceSite::ModelInfo => return f.write_str("animation_file_name"),
            ResourceSite::Bitmap(index) => write!(f, "textures[{index}]")?,
            ResourceSite::Attachment(index) => write!(f, "attachments[{index}]")?,
            ResourceSite::ParticleEmitter(index) => write!(f, "particle_emitters[{index}]")?,
            ResourceSite::ParticleEmitter2(index) => write!(f, "particle_emitters2[{index}]")?,
            ResourceSite::PopcornEmitter(index) => write!(f, "popcorn_emitters[{index}]")?,
            ResourceSite::FaceFx(index) => write!(f, "face_fx[{index}]")?,
        }
        f.write_str(match self.field {
            ResourceField::Path => ".path",
            ResourceField::ReplaceableId => ".replaceable_id",
        })
    }
}

/// A borrowed literal field or a game-provided resource ID.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceValue<'a> {
    /// Exact source bytes, including nonstandard encoding and padding.
    Path(&'a FixedText<260>),
    /// A nonzero replaceable ID, independent of any literal path.
    ReplaceableId(u32),
}

/// One occurrence of an authored external resource reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceReference<'a> {
    /// Exact field location in the current model layout.
    pub location: ResourceLocation,
    /// Original value, without path normalization or file resolution.
    pub value: ResourceValue<'a>,
}

/// A proposed change to one enumerated field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResourceEdit {
    /// Retain every original byte of the field.
    Keep,
    /// Replace a path with UTF-8 text; an empty string clears it.
    SetPath(String),
    /// Replace an enumerated ID; zero clears the binding without changing its path.
    SetReplaceableId(u32),
}

/// Owned field values used by a rewrite report, retaining exact path bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OwnedResourceValue {
    /// Fixed-width path bytes before or after editing.
    Path(Box<FixedText<260>>),
    /// A replaceable ID, including zero after clearing a binding.
    ReplaceableId(u32),
}

impl Display for OwnedResourceValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(path) => Display::fmt(&path.text(), f),
            Self::ReplaceableId(id) => write!(f, "replaceable:{id}"),
        }
    }
}

/// One actual change, with lossless before and after values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceChange {
    /// Exact position of the changed field.
    pub location: ResourceLocation,
    /// Original field value.
    pub before: OwnedResourceValue,
    /// Validated replacement value.
    pub after: OwnedResourceValue,
}

/// Successful edits in source traversal order. No-op proposals are omitted.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RewriteReport {
    /// Applied field changes; empty when every field was kept or unchanged.
    pub changes: Vec<ResourceChange>,
}

/// A failed rewrite. No proposed changes are applied on any of these errors.
#[derive(Debug)]
pub enum RewriteError {
    /// The caller could not decide an edit; the original error is retained as its source.
    Callback {
        /// Field being processed when the callback failed.
        location: ResourceLocation,
        /// Original callback error, available for source traversal and downcasting.
        error: Box<dyn Error + Send + Sync>,
    },
    /// A replacement path contains NUL or exceeds the field's byte capacity.
    InvalidPath {
        /// Field whose proposed text was invalid.
        location: ResourceLocation,
        /// Fixed-text validation failure.
        source: ValueError,
    },
    /// A path edit was requested for an ID field, or vice versa.
    IncompatibleEdit {
        /// Field whose value kind did not match the edit.
        location: ResourceLocation,
        /// Rejected edit.
        edit: ResourceEdit,
    },
}

impl RewriteError {
    /// Returns the exact location of the failing field.
    pub fn location(&self) -> ResourceLocation {
        match self {
            Self::Callback { location, .. }
            | Self::InvalidPath { location, .. }
            | Self::IncompatibleEdit { location, .. } => *location,
        }
    }
}

impl Display for RewriteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Callback { location, error } => write!(f, "{location}: {error}"),
            Self::InvalidPath { location, source } => write!(f, "{location}: {source}"),
            Self::IncompatibleEdit { location, edit } => {
                let kind = match edit {
                    ResourceEdit::Keep => "Keep",
                    ResourceEdit::SetPath(_) => "SetPath",
                    ResourceEdit::SetReplaceableId(_) => "SetReplaceableId",
                };
                write!(
                    f,
                    "{location}: {kind} is incompatible with {:?}",
                    location.field
                )
            }
        }
    }
}

impl Error for RewriteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Callback { error, .. } => Some(error.as_ref()),
            Self::InvalidPath { source, .. } => Some(source),
            Self::IncompatibleEdit { .. } => None,
        }
    }
}

struct Fields<P, I> {
    site: ResourceSite,
    path: Option<P>,
    replaceable_id: Option<I>,
}

// One definition of resource-bearing records serves reading and validated writes.
macro_rules! resource_fields {
    ($chunk:expr, $record:expr, $get:ident $(, $mutable:tt)?) => {{
        let index = $record;
        macro_rules! entry {
            ($records:expr, $site:ident, path) => {{
                let record = $records.$get(index)?;
                Some(Fields {
                    site: ResourceSite::$site(index),
                    path: Some(&$($mutable)? record.path),
                    replaceable_id: None,
                })
            }};
            ($records:expr, $site:ident, id) => {{
                let record = $records.$get(index)?;
                Some(Fields {
                    site: ResourceSite::$site(index),
                    path: None,
                    replaceable_id: Some(&$($mutable)? record.replaceable_id),
                })
            }};
            ($records:expr, $site:ident, both) => {{
                let record = $records.$get(index)?;
                Some(Fields {
                    site: ResourceSite::$site(index),
                    path: Some(&$($mutable)? record.path),
                    replaceable_id: Some(&$($mutable)? record.replaceable_id),
                })
            }};
        }
        match $chunk {
            ModelChunk::ModelInfo(chunk) if index == 0 => Some(Fields {
                site: ResourceSite::ModelInfo,
                path: Some(&$($mutable)? chunk.info.animation_file_name),
                replaceable_id: None,
            }),
            ModelChunk::Textures(chunk) => entry!(chunk.records, Bitmap, both),
            ModelChunk::Attachments(chunk) => entry!(chunk.records, Attachment, path),
            ModelChunk::ParticleEmitters(chunk) => entry!(chunk.records, ParticleEmitter, path),
            ModelChunk::ParticleEmitters2(chunk) => entry!(chunk.records, ParticleEmitter2, id),
            ModelChunk::PopcornEmitters(chunk) => entry!(chunk.records, PopcornEmitter, both),
            ModelChunk::FaceFx(chunk) => entry!(chunk.records, FaceFx, path),
            _ => None,
        }
    }};
}

fn fields<V: ModelVersion>(
    chunk: &ModelChunk<V>,
    record: usize,
) -> Option<Fields<&FixedText<260>, &u32>> {
    resource_fields!(chunk, record, get)
}

fn fields_mut<V: ModelVersion>(
    chunk: &mut ModelChunk<V>,
    record: usize,
) -> Option<Fields<&mut FixedText<260>, &mut u32>> {
    resource_fields!(chunk, record, get_mut, mut)
}

struct Resources<'a, V: ModelVersion> {
    chunks: &'a [ModelChunk<V>],
    chunk: usize,
    record: usize,
    field: ResourceField,
}

impl<'a, V: ModelVersion> Iterator for Resources<'a, V> {
    type Item = ResourceReference<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let chunk = self.chunks.get(self.chunk)?;
            let Some(fields) = fields(chunk, self.record) else {
                self.chunk += 1;
                self.record = 0;
                self.field = ResourceField::Path;
                continue;
            };
            let location = ResourceLocation {
                chunk: self.chunk,
                site: fields.site,
                field: self.field,
            };
            let value = match self.field {
                ResourceField::Path => {
                    self.field = ResourceField::ReplaceableId;
                    fields.path.map(ResourceValue::Path)
                }
                ResourceField::ReplaceableId => {
                    self.record += 1;
                    self.field = ResourceField::Path;
                    fields
                        .replaceable_id
                        .filter(|&&id| id != 0)
                        .map(|&id| ResourceValue::ReplaceableId(id))
                }
            };
            if let Some(value) = value {
                return Some(ResourceReference { location, value });
            }
        }
    }
}

impl<V: ModelVersion> FusedIterator for Resources<'_, V> {}

impl<V: ModelVersion> Model<V> {
    /// Enumerate all direct authored references in chunk, record, and field order.
    /// Paths precede IDs within each record. Empty paths and duplicate occurrences
    /// remain; zero replaceable IDs and internal indices are excluded.
    pub fn resources(&self) -> impl Iterator<Item = ResourceReference<'_>> + '_ {
        Resources {
            chunks: &self.chunks,
            chunk: 0,
            record: 0,
            field: ResourceField::Path,
        }
    }

    /// Collect and validate edits before changing any field. Callback failures,
    /// invalid paths, and incompatible edits leave the entire model unchanged.
    /// Path capacity is 259 UTF-8 bytes plus NUL. Setting unchanged UTF-8 text
    /// retains original padding; untouched fields and opaque chunks retain every byte.
    /// Structural chunk/record organization and other fields are never altered.
    pub fn rewrite_resources<E>(
        &mut self,
        mut edit: impl FnMut(ResourceReference<'_>) -> Result<ResourceEdit, E>,
    ) -> Result<RewriteReport, RewriteError>
    where
        E: Into<Box<dyn Error + Send + Sync>>,
    {
        let mut report = RewriteReport::default();
        for reference in self.resources() {
            let location = reference.location;
            let proposed = edit(reference).map_err(|error| RewriteError::Callback {
                location,
                error: error.into(),
            })?;
            let (before, after) = match (reference.value, proposed) {
                (_, ResourceEdit::Keep) => continue,
                (ResourceValue::Path(old), ResourceEdit::SetPath(text)) => {
                    let mut new = FixedText::default();
                    new.set_text(&text)
                        .map_err(|source| RewriteError::InvalidPath { location, source })?;
                    let end = old
                        .as_bytes()
                        .iter()
                        .position(|&byte| byte == 0)
                        .unwrap_or(260);
                    if old.as_bytes()[..end] == *text.as_bytes() {
                        continue;
                    }
                    (
                        OwnedResourceValue::Path(Box::new(*old)),
                        OwnedResourceValue::Path(Box::new(new)),
                    )
                }
                (ResourceValue::ReplaceableId(old), ResourceEdit::SetReplaceableId(new)) => {
                    if old == new {
                        continue;
                    }
                    (
                        OwnedResourceValue::ReplaceableId(old),
                        OwnedResourceValue::ReplaceableId(new),
                    )
                }
                (_, edit) => return Err(RewriteError::IncompatibleEdit { location, edit }),
            };
            report.changes.push(ResourceChange {
                location,
                before,
                after,
            });
        }
        for change in &report.changes {
            let location = change.location;
            let fields = fields_mut(&mut self.chunks[location.chunk], location.site.record())
                .expect("validated resource location");
            match &change.after {
                OwnedResourceValue::Path(path) => {
                    *fields.path.expect("validated path field") = **path
                }
                OwnedResourceValue::ReplaceableId(id) => {
                    *fields.replaceable_id.expect("validated ID field") = *id
                }
            }
        }
        Ok(report)
    }
}

impl DynamicModel {
    /// Enumerate direct authored references with the same order and guarantees as
    /// [`Model::resources`], preserving the decoded source version and chunk layout.
    pub fn resources(&self) -> impl Iterator<Item = ResourceReference<'_>> + '_ {
        visit_model!(self, |model| Box::new(model.resources())
            as Box<dyn Iterator<Item = ResourceReference<'_>> + '_>)
    }

    /// Apply a transactional rewrite to the contained version-specific model.
    /// See [`Model::rewrite_resources`] for validation and byte-preservation rules.
    pub fn rewrite_resources<E>(
        &mut self,
        edit: impl FnMut(ResourceReference<'_>) -> Result<ResourceEdit, E>,
    ) -> Result<RewriteReport, RewriteError>
    where
        E: Into<Box<dyn Error + Send + Sync>>,
    {
        visit_model!(self, |model| model.rewrite_resources(edit))
    }
}
