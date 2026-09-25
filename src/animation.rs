//! Keyframe tracks for node translation, rotation, and scaling.

use crate::Error;

/// One decoded keyframe. Tangents are present for Hermite and Bezier tracks.
#[derive(Clone, Debug, PartialEq)]
pub struct Keyframe {
    /// Frame time in milliseconds.
    pub frame: u32,
    /// One, three, or four component values, depending on the track tag.
    pub value: Vec<f32>,
    /// Incoming tangent for interpolation modes 2 and 3.
    pub in_tangent: Option<Vec<f32>>,
    /// Outgoing tangent for interpolation modes 2 and 3.
    pub out_tangent: Option<Vec<f32>>,
}

/// A decoded node animation track.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationTrack {
    /// `KGTR`, `KGRT`, or `KGSC` for a node transform.
    pub tag: [u8; 4],
    /// 0 = none, 1 = linear, 2 = Hermite, 3 = Bezier.
    pub interpolation: u32,
    /// Global sequence index, or `u32::MAX` when absent.
    pub global_sequence_id: u32,
    /// Keyframes in source order.
    pub keyframes: Vec<Keyframe>,
}

impl AnimationTrack {
    /// Parses one known track and returns the number of bytes consumed.
    pub(crate) fn parse(data: &[u8], offset: usize) -> Result<(Self, usize), Error> {
        let tag: [u8; 4] = data
            .get(offset..offset.saturating_add(4))
            .ok_or(Error::MalformedRecord {
                tag: *b"KGTR",
                offset,
            })?
            .try_into()
            .expect("four-byte tag");
        let components = components(tag).ok_or(Error::MalformedRecord { tag, offset })?;
        let header = data
            .get(offset + 4..offset.saturating_add(16))
            .ok_or(Error::MalformedRecord { tag, offset })?;
        let count = u32::from_le_bytes(header[..4].try_into().expect("four-byte count")) as usize;
        let interpolation = u32::from_le_bytes(header[4..8].try_into().expect("four-byte field"));
        if interpolation > 3 {
            return Err(Error::MalformedRecord { tag, offset });
        }
        let global_sequence_id =
            u32::from_le_bytes(header[8..12].try_into().expect("four-byte field"));
        let vector_count = if interpolation >= 2 { 3 } else { 1 };
        let key_size = components
            .checked_mul(vector_count)
            .and_then(|n| n.checked_mul(4))
            .and_then(|n| n.checked_add(4))
            .ok_or(Error::MalformedRecord { tag, offset })?;
        let body_size = count
            .checked_mul(key_size)
            .ok_or(Error::MalformedRecord { tag, offset })?;
        let end = offset
            .checked_add(16)
            .and_then(|n| n.checked_add(body_size))
            .filter(|&end| end <= data.len())
            .ok_or(Error::MalformedRecord { tag, offset })?;
        let mut cursor = offset + 16;
        let mut keyframes = Vec::with_capacity(count);
        while cursor < end {
            let frame =
                u32::from_le_bytes(data[cursor..cursor + 4].try_into().expect("bounded frame"));
            cursor += 4;
            let mut read_vector = || {
                let vector = (0..components)
                    .map(|_| {
                        let value = f32::from_le_bytes(
                            data[cursor..cursor + 4].try_into().expect("bounded value"),
                        );
                        cursor += 4;
                        value
                    })
                    .collect();
                vector
            };
            let value = read_vector();
            let in_tangent = (interpolation >= 2).then(&mut read_vector);
            let out_tangent = (interpolation >= 2).then(&mut read_vector);
            keyframes.push(Keyframe {
                frame,
                value,
                in_tangent,
                out_tangent,
            });
        }
        Ok((
            Self {
                tag,
                interpolation,
                global_sequence_id,
                keyframes,
            },
            end - offset,
        ))
    }

    /// Encodes a track after checking its tag, interpolation, and key widths.
    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let components = components(self.tag).ok_or(Error::MalformedRecord {
            tag: self.tag,
            offset: 0,
        })?;
        if self.interpolation > 3 || self.keyframes.len() > u32::MAX as usize {
            return Err(Error::MalformedRecord {
                tag: self.tag,
                offset: 0,
            });
        }
        let tangents = self.interpolation >= 2;
        let mut output = Vec::new();
        output.extend_from_slice(&self.tag);
        output.extend_from_slice(&(self.keyframes.len() as u32).to_le_bytes());
        output.extend_from_slice(&self.interpolation.to_le_bytes());
        output.extend_from_slice(&self.global_sequence_id.to_le_bytes());
        for (index, key) in self.keyframes.iter().enumerate() {
            if key.value.len() != components
                || key.in_tangent.is_some() != tangents
                || key.out_tangent.is_some() != tangents
                || key
                    .in_tangent
                    .as_ref()
                    .is_some_and(|v| v.len() != components)
                || key
                    .out_tangent
                    .as_ref()
                    .is_some_and(|v| v.len() != components)
            {
                return Err(Error::MalformedRecord {
                    tag: self.tag,
                    offset: index,
                });
            }
            output.extend_from_slice(&key.frame.to_le_bytes());
            for vector in [
                Some(&key.value),
                key.in_tangent.as_ref(),
                key.out_tangent.as_ref(),
            ]
            .into_iter()
            .flatten()
            {
                for value in vector {
                    output.extend_from_slice(&value.to_le_bytes());
                }
            }
        }
        Ok(output)
    }
}

fn components(tag: [u8; 4]) -> Option<usize> {
    match &tag {
        b"KGTR" | b"KGSC" => Some(3),
        b"KGRT" => Some(4),
        _ => None,
    }
}
