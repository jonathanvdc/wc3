//! Native records for the explicitly supported Diablo III revisions.
//! Names follow the native field vocabulary; offset descriptors are resolved.
use super::super::codec::{copy_value, Decoder, Encoder};
use super::super::{Read, ReadError, RecordLayout, Write, WriteError};
use crate::model::common::Ownership;
use zerovec::{ule::AsULE, ZeroVec};

use super::{Bounds, Sphere, TagMapEntry, UberMaterial};
/// Native AppearanceLook record (64 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy)]
pub struct AppearanceLook {
    /// Native `szName` at byte 0.
    pub name: [u8; 64],
}
impl Default for AppearanceLook {
    fn default() -> Self {
        Self {
            name: [Default::default(); 64],
        }
    }
}

impl RecordLayout for AppearanceLook {
    const SIZE: usize = 64;
}
impl<'data> Read<'data> for AppearanceLook {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            name: decoder.value(offset)?,
        };
        Ok(result)
    }
}
impl Write for AppearanceLook {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.name)?;
        Ok(())
    }
}
impl AsULE for AppearanceLook {
    type ULE = <[u8; 64] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 64];
        copy_value(&mut bytes[0..64], self.name);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            name: ZeroVec::<[u8; 64]>::parse_bytes(&bytes[0..64])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native SubObjectAppearance record (248 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq)]
pub struct SubObjectAppearance<'data> {
    /// Native `dwUnknown00` at byte 0.
    pub unknown00: i32,
    /// Native `snoCloth` at byte 4.
    pub cloth: i32,
    /// Native `arShaderParams` at byte 8.
    pub shader_params: ZeroVec<'data, TagMapEntry>,
    /// Native `_pad0` at byte 16.
    pub _pad0: [u8; 8],
    /// Native `tMaterial` at byte 24.
    pub material: UberMaterial<'data>,
    /// Native `snoMaterial` at byte 128.
    pub material_sno: i32,
    /// Native `_pad1` at byte 132.
    pub _pad1: [u8; 116],
}
impl Default for SubObjectAppearance<'_> {
    fn default() -> Self {
        Self {
            unknown00: Default::default(),
            cloth: Default::default(),
            shader_params: Default::default(),
            _pad0: [Default::default(); 8],
            material: Default::default(),
            material_sno: Default::default(),
            _pad1: [Default::default(); 116],
        }
    }
}

impl RecordLayout for SubObjectAppearance<'_> {
    const SIZE: usize = 248;
}
impl<'data> Read<'data> for SubObjectAppearance<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            unknown00: decoder.value(offset)?,
            cloth: decoder.value(offset + 4)?,
            shader_params: decoder.array(offset + 8, 4)?,
            _pad0: decoder.value(offset + 16)?,
            material: UberMaterial::read(decoder, offset + 24)?,
            material_sno: decoder.value(offset + 128)?,
            _pad1: decoder.value(offset + 132)?,
        };
        Ok(result)
    }
}
impl Write for SubObjectAppearance<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.unknown00)?;
        encoder.value(offset + 4, self.cloth)?;
        encoder.array(offset + 8, &self.shader_params, 4)?;
        encoder.value(offset + 16, self._pad0)?;
        self.material.write(encoder, offset + 24)?;
        encoder.value(offset + 128, self.material_sno)?;
        encoder.value(offset + 132, self._pad1)?;
        Ok(())
    }
}

/// Native AppearanceMaterial record (144 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq)]
pub struct AppearanceMaterial<'data> {
    /// Native `szName` at byte 0.
    pub name: [u8; 128],
    /// Native `arVariants` at byte 128.
    pub variants: Vec<SubObjectAppearance<'data>>,
    /// Native `_pad0` at byte 136.
    pub _pad0: [u8; 8],
}
impl Default for AppearanceMaterial<'_> {
    fn default() -> Self {
        Self {
            name: [Default::default(); 128],
            variants: Default::default(),
            _pad0: [Default::default(); 8],
        }
    }
}

impl RecordLayout for AppearanceMaterial<'_> {
    const SIZE: usize = 144;
}
impl<'data> Read<'data> for AppearanceMaterial<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            name: decoder.value(offset)?,
            variants: decoder.records(offset + 128, 0)?,
            _pad0: decoder.value(offset + 136)?,
        };
        Ok(result)
    }
}
impl Write for AppearanceMaterial<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.name)?;
        encoder.records(offset + 128, &self.variants, 0)?;
        encoder.value(offset + 136, self._pad0)?;
        Ok(())
    }
}

/// Native PRSTransform record (32 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct Transform {
    /// Native `qRotation` at byte 0.
    pub rotation: [f32; 4],
    /// Native `vTranslation` at byte 16.
    pub translation: [f32; 3],
    /// Native `flScale` at byte 28.
    pub scale: f32,
}

impl RecordLayout for Transform {
    const SIZE: usize = 32;
}
impl<'data> Read<'data> for Transform {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            rotation: decoder.value(offset)?,
            translation: decoder.value(offset + 16)?,
            scale: decoder.value(offset + 28)?,
        };
        Ok(result)
    }
}
impl Write for Transform {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.rotation)?;
        encoder.value(offset + 16, self.translation)?;
        encoder.value(offset + 28, self.scale)?;
        Ok(())
    }
}
impl AsULE for Transform {
    type ULE = <[u8; 32] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 32];
        copy_value(&mut bytes[0..16], self.rotation);
        copy_value(&mut bytes[16..28], self.translation);
        copy_value(&mut bytes[28..32], self.scale);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            rotation: ZeroVec::<[f32; 4]>::parse_bytes(&bytes[0..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            translation: ZeroVec::<[f32; 3]>::parse_bytes(&bytes[16..28])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            scale: ZeroVec::<f32>::parse_bytes(&bytes[28..32])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native CollisionShape record (80 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Default)]
pub struct CollisionShape<'data> {
    /// Native `dwFlags` at byte 0.
    pub flags: i32,
    /// Native `eShapeType` at byte 4.
    pub shape_type: i32,
    /// Native `dwUnknown08` at byte 8.
    pub unknown08: i32,
    /// Native `nLodIndex` at byte 12.
    pub lod_index: i32,
    /// Native `flScaleX` at byte 16.
    pub scale_x: f32,
    /// Native `flScaleY` at byte 20.
    pub scale_y: f32,
    /// Native `flScaleZ` at byte 24.
    pub scale_z: f32,
    /// Native `arPolytopeData` at byte 28.
    pub polytope_data: ZeroVec<'data, u8>,
    /// Native `_pad0` at byte 36.
    pub _pad0: [u8; 12],
    /// Native `vPointA` at byte 48.
    pub point_a: [f32; 3],
    /// Native `vPointB` at byte 60.
    pub point_b: [f32; 3],
    /// Native `flRadius` at byte 72.
    pub radius: f32,
    /// Native `_pad1` at byte 76.
    pub _pad1: [u8; 4],
}

impl RecordLayout for CollisionShape<'_> {
    const SIZE: usize = 80;
}
impl<'data> Read<'data> for CollisionShape<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            flags: decoder.value(offset)?,
            shape_type: decoder.value(offset + 4)?,
            unknown08: decoder.value(offset + 8)?,
            lod_index: decoder.value(offset + 12)?,
            scale_x: decoder.value(offset + 16)?,
            scale_y: decoder.value(offset + 20)?,
            scale_z: decoder.value(offset + 24)?,
            polytope_data: decoder.array(offset + 28, 0)?,
            _pad0: decoder.value(offset + 36)?,
            point_a: decoder.value(offset + 48)?,
            point_b: decoder.value(offset + 60)?,
            radius: decoder.value(offset + 72)?,
            _pad1: decoder.value(offset + 76)?,
        };
        Ok(result)
    }
}
impl Write for CollisionShape<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.flags)?;
        encoder.value(offset + 4, self.shape_type)?;
        encoder.value(offset + 8, self.unknown08)?;
        encoder.value(offset + 12, self.lod_index)?;
        encoder.value(offset + 16, self.scale_x)?;
        encoder.value(offset + 20, self.scale_y)?;
        encoder.value(offset + 24, self.scale_z)?;
        encoder.array(offset + 28, &self.polytope_data, 0)?;
        encoder.value(offset + 36, self._pad0)?;
        encoder.value(offset + 48, self.point_a)?;
        encoder.value(offset + 60, self.point_b)?;
        encoder.value(offset + 72, self.radius)?;
        encoder.value(offset + 76, self._pad1)?;
        Ok(())
    }
}

/// Native PRTransform record (28 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct RigidTransform {
    /// Native `qRotation` at byte 0.
    pub rotation: [f32; 4],
    /// Native `vTranslation` at byte 16.
    pub translation: [f32; 3],
}

