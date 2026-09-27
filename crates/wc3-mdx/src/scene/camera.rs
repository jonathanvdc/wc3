//! Typed camera records in `CAMS` chunks.
use crate::ModelVersion;
use std::fmt::Debug;
crate::animation::track_group! {
    pub enum CameraTrack {
        Translation: CameraTranslation,
        TargetTranslation: CameraTargetTranslation,
        Rotation: CameraRotation,
    }
}

use crate::EncodeError;
use crate::Encoder;
use crate::KnownChunk;
use crate::ValueError;
use crate::Vec3;

use crate::{CamerasChunk, Cursor};
use crate::{Encodable, Readable};
use std::borrow::Cow;

use crate::FixedText;
use crate::{DecodeError, Model};

const NAME_SIZE: usize = 80;
const MAX_RECORD_SIZE: usize = 0x00ff_ffff;

/// The high-byte record flags used by a camera layout.
pub trait CameraFlags: Clone + Debug + PartialEq {
    fn empty() -> Self;
    fn from_bits(bits: u8) -> Self;
    fn bits(&self) -> u8;
    fn set_bits(&mut self, bits: u8);
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClassicCameraFlags(u8);

impl CameraFlags for ClassicCameraFlags {
    fn empty() -> Self {
        Self(0)
    }
    fn from_bits(bits: u8) -> Self {
        Self(bits)
    }
    fn bits(&self) -> u8 {
        self.0
    }
    fn set_bits(&mut self, bits: u8) {
        self.0 = bits;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModernCameraFlags(u8);

impl CameraFlags for ModernCameraFlags {
    fn empty() -> Self {
        Self(3)
    }
    fn from_bits(bits: u8) -> Self {
        Self(bits)
    }
    fn bits(&self) -> u8 {
        self.0
    }
    fn set_bits(&mut self, bits: u8) {
        self.0 = bits;
    }
}

pub trait CameraLayout {
    type Flags: CameraFlags;
}

use crate::{V1000, V1100, V1200, V1800, V800, V900};
impl CameraLayout for V800 {
    type Flags = ClassicCameraFlags;
}
impl CameraLayout for V900 {
    type Flags = ClassicCameraFlags;
}
impl CameraLayout for V1000 {
    type Flags = ClassicCameraFlags;
}
impl CameraLayout for V1100 {
    type Flags = ClassicCameraFlags;
}
impl CameraLayout for V1200 {
    type Flags = ModernCameraFlags;
}
impl CameraLayout for V1800 {
    type Flags = ModernCameraFlags;
}

/// A camera with decoded fixed fields and animation tracks.
#[derive(Clone, Debug, PartialEq)]
pub struct Camera<V: ModelVersion> {
    name: FixedText<NAME_SIZE>,
    record_flags: V::Flags,
    position: Vec3,
    field_of_view: f32,
    far_clip: f32,
    near_clip: f32,
    target_position: Vec3,
    tracks: Vec<CameraTrack>,
}

impl<V: ModelVersion> Camera<V> {
    /// Creates a camera with zeroed position and target fields.
    pub fn new(name: &str) -> Result<Self, ValueError> {
        let mut camera = Self {
            name: FixedText::default(),
            record_flags: V::Flags::empty(),
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

    /// Returns the upper-byte record flags.
    pub fn record_flags(&self) -> u8 {
        self.record_flags.bits()
    }
    /// Changes the upper-byte record flags.
    pub fn set_record_flags(&mut self, flags: u8) {
        self.record_flags.set_bits(flags);
    }
    /// Returns the name up to its first NUL.
    pub fn name(&self) -> Cow<'_, str> {
        self.name.text()
    }
    /// Changes the name and clears unused bytes.
    pub fn set_name(&mut self, name: &str) -> Result<(), ValueError> {
        self.name.set_text(name)
    }
    /// Returns camera XYZ position.
    pub fn position(&self) -> Vec3 {
        self.position
    }
    /// Changes camera XYZ position.
    pub fn set_position(&mut self, position: Vec3) {
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
    pub fn target_position(&self) -> Vec3 {
        self.target_position
    }
    /// Changes target XYZ position.
    pub fn set_target_position(&mut self, target: Vec3) {
        self.target_position = target;
    }
    /// Borrows decoded camera tracks without reparsing.
    pub fn tracks(&self) -> &[CameraTrack] {
        &self.tracks
    }
    /// Replaces camera tracks.
    pub fn set_tracks(&mut self, tracks: &[CameraTrack]) {
        self.tracks = tracks.to_vec();
    }
}

impl<V: ModelVersion> Model<V> {
    /// Decodes all camera records in `CAMS` chunks.
    pub fn cameras(&self) -> Vec<Camera<V>> {
        self.collect_chunk_records::<CamerasChunk<V>>()
    }

    /// Replaces cameras in the first `CAMS` chunk.
    pub fn set_cameras(&mut self, cameras: &[Camera<V>]) {
        self.replace_chunk(CamerasChunk::new(cameras.to_vec()));
    }
}

impl<V: ModelVersion> Readable for Camera<V> {
    fn read_from(source: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        let start = source.absolute_position();
        let size_word: u32 = source.read()?;
        let length = (size_word & 0x00ff_ffff) as usize;
        let body_len = length
            .checked_sub(4)
            .ok_or(DecodeError::InvalidRecordLength {
                offset: start,
                length,
            })?;
        let mut cursor = source.slice(body_len)?;
        let name = cursor.read()?;
        let position = cursor.read()?;
        let field_of_view = cursor.read()?;
        let far_clip = cursor.read()?;
        let near_clip = cursor.read()?;
        let target_position = cursor.read()?;
        let mut tracks = Vec::new();
        while !cursor.remaining().is_empty() {
            tracks.push(cursor.read::<CameraTrack>()?);
        }
        cursor.finish()?;
        Ok(Self {
            name,
            record_flags: V::Flags::from_bits((size_word >> 24) as u8),
            position,
            field_of_view,
            far_clip,
            near_clip,
            target_position,
            tracks,
        })
    }
}

impl<V: ModelVersion> Encodable for Camera<V> {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        let start = bytes.position();
        let marker = bytes.begin_sized();
        bytes.write(&self.name);
        bytes.write(self.position);
        bytes.write(self.field_of_view);
        bytes.write(self.far_clip);
        bytes.write(self.near_clip);
        bytes.write(self.target_position);
        for track in &self.tracks {
            bytes.write(track);
        }
        if bytes.position() - start > MAX_RECORD_SIZE {
            return Err(EncodeError::ChunkTooLarge {
                tag: CamerasChunk::<V>::TAG,
                size: bytes.position() - start,
            });
        }
        bytes.finish_sized_with_flags(
            marker,
            CamerasChunk::<V>::TAG,
            u32::from(self.record_flags.bits()) << 24,
        )?;
        Ok(())
    }
}
