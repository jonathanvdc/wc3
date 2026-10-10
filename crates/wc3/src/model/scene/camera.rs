//! Camera views, targets, and lens animation.
use crate::model::animation::Track;
use crate::model::conversion::ConversionContext;
use crate::model::mdl;
use crate::model::mdx;
use crate::model::ModelDialect;
use crate::model::ModelVersion;
use crate::model::{ConversionError, ConversionIssueKind};
use mdl_codec::Target;
mod mdl_codec;
use crate::model::Encoder;
use crate::model::FixedText;
use crate::model::KnownChunk;
use crate::model::Model;
use crate::model::ValueError;
use crate::model::Vec3;
use crate::model::{CamerasChunk, Cursor};
use std::marker::PhantomData;

const NAME_SIZE: usize = 80;
const MAX_RECORD_SIZE: usize = 0x00ff_ffff;

/// Binary camera layout. Preserve the decoded variant when editing MDX.
/// Version conversion normalizes equivalent variants 0/3 to the target default.
/// Variants with extra bytes and unknown variants retain their exact layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CameraVariant {
    /// Variant 0, without an additional prefix payload.
    Variant0,
    /// Variant 1, preserving its twelve uninterpreted prefix bytes.
    Variant1([u8; 12]),
    /// Variant 2, preserving its twelve uninterpreted prefix bytes.
    Variant2([u8; 12]),
    /// Variant 3, without an additional prefix payload.
    Variant3,
    /// An unrecognized variant identifier.
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

/// Selects the default camera prefix for a model version.
pub trait CameraLayout {
    /// Camera variant used when creating a new camera.
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

/// A camera view with a position, look-at target, clipping planes, and animation.
#[derive(Clone, Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(block = "Camera", validate_write = "Self::validate_mdl",
    write_order(position, translation, rotation, field_of_view, far_clip, near_clip,
        focus_distance, focal_length, f_stop, target, visibility),
    virtual_fields(
        #[mdl(block = "Target", default, get = "Self::mdl_target", set = "Self::set_mdl_target")]
        target: Target,
    )
)]
pub struct Camera<V: ModelVersion> {
    #[mdl(header)]
    /// Name used to identify this camera view.
    pub name: FixedText<NAME_SIZE>,
    #[mdl(skip, default = "Self::mdl_variant")]
    /// Record layout variant and any associated bytes.
    pub variant: CameraVariant,
    #[mdl(property = "Position", default)]
    /// Camera XYZ position.
    pub position: Vec3,
    #[mdl(property = "FieldOfView")]
    /// Vertical field of view in radians.
    pub field_of_view: f32,
    #[mdl(property = "FarClip")]
    /// Far clipping distance.
    pub far_clip: f32,
    #[mdl(property = "NearClip", default)]
    /// Near clipping distance.
    pub near_clip: f32,
    #[mdl(skip, default)]
    /// Target XYZ position.
    pub target_position: Vec3,
    #[mdl(property = "Translation")]
    /// Optional eye-position offset animation.
    pub translation: Option<Track<Vec3>>,
    #[mdl(property = "Rotation")]
    /// Optional camera roll animation in radians.
    pub rotation: Option<Track<f32>>,
    #[mdl(skip, default)]
    /// Optional target-position offset animation.
    pub target_translation: Option<Track<Vec3>>,
    #[mdl(property = "Visibility")]
    /// Optional authored visibility animation.
    pub visibility: Option<Track<f32>>,
    #[mdl(property = "FocusDistanceKeys", constant = "DOFDistance")]
    /// Optional depth-of-field focus-distance animation.
    pub focus_distance: Option<Track<f32>>,
    #[mdl(property = "FocalLengthKeys", constant = "FocalLength")]
    /// Optional focal-length animation.
    pub focal_length: Option<Track<f32>>,
    #[mdl(property = "FStopKeys", constant = "FStop")]
    /// Optional aperture f-stop animation.
    pub f_stop: Option<Track<f32>>,
    #[mdl(skip, default)]
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
            translation: None,
            rotation: None,
            target_translation: None,
            visibility: None,
            focus_distance: None,
            focal_length: None,
            f_stop: None,
            version: PhantomData,
        };
        camera.name.set_text(name)?;
        Ok(camera)
    }
}

impl<D: ModelDialect> Model<D> {
    /// Returns owned copies of camera records in `CAMS` chunks.
    pub fn cameras(&self) -> Vec<Camera<D::Version>> {
        self.collect_chunk_records::<CamerasChunk<D::Version>>()
    }

