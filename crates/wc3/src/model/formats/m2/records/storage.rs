//! Flat and per-sequence animation storage selected by a record layout.
use super::Interpolation;
use super::{ArrayVisitor, Validate, VisitArrays};
use crate::model::{common::Ownership, formats::m2};
use m2::codec::{Array, ArrayRef, Decoder, Encoder, Value};
use m2::ModelLayout;
use m2::{Read, ReadError, RecordLayout, Write, WriteError};
use std::fmt::Debug;
use zerovec::{ZeroSlice, ZeroVec};

fn borrowed_leaf<'view, T: Value>(leaf: &'view Array<'_, T>) -> Array<'view, T> {
    match leaf {
        Array::Inline(v) => Array::Inline(v.as_slice().as_zerovec()),
        Array::External(r) => Array::External(*r),
    }
}
fn leaf_length<T: Value>(leaf: &Array<'_, T>) -> usize {
    match leaf {
        Array::Inline(v) => v.len(),
        Array::External(r) => r.count as usize,
    }
}
fn read_leaves<'data, T: Value>(
    input: &Decoder<'data>,
    reference: ArrayRef,
    global: u16,
) -> Result<Vec<Array<'data, T>>, ReadError> {
    if reference.count == 0 {
        return Ok(Vec::new());
    }
    let length = (reference.count as usize)
        .checked_mul(8)
        .ok_or_else(|| ReadError::invalid(input.position(), "track descriptor overflow"))?;
    input.at(reference.offset)?.bytes(length)?;
    let mut cursor = input.at(reference.offset)?;
    let mut leaves = Vec::with_capacity(reference.count as usize);
    for index in 0..reference.count as usize {
        let r = cursor.descriptor()?;
        let external = global == u16::MAX
            && input
                .external_sequences
                .get(index)
                .copied()
                .unwrap_or(input.unresolved_sequences);
        leaves.push(if external {
            Array::External(r)
        } else {
            Array::Inline(input.array(r)?)
        });
    }
    Ok(leaves)
}
fn write_leaf<T: Value>(output: &mut Encoder, leaf: &Array<'_, T>) -> Result<ArrayRef, WriteError> {
    match leaf {
        Array::Inline(v) => output.array(v),
        Array::External(r) => Ok(*r),
    }
}
fn write_leaves<T: Value>(
    output: &mut Encoder,
    leaves: &[Array<'_, T>],
) -> Result<ArrayRef, WriteError> {
    if leaves.is_empty() {
        return Ok(ArrayRef::default());
    }
    let count = u32::try_from(leaves.len()).map_err(|_| WriteError::invalid("track count"))?;
    output.align(16)?;
    let offset = output.reserve(
        leaves
            .len()
            .checked_mul(8)
            .ok_or_else(|| WriteError::invalid("track count overflow"))?,
    )?;
    for (index, leaf) in leaves.iter().enumerate() {
        let r = write_leaf(output, leaf)?;
        output.patch_u32(offset + index * 8, r.count)?;
        output.patch_u32(offset + index * 8 + 4, r.offset)?;
    }
    Ok(ArrayRef {
        count,
        offset: offset as u32,
    })
}
fn sorted<T: Value + PartialOrd>(values: &ZeroVec<'_, T>) -> bool {
    let mut previous = None;
    for value in values.iter() {
        if previous.is_some_and(|previous| value < previous) {
            return false;
        }
        previous = Some(value);
    }
    true
}

/// Timestamp storage of one native animation family.
pub trait TimelineStorage<'data>:
    Clone + Debug + PartialEq + Default + Write + RecordLayout + Validate
{
    /// Reads timestamp descriptors with the owning track's global-loop context.
    fn read_context(input: &mut Decoder<'data>, global: u16) -> Result<Self, ReadError>;
    /// Number of native timestamp leaves.
    fn len(&self) -> usize;
    /// Whether the storage has no leaves.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Borrows one encoded or external leaf.
    fn leaf(&self, index: usize) -> Option<Array<'_, u32>>;
    /// Borrows inline timestamp bytes directly.
    fn inline(&self, index: usize) -> Option<&ZeroSlice<u32>>;
    /// Legacy inclusive interpolation ranges, or an empty slice.
    fn ranges(&self) -> &ZeroSlice<[u32; 2]>;
    /// Replaces one native leaf without changing its family.
    fn replace(&mut self, index: usize, leaf: Array<'data, u32>) -> Result<(), ReadError>;
    /// Visits timestamp leaves with the owning global-loop context.
    fn visit_context<V: ArrayVisitor<'data>>(
        &mut self,
        global: u16,
        visitor: &mut V,
    ) -> Result<(), V::Error>;
}
/// Legacy flat timestamps and inclusive sequence windows.
#[derive(Clone, Debug, PartialEq, Default, Ownership, m2::Read, m2::Write, m2::RecordLayout)]
pub struct LegacyTimeline<'data> {
    /// Inclusive first/last indices into the flat timeline.
    pub ranges: ZeroVec<'data, [u32; 2]>,
    /// Absolute millisecond keys on the shared timeline.
    pub timestamps: ZeroVec<'data, u32>,
}
impl Validate for LegacyTimeline<'_> {
    fn validate(&self) -> Result<(), ReadError> {
        if !sorted(&self.timestamps) {
            return Err(ReadError::invalid(0, "track timestamps decrease"));
        }
        for [first, last] in self.ranges.iter() {
            if first > last
                || (self.timestamps.is_empty() && (first != 0 || last != 0))
                || (!self.timestamps.is_empty() && last as usize >= self.timestamps.len())
            {
                return Err(ReadError::invalid(0, "legacy track range"));
            }
        }
        Ok(())
    }
}
impl<'data> TimelineStorage<'data> for LegacyTimeline<'data> {
    fn read_context(input: &mut Decoder<'data>, _: u16) -> Result<Self, ReadError> {
        Self::read(input)
    }
    fn len(&self) -> usize {
        1
    }
    fn leaf(&self, index: usize) -> Option<Array<'_, u32>> {
        (index == 0).then(|| Array::Inline(self.timestamps.as_slice().as_zerovec()))
    }
    fn inline(&self, index: usize) -> Option<&ZeroSlice<u32>> {
        (index == 0).then(|| self.timestamps.as_slice())
    }
    fn ranges(&self) -> &ZeroSlice<[u32; 2]> {
        self.ranges.as_slice()
    }
    fn replace(&mut self, index: usize, leaf: Array<'data, u32>) -> Result<(), ReadError> {
        match (index, leaf) {
            (0, Array::Inline(v)) => {
                self.timestamps = v;
                Ok(())
            }
            _ => Err(ReadError::invalid(
                0,
                "legacy timestamps must remain inline",
            )),
        }
    }
    fn visit_context<V: ArrayVisitor<'data>>(&mut self, _: u16, _: &mut V) -> Result<(), V::Error> {
        Ok(())
    }
}
impl<'data> VisitArrays<'data> for LegacyTimeline<'data> {
    fn visit_arrays<V: ArrayVisitor<'data>>(&mut self, _: &mut V) -> Result<(), V::Error> {
        Ok(())
    }
}
/// Modern timestamp leaves, one per ordinary sequence or global loop.
#[derive(Clone, Debug, PartialEq, Default, Ownership)]
pub struct ModernTimeline<'data> {
    /// Per-sequence millisecond timestamp leaves.
    pub timestamps: Vec<Array<'data, u32>>,
}
impl Write for ModernTimeline<'_> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.reserve_metadata(8)?;
        let r = write_leaves(output, &self.timestamps)?;
        output.write_descriptor(r)
    }
}
impl RecordLayout for ModernTimeline<'_> {
    fn size(_: u32, _: u32) -> usize {
        8
    }
}
impl Validate for ModernTimeline<'_> {
    fn validate(&self) -> Result<(), ReadError> {
        for leaf in &self.timestamps {
            if let Array::Inline(v) = leaf {
                if !sorted(v) {
                    return Err(ReadError::invalid(0, "track timestamps decrease"));
                }
            }
        }
        Ok(())
    }
}
impl<'data> TimelineStorage<'data> for ModernTimeline<'data> {
    fn read_context(input: &mut Decoder<'data>, global: u16) -> Result<Self, ReadError> {
        let r = input.descriptor()?;
        Ok(Self {
            timestamps: read_leaves(input, r, global)?,
        })
    }
    fn len(&self) -> usize {
        self.timestamps.len()
    }
    fn leaf(&self, index: usize) -> Option<Array<'_, u32>> {
        self.timestamps.get(index).map(borrowed_leaf)
    }
    fn inline(&self, index: usize) -> Option<&ZeroSlice<u32>> {
        match self.timestamps.get(index) {
            Some(Array::Inline(v)) => Some(v.as_slice()),
            _ => None,
        }
    }
    fn ranges(&self) -> &ZeroSlice<[u32; 2]> {
        ZeroSlice::new_empty()
    }
    fn replace(&mut self, index: usize, leaf: Array<'data, u32>) -> Result<(), ReadError> {
        if self.timestamps.len() <= index {
            self.timestamps.resize_with(index + 1, Array::default);
        }
        self.timestamps[index] = leaf;
        Ok(())
    }
    fn visit_context<V: ArrayVisitor<'data>>(
        &mut self,
        global: u16,
        visitor: &mut V,
    ) -> Result<(), V::Error> {
        for (index, leaf) in self.timestamps.iter_mut().enumerate() {
            visitor.visit(index, global, leaf)?;
        }
        Ok(())
    }
}
impl<'data> VisitArrays<'data> for ModernTimeline<'data> {
    fn visit_arrays<V: ArrayVisitor<'data>>(&mut self, v: &mut V) -> Result<(), V::Error> {
        self.visit_context(u16::MAX, v)
    }
}

