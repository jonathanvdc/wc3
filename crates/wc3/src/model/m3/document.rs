//! Indexed section storage, checked reference resolution, and relocating writes.
use super::codec::Encoder;
use super::records::{known_record_size, validate_known_section, ReferenceFlags};
use super::{
    Array, Envelope, Md34, RawReference, Read, ReadError, Record, RecordLayout, Ref, Target, Write,
    WriteError,
};
use crate::model::common::binary::{align_position, AddressSpace};
use crate::model::common::{Borrowed, IntoOwned};
use std::borrow::Cow;
use std::marker::PhantomData;
use std::str::from_utf8;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT_DOCUMENT: AtomicU64 = AtomicU64::new(1);

/// Stable index-table identity. Appending sections never changes existing IDs.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SectionId {
    index: u32,
    owner: u64,
}
impl SectionId {
    /// Index in this document's section table.
    pub fn index(self) -> u32 {
        self.index
    }
}

/// One indexed array. Unknown tags and unsupported revisions retain raw bytes.
#[derive(Clone, Debug)]
pub struct Section<'data> {
    tag: [u8; 4],
    revision: u32,
    count: u32,
    data: Cow<'data, [u8]>,
    original: Option<(usize, usize)>,
}
impl Section<'_> {
    /// Canonical (human-readable) section tag, independent of wire byte order.
    pub fn tag(&self) -> [u8; 4] {
        self.tag
    }
    /// Native record revision from the index entry.
    pub fn revision(&self) -> u32 {
        self.revision
    }
    /// Number of records advertised by the section index.
    pub fn count(&self) -> u32 {
        self.count
    }
    /// Native encoded records, or the bounded bytes of an opaque section.
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }
    /// True while this section still borrows its input.
    pub fn is_borrowed(&self) -> bool {
        matches!(self.data, Cow::Borrowed(_))
    }
}

/// An indexed document that may contain unknown sections or have no model root.
#[derive(Clone, Debug)]
pub struct Document<'data, E: Envelope = Md34> {
    sections: Vec<Section<'data>>,
    owner: u64,
    root: RawReference,
    source: Option<Cow<'data, [u8]>>,
    dirty: bool,
    envelope: PhantomData<E>,
}

fn word(data: &[u8], offset: usize) -> Result<u32, ReadError> {
    AddressSpace::new(data)
        .value(offset)
        .map_err(|_| ReadError::invalid(offset, "truncated integer"))
}
fn tag(data: &[u8], offset: usize) -> Result<[u8; 4], ReadError> {
    let mut tag: [u8; 4] = AddressSpace::new(data)
        .range(offset, 4)
        .map_err(|_| ReadError::invalid(offset, "section tag"))?
        .try_into()
        .unwrap();
    tag.reverse();
    Ok(tag)
}
fn align(value: usize) -> Result<usize, WriteError> {
    align_position(value, 16).map_err(|_| WriteError::invalid("file size"))
}
fn u32_size(value: usize) -> Result<u32, WriteError> {
    u32::try_from(value).map_err(|_| WriteError::invalid("32-bit file size"))
}