    /// Replaces cameras with one `CAMS` chunk, removing any duplicate chunks.
    pub fn set_cameras(&mut self, cameras: &[Camera<D::Version>]) {
        self.replace_chunk(CamerasChunk::new(cameras.to_vec()));
    }
}

impl<V: ModelVersion> mdx::Read for Camera<V> {
    fn read_mdx(source: &mut Cursor<'_>) -> Result<Self, mdx::ReadError> {
        let start = source.absolute_position();
        let size_word: u32 = source.read()?;
        let length = (size_word & 0x00ff_ffff) as usize;
        let body_len = length.checked_sub(4).ok_or(mdx::ReadError::new(
            start,
            mdx::ReadErrorKind::InvalidRecordLength { length },
        ))?;
        let mut cursor = source.subcursor(body_len)?;
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
        let mut value = Self::new("").expect("empty camera name");
        while !cursor.remaining().is_empty() {
            let offset = cursor.absolute_position();
            let tag = cursor.read()?;
            match &tag {
                b"KCTR" => value.translation = Some(cursor.read()?),
                b"KCRL" => value.rotation = Some(cursor.read()?),
                b"KTTR" => value.target_translation = Some(cursor.read()?),
                b"KCVS" => value.visibility = Some(cursor.read()?),
                b"IDUF" => value.focus_distance = Some(cursor.read()?),
                b"ELAF" => value.focal_length = Some(cursor.read()?),
                b"PTSF" => value.f_stop = Some(cursor.read()?),
                _ => {
                    return Err(mdx::ReadError::new(
                        offset,
                        mdx::ReadErrorKind::UnknownTag { actual: tag },
                    )
                    .with_tag(tag))
                }
            }
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
            translation: value.translation,
            rotation: value.rotation,
            target_translation: value.target_translation,
            visibility: value.visibility,
            focus_distance: value.focus_distance,
            focal_length: value.focal_length,
            f_stop: value.f_stop,
            version: PhantomData,
        })
    }
}

impl<V: ModelVersion> mdx::Write for Camera<V> {
    fn write_mdx(&self, bytes: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
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
        mdx::WriteTrackProperty::write_mdx_track_property(&self.translation, *b"KCTR", bytes)?;
        mdx::WriteTrackProperty::write_mdx_track_property(&self.rotation, *b"KCRL", bytes)?;
        mdx::WriteTrackProperty::write_mdx_track_property(
            &self.target_translation,
            *b"KTTR",
            bytes,
        )?;
        mdx::WriteTrackProperty::write_mdx_track_property(&self.visibility, *b"KCVS", bytes)?;
        mdx::WriteTrackProperty::write_mdx_track_property(&self.focus_distance, *b"IDUF", bytes)?;
        mdx::WriteTrackProperty::write_mdx_track_property(&self.focal_length, *b"ELAF", bytes)?;
        mdx::WriteTrackProperty::write_mdx_track_property(&self.f_stop, *b"PTSF", bytes)?;
        if bytes.position() - start > MAX_RECORD_SIZE {
            return Err(mdx::WriteError::SizeOverflow {
                field: "encoded size",
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
        context: &mut ConversionContext<'_>,
        path: &str,
    ) -> Result<Camera<T>, ConversionError> {
        let variant = match self.variant {
            CameraVariant::Variant0 | CameraVariant::Variant3 => T::DEFAULT_VARIANT,
            variant => variant,
        };
        if variant != self.variant {
            context.issue(
                &format!("{path}.variant"),
                ConversionIssueKind::Normalized,
                &format!(
                    "normalized camera variant {} to {} for version {}",
                    self.variant.value(),
                    variant.value(),
                    T::NUMBER
                ),
            );
        }
        Ok(Camera {
            name: self.name,
            variant,
            position: self.position,
            field_of_view: self.field_of_view,
            far_clip: self.far_clip,
            near_clip: self.near_clip,
            target_position: self.target_position,
            translation: self.translation.clone(),
            rotation: self.rotation.clone(),
            target_translation: self.target_translation.clone(),
            visibility: self.visibility.clone(),
            focus_distance: self.focus_distance.clone(),
            focal_length: self.focal_length.clone(),
            f_stop: self.f_stop.clone(),
            version: PhantomData,
        })
    }
}
