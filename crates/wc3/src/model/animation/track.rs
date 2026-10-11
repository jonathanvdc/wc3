//! Typed keyframe tracks.
use super::{
    AnimationTime, Interpolate, Interpolation, Keyframe, TangentKeyframe, TrackValue, ValueKeyframe,
};
use crate::model::mdl::{Parser, TokenKind, Writer};
use crate::model::mdx::read_array;
use crate::model::mdx::read_array_layout;
use crate::model::IoError;
use crate::model::ModelOwnership;
use crate::model::{mdl, mdx};
use crate::model::{Cursor, Encoder, ValueError, ZeroVec};
use std::io::Write as IoWrite;
use std::iter::once;
use std::ops::{Range, RangeInclusive};
use zerovec::ule::AsULE;

#[derive(Clone, Debug, PartialEq, ModelOwnership)]
enum Keyframes<'data, T: TrackValue> {
    Step(ZeroVec<'data, ValueKeyframe<T>>),
    Linear(ZeroVec<'data, ValueKeyframe<T>>),
    Hermite(ZeroVec<'data, TangentKeyframe<T>>),
    Bezier(ZeroVec<'data, TangentKeyframe<T>>),
}

/// Keyframes for one animated property.
///
/// The type `T` selects the keyframe value type. Constructors select the
/// interpolation and matching keyframe shape. Times are in milliseconds;
/// `global_sequence_id` is an index into the model global-sequence collection,
/// or `None` to use the current model sequence.
///
/// Constructors and readers stably sort keys by ascending frame time. Duplicate
/// timestamps retain their input order, and evaluation uses the last key at each
/// timestamp. Writing a parsed track emits this sorted order.
#[derive(Clone, Debug, PartialEq, ModelOwnership)]
pub struct Track<'data, T: TrackValue> {
    global_sequence_id: Option<u32>,
    keyframes: Keyframes<'data, T>,
}

impl<'data, T: TrackValue> Track<'data, T> {
    /// A constant animation represented by one stepped key at time zero.
    pub fn constant(value: T) -> Self {
        Self::step(once(ValueKeyframe { frame: 0, value }).collect(), None)
            .expect("one key without a global sequence is valid")
    }

    /// Creates stepped animation: each key value holds until the next key.
    pub fn step(
        keys: ZeroVec<'data, ValueKeyframe<T>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Step(keys), global_sequence_id)
    }
    /// Creates animation that interpolates linearly between adjacent key values.
    pub fn linear(
        keys: ZeroVec<'data, ValueKeyframe<T>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Linear(keys), global_sequence_id)
    }
    /// Creates a Hermite track whose keys each have two tangents.
    pub fn hermite(
        keys: ZeroVec<'data, TangentKeyframe<T>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Hermite(keys), global_sequence_id)
    }
    /// Creates a Bezier track whose keys each have two tangents.
    pub fn bezier(
        keys: ZeroVec<'data, TangentKeyframe<T>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Bezier(keys), global_sequence_id)
    }
    fn new(
        keyframes: Keyframes<'data, T>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        if keyframes.len() > u32::MAX as usize {
            return Err(ValueError::CountTooLarge {
                tag: *b"TRAK",
                count: keyframes.len(),
            });
        }
        if global_sequence_id == Some(u32::MAX) {
            return Err(ValueError::InvalidGlobalSequenceId { id: u32::MAX });
        }
        Ok(Self {
            global_sequence_id,
            keyframes: keyframes.into_sorted(),
        })
    }
    /// Returns the optional global sequence index.
    pub fn global_sequence_id(&self) -> Option<u32> {
        self.global_sequence_id
    }
    /// Returns the interpolation mode shared by all keys.
    pub fn interpolation(&self) -> Interpolation {
        match self.keyframes {
            Keyframes::Step(_) => Interpolation::Step,
            Keyframes::Linear(_) => Interpolation::Linear,
            Keyframes::Hermite(_) => Interpolation::Hermite,
            Keyframes::Bezier(_) => Interpolation::Bezier,
        }
    }
    /// Returns keys in ascending frame order only for stepped interpolation.
    pub fn step_keys(&self) -> Option<&ZeroVec<'data, ValueKeyframe<T>>> {
        match &self.keyframes {
            Keyframes::Step(keys) => Some(keys),
            _ => None,
        }
    }
    /// Returns keys in ascending frame order only for linear interpolation.
    pub fn linear_keys(&self) -> Option<&ZeroVec<'data, ValueKeyframe<T>>> {
        match &self.keyframes {
            Keyframes::Linear(keys) => Some(keys),
            _ => None,
        }
    }
    /// Returns keys and tangents in ascending frame order only for Hermite interpolation.
    pub fn hermite_keys(&self) -> Option<&ZeroVec<'data, TangentKeyframe<T>>> {
        match &self.keyframes {
            Keyframes::Hermite(keys) => Some(keys),
            _ => None,
        }
    }
    /// Returns keys and tangents in ascending frame order only for Bezier interpolation.
    pub fn bezier_keys(&self) -> Option<&ZeroVec<'data, TangentKeyframe<T>>> {
        match &self.keyframes {
            Keyframes::Bezier(keys) => Some(keys),
            _ => None,
        }
    }
}
impl<'data, T: TrackValue> Keyframes<'data, T> {
    fn into_sorted(mut self) -> Self {
        match &mut self {
            Self::Step(keys) | Self::Linear(keys) => sort_keys(keys),
            Self::Hermite(keys) | Self::Bezier(keys) => sort_keys(keys),
        }
        self
    }

