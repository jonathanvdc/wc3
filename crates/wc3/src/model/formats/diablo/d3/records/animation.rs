//! Native records for the explicitly supported Diablo III revisions.
//! Names follow the native field vocabulary; offset descriptors are resolved.
use super::super::codec::{copy_value, Decoder, Encoder};
use super::super::{Read, ReadError, RecordLayout, Write, WriteError};
use crate::model::common::Ownership;
use zerovec::{ule::AsULE, ZeroVec};

use super::{HardpointLink, TriggerEvent};
/// Native BoneName record (64 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy)]
pub struct BoneName {
    /// Native `szBoneName` at byte 0.
    pub bone_name: [u8; 64],
}
impl Default for BoneName {
    fn default() -> Self {
        Self {
            bone_name: [Default::default(); 64],
        }
    }
}

impl RecordLayout for BoneName {
    const SIZE: usize = 64;
}
impl<'data> Read<'data> for BoneName {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            bone_name: decoder.value(offset)?,
        };
        Ok(result)
    }
}
impl Write for BoneName {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.bone_name)?;
        Ok(())
    }
}
impl AsULE for BoneName {
    type ULE = <[u8; 64] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 64];
        copy_value(&mut bytes[0..64], self.bone_name);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            bone_name: ZeroVec::<[u8; 64]>::parse_bytes(&bytes[0..64])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native TranslationKey record (16 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct TranslationKey {
    /// Native `nFrame` at byte 0.
    pub frame: i32,
    /// Native `vPosition` at byte 4.
    pub position: [f32; 3],
}

impl RecordLayout for TranslationKey {
    const SIZE: usize = 16;
}
impl<'data> Read<'data> for TranslationKey {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            frame: decoder.value(offset)?,
            position: decoder.value(offset + 4)?,
        };
        Ok(result)
    }
}
impl Write for TranslationKey {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.frame)?;
        encoder.value(offset + 4, self.position)?;
        Ok(())
    }
}
impl AsULE for TranslationKey {
    type ULE = <[u8; 16] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 16];
        copy_value(&mut bytes[0..4], self.frame);
        copy_value(&mut bytes[4..16], self.position);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            frame: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            position: ZeroVec::<[f32; 3]>::parse_bytes(&bytes[4..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native TranslationCurve record (24 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Default)]
pub struct TranslationCurve<'data> {
    /// Native `dwKeyCount` at byte 0.
    pub key_count: i32,
    /// Native `arKeys` at byte 4.
    pub keys: ZeroVec<'data, TranslationKey>,
    /// Native `_pad0` at byte 12.
    pub _pad0: [u8; 12],
}

impl RecordLayout for TranslationCurve<'_> {
    const SIZE: usize = 24;
}
impl<'data> Read<'data> for TranslationCurve<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            key_count: decoder.value(offset)?,
            keys: decoder.array(offset + 4, 0)?,
            _pad0: decoder.value(offset + 12)?,
        };
        decoder.count(offset, result.keys.len())?;
        Ok(result)
    }
}
impl Write for TranslationCurve<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.count(offset, self.keys.len())?;
        encoder.array(offset + 4, &self.keys, 0)?;
        encoder.value(offset + 12, self._pad0)?;
        Ok(())
    }
}

/// Native Quaternion16 record (8 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct Quaternion16 {
    /// Native `nX` at byte 0.
    pub x: u16,
    /// Native `nY` at byte 2.
    pub y: u16,
    /// Native `nZ` at byte 4.
    pub z: u16,
    /// Native `nW` at byte 6.
    pub w: u16,
}

impl RecordLayout for Quaternion16 {
    const SIZE: usize = 8;
}
impl<'data> Read<'data> for Quaternion16 {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            x: decoder.value(offset)?,
            y: decoder.value(offset + 2)?,
            z: decoder.value(offset + 4)?,
            w: decoder.value(offset + 6)?,
        };
        Ok(result)
    }
}
impl Write for Quaternion16 {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.x)?;
        encoder.value(offset + 2, self.y)?;
        encoder.value(offset + 4, self.z)?;
        encoder.value(offset + 6, self.w)?;
        Ok(())
    }
}
impl AsULE for Quaternion16 {
    type ULE = <[u8; 8] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 8];
        copy_value(&mut bytes[0..2], self.x);
        copy_value(&mut bytes[2..4], self.y);
        copy_value(&mut bytes[4..6], self.z);
        copy_value(&mut bytes[6..8], self.w);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            x: ZeroVec::<u16>::parse_bytes(&bytes[0..2])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            y: ZeroVec::<u16>::parse_bytes(&bytes[2..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            z: ZeroVec::<u16>::parse_bytes(&bytes[4..6])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            w: ZeroVec::<u16>::parse_bytes(&bytes[6..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native RotationKey record (12 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct RotationKey {
    /// Native `nFrame` at byte 0.
    pub frame: i32,
    /// Native `tRotation` at byte 4.
    pub rotation: Quaternion16,
}

impl RecordLayout for RotationKey {
    const SIZE: usize = 12;
}
impl<'data> Read<'data> for RotationKey {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            frame: decoder.value(offset)?,
            rotation: decoder.value(offset + 4)?,
        };
        Ok(result)
    }
}
impl Write for RotationKey {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.frame)?;
        encoder.value(offset + 4, self.rotation)?;
        Ok(())
    }
}
impl AsULE for RotationKey {
    type ULE = <[u8; 12] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 12];
        copy_value(&mut bytes[0..4], self.frame);
        copy_value(&mut bytes[4..12], self.rotation);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            frame: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            rotation: ZeroVec::<Quaternion16>::parse_bytes(&bytes[4..12])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native RotationCurve record (24 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Default)]
