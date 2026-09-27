//! A model in the Warcraft III MDX format.
use crate::Encoder;
use crate::{EncodeError, ValueError};
use crate::{
    ModelVersion, Tag, Version, V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900,
};

use crate::Cursor;
use crate::{CollectionChunk, DecodeError, ModelChunk, Readable, VersionChunk, Writable};

/// The four bytes at the start of an MDX file.
pub const MAGIC: Tag = *b"MDLX";

/// An ordered MDX model whose known chunks and versioned records use layout `V`.
///
/// ```compile_fail
/// use wc3_mdx::{Model, V800, V1100};
/// use wc3_mdx::materials::Material;
/// let mut model = Model::<V800>::new();
/// model.set_materials(&[Material::<V1100>::new()]);
/// ```
#[derive(Clone, Debug)]
pub struct Model<V: ModelVersion> {
    /// The ordered list of chunks in the model.
    chunks: Vec<ModelChunk<V>>,
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

    /// Returns the first `VERS` value, if present.
    pub fn stored_version(&self) -> Option<u32> {
        self.chunks.iter().find_map(|chunk| match chunk {
            ModelChunk::Version(_) => Some(V::NUMBER),
            _ => None,
        })
    }

    /// Returns the model version, producing the value from the `VERS` chunk
    /// if there is such a chunk, or the default version otherwise.
    pub fn version(&self) -> Version {
        V::NUMBER
    }

    /// Returns the ordered list of chunks.
    pub fn chunks(&self) -> &[ModelChunk<V>] {
        &self.chunks
    }

    /// Returns a mutable ordered list of chunks.
    pub fn chunks_mut(&mut self) -> &mut Vec<ModelChunk<V>> {
        &mut self.chunks
    }

    /// Finds the first chunk with the given tag.
    pub fn chunk(&self, tag: Tag) -> Option<&ModelChunk<V>> {
        self.chunks.iter().find(|chunk| chunk.tag() == tag)
    }

    /// Finds the first mutable chunk with the given tag.
    pub fn chunk_mut(&mut self, tag: Tag) -> Option<&mut ModelChunk<V>> {
        self.chunks.iter_mut().find(|chunk| chunk.tag() == tag)
    }

    /// Appends a chunk.
    pub fn push(&mut self, chunk: ModelChunk<V>) {
        self.chunks.push(chunk);
    }

