//! Typed and runtime Warcraft III models with MDX and MDL I/O.
use crate::model::mdx;
use crate::model::mdx::{Read as _, Write as _};
use crate::model::scene::Node;
use crate::model::Cursor;
use crate::model::Encoder;
use crate::model::ValueError;
use crate::model::{CollectionChunk, ModelChunk, VersionChunk};
use crate::model::{
    ModelVersion, Tag, Version, V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900,
};

/// The four bytes at the start of an MDX file.
pub const MAGIC: Tag = *b"MDLX";

/// A model with a known format version.
///
/// `V` keeps version-dependent records compatible with the model. Import the
/// [`mdx`] or [`crate::model::mdl`] codec traits for whole-file reading and writing.
/// MDX retains chunk organization; MDL produces canonical text and rejects
/// data without a faithful text representation.
///
/// Collection getters return owned copies. Set edited copies back on the model,
/// or use [`Model::chunks`] to edit records in place.
///
/// ```compile_fail
/// use wc3::model::{Model, V800, V1100};
/// use wc3::model::materials::Material;
/// let mut model = Model::<V800>::new();
/// model.set_materials(&[Material::<V1100>::new()]);
/// ```
#[derive(Clone, Debug)]
pub struct Model<V: ModelVersion> {
    /// The ordered list of chunks in the model.
    pub chunks: Vec<ModelChunk<V>>,
}

impl<V: ModelVersion> Default for Model<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: ModelVersion> Model<V> {
    pub(crate) fn check_chunk_version(&self, tag: Tag) -> Result<(), ValueError> {
        (V::NUMBER >= 900)
            .then_some(())
            .ok_or(ValueError::UnsupportedVersion {
                tag,
                minimum: 900,
                actual: V::NUMBER,
            })
    }