pub struct RotationCurve<'data> {
    /// Native `dwKeyCount` at byte 0.
    pub key_count: i32,
    /// Native `arKeys` at byte 4.
    pub keys: ZeroVec<'data, RotationKey>,
    /// Native `_pad0` at byte 12.
    pub _pad0: [u8; 12],
}

impl RecordLayout for RotationCurve<'_> {
    const SIZE: usize = 24;
}
impl<'data> Read<'data> for RotationCurve<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            key_count: decoder.value(offset)?,
            keys: decoder.array(offset + 4, 0)?,
            _pad0: decoder.value(offset + 12)?,
        };
        decoder.count(offset, result.keys.len())?;
        Ok(result)
    }
}
impl Write for RotationCurve<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.count(offset, self.keys.len())?;
        encoder.array(offset + 4, &self.keys, 0)?;
        encoder.value(offset + 12, self._pad0)?;
        Ok(())
    }
}

/// Native ScaleKey record (8 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct ScaleKey {
    /// Native `nFrame` at byte 0.
    pub frame: i32,
    /// Native `flScale` at byte 4.
    pub scale: f32,
}

impl RecordLayout for ScaleKey {
    const SIZE: usize = 8;
}
impl<'data> Read<'data> for ScaleKey {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            frame: decoder.value(offset)?,
            scale: decoder.value(offset + 4)?,
        };
        Ok(result)
    }
}
impl Write for ScaleKey {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.frame)?;
        encoder.value(offset + 4, self.scale)?;
        Ok(())
    }
}
impl AsULE for ScaleKey {
    type ULE = <[u8; 8] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 8];
        copy_value(&mut bytes[0..4], self.frame);
        copy_value(&mut bytes[4..8], self.scale);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            frame: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            scale: ZeroVec::<f32>::parse_bytes(&bytes[4..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native ScaleCurve record (24 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Default)]
pub struct ScaleCurve<'data> {
    /// Native `dwKeyCount` at byte 0.
    pub key_count: i32,
    /// Native `arKeys` at byte 4.
    pub keys: ZeroVec<'data, ScaleKey>,
    /// Native `_pad0` at byte 12.
    pub _pad0: [u8; 12],
}

impl RecordLayout for ScaleCurve<'_> {
    const SIZE: usize = 24;
}
impl<'data> Read<'data> for ScaleCurve<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            key_count: decoder.value(offset)?,
            keys: decoder.array(offset + 4, 0)?,
            _pad0: decoder.value(offset + 12)?,
        };
        decoder.count(offset, result.keys.len())?;
        Ok(result)
    }
}
impl Write for ScaleCurve<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.count(offset, self.keys.len())?;
        encoder.array(offset + 4, &self.keys, 0)?;
        encoder.value(offset + 12, self._pad0)?;
        Ok(())
    }
}

/// Native KeyframedAttachment record (412 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct KeyframedAttachment {
    /// Native `flFrame` at byte 0.
    pub frame: f32,
    /// Native `tEvent` at byte 4.
    pub event: TriggerEvent,
}

impl RecordLayout for KeyframedAttachment {
    const SIZE: usize = 412;
}
impl<'data> Read<'data> for KeyframedAttachment {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            frame: decoder.value(offset)?,
            event: decoder.value(offset + 4)?,
        };
        Ok(result)
    }
}
impl Write for KeyframedAttachment {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.frame)?;
        encoder.value(offset + 4, self.event)?;
        Ok(())
    }
}
impl AsULE for KeyframedAttachment {
    type ULE = <[u8; 412] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 412];
        copy_value(&mut bytes[0..4], self.frame);
        copy_value(&mut bytes[4..412], self.event);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            frame: ZeroVec::<f32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            event: ZeroVec::<TriggerEvent>::parse_bytes(&bytes[4..412])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native AnimPermutation record (408 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq)]
