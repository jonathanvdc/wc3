//! Typed and runtime Warcraft III models with MDX and MDL I/O.
use crate::model::mdx;
use crate::model::mdx::{Read as _, Write as _};
use crate::model::scene::NodeRef;
use crate::model::Cursor;
use crate::model::Encoder;
use crate::model::ModelOwnership;
use crate::model::ValueError;
use crate::model::{Chunk, CollectionChunk, ModelChunk, VersionChunk};
use crate::model::{Extended, ModelDialect, NoExtensions};
use crate::model::{
    ModelVersion, Tag, Version, V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900,
};
use std::fmt::Debug;

/// The four bytes at the start of an MDX file.
pub const MAGIC: Tag = *b"MDLX";

/// A model with a statically selected dialect.
///
/// `D` selects a standard format version and application extension codec. Standard
/// markers such as `V800` are dialects with no extensions. Records use `D::Version`,
/// so extended models accept the same records as their base format. Import the
/// [`mdx`] or [`crate::model::mdl`] codec traits for whole-file reading and writing.
/// MDX retains chunk organization; MDL produces canonical text and rejects
/// data without a faithful text representation.
///
/// Getters for borrowed record collections return iterators without allocating. Use
/// `.cloned().collect::<Vec<_>>()` to clone records, preserving each buffer's
/// ownership state, or [`Model::records_mut`] to edit records in place.
///
/// ```compile_fail
/// use wc3::model::{Model, V800, V1100};
/// use wc3::model::materials::Material;
/// let mut model = Model::<V800>::new();
/// model.set_materials([Material::<V1100>::new()]);
/// ```
#[derive(Clone, Debug, ModelOwnership)]
pub struct Model<'data, D: ModelDialect> {
    /// The ordered list of chunks in the model.
    pub chunks: Vec<ModelChunk<'data, D>>,
}

impl<'data, D: ModelDialect> Default for Model<'data, D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'data, D: ModelDialect> Model<'data, D> {
    pub(crate) fn check_chunk_version(&self, tag: Tag) -> Result<(), ValueError> {
        (D::Version::NUMBER >= 900)
            .then_some(())
            .ok_or(ValueError::UnsupportedVersion {
                tag,
                minimum: 900,
                actual: D::Version::NUMBER,
            })
    }

    pub(crate) fn decoded_chunks<'a, C: 'a>(
        &'a self,
    ) -> impl Iterator<Item = &'a C> + use<'data, 'a, C, D>
    where
        for<'b> &'b C: TryFrom<&'b ModelChunk<'data, D>>,
    {
        self.chunks
            .iter()
            .filter_map(|chunk| <&C>::try_from(chunk).ok())
    }

    /// Borrows all records of a chunk type, preserving duplicate chunks.
    pub fn records<'s, C: CollectionChunk + 's>(
        &'s self,
    ) -> impl Iterator<Item = &'s C::Item> + use<'data, 's, C, D>
    where
        for<'a> &'a C: TryFrom<&'a ModelChunk<'data, D>>,
    {
        self.decoded_chunks::<C>().flat_map(|chunk| chunk.records())
    }

    /// Mutably borrows records in file order without replacing their chunks.
    pub fn records_mut<'s, C: CollectionChunk + 's>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut C::Item> + use<'data, 's, C, D>
    where
        for<'a> &'a mut C: TryFrom<&'a mut ModelChunk<'data, D>>,
    {
        self.chunks
            .iter_mut()
            .filter_map(|chunk| <&mut C>::try_from(chunk).ok())
            .flat_map(|chunk| chunk.records_mut())
    }

    /// Moves records into one replacement chunk, removing duplicate chunks of that kind.
    pub fn replace_records<C>(&mut self, records: Vec<C::Item>)
    where
        C: CollectionChunk + Into<ModelChunk<'data, D>>,
    {
        self.replace_chunk(C::from_records(records));
    }

    /// Creates a model with a `VERS` chunk for this type's version.
    pub fn new() -> Self {
        Self {
            chunks: vec![ModelChunk::from(VersionChunk::<D::Version>::new())],
        }
    }

    /// Returns the version selected by the model type.
    pub fn version(&self) -> Version {
        D::Version::NUMBER
    }

