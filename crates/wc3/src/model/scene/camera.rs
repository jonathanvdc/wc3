//! Typed camera records in `CAMS` chunks.
use crate::model::animation::{
    AnimationTrack, CameraRotation, CameraTargetTranslation, CameraTranslation, ValueKeyframe,
};
use crate::model::conversion::ConversionContext;
use crate::model::mdl;
use crate::model::mdl::{MdlWriter, Parser, ReadErrorKind, TokenKind};
use crate::model::mdx;
use crate::model::ConversionError;
use crate::model::ModelVersion;
use std::io::Write as IoWrite;
crate::model::animation::track_group! {
    @binary pub enum CameraTrack {
        Translation: CameraTranslation,
        TargetTranslation: CameraTargetTranslation,
        Rotation: CameraRotation,
        Visibility: CameraVisibility,
        FocusDistance: CameraFocusDistance,
        FocalLength: CameraFocalLength,
        FStop: CameraFStop,
    }
}

use crate::model::Encoder;
use crate::model::KnownChunk;
use crate::model::ValueError;
use crate::model::Vec3;
use crate::model::WriteError;

use crate::model::{CamerasChunk, Cursor};

use std::borrow::Cow;
use std::marker::PhantomData;

use crate::model::FixedText;
use crate::model::{Model, ReadError};

const NAME_SIZE: usize = 80;
const MAX_RECORD_SIZE: usize = 0x00ff_ffff;

/// Camera record layout selected by the high byte of its size word.
/// Variants 1 and 2 contain twelve otherwise uninterpreted bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CameraVariant {
    Variant0,
    Variant1([u8; 12]),
    Variant2([u8; 12]),
    Variant3,
    Unknown(u8),
}

impl CameraVariant {
    /// Returns the exact high-byte value stored in the record.
    pub const fn value(self) -> u8 {
        match self {
            Self::Variant0 => 0,
            Self::Variant1(_) => 1,
            Self::Variant2(_) => 2,
            Self::Variant3 => 3,
            Self::Unknown(value) => value,
        }
    }
}

pub trait CameraLayout {
    const DEFAULT_VARIANT: CameraVariant;
}

use crate::model::{V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900};
impl CameraLayout for V800 {
    const DEFAULT_VARIANT: CameraVariant = CameraVariant::Variant0;
}
impl CameraLayout for V900 {
    const DEFAULT_VARIANT: CameraVariant = CameraVariant::Variant0;
}
impl CameraLayout for V1000 {
    const DEFAULT_VARIANT: CameraVariant = CameraVariant::Variant0;
}
impl CameraLayout for V1100 {
    const DEFAULT_VARIANT: CameraVariant = CameraVariant::Variant0;
}
impl CameraLayout for V1200 {
    const DEFAULT_VARIANT: CameraVariant = CameraVariant::Variant3;
}
impl CameraLayout for V1300 {
    const DEFAULT_VARIANT: CameraVariant = CameraVariant::Variant3;
}
impl CameraLayout for V1400 {
    const DEFAULT_VARIANT: CameraVariant = CameraVariant::Variant3;
}
impl CameraLayout for V1600 {
    const DEFAULT_VARIANT: CameraVariant = CameraVariant::Variant3;
}
impl CameraLayout for V1800 {
    const DEFAULT_VARIANT: CameraVariant = CameraVariant::Variant3;
}

/// A camera with decoded fixed fields and animation tracks.
#[derive(Clone, Debug, PartialEq)]
pub struct Camera<V: ModelVersion> {
    name: FixedText<NAME_SIZE>,
    variant: CameraVariant,
    position: Vec3,
    field_of_view: f32,
    far_clip: f32,
    near_clip: f32,
    target_position: Vec3,
    tracks: Vec<CameraTrack>,
    version: PhantomData<V>,
}

impl<V: ModelVersion> Camera<V> {
    /// Creates a camera with zeroed position and target fields.
    pub fn new(name: &str) -> Result<Self, ValueError> {
        let mut camera = Self {
            name: FixedText::default(),
            variant: V::DEFAULT_VARIANT,
            position: [0.0; 3],
            field_of_view: 0.0,
            far_clip: 0.0,
            near_clip: 0.0,
            target_position: [0.0; 3],
            tracks: Vec::new(),
            version: PhantomData,
        };
        camera.set_name(name)?;
        Ok(camera)
    }

