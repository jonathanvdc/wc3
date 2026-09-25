//! Scalar global sequences and XYZ pivot points.

use crate::{Chunk, ChunkRecord, Error, Model, Record};

/// The complete `GLBS` payload.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GlobalSequencesChunk {
    pub durations: Vec<u32>,
}

impl Record for GlobalSequencesChunk {
    fn decode(bytes: &[u8], _version: u32) -> Result<Self, Error> {
        if bytes.len() % 4 != 0 {
            return Err(Error::MalformedChunk {
                tag: Self::TAG,
                size: bytes.len(),
                expected: 4,
            });
        }
        Ok(Self {
            durations: bytes
                .chunks_exact(4)
                .map(|word| u32::from_le_bytes(word.try_into().expect("four-byte field")))
                .collect(),
        })
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let size = checked_chunk_size(self.durations.len(), 4, Self::TAG)?;
        let mut bytes = Vec::with_capacity(size);
        for duration in &self.durations {
            bytes.extend_from_slice(&duration.to_le_bytes());
        }
        Ok(bytes)
    }
}

impl ChunkRecord for GlobalSequencesChunk {
    const TAG: [u8; 4] = *b"GLBS";
}

/// The complete `PIVT` payload.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PivotPointsChunk {
    pub points: Vec<[f32; 3]>,
}

impl Record for PivotPointsChunk {
    fn decode(bytes: &[u8], _version: u32) -> Result<Self, Error> {
        if bytes.len() % 12 != 0 {
            return Err(Error::MalformedChunk {
                tag: Self::TAG,
                size: bytes.len(),
                expected: 12,
            });
        }
        Ok(Self {
            points: bytes
                .chunks_exact(12)
                .map(|point| {
                    std::array::from_fn(|index| {
                        let offset = index * 4;
                        f32::from_le_bytes(
                            point[offset..offset + 4]
                                .try_into()
                                .expect("four-byte field"),
                        )
                    })
                })
                .collect(),
        })
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let size = checked_chunk_size(self.points.len(), 12, Self::TAG)?;
        let mut bytes = Vec::with_capacity(size);
        for point in &self.points {
            for coordinate in point {
                bytes.extend_from_slice(&coordinate.to_le_bytes());
            }
        }
        Ok(bytes)
    }
}

impl ChunkRecord for PivotPointsChunk {
    const TAG: [u8; 4] = *b"PIVT";
}

fn checked_chunk_size(count: usize, width: usize, tag: [u8; 4]) -> Result<usize, Error> {
    let size = count.checked_mul(width).ok_or(Error::ChunkTooLarge {
        tag,
        size: usize::MAX,
    })?;
    if size > u32::MAX as usize {
        return Err(Error::ChunkTooLarge { tag, size });
    }
    Ok(size)
}

#[cfg(test)]
mod chunk_record_tests {
    use super::*;

    #[test]
    fn scalar_chunks_round_trip_complete_payloads() {
        let globals = GlobalSequencesChunk {
            durations: vec![100, 200, 300],
        };
        assert_eq!(
            GlobalSequencesChunk::decode_chunk(&globals.encode_chunk().unwrap()).unwrap(),
            globals
        );

        let pivots = PivotPointsChunk {
            points: vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
        };
        assert_eq!(
            PivotPointsChunk::decode_chunk(&pivots.encode_chunk().unwrap()).unwrap(),
            pivots
        );
        assert!(GlobalSequencesChunk::decode(&[1], 800).is_err());
        assert!(PivotPointsChunk::decode(&[1], 800).is_err());
    }
}

impl Model {
    /// Returns durations from every `GLBS` chunk in file order.
    pub fn global_sequences(&self) -> Result<Vec<u32>, Error> {
        let mut durations = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == *b"GLBS") {
            if chunk.data.len() % 4 != 0 {
                return Err(Error::MalformedChunk {
                    tag: *b"GLBS",
                    size: chunk.data.len(),
                    expected: 4,
                });
            }
            durations.extend(
                chunk
                    .data
                    .chunks_exact(4)
                    .map(|bytes| u32::from_le_bytes(bytes.try_into().expect("four-byte field"))),
            );
        }
        Ok(durations)
    }

    /// Writes global sequence durations to a `GLBS` chunk.
    pub fn set_global_sequences(&mut self, durations: &[u32]) -> Result<(), Error> {
        let size = durations.len().checked_mul(4).ok_or(Error::ChunkTooLarge {
            tag: *b"GLBS",
            size: usize::MAX,
        })?;
        if size > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: *b"GLBS",
                size,
            });
        }
        let mut data = Vec::with_capacity(size);
        for duration in durations {
            data.extend_from_slice(&duration.to_le_bytes());
        }
        self.replace_chunks(*b"GLBS", data);
        Ok(())
    }

    /// Returns XYZ pivot points from every `PIVT` chunk in file order.
    pub fn pivot_points(&self) -> Result<Vec<[f32; 3]>, Error> {
        let mut points = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == *b"PIVT") {
            if chunk.data.len() % 12 != 0 {
                return Err(Error::MalformedChunk {
                    tag: *b"PIVT",
                    size: chunk.data.len(),
                    expected: 12,
                });
            }
            points.extend(chunk.data.chunks_exact(12).map(|bytes| {
                std::array::from_fn(|index| {
                    let offset = index * 4;
                    f32::from_le_bytes(
                        bytes[offset..offset + 4]
                            .try_into()
                            .expect("four-byte field"),
                    )
                })
            }));
        }
        Ok(points)
    }

    /// Writes XYZ pivot points to a `PIVT` chunk.
    pub fn set_pivot_points(&mut self, points: &[[f32; 3]]) -> Result<(), Error> {
        let size = points.len().checked_mul(12).ok_or(Error::ChunkTooLarge {
            tag: *b"PIVT",
            size: usize::MAX,
        })?;
        if size > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: *b"PIVT",
                size,
            });
        }
        let mut data = Vec::with_capacity(size);
        for point in points {
            for coordinate in point {
                data.extend_from_slice(&coordinate.to_le_bytes());
            }
        }
        self.replace_chunks(*b"PIVT", data);
        Ok(())
    }

    pub(crate) fn replace_chunks(&mut self, tag: [u8; 4], data: Vec<u8>) {
        if let Some(first) = self.chunks_mut().iter().position(|chunk| chunk.tag == tag) {
            let lengths: Vec<_> = self
                .chunks_mut()
                .iter()
                .filter(|chunk| chunk.tag == tag)
                .map(|chunk| chunk.data.len())
                .collect();
            if lengths
                .iter()
                .try_fold(0usize, |sum, length| sum.checked_add(*length))
                == Some(data.len())
            {
                let mut offset = 0;
                for chunk in self
                    .chunks_mut()
                    .iter_mut()
                    .filter(|chunk| chunk.tag == tag)
                {
                    let end = offset + chunk.data.len();
                    chunk.data = data[offset..end].to_vec();
                    offset = end;
                }
                return;
            }
            self.chunks_mut()[first].data = data;
            let mut seen = false;
            self.chunks_mut().retain(|chunk| {
                if chunk.tag != tag {
                    return true;
                }
                if seen {
                    false
                } else {
                    seen = true;
                    true
                }
            });
        } else {
            self.push(Chunk::new(tag, data));
        }
    }
}