    /// Borrows every transform node in chunk and record order.
    ///
    /// Emitter-specific flags are reinterpreted as ordinary node flags without
    /// changing their stored bits. Unknown chunks do not contribute nodes.
    pub fn nodes(&self) -> Vec<NodeRef<'_>> {
        let mut nodes = Vec::new();
        for chunk in &self.chunks {
            match chunk {
                ModelChunk::Bones(chunk) => {
                    nodes.extend(chunk.records.iter().map(|record| record.node.as_ref()))
                }
                ModelChunk::Helpers(chunk) => {
                    nodes.extend(chunk.records.iter().map(|node| node.as_ref()))
                }
                ModelChunk::Attachments(chunk) => {
                    nodes.extend(chunk.records.iter().map(|record| record.node.as_ref()))
                }
                ModelChunk::Lights(chunk) => {
                    nodes.extend(chunk.records.iter().map(|record| record.node.as_ref()))
                }
                ModelChunk::EventObjects(chunk) => {
                    nodes.extend(chunk.records.iter().map(|record| record.node.as_ref()))
                }
                ModelChunk::CollisionShapes(chunk) => {
                    nodes.extend(chunk.records.iter().map(|record| record.node.as_ref()))
                }
                ModelChunk::RibbonEmitters(chunk) => {
                    nodes.extend(chunk.records.iter().map(|record| record.node.as_ref()))
                }
                ModelChunk::ParticleEmitters(chunk) => {
                    nodes.extend(chunk.records.iter().map(|record| record.node.as_ref()))
                }
                ModelChunk::ParticleEmitters2(chunk) => {
                    nodes.extend(chunk.records.iter().map(|record| record.node.as_ref()))
                }
                ModelChunk::PopcornEmitters(chunk) => {
                    nodes.extend(chunk.records.iter().map(|record| record.node.as_ref()))
                }
                _ => {}
            }
        }
        nodes
    }

    /// Finds the first chunk with the given tag.
    pub fn chunk(&self, tag: Tag) -> Option<&ModelChunk<'data, D>>
    where
        D::Extension: Chunk,
    {
        self.chunks.iter().find(|chunk| chunk.tag() == tag)
    }

    /// Finds the first mutable chunk with the given tag.
    pub fn chunk_mut(&mut self, tag: Tag) -> Option<&mut ModelChunk<'data, D>>
    where
        D::Extension: Chunk,
    {
        self.chunks.iter_mut().find(|chunk| chunk.tag() == tag)
    }

    pub(crate) fn replace_chunk(&mut self, chunk: impl Into<ModelChunk<'data, D>>) {
        let chunk = chunk.into();
        let tag = chunk
            .standard_tag()
            .expect("replacement is a standard chunk");
        if let Some(index) = self
            .chunks
            .iter()
            .position(|existing| existing.standard_tag() == Some(tag))
        {
            self.chunks[index] = chunk;
            let mut seen = false;
            self.chunks.retain(|existing| {
                if existing.standard_tag() != Some(tag) {
                    return true;
                }
                if seen {
                    false
                } else {
                    seen = true;
                    true
                }
            });
        } else {
            self.chunks.push(chunk);
        }
    }
}

#[cfg(test)]
mod node_tests {
    use super::*;
    use crate::model::scene::{Bone, Node};
    use crate::model::{BonesChunk, HelpersChunk};

    #[test]
    fn nodes_follow_chunk_and_record_order_including_duplicate_chunks() {
        let first = Node::new("first", 4).unwrap();
        let second = Node::new("second", 2).unwrap();
        let third = Node::new("third", 9).unwrap();
        let model = Model::<V800> {
            chunks: vec![
                HelpersChunk::new(vec![first, second]).into(),
                BonesChunk::new(vec![Bone::new(third, u32::MAX, u32::MAX)]).into(),
                HelpersChunk::new(vec![Node::new("fourth", 1).unwrap()]).into(),
            ],
        };
        let ids: Vec<_> = model.nodes().iter().map(|node| node.object_id).collect();
        assert_eq!(ids, [4, 2, 9, 1]);
    }
}

impl<'data, D: ModelDialect> mdx::Read<'data> for Model<'data, D>
where
    D::Extension: mdx::Extension<'data>,
{
    fn read_mdx(cursor: &mut Cursor<'data>) -> Result<Self, mdx::ReadError> {
        if let Some((actual, offset)) = scan_version(*cursor)? {
            if actual != D::Version::NUMBER {
                return Err(mdx::ReadError::new(
                    offset,
                    mdx::ReadErrorKind::VersionMismatch {
                        expected: D::Version::NUMBER,
                        actual,
                    },
                )
                .with_tag(*b"VERS"));
            }
        }

        let mut parse = *cursor;
        parse.read_bytes(4)?;
        let mut chunks = Vec::new();
        while !parse.remaining().is_empty() {
            let (tag, _, mut payload) = read_chunk(&mut parse)?;
            chunks.push(
                ModelChunk::decode_from(tag, &mut payload).map_err(|error| error.in_chunk(tag))?,
            );
        }
        *cursor = parse;
        Ok(Self { chunks })
    }
}