    fn len(&self) -> usize {
        match self {
            Self::Step(v) | Self::Linear(v) => v.len(),
            Self::Hermite(v) | Self::Bezier(v) => v.len(),
        }
    }
    fn interpolation(&self) -> u32 {
        match self {
            Self::Step(_) => 0,
            Self::Linear(_) => 1,
            Self::Hermite(_) => 2,
            Self::Bezier(_) => 3,
        }
    }
}
impl<'data, T: TrackValue> mdx::Read<'data> for Track<'data, T> {
    fn read_mdx(cursor: &mut Cursor<'data>) -> Result<Self, mdx::ReadError> {
        let mut next = *cursor;
        let offset = next.absolute_position();
        let overflow = || mdx::ReadError::new(offset, mdx::ReadErrorKind::SizeOverflow);
        let count = next.read::<u32>()? as usize;
        let interpolation = next.read::<u32>()?;
        if interpolation > 3 {
            return Err(mdx::ReadError::new(
                offset + 4,
                mdx::ReadErrorKind::UnknownEnumValue {
                    enum_name: "Interpolation",
                    value: interpolation,
                },
            ));
        }
        let sequence = next.read::<u32>()?;
        let components = T::COMPONENTS;
        let vector_count = if interpolation >= 2 { 3usize } else { 1usize };
        let key_size = components
            .checked_mul(vector_count)
            .and_then(|n| n.checked_mul(4))
            .and_then(|n| n.checked_add(4))
            .ok_or_else(overflow)?;
        let body_size = count.checked_mul(key_size).ok_or_else(overflow)?;
        let body = next.subcursor(body_size)?;
        let keyframes = match interpolation {
            0 | 1 => {
                let keys = if T::ALL_BITS_VALID {
                    read_array_layout(body, key_size)?
                } else {
                    read_array(body, key_size)?
                };
                if interpolation == 0 {
                    Keyframes::Step(keys)
                } else {
                    Keyframes::Linear(keys)
                }
            }
            2 | 3 => {
                let keys = if T::ALL_BITS_VALID {
                    read_array_layout(body, key_size)?
                } else {
                    read_array(body, key_size)?
                };
                if interpolation == 2 {
                    Keyframes::Hermite(keys)
                } else {
                    Keyframes::Bezier(keys)
                }
            }
            _ => unreachable!(),
        };
        *cursor = next;
        Ok(Self {
            global_sequence_id: (sequence != u32::MAX).then_some(sequence),
            keyframes: keyframes.into_sorted(),
        })
    }
}
impl<'data, T: TrackValue> mdx::Write for Track<'data, T> {
    fn write_mdx(&self, encoder: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        if T::COMPONENTS.checked_mul(4) != Some(size_of::<T::ULE>()) {
            return Err(mdx::WriteError::InvalidValue {
                field: "track value layout",
                tag: *b"TRAK",
            });
        }
        encoder.write(&(self.keyframes.len() as u32))?;
        encoder.write(&(self.keyframes.interpolation()))?;
        encoder.write(&(self.global_sequence_id.unwrap_or(u32::MAX)))?;
        match &self.keyframes {
            Keyframes::Step(keys) | Keyframes::Linear(keys) => {
                encoder.write_bytes(keys.as_bytes());
            }
            Keyframes::Hermite(keys) | Keyframes::Bezier(keys) => {
                encoder.write_bytes(keys.as_bytes());
            }
        }
        Ok(())
    }
}

