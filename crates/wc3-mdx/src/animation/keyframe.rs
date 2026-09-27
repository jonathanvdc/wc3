//! Keyframe values and their binary representations.
use super::TrackValue;
use crate::{EncodeError, Encoder, Readable, Writable};

/// An interpolation mode shared by all keys in a track.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Interpolation {
    Step,
    Linear,
    Hermite,
    Bezier,
}

/// A keyframe without interpolation tangents.
#[derive(Clone, Debug, PartialEq, Readable)]
pub struct ValueKeyframe<T> {
    /// Frame time in milliseconds.
    pub frame: u32,
    /// The value at this frame.
    pub value: T,
}

/// A keyframe with both interpolation tangents.
#[derive(Clone, Debug, PartialEq, Readable)]
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

impl<T: TrackValue> Writable for ValueKeyframe<T> {
    fn write_to(self, encoder: &mut Encoder<'_>) -> Result<(), EncodeError> {
        encoder.write(&self)?;
        Ok(())
    }
}
impl<T: TrackValue> Writable for TangentKeyframe<T> {
    fn write_to(self, encoder: &mut Encoder<'_>) -> Result<(), EncodeError> {
        encoder.write(&self)?;
        Ok(())
    }
}

impl<T: TrackValue> Writable for &ValueKeyframe<T> {
    fn write_to(self, encoder: &mut Encoder<'_>) -> Result<(), EncodeError> {
        encoder.write(self.frame)?;
        encoder.write(self.value)?;
        Ok(())
    }
}
impl<T: TrackValue> Writable for &TangentKeyframe<T> {
    fn write_to(self, encoder: &mut Encoder<'_>) -> Result<(), EncodeError> {
        encoder.write(self.frame)?;
        encoder.write(self.value)?;
        encoder.write(self.in_tangent)?;
        encoder.write(self.out_tangent)?;
        Ok(())
    }
}