impl RecordLayout for RigidTransform {
    const SIZE: usize = 28;
}
impl<'data> Read<'data> for RigidTransform {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            rotation: decoder.value(offset)?,
            translation: decoder.value(offset + 16)?,
        };
        Ok(result)
    }
}
impl Write for RigidTransform {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.rotation)?;
        encoder.value(offset + 16, self.translation)?;
        Ok(())
    }
}
impl AsULE for RigidTransform {
    type ULE = <[u8; 28] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 28];
        copy_value(&mut bytes[0..16], self.rotation);
        copy_value(&mut bytes[16..28], self.translation);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            rotation: ZeroVec::<[f32; 4]>::parse_bytes(&bytes[0..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            translation: ZeroVec::<[f32; 3]>::parse_bytes(&bytes[16..28])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native ConstraintParameters record (292 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy)]
pub struct ConstraintParameters {
    /// Native `szName` at byte 0.
    pub name: [u8; 64],
    /// Native `dwFlags` at byte 64.
    pub flags: i32,
    /// Native `eConstraintType` at byte 68.
    pub constraint_type: i32,
    /// Native `nBoneIndexA` at byte 72.
    pub bone_index_a: i32,
    /// Native `nBoneIndexB` at byte 76.
    pub bone_index_b: i32,
    /// Native `tFrameA` at byte 80.
    pub frame_a: RigidTransform,
    /// Native `vUnknown34` at byte 108.
    pub unknown34: [f32; 3],
    /// Native `tFrameB` at byte 120.
    pub frame_b: RigidTransform,
    /// Native `tFrameC` at byte 148.
    pub frame_c: RigidTransform,
    /// Native `flLimitLower` at byte 176.
    pub limit_lower: f32,
    /// Native `flLimitUpper` at byte 180.
    pub limit_upper: f32,
    /// Native `flConeAngle` at byte 184.
    pub cone_angle: f32,
    /// Native `flTwistLower` at byte 188.
    pub twist_lower: f32,
    /// Native `flTwistUpper` at byte 192.
    pub twist_upper: f32,
    /// Native `flParam05` at byte 196.
    pub param05: f32,
    /// Native `flParam06` at byte 200.
    pub param06: f32,
    /// Native `flParam07` at byte 204.
    pub param07: f32,
    /// Native `flParam08` at byte 208.
    pub param08: f32,
    /// Native `flParam09` at byte 212.
    pub param09: f32,
    /// Native `flParam10` at byte 216.
    pub param10: f32,
    /// Native `flParam11` at byte 220.
    pub param11: f32,
    /// Native `flParam12` at byte 224.
    pub param12: f32,
    /// Native `szBreakEffect` at byte 228.
    pub break_effect: [u8; 64],
}
impl Default for ConstraintParameters {
    fn default() -> Self {
        Self {
            name: [Default::default(); 64],
            flags: Default::default(),
            constraint_type: Default::default(),
            bone_index_a: Default::default(),
            bone_index_b: Default::default(),
            frame_a: Default::default(),
            unknown34: [Default::default(); 3],
            frame_b: Default::default(),
            frame_c: Default::default(),
            limit_lower: Default::default(),
            limit_upper: Default::default(),
            cone_angle: Default::default(),
            twist_lower: Default::default(),
            twist_upper: Default::default(),
            param05: Default::default(),
            param06: Default::default(),
            param07: Default::default(),
            param08: Default::default(),
            param09: Default::default(),
            param10: Default::default(),
            param11: Default::default(),
            param12: Default::default(),
            break_effect: [Default::default(); 64],
        }
    }
}

impl RecordLayout for ConstraintParameters {
    const SIZE: usize = 292;
}
impl<'data> Read<'data> for ConstraintParameters {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            name: decoder.value(offset)?,
            flags: decoder.value(offset + 64)?,
            constraint_type: decoder.value(offset + 68)?,
            bone_index_a: decoder.value(offset + 72)?,
            bone_index_b: decoder.value(offset + 76)?,
            frame_a: decoder.value(offset + 80)?,
            unknown34: decoder.value(offset + 108)?,
            frame_b: decoder.value(offset + 120)?,
            frame_c: decoder.value(offset + 148)?,
            limit_lower: decoder.value(offset + 176)?,
            limit_upper: decoder.value(offset + 180)?,
            cone_angle: decoder.value(offset + 184)?,
            twist_lower: decoder.value(offset + 188)?,
            twist_upper: decoder.value(offset + 192)?,
            param05: decoder.value(offset + 196)?,
            param06: decoder.value(offset + 200)?,
            param07: decoder.value(offset + 204)?,
            param08: decoder.value(offset + 208)?,
            param09: decoder.value(offset + 212)?,
            param10: decoder.value(offset + 216)?,
            param11: decoder.value(offset + 220)?,
            param12: decoder.value(offset + 224)?,
            break_effect: decoder.value(offset + 228)?,
        };
        Ok(result)
    }
}
impl Write for ConstraintParameters {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.name)?;
        encoder.value(offset + 64, self.flags)?;
        encoder.value(offset + 68, self.constraint_type)?;
        encoder.value(offset + 72, self.bone_index_a)?;
        encoder.value(offset + 76, self.bone_index_b)?;
        encoder.value(offset + 80, self.frame_a)?;
        encoder.value(offset + 108, self.unknown34)?;
        encoder.value(offset + 120, self.frame_b)?;
        encoder.value(offset + 148, self.frame_c)?;
        encoder.value(offset + 176, self.limit_lower)?;
        encoder.value(offset + 180, self.limit_upper)?;
        encoder.value(offset + 184, self.cone_angle)?;
        encoder.value(offset + 188, self.twist_lower)?;
        encoder.value(offset + 192, self.twist_upper)?;
        encoder.value(offset + 196, self.param05)?;
        encoder.value(offset + 200, self.param06)?;
        encoder.value(offset + 204, self.param07)?;
        encoder.value(offset + 208, self.param08)?;
        encoder.value(offset + 212, self.param09)?;
        encoder.value(offset + 216, self.param10)?;
        encoder.value(offset + 220, self.param11)?;
        encoder.value(offset + 224, self.param12)?;
        encoder.value(offset + 228, self.break_effect)?;
        Ok(())
    }
}
impl AsULE for ConstraintParameters {
    type ULE = <[u8; 292] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 292];
        copy_value(&mut bytes[0..64], self.name);
        copy_value(&mut bytes[64..68], self.flags);
        copy_value(&mut bytes[68..72], self.constraint_type);
        copy_value(&mut bytes[72..76], self.bone_index_a);
        copy_value(&mut bytes[76..80], self.bone_index_b);
        copy_value(&mut bytes[80..108], self.frame_a);
        copy_value(&mut bytes[108..120], self.unknown34);
        copy_value(&mut bytes[120..148], self.frame_b);
        copy_value(&mut bytes[148..176], self.frame_c);
        copy_value(&mut bytes[176..180], self.limit_lower);
        copy_value(&mut bytes[180..184], self.limit_upper);
        copy_value(&mut bytes[184..188], self.cone_angle);
        copy_value(&mut bytes[188..192], self.twist_lower);
        copy_value(&mut bytes[192..196], self.twist_upper);
        copy_value(&mut bytes[196..200], self.param05);
        copy_value(&mut bytes[200..204], self.param06);
        copy_value(&mut bytes[204..208], self.param07);
        copy_value(&mut bytes[208..212], self.param08);
        copy_value(&mut bytes[212..216], self.param09);
        copy_value(&mut bytes[216..220], self.param10);
        copy_value(&mut bytes[220..224], self.param11);
        copy_value(&mut bytes[224..228], self.param12);
        copy_value(&mut bytes[228..292], self.break_effect);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            name: ZeroVec::<[u8; 64]>::parse_bytes(&bytes[0..64])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            flags: ZeroVec::<i32>::parse_bytes(&bytes[64..68])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            constraint_type: ZeroVec::<i32>::parse_bytes(&bytes[68..72])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            bone_index_a: ZeroVec::<i32>::parse_bytes(&bytes[72..76])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            bone_index_b: ZeroVec::<i32>::parse_bytes(&bytes[76..80])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            frame_a: ZeroVec::<RigidTransform>::parse_bytes(&bytes[80..108])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown34: ZeroVec::<[f32; 3]>::parse_bytes(&bytes[108..120])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            frame_b: ZeroVec::<RigidTransform>::parse_bytes(&bytes[120..148])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            frame_c: ZeroVec::<RigidTransform>::parse_bytes(&bytes[148..176])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            limit_lower: ZeroVec::<f32>::parse_bytes(&bytes[176..180])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            limit_upper: ZeroVec::<f32>::parse_bytes(&bytes[180..184])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            cone_angle: ZeroVec::<f32>::parse_bytes(&bytes[184..188])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            twist_lower: ZeroVec::<f32>::parse_bytes(&bytes[188..192])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            twist_upper: ZeroVec::<f32>::parse_bytes(&bytes[192..196])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            param05: ZeroVec::<f32>::parse_bytes(&bytes[196..200])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            param06: ZeroVec::<f32>::parse_bytes(&bytes[200..204])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            param07: ZeroVec::<f32>::parse_bytes(&bytes[204..208])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            param08: ZeroVec::<f32>::parse_bytes(&bytes[208..212])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            param09: ZeroVec::<f32>::parse_bytes(&bytes[212..216])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            param10: ZeroVec::<f32>::parse_bytes(&bytes[216..220])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            param11: ZeroVec::<f32>::parse_bytes(&bytes[220..224])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            param12: ZeroVec::<f32>::parse_bytes(&bytes[224..228])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            break_effect: ZeroVec::<[u8; 64]>::parse_bytes(&bytes[228..292])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native BoneStructure record (312 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq)]
pub struct BoneStructure<'data> {
    /// Native `szName` at byte 0.
    pub name: [u8; 64],
    /// Native `nParentIndex` at byte 64.
    pub parent_index: i32,
    /// Native `tBounds` at byte 68.
    pub bounds: Bounds,
    /// Native `tSphere` at byte 92.
    pub sphere: Sphere,
    /// Native `tTransform0` at byte 108.
    pub transform0: Transform,
    /// Native `tTransform1` at byte 140.
    pub transform1: Transform,
    /// Native `tTransform2` at byte 172.
    pub transform2: Transform,
    /// Native `tTransform3` at byte 204.
    pub transform3: Transform,
    /// Native `tTransform4` at byte 236.
    pub transform4: Transform,
    /// Native `dwCollisionShapeCount` at byte 268.
    pub collision_shape_count: i32,
    /// Native `arCollisionShapes` at byte 272.
    pub collision_shapes: Vec<CollisionShape<'data>>,
    /// Native `_pad0` at byte 280.
    pub _pad0: [u8; 8],
    /// Native `arConstraints` at byte 288.
    pub constraints: ZeroVec<'data, ConstraintParameters>,
    /// Native `_pad1` at byte 296.
    pub _pad1: [u8; 8],
    /// Native `snoParticle` at byte 304.
    pub particle: i32,
    /// Native `_pad2` at byte 308.
    pub _pad2: [u8; 4],
}
impl Default for BoneStructure<'_> {
    fn default() -> Self {
        Self {
            name: [Default::default(); 64],
            parent_index: Default::default(),
            bounds: Default::default(),
            sphere: Default::default(),
            transform0: Default::default(),
            transform1: Default::default(),
            transform2: Default::default(),
            transform3: Default::default(),
            transform4: Default::default(),
            collision_shape_count: Default::default(),
            collision_shapes: Default::default(),
            _pad0: [Default::default(); 8],
            constraints: Default::default(),
            _pad1: [Default::default(); 8],
            particle: Default::default(),
            _pad2: [Default::default(); 4],
        }
    }
}