impl<'data, T: TrackValue> mdl::Read for Track<'data, T>
where
    T: mdl::Read,
{
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        parser.expect_ident("Track")?;
        Self::read_mdl_payload(parser)
    }
}
impl<'data, T: TrackValue> Track<'data, T>
where
    T: mdl::Read,
{
    /// Reads the ordinary track grammar under an enclosing record's alias.
    pub fn read_mdl_named(
        parser: &mut Parser<'_>,
        name: &'static str,
    ) -> Result<Self, mdl::ReadError> {
        parser.expect_ident(name)?;
        Self::read_mdl_payload(parser)
    }

    /// Reads a counted MDL track body after its property name.
    /// The payload includes the key count, interpolation mode, and optional global sequence.
    pub fn read_mdl_payload(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        let count = parser.read::<u32>()? as usize;
        parser.expect(TokenKind::OpenBrace)?;
        let mut interpolation = None;
        let mut sequence = None;
        while let Some(token) = parser.peek()? {
            let TokenKind::Ident(name) = token.kind else {
                break;
            };
            if name == "GlobalSeqId" {
                parser.next_token()?;
                if sequence.is_some() {
                    return Err(mdl::ReadError::new(
                        token.span,
                        mdl::ReadErrorKind::DuplicateField,
                    ));
                }
                parser.peek()?;
                let span = parser
                    .error(mdl::ReadErrorKind::InvalidNumber("global sequence ID"))
                    .span;
                let id = parser.read_property::<u32>()?;
                if id == u32::MAX {
                    return Err(mdl::ReadError::new(
                        span,
                        mdl::ReadErrorKind::InvalidNumber("global sequence ID"),
                    ));
                }
                sequence = Some(id);
            } else {
                let mode = parser.read::<Interpolation>()?;
                if interpolation.replace(mode).is_some() {
                    return Err(mdl::ReadError::new(
                        token.span,
                        mdl::ReadErrorKind::DuplicateField,
                    ));
                }
                parser.expect(TokenKind::Comma)?;
            }
        }
        let interpolation = interpolation
            .ok_or_else(|| parser.error(mdl::ReadErrorKind::MissingField("interpolation")))?;
        // Grow from actual input, rather than trusting the declared count.
        let mut values = Vec::new();
        let mut tangents = Vec::new();
        let mut actual = 0;
        loop {
            if let Some(token) = parser.peek()? {
                if token.kind == TokenKind::CloseBrace {
                    if actual != count {
                        return Err(mdl::ReadError::new(
                            token.span,
                            mdl::ReadErrorKind::CountMismatch {
                                expected: count,
                                actual,
                            },
                        ));
                    }
                    parser.next_token()?;
                    break;
                }
            }
            if actual == count {
                return Err(parser.error(mdl::ReadErrorKind::CountMismatch {
                    expected: count,
                    actual: actual + 1,
                }));
            }
            let frame = parser.read::<i32>()?;
            parser.expect(TokenKind::Colon)?;
            let value = parser.read_property::<T>()?;
            if matches!(
                interpolation,
                Interpolation::Hermite | Interpolation::Bezier
            ) {
                parser.expect_ident("InTan")?;
                let in_tangent = parser.read_property()?;
                parser.expect_ident("OutTan")?;
                let out_tangent = parser.read_property()?;
                tangents.push(
                    TangentKeyframe {
                        frame,
                        value,
                        in_tangent,
                        out_tangent,
                    }
                    .to_unaligned(),
                );
            } else {
                values.push(ValueKeyframe { frame, value }.to_unaligned());
            }
            actual += 1;
        }
        let keyframes = match interpolation {
            Interpolation::Step => Keyframes::Step(ZeroVec::new_owned(values)),
            Interpolation::Linear => Keyframes::Linear(ZeroVec::new_owned(values)),
            Interpolation::Hermite => Keyframes::Hermite(ZeroVec::new_owned(tangents)),
            Interpolation::Bezier => Keyframes::Bezier(ZeroVec::new_owned(tangents)),
        };
        Ok(Self {
            global_sequence_id: sequence,
            keyframes: keyframes.into_sorted(),
        })
    }
}

