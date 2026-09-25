//! Camera records in `CAMS` chunks.

use std::borrow::Cow;

use crate::{AnimationTrack, Error, Model};

const TAG: [u8; 4] = *b"CAMS";
const HEADER_SIZE: usize = 120;
const NAME_SIZE: usize = 80;

/// A camera record with its optional animation tracks retained.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Camera {
    bytes: Vec<u8>,
}

impl Camera {
    /// Creates a camera with zeroed position and target fields.
    pub fn new(name: &str) -> Result<Self, Error> {
        let mut bytes = vec![0; HEADER_SIZE];
        bytes[..4].copy_from_slice(&(HEADER_SIZE as u32).to_le_bytes());
        let mut camera = Self { bytes };
        camera.set_name(name)?;
        Ok(camera)
    }

    /// Creates a camera with the record flags used by newer models.
    pub fn new_for_version(name: &str, version: u32) -> Result<Self, Error> {
        let mut camera = Self::new(name)?;
        if version >= 1200 {
            camera.set_record_flags(3);
        }
        Ok(camera)
    }

    /// Wraps one inclusive-size camera record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < HEADER_SIZE {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        let size = (u32::from_le_bytes(bytes[..4].try_into().expect("four-byte size"))
            & 0x00ff_ffff) as usize;
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

    /// Returns the complete record.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the upper-byte record flags in the inclusive size word.
    pub fn record_flags(&self) -> u8 {
        self.bytes[3]
    }

    /// Sets the upper-byte record flags without changing the record length.
    pub fn set_record_flags(&mut self, flags: u8) {
        self.bytes[3] = flags;
    }

    /// Returns the camera name up to the first NUL.
    pub fn name(&self) -> Cow<'_, str> {
        let field = &self.bytes[4..84];
        let end = field
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(NAME_SIZE);
        String::from_utf8_lossy(&field[..end])
    }

    /// Sets the camera name.
    pub fn set_name(&mut self, name: &str) -> Result<(), Error> {
        if name.len() >= NAME_SIZE || name.as_bytes().contains(&0) {
            return Err(Error::InvalidString {
                max_bytes: NAME_SIZE - 1,
            });
        }
        self.bytes[4..84].fill(0);
        self.bytes[4..4 + name.len()].copy_from_slice(name.as_bytes());
        Ok(())
    }

    /// Returns the camera XYZ position.
    pub fn position(&self) -> [f32; 3] {
        self.vec3_at(84)
    }

    /// Sets the camera XYZ position.
    pub fn set_position(&mut self, position: [f32; 3]) {
        self.set_vec3_at(84, position);
    }

    /// Returns the field of view.
    pub fn field_of_view(&self) -> f32 {
        self.f32_at(96)
    }

    /// Sets the field of view.
    pub fn set_field_of_view(&mut self, value: f32) {
        self.set_f32_at(96, value);
    }

    /// Returns the far clipping distance.
    pub fn far_clip(&self) -> f32 {
        self.f32_at(100)
    }

    /// Sets the far clipping distance.
    pub fn set_far_clip(&mut self, value: f32) {
        self.set_f32_at(100, value);
    }

    /// Returns the near clipping distance.
    pub fn near_clip(&self) -> f32 {
        self.f32_at(104)
    }

    /// Sets the near clipping distance.
    pub fn set_near_clip(&mut self, value: f32) {
        self.set_f32_at(104, value);
    }

    /// Returns the camera target XYZ position.
    pub fn target_position(&self) -> [f32; 3] {
        self.vec3_at(108)
    }

    /// Sets the camera target XYZ position.
    pub fn set_target_position(&mut self, target: [f32; 3]) {
        self.set_vec3_at(108, target);
    }

    /// Returns undecoded track bytes.
    pub fn track_bytes(&self) -> &[u8] {
        &self.bytes[HEADER_SIZE..]
    }

    /// Decodes camera translation, target translation, and rotation tracks.
    pub fn tracks(&self) -> Result<Vec<AnimationTrack>, Error> {
        let mut tracks = Vec::new();
        let mut offset = HEADER_SIZE;
        while offset < self.bytes.len() {
            let (track, consumed) = AnimationTrack::parse(&self.bytes, offset)?;
            if !matches!(&track.tag, b"KCTR" | b"KTTR" | b"KCRL") {
                return Err(Error::MalformedRecord { tag: TAG, offset });
            }
            tracks.push(track);
            offset += consumed;
        }
        Ok(tracks)
    }

    /// Replaces camera tracks and updates the inclusive record size.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), Error> {
        let mut bytes = self.bytes[..HEADER_SIZE].to_vec();
        for track in tracks {
            if !matches!(&track.tag, b"KCTR" | b"KTTR" | b"KCRL") {
                return Err(Error::MalformedRecord {
                    tag: TAG,
                    offset: bytes.len(),
                });
            }
            bytes.extend_from_slice(&track.to_bytes()?);
        }
        if bytes.len() > 0x00ff_ffff {
            return Err(Error::ChunkTooLarge {
                tag: TAG,
                size: bytes.len(),
            });
        }
        let size = bytes.len() as u32 | (u32::from(self.record_flags()) << 24);
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        self.bytes = bytes;
        Ok(())
    }

    fn f32_at(&self, offset: usize) -> f32 {
        f32::from_le_bytes(
            self.bytes[offset..offset + 4]
                .try_into()
                .expect("four-byte field"),
        )
    }

    fn set_f32_at(&mut self, offset: usize, value: f32) {
        self.bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn vec3_at(&self, offset: usize) -> [f32; 3] {
        std::array::from_fn(|axis| self.f32_at(offset + axis * 4))
    }

    fn set_vec3_at(&mut self, offset: usize, vector: [f32; 3]) {
        for (axis, value) in vector.into_iter().enumerate() {
            self.set_f32_at(offset + axis * 4, value);
        }
    }
}

impl Model {
    /// Decodes all camera records in `CAMS` chunks.
    pub fn cameras(&self) -> Result<Vec<Camera>, Error> {
        let mut cameras = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == TAG) {
            let mut offset = 0;
            while offset < chunk.data.len() {
                let size_bytes = chunk
                    .data
                    .get(offset..offset.saturating_add(4))
                    .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
                let size = (u32::from_le_bytes(size_bytes.try_into().expect("four-byte size"))
                    & 0x00ff_ffff) as usize;
                if size < HEADER_SIZE {
                    return Err(Error::MalformedRecord { tag: TAG, offset });
                }
                let end = offset
                    .checked_add(size)
                    .filter(|&end| end <= chunk.data.len())
                    .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
                cameras.push(Camera::from_bytes(&chunk.data[offset..end])?);
                offset = end;
            }
        }
        Ok(cameras)
    }

    /// Replaces cameras in the first `CAMS` chunk.
    pub fn set_cameras(&mut self, cameras: &[Camera]) -> Result<(), Error> {
        let size = cameras.iter().try_fold(0usize, |sum, camera| {
            sum.checked_add(camera.bytes.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for camera in cameras {
            data.extend_from_slice(camera.as_bytes());
        }
        self.replace_chunks(TAG, data);
        Ok(())
    }
}
