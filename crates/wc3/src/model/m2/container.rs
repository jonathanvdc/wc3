//! Typed flat and chunked model documents.
use super::chunks::{Chunk, ChunkContext};
use super::codec::{Decoder, Encoder};
use super::model::Core;
use super::{Chunked, Container, Dialect, Flat, ModelLayout, Read, ReadError, Write, WriteError};
use crate::model::common::{binary::Chunks, CommonPartialEq, Ownership};
use std::fmt::Debug;
use std::iter::once;
use std::marker::PhantomData;

/// A chunk in a model document, retaining its original position.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum ModelChunk<'data, V: ModelLayout> {
    /// Complete typed MD20 core stored inside MD21 framing.
    Core(Box<Core<'data, V>>),
    /// Native extension or preserved unknown chunk.
    Extension(Chunk<'data, V>),
}
/// Native storage for a chunk stream, retaining all core and extension positions.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct ChunkStream<'data, V: ModelLayout> {
    chunks: Vec<ModelChunk<'data, V>>,
}
/// Codec and access operations provided by an envelope's native storage.
pub trait DocumentStorage<'data, V: ModelLayout>: Clone + Debug + PartialEq {
    /// Creates storage containing an empty core.
    fn empty() -> Self;
    /// Returns the primary core.
    fn core(&self) -> &Core<'data, V>;
    /// Returns mutable primary core.
    fn core_mut(&mut self) -> &mut Core<'data, V>;
    /// Enumerates all native core records.
    fn cores<'view>(&'view self) -> impl Iterator<Item = &'view Core<'data, V>>
    where
        'data: 'view;
    /// Returns a chunk slice where framing is present.
    fn chunks(&self) -> &[ModelChunk<'data, V>];
    /// Returns a mutable chunk slice where framing is present.
    fn chunks_mut(&mut self) -> &mut [ModelChunk<'data, V>];
    /// Decodes the envelope and its typed core.
    fn decode(data: &'data [u8]) -> Result<Self, ReadError>;
    /// Encodes native framing and typed core records.
    fn encode(&self) -> Result<Vec<u8>, WriteError>;
    /// Reports whether this native storage contains chunk framing.
    fn is_chunked(&self) -> bool;
}
impl<'data, V: ModelLayout> DocumentStorage<'data, V> for Core<'data, V> {
    fn empty() -> Self {
        Self::new()
    }
    fn core(&self) -> &Self {
        self
    }
    fn core_mut(&mut self) -> &mut Self {
        self
    }
    fn cores<'view>(&'view self) -> impl Iterator<Item = &'view Self>
    where
        'data: 'view,
    {
        once(self)
    }
    fn chunks(&self) -> &[ModelChunk<'data, V>] {
        &[]
    }
    fn chunks_mut(&mut self) -> &mut [ModelChunk<'data, V>] {
        &mut []
    }
    fn decode(data: &'data [u8]) -> Result<Self, ReadError> {
        Self::decode(data, 0)
    }
    fn encode(&self) -> Result<Vec<u8>, WriteError> {
        Self::encode(self)
    }
    fn is_chunked(&self) -> bool {
        false
    }
}
impl<'data, V: ModelLayout> DocumentStorage<'data, V> for ChunkStream<'data, V> {
    fn empty() -> Self {
        Self {
            chunks: vec![ModelChunk::Core(Box::new(Core::new()))],
        }
    }
    fn core(&self) -> &Core<'data, V> {
        self.cores().next().expect("chunk stream has a core")
    }
    fn core_mut(&mut self) -> &mut Core<'data, V> {
        self.chunks
            .iter_mut()
            .find_map(|chunk| match chunk {
                ModelChunk::Core(core) => Some(core.as_mut()),
                _ => None,
            })
            .expect("chunk stream has a core")
    }
    fn cores<'view>(&'view self) -> impl Iterator<Item = &'view Core<'data, V>>
    where
        'data: 'view,
    {
        self.chunks.iter().filter_map(|chunk| match chunk {
            ModelChunk::Core(core) => Some(core.as_ref()),
            _ => None,
        })
    }
    fn chunks(&self) -> &[ModelChunk<'data, V>] {
        &self.chunks
    }
    fn chunks_mut(&mut self) -> &mut [ModelChunk<'data, V>] {
        &mut self.chunks
    }
    fn is_chunked(&self) -> bool {
        true
    }
    fn decode(data: &'data [u8]) -> Result<Self, ReadError> {
        let raw = Chunks::new(data, 0)
            .map(|chunk| {
                let chunk =
                    chunk.map_err(|error| ReadError::invalid(error.offset, "chunk bounds"))?;
                Ok((chunk.header.tag, chunk.data, chunk.offset))
            })
            .collect::<Result<Vec<_>, ReadError>>()?;
        let primary = raw
            .iter()
            .find(|(tag, _, _)| tag == b"MD21")
            .ok_or_else(|| ReadError::invalid(0, "missing MD21 core"))?;
        let core = Core::decode(primary.1, primary.2)?;
        let external: Vec<bool> = core
            .sequences
            .iter()
            .map(|sequence| sequence.flags & 0x130 == 0)
            .collect();
        let flags = core.flags;
        let core_light_count = core.lights.len();
        let mut primary_core = Some(core);
        let mut chunks = Vec::new();
        for (tag, payload, origin) in raw {
            if tag == *b"MD21" {
                let core = match primary_core.take() {
                    Some(core) => core,
                    None => Core::decode(payload, origin)?,
                };
                chunks.push(ModelChunk::Core(Box::new(core)));
            } else {
                let context =
                    ChunkContext::<V>::new(flags, Some(core_light_count), external.clone().into());
                let chunk = Chunk::read(tag, payload, &context).map_err(|mut error| {
                    error.offset = error.offset.saturating_add(origin);
                    error
                })?;
                chunks.push(ModelChunk::Extension(chunk));
            }
        }
        Ok(Self { chunks })
    }
    /// Rebuilds the document and relocates all locally stored payloads.
    fn encode(&self) -> Result<Vec<u8>, WriteError> {
        let mut output = Encoder::new(V::VERSION, self.core().flags);
        for chunk in &self.chunks {
            let (tag, bytes) = match chunk {
                ModelChunk::Core(core) => (*b"MD21", core.encode()?),
                ModelChunk::Extension(extension) => {
                    (extension.tag(), extension.payload(self.core().flags)?)
                }
            };
            output.bytes(&tag)?;
            output
                .u32(u32::try_from(bytes.len()).map_err(|_| WriteError::invalid("chunk size"))?)?;
            output.bytes(&bytes)?;
        }
        Ok(output.finish())
    }
}
/// An editable document whose layout and native envelope storage are static.
///
/// Flat models expose only their core; extension insertion requires native chunk
/// storage, obtained with `into_chunked()`.
///
/// ```compile_fail
/// use wc3::model::m2::{Flat,Model,V264,chunks::Chunk};
/// let mut model = Model::<Flat<V264>>::new();
/// model.insert_extension(0,Chunk::PhysicsId(42));
/// ```
#[derive(Clone, Debug, CommonPartialEq, Ownership)]
pub struct Model<'data, D: Dialect> {
    storage: <D::Container as Container>::Storage<'data, D::Layout>,
    dialect: PhantomData<D>,
}
impl<'data, D: Dialect> Default for Model<'data, D> {
    fn default() -> Self {
        Self::new()
    }
}
impl<'data, D: Dialect> Model<'data, D> {
    /// Creates an empty document with the selected native layout.
    pub fn new() -> Self {
        Self {
            storage: DocumentStorage::empty(),
            dialect: PhantomData,
        }
    }
    /// Returns the exact native version number.
    pub fn version(&self) -> u32 {
        D::Layout::VERSION
    }
    /// Reports whether the native envelope contains chunk framing.
    pub fn is_chunked(&self) -> bool {
        self.storage.is_chunked()
    }
    /// Returns primary typed core records.
    pub fn core(&self) -> &Core<'data, D::Layout> {
        self.storage.core()
    }
    /// Returns mutable primary typed core records.
    pub fn core_mut(&mut self) -> &mut Core<'data, D::Layout> {
        self.storage.core_mut()
    }
    pub(crate) fn cores(&self) -> impl Iterator<Item = &Core<'data, D::Layout>> {
        self.storage.cores()
    }
    /// Iterates extension payloads in their native order.
    pub fn extensions(&self) -> impl Iterator<Item = &Chunk<'data, D::Layout>> {
        self.storage
            .chunks()
            .iter()
            .filter_map(|chunk| match chunk {
                ModelChunk::Extension(value) => Some(value),
                _ => None,
            })
    }
    /// Iterates mutable extension payloads in their native order.
    pub fn extensions_mut(&mut self) -> impl Iterator<Item = &mut Chunk<'data, D::Layout>> {
        self.storage
            .chunks_mut()
            .iter_mut()
            .filter_map(|chunk| match chunk {
                ModelChunk::Extension(value) => Some(value),
                _ => None,
            })
    }
    /// Decodes the selected native envelope and layout.
    pub fn decode_m2(data: &'data [u8]) -> Result<Self, ReadError> {
        Ok(Self {
            storage: DocumentStorage::decode(data)?,
            dialect: PhantomData,
        })
    }
    /// Encodes the selected native envelope and layout.
    pub fn encode_m2(&self) -> Result<Vec<u8>, WriteError> {
        self.storage.encode()
    }
}
impl<'data, D: Dialect> Read<'data> for Model<'data, D> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let base = input.absolute_position();
        let remaining = input.data().len() - input.position();
        Self::decode_m2(input.bytes(remaining)?).map_err(|mut error| {
            error.offset = base.saturating_add(error.offset);
            error
        })
    }
}
impl<D: Dialect> Write for Model<'_, D> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.encode_m2()?)
    }
}
impl<'data, V: ModelLayout> Model<'data, Chunked<V>> {
    /// Returns every native chunk in order.
    pub fn chunks(&self) -> &[ModelChunk<'data, V>] {
        &self.storage.chunks
    }
    /// Inserts a typed extension at a native chunk position.
    pub fn insert_extension(
        &mut self,
        index: usize,
        extension: Chunk<'data, V>,
    ) -> Result<(), WriteError> {
        if extension.tag() == *b"MD21" {
            return Err(WriteError::invalid("MD21 requires typed core storage"));
        }
        if index > self.storage.chunks.len() {
            return Err(WriteError::invalid("chunk insertion position"));
        }
        self.storage
            .chunks
            .insert(index, ModelChunk::Extension(extension));
        Ok(())
    }
    /// Removes all extensions with the supplied tag, retaining core payloads.
    pub fn remove_extensions(&mut self, tag: [u8; 4]) {
        self.storage
            .chunks
            .retain(|chunk| !matches!(chunk,ModelChunk::Extension(value) if value.tag()==tag));
    }
    /// Removes framing when no additional chunks would be lost.
    pub fn into_flat(mut self) -> Result<Model<'data, Flat<V>>, WriteError> {
        if self.storage.chunks.len() != 1 {
            return Err(WriteError::invalid("additional chunks in flat envelope"));
        }
        let ModelChunk::Core(core) = self.storage.chunks.pop().unwrap() else {
            unreachable!("chunk stream has a core")
        };
        Ok(Model {
            storage: *core,
            dialect: PhantomData,
        })
    }
}
impl<'data, V: ModelLayout> Model<'data, Flat<V>> {
    /// Adds native chunk framing without converting core records.
    pub fn into_chunked(self) -> Model<'data, Chunked<V>> {
        Model {
            storage: ChunkStream {
                chunks: vec![ModelChunk::Core(Box::new(self.storage))],
            },
            dialect: PhantomData,
        }
    }
}