impl<'data, E: Envelope> Document<'data, E> {
    /// Creates an empty document. Set a root before converting it to a model.
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
            owner: NEXT_DOCUMENT.fetch_add(1, Ordering::Relaxed),
            root: RawReference {
                entries: 0,
                index: 0,
                flags: ReferenceFlags(0),
                target: Some(*b"MODL"),
            },
            source: None,
            dirty: true,
            envelope: PhantomData,
        }
    }
    /// Reads bounded index entries without copying section payloads.
    pub fn read(bytes: &'data [u8]) -> Result<Self, ReadError> {
        if tag(bytes, 0)? != E::MAGIC {
            return Err(ReadError::invalid(0, "envelope magic"));
        }
        let header_size = if E::REFERENCE_SIZE == 8 { 20 } else { 24 };
        if bytes.len() < header_size {
            return Err(ReadError::invalid(0, "header"));
        }
        let table = word(bytes, 4)? as usize;
        let count = word(bytes, 8)? as usize;
        let table_size = count
            .checked_mul(16)
            .ok_or_else(|| ReadError::invalid(8, "index count"))?;
        let table_end = table
            .checked_add(table_size)
            .ok_or_else(|| ReadError::invalid(4, "index span"))?;
        if table < header_size || table_end > bytes.len() {
            return Err(ReadError::invalid(table, "index span"));
        }
        let root = RawReference {
            entries: word(bytes, 12)?,
            index: word(bytes, 16)?,
            flags: if E::REFERENCE_SIZE == 8 {
                ReferenceFlags(0)
            } else {
                ReferenceFlags(word(bytes, 20)?)
            },
            target: Some(*b"MODL"),
        };
        let mut entries = Vec::new();
        entries
            .try_reserve(count)
            .map_err(|_| ReadError::invalid(8, "index allocation"))?;
        for i in 0..count {
            let pos = table + i * 16;
            let entry = (
                tag(bytes, pos)?,
                word(bytes, pos + 4)? as usize,
                word(bytes, pos + 8)?,
                word(bytes, pos + 12)?,
            );
            if entry.1 > bytes.len() || (entry.1 >= table && entry.1 < table_end) {
                return Err(ReadError::invalid(pos + 4, "section offset"));
            }
            entries.push(entry);
        }
        let mut boundaries: Vec<usize> = entries.iter().map(|entry| entry.1).collect();
        boundaries.extend([table, table_end, bytes.len()]);
        boundaries.sort_unstable();
        boundaries.dedup();
        let mut sections = Vec::with_capacity(count);
        let mut spans = Vec::new();
        for (section_tag, offset, count, revision) in entries {
            let header = section_tag == E::MAGIC;
            let size = if header {
                Some(header_size)
            } else if count == 0 {
                Some(0)
            } else {
                known_record_size::<E>(section_tag, revision)
                    .map(|size| {
                        size.checked_mul(count as usize)
                            .ok_or_else(|| ReadError::invalid(offset, "section size"))
                    })
                    .transpose()?
            };
            let end = match size {
                Some(size) => offset
                    .checked_add(size)
                    .ok_or_else(|| ReadError::invalid(offset, "section size"))?,
                None => *boundaries
                    .get(boundaries.partition_point(|&bound| bound <= offset))
                    .ok_or_else(|| ReadError::invalid(offset, "opaque section span"))?,
            };
            if end > bytes.len() || (offset < table_end && end > table) {
                return Err(ReadError::invalid(offset, "section span"));
            }
            if header && (offset != 0 || count != 1 || revision != 11) {
                return Err(ReadError::invalid(offset, "header index entry"));
            }
            if !header && offset < header_size && end > offset {
                return Err(ReadError::invalid(offset, "section overlaps header"));
            }
            if end > offset {
                spans.push((offset, end));
            }
            sections.push(Section {
                tag: section_tag,
                revision,
                count,
                data: Cow::Borrowed(&bytes[offset..end]),
                original: Some((offset, end - offset)),
            });
        }
        spans.sort_unstable();
        if spans.windows(2).any(|w| w[0].1 > w[1].0) {
            return Err(ReadError::invalid(0, "overlapping sections"));
        }
        Ok(Self {
            sections,
            owner: NEXT_DOCUMENT.fetch_add(1, Ordering::Relaxed),
            root,
            source: Some(Cow::Borrowed(bytes)),
            dirty: false,
            envelope: PhantomData,
        })
    }
    /// Sections in stable index order, including the indexed envelope header.
    pub fn sections(&self) -> impl ExactSizeIterator<Item = (SectionId, &Section<'data>)> {
        self.sections.iter().enumerate().map(|(i, s)| {
            (
                SectionId {
                    index: i as u32,
                    owner: self.owner,
                },
                s,
            )
        })
    }
    /// Returns an existing indexed section.
    pub fn section(&self, id: SectionId) -> Option<&Section<'data>> {
        if id.owner == self.owner {
            self.sections.get(id.index as usize)
        } else {
            None
        }
    }
    /// Recovers a document-owned identity from a native reference index.
    pub fn section_id(&self, index: u32) -> Option<SectionId> {
        self.sections.get(index as usize).map(|_| SectionId {
            index,
            owner: self.owner,
        })
    }
    /// Checks a family reference and returns its section without choosing a revision.
    pub fn referenced_section<T: Target>(
        &self,
        reference: Ref<T>,
    ) -> Result<Option<&Section<'data>>, ReadError> {
        if reference.is_empty() {
            return Ok(None);
        }
        self.check_reference(reference.raw())?;
        Ok(self.sections.get(reference.index as usize))
    }
    /// Replaces native bytes while retaining the tag, revision and record count.
    pub fn replace_bytes(&mut self, id: SectionId, bytes: Vec<u8>) -> Result<(), WriteError> {
        if id.owner != self.owner {
            return Err(WriteError::invalid("foreign section identity"));
        }
        let section = self
            .sections
            .get_mut(id.index as usize)
            .ok_or_else(|| WriteError::invalid("section identity"))?;
        if section.tag == E::MAGIC {
            return Err(WriteError::invalid("replace envelope bytes"));
        }
        if let Some(size) = known_record_size::<E>(section.tag, section.revision) {
            if size.checked_mul(section.count as usize) != Some(bytes.len()) {
                return Err(WriteError::invalid("section record span"));
            }
        }
        section.data = Cow::Owned(bytes);
        self.dirty = true;
        Ok(())
    }
    /// Header reference, including tool metadata flags.
    pub fn root_reference(&self) -> RawReference {
        self.root
    }
    /// Sets the header's reference. Model validation checks the referenced root.
    pub fn set_root<T: Target>(&mut self, reference: Ref<T>) {
        self.root = RawReference {
            entries: reference.entries,
            index: reference.index,
            flags: reference.flags,
            target: Some(*b"MODL"),
        };
        self.dirty = true;
    }
    /// Resolves a native null-terminated UTF-8 CHAR array without copying.
    /// Empty references return `None`; raw bytes remain accessible for other encodings.
    pub fn string<T: Target>(&self, reference: Ref<T>) -> Result<Option<&str>, ReadError> {
        let Some(section) = self.referenced_section(reference)? else {
            return Ok(None);
        };
        if section.tag != *b"CHAR" || section.revision != 0 {
            return Err(ReadError::invalid(0, "string section layout"));
        }
        let bytes = section
            .data
            .get(..reference.entries as usize)
            .ok_or_else(|| ReadError::invalid(0, "string span"))?;
        let text = bytes
            .strip_suffix(&[0])
            .ok_or_else(|| ReadError::invalid(0, "string terminator"))?;
        from_utf8(text)
            .map(Some)
            .map_err(|_| ReadError::invalid(0, "string encoding"))
    }
    /// Checks every understood record and its reference targets without following
    /// references recursively. Shared targets and cyclic graphs are well defined.
    pub fn validate(&self) -> Result<(), ReadError> {
        self.check_reference(self.root)?;
        for section in &self.sections {
            if section.tag == E::MAGIC {
                continue;
            }
            validate_known_section::<E>(
                section.tag,
                section.revision,
                &section.data,
                section.count,
                &mut |reference| self.check_reference(reference),
            )?;
        }
        Ok(())
    }
    fn check_reference(&self, reference: RawReference) -> Result<(), ReadError> {
        if reference.entries == 0 {
            return Ok(());
        }
        let section = self
            .sections
            .get(reference.index as usize)
            .ok_or_else(|| ReadError::invalid(0, "reference index"))?;
        if reference.target.is_some_and(|target| target != section.tag) {
            return Err(ReadError::invalid(0, "reference target tag"));
        }
        if reference.entries > section.count {
            return Err(ReadError::invalid(0, "reference entry count"));
        }
        Ok(())
    }
    /// Resolves a typed reference, checking both the tag and record revision.
    /// Empty references return an empty borrowed array and ignore the index.
    pub fn resolve<'view, T>(
        &'view self,
        reference: Ref<T>,
    ) -> Result<Array<'view, T, E>, ReadError>
    where
        T: Target + Record + RecordLayout<E> + Read<'view, E>,
    {
        if reference.entries == 0 {
            return Array::read(&[]);
        }
        self.check_reference(RawReference {
            entries: reference.entries,
            index: reference.index,
            flags: reference.flags,
            target: Some(<T as Record>::TAG),
        })?;
        let section = &self.sections[reference.index as usize];
        if section.revision != T::VERSION {
            return Err(ReadError::invalid(0, "reference target revision"));
        }
        let size = T::size()
            .checked_mul(reference.entries as usize)
            .ok_or_else(|| ReadError::invalid(0, "array size"))?;
        let bytes = section
            .data
            .get(..size)
            .ok_or_else(|| ReadError::invalid(0, "array span"))?;
        Array::read(bytes)
    }
    /// Opens all records of a section through its exact native revision.
    pub fn array<'view, T>(&'view self, id: SectionId) -> Result<Array<'view, T, E>, ReadError>
    where
        T: Target + Record + RecordLayout<E> + Read<'view, E>,
    {
        let section = self
            .section(id)
            .ok_or_else(|| ReadError::invalid(0, "section identity"))?;
        self.resolve(Ref::from_section(section.count, id.index))
    }
    fn ensure_header(&mut self) {
        if self.sections.is_empty() {
            self.sections.push(Section {
                tag: E::MAGIC,
                revision: 11,
                count: 1,
                data: Cow::Owned(vec![0; if E::REFERENCE_SIZE == 8 { 20 } else { 24 }]),
                original: None,
            });
        }
    }
    /// Appends an array and returns its stable identity. Existing IDs are retained.
    pub fn append<T: Record + Write<E> + RecordLayout<E>>(
        &mut self,
        values: &[T],
    ) -> Result<SectionId, WriteError> {
        let mut output = Encoder::<E>::new();
        for value in values {
            value.write(&mut output)?;
        }
        let bytes = output.finish();
        if <T as RecordLayout<E>>::size().checked_mul(values.len()) != Some(bytes.len()) {
            return Err(WriteError::invalid("encoded record span"));
        }
        self.append_raw(
            <T as Record>::TAG,
            T::VERSION,
            u32_size(values.len())?,
            bytes,
        )
    }
    /// Appends borrowed or owned indexed storage. Known layouts must have exactly count × size
    /// bytes; unknown layouts can only be preserved without relocation afterward.
    pub fn append_raw(
        &mut self,
        tag: [u8; 4],
        revision: u32,
        count: u32,
        bytes: impl Into<Cow<'data, [u8]>>,
    ) -> Result<SectionId, WriteError> {
        let bytes = bytes.into();
        if tag == E::MAGIC {
            return Err(WriteError::invalid("duplicate envelope section"));
        }
        if let Some(size) = known_record_size::<E>(tag, revision) {
            if size.checked_mul(count as usize) != Some(bytes.len()) {
                return Err(WriteError::invalid("section record span"));
            }
        }
        self.ensure_header();
        let id = SectionId {
            index: u32_size(self.sections.len())?,
            owner: self.owner,
        };
        self.sections.push(Section {
            tag,
            revision,
            count,
            data: bytes,
            original: None,
        });
        self.dirty = true;
        Ok(id)
    }
    /// Replaces a section's records without changing its identity or references.
    pub fn replace<T: Record + Write<E> + RecordLayout<E>>(
        &mut self,
        id: SectionId,
        values: &[T],
    ) -> Result<(), WriteError> {
        if id.owner != self.owner {
            return Err(WriteError::invalid("foreign section identity"));
        }
        let section = self
            .sections
            .get_mut(id.index as usize)
            .ok_or_else(|| WriteError::invalid("section identity"))?;
        if section.tag != <T as Record>::TAG || section.revision != T::VERSION {
            return Err(WriteError::invalid("replacement record revision"));
        }
        let mut output = Encoder::<E>::new();
        for value in values {
            value.write(&mut output)?;
        }
        let bytes = output.finish();
        if <T as RecordLayout<E>>::size().checked_mul(values.len()) != Some(bytes.len()) {
            return Err(WriteError::invalid("encoded record span"));
        }
        section.count = u32_size(values.len())?;
        section.data = Cow::Owned(bytes);
        self.dirty = true;
        Ok(())
    }
    /// Removes unreferenced records while retaining their table slot. Unknown
    /// payloads prevent deletion because their references cannot be inspected.
    pub fn clear(&mut self, id: SectionId) -> Result<(), WriteError> {
        if id.owner != self.owner {
            return Err(WriteError::invalid("foreign section identity"));
        }
        if self
            .sections
            .iter()
            .any(|s| s.tag != E::MAGIC && known_record_size::<E>(s.tag, s.revision).is_none())
        {
            return Err(WriteError::invalid("deletion with opaque sections"));
        }
        if self.root.entries > 0 && self.root.index == id.index {
            return Err(WriteError::invalid("delete model root"));
        }
        for section in &self.sections {
            if section.tag == E::MAGIC {
                continue;
            }
            let mut referenced = false;
            validate_known_section::<E>(
                section.tag,
                section.revision,
                &section.data,
                section.count,
                &mut |reference| {
                    if reference.entries > 0 && reference.index == id.index {
                        referenced = true;
                    }
                    Ok(())
                },
            )
            .map_err(|_| WriteError::invalid("record validation"))?;
            if referenced {
                return Err(WriteError::invalid("delete referenced section"));
            }
        }
        let section = self
            .sections
            .get_mut(id.index as usize)
            .ok_or_else(|| WriteError::invalid("section identity"))?;
        if section.tag == E::MAGIC {
            return Err(WriteError::invalid("delete envelope"));
        }
        section.count = 0;
        section.data = Cow::Owned(Vec::new());
        self.dirty = true;
        Ok(())
    }
    /// Writes an unchanged source exactly. Edits relocate understood sections,
    /// preserve table identities and flags, and rebuild the index. With opaque
    /// sections, edits must fit the original spans so no unknown pointer moves.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        self.validate()
            .map_err(|_| WriteError::invalid("document validation"))?;
        if E::REFERENCE_SIZE == 8 && self.root.flags.bits() != 0 {
            return Err(WriteError::invalid("MD33 reference flags"));
        }
        if !self.dirty {
            if let Some(source) = &self.source {
                return Ok(source.to_vec());
            }
        }
        let opaque = self
            .sections
            .iter()
            .any(|s| s.tag != E::MAGIC && known_record_size::<E>(s.tag, s.revision).is_none());
        if opaque {
            return self.write_in_place();
        }
        let mut bytes = vec![0xaa; 32];
        let mut offsets = Vec::with_capacity(self.sections.len());
        for section in &self.sections {
            if section.tag == E::MAGIC {
                offsets.push(0);
                continue;
            }
            bytes.resize(align(bytes.len())?, 0xaa);
            offsets.push(u32_size(bytes.len())?);
            bytes.extend_from_slice(&section.data);
        }
        bytes.resize(align(bytes.len())?, 0xaa);
        let table = u32_size(bytes.len())?;
        for (section, &offset) in self.sections.iter().zip(&offsets) {
            let mut tag = section.tag;
            tag.reverse();
            bytes.extend_from_slice(&tag);
            bytes.extend_from_slice(&offset.to_le_bytes());
            bytes.extend_from_slice(&section.count.to_le_bytes());
            bytes.extend_from_slice(&section.revision.to_le_bytes());
        }
        self.write_header(&mut bytes, table)?;
        Ok(bytes)
    }
    fn write_header(&self, bytes: &mut [u8], table: u32) -> Result<(), WriteError> {
        let mut magic = E::MAGIC;
        magic.reverse();
        bytes[..4].copy_from_slice(&magic);
        bytes[4..8].copy_from_slice(&table.to_le_bytes());
        bytes[8..12].copy_from_slice(&u32_size(self.sections.len())?.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.root.entries.to_le_bytes());
        bytes[16..20].copy_from_slice(&self.root.index.to_le_bytes());
        if E::REFERENCE_SIZE == 12 {
            bytes[20..24].copy_from_slice(&self.root.flags.bits().to_le_bytes());
        }
        Ok(())
    }
    fn write_in_place(&self) -> Result<Vec<u8>, WriteError> {
        let source = self
            .source
            .as_ref()
            .ok_or_else(|| WriteError::invalid("opaque sections require original placement"))?;
        let mut bytes = source.to_vec();
        let table = word(&bytes, 4).map_err(|_| WriteError::invalid("source index"))? as usize;
        if word(&bytes, 8).map_err(|_| WriteError::invalid("source index"))? as usize
            != self.sections.len()
        {
            return Err(WriteError::invalid("append with opaque sections"));
        }
        for (i, section) in self.sections.iter().enumerate() {
            if section.tag == E::MAGIC {
                continue;
            }
            let (offset, len) = section
                .original
                .ok_or_else(|| WriteError::invalid("opaque section placement"))?;
            if section.data.len() != len {
                return Err(WriteError::invalid("resize with opaque sections"));
            }
            bytes[offset..offset + len].copy_from_slice(&section.data);
            bytes[table + i * 16 + 8..table + i * 16 + 12]
                .copy_from_slice(&section.count.to_le_bytes());
        }
        self.write_header(&mut bytes, u32_size(table)?)?;
        Ok(bytes)
    }
    /// Detaches all source buffers while retaining graph identities and placement.
    pub fn into_owned(self) -> Document<'static, E> {
        Document {
            owner: self.owner,
            sections: self
                .sections
                .into_iter()
                .map(|s| Section {
                    tag: s.tag,
                    revision: s.revision,
                    count: s.count,
                    data: Cow::Owned(s.data.into_owned()),
                    original: s.original,
                })
                .collect(),
            root: self.root,
            source: self.source.map(|s| Cow::Owned(s.into_owned())),
            dirty: self.dirty,
            envelope: PhantomData,
        }
    }
}
impl<E: Envelope> Default for Document<'_, E> {
    fn default() -> Self {
        Self::new()
    }
}

