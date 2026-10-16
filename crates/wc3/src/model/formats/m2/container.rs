//! Typed flat and chunked model documents.
use super::bundle::visit_core;
use super::chunks::{Chunk, ChunkContext};
use super::codec::{Array, Decoder, Encoder, Value};
use super::model::Core;
use super::records::{ArrayVisitor, VisitArrays};
use super::{Chunked, Container, Dialect, Flat, ModelLayout, Read, ReadError, Write, WriteError};
use crate::model::common::{binary::Chunks, CommonPartialEq, Ownership};
use std::convert::Infallible;
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
    unresolved_sequences: bool,
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
    /// Decodes ordinary track storage using an explicitly resolved skeleton context.
    fn decode_with_sequences(data: &'data [u8], _external: &[bool]) -> Result<Self, ReadError> {
        Self::decode(data)
    }
    /// Encodes native framing and typed core records.
    fn encode(&self) -> Result<Vec<u8>, WriteError>;
    /// Reports whether nonempty animation descriptors need skeleton storage context.
    fn requires_sequence_context(&self) -> bool {
        false
    }
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
            unresolved_sequences: false,
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
    fn requires_sequence_context(&self) -> bool {
        self.unresolved_sequences
    }
    fn decode(data: &'data [u8]) -> Result<Self, ReadError> {
        Self::decode_context(data, None)
    }
    fn decode_with_sequences(data: &'data [u8], external: &[bool]) -> Result<Self, ReadError> {
        Self::decode_context(data, Some(external))
    }
    /// Rebuilds the document and relocates all locally stored payloads.
    fn encode(&self) -> Result<Vec<u8>, WriteError> {
        self.encode_chunks()
    }
}
impl<'data, V: ModelLayout> ChunkStream<'data, V> {
    fn decode_context(data: &'data [u8], inherited: Option<&[bool]>) -> Result<Self, ReadError> {
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
        let deferred = raw.iter().any(|(tag, payload, _)| {
            tag == b"SKID" && payload.get(..4).is_some_and(|id| id != [0; 4])
        });
        let core = Core::decode_context(primary.1, primary.2, inherited, deferred)?;
        let external: Vec<bool> = if core.sequences.is_empty() {
            inherited.unwrap_or_default().to_vec()
        } else {
            core.sequences
                .iter()
                .map(|sequence| sequence.flags.external_animation())
                .collect()
        };
        let unresolved_sequences = deferred && inherited.is_none() && core.sequences.is_empty();
        let flags = core.flags;
        let core_light_count = core.lights.len();
        let mut primary_core = Some(core);
        let mut pending = PendingArrays::default();
        let mut chunks = Vec::new();
        for (tag, payload, origin) in raw {
            if tag == *b"MD21" {
                let mut core = match primary_core.take() {
                    Some(core) => core,
                    None => Core::decode_context(payload, origin, inherited, deferred)?,
                };
                if deferred && inherited.is_none() && core.sequences.is_empty() {
                    let _ = visit_core(&mut core, &mut pending);
                }
                chunks.push(ModelChunk::Core(Box::new(core)));
            } else {
                let mut context =
                    ChunkContext::<V>::new(flags, Some(core_light_count), external.clone().into());
                context.unresolved_sequences = unresolved_sequences;
                let mut chunk = Chunk::read(tag, payload, &context).map_err(|mut error| {
                    error.offset = error.offset.saturating_add(origin);
                    error
                })?;
                if unresolved_sequences {
                    match &mut chunk {
                        Chunk::ParentTextureWeights(records) => {
                            for record in records {
                                let _ = record.visit_arrays(&mut pending);
                            }
                        }
                        Chunk::ParentEventTracks(records) => {
                            for record in records {
                                let _ = record.visit_arrays(&mut pending);
                            }
                        }
                        _ => {}
                    }
                }
                chunks.push(ModelChunk::Extension(chunk));
            }
        }
        Ok(Self {
            chunks,
            unresolved_sequences: pending.0,
        })
    }
    /// Rebuilds the document and relocates all locally stored payloads.
    fn encode_chunks(&self) -> Result<Vec<u8>, WriteError> {
        if self.unresolved_sequences {
            return Err(WriteError::invalid("skeleton sequence storage is unresolved; encode a decoded bundle or supply sequence context"));
        }
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
#[derive(Default)]
struct PendingArrays(bool);
impl<'data> ArrayVisitor<'data> for PendingArrays {
    type Error = Infallible;
    fn visit<T: Value>(
        &mut self,
        _: usize,
        global: u16,
        values: &mut Array<'data, T>,
    ) -> Result<(), Infallible> {
        self.0 |= global == u16::MAX
            && matches!(values, Array::External(reference) if reference.count != 0);
        Ok(())
    }
}
/// An editable document whose layout and native envelope storage are static.
///
/// Flat models expose only their core; extension insertion requires native chunk
/// storage, obtained with `into_chunked()`.
///
/// ```compile_fail
/// use wc3::model::formats::m2::{Flat,Document,V264,chunks::Chunk};
/// let mut model = Document::<Flat<V264>>::new();
/// model.insert_extension(0,Chunk::PhysicsId(42));
/// ```
#[derive(Clone, Debug, CommonPartialEq, Ownership)]
pub struct Document<'data, D: Dialect> {
    storage: <D::Container as Container>::Storage<'data, D::Layout>,
    dialect: PhantomData<D>,
}
impl<'data, D: Dialect> Default for Document<'data, D> {
    fn default() -> Self {
        Self::new()
    }
}
impl<'data, D: Dialect> Document<'data, D> {
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
    /// Whether nonempty ordinary leaves await their skeleton's sequence-storage flags.
    ///
    /// Metadata and dependencies remain inspectable. Validation and encoding
    /// require [`Self::decode_m2_with_sequences`] or [`super::Bundle::decode`]
    /// to establish which deferred descriptors are inline and which are external.
    pub fn requires_sequence_context(&self) -> bool {
        self.storage.requires_sequence_context()
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
    /// Decodes with the external-storage flags of the resolved skeleton's sequences.
    ///
    /// The model's own sequence table takes precedence when present. Use this
    /// when decoding an SKID model directly; without this context, ordinary
    /// nonempty leaves stay unresolved and standalone encoding is rejected.
    /// [`super::Bundle::decode`] loads this context through its resolver.
    pub fn decode_m2_with_sequences(
        data: &'data [u8],
        external: &[bool],
    ) -> Result<Self, ReadError> {
        Ok(Self {
            storage: DocumentStorage::decode_with_sequences(data, external)?,
            dialect: PhantomData,
        })
    }
    /// Encodes the selected native envelope and layout.
    pub fn encode_m2(&self) -> Result<Vec<u8>, WriteError> {
        self.storage.encode()
    }
}
impl<'data, D: Dialect> Read<'data> for Document<'data, D> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let base = input.absolute_position();
        let remaining = input.data().len() - input.position();
        Self::decode_m2(input.bytes(remaining)?).map_err(|mut error| {
            error.offset = base.saturating_add(error.offset);
            error
        })
    }
}
impl<D: Dialect> Write for Document<'_, D> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.encode_m2()?)
    }
}
impl<'data, V: ModelLayout> Document<'data, Chunked<V>> {
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
    pub fn into_flat(mut self) -> Result<Document<'data, Flat<V>>, WriteError> {
        if self.storage.unresolved_sequences {
            return Err(WriteError::invalid(
                "skeleton sequence storage is unresolved",
            ));
        }
        if self.storage.chunks.len() != 1 {
            return Err(WriteError::invalid("additional chunks in flat envelope"));
        }
        let ModelChunk::Core(core) = self.storage.chunks.pop().unwrap() else {
            unreachable!("chunk stream has a core")
        };
        Ok(Document {
            storage: *core,
            dialect: PhantomData,
        })
    }
}
impl<'data, V: ModelLayout> Document<'data, Flat<V>> {
    /// Adds native chunk framing without converting core records.
    pub fn into_chunked(self) -> Document<'data, Chunked<V>> {
        Document {
            storage: ChunkStream {
                chunks: vec![ModelChunk::Core(Box::new(self.storage))],
                unresolved_sequences: false,
            },
            dialect: PhantomData,
        }
    }
}
