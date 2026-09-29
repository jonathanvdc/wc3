//! Typed keyframe tracks.
use super::{Interpolation, TangentKeyframe, TrackValue, ValueKeyframe};
use crate::model::mdl::{Parser, ReadErrorKind, TokenKind, Writer};
use crate::model::{mdl, mdx};
use crate::model::{Cursor, Encoder, ReadError, ValueError, WriteError};
use std::io::Write as IoWrite;

#[derive(Clone, Debug, PartialEq)]
enum Keyframes<T> {
    Step(Vec<ValueKeyframe<T>>),
    Linear(Vec<ValueKeyframe<T>>),
    Hermite(Vec<TangentKeyframe<T>>),
    Bezier(Vec<TangentKeyframe<T>>),
}

/// Keyframes for one animated property.
///
/// The type `T` selects the keyframe value type. Constructors select the
/// interpolation and matching keyframe shape. Times are in milliseconds;
/// `global_sequence_id` is an index into the model global-sequence collection,
/// or `None` to use the current model sequence.
#[derive(Clone, Debug, PartialEq)]
pub struct Track<T: TrackValue> {
    global_sequence_id: Option<u32>,
    keyframes: Keyframes<T>,
}

impl<T: TrackValue> Track<T> {
    /// A constant animation represented by one stepped key at time zero.
    pub fn constant(value: T) -> Self {
        Self::step(vec![ValueKeyframe { frame: 0, value }], None)
            .expect("one key without a global sequence is valid")
    }