pub struct AnimationPermutation<'data> {
    /// Native `dwFlags` at byte 0.
    pub flags: i32,
    /// Native `szName` at byte 4.
    pub name: [u8; 68],
    /// Native `flFramesPerTick` at byte 72.
    pub frames_per_tick: f32,
    /// Native `flUnknown4C` at byte 76.
    pub unknown4_c: f32,
    /// Native `flFramesPerTranslationKey` at byte 80.
    pub frames_per_translation_key: f32,
    /// Native `flFramesPerRotationKey` at byte 84.
    pub frames_per_rotation_key: f32,
    /// Native `flFramesPerScaleKey` at byte 88.
    pub frames_per_scale_key: f32,
    /// Native `nBlendTicksFromOtherAnim` at byte 92.
    pub blend_ticks_from_other_anim: i32,
    /// Native `nBlendTicksSamePermSwap` at byte 96.
    pub blend_ticks_same_perm_swap: i32,
    /// Native `dwSelectionWeight` at byte 100.
    pub selection_weight: i32,
    /// Native `flSpeedScalar` at byte 104.
    pub speed_scalar: f32,
    /// Native `flUnknown6C` at byte 108.
    pub unknown6_c: f32,
    /// Native `flUnknown70` at byte 112.
    pub unknown70: f32,
    /// Native `flUnknown74` at byte 116.
    pub unknown74: f32,
    /// Native `dwBoneCount` at byte 120.
    pub bone_count: i32,
    /// Native `arBoneNames` at byte 124.
    pub bone_names: ZeroVec<'data, BoneName>,
    /// Native `_pad0` at byte 132.
    pub _pad0: [u8; 12],
    /// Native `dwFrameCount` at byte 144.
    pub frame_count: i32,
    /// Native `arTranslationCurves` at byte 148.
    pub translation_curves: Vec<TranslationCurve<'data>>,
    /// Native `_pad1` at byte 156.
    pub _pad1: [u8; 12],
    /// Native `arRotationCurves` at byte 168.
    pub rotation_curves: Vec<RotationCurve<'data>>,
    /// Native `_pad2` at byte 176.
    pub _pad2: [u8; 8],
    /// Native `arScaleCurves` at byte 184.
    pub scale_curves: Vec<ScaleCurve<'data>>,
    /// Native `_pad3` at byte 192.
    pub _pad3: [u8; 8],
    /// Native `flMarkerFrame0` at byte 200.
    pub marker_frame0: f32,
    /// Native `flMarkerFrame1` at byte 204.
    pub marker_frame1: f32,
    /// Native `flMarkerFrame2` at byte 208.
    pub marker_frame2: f32,
    /// Native `flMarkerFrame3` at byte 212.
    pub marker_frame3: f32,
    /// Native `vPoint0` at byte 216.
    pub point0: [f32; 3],
    /// Native `vPoint1` at byte 228.
    pub point1: [f32; 3],
    /// Native `vPoint2` at byte 240.
    pub point2: [f32; 3],
    /// Native `vPoint3` at byte 252.
    pub point3: [f32; 3],
    /// Native `flEventFrame` at byte 264.
    pub event_frame: f32,
    /// Native `arAttachments` at byte 268.
    pub attachments: ZeroVec<'data, KeyframedAttachment>,
    /// Native `dwAttachmentCount` at byte 276.
    pub attachment_count: i32,
    /// Native `_pad4` at byte 280.
    pub _pad4: [u8; 8],
    /// Native `arRootMotion` at byte 288.
    pub root_motion: ZeroVec<'data, [f32; 3]>,
    /// Native `_pad5` at byte 296.
    pub _pad5: [u8; 8],
    /// Native `arRootMotionInPlace` at byte 304.
    pub root_motion_in_place: ZeroVec<'data, [f32; 3]>,
    /// Native `_pad6` at byte 312.
    pub _pad6: [u8; 8],
    /// Native `vMovementVelocity` at byte 320.
    pub movement_velocity: [f32; 3],
    /// Native `tHardpoint` at byte 332.
    pub hardpoint: HardpointLink,
    /// Native `_pad7` at byte 400.
    pub _pad7: [u8; 8],
}
impl Default for AnimationPermutation<'_> {
    fn default() -> Self {
        Self {
            flags: Default::default(),
            name: [Default::default(); 68],
            frames_per_tick: Default::default(),
            unknown4_c: Default::default(),
            frames_per_translation_key: Default::default(),
            frames_per_rotation_key: Default::default(),
            frames_per_scale_key: Default::default(),
            blend_ticks_from_other_anim: Default::default(),
            blend_ticks_same_perm_swap: Default::default(),
            selection_weight: Default::default(),
            speed_scalar: Default::default(),
            unknown6_c: Default::default(),
            unknown70: Default::default(),
            unknown74: Default::default(),
            bone_count: Default::default(),
            bone_names: Default::default(),
            _pad0: [Default::default(); 12],
            frame_count: Default::default(),
            translation_curves: Default::default(),
            _pad1: [Default::default(); 12],
            rotation_curves: Default::default(),
            _pad2: [Default::default(); 8],
            scale_curves: Default::default(),
            _pad3: [Default::default(); 8],
            marker_frame0: Default::default(),
            marker_frame1: Default::default(),
            marker_frame2: Default::default(),
            marker_frame3: Default::default(),
            point0: [Default::default(); 3],
            point1: [Default::default(); 3],
            point2: [Default::default(); 3],
            point3: [Default::default(); 3],
            event_frame: Default::default(),
            attachments: Default::default(),
            attachment_count: Default::default(),
            _pad4: [Default::default(); 8],
            root_motion: Default::default(),
            _pad5: [Default::default(); 8],
            root_motion_in_place: Default::default(),
            _pad6: [Default::default(); 8],
            movement_velocity: [Default::default(); 3],
            hardpoint: Default::default(),
            _pad7: [Default::default(); 8],
        }
    }
}