    pub(crate) fn decoded_chunks<'a, C: 'a>(&'a self) -> impl Iterator<Item = &'a C>
    where
        for<'b> &'b C: TryFrom<&'b ModelChunk<V>>,
    {
        self.chunks
            .iter()
            .filter_map(|chunk| <&C>::try_from(chunk).ok())
    }

    pub(crate) fn collect_chunk_records<C: CollectionChunk>(&self) -> Vec<C::Item>
    where
        C::Item: Clone,
        for<'a> &'a C: TryFrom<&'a ModelChunk<V>>,
    {
        self.collect_chunk_items::<C, _>(|collection| collection.records())
    }

    pub(crate) fn collect_chunk_items<C, T: Clone>(&self, items: impl Fn(&C) -> &[T]) -> Vec<T>
    where
        for<'a> &'a C: TryFrom<&'a ModelChunk<V>>,
    {
        let mut result = Vec::new();
        for chunk in self.decoded_chunks::<C>() {
            result.extend_from_slice(items(chunk));
        }
        result
    }

    /// Creates a model with a `VERS` chunk for this type's version.
    pub fn new() -> Self {
        Self {
            chunks: vec![ModelChunk::from(VersionChunk::<V>::new())],
        }
    }

    /// Returns the version selected by the model type.
    pub fn version(&self) -> Version {
        V::NUMBER
    }

    /// Returns owned copies of every transform node in chunk and record order.
    ///
    /// Emitter-specific flags are reinterpreted as ordinary node flags without
    /// changing their stored bits. Unknown chunks do not contribute nodes.
    pub fn nodes(&self) -> Vec<Node> {
        let mut nodes = Vec::new();
        for chunk in &self.chunks {
            match chunk {
                ModelChunk::Bones(chunk) => {
                    nodes.extend(chunk.records.iter().map(|record| record.node.clone()))
                }
                ModelChunk::Helpers(chunk) => nodes.extend(chunk.records.iter().cloned()),
                ModelChunk::Attachments(chunk) => {
                    nodes.extend(chunk.records.iter().map(|record| record.node.clone()))
                }
                ModelChunk::Lights(chunk) => {
                    nodes.extend(chunk.records.iter().map(|record| record.node.clone()))
                }
                ModelChunk::EventObjects(chunk) => {
                    nodes.extend(chunk.records.iter().map(|record| record.node.clone()))
                }
                ModelChunk::CollisionShapes(chunk) => {
                    nodes.extend(chunk.records.iter().map(|record| record.node.clone()))
                }
                ModelChunk::RibbonEmitters(chunk) => {
                    nodes.extend(chunk.records.iter().map(|record| record.node.clone()))
                }
                ModelChunk::ParticleEmitters(chunk) => nodes.extend(
                    chunk
                        .records
                        .iter()
                        .map(|record| record.node.clone().cast_flags()),
                ),
                ModelChunk::ParticleEmitters2(chunk) => nodes.extend(
                    chunk
                        .records
                        .iter()
                        .map(|record| record.node.clone().cast_flags()),
                ),
                ModelChunk::PopcornEmitters(chunk) => nodes.extend(
                    chunk
                        .records
                        .iter()
                        .map(|record| record.node.clone().cast_flags()),
                ),
                _ => {}
            }
        }
        nodes
    }

    /// Finds the first chunk with the given tag.
    pub fn chunk(&self, tag: Tag) -> Option<&ModelChunk<V>> {
        self.chunks.iter().find(|chunk| chunk.tag() == tag)
    }

    /// Finds the first mutable chunk with the given tag.
    pub fn chunk_mut(&mut self, tag: Tag) -> Option<&mut ModelChunk<V>> {
        self.chunks.iter_mut().find(|chunk| chunk.tag() == tag)
    }

    pub(crate) fn replace_chunk(&mut self, chunk: impl Into<ModelChunk<V>>) {
        let chunk = chunk.into();
        let tag = chunk.tag();
        if let Some(index) = self
            .chunks
            .iter()
            .position(|existing| existing.tag() == tag)
        {
            self.chunks[index] = chunk;
            let mut seen = false;
            self.chunks.retain(|existing| {
                if existing.tag() != tag {
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
    use crate::model::scene::Bone;
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

impl<V: ModelVersion> mdx::Read for Model<V> {
    fn read_mdx(cursor: &mut Cursor<'_>) -> Result<Self, mdx::ReadError> {
        if let Some((actual, offset)) = scan_version(*cursor)? {
            if actual != V::NUMBER {
                return Err(mdx::ReadError::new(
                    offset,
                    mdx::ReadErrorKind::VersionMismatch {
                        expected: V::NUMBER,
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

impl<V: ModelVersion> mdx::Write for Model<V> {
    fn write_mdx(&self, output: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        output.write_bytes(&MAGIC);
        for chunk in &self.chunks {
            output.write(chunk)?;
        }
        Ok(())
    }
}

/// A decoded model whose version is determined by its `VERS` chunk or MDL
/// Version block. `mdl::Read::decode_mdl` requires an explicit FormatVersion.
#[derive(Clone, Debug)]
pub enum DynamicModel {
    V800(Model<V800>),
    V900(Model<V900>),
    V1000(Model<V1000>),
    V1100(Model<V1100>),
    V1200(Model<V1200>),
    V1300(Model<V1300>),
    V1400(Model<V1400>),
    V1600(Model<V1600>),
    V1800(Model<V1800>),
}

/// Runs an expression against the typed model inside an [`DynamicModel`].
///
/// The expression is compiled for each supported version and must return the
/// same type for every version. Matching a reference allows read-only access;
/// matching a mutable reference allows edits.
///
/// ```
/// use wc3::model::{visit_model, DynamicModel, Model, V800};
/// let model = DynamicModel::V800(Model::<V800>::new());
/// let count = visit_model!(&model, |typed| typed.geosets().len());
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

impl DynamicModel {
    /// Decodes a model, using `default_version` when no `VERS` chunk is present.
    pub fn decode_mdx(bytes: &[u8], default_version: Version) -> Result<Self, mdx::ReadError> {
        let declared_version = scan_version(Cursor::new(bytes))?;
        let (version, offset) = declared_version.unwrap_or((default_version, 0));
        match version {
            800 => Model::<V800>::decode_mdx(bytes).map(Self::V800),
            900 => Model::<V900>::decode_mdx(bytes).map(Self::V900),
            1000 => Model::<V1000>::decode_mdx(bytes).map(Self::V1000),
            1100 => Model::<V1100>::decode_mdx(bytes).map(Self::V1100),
            1200 => Model::<V1200>::decode_mdx(bytes).map(Self::V1200),
            1300 => Model::<V1300>::decode_mdx(bytes).map(Self::V1300),
            1400 => Model::<V1400>::decode_mdx(bytes).map(Self::V1400),
            1600 => Model::<V1600>::decode_mdx(bytes).map(Self::V1600),
            1800 => Model::<V1800>::decode_mdx(bytes).map(Self::V1800),
            _ => Err(mdx::ReadError {
                offset,
                tag: declared_version.map(|_| *b"VERS"),
                kind: mdx::ReadErrorKind::UnsupportedVersion { version },
            }),
        }
    }

    pub fn version(&self) -> Version {
        visit_model!(self, |model| model.version())
    }

    pub fn encode_mdx(&self) -> Result<Vec<u8>, mdx::WriteError> {
        visit_model!(self, |model| model.encode_mdx())
    }
}

impl mdx::Write for DynamicModel {
    fn write_mdx(&self, encoder: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        visit_model!(self, |model| model.write_mdx(encoder))
    }
}

fn scan_version(mut cursor: Cursor<'_>) -> Result<Option<(Version, usize)>, mdx::ReadError> {
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
            DynamicModel::decode_mdx(&bytes, 800),
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
