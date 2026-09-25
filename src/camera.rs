//! Typed camera records in `CAMS` chunks.

use crate::Record;
use std::borrow::Cow;

use crate::utils::field;
use crate::{AnimationTrack, ChunkRecord, Error, Model};

const HEADER_SIZE: usize = 120;
const NAME_SIZE: usize = 80;
const MAX_RECORD_SIZE: usize = 0x00ff_ffff;

/// A camera with decoded fixed fields and animation tracks.
#[derive(Clone, Debug, PartialEq)]
pub struct Camera {
    name: [u8; NAME_SIZE],
    record_flags: u8,
    position: [f32; 3],
    field_of_view: f32,
    far_clip: f32,
    near_clip: f32,
    target_position: [f32; 3],
    tracks: Vec<AnimationTrack>,
}

impl Camera {
    /// Creates a camera with zeroed position and target fields.
    pub fn new(name: &str) -> Result<Self, Error> {
        let mut camera = Self {
            name: [0; NAME_SIZE],
            record_flags: 0,
            position: [0.0; 3],
            field_of_view: 0.0,
            far_clip: 0.0,
            near_clip: 0.0,
            target_position: [0.0; 3],
            tracks: Vec::new(),
        };
        camera.set_name(name)?;
        Ok(camera)
    }

    /// Creates a camera with the record flags used by newer models.
    pub fn new_for_version(name: &str, version: u32) -> Result<Self, Error> {
        let mut camera = Self::new(name)?;
        if version >= 1200 {
            camera.record_flags = 3;
        }
        Ok(camera)
    }

    /// Returns the upper-byte record flags.
    pub fn record_flags(&self) -> u8 {
        self.record_flags
    }
    /// Changes the upper-byte record flags.
    pub fn set_record_flags(&mut self, flags: u8) {
        self.record_flags = flags;
    }
    /// Returns the name up to its first NUL.
    pub fn name(&self) -> Cow<'_, str> {
        field::text(&self.name)
    }
    /// Changes the name and clears unused bytes.
    pub fn set_name(&mut self, name: &str) -> Result<(), Error> {
        field::set_text(&mut self.name, name)
    }
    /// Returns camera XYZ position.
    pub fn position(&self) -> [f32; 3] {
        self.position
    }
    /// Changes camera XYZ position.
    pub fn set_position(&mut self, position: [f32; 3]) {
        self.position = position;
    }
    /// Returns field of view.
    pub fn field_of_view(&self) -> f32 {
        self.field_of_view
    }
    /// Changes field of view.
    pub fn set_field_of_view(&mut self, value: f32) {
        self.field_of_view = value;
    }
    /// Returns far clipping distance.
    pub fn far_clip(&self) -> f32 {
        self.far_clip
    }
    /// Changes far clipping distance.
    pub fn set_far_clip(&mut self, value: f32) {
        self.far_clip = value;
    }
    /// Returns near clipping distance.
    pub fn near_clip(&self) -> f32 {
        self.near_clip
    }
    /// Changes near clipping distance.
    pub fn set_near_clip(&mut self, value: f32) {
        self.near_clip = value;
    }
    /// Returns target XYZ position.
    pub fn target_position(&self) -> [f32; 3] {
        self.target_position
    }
    /// Changes target XYZ position.
    pub fn set_target_position(&mut self, target: [f32; 3]) {
        self.target_position = target;
    }
    /// Borrows decoded camera tracks without reparsing.
    pub fn tracks(&self) -> &[AnimationTrack] {
        &self.tracks
    }
    /// Replaces camera tracks.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), Error> {
        let mut size = HEADER_SIZE;
        for track in tracks {
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: Camera::TAG,
                    offset: size,
                });
            }
            size = size
                .checked_add(track.encode()?.len())
                .filter(|&size| size <= MAX_RECORD_SIZE)
                .ok_or(Error::ChunkTooLarge {
                    tag: Camera::TAG,
                    size: usize::MAX,
                })?;
        }
        self.tracks = tracks.to_vec();
        Ok(())
    }
}

fn is_track(tag: [u8; 4]) -> bool {
    matches!(&tag, b"KCTR" | b"KTTR" | b"KCRL")
}