/// A borrowed native leaf retaining runtime spline selection.
#[derive(Clone, Debug, PartialEq)]
pub enum ValuesLeaf<'view, T: Value>
where
    [T; 3]: Value,
{
    /// Step or linear keys.
    Values(Array<'view, T>),
    /// Value/incoming/outgoing tangent triplets.
    Splines(Array<'view, [T; 3]>),
}
/// A directly borrowed inline value window.
#[derive(Clone, Copy, Debug)]
pub enum SequenceValues<'view, T: Value>
where
    [T; 3]: Value,
{
    /// Ordinary keys.
    Values(&'view ZeroSlice<T>),
    /// Native tangent triplets.
    Splines(&'view ZeroSlice<[T; 3]>),
}
/// Value storage selected by an animation layout family.
pub trait ValuesStorage<'data, T: Value>:
    Clone + Debug + PartialEq + Default + Write + RecordLayout
where
    [T; 3]: Value,
{
    /// Reads the value descriptor according to interpolation and global-loop context.
    fn read_context(
        input: &mut Decoder<'data>,
        interpolation: Interpolation,
        global: u16,
    ) -> Result<Self, ReadError>;
    /// Number of native value leaves.
    fn len(&self) -> usize;
    /// Whether no leaves exist.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Whether the runtime key representation stores tangent triplets.
    fn is_spline(&self) -> bool;
    /// Borrows one native leaf.
    fn leaf(&self, index: usize) -> Option<ValuesLeaf<'_, T>>;
    /// Borrows one inline leaf directly.
    fn inline(&self, index: usize) -> Option<SequenceValues<'_, T>>;
    /// Replaces a leaf, preserving the runtime interpolation shape.
    fn replace(&mut self, index: usize, leaf: ValuesLeaf<'data, T>) -> Result<(), ReadError>;
    /// Visits keys with the owning global-loop context.
    fn visit_context<V: ArrayVisitor<'data>>(
        &mut self,
        global: u16,
        visitor: &mut V,
    ) -> Result<(), V::Error>;
}
/// Legacy keys have one flat inline array; interpolation selects tangent storage.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum LegacyValues<'data, T: Value>
where
    [T; 3]: Value,
{
    /// Step or linear values.
    Values(ZeroVec<'data, T>),
    /// Native tangent triplets.
    Splines(ZeroVec<'data, [T; 3]>),
}
impl<T: Value> Default for LegacyValues<'_, T>
where
    [T; 3]: Value,
{
    fn default() -> Self {
        Self::Values(ZeroVec::new())
    }
}
impl<T: Value> RecordLayout for LegacyValues<'_, T>
where
    [T; 3]: Value,
{
    fn size(_: u32, _: u32) -> usize {
        8
    }
}
impl<T: Value> Write for LegacyValues<'_, T>
where
    [T; 3]: Value,
{
    fn write(&self, o: &mut Encoder) -> Result<(), WriteError> {
        match self {
            Self::Values(v) => v.write(o),
            Self::Splines(v) => v.write(o),
        }
    }
}
impl<'data, T: Value> ValuesStorage<'data, T> for LegacyValues<'data, T>
where
    [T; 3]: Value,
{
    fn read_context(i: &mut Decoder<'data>, p: Interpolation, _: u16) -> Result<Self, ReadError> {
        let r = i.descriptor()?;
        Ok(if p.spline() {
            Self::Splines(i.array(r)?)
        } else {
            Self::Values(i.array(r)?)
        })
    }
    fn len(&self) -> usize {
        1
    }
    fn is_spline(&self) -> bool {
        matches!(self, Self::Splines(_))
    }
    fn leaf(&self, n: usize) -> Option<ValuesLeaf<'_, T>> {
        if n != 0 {
            return None;
        }
        Some(match self {
            Self::Values(v) => ValuesLeaf::Values(Array::Inline(v.as_slice().as_zerovec())),
            Self::Splines(v) => ValuesLeaf::Splines(Array::Inline(v.as_slice().as_zerovec())),
        })
    }
    fn inline(&self, n: usize) -> Option<SequenceValues<'_, T>> {
        if n != 0 {
            return None;
        }
        Some(match self {
            Self::Values(v) => SequenceValues::Values(v.as_slice()),
            Self::Splines(v) => SequenceValues::Splines(v.as_slice()),
        })
    }
    fn replace(&mut self, n: usize, v: ValuesLeaf<'data, T>) -> Result<(), ReadError> {
        match (self, n, v) {
            (Self::Values(a), 0, ValuesLeaf::Values(Array::Inline(b))) => *a = b,
            (Self::Splines(a), 0, ValuesLeaf::Splines(Array::Inline(b))) => *a = b,
            _ => return Err(ReadError::invalid(0, "legacy value representation")),
        }
        Ok(())
    }
    fn visit_context<V: ArrayVisitor<'data>>(&mut self, _: u16, _: &mut V) -> Result<(), V::Error> {
        Ok(())
    }
}
/// Per-sequence modern keys; interpolation selects native tangent triplets.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum TrackValues<'data, T: Value>
where
    [T; 3]: Value,
{
    /// Ordinary keys per sequence.
    Values(Vec<Array<'data, T>>),
    /// Tangent triplets per sequence.
    Splines(Vec<Array<'data, [T; 3]>>),
}
impl<T: Value> Default for TrackValues<'_, T>
where
    [T; 3]: Value,
{
    fn default() -> Self {
        Self::Values(Vec::new())
    }
}
impl<T: Value> RecordLayout for TrackValues<'_, T>
where
    [T; 3]: Value,
{
    fn size(_: u32, _: u32) -> usize {
        8
    }
}
impl<T: Value> Write for TrackValues<'_, T>
where
    [T; 3]: Value,
{
    fn write(&self, o: &mut Encoder) -> Result<(), WriteError> {
        o.reserve_metadata(8)?;
        let r = match self {
            Self::Values(v) => write_leaves(o, v)?,
            Self::Splines(v) => write_leaves(o, v)?,
        };
        o.write_descriptor(r)
    }
}
impl<'data, T: Value> ValuesStorage<'data, T> for TrackValues<'data, T>
where
    [T; 3]: Value,
{
    fn read_context(i: &mut Decoder<'data>, p: Interpolation, g: u16) -> Result<Self, ReadError> {
        let r = i.descriptor()?;
        Ok(if p.spline() {
            Self::Splines(read_leaves(i, r, g)?)
        } else {
            Self::Values(read_leaves(i, r, g)?)
        })
    }
    fn len(&self) -> usize {
        match self {
            Self::Values(v) => v.len(),
            Self::Splines(v) => v.len(),
        }
    }
    fn is_spline(&self) -> bool {
        matches!(self, Self::Splines(_))
    }
    fn leaf(&self, n: usize) -> Option<ValuesLeaf<'_, T>> {
        match self {
            Self::Values(v) => v.get(n).map(|a| ValuesLeaf::Values(borrowed_leaf(a))),
            Self::Splines(v) => v.get(n).map(|a| ValuesLeaf::Splines(borrowed_leaf(a))),
        }
    }
    fn inline(&self, n: usize) -> Option<SequenceValues<'_, T>> {
        match self {
            Self::Values(v) => match v.get(n) {
                Some(Array::Inline(v)) => Some(SequenceValues::Values(v.as_slice())),
                _ => None,
            },
            Self::Splines(v) => match v.get(n) {
                Some(Array::Inline(v)) => Some(SequenceValues::Splines(v.as_slice())),
                _ => None,
            },
        }
    }
    fn replace(&mut self, n: usize, v: ValuesLeaf<'data, T>) -> Result<(), ReadError> {
        match (self, v) {
            (Self::Values(a), ValuesLeaf::Values(v)) => {
                if a.len() <= n {
                    a.resize_with(n + 1, Array::default);
                }
                a[n] = v
            }
            (Self::Splines(a), ValuesLeaf::Splines(v)) => {
                if a.len() <= n {
                    a.resize_with(n + 1, Array::default);
                }
                a[n] = v
            }
            _ => return Err(ReadError::invalid(0, "track value representation")),
        }
        Ok(())
    }
    fn visit_context<V: ArrayVisitor<'data>>(&mut self, g: u16, v: &mut V) -> Result<(), V::Error> {
        match self {
            Self::Values(a) => {
                for (n, a) in a.iter_mut().enumerate() {
                    v.visit(n, g, a)?;
                }
            }
            Self::Splines(a) => {
                for (n, a) in a.iter_mut().enumerate() {
                    v.visit(n, g, a)?;
                }
            }
        }
        Ok(())
    }
}

/// Camera spline storage selected independently of interpolation.
pub trait SplineStorage<'data, T: Value>:
    Clone + Debug + PartialEq + Default + Write + RecordLayout
where
    [T; 3]: Value,
{
    /// Reads native triplets with the owning global-loop context.
    fn read_context(i: &mut Decoder<'data>, global: u16) -> Result<Self, ReadError>;
    /// Number of native leaves.
    fn len(&self) -> usize;
    /// Whether no leaves exist.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Borrows one leaf.
    fn leaf(&self, index: usize) -> Option<Array<'_, [T; 3]>>;
    /// Replaces one leaf without changing its family.
    fn replace(&mut self, index: usize, leaf: Array<'data, [T; 3]>) -> Result<(), ReadError>;
    /// Visits each leaf with its global-loop context.
    fn visit_context<V: ArrayVisitor<'data>>(
        &mut self,
        global: u16,
        visitor: &mut V,
    ) -> Result<(), V::Error>;
}
/// One flat legacy spline-key array.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct LegacySplineValues<'data, T: Value>
where
    [T; 3]: Value,
{
    /// Native value/incoming/outgoing tangent triplets.
    pub values: ZeroVec<'data, [T; 3]>,
}
impl<T: Value> Default for LegacySplineValues<'_, T>
where
    [T; 3]: Value,
{
    fn default() -> Self {
        Self {
            values: ZeroVec::new(),
        }
    }
}
impl<T: Value> Write for LegacySplineValues<'_, T>
where
    [T; 3]: Value,
{
    fn write(&self, o: &mut Encoder) -> Result<(), WriteError> {
        self.values.write(o)
    }
}
impl<T: Value> RecordLayout for LegacySplineValues<'_, T>
where
    [T; 3]: Value,
{
    fn size(_: u32, _: u32) -> usize {
        8
    }
}
impl<'data, T: Value> SplineStorage<'data, T> for LegacySplineValues<'data, T>
where
    [T; 3]: Value,
{
    fn read_context(i: &mut Decoder<'data>, _: u16) -> Result<Self, ReadError> {
        let r = i.descriptor()?;
        Ok(Self {
            values: i.array(r)?,
        })
    }
    fn len(&self) -> usize {
        1
    }
    fn leaf(&self, n: usize) -> Option<Array<'_, [T; 3]>> {
        (n == 0).then(|| Array::Inline(self.values.as_slice().as_zerovec()))
    }
    fn replace(&mut self, n: usize, v: Array<'data, [T; 3]>) -> Result<(), ReadError> {
        match (n, v) {
            (0, Array::Inline(v)) => {
                self.values = v;
                Ok(())
            }
            _ => Err(ReadError::invalid(
                0,
                "legacy spline values must remain inline",
            )),
        }
    }
    fn visit_context<V: ArrayVisitor<'data>>(&mut self, _: u16, _: &mut V) -> Result<(), V::Error> {
        Ok(())
    }
}
/// Modern per-sequence spline-key arrays.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct ModernSplineValues<'data, T: Value>
where
    [T; 3]: Value,
{
    /// Native value/incoming/outgoing tangent leaves.
    pub values: Vec<Array<'data, [T; 3]>>,
}
impl<T: Value> Default for ModernSplineValues<'_, T>
where
    [T; 3]: Value,
{
    fn default() -> Self {
        Self { values: Vec::new() }
    }
}
impl<T: Value> Write for ModernSplineValues<'_, T>
where
    [T; 3]: Value,
{
    fn write(&self, o: &mut Encoder) -> Result<(), WriteError> {
        o.reserve_metadata(8)?;
        let r = write_leaves(o, &self.values)?;
        o.write_descriptor(r)
    }
}
impl<T: Value> RecordLayout for ModernSplineValues<'_, T>
where
    [T; 3]: Value,
{
    fn size(_: u32, _: u32) -> usize {
        8
    }
}
impl<'data, T: Value> SplineStorage<'data, T> for ModernSplineValues<'data, T>
where
    [T; 3]: Value,
{
    fn read_context(i: &mut Decoder<'data>, g: u16) -> Result<Self, ReadError> {
        let r = i.descriptor()?;
        Ok(Self {
            values: read_leaves(i, r, g)?,
        })
    }
    fn len(&self) -> usize {
        self.values.len()
    }
    fn leaf(&self, n: usize) -> Option<Array<'_, [T; 3]>> {
        self.values.get(n).map(borrowed_leaf)
    }
    fn replace(&mut self, n: usize, v: Array<'data, [T; 3]>) -> Result<(), ReadError> {
        if self.values.len() <= n {
            self.values.resize_with(n + 1, Array::default);
        }
        self.values[n] = v;
        Ok(())
    }
    fn visit_context<V: ArrayVisitor<'data>>(&mut self, g: u16, v: &mut V) -> Result<(), V::Error> {
        for (n, a) in self.values.iter_mut().enumerate() {
            v.visit(n, g, a)?;
        }
        Ok(())
    }
}