impl RecordLayout for BoneStructure<'_> {
    const SIZE: usize = 312;
}
impl<'data> Read<'data> for BoneStructure<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            name: decoder.value(offset)?,
            parent_index: decoder.value(offset + 64)?,
            bounds: decoder.value(offset + 68)?,
            sphere: decoder.value(offset + 92)?,
            transform0: decoder.value(offset + 108)?,
            transform1: decoder.value(offset + 140)?,
            transform2: decoder.value(offset + 172)?,
            transform3: decoder.value(offset + 204)?,
            transform4: decoder.value(offset + 236)?,
            collision_shape_count: decoder.value(offset + 268)?,
            collision_shapes: decoder.records(offset + 272, 0)?,
            _pad0: decoder.value(offset + 280)?,
            constraints: decoder.array(offset + 288, 0)?,
            _pad1: decoder.value(offset + 296)?,
            particle: decoder.value(offset + 304)?,
            _pad2: decoder.value(offset + 308)?,
        };
        decoder.count(offset + 268, result.collision_shapes.len())?;
        Ok(result)
    }
}
impl Write for BoneStructure<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.name)?;
        encoder.value(offset + 64, self.parent_index)?;
        encoder.value(offset + 68, self.bounds)?;
        encoder.value(offset + 92, self.sphere)?;
        encoder.value(offset + 108, self.transform0)?;
        encoder.value(offset + 140, self.transform1)?;
        encoder.value(offset + 172, self.transform2)?;
        encoder.value(offset + 204, self.transform3)?;
        encoder.value(offset + 236, self.transform4)?;
        encoder.count(offset + 268, self.collision_shapes.len())?;
        encoder.records(offset + 272, &self.collision_shapes, 0)?;
        encoder.value(offset + 280, self._pad0)?;
        encoder.array(offset + 288, &self.constraints, 0)?;
        encoder.value(offset + 296, self._pad1)?;
        encoder.value(offset + 304, self.particle)?;
        encoder.value(offset + 308, self._pad2)?;
        Ok(())
    }
}

/// Native LookAtData record (92 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy)]
pub struct LookAtData {
    /// Native `nEnabled` at byte 0.
    pub enabled: i32,
    /// Native `szBoneName` at byte 4.
    pub bone_name: [u8; 64],
    /// Native `flYawMin` at byte 68.
    pub yaw_min: f32,
    /// Native `flYawMax` at byte 72.
    pub yaw_max: f32,
    /// Native `flPitchMin` at byte 76.
    pub pitch_min: f32,
    /// Native `flPitchMax` at byte 80.
    pub pitch_max: f32,
    /// Native `flAngularVelocity` at byte 84.
    pub angular_velocity: f32,
    /// Native `flUnknown58` at byte 88.
    pub unknown58: f32,
}
impl Default for LookAtData {
    fn default() -> Self {
        Self {
            enabled: Default::default(),
            bone_name: [Default::default(); 64],
            yaw_min: Default::default(),
            yaw_max: Default::default(),
            pitch_min: Default::default(),
            pitch_max: Default::default(),
            angular_velocity: Default::default(),
            unknown58: Default::default(),
        }
    }
}