    /// Returns the record layout variant and any associated bytes.
    pub fn variant(&self) -> CameraVariant {
        self.variant
    }
    /// Changes the record layout variant.
    pub fn set_variant(&mut self, variant: CameraVariant) {
        self.variant = variant;
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

impl<V: ModelVersion> mdx::Read for Camera<V> {
    fn read_mdx(source: &mut Cursor<'_>) -> Result<Self, ReadError> {
        let start = source.absolute_position();
        let size_word: u32 = source.read()?;
        let length = (size_word & 0x00ff_ffff) as usize;
        let body_len = length
            .checked_sub(4)
            .ok_or(ReadError::InvalidRecordLength {
                offset: start,
                length,
            })?;
        let mut cursor = source.slice(body_len)?;
        let name = cursor.read()?;
        let position = cursor.read()?;
        let field_of_view = cursor.read()?;
        let far_clip = cursor.read()?;
        let near_clip = cursor.read()?;
        let variant = match (size_word >> 24) as u8 {
            0 => CameraVariant::Variant0,
            1 => CameraVariant::Variant1(cursor.read()?),
            2 => CameraVariant::Variant2(cursor.read()?),
            3 => CameraVariant::Variant3,
            value => CameraVariant::Unknown(value),
        };
        let target_position = cursor.read()?;
        let mut tracks = Vec::new();
        while !cursor.remaining().is_empty() {
            tracks.push(cursor.read::<CameraTrack>()?);
        }
        cursor.finish()?;
        Ok(Self {
            name,
            variant,
            position,
            field_of_view,
            far_clip,
            near_clip,
            target_position,
            tracks,
            version: PhantomData,
        })
    }
}

impl<V: ModelVersion> mdx::Write for Camera<V> {
    fn write_mdx(&self, bytes: &mut Encoder<'_>) -> Result<(), WriteError> {
        let start = bytes.position();
        let marker = bytes.begin_sized();
        bytes.write(&self.name)?;
        bytes.write(&self.position)?;
        bytes.write(&self.field_of_view)?;
        bytes.write(&self.far_clip)?;
        bytes.write(&self.near_clip)?;
        match self.variant {
            CameraVariant::Variant1(extra) | CameraVariant::Variant2(extra) => {
                bytes.write(&extra)?;
            }
            _ => {}
        }
        bytes.write(&self.target_position)?;
        for track in &self.tracks {
            bytes.write(track)?;
        }
        if bytes.position() - start > MAX_RECORD_SIZE {
            return Err(WriteError::ChunkTooLarge {
                tag: CamerasChunk::<V>::TAG,
                size: bytes.position() - start,
            });
        }
        bytes.finish_sized_with_flags(
            marker,
            CamerasChunk::<V>::TAG,
            u32::from(self.variant.value()) << 24,
        )?;
        Ok(())
    }
}

impl<V: ModelVersion> Camera<V> {
    pub(crate) fn convert_with<T: ModelVersion>(
        &self,
        _: &mut ConversionContext<'_>,
        _: &str,
    ) -> Result<Camera<T>, ConversionError> {
        Ok(Camera {
            name: self.name,
            variant: self.variant,
            position: self.position,
            field_of_view: self.field_of_view,
            far_clip: self.far_clip,
            near_clip: self.near_clip,
            target_position: self.target_position,
            tracks: self.tracks.clone(),
            version: PhantomData,
        })
    }
}

impl mdl::Read for CameraTrack {
    /// Reads a track in the camera body. Target tracks require read_mdl_target.
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        match parser.peek()?.map(|token| token.kind) {
            Some(TokenKind::Ident("Translation")) => Ok(Self::Translation(
                parser.read::<AnimationTrack<CameraTranslation>>()?,
            )),
            Some(TokenKind::Ident("Rotation")) => Ok(Self::Rotation(
                parser.read::<AnimationTrack<CameraRotation>>()?,
            )),
            Some(TokenKind::Ident("DOFDistance")) => {
                parser.next_token()?;
                Ok(Self::focus_distance(parser.read_property()?))
            }
            Some(TokenKind::Ident("FocalLength")) => {
                parser.next_token()?;
                Ok(Self::focal_length(parser.read_property()?))
            }
            Some(TokenKind::Ident("FStop")) => {
                parser.next_token()?;
                Ok(Self::f_stop(parser.read_property()?))
            }
            Some(TokenKind::Ident("Visibility")) => Ok(Self::Visibility(parser.read()?)),
            Some(TokenKind::Ident("FocusDistanceKeys")) => Ok(Self::FocusDistance(parser.read()?)),
            Some(TokenKind::Ident("FocalLengthKeys")) => Ok(Self::FocalLength(parser.read()?)),
            Some(TokenKind::Ident("FStopKeys")) => Ok(Self::FStop(parser.read()?)),
            _ => Err(parser.error(ReadErrorKind::UnknownField)),
        }
    }
}

impl CameraTrack {
    /// Represents a constant focus distance as a stepped key at time zero.
    pub fn focus_distance(value: f32) -> Self {
        Self::FocusDistance(
            AnimationTrack::step(vec![ValueKeyframe { frame: 0, value }], None)
                .expect("one key without a global sequence is valid"),
        )
    }

    /// Represents a constant focal length as a stepped key at time zero.
    pub fn focal_length(value: f32) -> Self {
        Self::FocalLength(
            AnimationTrack::step(vec![ValueKeyframe { frame: 0, value }], None)
                .expect("one key without a global sequence is valid"),
        )
    }

    /// Represents a constant f-stop as a stepped key at time zero.
    pub fn f_stop(value: f32) -> Self {
        Self::FStop(
            AnimationTrack::step(vec![ValueKeyframe { frame: 0, value }], None)
                .expect("one key without a global sequence is valid"),
        )
    }

    /// Reads a track inside an already opened Camera Target block.
    /// The enclosing record reader owns Target framing and Position properties.
    pub fn read_mdl_target(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        Ok(Self::TargetTranslation(
            parser.read::<AnimationTrack<CameraTargetTranslation>>()?,
        ))
    }

    /// Writes a track inside an already opened Camera Target block.
    pub fn write_mdl_target<W: IoWrite>(
        &self,
        writer: &mut MdlWriter<W>,
    ) -> Result<(), mdl::WriteError> {
        match self {
            Self::TargetTranslation(track) => writer.write(track),
            _ => Err(mdl::WriteError::Unsupported(
                "camera eye track inside Target",
            )),
        }
    }
}

impl mdl::Write for CameraTrack {
    fn write_mdl<W: IoWrite>(&self, writer: &mut MdlWriter<W>) -> Result<(), mdl::WriteError> {
        match self {
            Self::Translation(track) => writer.write(track),
            Self::Rotation(track) => writer.write(track),
            Self::Visibility(track) => writer.write(track),
            Self::FocusDistance(track) => writer.write(track),
            Self::FocalLength(track) => writer.write(track),
            Self::FStop(track) => writer.write(track),
            Self::TargetTranslation(_) => Err(mdl::WriteError::Unsupported(
                "target translation requires a Target block",
            )),
        }
    }
}