impl RecordLayout for AnimationPermutation<'_> {
    const SIZE: usize = 408;
}
impl<'data> Read<'data> for AnimationPermutation<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            flags: decoder.value(offset)?,
            name: decoder.value(offset + 4)?,
            frames_per_tick: decoder.value(offset + 72)?,
            unknown4_c: decoder.value(offset + 76)?,
            frames_per_translation_key: decoder.value(offset + 80)?,
            frames_per_rotation_key: decoder.value(offset + 84)?,
            frames_per_scale_key: decoder.value(offset + 88)?,
            blend_ticks_from_other_anim: decoder.value(offset + 92)?,
            blend_ticks_same_perm_swap: decoder.value(offset + 96)?,
            selection_weight: decoder.value(offset + 100)?,
            speed_scalar: decoder.value(offset + 104)?,
            unknown6_c: decoder.value(offset + 108)?,
            unknown70: decoder.value(offset + 112)?,
            unknown74: decoder.value(offset + 116)?,
            bone_count: decoder.value(offset + 120)?,
            bone_names: decoder.array(offset + 124, 0)?,
            _pad0: decoder.value(offset + 132)?,
            frame_count: decoder.value(offset + 144)?,
            translation_curves: decoder.records(offset + 148, 0)?,
            _pad1: decoder.value(offset + 156)?,
            rotation_curves: decoder.records(offset + 168, 0)?,
            _pad2: decoder.value(offset + 176)?,
            scale_curves: decoder.records(offset + 184, 0)?,
            _pad3: decoder.value(offset + 192)?,
            marker_frame0: decoder.value(offset + 200)?,
            marker_frame1: decoder.value(offset + 204)?,
            marker_frame2: decoder.value(offset + 208)?,
            marker_frame3: decoder.value(offset + 212)?,
            point0: decoder.value(offset + 216)?,
            point1: decoder.value(offset + 228)?,
            point2: decoder.value(offset + 240)?,
            point3: decoder.value(offset + 252)?,
            event_frame: decoder.value(offset + 264)?,
            attachments: decoder.array(offset + 268, 0)?,
            attachment_count: decoder.value(offset + 276)?,
            _pad4: decoder.value(offset + 280)?,
            root_motion: decoder.array(offset + 288, 0)?,
            _pad5: decoder.value(offset + 296)?,
            root_motion_in_place: decoder.array(offset + 304, 0)?,
            _pad6: decoder.value(offset + 312)?,
            movement_velocity: decoder.value(offset + 320)?,
            hardpoint: decoder.value(offset + 332)?,
            _pad7: decoder.value(offset + 400)?,
        };
        decoder.count(offset + 120, result.bone_names.len())?;
        decoder.count(offset + 276, result.attachments.len())?;
        Ok(result)
    }
}
impl Write for AnimationPermutation<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.flags)?;
        encoder.value(offset + 4, self.name)?;
        encoder.value(offset + 72, self.frames_per_tick)?;
        encoder.value(offset + 76, self.unknown4_c)?;
        encoder.value(offset + 80, self.frames_per_translation_key)?;
        encoder.value(offset + 84, self.frames_per_rotation_key)?;
        encoder.value(offset + 88, self.frames_per_scale_key)?;
        encoder.value(offset + 92, self.blend_ticks_from_other_anim)?;
        encoder.value(offset + 96, self.blend_ticks_same_perm_swap)?;
        encoder.value(offset + 100, self.selection_weight)?;
        encoder.value(offset + 104, self.speed_scalar)?;
        encoder.value(offset + 108, self.unknown6_c)?;
        encoder.value(offset + 112, self.unknown70)?;
        encoder.value(offset + 116, self.unknown74)?;
        encoder.count(offset + 120, self.bone_names.len())?;
        encoder.array(offset + 124, &self.bone_names, 0)?;
        encoder.value(offset + 132, self._pad0)?;
        encoder.value(offset + 144, self.frame_count)?;
        encoder.records(offset + 148, &self.translation_curves, 0)?;
        encoder.value(offset + 156, self._pad1)?;
        encoder.records(offset + 168, &self.rotation_curves, 0)?;
        encoder.value(offset + 176, self._pad2)?;
        encoder.records(offset + 184, &self.scale_curves, 0)?;
        encoder.value(offset + 192, self._pad3)?;
        encoder.value(offset + 200, self.marker_frame0)?;
        encoder.value(offset + 204, self.marker_frame1)?;
        encoder.value(offset + 208, self.marker_frame2)?;
        encoder.value(offset + 212, self.marker_frame3)?;
        encoder.value(offset + 216, self.point0)?;
        encoder.value(offset + 228, self.point1)?;
        encoder.value(offset + 240, self.point2)?;
        encoder.value(offset + 252, self.point3)?;
        encoder.value(offset + 264, self.event_frame)?;
        encoder.array(offset + 268, &self.attachments, 0)?;
        encoder.count(offset + 276, self.attachments.len())?;
        encoder.value(offset + 280, self._pad4)?;
        encoder.array(offset + 288, &self.root_motion, 0)?;
        encoder.value(offset + 296, self._pad5)?;
        encoder.array(offset + 304, &self.root_motion_in_place, 0)?;
        encoder.value(offset + 312, self._pad6)?;
        encoder.value(offset + 320, self.movement_velocity)?;
        encoder.value(offset + 332, self.hardpoint)?;
        encoder.value(offset + 400, self._pad7)?;
        Ok(())
    }
}