impl Model {
    /// Decodes all camera records in `CAMS` chunks.
    pub fn cameras(&self) -> Result<Vec<Camera>, Error> {
        let mut cameras = Vec::new();
        for chunk in self
            .chunks()
            .iter()
            .filter(|chunk| chunk.tag == Camera::TAG)
        {
            let mut offset = 0;
            while offset < chunk.data.len() {
                let size_bytes = chunk.data.get(offset..offset.saturating_add(4)).ok_or(
                    Error::MalformedRecord {
                        tag: Camera::TAG,
                        offset,
                    },
                )?;
                let size = (u32::from_le_bytes(size_bytes.try_into().expect("four-byte size"))
                    & 0x00ff_ffff) as usize;
                if size < HEADER_SIZE {
                    return Err(Error::MalformedRecord {
                        tag: Camera::TAG,
                        offset,
                    });
                }
                let end = offset
                    .checked_add(size)
                    .filter(|&end| end <= chunk.data.len())
                    .ok_or(Error::MalformedRecord {
                        tag: Camera::TAG,
                        offset,
                    })?;
                cameras.push(Camera::decode(&chunk.data[offset..end], 0)?);
                offset = end;
            }
        }
        Ok(cameras)
    }

    /// Replaces cameras in the first `CAMS` chunk.
    pub fn set_cameras(&mut self, cameras: &[Camera]) -> Result<(), Error> {
        let mut data = Vec::new();
        for camera in cameras {
            data.extend_from_slice(&camera.encode()?);
            if data.len() > u32::MAX as usize {
                return Err(Error::ChunkTooLarge {
                    tag: Camera::TAG,
                    size: data.len(),
                });
            }
        }
        self.replace_chunks(Camera::TAG, data);
        Ok(())
    }
}

impl Record for Camera {
    fn decode(bytes: &[u8], _version: u32) -> Result<Self, Error> {
        if bytes.len() < HEADER_SIZE {
            return Err(Error::MalformedRecord {
                tag: Camera::TAG,
                offset: 0,
            });
        }
        let size_word = u32::from_le_bytes(bytes[..4].try_into().expect("four-byte size"));
        if (size_word & 0x00ff_ffff) as usize != bytes.len() {
            return Err(Error::MalformedRecord {
                tag: Camera::TAG,
                offset: 0,
            });
        }
        let float = |offset: usize| {
            f32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("camera field"))
        };
        let mut tracks = Vec::new();
        let mut offset = HEADER_SIZE;
        while offset < bytes.len() {
            let (track, size) = AnimationTrack::parse(bytes, offset)?;
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: Camera::TAG,
                    offset,
                });
            }
            tracks.push(track);
            offset += size;
        }
        Ok(Self {
            name: bytes[4..84].try_into().expect("fixed-width name"),
            record_flags: (size_word >> 24) as u8,
            position: [float(84), float(88), float(92)],
            field_of_view: float(96),
            far_clip: float(100),
            near_clip: float(104),
            target_position: [float(108), float(112), float(116)],
            tracks,
        })
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![0; HEADER_SIZE];
        bytes[4..84].copy_from_slice(&self.name);
        for (index, value) in self.position.into_iter().enumerate() {
            bytes[84 + index * 4..88 + index * 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes[96..100].copy_from_slice(&self.field_of_view.to_le_bytes());
        bytes[100..104].copy_from_slice(&self.far_clip.to_le_bytes());
        bytes[104..108].copy_from_slice(&self.near_clip.to_le_bytes());
        for (index, value) in self.target_position.into_iter().enumerate() {
            bytes[108 + index * 4..112 + index * 4].copy_from_slice(&value.to_le_bytes());
        }
        for track in &self.tracks {
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: Camera::TAG,
                    offset: bytes.len(),
                });
            }
            bytes.extend_from_slice(&track.encode()?);
        }
        if bytes.len() > MAX_RECORD_SIZE {
            return Err(Error::ChunkTooLarge {
                tag: Camera::TAG,
                size: bytes.len(),
            });
        }
        let size = bytes.len() as u32 | (u32::from(self.record_flags) << 24);
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        Ok(bytes)
    }
}

impl ChunkRecord for Camera {
    const TAG: [u8; 4] = *b"CAMS";
}