impl<'data, T: TrackValue> mdl::Write for Track<'data, T>
where
    T: mdl::Write,
{
    fn write_mdl<W: IoWrite>(
        &self,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        self.write_mdl_named(writer, "Track")
    }
}
impl<'data, T: TrackValue> Track<'data, T>
where
    T: mdl::Write,
{
    /// Writes a borrowed track under an enclosing record's alias.
    pub fn write_mdl_named<W: IoWrite>(
        &self,
        writer: &mut Writer<W>,
        name: &str,
    ) -> Result<(), IoError<mdl::WriteError>> {
        writer.begin_counted_block(name, self.keyframes.len())?;
        writer.indent()?;
        writer.write(&self.interpolation())?;
        writer.raw(",\n")?;
        if let Some(sequence) = self.global_sequence_id {
            writer.property("GlobalSeqId", &sequence)?;
        }
        match &self.keyframes {
            Keyframes::Step(keys) | Keyframes::Linear(keys) => {
                for key in keys.iter() {
                    write_key(writer, key.frame, &key.value)?;
                }
            }
            Keyframes::Hermite(keys) | Keyframes::Bezier(keys) => {
                for key in keys.iter() {
                    write_key(writer, key.frame, &key.value)?;
                    write_tangent(writer, "InTan", &key.in_tangent)?;
                    write_tangent(writer, "OutTan", &key.out_tangent)?;
                }
            }
        }
        writer.end_block()
    }
}

fn write_key<W: IoWrite, T: mdl::Write>(
    writer: &mut Writer<W>,
    frame: i32,
    value: &T,
) -> Result<(), IoError<mdl::WriteError>> {
    writer.indent()?;
    writer.write(&frame)?;
    writer.raw(": ")?;
    writer.write(value)?;
    writer.raw(",\n")
}

fn write_tangent<W: IoWrite, T: mdl::Write>(
    writer: &mut Writer<W>,
    name: &str,
    value: &T,
) -> Result<(), IoError<mdl::WriteError>> {
    writer.indent()?;
    writer.raw("\t")?;
    writer.identifier(name)?;
    writer.raw(" ")?;
    writer.write(value)?;
    writer.raw(",\n")
}

impl<'data, T: TrackValue> Track<'data, T> {
    /// Returns the latest key timestamp at or before an absolute track time.
    ///
    /// Lookup takes O(log n) time. Empty tracks, nonfinite times, and times before
    /// the first key return `None`. Callers resolve clocks and looping.
    pub fn key_frame_at_or_before(&self, time_ms: f64) -> Option<i32> {
        self.key_frame_at_or_before_in(time_ms, i32::MIN..=i32::MAX)
    }

    /// Returns the latest key timestamp at or before an absolute track time,
    /// considering only keys within the inclusive sequence interval.
    pub fn key_frame_at_or_before_in(
        &self,
        time_ms: f64,
        interval: RangeInclusive<i32>,
    ) -> Option<i32> {
        if !time_ms.is_finite() || interval.is_empty() {
            return None;
        }
        match &self.keyframes {
            Keyframes::Step(keys) | Keyframes::Linear(keys) => {
                latest_frame(keys, time_ms, &interval)
            }
            Keyframes::Hermite(keys) | Keyframes::Bezier(keys) => {
                latest_frame(keys, time_ms, &interval)
            }
        }
    }
}

// Encoded ValueKeyframe and TangentKeyframe arrays both start each key with
// an i32 timestamp. Validation has already checked their complete wire layout.
fn key_frame<'data, K: Keyframe + AsULE>(keys: &ZeroVec<'data, K>, index: usize) -> i32 {
    let offset = index * size_of::<K::ULE>();
    i32::from_le_bytes(
        keys.as_bytes()[offset..offset + 4]
            .try_into()
            .expect("validated timestamp"),
    )
}
fn sort_keys<'data, K: Keyframe + AsULE>(keys: &mut ZeroVec<'data, K>) {
    let sorted = (1..keys.len()).all(|index| key_frame(keys, index - 1) <= key_frame(keys, index));
    if !sorted {
        keys.with_mut(|values| values.sort_by_key(|value| K::from_unaligned(*value).frame()));
    }
}
fn partition<'data, K: Keyframe + AsULE>(
    keys: &ZeroVec<'data, K>,
    range: Range<usize>,
    predicate: impl Fn(i32) -> bool,
) -> usize {
    let mut start = range.start;
    let mut end = range.end;
    while start < end {
        let middle = start + (end - start) / 2;
        if predicate(key_frame(keys, middle)) {
            start = middle + 1;
        } else {
            end = middle;
        }
    }
    start
}
fn key_range<'data, K: Keyframe + AsULE>(
    keys: &ZeroVec<'data, K>,
    interval: &RangeInclusive<i32>,
) -> (usize, usize) {
    let start = partition(keys, 0..keys.len(), |frame| frame < *interval.start());
    let end = partition(keys, start..keys.len(), |frame| frame <= *interval.end());
    (start, end)
}
fn latest_frame<'data, K: Keyframe + AsULE>(
    keys: &ZeroVec<'data, K>,
    time_ms: f64,
    interval: &RangeInclusive<i32>,
) -> Option<i32> {
    let (start, end) = key_range(keys, interval);
    let after = partition(keys, start..end, |frame| f64::from(frame) <= time_ms);
    (after > start).then(|| key_frame(keys, after - 1))
}