/// Native Anim record (56 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Default)]
pub struct AnimationRecord<'data> {
    /// Native `dwSnoId` at byte 0.
    pub sno_id: i32,
    /// Native `_pad0` at byte 4.
    pub _pad0: [u8; 8],
    /// Native `dwFlags` at byte 12.
    pub flags: i32,
    /// Native `dwFlags2` at byte 16.
    pub flags2: i32,
    /// Native `snoAppearance` at byte 20.
    pub appearance: i32,
    /// Native `arPermutations` at byte 24.
    pub permutations: Vec<AnimationPermutation<'data>>,
    /// Native `dwPermutationCount` at byte 32.
    pub permutation_count: i32,
    /// Native `_pad1` at byte 36.
    pub _pad1: [u8; 12],
    /// Native `dwNameHash` at byte 48.
    pub name_hash: i32,
    /// Native `_pad2` at byte 52.
    pub _pad2: [u8; 4],
}

impl RecordLayout for AnimationRecord<'_> {
    const SIZE: usize = 56;
}
impl<'data> Read<'data> for AnimationRecord<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            sno_id: decoder.value(offset)?,
            _pad0: decoder.value(offset + 4)?,
            flags: decoder.value(offset + 12)?,
            flags2: decoder.value(offset + 16)?,
            appearance: decoder.value(offset + 20)?,
            permutations: decoder.records(offset + 24, 0)?,
            permutation_count: decoder.value(offset + 32)?,
            _pad1: decoder.value(offset + 36)?,
            name_hash: decoder.value(offset + 48)?,
            _pad2: decoder.value(offset + 52)?,
        };
        decoder.count(offset + 32, result.permutations.len())?;
        Ok(result)
    }
}
impl Write for AnimationRecord<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.sno_id)?;
        encoder.value(offset + 4, self._pad0)?;
        encoder.value(offset + 12, self.flags)?;
        encoder.value(offset + 16, self.flags2)?;
        encoder.value(offset + 20, self.appearance)?;
        encoder.records(offset + 24, &self.permutations, 0)?;
        encoder.count(offset + 32, self.permutations.len())?;
        encoder.value(offset + 36, self._pad1)?;
        encoder.value(offset + 48, self.name_hash)?;
        encoder.value(offset + 52, self._pad2)?;
        Ok(())
    }
}

/// Native AnimSetTagMapEntry record (12 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct AnimSetTagMapEntry {
    /// Native `nValueType` at byte 0.
    pub value_type: i32,
    /// Native `dwTagId` at byte 4.
    pub tag_id: i32,
    /// Native `snoAnim` at byte 8.
    pub anim: i32,
}