/// Timestamp-only native track with a statically selected timeline family.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct TrackBase<'data, V: ModelLayout> {
    /// Runtime interpolation operation: step, linear, Hermite, or Bezier.
    pub interpolation: Interpolation,
    /// Index into the owning core's `global_sequences` durations, or `0xffff` for ordinary sequences.
    pub global_sequence: u16,
    /// Legacy flat timeline or modern per-sequence leaves.
    pub timeline: V::Timeline<'data>,
}
impl<V: ModelLayout> Default for TrackBase<'_, V> {
    fn default() -> Self {
        Self {
            interpolation: Interpolation::Step,
            global_sequence: u16::MAX,
            timeline: Default::default(),
        }
    }
}
impl<'data, V: ModelLayout> Read<'data> for TrackBase<'data, V> {
    fn read(i: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let interpolation = Interpolation::from_raw(i.u16()?);
        let global_sequence = i.u16()?;
        Ok(Self {
            interpolation,
            global_sequence,
            timeline: V::Timeline::read_context(i, global_sequence)?,
        })
    }
}
impl<V: ModelLayout> Write for TrackBase<'_, V> {
    fn write(&self, o: &mut Encoder) -> Result<(), WriteError> {
        o.reserve_metadata(Self::size(o.version, o.flags.bits()))?;
        o.u16(self.interpolation.raw())?;
        o.u16(self.global_sequence)?;
        self.timeline.write(o)
    }
}
impl<V: ModelLayout> RecordLayout for TrackBase<'_, V> {
    fn size(v: u32, f: u32) -> usize {
        4 + V::Timeline::size(v, f)
    }
}
impl<V: ModelLayout> Validate for TrackBase<'_, V> {
    fn validate(&self) -> Result<(), ReadError> {
        self.timeline.validate()
    }
}
impl<'data, L: ModelLayout> VisitArrays<'data> for TrackBase<'data, L> {
    fn visit_arrays<V: ArrayVisitor<'data>>(&mut self, v: &mut V) -> Result<(), V::Error> {
        self.timeline.visit_context(self.global_sequence, v)
    }
}
/// Ordinary track whose timeline and leaf cardinality are selected by V.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct Track<'data, T: Value, V: ModelLayout>
where
    [T; 3]: Value,
{
    /// Timestamp metadata and selected timeline.
    pub base: TrackBase<'data, V>,
    /// Selected flat/per-sequence storage and runtime interpolation shape.
    pub values: V::TrackValues<'data, T>,
}
impl<T: Value, V: ModelLayout> Default for Track<'_, T, V>
where
    [T; 3]: Value,
{
    fn default() -> Self {
        Self {
            base: Default::default(),
            values: Default::default(),
        }
    }
}
impl<'data, T: Value, V: ModelLayout> Read<'data> for Track<'data, T, V>
where
    [T; 3]: Value,
{
    fn read(i: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let base = TrackBase::read(i)?;
        let values = V::TrackValues::read_context(i, base.interpolation, base.global_sequence)?;
        Ok(Self { base, values })
    }
}
impl<T: Value, V: ModelLayout> Write for Track<'_, T, V>
where
    [T; 3]: Value,
{
    fn write(&self, o: &mut Encoder) -> Result<(), WriteError> {
        if self.base.interpolation.spline() != self.values.is_spline() {
            return Err(WriteError::invalid(
                "track interpolation and key representation",
            ));
        }
        o.reserve_metadata(Self::size(o.version, o.flags.bits()))?;
        self.base.write(o)?;
        self.values.write(o)
    }
}
impl<T: Value, V: ModelLayout> RecordLayout for Track<'_, T, V>
where
    [T; 3]: Value,
{
    fn size(v: u32, f: u32) -> usize {
        TrackBase::<V>::size(v, f) + 8
    }
}
impl<T: Value, V: ModelLayout> Validate for Track<'_, T, V>
where
    [T; 3]: Value,
{
    fn validate(&self) -> Result<(), ReadError> {
        self.base.validate()?;
        if self.base.interpolation.spline() != self.values.is_spline() {
            return Err(ReadError::invalid(
                0,
                "track interpolation and key representation",
            ));
        }
        if self.base.timeline.len() != self.values.len() {
            return Err(ReadError::invalid(0, "track timeline counts"));
        }
        for n in 0..self.values.len() {
            let times = self
                .base
                .timeline
                .leaf(n)
                .ok_or_else(|| ReadError::invalid(0, "timestamp leaf"))?;
            let values = self
                .values
                .leaf(n)
                .ok_or_else(|| ReadError::invalid(0, "value leaf"))?;
            let count = match values {
                ValuesLeaf::Values(v) => leaf_length(&v),
                ValuesLeaf::Splines(v) => leaf_length(&v),
            };
            if leaf_length(&times) != count {
                return Err(ReadError::invalid(0, "track timestamp/value key counts"));
            }
        }
        Ok(())
    }
}
impl<'data, T: Value, L: ModelLayout> VisitArrays<'data> for Track<'data, T, L>
where
    [T; 3]: Value,
{
    fn visit_arrays<V: ArrayVisitor<'data>>(&mut self, v: &mut V) -> Result<(), V::Error> {
        self.base.visit_arrays(v)?;
        self.values.visit_context(self.base.global_sequence, v)
    }
}
/// Native camera spline track; keys are always tangent triplets.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct SplineTrack<'data, T: Value, V: ModelLayout>
where
    [T; 3]: Value,
{
    /// Runtime interpolation and selected timestamp family.
    pub base: TrackBase<'data, V>,
    /// Selected flat or per-sequence spline storage.
    pub values: V::SplineValues<'data, T>,
}
impl<T: Value, V: ModelLayout> Default for SplineTrack<'_, T, V>
where
    [T; 3]: Value,
{
    fn default() -> Self {
        Self {
            base: Default::default(),
            values: Default::default(),
        }
    }
}
impl<'data, T: Value, V: ModelLayout> Read<'data> for SplineTrack<'data, T, V>
where
    [T; 3]: Value,
{
    fn read(i: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let base = TrackBase::read(i)?;
        let values = V::SplineValues::read_context(i, base.global_sequence)?;
        Ok(Self { base, values })
    }
}
impl<T: Value, V: ModelLayout> Write for SplineTrack<'_, T, V>
where
    [T; 3]: Value,
{
    fn write(&self, o: &mut Encoder) -> Result<(), WriteError> {
        o.reserve_metadata(Self::size(o.version, o.flags.bits()))?;
        self.base.write(o)?;
        self.values.write(o)
    }
}
impl<T: Value, V: ModelLayout> RecordLayout for SplineTrack<'_, T, V>
where
    [T; 3]: Value,
{
    fn size(v: u32, f: u32) -> usize {
        TrackBase::<V>::size(v, f) + 8
    }
}
impl<T: Value, V: ModelLayout> Validate for SplineTrack<'_, T, V>
where
    [T; 3]: Value,
{
    fn validate(&self) -> Result<(), ReadError> {
        self.base.validate()?;
        if self.base.timeline.len() != self.values.len() {
            return Err(ReadError::invalid(0, "spline timeline counts"));
        }
        for n in 0..self.values.len() {
            if leaf_length(
                &self
                    .base
                    .timeline
                    .leaf(n)
                    .ok_or_else(|| ReadError::invalid(0, "timestamp leaf"))?,
            ) != leaf_length(
                &self
                    .values
                    .leaf(n)
                    .ok_or_else(|| ReadError::invalid(0, "spline leaf"))?,
            ) {
                return Err(ReadError::invalid(0, "spline timestamp/value counts"));
            }
        }
        Ok(())
    }
}
impl<'data, T: Value, L: ModelLayout> VisitArrays<'data> for SplineTrack<'data, T, L>
where
    [T; 3]: Value,
{
    fn visit_arrays<V: ArrayVisitor<'data>>(&mut self, v: &mut V) -> Result<(), V::Error> {
        self.base.visit_arrays(v)?;
        self.values.visit_context(self.base.global_sequence, v)
    }
}
/// Particle-age track independent of model sequence layouts.
#[derive(Clone, Debug, PartialEq, Ownership, m2::Read, m2::Write, m2::RecordLayout)]
pub struct ParticleTrack<'data, T: Value> {
    /// Normalized particle ages: zero is spawn and `0xffff` is the end of life.
    pub timestamps: ZeroVec<'data, u16>,
    /// Property keys at the corresponding ages, shared by all model sequences.
    pub values: ZeroVec<'data, T>,
}
impl<T: Value> Default for ParticleTrack<'_, T> {
    fn default() -> Self {
        Self {
            timestamps: ZeroVec::new(),
            values: ZeroVec::new(),
        }
    }
}
impl<T: Value> Validate for ParticleTrack<'_, T> {
    fn validate(&self) -> Result<(), ReadError> {
        if self.timestamps.len() != self.values.len() || !sorted(&self.timestamps) {
            return Err(ReadError::invalid(0, "particle lifetime keys"));
        }
        Ok(())
    }
}
impl<'data, T: Value> VisitArrays<'data> for ParticleTrack<'data, T> {
    fn visit_arrays<V: ArrayVisitor<'data>>(&mut self, _: &mut V) -> Result<(), V::Error> {
        Ok(())
    }
}

