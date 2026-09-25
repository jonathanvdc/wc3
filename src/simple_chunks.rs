//! Model accessors for scalar chunks.

use crate::{Error, GlobalSequencesChunk, Model, ModelChunk, PivotPointsChunk, RawChunk, Record};

impl Model {
    /// Returns durations from every `GLBS` chunk in file order.
    pub fn global_sequences(&self) -> Result<Vec<u32>, Error> {
        let mut durations = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag() == *b"GLBS") {
            match chunk {
                ModelChunk::GlobalSequences(decoded) => {
                    durations.extend_from_slice(&decoded.durations)
                }
                ModelChunk::Malformed(malformed) => return Err(malformed.error().clone()),
                ModelChunk::Unknown(raw) => durations
                    .extend(GlobalSequencesChunk::decode(&raw.data, self.version())?.durations),
                _ => unreachable!("GLBS tag matched another typed chunk"),
            }
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
        self.replace_chunks(*b"GLBS", data)?;
        Ok(())
    }

    /// Returns XYZ pivot points from every `PIVT` chunk in file order.
    pub fn pivot_points(&self) -> Result<Vec<[f32; 3]>, Error> {
        let mut points = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag() == *b"PIVT") {
            match chunk {
                ModelChunk::PivotPoints(decoded) => points.extend_from_slice(&decoded.points),
                ModelChunk::Malformed(malformed) => return Err(malformed.error().clone()),
                ModelChunk::Unknown(raw) => {
                    points.extend(PivotPointsChunk::decode(&raw.data, self.version())?.points)
                }
                _ => unreachable!("PIVT tag matched another typed chunk"),
            }
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
        self.replace_chunks(*b"PIVT", data)?;
        Ok(())
    }

    pub(crate) fn replace_chunks(&mut self, tag: [u8; 4], data: Vec<u8>) -> Result<(), Error> {
        let version = self.version();
        let mut positions = Vec::new();
        let mut lengths = Vec::new();
        for (index, chunk) in self.chunks().iter().enumerate() {
            if chunk.tag() == tag {
                positions.push(index);
                lengths.push(chunk.to_raw()?.data.len());
            }
        }
        if positions.is_empty() {
            self.push(ModelChunk::from_raw(RawChunk::new(tag, data), version));
        } else {
            self.chunks_mut()[positions[0]] =
                ModelChunk::from_raw(RawChunk::new(tag, data), version);
            let mut seen = false;
            self.chunks_mut().retain(|chunk| {
                if chunk.tag() != tag {
                    return true;
                }
                if seen {
                    false
                } else {
                    seen = true;
                    true
                }
            });
        }
        Ok(())
    }
}