impl RecordLayout for AnimSetTagMapEntry {
    const SIZE: usize = 12;
}
impl<'data> Read<'data> for AnimSetTagMapEntry {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            value_type: decoder.value(offset)?,
            tag_id: decoder.value(offset + 4)?,
            anim: decoder.value(offset + 8)?,
        };
        Ok(result)
    }
}
impl Write for AnimSetTagMapEntry {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.value_type)?;
        encoder.value(offset + 4, self.tag_id)?;
        encoder.value(offset + 8, self.anim)?;
        Ok(())
    }
}
impl AsULE for AnimSetTagMapEntry {
    type ULE = <[u8; 12] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 12];
        copy_value(&mut bytes[0..4], self.value_type);
        copy_value(&mut bytes[4..8], self.tag_id);
        copy_value(&mut bytes[8..12], self.anim);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            value_type: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            tag_id: ZeroVec::<i32>::parse_bytes(&bytes[4..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            anim: ZeroVec::<i32>::parse_bytes(&bytes[8..12])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native AnimSet record (480 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Default)]
pub struct AnimationSetRecord<'data> {
    /// Native `dwSnoId` at byte 0.
    pub sno_id: i32,
    /// Native `_pad0` at byte 4.
    pub _pad0: [u8; 8],
    /// Native `snoBaseAnimSet` at byte 12.
    pub base_anim_set: i32,
    /// Native `tCoreTagMap` at byte 16.
    pub core_tag_map: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad1` at byte 24.
    pub _pad1: [u8; 8],
    /// Native `tmHth` at byte 32.
    pub hth: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad2` at byte 40.
    pub _pad2: [u8; 8],
    /// Native `tm1HSwing` at byte 48.
    pub weapon_1_h_swing: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad3` at byte 56.
    pub _pad3: [u8; 8],
    /// Native `tm1HThrust` at byte 64.
    pub weapon_1_h_thrust: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad4` at byte 72.
    pub _pad4: [u8; 8],
    /// Native `tm2HSwing` at byte 80.
    pub weapon_2_h_swing: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad5` at byte 88.
    pub _pad5: [u8; 8],
    /// Native `tm2HThrust` at byte 96.
    pub weapon_2_h_thrust: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad6` at byte 104.
    pub _pad6: [u8; 8],
    /// Native `tmStaff` at byte 112.
    pub staff: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad7` at byte 120.
    pub _pad7: [u8; 8],
    /// Native `tmBow` at byte 128.
    pub bow: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad8` at byte 136.
    pub _pad8: [u8; 8],
    /// Native `tmXBow` at byte 144.
    pub x_bow: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad9` at byte 152.
    pub _pad9: [u8; 8],
    /// Native `tmWand` at byte 160.
    pub wand: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad10` at byte 168.
    pub _pad10: [u8; 8],
    /// Native `tmDualWield` at byte 176.
    pub dual_wield: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad11` at byte 184.
    pub _pad11: [u8; 8],
    /// Native `tmHthWithOrb` at byte 192.
    pub hth_with_orb: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad12` at byte 200.
    pub _pad12: [u8; 8],
    /// Native `tm1HSwingWithOrb` at byte 208.
    pub weapon_1_h_swing_with_orb: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad13` at byte 216.
    pub _pad13: [u8; 8],
    /// Native `tm1HThrustWithOrb` at byte 224.
    pub weapon_1_h_thrust_with_orb: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad14` at byte 232.
    pub _pad14: [u8; 8],
    /// Native `tmDualWieldSwordFist` at byte 240.
    pub dual_wield_sword_fist: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad15` at byte 248.
    pub _pad15: [u8; 8],
    /// Native `tmDualWieldFistFist` at byte 256.
    pub dual_wield_fist_fist: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad16` at byte 264.
    pub _pad16: [u8; 8],
    /// Native `tm1HFist` at byte 272.
    pub weapon_1_h_fist: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad17` at byte 280.
    pub _pad17: [u8; 8],
    /// Native `tm2HAxeMace` at byte 288.
    pub weapon_2_h_axe_mace: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad18` at byte 296.
    pub _pad18: [u8; 8],
    /// Native `tmHandXBow` at byte 304.
    pub hand_x_bow: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad19` at byte 312.
    pub _pad19: [u8; 8],
    /// Native `tmWandWithOrb` at byte 320.
    pub wand_with_orb: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad20` at byte 328.
    pub _pad20: [u8; 8],
    /// Native `tm1HSwingWithShield` at byte 336.
    pub weapon_1_h_swing_with_shield: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad21` at byte 344.
    pub _pad21: [u8; 8],
    /// Native `tm1HThrustWithShield` at byte 352.
    pub weapon_1_h_thrust_with_shield: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad22` at byte 360.
    pub _pad22: [u8; 8],
    /// Native `tmHthWithShield` at byte 368.
    pub hth_with_shield: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad23` at byte 376.
    pub _pad23: [u8; 8],
    /// Native `tm2HSwingWithShield` at byte 384.
    pub weapon_2_h_swing_with_shield: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad24` at byte 392.
    pub _pad24: [u8; 8],
    /// Native `tm2HThrustWithShield` at byte 400.
    pub weapon_2_h_thrust_with_shield: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad25` at byte 408.
    pub _pad25: [u8; 8],
    /// Native `tmStaffWithShield` at byte 416.
    pub staff_with_shield: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad26` at byte 424.
    pub _pad26: [u8; 8],
    /// Native `tm2HFlail` at byte 432.
    pub weapon_2_h_flail: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad27` at byte 440.
    pub _pad27: [u8; 8],
    /// Native `tm2HFlailWithShield` at byte 448.
    pub weapon_2_h_flail_with_shield: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad28` at byte 456.
    pub _pad28: [u8; 8],
    /// Native `tmOnHorse` at byte 464.
    pub on_horse: ZeroVec<'data, AnimSetTagMapEntry>,
    /// Native `_pad29` at byte 472.
    pub _pad29: [u8; 8],
}