/// A checked model graph with exactly one supported MODL root record.
#[derive(Clone, Debug)]
pub struct Model<'data, E: Envelope = Md34> {
    document: Document<'data, E>,
}
impl<'data, E: Envelope> Model<'data, E> {
    /// Reads a complete native model or animation document.
    pub fn read(bytes: &'data [u8]) -> Result<Self, ReadError> {
        Self::from_document(Document::read(bytes)?)
    }
    /// Checks root cardinality, native layout, and all understood graph links.
    pub fn from_document(document: Document<'data, E>) -> Result<Self, ReadError> {
        document.validate()?;
        let root = document.root;
        if root.entries != 1 {
            return Err(ReadError::invalid(12, "single model root"));
        }
        let section = &document.sections[root.index as usize];
        if section.tag != *b"MODL"
            || known_record_size::<E>(section.tag, section.revision).is_none()
        {
            return Err(ReadError::invalid(12, "supported model root"));
        }
        Ok(Self { document })
    }
    /// Native root section identity.
    pub fn root_id(&self) -> SectionId {
        SectionId {
            index: self.document.root.index,
            owner: self.document.owner,
        }
    }
    /// Checked section graph for resolving arrays and inspecting revisions.
    pub fn document(&self) -> &Document<'data, E> {
        &self.document
    }
    /// Returns the document for editing; conversion back checks the entire graph.
    pub fn into_document(self) -> Document<'data, E> {
        self.document
    }
    /// Decodes the root through its exact selected native revision.
    pub fn root<T>(&self) -> Result<T, ReadError>
    where
        T: Target + Record + RecordLayout<E>,
        for<'view> T: Read<'view, E>,
    {
        if <T as Record>::TAG != *b"MODL" {
            return Err(ReadError::invalid(0, "root record tag"));
        }
        self.document
            .array::<T>(self.root_id())?
            .get(0)
            .ok_or_else(|| ReadError::invalid(0, "root record"))?
    }
    /// Encodes the native document without converting section revisions.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        self.document.write()
    }
    /// Detaches source storage for long-lived use.
    pub fn into_owned(self) -> Model<'static, E> {
        Model {
            document: self.document.into_owned(),
        }
    }
}