impl RecordLayout for LookAtData {
    const SIZE: usize = 92;
}
impl<'data> Read<'data> for LookAtData {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            enabled: decoder.value(offset)?,
            bone_name: decoder.value(offset + 4)?,
            yaw_min: decoder.value(offset + 68)?,
            yaw_max: decoder.value(offset + 72)?,
            pitch_min: decoder.value(offset + 76)?,
            pitch_max: decoder.value(offset + 80)?,
            angular_velocity: decoder.value(offset + 84)?,
            unknown58: decoder.value(offset + 88)?,
        };
        Ok(result)
    }
}
impl Write for LookAtData {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.enabled)?;
        encoder.value(offset + 4, self.bone_name)?;
        encoder.value(offset + 68, self.yaw_min)?;
        encoder.value(offset + 72, self.yaw_max)?;
        encoder.value(offset + 76, self.pitch_min)?;
        encoder.value(offset + 80, self.pitch_max)?;
        encoder.value(offset + 84, self.angular_velocity)?;
        encoder.value(offset + 88, self.unknown58)?;
        Ok(())
    }
}
impl AsULE for LookAtData {
    type ULE = <[u8; 92] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 92];
        copy_value(&mut bytes[0..4], self.enabled);
        copy_value(&mut bytes[4..68], self.bone_name);
        copy_value(&mut bytes[68..72], self.yaw_min);
        copy_value(&mut bytes[72..76], self.yaw_max);
        copy_value(&mut bytes[76..80], self.pitch_min);
        copy_value(&mut bytes[80..84], self.pitch_max);
        copy_value(&mut bytes[84..88], self.angular_velocity);
        copy_value(&mut bytes[88..92], self.unknown58);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            enabled: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            bone_name: ZeroVec::<[u8; 64]>::parse_bytes(&bytes[4..68])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            yaw_min: ZeroVec::<f32>::parse_bytes(&bytes[68..72])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            yaw_max: ZeroVec::<f32>::parse_bytes(&bytes[72..76])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            pitch_min: ZeroVec::<f32>::parse_bytes(&bytes[76..80])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            pitch_max: ZeroVec::<f32>::parse_bytes(&bytes[80..84])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            angular_velocity: ZeroVec::<f32>::parse_bytes(&bytes[84..88])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown58: ZeroVec::<f32>::parse_bytes(&bytes[88..92])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native FatVertex record (44 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct FatVertex {
    /// Native `vPosition` at byte 0.
    pub position: [f32; 3],
    /// Native `dwNormal` at byte 12.
    pub normal: u32,
    /// Native `dwColor` at byte 16.
    pub color: u32,
    /// Native `dwAuxColor` at byte 20.
    pub aux_color: u32,
    /// Native `dwTexCoord0` at byte 24.
    pub tex_coord0: u32,
    /// Native `dwTexCoord1` at byte 28.
    pub tex_coord1: u32,
    /// Native `dwTangent` at byte 32.
    pub tangent: u32,
    /// Native `dwBinormal` at byte 36.
    pub binormal: u32,
    /// Native `dwClothVertexIndex` at byte 40.
    pub cloth_vertex_index: u32,
}

impl RecordLayout for FatVertex {
    const SIZE: usize = 44;
}
impl<'data> Read<'data> for FatVertex {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            position: decoder.value(offset)?,
            normal: decoder.value(offset + 12)?,
            color: decoder.value(offset + 16)?,
            aux_color: decoder.value(offset + 20)?,
            tex_coord0: decoder.value(offset + 24)?,
            tex_coord1: decoder.value(offset + 28)?,
            tangent: decoder.value(offset + 32)?,
            binormal: decoder.value(offset + 36)?,
            cloth_vertex_index: decoder.value(offset + 40)?,
        };
        Ok(result)
    }
}
impl Write for FatVertex {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.position)?;
        encoder.value(offset + 12, self.normal)?;
        encoder.value(offset + 16, self.color)?;
        encoder.value(offset + 20, self.aux_color)?;
        encoder.value(offset + 24, self.tex_coord0)?;
        encoder.value(offset + 28, self.tex_coord1)?;
        encoder.value(offset + 32, self.tangent)?;
        encoder.value(offset + 36, self.binormal)?;
        encoder.value(offset + 40, self.cloth_vertex_index)?;
        Ok(())
    }
}
impl AsULE for FatVertex {
    type ULE = <[u8; 44] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 44];
        copy_value(&mut bytes[0..12], self.position);
        copy_value(&mut bytes[12..16], self.normal);
        copy_value(&mut bytes[16..20], self.color);
        copy_value(&mut bytes[20..24], self.aux_color);
        copy_value(&mut bytes[24..28], self.tex_coord0);
        copy_value(&mut bytes[28..32], self.tex_coord1);
        copy_value(&mut bytes[32..36], self.tangent);
        copy_value(&mut bytes[36..40], self.binormal);
        copy_value(&mut bytes[40..44], self.cloth_vertex_index);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            position: ZeroVec::<[f32; 3]>::parse_bytes(&bytes[0..12])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            normal: ZeroVec::<u32>::parse_bytes(&bytes[12..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            color: ZeroVec::<u32>::parse_bytes(&bytes[16..20])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            aux_color: ZeroVec::<u32>::parse_bytes(&bytes[20..24])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            tex_coord0: ZeroVec::<u32>::parse_bytes(&bytes[24..28])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            tex_coord1: ZeroVec::<u32>::parse_bytes(&bytes[28..32])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            tangent: ZeroVec::<u32>::parse_bytes(&bytes[32..36])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            binormal: ZeroVec::<u32>::parse_bytes(&bytes[36..40])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            cloth_vertex_index: ZeroVec::<u32>::parse_bytes(&bytes[40..44])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native Influence record (8 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct Influence {
    /// Native `nBoneIndex` at byte 0.
    pub bone_index: i32,
    /// Native `flWeight` at byte 4.
    pub weight: f32,
}

impl RecordLayout for Influence {
    const SIZE: usize = 8;
}
impl<'data> Read<'data> for Influence {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            bone_index: decoder.value(offset)?,
            weight: decoder.value(offset + 4)?,
        };
        Ok(result)
    }
}
impl Write for Influence {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.bone_index)?;
        encoder.value(offset + 4, self.weight)?;
        Ok(())
    }
}
impl AsULE for Influence {
    type ULE = <[u8; 8] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 8];
        copy_value(&mut bytes[0..4], self.bone_index);
        copy_value(&mut bytes[4..8], self.weight);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            bone_index: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            weight: ZeroVec::<f32>::parse_bytes(&bytes[4..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native VertInfluences record (24 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct VertexInfluences {
    /// Native `tInfluence0` at byte 0.
    pub influence0: Influence,
    /// Native `tInfluence1` at byte 8.
    pub influence1: Influence,
    /// Native `tInfluence2` at byte 16.
    pub influence2: Influence,
}

impl RecordLayout for VertexInfluences {
    const SIZE: usize = 24;
}
impl<'data> Read<'data> for VertexInfluences {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            influence0: decoder.value(offset)?,
            influence1: decoder.value(offset + 8)?,
            influence2: decoder.value(offset + 16)?,
        };
        Ok(result)
    }
}
impl Write for VertexInfluences {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.influence0)?;
        encoder.value(offset + 8, self.influence1)?;
        encoder.value(offset + 16, self.influence2)?;
        Ok(())
    }
}
impl AsULE for VertexInfluences {
    type ULE = <[u8; 24] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 24];
        copy_value(&mut bytes[0..8], self.influence0);
        copy_value(&mut bytes[8..16], self.influence1);
        copy_value(&mut bytes[16..24], self.influence2);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            influence0: ZeroVec::<Influence>::parse_bytes(&bytes[0..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            influence1: ZeroVec::<Influence>::parse_bytes(&bytes[8..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            influence2: ZeroVec::<Influence>::parse_bytes(&bytes[16..24])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native ClothVertex record (84 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct ClothVertex {
    /// Native `vPosition` at byte 0.
    pub position: [f32; 3],
    /// Native `vNormal` at byte 12.
    pub normal: [f32; 3],
    /// Native `vPrevPosition` at byte 24.
    pub prev_position: [f32; 3],
    /// Native `vVelocity` at byte 36.
    pub velocity: [f32; 3],
    /// Native `flInvMass` at byte 48.
    pub inv_mass: f32,
    /// Native `nCollisionProxyVertex` at byte 52.
    pub collision_proxy_vertex: i32,
    /// Native `nPinDistance` at byte 56.
    pub pin_distance: i32,
    /// Native `nMeshVertexIndex` at byte 60.
    pub mesh_vertex_index: i32,
    /// Native `nDrivingBone` at byte 64.
    pub driving_bone: i32,
    /// Native `vContactNormal` at byte 68.
    pub contact_normal: [f32; 3],
    /// Native `nContactFlag` at byte 80.
    pub contact_flag: i32,
}

impl RecordLayout for ClothVertex {
    const SIZE: usize = 84;
}
impl<'data> Read<'data> for ClothVertex {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            position: decoder.value(offset)?,
            normal: decoder.value(offset + 12)?,
            prev_position: decoder.value(offset + 24)?,
            velocity: decoder.value(offset + 36)?,
            inv_mass: decoder.value(offset + 48)?,
            collision_proxy_vertex: decoder.value(offset + 52)?,
            pin_distance: decoder.value(offset + 56)?,
            mesh_vertex_index: decoder.value(offset + 60)?,
            driving_bone: decoder.value(offset + 64)?,
            contact_normal: decoder.value(offset + 68)?,
            contact_flag: decoder.value(offset + 80)?,
        };
        Ok(result)
    }
}
impl Write for ClothVertex {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.position)?;
        encoder.value(offset + 12, self.normal)?;
        encoder.value(offset + 24, self.prev_position)?;
        encoder.value(offset + 36, self.velocity)?;
        encoder.value(offset + 48, self.inv_mass)?;
        encoder.value(offset + 52, self.collision_proxy_vertex)?;
        encoder.value(offset + 56, self.pin_distance)?;
        encoder.value(offset + 60, self.mesh_vertex_index)?;
        encoder.value(offset + 64, self.driving_bone)?;
        encoder.value(offset + 68, self.contact_normal)?;
        encoder.value(offset + 80, self.contact_flag)?;
        Ok(())
    }
}
impl AsULE for ClothVertex {
    type ULE = <[u8; 84] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 84];
        copy_value(&mut bytes[0..12], self.position);
        copy_value(&mut bytes[12..24], self.normal);
        copy_value(&mut bytes[24..36], self.prev_position);
        copy_value(&mut bytes[36..48], self.velocity);
        copy_value(&mut bytes[48..52], self.inv_mass);
        copy_value(&mut bytes[52..56], self.collision_proxy_vertex);
        copy_value(&mut bytes[56..60], self.pin_distance);
        copy_value(&mut bytes[60..64], self.mesh_vertex_index);
        copy_value(&mut bytes[64..68], self.driving_bone);
        copy_value(&mut bytes[68..80], self.contact_normal);
        copy_value(&mut bytes[80..84], self.contact_flag);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            position: ZeroVec::<[f32; 3]>::parse_bytes(&bytes[0..12])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            normal: ZeroVec::<[f32; 3]>::parse_bytes(&bytes[12..24])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            prev_position: ZeroVec::<[f32; 3]>::parse_bytes(&bytes[24..36])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            velocity: ZeroVec::<[f32; 3]>::parse_bytes(&bytes[36..48])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            inv_mass: ZeroVec::<f32>::parse_bytes(&bytes[48..52])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            collision_proxy_vertex: ZeroVec::<i32>::parse_bytes(&bytes[52..56])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            pin_distance: ZeroVec::<i32>::parse_bytes(&bytes[56..60])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            mesh_vertex_index: ZeroVec::<i32>::parse_bytes(&bytes[60..64])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            driving_bone: ZeroVec::<i32>::parse_bytes(&bytes[64..68])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            contact_normal: ZeroVec::<[f32; 3]>::parse_bytes(&bytes[68..80])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            contact_flag: ZeroVec::<i32>::parse_bytes(&bytes[80..84])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native ClothFace record (28 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct ClothFace {
    /// Native `vNormal` at byte 0.
    pub normal: [f32; 3],
    /// Native `flRestArea` at byte 12.
    pub rest_area: f32,
    /// Native `nVertex0` at byte 16.
    pub vertex0: i32,
    /// Native `nVertex1` at byte 20.
    pub vertex1: i32,
    /// Native `nVertex2` at byte 24.
    pub vertex2: i32,
}

impl RecordLayout for ClothFace {
    const SIZE: usize = 28;
}
impl<'data> Read<'data> for ClothFace {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            normal: decoder.value(offset)?,
            rest_area: decoder.value(offset + 12)?,
            vertex0: decoder.value(offset + 16)?,
            vertex1: decoder.value(offset + 20)?,
            vertex2: decoder.value(offset + 24)?,
        };
        Ok(result)
    }
}
impl Write for ClothFace {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.normal)?;
        encoder.value(offset + 12, self.rest_area)?;
        encoder.value(offset + 16, self.vertex0)?;
        encoder.value(offset + 20, self.vertex1)?;
        encoder.value(offset + 24, self.vertex2)?;
        Ok(())
    }
}
impl AsULE for ClothFace {
    type ULE = <[u8; 28] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 28];
        copy_value(&mut bytes[0..12], self.normal);
        copy_value(&mut bytes[12..16], self.rest_area);
        copy_value(&mut bytes[16..20], self.vertex0);
        copy_value(&mut bytes[20..24], self.vertex1);
        copy_value(&mut bytes[24..28], self.vertex2);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            normal: ZeroVec::<[f32; 3]>::parse_bytes(&bytes[0..12])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            rest_area: ZeroVec::<f32>::parse_bytes(&bytes[12..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            vertex0: ZeroVec::<i32>::parse_bytes(&bytes[16..20])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            vertex1: ZeroVec::<i32>::parse_bytes(&bytes[20..24])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            vertex2: ZeroVec::<i32>::parse_bytes(&bytes[24..28])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native ClothStaple record (28 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct ClothStaple {
    /// Native `nVertexIndex` at byte 0.
    pub vertex_index: i32,
    /// Native `nBoneIndex0` at byte 4.
    pub bone_index0: i32,
    /// Native `nBoneIndex1` at byte 8.
    pub bone_index1: i32,
    /// Native `nBoneIndex2` at byte 12.
    pub bone_index2: i32,
    /// Native `flWeight0` at byte 16.
    pub weight0: f32,
    /// Native `flWeight1` at byte 20.
    pub weight1: f32,
    /// Native `flWeight2` at byte 24.
    pub weight2: f32,
}

impl RecordLayout for ClothStaple {
    const SIZE: usize = 28;
}
impl<'data> Read<'data> for ClothStaple {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            vertex_index: decoder.value(offset)?,
            bone_index0: decoder.value(offset + 4)?,
            bone_index1: decoder.value(offset + 8)?,
            bone_index2: decoder.value(offset + 12)?,
            weight0: decoder.value(offset + 16)?,
            weight1: decoder.value(offset + 20)?,
            weight2: decoder.value(offset + 24)?,
        };
        Ok(result)
    }
}
impl Write for ClothStaple {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.vertex_index)?;
        encoder.value(offset + 4, self.bone_index0)?;
        encoder.value(offset + 8, self.bone_index1)?;
        encoder.value(offset + 12, self.bone_index2)?;
        encoder.value(offset + 16, self.weight0)?;
        encoder.value(offset + 20, self.weight1)?;
        encoder.value(offset + 24, self.weight2)?;
        Ok(())
    }
}
impl AsULE for ClothStaple {
    type ULE = <[u8; 28] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 28];
        copy_value(&mut bytes[0..4], self.vertex_index);
        copy_value(&mut bytes[4..8], self.bone_index0);
        copy_value(&mut bytes[8..12], self.bone_index1);
        copy_value(&mut bytes[12..16], self.bone_index2);
        copy_value(&mut bytes[16..20], self.weight0);
        copy_value(&mut bytes[20..24], self.weight1);
        copy_value(&mut bytes[24..28], self.weight2);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            vertex_index: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            bone_index0: ZeroVec::<i32>::parse_bytes(&bytes[4..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            bone_index1: ZeroVec::<i32>::parse_bytes(&bytes[8..12])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            bone_index2: ZeroVec::<i32>::parse_bytes(&bytes[12..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            weight0: ZeroVec::<f32>::parse_bytes(&bytes[16..20])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            weight1: ZeroVec::<f32>::parse_bytes(&bytes[20..24])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            weight2: ZeroVec::<f32>::parse_bytes(&bytes[24..28])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native ClothConstraint record (24 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct ClothConstraint {
    /// Native `nVertex0` at byte 0.
    pub vertex0: i32,
    /// Native `nVertex1` at byte 4.
    pub vertex1: i32,
    /// Native `flRestLengthSq` at byte 8.
    pub rest_length_sq: f32,
    /// Native `flWeight0` at byte 12.
    pub weight0: f32,
    /// Native `flWeight1` at byte 16.
    pub weight1: f32,
    /// Native `flStiffnessBlend` at byte 20.
    pub stiffness_blend: f32,
}

impl RecordLayout for ClothConstraint {
    const SIZE: usize = 24;
}
impl<'data> Read<'data> for ClothConstraint {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            vertex0: decoder.value(offset)?,
            vertex1: decoder.value(offset + 4)?,
            rest_length_sq: decoder.value(offset + 8)?,
            weight0: decoder.value(offset + 12)?,
            weight1: decoder.value(offset + 16)?,
            stiffness_blend: decoder.value(offset + 20)?,
        };
        Ok(result)
    }
}
impl Write for ClothConstraint {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.vertex0)?;
        encoder.value(offset + 4, self.vertex1)?;
        encoder.value(offset + 8, self.rest_length_sq)?;
        encoder.value(offset + 12, self.weight0)?;
        encoder.value(offset + 16, self.weight1)?;
        encoder.value(offset + 20, self.stiffness_blend)?;
        Ok(())
    }
}
impl AsULE for ClothConstraint {
    type ULE = <[u8; 24] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 24];
        copy_value(&mut bytes[0..4], self.vertex0);
        copy_value(&mut bytes[4..8], self.vertex1);
        copy_value(&mut bytes[8..12], self.rest_length_sq);
        copy_value(&mut bytes[12..16], self.weight0);
        copy_value(&mut bytes[16..20], self.weight1);
        copy_value(&mut bytes[20..24], self.stiffness_blend);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            vertex0: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            vertex1: ZeroVec::<i32>::parse_bytes(&bytes[4..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            rest_length_sq: ZeroVec::<f32>::parse_bytes(&bytes[8..12])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            weight0: ZeroVec::<f32>::parse_bytes(&bytes[12..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            weight1: ZeroVec::<f32>::parse_bytes(&bytes[16..20])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            stiffness_blend: ZeroVec::<f32>::parse_bytes(&bytes[20..24])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native ClothStructure record (128 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Default)]
pub struct ClothStructure<'data> {
    /// Native `dwVertexCount` at byte 0.
    pub vertex_count: i32,
    /// Native `arVertices` at byte 4.
    pub vertices: ZeroVec<'data, ClothVertex>,
    /// Native `_pad0` at byte 12.
    pub _pad0: [u8; 12],
    /// Native `dwFaceCount` at byte 24.
    pub face_count: i32,
    /// Native `arFaces` at byte 28.
    pub faces: ZeroVec<'data, ClothFace>,
    /// Native `_pad1` at byte 36.
    pub _pad1: [u8; 12],
    /// Native `dwStapleCount` at byte 48.
    pub staple_count: i32,
    /// Native `arStaples` at byte 52.
    pub staples: ZeroVec<'data, ClothStaple>,
    /// Native `_pad2` at byte 60.
    pub _pad2: [u8; 12],
    /// Native `dwStretchConstraintCount` at byte 72.
    pub stretch_constraint_count: i32,
    /// Native `arStretchConstraints` at byte 76.
    pub stretch_constraints: ZeroVec<'data, ClothConstraint>,
    /// Native `_pad3` at byte 84.
    pub _pad3: [u8; 12],
    /// Native `dwBendConstraintCount` at byte 96.
    pub bend_constraint_count: i32,
    /// Native `arBendConstraints` at byte 100.
    pub bend_constraints: ZeroVec<'data, ClothConstraint>,
    /// Native `_pad4` at byte 108.
    pub _pad4: [u8; 12],
    /// Native `dwDrivingBoneCount` at byte 120.
    pub driving_bone_count: i32,
    /// Native `flExternalForceScale` at byte 124.
    pub external_force_scale: f32,
}

impl RecordLayout for ClothStructure<'_> {
    const SIZE: usize = 128;
}
impl<'data> Read<'data> for ClothStructure<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            vertex_count: decoder.value(offset)?,
            vertices: decoder.array(offset + 4, 0)?,
            _pad0: decoder.value(offset + 12)?,
            face_count: decoder.value(offset + 24)?,
            faces: decoder.array(offset + 28, 0)?,
            _pad1: decoder.value(offset + 36)?,
            staple_count: decoder.value(offset + 48)?,
            staples: decoder.array(offset + 52, 0)?,
            _pad2: decoder.value(offset + 60)?,
            stretch_constraint_count: decoder.value(offset + 72)?,
            stretch_constraints: decoder.array(offset + 76, 0)?,
            _pad3: decoder.value(offset + 84)?,
            bend_constraint_count: decoder.value(offset + 96)?,
            bend_constraints: decoder.array(offset + 100, 0)?,
            _pad4: decoder.value(offset + 108)?,
            driving_bone_count: decoder.value(offset + 120)?,
            external_force_scale: decoder.value(offset + 124)?,
        };
        Ok(result)
    }
}
impl Write for ClothStructure<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.vertex_count)?;
        encoder.array(offset + 4, &self.vertices, 0)?;
        encoder.value(offset + 12, self._pad0)?;
        encoder.value(offset + 24, self.face_count)?;
        encoder.array(offset + 28, &self.faces, 0)?;
        encoder.value(offset + 36, self._pad1)?;
        encoder.value(offset + 48, self.staple_count)?;
        encoder.array(offset + 52, &self.staples, 0)?;
        encoder.value(offset + 60, self._pad2)?;
        encoder.value(offset + 72, self.stretch_constraint_count)?;
        encoder.array(offset + 76, &self.stretch_constraints, 0)?;
        encoder.value(offset + 84, self._pad3)?;
        encoder.value(offset + 96, self.bend_constraint_count)?;
        encoder.array(offset + 100, &self.bend_constraints, 0)?;
        encoder.value(offset + 108, self._pad4)?;
        encoder.value(offset + 120, self.driving_bone_count)?;
        encoder.value(offset + 124, self.external_force_scale)?;
        Ok(())
    }
}