impl RecordLayout for AnimationSetRecord<'_> {
    const SIZE: usize = 480;
}
impl<'data> Read<'data> for AnimationSetRecord<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            sno_id: decoder.value(offset)?,
            _pad0: decoder.value(offset + 4)?,
            base_anim_set: decoder.value(offset + 12)?,
            core_tag_map: decoder.array(offset + 16, 4)?,
            _pad1: decoder.value(offset + 24)?,
            hth: decoder.array(offset + 32, 4)?,
            _pad2: decoder.value(offset + 40)?,
            weapon_1_h_swing: decoder.array(offset + 48, 4)?,
            _pad3: decoder.value(offset + 56)?,
            weapon_1_h_thrust: decoder.array(offset + 64, 4)?,
            _pad4: decoder.value(offset + 72)?,
            weapon_2_h_swing: decoder.array(offset + 80, 4)?,
            _pad5: decoder.value(offset + 88)?,
            weapon_2_h_thrust: decoder.array(offset + 96, 4)?,
            _pad6: decoder.value(offset + 104)?,
            staff: decoder.array(offset + 112, 4)?,
            _pad7: decoder.value(offset + 120)?,
            bow: decoder.array(offset + 128, 4)?,
            _pad8: decoder.value(offset + 136)?,
            x_bow: decoder.array(offset + 144, 4)?,
            _pad9: decoder.value(offset + 152)?,
            wand: decoder.array(offset + 160, 4)?,
            _pad10: decoder.value(offset + 168)?,
            dual_wield: decoder.array(offset + 176, 4)?,
            _pad11: decoder.value(offset + 184)?,
            hth_with_orb: decoder.array(offset + 192, 4)?,
            _pad12: decoder.value(offset + 200)?,
            weapon_1_h_swing_with_orb: decoder.array(offset + 208, 4)?,
            _pad13: decoder.value(offset + 216)?,
            weapon_1_h_thrust_with_orb: decoder.array(offset + 224, 4)?,
            _pad14: decoder.value(offset + 232)?,
            dual_wield_sword_fist: decoder.array(offset + 240, 4)?,
            _pad15: decoder.value(offset + 248)?,
            dual_wield_fist_fist: decoder.array(offset + 256, 4)?,
            _pad16: decoder.value(offset + 264)?,
            weapon_1_h_fist: decoder.array(offset + 272, 4)?,
            _pad17: decoder.value(offset + 280)?,
            weapon_2_h_axe_mace: decoder.array(offset + 288, 4)?,
            _pad18: decoder.value(offset + 296)?,
            hand_x_bow: decoder.array(offset + 304, 4)?,
            _pad19: decoder.value(offset + 312)?,
            wand_with_orb: decoder.array(offset + 320, 4)?,
            _pad20: decoder.value(offset + 328)?,
            weapon_1_h_swing_with_shield: decoder.array(offset + 336, 4)?,
            _pad21: decoder.value(offset + 344)?,
            weapon_1_h_thrust_with_shield: decoder.array(offset + 352, 4)?,
            _pad22: decoder.value(offset + 360)?,
            hth_with_shield: decoder.array(offset + 368, 4)?,
            _pad23: decoder.value(offset + 376)?,
            weapon_2_h_swing_with_shield: decoder.array(offset + 384, 4)?,
            _pad24: decoder.value(offset + 392)?,
            weapon_2_h_thrust_with_shield: decoder.array(offset + 400, 4)?,
            _pad25: decoder.value(offset + 408)?,
            staff_with_shield: decoder.array(offset + 416, 4)?,
            _pad26: decoder.value(offset + 424)?,
            weapon_2_h_flail: decoder.array(offset + 432, 4)?,
            _pad27: decoder.value(offset + 440)?,
            weapon_2_h_flail_with_shield: decoder.array(offset + 448, 4)?,
            _pad28: decoder.value(offset + 456)?,
            on_horse: decoder.array(offset + 464, 4)?,
            _pad29: decoder.value(offset + 472)?,
        };
        Ok(result)
    }
}
impl Write for AnimationSetRecord<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.sno_id)?;
        encoder.value(offset + 4, self._pad0)?;
        encoder.value(offset + 12, self.base_anim_set)?;
        encoder.array(offset + 16, &self.core_tag_map, 4)?;
        encoder.value(offset + 24, self._pad1)?;
        encoder.array(offset + 32, &self.hth, 4)?;
        encoder.value(offset + 40, self._pad2)?;
        encoder.array(offset + 48, &self.weapon_1_h_swing, 4)?;
        encoder.value(offset + 56, self._pad3)?;
        encoder.array(offset + 64, &self.weapon_1_h_thrust, 4)?;
        encoder.value(offset + 72, self._pad4)?;
        encoder.array(offset + 80, &self.weapon_2_h_swing, 4)?;
        encoder.value(offset + 88, self._pad5)?;
        encoder.array(offset + 96, &self.weapon_2_h_thrust, 4)?;
        encoder.value(offset + 104, self._pad6)?;
        encoder.array(offset + 112, &self.staff, 4)?;
        encoder.value(offset + 120, self._pad7)?;
        encoder.array(offset + 128, &self.bow, 4)?;
        encoder.value(offset + 136, self._pad8)?;
        encoder.array(offset + 144, &self.x_bow, 4)?;
        encoder.value(offset + 152, self._pad9)?;
        encoder.array(offset + 160, &self.wand, 4)?;
        encoder.value(offset + 168, self._pad10)?;
        encoder.array(offset + 176, &self.dual_wield, 4)?;
        encoder.value(offset + 184, self._pad11)?;
        encoder.array(offset + 192, &self.hth_with_orb, 4)?;
        encoder.value(offset + 200, self._pad12)?;
        encoder.array(offset + 208, &self.weapon_1_h_swing_with_orb, 4)?;
        encoder.value(offset + 216, self._pad13)?;
        encoder.array(offset + 224, &self.weapon_1_h_thrust_with_orb, 4)?;
        encoder.value(offset + 232, self._pad14)?;
        encoder.array(offset + 240, &self.dual_wield_sword_fist, 4)?;
        encoder.value(offset + 248, self._pad15)?;
        encoder.array(offset + 256, &self.dual_wield_fist_fist, 4)?;
        encoder.value(offset + 264, self._pad16)?;
        encoder.array(offset + 272, &self.weapon_1_h_fist, 4)?;
        encoder.value(offset + 280, self._pad17)?;
        encoder.array(offset + 288, &self.weapon_2_h_axe_mace, 4)?;
        encoder.value(offset + 296, self._pad18)?;
        encoder.array(offset + 304, &self.hand_x_bow, 4)?;
        encoder.value(offset + 312, self._pad19)?;
        encoder.array(offset + 320, &self.wand_with_orb, 4)?;
        encoder.value(offset + 328, self._pad20)?;
        encoder.array(offset + 336, &self.weapon_1_h_swing_with_shield, 4)?;
        encoder.value(offset + 344, self._pad21)?;
        encoder.array(offset + 352, &self.weapon_1_h_thrust_with_shield, 4)?;
        encoder.value(offset + 360, self._pad22)?;
        encoder.array(offset + 368, &self.hth_with_shield, 4)?;
        encoder.value(offset + 376, self._pad23)?;
        encoder.array(offset + 384, &self.weapon_2_h_swing_with_shield, 4)?;
        encoder.value(offset + 392, self._pad24)?;
        encoder.array(offset + 400, &self.weapon_2_h_thrust_with_shield, 4)?;
        encoder.value(offset + 408, self._pad25)?;
        encoder.array(offset + 416, &self.staff_with_shield, 4)?;
        encoder.value(offset + 424, self._pad26)?;
        encoder.array(offset + 432, &self.weapon_2_h_flail, 4)?;
        encoder.value(offset + 440, self._pad27)?;
        encoder.array(offset + 448, &self.weapon_2_h_flail_with_shield, 4)?;
        encoder.value(offset + 456, self._pad28)?;
        encoder.array(offset + 464, &self.on_horse, 4)?;
        encoder.value(offset + 472, self._pad29)?;
        Ok(())
    }
}
