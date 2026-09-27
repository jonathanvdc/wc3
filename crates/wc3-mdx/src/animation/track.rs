//! Typed keyframe tracks.
use super::{Interpolation, TangentKeyframe, TrackKind, TrackTag, TrackValue, ValueKeyframe};
use crate::{Cursor, DecodeError, EncodeError, Encoder, Readable, Tag, ValueError, Writable};
use std::marker::PhantomData;

#[derive(Clone, Debug, PartialEq)]
enum Keyframes<T> {
    Step(Vec<ValueKeyframe<T>>),
    Linear(Vec<ValueKeyframe<T>>),
    Hermite(Vec<TangentKeyframe<T>>),
    Bezier(Vec<TangentKeyframe<T>>),
}

/// An animation track whose tag, value type, and tangent shape are linked.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationTrack<K: TrackKind> {
    global_sequence_id: Option<u32>,
    keyframes: Keyframes<K::Value>,
    kind: PhantomData<K>,
}

impl<K: TrackKind> AnimationTrack<K> {
    /// Creates a track with no interpolation or tangents.
    pub fn step(
        keys: Vec<ValueKeyframe<K::Value>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Step(keys), global_sequence_id)
    }
    /// Creates a track with linear interpolation and no tangents.
    pub fn linear(
        keys: Vec<ValueKeyframe<K::Value>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Linear(keys), global_sequence_id)
    }
    /// Creates a Hermite track whose keys each have two tangents.
    pub fn hermite(
        keys: Vec<TangentKeyframe<K::Value>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Hermite(keys), global_sequence_id)
    }
    /// Creates a Bezier track whose keys each have two tangents.
    pub fn bezier(
        keys: Vec<TangentKeyframe<K::Value>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Bezier(keys), global_sequence_id)
    }
    fn new(
        keyframes: Keyframes<K::Value>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        if keyframes.len() > u32::MAX as usize {
            return Err(ValueError::CountTooLarge {
                tag: K::TAG,
                count: keyframes.len(),
            });
        }
        if global_sequence_id == Some(u32::MAX) {
            return Err(ValueError::InvalidGlobalSequenceId { id: u32::MAX });
        }
        Ok(Self {
            global_sequence_id,
            keyframes,
            kind: PhantomData,
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
    pub fn step_keys(&self) -> Option<&[ValueKeyframe<K::Value>]> {
        match &self.keyframes {
            Keyframes::Step(keys) => Some(keys),
            _ => None,
        }
    }
    pub fn linear_keys(&self) -> Option<&[ValueKeyframe<K::Value>]> {
        match &self.keyframes {
            Keyframes::Linear(keys) => Some(keys),
            _ => None,
        }
    }
    pub fn hermite_keys(&self) -> Option<&[TangentKeyframe<K::Value>]> {
        match &self.keyframes {
            Keyframes::Hermite(keys) => Some(keys),
            _ => None,
        }
    }
    pub fn bezier_keys(&self) -> Option<&[TangentKeyframe<K::Value>]> {
        match &self.keyframes {
            Keyframes::Bezier(keys) => Some(keys),
            _ => None,
        }
    }
    /// Returns this track's tag.
    pub fn tag(&self) -> TrackTag {
        K::TAG_KIND
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
impl<K: TrackKind> Readable for AnimationTrack<K> {
    fn read_from(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        let mut next = *cursor;
        let offset = next.absolute_position();
        let tag: Tag = next.read_exact(4)?.try_into().expect("four-byte tag");
        let malformed = || DecodeError::MalformedRecord { tag, offset };
        if tag != K::TAG {
            return Err(malformed());
        }
        let count = next.read::<u32>().map_err(|_| malformed())? as usize;
        let interpolation = next.read::<u32>().map_err(|_| malformed())?;
        if interpolation > 3 {
            return Err(malformed());
        }
        let sequence = next.read::<u32>().map_err(|_| malformed())?;
        let components = K::Value::COMPONENTS;
        let vector_count = if interpolation >= 2 { 3usize } else { 1usize };
        let key_size = components
            .checked_mul(vector_count)
            .and_then(|n| n.checked_mul(4))
            .and_then(|n| n.checked_add(4))
            .ok_or_else(malformed)?;
        let body_size = count.checked_mul(key_size).ok_or_else(malformed)?;
        let mut body = next.slice(body_size).map_err(|_| malformed())?;
        let keyframes = match interpolation {
            0 | 1 => {
                let mut keys = Vec::with_capacity(count);
                for _ in 0..count {
                    keys.push(
                        body.read::<ValueKeyframe<K::Value>>()
                            .map_err(|_| malformed())?,
                    );
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
                    keys.push(
                        body.read::<TangentKeyframe<K::Value>>()
                            .map_err(|_| malformed())?,
                    );
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
            kind: PhantomData,
        })
    }
}
impl<K: TrackKind> Writable for AnimationTrack<K> {
    fn write_to(&self, encoder: &mut Encoder<'_>) -> Result<(), EncodeError> {
        encoder.write_bytes(&K::TAG);
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