/// Native SubObject record (400 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq)]
pub struct SubObject<'data> {
    /// Native `dwVertexFormat` at byte 0.
    pub vertex_format: i32,
    /// Native `dwVertexCount` at byte 4.
    pub vertex_count: i32,
    /// Native `arVertices` at byte 8.
    pub vertices: ZeroVec<'data, FatVertex>,
    /// Native `_pad0` at byte 16.
    pub _pad0: [u8; 8],
    /// Native `arVertexInfluences` at byte 24.
    pub vertex_influences: ZeroVec<'data, VertexInfluences>,
    /// Native `_pad1` at byte 32.
    pub _pad1: [u8; 8],
    /// Native `dwIndexCount` at byte 40.
    pub index_count: i32,
    /// Native `arIndices` at byte 44.
    pub indices: ZeroVec<'data, u16>,
    /// Native `_pad2` at byte 52.
    pub _pad2: [u8; 12],
    /// Native `arClothData` at byte 64.
    pub cloth_data: Vec<ClothStructure<'data>>,
    /// Native `_pad3` at byte 72.
    pub _pad3: [u8; 8],
    /// Native `snoSurface` at byte 80.
    pub surface: i32,
    /// Native `nBoneIndex` at byte 84.
    pub bone_index: i32,
    /// Native `flUnknown58` at byte 88.
    pub unknown58: f32,
    /// Native `szName` at byte 92.
    pub name: [u8; 128],
    /// Native `szMaterialName` at byte 220.
    pub material_name: [u8; 128],
    /// Native `tBounds` at byte 348.
    pub bounds: Bounds,
    /// Native `dwCollisionShapeCount` at byte 372.
    pub collision_shape_count: i32,
    /// Native `arCollisionShapes` at byte 376.
    pub collision_shapes: Vec<CollisionShape<'data>>,
    /// Native `_pad4` at byte 384.
    pub _pad4: [u8; 16],
}
impl Default for SubObject<'_> {
    fn default() -> Self {
        Self {
            vertex_format: Default::default(),
            vertex_count: Default::default(),
            vertices: Default::default(),
            _pad0: [Default::default(); 8],
            vertex_influences: Default::default(),
            _pad1: [Default::default(); 8],
            index_count: Default::default(),
            indices: Default::default(),
            _pad2: [Default::default(); 12],
            cloth_data: Default::default(),
            _pad3: [Default::default(); 8],
            surface: Default::default(),
            bone_index: Default::default(),
            unknown58: Default::default(),
            name: [Default::default(); 128],
            material_name: [Default::default(); 128],
            bounds: Default::default(),
            collision_shape_count: Default::default(),
            collision_shapes: Default::default(),
            _pad4: [Default::default(); 16],
        }
    }
}

impl RecordLayout for SubObject<'_> {
    const SIZE: usize = 400;
}
impl<'data> Read<'data> for SubObject<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            vertex_format: decoder.value(offset)?,
            vertex_count: decoder.value(offset + 4)?,
            vertices: decoder.array(offset + 8, 0)?,
            _pad0: decoder.value(offset + 16)?,
            vertex_influences: decoder.array(offset + 24, 0)?,
            _pad1: decoder.value(offset + 32)?,
            index_count: decoder.value(offset + 40)?,
            indices: decoder.array(offset + 44, 0)?,
            _pad2: decoder.value(offset + 52)?,
            cloth_data: decoder.records(offset + 64, 0)?,
            _pad3: decoder.value(offset + 72)?,
            surface: decoder.value(offset + 80)?,
            bone_index: decoder.value(offset + 84)?,
            unknown58: decoder.value(offset + 88)?,
            name: decoder.value(offset + 92)?,
            material_name: decoder.value(offset + 220)?,
            bounds: decoder.value(offset + 348)?,
            collision_shape_count: decoder.value(offset + 372)?,
            collision_shapes: decoder.records(offset + 376, 0)?,
            _pad4: decoder.value(offset + 384)?,
        };
        decoder.count(offset + 4, result.vertices.len())?;
        decoder.count(offset + 40, result.indices.len())?;
        decoder.count(offset + 372, result.collision_shapes.len())?;
        if result.vertex_format as u32 & 0x18000 == 0x8000 {
            return Err(ReadError::VertexFormat(result.vertex_format as u32));
        }
        Ok(result)
    }
}
impl Write for SubObject<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        if self.vertex_format as u32 & 0x18000 == 0x8000 {
            return Err(WriteError::VertexFormat(self.vertex_format as u32));
        }
        encoder.value(offset, self.vertex_format)?;
        encoder.count(offset + 4, self.vertices.len())?;
        encoder.array(offset + 8, &self.vertices, 0)?;
        encoder.value(offset + 16, self._pad0)?;
        encoder.array(offset + 24, &self.vertex_influences, 0)?;
        encoder.value(offset + 32, self._pad1)?;
        encoder.count(offset + 40, self.indices.len())?;
        encoder.array(offset + 44, &self.indices, 0)?;
        encoder.value(offset + 52, self._pad2)?;
        encoder.records(offset + 64, &self.cloth_data, 0)?;
        encoder.value(offset + 72, self._pad3)?;
        encoder.value(offset + 80, self.surface)?;
        encoder.value(offset + 84, self.bone_index)?;
        encoder.value(offset + 88, self.unknown58)?;
        encoder.value(offset + 92, self.name)?;
        encoder.value(offset + 220, self.material_name)?;
        encoder.value(offset + 348, self.bounds)?;
        encoder.count(offset + 372, self.collision_shapes.len())?;
        encoder.records(offset + 376, &self.collision_shapes, 0)?;
        encoder.value(offset + 384, self._pad4)?;
        Ok(())
    }
}

