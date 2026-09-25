//! Typed camera records in `CAMS` chunks.

use crate::Record;
use crate::{CamerasChunk, Cursor, ModelChunk};
use std::borrow::Cow;

use crate::utils::field;
use crate::{AnimationTrack, Error, Model};

pub(crate) const HEADER_SIZE: usize = 120;
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
        self.collect_chunk_records::<CamerasChunk>(|chunk| match chunk {
            ModelChunk::Cameras(decoded) => Some(decoded),
            _ => None,
        })
    }

    /// Replaces cameras in the first `CAMS` chunk.
    pub fn set_cameras(&mut self, cameras: &[Camera]) -> Result<(), Error> {
        self.replace_chunk(ModelChunk::Cameras(CamerasChunk::new(cameras.to_vec())))
    }
}

impl Record for Camera {
    fn decode_one(source: &mut Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let start = source.absolute_position();
        let size_word = source.read_u32()?;
        let length = (size_word & 0x00ff_ffff) as usize;
        let body_len = length.checked_sub(4).ok_or(Error::InvalidRecordLength {
            offset: start,
            length,
        })?;
        let mut cursor = source.slice(body_len)?;
        let name = cursor.read_exact(80)?.try_into().expect("fixed-width name");
        let position = cursor.read_vec3()?;
        let field_of_view = cursor.read_f32()?;
        let far_clip = cursor.read_f32()?;
        let near_clip = cursor.read_f32()?;
        let target_position = cursor.read_vec3()?;
        let mut tracks = Vec::new();
        while !cursor.remaining().is_empty() {
            let offset = cursor.absolute_position();
            let track = AnimationTrack::decode_one(&mut cursor, 0)?;
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: Self::TAG,
                    offset,
                });
            }

            tracks.push(track);
        }
        cursor.finish()?;
        Ok(Self {
            name,
            record_flags: (size_word >> 24) as u8,
            position,
            field_of_view,
            far_clip,
            near_clip,
            target_position,
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

impl Camera {
    /// The tag of the chunk containing this record.
    pub const TAG: [u8; 4] = *b"CAMS";
}