impl<'data, T: Interpolate> Track<'data, T> {
    /// Samples this track using its model or global sequence clock.
    pub fn sample(&self, time: &AnimationTime<'_>) -> Option<T> {
        let (time_ms, interval) = time.track_time(self)?;
        self.evaluate_in(time_ms, interval)
    }

    /// Samples at a time in milliseconds in this track's timeline.
    ///
    /// Empty tracks and nonfinite times return `None`. Outside the key range,
    /// the nearest endpoint is returned. At duplicate timestamps the last
    /// stored key wins. Evaluation takes O(log n) time.
    /// Global sequence IDs do not affect sampling: callers resolve clocks and
    /// looping before calling this method.
    pub fn evaluate(&self, time_ms: f64) -> Option<T> {
        self.evaluate_in(time_ms, i32::MIN..=i32::MAX)
    }

    /// Samples using only keys within the inclusive sequence interval.
    ///
    /// Returns `None` when the interval contains no keys or is reversed.
    /// Time is an absolute track timestamp, not an offset from the interval's
    /// start. Endpoint clamping and duplicate handling match [`Self::evaluate`].
    pub fn evaluate_in(&self, time_ms: f64, interval: RangeInclusive<i32>) -> Option<T> {
        if !time_ms.is_finite() || interval.is_empty() {
            return None;
        }
        match &self.keyframes {
            Keyframes::Step(keys) | Keyframes::Linear(keys) => {
                let (a, b) = surrounding(keys, time_ms, &interval)?;
                if a.frame == b.frame || matches!(self.keyframes, Keyframes::Step(_)) {
                    return Some(a.value);
                }
                let t = ((time_ms - f64::from(a.frame)) / (f64::from(b.frame) - f64::from(a.frame)))
                    as f32;
                Some(T::linear(a.value, b.value, t))
            }
            Keyframes::Hermite(keys) | Keyframes::Bezier(keys) => {
                let (a, b) = surrounding(keys, time_ms, &interval)?;
                if a.frame == b.frame {
                    return Some(a.value);
                }
                let t = ((time_ms - f64::from(a.frame)) / (f64::from(b.frame) - f64::from(a.frame)))
                    as f32;
                Some(if matches!(self.keyframes, Keyframes::Hermite(_)) {
                    T::hermite(a.value, a.out_tangent, b.in_tangent, b.value, t)
                } else {
                    T::bezier(a.value, a.out_tangent, b.in_tangent, b.value, t)
                })
            }
        }
    }
}

fn surrounding<'data, K: Keyframe + AsULE>(
    keys: &ZeroVec<'data, K>,
    time: f64,
    interval: &RangeInclusive<i32>,
) -> Option<(K, K)> {
    let (start, end) = key_range(keys, interval);
    if start == end {
        return None;
    }
    let after = partition(keys, start..end, |frame| f64::from(frame) <= time);
    let left = (after > start).then(|| keys.get(after - 1).expect("key index"));
    let right = if left
        .as_ref()
        .is_some_and(|key| f64::from(key.frame()) == time)
    {
        left
    } else if after < end {
        let frame = keys.get(after).expect("key index").frame();
        let next = partition(keys, after..end, |key_frame| key_frame <= frame);
        keys.get(next - 1)
    } else {
        None
    };
    Some((left.or(right)?, right.or(left)?))
}

#[cfg(test)]
mod key_lookup_tests {
    use super::*;

    #[test]
    fn lookup_respects_time_and_sequence_for_unordered_keys() {
        let track = Track::step(
            (vec![
                ValueKeyframe {
                    frame: 200,
                    value: 1.0,
                },
                ValueKeyframe {
                    frame: 50,
                    value: 2.0,
                },
                ValueKeyframe {
                    frame: 150,
                    value: 3.0,
                },
            ])
            .into_iter()
            .collect(),
            None,
        )
        .unwrap();
        assert_eq!(track.key_frame_at_or_before(175.0), Some(150));
        assert_eq!(track.key_frame_at_or_before(25.0), None);
        assert_eq!(track.key_frame_at_or_before_in(125.0, 100..=200), None);
        assert_eq!(track.key_frame_at_or_before_in(250.0, 100..=175), Some(150));
        assert_eq!(track.key_frame_at_or_before(f64::NAN), None);
    }
}