/// Native GeoSet record (24 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Default)]
pub struct GeoSet<'data> {
    /// Native `dwSubObjectCount` at byte 0.
    pub sub_object_count: i32,
    /// Native `arSubObjects` at byte 4.
    pub sub_objects: Vec<SubObject<'data>>,
    /// Native `_pad0` at byte 12.
    pub _pad0: [u8; 12],
}

impl RecordLayout for GeoSet<'_> {
    const SIZE: usize = 24;
}
impl<'data> Read<'data> for GeoSet<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            sub_object_count: decoder.value(offset)?,
            sub_objects: decoder.records(offset + 4, 0)?,
            _pad0: decoder.value(offset + 12)?,
        };
        decoder.count(offset, result.sub_objects.len())?;
        Ok(result)
    }
}
impl Write for GeoSet<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.count(offset, self.sub_objects.len())?;
        encoder.records(offset + 4, &self.sub_objects, 0)?;
        encoder.value(offset + 12, self._pad0)?;
        Ok(())
    }
}

/// Native Hardpoint record (96 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy)]
pub struct Hardpoint {
    /// Native `szName` at byte 0.
    pub name: [u8; 64],
    /// Native `nBoneIndex` at byte 64.
    pub bone_index: i32,
    /// Native `tTransform` at byte 68.
    pub transform: RigidTransform,
}
impl Default for Hardpoint {
    fn default() -> Self {
        Self {
            name: [Default::default(); 64],
            bone_index: Default::default(),
            transform: Default::default(),
        }
    }
}

impl RecordLayout for Hardpoint {
    const SIZE: usize = 96;
}
impl<'data> Read<'data> for Hardpoint {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            name: decoder.value(offset)?,
            bone_index: decoder.value(offset + 64)?,
            transform: decoder.value(offset + 68)?,
        };
        Ok(result)
    }
}
impl Write for Hardpoint {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.name)?;
        encoder.value(offset + 64, self.bone_index)?;
        encoder.value(offset + 68, self.transform)?;
        Ok(())
    }
}
impl AsULE for Hardpoint {
    type ULE = <[u8; 96] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 96];
        copy_value(&mut bytes[0..64], self.name);
        copy_value(&mut bytes[64..68], self.bone_index);
        copy_value(&mut bytes[68..96], self.transform);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            name: ZeroVec::<[u8; 64]>::parse_bytes(&bytes[0..64])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            bone_index: ZeroVec::<i32>::parse_bytes(&bytes[64..68])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            transform: ZeroVec::<RigidTransform>::parse_bytes(&bytes[68..96])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native CollisionCapsule record (104 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct CollisionCapsule {
    /// Native `flRadius` at byte 0.
    pub radius: f32,
    /// Native `flLength` at byte 4.
    pub length: f32,
    /// Native `tHardpoint` at byte 8.
    pub hardpoint: Hardpoint,
}

impl RecordLayout for CollisionCapsule {
    const SIZE: usize = 104;
}
impl<'data> Read<'data> for CollisionCapsule {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            radius: decoder.value(offset)?,
            length: decoder.value(offset + 4)?,
            hardpoint: decoder.value(offset + 8)?,
        };
        Ok(result)
    }
}
impl Write for CollisionCapsule {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.radius)?;
        encoder.value(offset + 4, self.length)?;
        encoder.value(offset + 8, self.hardpoint)?;
        Ok(())
    }
}
impl AsULE for CollisionCapsule {
    type ULE = <[u8; 104] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 104];
        copy_value(&mut bytes[0..4], self.radius);
        copy_value(&mut bytes[4..8], self.length);
        copy_value(&mut bytes[8..104], self.hardpoint);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            radius: ZeroVec::<f32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            length: ZeroVec::<f32>::parse_bytes(&bytes[4..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            hardpoint: ZeroVec::<Hardpoint>::parse_bytes(&bytes[8..104])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native OctreeCube record (16 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct OctreeCube {
    /// Native `vMin` at byte 0.
    pub min: [f32; 3],
    /// Native `flSize` at byte 12.
    pub size: f32,
}

impl RecordLayout for OctreeCube {
    const SIZE: usize = 16;
}
impl<'data> Read<'data> for OctreeCube {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            min: decoder.value(offset)?,
            size: decoder.value(offset + 12)?,
        };
        Ok(result)
    }
}
impl Write for OctreeCube {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.min)?;
        encoder.value(offset + 12, self.size)?;
        Ok(())
    }
}
impl AsULE for OctreeCube {
    type ULE = <[u8; 16] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 16];
        copy_value(&mut bytes[0..12], self.min);
        copy_value(&mut bytes[12..16], self.size);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            min: ZeroVec::<[f32; 3]>::parse_bytes(&bytes[0..12])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            size: ZeroVec::<f32>::parse_bytes(&bytes[12..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native OctreeNode record (48 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct OctreeNode {
    /// Native `tCube` at byte 0.
    pub cube: OctreeCube,
    /// Native `nChild0` at byte 16.
    pub child0: i32,
    /// Native `nChild1` at byte 20.
    pub child1: i32,
    /// Native `nChild2` at byte 24.
    pub child2: i32,
    /// Native `nChild3` at byte 28.
    pub child3: i32,
    /// Native `nChild4` at byte 32.
    pub child4: i32,
    /// Native `nChild5` at byte 36.
    pub child5: i32,
    /// Native `nChild6` at byte 40.
    pub child6: i32,
    /// Native `nChild7` at byte 44.
    pub child7: i32,
}

impl RecordLayout for OctreeNode {
    const SIZE: usize = 48;
}
impl<'data> Read<'data> for OctreeNode {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            cube: decoder.value(offset)?,
            child0: decoder.value(offset + 16)?,
            child1: decoder.value(offset + 20)?,
            child2: decoder.value(offset + 24)?,
            child3: decoder.value(offset + 28)?,
            child4: decoder.value(offset + 32)?,
            child5: decoder.value(offset + 36)?,
            child6: decoder.value(offset + 40)?,
            child7: decoder.value(offset + 44)?,
        };
        Ok(result)
    }
}
impl Write for OctreeNode {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.cube)?;
        encoder.value(offset + 16, self.child0)?;
        encoder.value(offset + 20, self.child1)?;
        encoder.value(offset + 24, self.child2)?;
        encoder.value(offset + 28, self.child3)?;
        encoder.value(offset + 32, self.child4)?;
        encoder.value(offset + 36, self.child5)?;
        encoder.value(offset + 40, self.child6)?;
        encoder.value(offset + 44, self.child7)?;
        Ok(())
    }
}
impl AsULE for OctreeNode {
    type ULE = <[u8; 48] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 48];
        copy_value(&mut bytes[0..16], self.cube);
        copy_value(&mut bytes[16..20], self.child0);
        copy_value(&mut bytes[20..24], self.child1);
        copy_value(&mut bytes[24..28], self.child2);
        copy_value(&mut bytes[28..32], self.child3);
        copy_value(&mut bytes[32..36], self.child4);
        copy_value(&mut bytes[36..40], self.child5);
        copy_value(&mut bytes[40..44], self.child6);
        copy_value(&mut bytes[44..48], self.child7);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            cube: ZeroVec::<OctreeCube>::parse_bytes(&bytes[0..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            child0: ZeroVec::<i32>::parse_bytes(&bytes[16..20])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            child1: ZeroVec::<i32>::parse_bytes(&bytes[20..24])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            child2: ZeroVec::<i32>::parse_bytes(&bytes[24..28])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            child3: ZeroVec::<i32>::parse_bytes(&bytes[28..32])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            child4: ZeroVec::<i32>::parse_bytes(&bytes[32..36])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            child5: ZeroVec::<i32>::parse_bytes(&bytes[36..40])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            child6: ZeroVec::<i32>::parse_bytes(&bytes[40..44])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            child7: ZeroVec::<i32>::parse_bytes(&bytes[44..48])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native OctreeLeaf record (24 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct OctreeLeaf {
    /// Native `tCube` at byte 0.
    pub cube: OctreeCube,
    /// Native `dwFirstPrimitive` at byte 16.
    pub first_primitive: i32,
    /// Native `dwPrimitiveCount` at byte 20.
    pub primitive_count: i32,
}

impl RecordLayout for OctreeLeaf {
    const SIZE: usize = 24;
}
impl<'data> Read<'data> for OctreeLeaf {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            cube: decoder.value(offset)?,
            first_primitive: decoder.value(offset + 16)?,
            primitive_count: decoder.value(offset + 20)?,
        };
        Ok(result)
    }
}
impl Write for OctreeLeaf {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.cube)?;
        encoder.value(offset + 16, self.first_primitive)?;
        encoder.value(offset + 20, self.primitive_count)?;
        Ok(())
    }
}
impl AsULE for OctreeLeaf {
    type ULE = <[u8; 24] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 24];
        copy_value(&mut bytes[0..16], self.cube);
        copy_value(&mut bytes[16..20], self.first_primitive);
        copy_value(&mut bytes[20..24], self.primitive_count);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            cube: ZeroVec::<OctreeCube>::parse_bytes(&bytes[0..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            first_primitive: ZeroVec::<i32>::parse_bytes(&bytes[16..20])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            primitive_count: ZeroVec::<i32>::parse_bytes(&bytes[20..24])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native OctreePrimitive record (8 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct OctreePrimitive {
    /// Native `nUnknown00` at byte 0.
    pub unknown00: i32,
    /// Native `nUnknown04` at byte 4.
    pub unknown04: i32,
}

impl RecordLayout for OctreePrimitive {
    const SIZE: usize = 8;
}
impl<'data> Read<'data> for OctreePrimitive {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            unknown00: decoder.value(offset)?,
            unknown04: decoder.value(offset + 4)?,
        };
        Ok(result)
    }
}
impl Write for OctreePrimitive {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.unknown00)?;
        encoder.value(offset + 4, self.unknown04)?;
        Ok(())
    }
}
impl AsULE for OctreePrimitive {
    type ULE = <[u8; 8] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 8];
        copy_value(&mut bytes[0..4], self.unknown00);
        copy_value(&mut bytes[4..8], self.unknown04);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            unknown00: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown04: ZeroVec::<i32>::parse_bytes(&bytes[4..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native Octree record (72 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Default)]
pub struct Octree<'data> {
    /// Native `dwUnknown00` at byte 0.
    pub unknown00: i32,
    /// Native `dwUnknown04` at byte 4.
    pub unknown04: i32,
    /// Native `dwUnknown08` at byte 8.
    pub unknown08: i32,
    /// Native `dwNodeCount` at byte 12.
    pub node_count: i32,
    /// Native `dwLeafCount` at byte 16.
    pub leaf_count: i32,
    /// Native `dwPrimitiveCount` at byte 20.
    pub primitive_count: i32,
    /// Native `arNodes` at byte 24.
    pub nodes: ZeroVec<'data, OctreeNode>,
    /// Native `_pad0` at byte 32.
    pub _pad0: [u8; 8],
    /// Native `arLeaves` at byte 40.
    pub leaves: ZeroVec<'data, OctreeLeaf>,
    /// Native `_pad1` at byte 48.
    pub _pad1: [u8; 8],
    /// Native `arPrimitives` at byte 56.
    pub primitives: ZeroVec<'data, OctreePrimitive>,
    /// Native `_pad2` at byte 64.
    pub _pad2: [u8; 8],
}

