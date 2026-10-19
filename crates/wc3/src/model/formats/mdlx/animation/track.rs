//! Typed keyframe tracks.
use super::{Interpolation, TangentKeyframe, TrackValue, ValueKeyframe};
use crate::model::common::Ownership;
use crate::model::formats::mdlx::mdl::{Parser, TokenKind, Writer};
use crate::model::formats::mdlx::mdx::read_array;
use crate::model::formats::mdlx::mdx::read_array_layout;
use crate::model::formats::mdlx::IoError;
use crate::model::formats::mdlx::{mdl, mdx};
use crate::model::formats::mdlx::{Cursor, Encoder, IntoZeroVec, ValueError, ZeroVec};
use std::io::Write as IoWrite;
use zerovec::ule::AsULE;

#[derive(Clone, Debug, PartialEq, Ownership, mdx::ReadOwned)]
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
/// Constructors and readers preserve key order and duplicate timestamps exactly.
/// Key accessors expose encoded storage without sorting or evaluating it.
#[derive(Clone, Debug, PartialEq, Ownership, mdx::ReadOwned)]
pub struct Track<'data, T: TrackValue> {
    global_sequence_id: Option<u32>,
    keyframes: Keyframes<'data, T>,
}

impl<'data, T: TrackValue> Track<'data, T> {
    /// A constant animation represented by one stepped key at time zero.
    pub fn constant(value: T) -> Self {
        Self::step([ValueKeyframe { frame: 0, value }], None)
            .expect("one key without a global sequence is valid")
    }

    /// Creates stepped animation: each key value holds until the next key.
    pub fn step(
        keys: impl IntoZeroVec<'data, ValueKeyframe<T>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Step(keys.into_zero_vec()), global_sequence_id)
    }
    /// Creates animation that interpolates linearly between adjacent key values.
    pub fn linear(
        keys: impl IntoZeroVec<'data, ValueKeyframe<T>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Linear(keys.into_zero_vec()), global_sequence_id)
    }
    /// Creates a Hermite track whose keys each have two tangents.
    pub fn hermite(
        keys: impl IntoZeroVec<'data, TangentKeyframe<T>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Hermite(keys.into_zero_vec()), global_sequence_id)
    }
    /// Creates a Bezier track whose keys each have two tangents.
    pub fn bezier(
        keys: impl IntoZeroVec<'data, TangentKeyframe<T>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Bezier(keys.into_zero_vec()), global_sequence_id)
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
            keyframes,
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
    /// Returns keys in stored order only for stepped interpolation.
    pub fn step_keys(&self) -> Option<&ZeroVec<'data, ValueKeyframe<T>>> {
        match &self.keyframes {
            Keyframes::Step(keys) => Some(keys),
            _ => None,
        }
    }
    /// Returns keys in stored order only for linear interpolation.
    pub fn linear_keys(&self) -> Option<&ZeroVec<'data, ValueKeyframe<T>>> {
        match &self.keyframes {
            Keyframes::Linear(keys) => Some(keys),
            _ => None,
        }
    }
    /// Returns keys and tangents in stored order only for Hermite interpolation.
    pub fn hermite_keys(&self) -> Option<&ZeroVec<'data, TangentKeyframe<T>>> {
        match &self.keyframes {
            Keyframes::Hermite(keys) => Some(keys),
            _ => None,
        }
    }
    /// Returns keys and tangents in stored order only for Bezier interpolation.
    pub fn bezier_keys(&self) -> Option<&ZeroVec<'data, TangentKeyframe<T>>> {
        match &self.keyframes {
            Keyframes::Bezier(keys) => Some(keys),
            _ => None,
        }
    }
}
impl<'data, T: TrackValue> Keyframes<'data, T> {
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
            keyframes,
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
            keyframes,
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