impl<'data, D: ModelDialect> mdx::Write for Model<'data, D>
where
    D::Extension: Chunk,
{
    fn write_mdx(&self, output: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        output.write_bytes(&MAGIC);
        for chunk in &self.chunks {
            output.write(chunk)?;
        }
        Ok(())
    }
}

/// A decoded model whose version is determined by its `VERS` chunk or MDL
/// Version block. `E` selects the application extension codec while the standard
/// record version is selected at runtime. Variants contain `Model<Extended<V, E>>`.
/// `mdl::Read::decode_mdl` requires an explicit FormatVersion.
///
/// The default extension type is [`NoExtensions`]. For standalone decoding calls,
/// supply a result type (`let model: DynamicModel = ...`) or an explicit extension
/// argument (`DynamicModel::<NoExtensions>::decode_mdx(...)`).
#[derive(Clone, Debug, ModelOwnership)]
pub enum DynamicModel<'data, E: Clone + Debug = NoExtensions> {
    /// A model using format version 800.
    V800(Model<'data, Extended<V800, E>>),
    /// A model using format version 900.
    V900(Model<'data, Extended<V900, E>>),
    /// A model using format version 1000.
    V1000(Model<'data, Extended<V1000, E>>),
    /// A model using format version 1100.
    V1100(Model<'data, Extended<V1100, E>>),
    /// A model using format version 1200.
    V1200(Model<'data, Extended<V1200, E>>),
    /// A model using format version 1300.
    V1300(Model<'data, Extended<V1300, E>>),
    /// A model using format version 1400.
    V1400(Model<'data, Extended<V1400, E>>),
    /// A model using format version 1600.
    V1600(Model<'data, Extended<V1600, E>>),
    /// A model using format version 1800.
    V1800(Model<'data, Extended<V1800, E>>),
}

/// Runs an expression against the typed model inside an [`DynamicModel`].
///
/// The expression is compiled for each supported version and must return the
/// same type for every version. Matching a reference allows read-only access;
/// matching a mutable reference allows edits.
///
/// ```
/// use wc3::model::{visit_model, DynamicModel, Model};
/// let model: DynamicModel = DynamicModel::V800(Model::new());
/// let count = visit_model!(&model, |typed| typed.geosets().count());
/// assert_eq!(count, 0);
/// ```
#[macro_export]
macro_rules! visit_model {
    ($value:expr, |$model:ident| $body:expr) => {
        match $value {
            $crate::model::DynamicModel::V800($model) => $body,
            $crate::model::DynamicModel::V900($model) => $body,
            $crate::model::DynamicModel::V1000($model) => $body,
            $crate::model::DynamicModel::V1100($model) => $body,
            $crate::model::DynamicModel::V1200($model) => $body,
            $crate::model::DynamicModel::V1300($model) => $body,
            $crate::model::DynamicModel::V1400($model) => $body,
            $crate::model::DynamicModel::V1600($model) => $body,
            $crate::model::DynamicModel::V1800($model) => $body,
        }
    };
}

impl<'data, E: Clone + Debug + mdx::Extension<'data>> DynamicModel<'data, E> {
    /// Decodes a model, using `default_version` when no `VERS` chunk is present.
    pub fn decode_mdx(
        bytes: &'data [u8],
        default_version: Version,
    ) -> Result<Self, mdx::ReadError> {
        let declared_version = scan_version(Cursor::new(bytes))?;
        let (version, offset) = declared_version.unwrap_or((default_version, 0));
        match version {
            800 => Model::<Extended<V800, E>>::decode_mdx(bytes).map(Self::V800),
            900 => Model::<Extended<V900, E>>::decode_mdx(bytes).map(Self::V900),
            1000 => Model::<Extended<V1000, E>>::decode_mdx(bytes).map(Self::V1000),
            1100 => Model::<Extended<V1100, E>>::decode_mdx(bytes).map(Self::V1100),
            1200 => Model::<Extended<V1200, E>>::decode_mdx(bytes).map(Self::V1200),
            1300 => Model::<Extended<V1300, E>>::decode_mdx(bytes).map(Self::V1300),
            1400 => Model::<Extended<V1400, E>>::decode_mdx(bytes).map(Self::V1400),
            1600 => Model::<Extended<V1600, E>>::decode_mdx(bytes).map(Self::V1600),
            1800 => Model::<Extended<V1800, E>>::decode_mdx(bytes).map(Self::V1800),
            _ => Err(mdx::ReadError {
                offset,
                tag: declared_version.map(|_| *b"VERS"),
                kind: mdx::ReadErrorKind::UnsupportedVersion { version },
            }),
        }
    }
}

impl<'data, E: Clone + Debug> DynamicModel<'data, E> {
    /// Returns the format version of the contained model.
    pub fn version(&self) -> Version {
        visit_model!(self, |model| model.version())
    }

    /// Encodes the contained model as a complete MDX file.
    pub fn encode_mdx(&self) -> Result<Vec<u8>, mdx::WriteError>
    where
        E: Chunk,
    {
        visit_model!(self, |model| model.encode_mdx())
    }
}

impl<'data, E: Clone + Debug + Chunk> mdx::Write for DynamicModel<'data, E> {
    fn write_mdx(&self, encoder: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        visit_model!(self, |model| model.write_mdx(encoder))
    }
}

fn scan_version<'data>(
    mut cursor: Cursor<'data>,
) -> Result<Option<(Version, usize)>, mdx::ReadError> {
    let offset = cursor.absolute_position();
    if cursor.read_bytes(4).ok() != Some(MAGIC.as_slice()) {
        return Err(mdx::ReadError::new(
            offset,
            mdx::ReadErrorKind::InvalidMagic,
        ));
    }
    let mut version = None;
    while !cursor.remaining().is_empty() {
        let (tag, _, mut payload) = read_chunk(&mut cursor)?;
        if tag == *b"VERS" {
            let offset = payload.absolute_position();
            let found = payload.read().map_err(|error| error.with_tag(tag))?;
            version.get_or_insert((found, offset));
        }
    }
    Ok(version)
}

fn read_chunk<'a>(cursor: &mut Cursor<'a>) -> Result<(Tag, u32, Cursor<'a>), mdx::ReadError> {
    let offset = cursor.absolute_position();
    if cursor.remaining().len() < 8 {
        return Err(mdx::ReadError::new(
            offset,
            mdx::ReadErrorKind::UnexpectedEnd {
                needed: 8,
                remaining: cursor.remaining().len(),
            },
        ));
    }
    let tag = cursor.read_bytes(4)?.try_into().expect("four-byte tag");
    let size = cursor.read()?;
    let payload = cursor
        .subcursor(size as usize)
        .map_err(|error| error.with_tag(tag))?;
    Ok((tag, size, payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{RawChunk, UnknownChunk};

    #[test]
    fn rejects_bad_magic_and_lengths() {
        assert!(matches!(
            Model::<V800>::decode_mdx(b"wrong"),
            Err(mdx::ReadError {
                offset: 0,
                tag: None,
                kind: mdx::ReadErrorKind::InvalidMagic
            })
        ));
        assert!(matches!(
            Model::<V800>::decode_mdx(b"MDLXVE"),
            Err(mdx::ReadError {
                offset: 4,
                tag: None,
                kind: mdx::ReadErrorKind::UnexpectedEnd {
                    needed: 8,
                    remaining: 2
                }
            })
        ));
        let mut bytes = b"MDLXTEST".to_vec();
        bytes.extend_from_slice(&5u32.to_le_bytes());
        bytes.push(1);
        assert!(matches!(
            Model::<V800>::decode_mdx(&bytes),
            Err(mdx::ReadError { offset: 12, tag: Some(tag), kind: mdx::ReadErrorKind::UnexpectedEnd { needed: 5, remaining: 1 } }) if tag == *b"TEST"
        ));
    }

    #[test]
    fn unknown_chunks_round_trip_and_bad_known_chunks_fail() {
        let mut model = Model::<V800>::new();
        model.chunks.push(ModelChunk::Unknown(
            UnknownChunk::new(RawChunk::new(*b"FUTR", vec![1, 2])).unwrap(),
        ));
        let bytes = model.encode_mdx().unwrap();
        let decoded = Model::<V800>::decode_mdx(&bytes).unwrap();
        assert_eq!(decoded.encode_mdx().unwrap(), bytes);

        let mut malformed = Model::<V800>::new().encode_mdx().unwrap();
        malformed.extend_from_slice(b"TEXS");
        malformed.extend_from_slice(&267u32.to_le_bytes());
        malformed.extend_from_slice(&[0; 267]);
        assert!(Model::<V800>::decode_mdx(&malformed).is_err());
    }

    #[test]
    fn runtime_dispatch_preserves_the_typed_version() {
        let bytes = Model::<V1100>::new().encode_mdx().unwrap();
        assert!(matches!(
            DynamicModel::<NoExtensions>::decode_mdx(&bytes, 800),
            Ok(DynamicModel::V1100(_))
        ));
        assert!(matches!(
            Model::<V800>::decode_mdx(&bytes),
            Err(mdx::ReadError {
                kind: mdx::ReadErrorKind::VersionMismatch { .. },
                ..
            })
        ));
    }
}
