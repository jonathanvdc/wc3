//! Stable indexed storage for editable-model files.
use super::codec::{Decoder, Encoder};
use super::records::{
    known_record_span, validate_known_section, DocumentRecord, ProfileId, ProfileMaterialSet, V1,
};
use super::{
    Array, NativeAbi, RawReference, Read, ReadError, Record, RecordLayout, Ref, Target, Write,
    WriteError,
};
use crate::model::common::{Borrowed, IntoOwned};
use std::borrow::Cow;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::str::from_utf8;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT_DOCUMENT: AtomicU64 = AtomicU64::new(1);
/// Explicit interpretation choices for native blocks without an ABI marker.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Options {
    /// Producer word width, required for ABI-dependent native material records.
    pub native_abi: Option<NativeAbi>,
}
/// Supported third-generation file envelope.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct V3;
mod sealed {
    pub trait Dialect {}
}
impl sealed::Dialect for V3 {}
/// File generation, independent of each section's record revision.
pub trait Dialect: sealed::Dialect + Clone + Copy + Debug + 'static {
    /// Version stored in the file header.
    const VERSION: u32;
}
impl Dialect for V3 {
    const VERSION: u32 = 3;
}
/// An index identity tied to its originating document.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct SectionId {
    index: u32,
    owner: u64,
}
impl SectionId {
    /// Native index-table position.
    pub fn index(self) -> u32 {
        self.index
    }
}
/// One indexed payload with its native record count and revision.
#[derive(Clone, Debug)]
pub struct Section<'data> {
    tag: [u8; 4],
    revision: u32,
    count: u32,
    data: Cow<'data, [u8]>,
    original: Option<(usize, usize)>,
}
impl Section<'_> {
    /// Literal native FourCC.
    pub fn tag(&self) -> [u8; 4] {
        self.tag
    }
    /// Section-specific layout revision.
    pub fn revision(&self) -> u32 {
        self.revision
    }
    /// Native number of rows in the section.
    pub fn count(&self) -> u32 {
        self.count
    }
    /// Encoded rows; opaque sections also include their original alignment tail.
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }
    /// Whether this payload still borrows its source.
    pub fn is_borrowed(&self) -> bool {
        matches!(self.data, Cow::Borrowed(_))
    }
}
/// Raw indexed storage, including unsupported generations or section layouts.
#[derive(Clone, Debug)]
pub struct RawDocument<'data> {
    sections: Vec<Section<'data>>,
    owner: u64,
    root: RawReference,
    version: u32,
    profile_mask: u32,
    options: Options,
    source: Option<Cow<'data, [u8]>>,
    dirty: bool,
}
fn word(bytes: &[u8], offset: usize) -> Result<u32, ReadError> {
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or_else(|| ReadError::invalid(offset, "integer span"))?
            .try_into()
            .unwrap(),
    ))
}
fn tag(bytes: &[u8], offset: usize) -> Result<[u8; 4], ReadError> {
    Ok(bytes
        .get(offset..offset + 4)
        .ok_or_else(|| ReadError::invalid(offset, "tag span"))?
        .try_into()
        .unwrap())
}
fn width(value: usize) -> Result<u32, WriteError> {
    u32::try_from(value).map_err(|_| WriteError::invalid("32-bit file size"))
}
fn align(value: usize) -> Result<usize, WriteError> {
    value
        .checked_add(15)
        .map(|v| v & !15)
        .ok_or_else(|| WriteError::invalid("file size"))
}
impl<'data> RawDocument<'data> {
    /// Starts a third-generation indexed document with no root.
    pub fn new() -> Self {
        Self::new_with_options(Options::default())
    }
    /// Starts a document with explicit native interpretation choices.
    pub fn new_with_options(options: Options) -> Self {
        Self {
            sections: Vec::new(),
            owner: NEXT_DOCUMENT.fetch_add(1, Ordering::Relaxed),
            root: RawReference {
                entries: 0,
                index: 0,
                flags: 0,
                target: Some(*b"WDOC"),
            },
            version: 3,
            profile_mask: 0,
            options,
            source: None,
            dirty: true,
        }
    }
    /// Reads an index without guessing a native producer ABI.
    pub fn read(bytes: &'data [u8]) -> Result<Self, ReadError> {
        Self::read_with_options(bytes, Options::default())
    }
    /// Reads bounded sections using explicit native producer information.
    pub fn read_with_options(bytes: &'data [u8], options: Options) -> Result<Self, ReadError> {
        if bytes.len() < 32 || bytes.get(..4) != Some(b"WOEM") {
            return Err(ReadError::invalid(0, "file header"));
        }
        let table = word(bytes, 8)? as usize;
        let count = word(bytes, 12)? as usize;
        let end = table
            .checked_add(
                count
                    .checked_mul(16)
                    .ok_or_else(|| ReadError::invalid(12, "index count"))?,
            )
            .ok_or_else(|| ReadError::invalid(8, "index span"))?;
        if table < 32 || end > bytes.len() {
            return Err(ReadError::invalid(table, "index span"));
        }
        let version = word(bytes, 4)?;
        let root = RawReference {
            entries: word(bytes, 16)?,
            index: word(bytes, 20)?,
            flags: word(bytes, 24)?,
            target: Some(*b"WDOC"),
        };
        let mut entries = Vec::with_capacity(count);
        for row in 0..count {
            let at = table + row * 16;
            let entry = (
                tag(bytes, at)?,
                word(bytes, at + 4)? as usize,
                word(bytes, at + 8)?,
                word(bytes, at + 12)?,
            );
            if entry.1 > bytes.len() || (entry.1 >= table && entry.1 < end) {
                return Err(ReadError::invalid(at + 4, "section offset"));
            }
            entries.push(entry);
        }
        let mut bounds = Vec::new();
        for &(tag, offset, count, revision) in &entries {
            // A known empty array has no physical span to bound another payload.
            if tag == *b"HOLE"
                || (version == 3
                    && count == 0
                    && known_record_span(tag, revision, &[], 0, options.native_abi)? == Some(0))
            {
                continue;
            }
            bounds.push(offset);
        }
        bounds.extend([table, end, bytes.len()]);
        bounds.sort_unstable();
        bounds.dedup();
        let mut sections = Vec::with_capacity(count);
        let mut spans = Vec::new();
        for (tag, offset, count, revision) in entries {
            let header = tag == *b"WOEM";
            let hole = tag == *b"HOLE";
            let boundary = *bounds
                .get(bounds.partition_point(|&b| b <= offset))
                .unwrap_or(&bytes.len());
            let length = if header {
                32
            } else if hole || (count == 0 && offset == 0) {
                0
            } else {
                let payload = bytes
                    .get(offset..boundary)
                    .ok_or_else(|| ReadError::invalid(offset, "section span"))?;
                if version == 3 {
                    known_record_span(tag, revision, payload, count, options.native_abi)
                        .map_err(|mut error| {
                            error.offset = error.offset.saturating_add(offset);
                            error
                        })?
                        .unwrap_or(payload.len())
                } else {
                    payload.len()
                }
            };
            let finish = offset
                .checked_add(length)
                .ok_or_else(|| ReadError::invalid(offset, "section size"))?;
            if finish > bytes.len()
                || (offset < end && finish > table)
                || (!header && offset < 32 && finish > offset)
            {
                return Err(ReadError::invalid(offset, "section span"));
            }
            if header && (offset != 0 || count != 1 || revision != 0) {
                return Err(ReadError::invalid(offset, "header index"));
            }
            if hole && (count != 0 || offset != 0) {
                return Err(ReadError::invalid(offset, "hole index"));
            }
            if finish > offset {
                spans.push((offset, finish));
            }
            sections.push(Section {
                tag,
                revision,
                count,
                data: Cow::Borrowed(&bytes[offset..finish]),
                original: Some((offset, length)),
            });
        }
        spans.sort_unstable();
        if spans.windows(2).any(|w| w[0].1 > w[1].0) {
            return Err(ReadError::invalid(0, "section overlap"));
        }
        Ok(Self {
            sections,
            owner: NEXT_DOCUMENT.fetch_add(1, Ordering::Relaxed),
            root,
            version,
            profile_mask: word(bytes, 28)?,
            options,
            source: Some(Cow::Borrowed(bytes)),
            dirty: false,
        })
    }
    /// Global file generation, which does not select section revisions.
    pub fn version(&self) -> u32 {
        self.version
    }
    /// Header profile bits, including an unspecified zero mask.
    pub fn profile_mask(&self) -> u32 {
        self.profile_mask
    }
    /// Sets profile metadata without normalizing document contents.
    pub fn set_profile_mask(&mut self, mask: u32) {
        self.profile_mask = mask;
        self.dirty = true;
    }
    /// Native decoding choices retained for lazy record access.
    pub fn options(&self) -> Options {
        self.options
    }
    /// Native header root link.
    pub fn root_reference(&self) -> RawReference {
        self.root
    }
    /// Sets the WDOC header reference; checked documents validate the target.
    pub fn set_root<T: Target>(&mut self, reference: Ref<T>) {
        self.root = RawReference {
            entries: reference.entries,
            index: reference.index,
            flags: reference.flags,
            target: Some(*b"WDOC"),
        };
        self.dirty = true;
    }
    /// All index slots in native order, including retained holes.
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
    /// Looks up a document-owned index slot.
    pub fn section(&self, id: SectionId) -> Option<&Section<'data>> {
        if id.owner == self.owner {
            self.sections.get(id.index as usize)
        } else {
            None
        }
    }
    /// Creates a document-owned identity from a native index.
    pub fn section_id(&self, index: u32) -> Option<SectionId> {
        self.sections.get(index as usize).map(|_| SectionId {
            index,
            owner: self.owner,
        })
    }
    fn check_reference(&self, reference: RawReference) -> Result<(), ReadError> {
        if reference.entries == 0 {
            return Ok(());
        }
        let section = self
            .sections
            .get(reference.index as usize)
            .ok_or_else(|| ReadError::invalid(0, "reference index"))?;
        if reference.target.is_some_and(|t| t != section.tag) {
            return Err(ReadError::invalid(0, "reference tag"));
        }
        if reference.entries > section.count {
            return Err(ReadError::invalid(0, "reference count"));
        }
        Ok(())
    }
    /// Resolves a family link while retaining its independent revision.
    pub fn referenced_section<T: Target>(
        &self,
        reference: Ref<T>,
    ) -> Result<Option<&Section<'data>>, ReadError> {
        if reference.entries == 0 {
            return Ok(None);
        }
        self.check_reference(reference.raw())?;
        Ok(self.sections.get(reference.index as usize))
    }
    /// Resolves exactly the requested prefix under a selected native record revision.
    pub fn resolve<'view, T>(&'view self, reference: Ref<T>) -> Result<Array<'view, T>, ReadError>
    where
        T: Record + RecordLayout + Read<'view>,
    {
        if reference.entries == 0 {
            return Array::read_with_abi(&[], 0, T::VERSION, self.options.native_abi);
        }
        if T::NATIVE_ABI.is_some() && T::NATIVE_ABI != self.options.native_abi {
            return Err(ReadError::invalid(0, "native record ABI"));
        }
        self.check_reference(reference.raw())?;
        let section = &self.sections[reference.index as usize];
        if section.revision != T::VERSION {
            return Err(ReadError::invalid(0, "reference revision"));
        }
        let minimum = T::minimum_size();
        if reference.entries != 0
            && (minimum == 0
                || (reference.entries as usize)
                    .checked_mul(minimum)
                    .is_none_or(|span| span > section.data.len()))
        {
            return Err(ReadError::invalid(0, "reference byte span"));
        }
        let mut input =
            Decoder::new(&section.data, section.revision).with_native_abi(self.options.native_abi);
        for _ in 0..reference.entries {
            T::skip(&mut input)?;
        }
        Array::read_with_abi(
            &section.data[..input.position()],
            reference.entries,
            section.revision,
            self.options.native_abi,
        )
    }
    /// Opens every row of an indexed section with an exact record revision.
    pub fn array<'view, T>(&'view self, id: SectionId) -> Result<Array<'view, T>, ReadError>
    where
        T: Record + RecordLayout + Read<'view>,
    {
        let section = self
            .section(id)
            .ok_or_else(|| ReadError::invalid(0, "section identity"))?;
        if section.tag != <T as Record>::TAG || section.revision != T::VERSION {
            return Err(ReadError::invalid(0, "array record layout"));
        }
        if T::NATIVE_ABI.is_some() && T::NATIVE_ABI != self.options.native_abi {
            return Err(ReadError::invalid(0, "native record ABI"));
        }
        self.resolve(Ref::new(section.count, id.index, 0))
    }
    /// Borrows a CHAR reference as UTF-8. WEM string counts exclude a terminator.
    pub fn string<T: Target>(&self, reference: Ref<T>) -> Result<Option<&str>, ReadError> {
        let Some(section) = self.referenced_section(reference)? else {
            return Ok(None);
        };
        if section.tag != *b"CHAR" || section.revision != 0 {
            return Err(ReadError::invalid(0, "string layout"));
        }
        from_utf8(
            section
                .data
                .get(..reference.entries as usize)
                .ok_or_else(|| ReadError::invalid(0, "string span"))?,
        )
        .map(Some)
        .map_err(|_| ReadError::invalid(0, "string encoding"))
    }
    /// Checks understood native records and graph links without recursive target loading.
    pub fn validate(&self) -> Result<(), ReadError> {
        self.check_reference(self.root)?;
        if self.version != 3 {
            return Ok(());
        }
        if self.sections.first().is_none_or(|s| s.tag != *b"WOEM") {
            return Err(ReadError::invalid(0, "header index slot"));
        }
        for section in &self.sections {
            if matches!(&section.tag, b"WOEM" | b"HOLE") {
                continue;
            }
            if known_record_span(
                section.tag,
                section.revision,
                &section.data,
                section.count,
                self.options.native_abi,
            )?
            .is_none()
            {
                continue;
            }
            validate_known_section(
                section.tag,
                section.revision,
                &section.data,
                section.count,
                self.options.native_abi,
                &mut |reference| self.check_reference(reference),
            )?;
        }
        Ok(())
    }
    fn ensure_header(&mut self) {
        if self.sections.is_empty() {
            self.sections.push(Section {
                tag: *b"WOEM",
                revision: 0,
                count: 1,
                data: Cow::Owned(vec![0; 32]),
                original: None,
            });
        }
    }
    /// Adds typed rows while preserving every existing slot identity.
    pub fn append<T: Record + Write>(&mut self, values: &[T]) -> Result<SectionId, WriteError> {
        if T::NATIVE_ABI.is_some() && T::NATIVE_ABI != self.options.native_abi {
            return Err(WriteError::invalid("native record ABI"));
        }
        let mut output = Encoder::new(T::VERSION).with_native_abi(self.options.native_abi);
        for value in values {
            value.write(&mut output)?;
        }
        self.append_raw(
            <T as Record>::TAG,
            T::VERSION,
            width(values.len())?,
            output.finish(),
        )
    }
    /// Appends explicitly supplied native bytes. Opaque authored sections cannot be relocated.
    pub fn append_raw(
        &mut self,
        tag: [u8; 4],
        revision: u32,
        count: u32,
        bytes: Vec<u8>,
    ) -> Result<SectionId, WriteError> {
        if matches!(&tag, b"WOEM" | b"HOLE") {
            return Err(WriteError::invalid("reserved section tag"));
        }
        if let Some(span) = known_record_span(tag, revision, &bytes, count, self.options.native_abi)
            .map_err(|_| WriteError::invalid("record span"))?
        {
            if span != bytes.len() {
                return Err(WriteError::invalid("record trailing bytes"));
            }
        }
        self.ensure_header();
        let id = SectionId {
            index: width(self.sections.len())?,
            owner: self.owner,
        };
        self.sections.push(Section {
            tag,
            revision,
            count,
            data: Cow::Owned(bytes),
            original: None,
        });
        self.dirty = true;
        Ok(id)
    }
    /// Replaces a typed section without renumbering it or upgrading its revision.
    pub fn replace<T: Record + Write>(
        &mut self,
        id: SectionId,
        values: &[T],
    ) -> Result<(), WriteError> {
        let section = self
            .section(id)
            .ok_or_else(|| WriteError::invalid("section identity"))?;
        if matches!(&section.tag, b"WOEM" | b"HOLE") {
            return Err(WriteError::invalid("reserved section edit"));
        }
        if section.tag != <T as Record>::TAG || section.revision != T::VERSION {
            return Err(WriteError::invalid("replacement revision"));
        }
        if T::NATIVE_ABI.is_some() && T::NATIVE_ABI != self.options.native_abi {
            return Err(WriteError::invalid("native record ABI"));
        }
        let mut output = Encoder::new(T::VERSION).with_native_abi(self.options.native_abi);
        for value in values {
            value.write(&mut output)?;
        }
        let bytes = output.finish();
        let count = width(values.len())?;
        if let Some(span) = known_record_span(
            <T as Record>::TAG,
            T::VERSION,
            &bytes,
            count,
            self.options.native_abi,
        )
        .map_err(|_| WriteError::invalid("record span"))?
        {
            if span != bytes.len() {
                return Err(WriteError::invalid("record trailing bytes"));
            }
        }
        let section = &mut self.sections[id.index as usize];
        section.count = count;
        section.data = Cow::Owned(bytes);
        self.dirty = true;
        Ok(())
    }
    /// Replaces bytes while preserving tag, revision and row count.
    pub fn replace_bytes(&mut self, id: SectionId, bytes: Vec<u8>) -> Result<(), WriteError> {
        let section = self
            .section(id)
            .ok_or_else(|| WriteError::invalid("section identity"))?;
        if matches!(&section.tag, b"WOEM" | b"HOLE") {
            return Err(WriteError::invalid("reserved section edit"));
        }
        if let Some(span) = known_record_span(
            section.tag,
            section.revision,
            &bytes,
            section.count,
            self.options.native_abi,
        )
        .map_err(|_| WriteError::invalid("record span"))?
        {
            if span != bytes.len() {
                return Err(WriteError::invalid("record trailing bytes"));
            }
        }
        self.sections[id.index as usize].data = Cow::Owned(bytes);
        self.dirty = true;
        Ok(())
    }
    fn opaque(&self) -> Result<bool, WriteError> {
        if self.version != 3 {
            return Ok(true);
        }
        for section in &self.sections {
            if matches!(&section.tag, b"WOEM" | b"HOLE") {
                continue;
            }
            if known_record_span(
                section.tag,
                section.revision,
                &section.data,
                section.count,
                self.options.native_abi,
            )
            .map_err(|_| WriteError::invalid("record span"))?
            .is_none()
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// Drops payloads unreachable from the document root without changing index identities.
    ///
    /// This requires understood reference layouts. Removed slots become holes, and
    /// the original container buffer is released because subsequent writes rebuild
    /// the graph. It is useful after replacing tables during semantic editing.
    pub fn prune_unreachable(&mut self) -> Result<usize, WriteError> {
        if self.opaque()? {
            return Err(WriteError::invalid("reachability with opaque sections"));
        }
        self.validate()
            .map_err(|_| WriteError::invalid("document graph"))?;
        let mut reachable = vec![false; self.sections.len()];
        if !reachable.is_empty() {
            reachable[0] = true;
        }
        let mut pending = Vec::new();
        if self.root.entries != 0 {
            pending.push(self.root.index);
        }
        while let Some(index) = pending.pop() {
            if reachable[index as usize] {
                continue;
            }
            reachable[index as usize] = true;
            let section = &self.sections[index as usize];
            validate_known_section(
                section.tag,
                section.revision,
                &section.data,
                section.count,
                self.options.native_abi,
                &mut |reference| {
                    if reference.entries != 0 {
                        pending.push(reference.index);
                    }
                    Ok(())
                },
            )
            .map_err(|_| WriteError::invalid("reference traversal"))?;
        }
        let mut removed = 0;
        for (section, live) in self.sections.iter_mut().zip(reachable) {
            if !live && section.tag != *b"HOLE" {
                section.tag = *b"HOLE";
                section.count = 0;
                section.revision = 0;
                section.data = Cow::Borrowed(&[]);
                section.original = None;
                removed += 1;
            }
        }
        self.source = None;
        self.dirty = true;
        Ok(removed)
    }
    /// Empties an unreferenced array, retaining its original index slot.
    pub fn clear(&mut self, id: SectionId) -> Result<(), WriteError> {
        let section = self
            .section(id)
            .ok_or_else(|| WriteError::invalid("section identity"))?;
        if matches!(&section.tag, b"WOEM" | b"HOLE")
            || self.root.entries > 0 && self.root.index == id.index
        {
            return Err(WriteError::invalid("required section removal"));
        }
        if self.opaque()? {
            return Err(WriteError::invalid("deletion with opaque sections"));
        }
        for section in &self.sections {
            if matches!(&section.tag, b"WOEM" | b"HOLE") {
                continue;
            }
            let mut incoming = false;
            validate_known_section(
                section.tag,
                section.revision,
                &section.data,
                section.count,
                self.options.native_abi,
                &mut |r| {
                    incoming |= r.entries > 0 && r.index == id.index;
                    Ok(())
                },
            )
            .map_err(|_| WriteError::invalid("record validation"))?;
            if incoming {
                return Err(WriteError::invalid("referenced section removal"));
            }
        }
        let section = &mut self.sections[id.index as usize];
        section.count = 0;
        section.data = Cow::Owned(Vec::new());
        self.dirty = true;
        Ok(())
    }
    /// Re-emits an unchanged source exactly. Known edits rebuild aligned payloads;
    /// opaque sections require unchanged slots and lengths at their original offsets.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        self.validate()
            .map_err(|_| WriteError::invalid("document validation"))?;
        if !self.dirty {
            if let Some(source) = &self.source {
                return Ok(source.to_vec());
            }
        }
        if self.opaque()? {
            return self.write_in_place();
        }
        let mut bytes = vec![0xaa; 32];
        let mut offsets = Vec::new();
        for section in &self.sections {
            if matches!(&section.tag, b"WOEM" | b"HOLE") {
                offsets.push(0);
                continue;
            }
            bytes.resize(align(bytes.len())?, 0xaa);
            offsets.push(width(bytes.len())?);
            bytes.extend_from_slice(&section.data);
        }
        bytes.resize(align(bytes.len())?, 0xaa);
        let table = width(bytes.len())?;
        for (section, offset) in self.sections.iter().zip(offsets) {
            bytes.extend_from_slice(&section.tag);
            bytes.extend_from_slice(&offset.to_le_bytes());
            bytes.extend_from_slice(&section.count.to_le_bytes());
            bytes.extend_from_slice(&section.revision.to_le_bytes());
        }
        self.header(&mut bytes, table)?;
        Ok(bytes)
    }
    fn header(&self, bytes: &mut [u8], table: u32) -> Result<(), WriteError> {
        bytes[..4].copy_from_slice(b"WOEM");
        for (at, value) in [
            (4, self.version),
            (8, table),
            (12, width(self.sections.len())?),
            (16, self.root.entries),
            (20, self.root.index),
            (24, self.root.flags),
            (28, self.profile_mask),
        ] {
            bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }
    fn write_in_place(&self) -> Result<Vec<u8>, WriteError> {
        let mut bytes = self
            .source
            .as_ref()
            .ok_or_else(|| WriteError::invalid("opaque source placement"))?
            .to_vec();
        let table = word(&bytes, 8).map_err(|_| WriteError::invalid("source index"))? as usize;
        if word(&bytes, 12).map_err(|_| WriteError::invalid("source index"))? as usize
            != self.sections.len()
        {
            return Err(WriteError::invalid("append with opaque sections"));
        }
        for (row, section) in self.sections.iter().enumerate() {
            if matches!(&section.tag, b"WOEM" | b"HOLE") {
                continue;
            }
            let (offset, length) = section
                .original
                .ok_or_else(|| WriteError::invalid("opaque placement"))?;
            if section.data.len() != length {
                return Err(WriteError::invalid("resize with opaque sections"));
            }
            bytes[offset..offset + length].copy_from_slice(&section.data);
            bytes[table + row * 16 + 8..table + row * 16 + 12]
                .copy_from_slice(&section.count.to_le_bytes());
        }
        self.header(&mut bytes, width(table)?)?;
        Ok(bytes)
    }
    /// Moves owned storage and copies borrowed source buffers.
    pub fn into_owned(self) -> RawDocument<'static> {
        RawDocument {
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
            owner: self.owner,
            root: self.root,
            version: self.version,
            profile_mask: self.profile_mask,
            options: self.options,
            source: self.source.map(|s| Cow::Owned(s.into_owned())),
            dirty: self.dirty,
        }
    }
}
impl Default for RawDocument<'_> {
    fn default() -> Self {
        Self::new()
    }
}
/// A checked editable-model graph with one supported WDOC root and multiple models.
#[derive(Clone, Debug)]
pub struct Document<'data, D: Dialect = V3> {
    raw: RawDocument<'data>,
    dialect: PhantomData<D>,
}
impl<'data, D: Dialect> Document<'data, D> {
    /// Creates a valid empty graph using the current root revision.
    pub fn new() -> Result<Self, WriteError> {
        Self::new_with_options(Options::default())
    }
    /// Creates an empty graph with explicit native block interpretation.
    pub fn new_with_options(options: Options) -> Result<Self, WriteError> {
        use super::InlineList;
        let mut raw = RawDocument::new_with_options(options);
        let root = DocumentRecord::<V1> {
            profiles: InlineList(vec![ProfileId::GENERIC]),
            unit_scale: 1.0,
            ..Default::default()
        };
        let id = raw.append(&[root])?;
        raw.set_root(Ref::<DocumentRecord<V1>>::new(1, id.index(), 0));
        Self::from_raw(raw).map_err(|_| WriteError::invalid("empty document"))
    }
    /// Reads the selected file generation without assuming a native ABI.
    pub fn read(bytes: &'data [u8]) -> Result<Self, ReadError> {
        Self::from_raw(RawDocument::read(bytes)?)
    }
    /// Reads the selected generation using explicit native block interpretation.
    pub fn read_with_options(bytes: &'data [u8], options: Options) -> Result<Self, ReadError> {
        Self::from_raw(RawDocument::read_with_options(bytes, options)?)
    }
    /// Checks the file generation, root cardinality, known root layout and links.
    pub fn from_raw(raw: RawDocument<'data>) -> Result<Self, ReadError> {
        if raw.version != D::VERSION || raw.root.entries != 1 {
            return Err(ReadError::invalid(4, "document generation or root"));
        }
        raw.validate()?;
        let section = &raw.sections[raw.root.index as usize];
        if section.tag != *b"WDOC"
            || section.count != 1
            || known_record_span(
                section.tag,
                section.revision,
                &section.data,
                section.count,
                raw.options.native_abi,
            )?
            .is_none()
        {
            return Err(ReadError::invalid(16, "supported document root"));
        }
        let root_id = SectionId {
            index: raw.root.index,
            owner: raw.owner,
        };
        let root = raw
            .array::<DocumentRecord<V1>>(root_id)?
            .get(0)
            .ok_or_else(|| ReadError::invalid(16, "document root"))??;
        if !root.profiles.0.contains(&root.default_profile) {
            return Err(ReadError::invalid(0, "undeclared default profile"));
        }
        for (id, section) in raw.sections() {
            if section.tag == *b"PROF" && section.revision == 1 {
                for row in raw.array::<ProfileMaterialSet>(id)?.iter() {
                    if !root.profiles.0.contains(&row?.profile) {
                        return Err(ReadError::invalid(0, "undeclared material profile"));
                    }
                }
            }
        }
        Ok(Self {
            raw,
            dialect: PhantomData,
        })
    }
    /// Indexed native storage for typed row access.
    pub fn raw(&self) -> &RawDocument<'data> {
        &self.raw
    }
    /// Alias for indexed storage inspection.
    pub fn document(&self) -> &RawDocument<'data> {
        &self.raw
    }
    /// Transfers the graph for editing and subsequent revalidation.
    pub fn into_raw(self) -> RawDocument<'data> {
        self.raw
    }
    /// Native root section identity.
    pub fn root_id(&self) -> SectionId {
        SectionId {
            index: self.raw.root.index,
            owner: self.raw.owner,
        }
    }
    /// Reads the exact selected WDOC revision.
    pub fn root<T>(&self) -> Result<T, ReadError>
    where
        T: Record + RecordLayout,
        for<'view> T: Read<'view>,
    {
        if <T as Record>::TAG != *b"WDOC" {
            return Err(ReadError::invalid(0, "root record tag"));
        }
        self.raw
            .array::<T>(self.root_id())?
            .get(0)
            .ok_or_else(|| ReadError::invalid(0, "root record"))?
    }
    /// Borrows a native string through its CHAR reference.
    pub fn string<T: Target>(&self, reference: Ref<T>) -> Result<Option<&str>, ReadError> {
        self.raw.string(reference)
    }
    /// Encodes native section revisions without converting their contents.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        self.raw.write()
    }
    /// Detaches buffers while retaining section identities.
    pub fn into_owned(self) -> Document<'static, D> {
        Document {
            raw: self.raw.into_owned(),
            dialect: PhantomData,
        }
    }
}
impl<'view> Borrowed<'view> for RawDocument<'_> {
    type View = RawDocument<'view>;
    fn as_borrowed(&'view self) -> Self::View {
        RawDocument {
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
            owner: self.owner,
            root: self.root,
            version: self.version,
            profile_mask: self.profile_mask,
            options: self.options,
            source: self.source.as_ref().map(|s| Cow::Borrowed(s.as_ref())),
            dirty: self.dirty,
        }
    }
}
impl IntoOwned for RawDocument<'_> {
    type Owned = RawDocument<'static>;
    fn into_owned(self) -> Self::Owned {
        RawDocument::into_owned(self)
    }
}
impl<'view, D: Dialect> Borrowed<'view> for Document<'_, D> {
    type View = Document<'view, D>;
    fn as_borrowed(&'view self) -> Self::View {
        Document {
            raw: self.raw.as_borrowed(),
            dialect: PhantomData,
        }
    }
}
impl<D: Dialect> IntoOwned for Document<'_, D> {
    type Owned = Document<'static, D>;
    fn into_owned(self) -> Self::Owned {
        Document::into_owned(self)
    }
}

#[cfg(test)]
#[path = "document_tests.rs"]
mod tests;
