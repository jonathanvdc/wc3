//! Scalar global sequences and XYZ pivot points.

use crate::{Chunk, Error, Model};

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
        if let Some(first) = self.chunks.iter().position(|chunk| chunk.tag == tag) {
            self.chunks[first].data = data;
            let mut seen = false;
            self.chunks.retain(|chunk| {
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