impl<'view, E: Envelope> Borrowed<'view> for Document<'_, E> {
    type View = Document<'view, E>;
    fn as_borrowed(&'view self) -> Self::View {
        Document {
            owner: self.owner,
            sections: self
                .sections
                .iter()
                .map(|s| Section {
                    tag: s.tag,
                    revision: s.revision,
                    count: s.count,
                    data: Cow::Borrowed(&s.data),
                    original: s.original,
                })
                .collect(),
            root: self.root,
            source: self.source.as_ref().map(|s| Cow::Borrowed(s.as_ref())),
            dirty: self.dirty,
            envelope: PhantomData,
        }
    }
}
impl<E: Envelope> IntoOwned for Document<'_, E> {
    type Owned = Document<'static, E>;
    fn into_owned(self) -> Self::Owned {
        Document::into_owned(self)
    }
}
impl<'view, E: Envelope> Borrowed<'view> for Model<'_, E> {
    type View = Model<'view, E>;
    fn as_borrowed(&'view self) -> Self::View {
        Model {
            document: self.document.as_borrowed(),
        }
    }
}
impl<E: Envelope> IntoOwned for Model<'_, E> {
    type Owned = Model<'static, E>;
    fn into_owned(self) -> Self::Owned {
        Model::into_owned(self)
    }
}
