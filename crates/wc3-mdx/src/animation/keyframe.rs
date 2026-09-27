//! Keyframe values and their binary representations.
use super::TrackValue;
use crate::{Cursor, DecodeError, Encoder, Readable, Writable};

/// An interpolation mode shared by all keys in a track.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Interpolation {
    Step,
    Linear,
    Hermite,
    Bezier,
}

/// A keyframe without interpolation tangents.
#[derive(Clone, Debug, PartialEq)]
pub struct ValueKeyframe<T> {
    /// Frame time in milliseconds.
    pub frame: u32,
    /// The value at this frame.
    pub value: T,
}

/// A keyframe with both interpolation tangents.
#[derive(Clone, Debug, PartialEq)]
pub struct TangentKeyframe<T> {
    /// Frame time in milliseconds.
    pub frame: u32,
    /// The value at this frame.
    pub value: T,
    /// Incoming tangent.
    pub in_tangent: T,
    /// Outgoing tangent.
    pub out_tangent: T,
}

impl<T: TrackValue> Readable for ValueKeyframe<T> {
    fn read_from(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            frame: cursor.read()?,
            value: cursor.read()?,
        })
    }
}
impl<T: TrackValue> Writable for ValueKeyframe<T> {
    fn write_to(self, encoder: &mut Encoder<'_>) {
        encoder.write(&self);
    }
}
impl<T: TrackValue> Readable for TangentKeyframe<T> {
    fn read_from(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            frame: cursor.read()?,
            value: cursor.read()?,
            in_tangent: cursor.read()?,
            out_tangent: cursor.read()?,
        })
    }
}
impl<T: TrackValue> Writable for TangentKeyframe<T> {
    fn write_to(self, encoder: &mut Encoder<'_>) {
        encoder.write(&self);
    }
}

impl<T: TrackValue> Writable for &ValueKeyframe<T> {
    fn write_to(self, encoder: &mut Encoder<'_>) {
        encoder.write(self.frame);
        encoder.write(self.value);
    }
}
impl<T: TrackValue> Writable for &TangentKeyframe<T> {
    fn write_to(self, encoder: &mut Encoder<'_>) {
        encoder.write(self.frame);
        encoder.write(self.value);
        encoder.write(self.in_tangent);
        encoder.write(self.out_tangent);
    }
}