impl RecordLayout for Octree<'_> {
    const SIZE: usize = 72;
}
impl<'data> Read<'data> for Octree<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            unknown00: decoder.value(offset)?,
            unknown04: decoder.value(offset + 4)?,
            unknown08: decoder.value(offset + 8)?,
            node_count: decoder.value(offset + 12)?,
            leaf_count: decoder.value(offset + 16)?,
            primitive_count: decoder.value(offset + 20)?,
            nodes: decoder.array(offset + 24, 0)?,
            _pad0: decoder.value(offset + 32)?,
            leaves: decoder.array(offset + 40, 0)?,
            _pad1: decoder.value(offset + 48)?,
            primitives: decoder.array(offset + 56, 0)?,
            _pad2: decoder.value(offset + 64)?,
        };
        Ok(result)
    }
}
impl Write for Octree<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.unknown00)?;
        encoder.value(offset + 4, self.unknown04)?;
        encoder.value(offset + 8, self.unknown08)?;
        encoder.value(offset + 12, self.node_count)?;
        encoder.value(offset + 16, self.leaf_count)?;
        encoder.value(offset + 20, self.primitive_count)?;
        encoder.array(offset + 24, &self.nodes, 0)?;
        encoder.value(offset + 32, self._pad0)?;
        encoder.array(offset + 40, &self.leaves, 0)?;
        encoder.value(offset + 48, self._pad1)?;
        encoder.array(offset + 56, &self.primitives, 0)?;
        encoder.value(offset + 64, self._pad2)?;
        Ok(())
    }
}

/// Native StaticLight record (92 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct StaticLight {
    /// Native `nType` at byte 0.
    pub kind: i32,
    /// Native `dwFlags` at byte 4.
    pub flags: i32,
    /// Native `vPosition` at byte 8.
    pub position: [f32; 3],
    /// Native `vDirection` at byte 20.
    pub direction: [f32; 3],
    /// Native `flIntensity` at byte 32.
    pub intensity: f32,
    /// Native `dwColor` at byte 36.
    pub color: u32,
    /// Native `flAttenuation0` at byte 40.
    pub attenuation0: f32,
    /// Native `flAttenuation1` at byte 44.
    pub attenuation1: f32,
    /// Native `flAttenuation2` at byte 48.
    pub attenuation2: f32,
    /// Native `flUnknown34` at byte 52.
    pub unknown34: f32,
    /// Native `flUnknown38` at byte 56.
    pub unknown38: f32,
    /// Native `flUnknown3C` at byte 60.
    pub unknown3_c: f32,
    /// Native `flUnknown40` at byte 64.
    pub unknown40: f32,
    /// Native `dwUnknown44` at byte 68.
    pub unknown44: i32,
    /// Native `dwUnknown48` at byte 72.
    pub unknown48: i32,
    /// Native `_pad0` at byte 76.
    pub _pad0: [u8; 16],
}

impl RecordLayout for StaticLight {
    const SIZE: usize = 92;
}
impl<'data> Read<'data> for StaticLight {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            kind: decoder.value(offset)?,
            flags: decoder.value(offset + 4)?,
            position: decoder.value(offset + 8)?,
            direction: decoder.value(offset + 20)?,
            intensity: decoder.value(offset + 32)?,
            color: decoder.value(offset + 36)?,
            attenuation0: decoder.value(offset + 40)?,
            attenuation1: decoder.value(offset + 44)?,
            attenuation2: decoder.value(offset + 48)?,
            unknown34: decoder.value(offset + 52)?,
            unknown38: decoder.value(offset + 56)?,
            unknown3_c: decoder.value(offset + 60)?,
            unknown40: decoder.value(offset + 64)?,
            unknown44: decoder.value(offset + 68)?,
            unknown48: decoder.value(offset + 72)?,
            _pad0: decoder.value(offset + 76)?,
        };
        Ok(result)
    }
}
impl Write for StaticLight {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.kind)?;
        encoder.value(offset + 4, self.flags)?;
        encoder.value(offset + 8, self.position)?;
        encoder.value(offset + 20, self.direction)?;
        encoder.value(offset + 32, self.intensity)?;
        encoder.value(offset + 36, self.color)?;
        encoder.value(offset + 40, self.attenuation0)?;
        encoder.value(offset + 44, self.attenuation1)?;
        encoder.value(offset + 48, self.attenuation2)?;
        encoder.value(offset + 52, self.unknown34)?;
        encoder.value(offset + 56, self.unknown38)?;
        encoder.value(offset + 60, self.unknown3_c)?;
        encoder.value(offset + 64, self.unknown40)?;
        encoder.value(offset + 68, self.unknown44)?;
        encoder.value(offset + 72, self.unknown48)?;
        encoder.value(offset + 76, self._pad0)?;
        Ok(())
    }
}
impl AsULE for StaticLight {
    type ULE = <[u8; 92] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 92];
        copy_value(&mut bytes[0..4], self.kind);
        copy_value(&mut bytes[4..8], self.flags);
        copy_value(&mut bytes[8..20], self.position);
        copy_value(&mut bytes[20..32], self.direction);
        copy_value(&mut bytes[32..36], self.intensity);
        copy_value(&mut bytes[36..40], self.color);
        copy_value(&mut bytes[40..44], self.attenuation0);
        copy_value(&mut bytes[44..48], self.attenuation1);
        copy_value(&mut bytes[48..52], self.attenuation2);
        copy_value(&mut bytes[52..56], self.unknown34);
        copy_value(&mut bytes[56..60], self.unknown38);
        copy_value(&mut bytes[60..64], self.unknown3_c);
        copy_value(&mut bytes[64..68], self.unknown40);
        copy_value(&mut bytes[68..72], self.unknown44);
        copy_value(&mut bytes[72..76], self.unknown48);
        copy_value(&mut bytes[76..92], self._pad0);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            kind: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            flags: ZeroVec::<i32>::parse_bytes(&bytes[4..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            position: ZeroVec::<[f32; 3]>::parse_bytes(&bytes[8..20])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            direction: ZeroVec::<[f32; 3]>::parse_bytes(&bytes[20..32])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            intensity: ZeroVec::<f32>::parse_bytes(&bytes[32..36])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            color: ZeroVec::<u32>::parse_bytes(&bytes[36..40])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            attenuation0: ZeroVec::<f32>::parse_bytes(&bytes[40..44])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            attenuation1: ZeroVec::<f32>::parse_bytes(&bytes[44..48])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            attenuation2: ZeroVec::<f32>::parse_bytes(&bytes[48..52])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown34: ZeroVec::<f32>::parse_bytes(&bytes[52..56])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown38: ZeroVec::<f32>::parse_bytes(&bytes[56..60])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown3_c: ZeroVec::<f32>::parse_bytes(&bytes[60..64])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown40: ZeroVec::<f32>::parse_bytes(&bytes[64..68])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown44: ZeroVec::<i32>::parse_bytes(&bytes[68..72])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown48: ZeroVec::<i32>::parse_bytes(&bytes[72..76])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            _pad0: ZeroVec::<[u8; 16]>::parse_bytes(&bytes[76..92])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native Appearances record (536 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Default)]
pub struct AppearanceRecord<'data> {
    /// Native `dwSnoId` at byte 0.
    pub sno_id: i32,
    /// Native `_pad0` at byte 4.
    pub _pad0: [u8; 12],
    /// Native `eObjectType` at byte 16.
    pub object_type: i32,
    /// Native `dwBoneCount` at byte 20.
    pub bone_count: i32,
    /// Native `arBones` at byte 24.
    pub bones: Vec<BoneStructure<'data>>,
    /// Native `_pad1` at byte 32.
    pub _pad1: [u8; 8],
    /// Native `tLookAt` at byte 40.
    pub look_at: LookAtData,
    /// Native `dwBonePulseCount` at byte 132.
    pub bone_pulse_count: i32,
    /// Native `_pad2` at byte 136.
    pub _pad2: [u8; 8],
    /// Native `arBonePulses` at byte 144.
    pub bone_pulses: ZeroVec<'data, u8>,
    /// Native `tGeoSet0` at byte 152.
    pub geo_set0: GeoSet<'data>,
    /// Native `tGeoSet1` at byte 176.
    pub geo_set1: GeoSet<'data>,
    /// Native `tCollisionSphere` at byte 200.
    pub collision_sphere: Sphere,
    /// Native `dwCollisionCapsuleCount` at byte 216.
    pub collision_capsule_count: i32,
    /// Native `arCollisionCapsules` at byte 220.
    pub collision_capsules: ZeroVec<'data, CollisionCapsule>,
    /// Native `_pad3` at byte 228.
    pub _pad3: [u8; 12],
    /// Native `dwHardpointCount` at byte 240.
    pub hardpoint_count: i32,
    /// Native `arHardpoints` at byte 244.
    pub hardpoints: ZeroVec<'data, Hardpoint>,
    /// Native `_pad4` at byte 252.
    pub _pad4: [u8; 12],
    /// Native `vUnknown108` at byte 264.
    pub unknown108: [f32; 3],
    /// Native `_pad5` at byte 276.
    pub _pad5: [u8; 4],
    /// Native `tOctree` at byte 280.
    pub octree: Octree<'data>,
    /// Native `tBounds` at byte 352.
    pub bounds: Bounds,
    /// Native `dwConstraintCount` at byte 376.
    pub constraint_count: i32,
    /// Native `arConstraints` at byte 380.
    pub constraints: ZeroVec<'data, ConstraintParameters>,
    /// Native `_pad6` at byte 388.
    pub _pad6: [u8; 12],
    /// Native `dwUnknown190` at byte 400.
    pub unknown190: i32,
    /// Native `_pad7` at byte 404.
    pub _pad7: [u8; 4],
    /// Native `nUnknown198` at byte 408.
    pub unknown198: i64,
    /// Native `dwLookCount` at byte 416.
    pub look_count: i32,
    /// Native `dwMaterialCount` at byte 420.
    pub material_count: i32,
    /// Native `arMaterials` at byte 424.
    pub materials: Vec<AppearanceMaterial<'data>>,
    /// Native `_pad8` at byte 432.
    pub _pad8: [u8; 8],
    /// Native `arLooks` at byte 440.
    pub looks: ZeroVec<'data, AppearanceLook>,
    /// Native `_pad9` at byte 448.
    pub _pad9: [u8; 8],
    /// Native `dwStaticLightCount` at byte 456.
    pub static_light_count: i32,
    /// Native `arStaticLights` at byte 460.
    pub static_lights: ZeroVec<'data, StaticLight>,
    /// Native `_pad10` at byte 468.
    pub _pad10: [u8; 12],
    /// Native `dwUnknown1E0` at byte 480.
    pub unknown1_e0: i32,
    /// Native `tmUnknown1E4` at byte 484.
    pub unknown1_e4: i32,
    /// Native `tmUnknown1E8` at byte 488.
    pub unknown1_e8: i32,
    /// Native `tmUnknown1EC` at byte 492.
    pub unknown1_ec: i32,
    /// Native `tmUnknown1F0` at byte 496.
    pub unknown1_f0: i32,
    /// Native `tmUnknown1F4` at byte 500.
    pub unknown1_f4: i32,
    /// Native `dwUnknown1F8` at byte 504.
    pub unknown1_f8: i32,
    /// Native `flUnknown1FC` at byte 508.
    pub unknown1_fc: f32,
    /// Native `dwUnknown200` at byte 512.
    pub unknown200: i32,
    /// Native `dwUnknown204` at byte 516.
    pub unknown204: i32,
    /// Native `nUnknown208` at byte 520.
    pub unknown208: i64,
    /// Native `dwUnknown210` at byte 528.
    pub unknown210: i32,
    /// Native `_pad11` at byte 532.
    pub _pad11: [u8; 4],
}