    /// Creates stepped animation: each key value holds until the next key.
    pub fn step(
        keys: Vec<ValueKeyframe<T>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Step(keys), global_sequence_id)
    }
    /// Creates animation that interpolates linearly between adjacent key values.
    pub fn linear(
        keys: Vec<ValueKeyframe<T>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Linear(keys), global_sequence_id)
    }
    /// Creates a Hermite track whose keys each have two tangents.
    pub fn hermite(
        keys: Vec<TangentKeyframe<T>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Hermite(keys), global_sequence_id)
    }
    /// Creates a Bezier track whose keys each have two tangents.
    pub fn bezier(
        keys: Vec<TangentKeyframe<T>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Bezier(keys), global_sequence_id)
    }
    fn new(keyframes: Keyframes<T>, global_sequence_id: Option<u32>) -> Result<Self, ValueError> {
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
    /// Returns keys only when this track uses stepped interpolation.
    pub fn step_keys(&self) -> Option<&[ValueKeyframe<T>]> {
        match &self.keyframes {
            Keyframes::Step(keys) => Some(keys),
            _ => None,
        }
    }
    /// Returns keys only when this track uses linear interpolation.
    pub fn linear_keys(&self) -> Option<&[ValueKeyframe<T>]> {
        match &self.keyframes {
            Keyframes::Linear(keys) => Some(keys),
            _ => None,
        }
    }
    /// Returns keys and tangents only when this track uses Hermite interpolation.
    pub fn hermite_keys(&self) -> Option<&[TangentKeyframe<T>]> {
        match &self.keyframes {
            Keyframes::Hermite(keys) => Some(keys),
            _ => None,
        }
    }
    /// Returns keys and tangents only when this track uses Bezier interpolation.
    pub fn bezier_keys(&self) -> Option<&[TangentKeyframe<T>]> {
        match &self.keyframes {
            Keyframes::Bezier(keys) => Some(keys),
            _ => None,
        }
    }
}
impl<T> Keyframes<T> {
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
impl<T: TrackValue> mdx::Read for Track<T> {
    fn read_mdx(cursor: &mut Cursor<'_>) -> Result<Self, ReadError> {
        let mut next = *cursor;
        let offset = next.absolute_position();
        let tag = *b"TRAK";
        let malformed = || ReadError::MalformedRecord { tag, offset };
        let count = next.read::<u32>().map_err(|_| malformed())? as usize;
        let interpolation = next.read::<u32>().map_err(|_| malformed())?;
        if interpolation > 3 {
            return Err(malformed());
        }
        let sequence = next.read::<u32>().map_err(|_| malformed())?;
        let components = T::COMPONENTS;
        let vector_count = if interpolation >= 2 { 3usize } else { 1usize };
        let key_size = components
            .checked_mul(vector_count)
            .and_then(|n| n.checked_mul(4))
            .and_then(|n| n.checked_add(4))
            .ok_or_else(malformed)?;
        let body_size = count.checked_mul(key_size).ok_or_else(malformed)?;
        let mut body = next.subcursor(body_size).map_err(|_| malformed())?;
        let keyframes = match interpolation {
            0 | 1 => {
                let mut keys = Vec::with_capacity(count);
                for _ in 0..count {
                    keys.push(body.read::<ValueKeyframe<T>>().map_err(|_| malformed())?);
                }
                if interpolation == 0 {
                    Keyframes::Step(keys)
                } else {
                    Keyframes::Linear(keys)
                }
            }
            2 | 3 => {
                let mut keys = Vec::with_capacity(count);
                for _ in 0..count {
                    keys.push(body.read::<TangentKeyframe<T>>().map_err(|_| malformed())?);
                }
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
impl<T: TrackValue> mdx::Write for Track<T> {
    fn write_mdx(&self, encoder: &mut Encoder<'_>) -> Result<(), WriteError> {
        encoder.write(&(self.keyframes.len() as u32))?;
        encoder.write(&(self.keyframes.interpolation()))?;
        encoder.write(&(self.global_sequence_id.unwrap_or(u32::MAX)))?;
        match &self.keyframes {
            Keyframes::Step(keys) | Keyframes::Linear(keys) => {
                for key in keys {
                    encoder.write(key)?;
                }
            }
            Keyframes::Hermite(keys) | Keyframes::Bezier(keys) => {
                for key in keys {
                    encoder.write(key)?;
                }
            }
        }
        Ok(())
    }
}

impl<T: TrackValue> mdl::Read for Track<T>
where
    T: mdl::Read,
{
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        parser.expect_ident("Track")?;
        Self::read_mdl_payload(parser)
    }
}
impl<T: TrackValue> Track<T>
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
                        ReadErrorKind::DuplicateField,
                    ));
                }
                parser.peek()?;
                let span = parser
                    .error(ReadErrorKind::InvalidNumber("global sequence ID"))
                    .span;
                let id = parser.read_property::<u32>()?;
                if id == u32::MAX {
                    return Err(mdl::ReadError::new(
                        span,
                        ReadErrorKind::InvalidNumber("global sequence ID"),
                    ));
                }
                sequence = Some(id);
            } else {
                let mode = parser.read::<Interpolation>()?;
                if interpolation.replace(mode).is_some() {
                    return Err(mdl::ReadError::new(
                        token.span,
                        ReadErrorKind::DuplicateField,
                    ));
                }
                parser.expect(TokenKind::Comma)?;
            }
        }
        let interpolation = interpolation
            .ok_or_else(|| parser.error(ReadErrorKind::MissingField("interpolation")))?;
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
                            ReadErrorKind::CountMismatch {
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
                return Err(parser.error(ReadErrorKind::CountMismatch {
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
                tangents.push(TangentKeyframe {
                    frame,
                    value,
                    in_tangent,
                    out_tangent,
                });
            } else {
                values.push(ValueKeyframe { frame, value });
            }
            actual += 1;
        }
        let keyframes = match interpolation {
            Interpolation::Step => Keyframes::Step(values),
            Interpolation::Linear => Keyframes::Linear(values),
            Interpolation::Hermite => Keyframes::Hermite(tangents),
            Interpolation::Bezier => Keyframes::Bezier(tangents),
        };
        Ok(Self {
            global_sequence_id: sequence,
            keyframes,
        })
    }
}

impl<T: TrackValue> mdl::Write for Track<T>
where
    T: mdl::Write,
{
    fn write_mdl<W: IoWrite>(&self, writer: &mut Writer<W>) -> Result<(), mdl::WriteError> {
        self.write_mdl_named(writer, "Track")
    }
}
impl<T: TrackValue> Track<T>
where
    T: mdl::Write,
{
    /// Writes a borrowed track under an enclosing record's alias.
    pub fn write_mdl_named<W: IoWrite>(
        &self,
        writer: &mut Writer<W>,
        name: &str,
    ) -> Result<(), mdl::WriteError> {
        writer.begin_counted_block(name, self.keyframes.len())?;
        writer.indent()?;
        writer.write(&self.interpolation())?;
        writer.raw(",\n")?;
        if let Some(sequence) = self.global_sequence_id {
            writer.property("GlobalSeqId", &sequence)?;
        }
        match &self.keyframes {
            Keyframes::Step(keys) | Keyframes::Linear(keys) => {
                for key in keys {
                    write_key(writer, key.frame, &key.value)?;
                }
            }
            Keyframes::Hermite(keys) | Keyframes::Bezier(keys) => {
                for key in keys {
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
) -> Result<(), mdl::WriteError> {
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
) -> Result<(), mdl::WriteError> {
    writer.indent()?;
    writer.raw("\t")?;
    writer.identifier(name)?;
    writer.raw(" ")?;
    writer.write(value)?;
    writer.raw(",\n")
}