/// A directly borrowed sequence window with native, unre-based timestamps.
#[derive(Clone, Copy, Debug)]
pub struct TrackSequence<'view, T: Value>
where
    [T; 3]: Value,
{
    /// Native millisecond timestamps.
    pub timestamps: &'view ZeroSlice<u32>,
    /// Ordinary values or spline triplets in the same window.
    pub values: SequenceValues<'view, T>,
}
impl<T: Value, V: ModelLayout> Track<'_, T, V>
where
    [T; 3]: Value,
{
    /// Borrows an inline sequence window without allocating or rebasing its clock.
    /// External leaves require a bundle; global-loop tracks ignore legacy ranges.
    pub fn sequence(&self, index: usize) -> Result<TrackSequence<'_, T>, ReadError> {
        self.validate()?;
        let global = self.base.global_sequence != u16::MAX;
        let index = if global { 0 } else { index };
        let flat = !global && !self.base.timeline.ranges().is_empty();
        let leaf = if flat { 0 } else { index };
        let times = self.base.timeline.inline(leaf).ok_or_else(|| {
            ReadError::invalid(0, "sequence timestamps require external resolution")
        })?;
        let range = if flat {
            let [first, last] = self
                .base
                .timeline
                .ranges()
                .get(index)
                .ok_or_else(|| ReadError::invalid(0, "legacy sequence range index"))?;
            if times.is_empty() {
                0..0
            } else {
                first as usize..last as usize + 1
            }
        } else {
            0..times.len()
        };
        let timestamps = times
            .get_subslice(range.clone())
            .ok_or_else(|| ReadError::invalid(0, "sequence timestamp window"))?;
        let values =
            match self.values.inline(leaf).ok_or_else(|| {
                ReadError::invalid(0, "sequence values require external resolution")
            })? {
                SequenceValues::Values(v) => SequenceValues::Values(
                    v.get_subslice(range)
                        .ok_or_else(|| ReadError::invalid(0, "sequence value window"))?,
                ),
                SequenceValues::Splines(v) => SequenceValues::Splines(
                    v.get_subslice(range)
                        .ok_or_else(|| ReadError::invalid(0, "sequence tangent window"))?,
                ),
            };
        Ok(TrackSequence { timestamps, values })
    }
}
