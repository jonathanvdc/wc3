//! Geoset animation records in `GEOA` chunks.

use crate::{AnimationTrack, Error, Model};

const TAG: [u8; 4] = *b"GEOA";
const HEADER_SIZE: usize = 28;

/// A geoset animation with optional track bytes retained.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeosetAnimation {
    bytes: Vec<u8>,
}

impl GeosetAnimation {
    /// Creates a geoset animation with opaque white color and full alpha.
    pub fn new(geoset_id: u32) -> Self {
        let mut bytes = vec![0; HEADER_SIZE];
        bytes[..4].copy_from_slice(&(HEADER_SIZE as u32).to_le_bytes());
        bytes[4..8].copy_from_slice(&1.0f32.to_le_bytes());
        for offset in [12, 16, 20] {
            bytes[offset..offset + 4].copy_from_slice(&1.0f32.to_le_bytes());
        }
        bytes[24..28].copy_from_slice(&geoset_id.to_le_bytes());
        Self { bytes }
    }

    /// Wraps one inclusive-size record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < HEADER_SIZE {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        let size = u32::from_le_bytes(bytes[..4].try_into().expect("four-byte size")) as usize;
        if size != bytes.len() {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Returns the complete record, including any animation tracks.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns base alpha.
    pub fn alpha(&self) -> f32 {
        f32::from_bits(self.u32_at(4))
    }

    /// Sets base alpha.
    pub fn set_alpha(&mut self, alpha: f32) {
        self.set_u32_at(4, alpha.to_bits());
    }

    /// Returns raw animation flags.
    pub fn flags(&self) -> u32 {
        self.u32_at(8)
    }

    /// Sets raw animation flags.
    pub fn set_flags(&mut self, flags: u32) {
        self.set_u32_at(8, flags);
    }

    /// Returns the RGB base color.
    pub fn color(&self) -> [f32; 3] {
        std::array::from_fn(|index| f32::from_bits(self.u32_at(12 + index * 4)))
    }

    /// Sets the RGB base color.
    pub fn set_color(&mut self, color: [f32; 3]) {
        for (index, value) in color.into_iter().enumerate() {
            self.set_u32_at(12 + index * 4, value.to_bits());
        }
    }

    /// Returns the referenced geoset index.
    pub fn geoset_id(&self) -> u32 {
        self.u32_at(24)
    }

    /// Sets the referenced geoset index.
    pub fn set_geoset_id(&mut self, id: u32) {
        self.set_u32_at(24, id);
    }

    /// Returns optional `KGAO` and `KGAC` track data without interpretation.
    pub fn track_bytes(&self) -> &[u8] {
        &self.bytes[HEADER_SIZE..]
    }

    /// Decodes alpha and color animation tracks.
    pub fn tracks(&self) -> Result<Vec<AnimationTrack>, Error> {
        let mut result = Vec::new();
        let mut offset = HEADER_SIZE;
        while offset < self.bytes.len() {
            let (track, consumed) = AnimationTrack::parse(&self.bytes, offset)?;
            if !matches!(&track.tag, b"KGAO" | b"KGAC") {
                return Err(Error::MalformedRecord { tag: TAG, offset });
            }
            result.push(track);
            offset += consumed;
        }
        Ok(result)
    }

    /// Replaces alpha and color animation tracks.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), Error> {
        let mut bytes = self.bytes[..HEADER_SIZE].to_vec();
        for track in tracks {
            if !matches!(&track.tag, b"KGAO" | b"KGAC") {
                return Err(Error::MalformedRecord {
                    tag: TAG,
                    offset: bytes.len(),
                });
            }
            bytes.extend_from_slice(&track.to_bytes()?);
        }
        if bytes.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: TAG,
                size: bytes.len(),
            });
        }
        let size = bytes.len() as u32;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        self.bytes = bytes;
        Ok(())
    }

    fn u32_at(&self, offset: usize) -> u32 {
        u32::from_le_bytes(
            self.bytes[offset..offset + 4]
                .try_into()
                .expect("four-byte field"),
        )
    }

    fn set_u32_at(&mut self, offset: usize, value: u32) {
        self.bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

impl Model {
    /// Decodes all `GEOA` records in file order.
    pub fn geoset_animations(&self) -> Result<Vec<GeosetAnimation>, Error> {
        let mut animations = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == TAG) {
            let mut offset = 0;
            while offset < chunk.data.len() {
                let size_bytes = chunk
                    .data
                    .get(offset..offset.saturating_add(4))
                    .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
                let size =
                    u32::from_le_bytes(size_bytes.try_into().expect("four-byte size")) as usize;
                if size < HEADER_SIZE {
                    return Err(Error::MalformedRecord { tag: TAG, offset });
                }
                let end = offset
                    .checked_add(size)
                    .filter(|&end| end <= chunk.data.len())
                    .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
                animations.push(GeosetAnimation::from_bytes(&chunk.data[offset..end])?);
                offset = end;
            }
        }
        Ok(animations)
    }

    /// Replaces all geoset animations in the first `GEOA` chunk.
    pub fn set_geoset_animations(&mut self, animations: &[GeosetAnimation]) -> Result<(), Error> {
        let size = animations.iter().try_fold(0usize, |sum, animation| {
            sum.checked_add(animation.bytes.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for animation in animations {
            data.extend_from_slice(animation.as_bytes());
        }
        self.replace_chunks(TAG, data);
        Ok(())
    }
}