impl RecordLayout for AppearanceRecord<'_> {
    const SIZE: usize = 536;
}
impl<'data> Read<'data> for AppearanceRecord<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            sno_id: decoder.value(offset)?,
            _pad0: decoder.value(offset + 4)?,
            object_type: decoder.value(offset + 16)?,
            bone_count: decoder.value(offset + 20)?,
            bones: decoder.records(offset + 24, 0)?,
            _pad1: decoder.value(offset + 32)?,
            look_at: decoder.value(offset + 40)?,
            bone_pulse_count: decoder.value(offset + 132)?,
            _pad2: decoder.value(offset + 136)?,
            bone_pulses: decoder.array(offset + 144, 0)?,
            geo_set0: GeoSet::read(decoder, offset + 152)?,
            geo_set1: GeoSet::read(decoder, offset + 176)?,
            collision_sphere: decoder.value(offset + 200)?,
            collision_capsule_count: decoder.value(offset + 216)?,
            collision_capsules: decoder.array(offset + 220, 0)?,
            _pad3: decoder.value(offset + 228)?,
            hardpoint_count: decoder.value(offset + 240)?,
            hardpoints: decoder.array(offset + 244, 0)?,
            _pad4: decoder.value(offset + 252)?,
            unknown108: decoder.value(offset + 264)?,
            _pad5: decoder.value(offset + 276)?,
            octree: Octree::read(decoder, offset + 280)?,
            bounds: decoder.value(offset + 352)?,
            constraint_count: decoder.value(offset + 376)?,
            constraints: decoder.array(offset + 380, 0)?,
            _pad6: decoder.value(offset + 388)?,
            unknown190: decoder.value(offset + 400)?,
            _pad7: decoder.value(offset + 404)?,
            unknown198: decoder.value(offset + 408)?,
            look_count: decoder.value(offset + 416)?,
            material_count: decoder.value(offset + 420)?,
            materials: decoder.records(offset + 424, 0)?,
            _pad8: decoder.value(offset + 432)?,
            looks: decoder.array(offset + 440, 0)?,
            _pad9: decoder.value(offset + 448)?,
            static_light_count: decoder.value(offset + 456)?,
            static_lights: decoder.array(offset + 460, 0)?,
            _pad10: decoder.value(offset + 468)?,
            unknown1_e0: decoder.value(offset + 480)?,
            unknown1_e4: decoder.value(offset + 484)?,
            unknown1_e8: decoder.value(offset + 488)?,
            unknown1_ec: decoder.value(offset + 492)?,
            unknown1_f0: decoder.value(offset + 496)?,
            unknown1_f4: decoder.value(offset + 500)?,
            unknown1_f8: decoder.value(offset + 504)?,
            unknown1_fc: decoder.value(offset + 508)?,
            unknown200: decoder.value(offset + 512)?,
            unknown204: decoder.value(offset + 516)?,
            unknown208: decoder.value(offset + 520)?,
            unknown210: decoder.value(offset + 528)?,
            _pad11: decoder.value(offset + 532)?,
        };
        decoder.count(offset + 20, result.bones.len())?;
        decoder.count(offset + 216, result.collision_capsules.len())?;
        decoder.count(offset + 240, result.hardpoints.len())?;
        decoder.count(offset + 376, result.constraints.len())?;
        decoder.count(offset + 416, result.looks.len())?;
        decoder.count(offset + 420, result.materials.len())?;
        decoder.count(offset + 456, result.static_lights.len())?;
        Ok(result)
    }
}
impl Write for AppearanceRecord<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.sno_id)?;
        encoder.value(offset + 4, self._pad0)?;
        encoder.value(offset + 16, self.object_type)?;
        encoder.count(offset + 20, self.bones.len())?;
        encoder.records(offset + 24, &self.bones, 0)?;
        encoder.value(offset + 32, self._pad1)?;
        encoder.value(offset + 40, self.look_at)?;
        encoder.value(offset + 132, self.bone_pulse_count)?;
        encoder.value(offset + 136, self._pad2)?;
        encoder.array(offset + 144, &self.bone_pulses, 0)?;
        self.geo_set0.write(encoder, offset + 152)?;
        self.geo_set1.write(encoder, offset + 176)?;
        encoder.value(offset + 200, self.collision_sphere)?;
        encoder.count(offset + 216, self.collision_capsules.len())?;
        encoder.array(offset + 220, &self.collision_capsules, 0)?;
        encoder.value(offset + 228, self._pad3)?;
        encoder.count(offset + 240, self.hardpoints.len())?;
        encoder.array(offset + 244, &self.hardpoints, 0)?;
        encoder.value(offset + 252, self._pad4)?;
        encoder.value(offset + 264, self.unknown108)?;
        encoder.value(offset + 276, self._pad5)?;
        self.octree.write(encoder, offset + 280)?;
        encoder.value(offset + 352, self.bounds)?;
        encoder.count(offset + 376, self.constraints.len())?;
        encoder.array(offset + 380, &self.constraints, 0)?;
        encoder.value(offset + 388, self._pad6)?;
        encoder.value(offset + 400, self.unknown190)?;
        encoder.value(offset + 404, self._pad7)?;
        encoder.value(offset + 408, self.unknown198)?;
        encoder.count(offset + 416, self.looks.len())?;
        encoder.count(offset + 420, self.materials.len())?;
        encoder.records(offset + 424, &self.materials, 0)?;
        encoder.value(offset + 432, self._pad8)?;
        encoder.array(offset + 440, &self.looks, 0)?;
        encoder.value(offset + 448, self._pad9)?;
        encoder.count(offset + 456, self.static_lights.len())?;
        encoder.array(offset + 460, &self.static_lights, 0)?;
        encoder.value(offset + 468, self._pad10)?;
        encoder.value(offset + 480, self.unknown1_e0)?;
        encoder.value(offset + 484, self.unknown1_e4)?;
        encoder.value(offset + 488, self.unknown1_e8)?;
        encoder.value(offset + 492, self.unknown1_ec)?;
        encoder.value(offset + 496, self.unknown1_f0)?;
        encoder.value(offset + 500, self.unknown1_f4)?;
        encoder.value(offset + 504, self.unknown1_f8)?;
        encoder.value(offset + 508, self.unknown1_fc)?;
        encoder.value(offset + 512, self.unknown200)?;
        encoder.value(offset + 516, self.unknown204)?;
        encoder.value(offset + 520, self.unknown208)?;
        encoder.value(offset + 528, self.unknown210)?;
        encoder.value(offset + 532, self._pad11)?;
        Ok(())
    }
}