    pub(crate) fn replace_chunk(&mut self, chunk: impl Into<ModelChunk<V>>) {
        let chunk = chunk.into();
        let tag = chunk.tag();
        if let Some(index) = self
            .chunks()
            .iter()
            .position(|existing| existing.tag() == tag)
        {
            self.chunks_mut()[index] = chunk;
            let mut seen = false;
            self.chunks_mut().retain(|existing| {
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
            self.push(chunk);
        }
    }
}

impl<V: ModelVersion> Readable for Model<V> {
    fn read_from(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        if let Some(actual) = scan_version(*cursor)? {
            if actual != V::NUMBER {
                return Err(DecodeError::VersionMismatch {
                    expected: V::NUMBER,
                    actual,
                });
            }
        }

        let mut parse = *cursor;
        parse.read_exact(4)?;
        let mut chunks = Vec::new();
        while !parse.remaining().is_empty() {
            let (tag, _, mut payload) = read_chunk(&mut parse)?;
            chunks.push(ModelChunk::decode_from(tag, &mut payload)?);
        }
        *cursor = parse;
        Ok(Self { chunks })
    }
}

impl<V: ModelVersion> Writable for Model<V> {
    fn write_to(&self, output: &mut Encoder<'_>) -> Result<(), EncodeError> {
        output.write_bytes(&MAGIC);
        for chunk in &self.chunks {
            output.write(chunk)?;
        }
        Ok(())
    }
}

/// A decoded model whose version is determined by its `VERS` chunk.
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
/// use wc3_mdx::{visit_model, DynamicModel, Model, V800};
/// let model = DynamicModel::V800(Model::<V800>::new());
/// let count = visit_model!(&model, |typed| typed.geosets().len());
/// assert_eq!(count, 0);
/// ```
#[macro_export]
macro_rules! visit_model {
    ($value:expr, |$model:ident| $body:expr) => {
        match $value {
            $crate::DynamicModel::V800($model) => $body,
            $crate::DynamicModel::V900($model) => $body,
            $crate::DynamicModel::V1000($model) => $body,
            $crate::DynamicModel::V1100($model) => $body,
            $crate::DynamicModel::V1200($model) => $body,
            $crate::DynamicModel::V1300($model) => $body,
            $crate::DynamicModel::V1400($model) => $body,
            $crate::DynamicModel::V1600($model) => $body,
            $crate::DynamicModel::V1800($model) => $body,
        }
    };
}

impl DynamicModel {
    /// Decodes a model, using `default_version` when no `VERS` chunk is present.
    pub fn decode(bytes: &[u8], default_version: Version) -> Result<Self, DecodeError> {
        let version = scan_version(Cursor::new(bytes))?.unwrap_or(default_version);
        match version {
            800 => Model::<V800>::decode(bytes).map(Self::V800),
            900 => Model::<V900>::decode(bytes).map(Self::V900),
            1000 => Model::<V1000>::decode(bytes).map(Self::V1000),
            1100 => Model::<V1100>::decode(bytes).map(Self::V1100),
            1200 => Model::<V1200>::decode(bytes).map(Self::V1200),
            1300 => Model::<V1300>::decode(bytes).map(Self::V1300),
            1400 => Model::<V1400>::decode(bytes).map(Self::V1400),
            1600 => Model::<V1600>::decode(bytes).map(Self::V1600),
            1800 => Model::<V1800>::decode(bytes).map(Self::V1800),
            _ => Err(DecodeError::UnsupportedVersion { version }),
        }
    }

    pub fn version(&self) -> Version {
        visit_model!(self, |model| model.version())
    }

    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        visit_model!(self, |model| model.encode())
    }
}

fn scan_version(mut cursor: Cursor<'_>) -> Result<Option<Version>, DecodeError> {
    if cursor.read_exact(4).ok() != Some(MAGIC.as_slice()) {
        return Err(DecodeError::InvalidMagic);
    }
    let mut version = None;
    while !cursor.remaining().is_empty() {
        let (tag, _, mut payload) = read_chunk(&mut cursor)?;
        if tag == *b"VERS" {
            let found = payload
                .read()
                .map_err(|_| DecodeError::InvalidVersionChunk)?;
            version.get_or_insert(found);
        }
    }
    Ok(version)
}

fn read_chunk<'a>(cursor: &mut Cursor<'a>) -> Result<(Tag, u32, Cursor<'a>), DecodeError> {
    let offset = cursor.absolute_position();
    if cursor.remaining().len() < 8 {
        return Err(DecodeError::TruncatedHeader { offset });
    }
    let tag = cursor.read_exact(4)?.try_into().expect("four-byte tag");
    let size = cursor.read()?;
    let payload = cursor
        .slice(size as usize)
        .map_err(|_| DecodeError::TruncatedChunk { tag, offset, size })?;
    Ok((tag, size, payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RawChunk, UnknownChunk};

    #[test]
    fn rejects_bad_magic_and_lengths() {
        assert!(matches!(
            Model::<V800>::decode(b"wrong"),
            Err(DecodeError::InvalidMagic)
        ));
        assert!(matches!(
            Model::<V800>::decode(b"MDLXVE"),
            Err(DecodeError::TruncatedHeader { offset: 4 })
        ));
        let mut bytes = b"MDLXTEST".to_vec();
        bytes.extend_from_slice(&5u32.to_le_bytes());
        bytes.push(1);
        assert!(matches!(
            Model::<V800>::decode(&bytes),
            Err(DecodeError::TruncatedChunk { tag, offset: 4, size: 5 }) if tag == *b"TEST"
        ));
    }

    #[test]
    fn unknown_chunks_round_trip_and_bad_known_chunks_fail() {
        let mut model = Model::<V800>::new();
        model.push(ModelChunk::Unknown(
            UnknownChunk::new(RawChunk::new(*b"FUTR", vec![1, 2])).unwrap(),
        ));
        let bytes = model.encode().unwrap();
        let decoded = Model::<V800>::decode(&bytes).unwrap();
        assert_eq!(decoded.encode().unwrap(), bytes);

        let mut malformed = Model::<V800>::new().encode().unwrap();
        malformed.extend_from_slice(b"TEXS");
        malformed.extend_from_slice(&267u32.to_le_bytes());
        malformed.extend_from_slice(&[0; 267]);
        assert!(Model::<V800>::decode(&malformed).is_err());
    }

    #[test]
    fn runtime_dispatch_preserves_the_typed_version() {
        let bytes = Model::<V1100>::new().encode().unwrap();
        assert!(matches!(
            DynamicModel::decode(&bytes, 800),
            Ok(DynamicModel::V1100(_))
        ));
        assert!(matches!(
            Model::<V800>::decode(&bytes),
            Err(DecodeError::VersionMismatch { .. })
        ));
    }
}
