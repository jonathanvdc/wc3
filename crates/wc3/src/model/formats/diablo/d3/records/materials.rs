//! Native records for the explicitly supported Diablo III revisions.
//! Names follow the native field vocabulary; offset descriptors are resolved.
use super::super::codec::{copy_value, Decoder, Encoder};
use super::super::{Read, ReadError, RecordLayout, Write, WriteError};
use crate::model::common::Ownership;
use zerovec::{ule::AsULE, ZeroVec};

use super::TagMapEntry;
/// Native MaterialColors record (72 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct MaterialColors {
    /// Native `vDiffuse` at byte 0.
    pub diffuse: [f32; 4],
    /// Native `vSpecular` at byte 16.
    pub specular: [f32; 4],
    /// Native `vEmissive` at byte 32.
    pub emissive: [f32; 4],
    /// Native `vAmbient` at byte 48.
    pub ambient: [f32; 4],
    /// Native `flShininess` at byte 64.
    pub shininess: f32,
    /// Native `dwMaterialFlags` at byte 68.
    pub material_flags: i32,
}

impl RecordLayout for MaterialColors {
    const SIZE: usize = 72;
}
impl<'data> Read<'data> for MaterialColors {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            diffuse: decoder.value(offset)?,
            specular: decoder.value(offset + 16)?,
            emissive: decoder.value(offset + 32)?,
            ambient: decoder.value(offset + 48)?,
            shininess: decoder.value(offset + 64)?,
            material_flags: decoder.value(offset + 68)?,
        };
        Ok(result)
    }
}
impl Write for MaterialColors {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.diffuse)?;
        encoder.value(offset + 16, self.specular)?;
        encoder.value(offset + 32, self.emissive)?;
        encoder.value(offset + 48, self.ambient)?;
        encoder.value(offset + 64, self.shininess)?;
        encoder.value(offset + 68, self.material_flags)?;
        Ok(())
    }
}
impl AsULE for MaterialColors {
    type ULE = <[u8; 72] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 72];
        copy_value(&mut bytes[0..16], self.diffuse);
        copy_value(&mut bytes[16..32], self.specular);
        copy_value(&mut bytes[32..48], self.emissive);
        copy_value(&mut bytes[48..64], self.ambient);
        copy_value(&mut bytes[64..68], self.shininess);
        copy_value(&mut bytes[68..72], self.material_flags);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            diffuse: ZeroVec::<[f32; 4]>::parse_bytes(&bytes[0..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            specular: ZeroVec::<[f32; 4]>::parse_bytes(&bytes[16..32])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            emissive: ZeroVec::<[f32; 4]>::parse_bytes(&bytes[32..48])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            ambient: ZeroVec::<[f32; 4]>::parse_bytes(&bytes[48..64])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            shininess: ZeroVec::<f32>::parse_bytes(&bytes[64..68])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            material_flags: ZeroVec::<i32>::parse_bytes(&bytes[68..72])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native MaterialTextureAnim record (12 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct MaterialTextureAnim {
    /// Native `flAmount` at byte 0.
    pub amount: f32,
    /// Native `flRate0` at byte 4.
    pub rate0: f32,
    /// Native `flRate1` at byte 8.
    pub rate1: f32,
}

impl RecordLayout for MaterialTextureAnim {
    const SIZE: usize = 12;
}
impl<'data> Read<'data> for MaterialTextureAnim {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            amount: decoder.value(offset)?,
            rate0: decoder.value(offset + 4)?,
            rate1: decoder.value(offset + 8)?,
        };
        Ok(result)
    }
}
impl Write for MaterialTextureAnim {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.amount)?;
        encoder.value(offset + 4, self.rate0)?;
        encoder.value(offset + 8, self.rate1)?;
        Ok(())
    }
}
impl AsULE for MaterialTextureAnim {
    type ULE = <[u8; 12] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 12];
        copy_value(&mut bytes[0..4], self.amount);
        copy_value(&mut bytes[4..8], self.rate0);
        copy_value(&mut bytes[8..12], self.rate1);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            amount: ZeroVec::<f32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            rate0: ZeroVec::<f32>::parse_bytes(&bytes[4..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            rate1: ZeroVec::<f32>::parse_bytes(&bytes[8..12])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native MaterialTextureEntry record (160 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct MaterialTextureEntry {
    /// Native `dwSlotIndex` at byte 0.
    pub texture_type: i32,
    /// Native `dwUnknown04` at byte 4.
    pub unknown04: i32,
    /// Native `snoTexture` at byte 8.
    pub texture: i32,
    /// Native `dwTextureFlags` at byte 12.
    pub uv_transform_mode: i32,
    /// Native `vUvRow0` at byte 16.
    pub uv_row0: [f32; 4],
    /// Native `vUvRow1` at byte 32.
    pub uv_row1: [f32; 4],
    /// Native `vUvRow2` at byte 48.
    pub uv_row2: [f32; 4],
    /// Native `vUvRow3` at byte 64.
    pub uv_row3: [f32; 4],
    /// Native `tAnimU` at byte 80.
    pub anim_u: MaterialTextureAnim,
    /// Native `tAnimV` at byte 92.
    pub anim_v: MaterialTextureAnim,
    /// Native `tAnimRotate` at byte 104.
    pub anim_rotate: MaterialTextureAnim,
    /// Native `tAnim3` at byte 116.
    pub anim3: MaterialTextureAnim,
    /// Native `tAnim4` at byte 128.
    pub anim4: MaterialTextureAnim,
    /// Native `tAnim5` at byte 140.
    pub anim5: MaterialTextureAnim,
    /// Native `dwTextureType` at byte 152.
    pub uv_flags: i32,
    /// Native `dwUnknown9C` at byte 156.
    pub unknown9_c: i32,
}

impl RecordLayout for MaterialTextureEntry {
    const SIZE: usize = 160;
}
impl<'data> Read<'data> for MaterialTextureEntry {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            texture_type: decoder.value(offset)?,
            unknown04: decoder.value(offset + 4)?,
            texture: decoder.value(offset + 8)?,
            uv_transform_mode: decoder.value(offset + 12)?,
            uv_row0: decoder.value(offset + 16)?,
            uv_row1: decoder.value(offset + 32)?,
            uv_row2: decoder.value(offset + 48)?,
            uv_row3: decoder.value(offset + 64)?,
            anim_u: decoder.value(offset + 80)?,
            anim_v: decoder.value(offset + 92)?,
            anim_rotate: decoder.value(offset + 104)?,
            anim3: decoder.value(offset + 116)?,
            anim4: decoder.value(offset + 128)?,
            anim5: decoder.value(offset + 140)?,
            uv_flags: decoder.value(offset + 152)?,
            unknown9_c: decoder.value(offset + 156)?,
        };
        Ok(result)
    }
}
impl Write for MaterialTextureEntry {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.texture_type)?;
        encoder.value(offset + 4, self.unknown04)?;
        encoder.value(offset + 8, self.texture)?;
        encoder.value(offset + 12, self.uv_transform_mode)?;
        encoder.value(offset + 16, self.uv_row0)?;
        encoder.value(offset + 32, self.uv_row1)?;
        encoder.value(offset + 48, self.uv_row2)?;
        encoder.value(offset + 64, self.uv_row3)?;
        encoder.value(offset + 80, self.anim_u)?;
        encoder.value(offset + 92, self.anim_v)?;
        encoder.value(offset + 104, self.anim_rotate)?;
        encoder.value(offset + 116, self.anim3)?;
        encoder.value(offset + 128, self.anim4)?;
        encoder.value(offset + 140, self.anim5)?;
        encoder.value(offset + 152, self.uv_flags)?;
        encoder.value(offset + 156, self.unknown9_c)?;
        Ok(())
    }
}
impl AsULE for MaterialTextureEntry {
    type ULE = <[u8; 160] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 160];
        copy_value(&mut bytes[0..4], self.texture_type);
        copy_value(&mut bytes[4..8], self.unknown04);
        copy_value(&mut bytes[8..12], self.texture);
        copy_value(&mut bytes[12..16], self.uv_transform_mode);
        copy_value(&mut bytes[16..32], self.uv_row0);
        copy_value(&mut bytes[32..48], self.uv_row1);
        copy_value(&mut bytes[48..64], self.uv_row2);
        copy_value(&mut bytes[64..80], self.uv_row3);
        copy_value(&mut bytes[80..92], self.anim_u);
        copy_value(&mut bytes[92..104], self.anim_v);
        copy_value(&mut bytes[104..116], self.anim_rotate);
        copy_value(&mut bytes[116..128], self.anim3);
        copy_value(&mut bytes[128..140], self.anim4);
        copy_value(&mut bytes[140..152], self.anim5);
        copy_value(&mut bytes[152..156], self.uv_flags);
        copy_value(&mut bytes[156..160], self.unknown9_c);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            texture_type: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown04: ZeroVec::<i32>::parse_bytes(&bytes[4..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            texture: ZeroVec::<i32>::parse_bytes(&bytes[8..12])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            uv_transform_mode: ZeroVec::<i32>::parse_bytes(&bytes[12..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            uv_row0: ZeroVec::<[f32; 4]>::parse_bytes(&bytes[16..32])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            uv_row1: ZeroVec::<[f32; 4]>::parse_bytes(&bytes[32..48])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            uv_row2: ZeroVec::<[f32; 4]>::parse_bytes(&bytes[48..64])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            uv_row3: ZeroVec::<[f32; 4]>::parse_bytes(&bytes[64..80])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            anim_u: ZeroVec::<MaterialTextureAnim>::parse_bytes(&bytes[80..92])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            anim_v: ZeroVec::<MaterialTextureAnim>::parse_bytes(&bytes[92..104])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            anim_rotate: ZeroVec::<MaterialTextureAnim>::parse_bytes(&bytes[104..116])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            anim3: ZeroVec::<MaterialTextureAnim>::parse_bytes(&bytes[116..128])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            anim4: ZeroVec::<MaterialTextureAnim>::parse_bytes(&bytes[128..140])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            anim5: ZeroVec::<MaterialTextureAnim>::parse_bytes(&bytes[140..152])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            uv_flags: ZeroVec::<i32>::parse_bytes(&bytes[152..156])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown9_c: ZeroVec::<i32>::parse_bytes(&bytes[156..160])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native UberMaterial record (104 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Default)]
pub struct UberMaterial<'data> {
    /// Native `snoShaderMap` at byte 0.
    pub shader_map: i32,
    /// Native `tColors` at byte 4.
    pub colors: MaterialColors,
    /// Native `arTextures` at byte 76.
    pub textures: ZeroVec<'data, MaterialTextureEntry>,
    /// Native `_pad0` at byte 84.
    pub _pad0: [u8; 20],
}

impl RecordLayout for UberMaterial<'_> {
    const SIZE: usize = 104;
}
impl<'data> Read<'data> for UberMaterial<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            shader_map: decoder.value(offset)?,
            colors: decoder.value(offset + 4)?,
            textures: decoder.array(offset + 76, 0)?,
            _pad0: decoder.value(offset + 84)?,
        };
        Ok(result)
    }
}
impl Write for UberMaterial<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.shader_map)?;
        encoder.value(offset + 4, self.colors)?;
        encoder.array(offset + 76, &self.textures, 0)?;
        encoder.value(offset + 84, self._pad0)?;
        Ok(())
    }
}

/// Native Material record (136 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Default)]
pub struct MaterialRecord<'data> {
    /// Native `dwSnoId` at byte 0.
    pub sno_id: i32,
    /// Native `_pad0` at byte 4.
    pub _pad0: [u8; 8],
    /// Native `dwUnknown0C` at byte 12.
    pub unknown0_c: i32,
    /// Native `arShaderParams` at byte 16.
    pub shader_params: ZeroVec<'data, TagMapEntry>,
    /// Native `_pad1` at byte 24.
    pub _pad1: [u8; 8],
    /// Native `tMaterial` at byte 32.
    pub material: UberMaterial<'data>,
}

impl RecordLayout for MaterialRecord<'_> {
    const SIZE: usize = 136;
}
impl<'data> Read<'data> for MaterialRecord<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            sno_id: decoder.value(offset)?,
            _pad0: decoder.value(offset + 4)?,
            unknown0_c: decoder.value(offset + 12)?,
            shader_params: decoder.array(offset + 16, 4)?,
            _pad1: decoder.value(offset + 24)?,
            material: UberMaterial::read(decoder, offset + 32)?,
        };
        Ok(result)
    }
}
impl Write for MaterialRecord<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.sno_id)?;
        encoder.value(offset + 4, self._pad0)?;
        encoder.value(offset + 12, self.unknown0_c)?;
        encoder.array(offset + 16, &self.shader_params, 4)?;
        encoder.value(offset + 24, self._pad1)?;
        self.material.write(encoder, offset + 32)?;
        Ok(())
    }
}

/// Native RenderParams record (96 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct RenderParams {
    /// Native `dwCullMode` at byte 0.
    pub cull_mode: i32,
    /// Native `dwZWriteEnable` at byte 4.
    pub z_write_enable: i32,
    /// Native `dwZFunc` at byte 8.
    pub z_func: i32,
    /// Native `flDepthBias` at byte 12.
    pub depth_bias: f32,
    /// Native `flUnknown10` at byte 16.
    pub unknown10: f32,
    /// Native `dwStencilEnable` at byte 20.
    pub stencil_enable: i32,
    /// Native `dwStencilFunc` at byte 24.
    pub stencil_func: i32,
    /// Native `dwStencilRef` at byte 28.
    pub stencil_ref: i32,
    /// Native `dwStencilPass` at byte 32.
    pub stencil_pass: i32,
    /// Native `dwStencilFail` at byte 36.
    pub stencil_fail: i32,
    /// Native `dwStencilZFail` at byte 40.
    pub stencil_z_fail: i32,
    /// Native `dwAlphaTestEnable` at byte 44.
    pub alpha_test_enable: i32,
    /// Native `dwAlphaFunc` at byte 48.
    pub alpha_func: i32,
    /// Native `bAlphaRef` at byte 52.
    pub b_alpha_ref: u8,
    /// Native `_pad0` at byte 53.
    pub _pad0: [u8; 3],
    /// Native `dwAlphaToCoverage` at byte 56.
    pub alpha_to_coverage: i32,
    /// Native `dwFogEnable` at byte 60.
    pub fog_enable: i32,
    /// Native `dwFillMode` at byte 64.
    pub fill_mode: i32,
    /// Native `dwColorWriteEnable` at byte 68.
    pub color_write_enable: i32,
    /// Native `dwAlphaWriteEnable` at byte 72.
    pub alpha_write_enable: i32,
    /// Native `dwAlphaBlendEnable` at byte 76.
    pub alpha_blend_enable: i32,
    /// Native `dwBlendOp` at byte 80.
    pub blend_op: i32,
    /// Native `dwSrcBlend` at byte 84.
    pub src_blend: i32,
    /// Native `dwDestBlend` at byte 88.
    pub dest_blend: i32,
    /// Native `dwConstantColor` at byte 92.
    pub constant_color: i32,
}

impl RecordLayout for RenderParams {
    const SIZE: usize = 96;
}
impl<'data> Read<'data> for RenderParams {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            cull_mode: decoder.value(offset)?,
            z_write_enable: decoder.value(offset + 4)?,
            z_func: decoder.value(offset + 8)?,
            depth_bias: decoder.value(offset + 12)?,
            unknown10: decoder.value(offset + 16)?,
            stencil_enable: decoder.value(offset + 20)?,
            stencil_func: decoder.value(offset + 24)?,
            stencil_ref: decoder.value(offset + 28)?,
            stencil_pass: decoder.value(offset + 32)?,
            stencil_fail: decoder.value(offset + 36)?,
            stencil_z_fail: decoder.value(offset + 40)?,
            alpha_test_enable: decoder.value(offset + 44)?,
            alpha_func: decoder.value(offset + 48)?,
            b_alpha_ref: decoder.value(offset + 52)?,
            _pad0: decoder.value(offset + 53)?,
            alpha_to_coverage: decoder.value(offset + 56)?,
            fog_enable: decoder.value(offset + 60)?,
            fill_mode: decoder.value(offset + 64)?,
            color_write_enable: decoder.value(offset + 68)?,
            alpha_write_enable: decoder.value(offset + 72)?,
            alpha_blend_enable: decoder.value(offset + 76)?,
            blend_op: decoder.value(offset + 80)?,
            src_blend: decoder.value(offset + 84)?,
            dest_blend: decoder.value(offset + 88)?,
            constant_color: decoder.value(offset + 92)?,
        };
        Ok(result)
    }
}
impl Write for RenderParams {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.cull_mode)?;
        encoder.value(offset + 4, self.z_write_enable)?;
        encoder.value(offset + 8, self.z_func)?;
        encoder.value(offset + 12, self.depth_bias)?;
        encoder.value(offset + 16, self.unknown10)?;
        encoder.value(offset + 20, self.stencil_enable)?;
        encoder.value(offset + 24, self.stencil_func)?;
        encoder.value(offset + 28, self.stencil_ref)?;
        encoder.value(offset + 32, self.stencil_pass)?;
        encoder.value(offset + 36, self.stencil_fail)?;
        encoder.value(offset + 40, self.stencil_z_fail)?;
        encoder.value(offset + 44, self.alpha_test_enable)?;
        encoder.value(offset + 48, self.alpha_func)?;
        encoder.value(offset + 52, self.b_alpha_ref)?;
        encoder.value(offset + 53, self._pad0)?;
        encoder.value(offset + 56, self.alpha_to_coverage)?;
        encoder.value(offset + 60, self.fog_enable)?;
        encoder.value(offset + 64, self.fill_mode)?;
        encoder.value(offset + 68, self.color_write_enable)?;
        encoder.value(offset + 72, self.alpha_write_enable)?;
        encoder.value(offset + 76, self.alpha_blend_enable)?;
        encoder.value(offset + 80, self.blend_op)?;
        encoder.value(offset + 84, self.src_blend)?;
        encoder.value(offset + 88, self.dest_blend)?;
        encoder.value(offset + 92, self.constant_color)?;
        Ok(())
    }
}
impl AsULE for RenderParams {
    type ULE = <[u8; 96] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 96];
        copy_value(&mut bytes[0..4], self.cull_mode);
        copy_value(&mut bytes[4..8], self.z_write_enable);
        copy_value(&mut bytes[8..12], self.z_func);
        copy_value(&mut bytes[12..16], self.depth_bias);
        copy_value(&mut bytes[16..20], self.unknown10);
        copy_value(&mut bytes[20..24], self.stencil_enable);
        copy_value(&mut bytes[24..28], self.stencil_func);
        copy_value(&mut bytes[28..32], self.stencil_ref);
        copy_value(&mut bytes[32..36], self.stencil_pass);
        copy_value(&mut bytes[36..40], self.stencil_fail);
        copy_value(&mut bytes[40..44], self.stencil_z_fail);
        copy_value(&mut bytes[44..48], self.alpha_test_enable);
        copy_value(&mut bytes[48..52], self.alpha_func);
        copy_value(&mut bytes[52..53], self.b_alpha_ref);
        copy_value(&mut bytes[53..56], self._pad0);
        copy_value(&mut bytes[56..60], self.alpha_to_coverage);
        copy_value(&mut bytes[60..64], self.fog_enable);
        copy_value(&mut bytes[64..68], self.fill_mode);
        copy_value(&mut bytes[68..72], self.color_write_enable);
        copy_value(&mut bytes[72..76], self.alpha_write_enable);
        copy_value(&mut bytes[76..80], self.alpha_blend_enable);
        copy_value(&mut bytes[80..84], self.blend_op);
        copy_value(&mut bytes[84..88], self.src_blend);
        copy_value(&mut bytes[88..92], self.dest_blend);
        copy_value(&mut bytes[92..96], self.constant_color);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            cull_mode: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            z_write_enable: ZeroVec::<i32>::parse_bytes(&bytes[4..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            z_func: ZeroVec::<i32>::parse_bytes(&bytes[8..12])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            depth_bias: ZeroVec::<f32>::parse_bytes(&bytes[12..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown10: ZeroVec::<f32>::parse_bytes(&bytes[16..20])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            stencil_enable: ZeroVec::<i32>::parse_bytes(&bytes[20..24])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            stencil_func: ZeroVec::<i32>::parse_bytes(&bytes[24..28])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            stencil_ref: ZeroVec::<i32>::parse_bytes(&bytes[28..32])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            stencil_pass: ZeroVec::<i32>::parse_bytes(&bytes[32..36])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            stencil_fail: ZeroVec::<i32>::parse_bytes(&bytes[36..40])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            stencil_z_fail: ZeroVec::<i32>::parse_bytes(&bytes[40..44])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            alpha_test_enable: ZeroVec::<i32>::parse_bytes(&bytes[44..48])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            alpha_func: ZeroVec::<i32>::parse_bytes(&bytes[48..52])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            b_alpha_ref: ZeroVec::<u8>::parse_bytes(&bytes[52..53])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            _pad0: ZeroVec::<[u8; 3]>::parse_bytes(&bytes[53..56])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            alpha_to_coverage: ZeroVec::<i32>::parse_bytes(&bytes[56..60])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            fog_enable: ZeroVec::<i32>::parse_bytes(&bytes[60..64])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            fill_mode: ZeroVec::<i32>::parse_bytes(&bytes[64..68])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            color_write_enable: ZeroVec::<i32>::parse_bytes(&bytes[68..72])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            alpha_write_enable: ZeroVec::<i32>::parse_bytes(&bytes[72..76])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            alpha_blend_enable: ZeroVec::<i32>::parse_bytes(&bytes[76..80])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            blend_op: ZeroVec::<i32>::parse_bytes(&bytes[80..84])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            src_blend: ZeroVec::<i32>::parse_bytes(&bytes[84..88])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            dest_blend: ZeroVec::<i32>::parse_bytes(&bytes[88..92])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            constant_color: ZeroVec::<i32>::parse_bytes(&bytes[92..96])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native TextureStageParams record (24 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct TextureStageParams {
    /// Native `dwTextureType` at byte 0.
    pub texture_type: i32,
    /// Native `dwAddressU` at byte 4.
    pub address_u: i32,
    /// Native `dwAddressV` at byte 8.
    pub address_v: i32,
    /// Native `dwAddressW` at byte 12.
    pub address_w: i32,
    /// Native `dwFilter` at byte 16.
    pub filter: i32,
    /// Native `flMipMapLodBias` at byte 20.
    pub mip_map_lod_bias: f32,
}

impl RecordLayout for TextureStageParams {
    const SIZE: usize = 24;
}
impl<'data> Read<'data> for TextureStageParams {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            texture_type: decoder.value(offset)?,
            address_u: decoder.value(offset + 4)?,
            address_v: decoder.value(offset + 8)?,
            address_w: decoder.value(offset + 12)?,
            filter: decoder.value(offset + 16)?,
            mip_map_lod_bias: decoder.value(offset + 20)?,
        };
        Ok(result)
    }
}
impl Write for TextureStageParams {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.texture_type)?;
        encoder.value(offset + 4, self.address_u)?;
        encoder.value(offset + 8, self.address_v)?;
        encoder.value(offset + 12, self.address_w)?;
        encoder.value(offset + 16, self.filter)?;
        encoder.value(offset + 20, self.mip_map_lod_bias)?;
        Ok(())
    }
}
impl AsULE for TextureStageParams {
    type ULE = <[u8; 24] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 24];
        copy_value(&mut bytes[0..4], self.texture_type);
        copy_value(&mut bytes[4..8], self.address_u);
        copy_value(&mut bytes[8..12], self.address_v);
        copy_value(&mut bytes[12..16], self.address_w);
        copy_value(&mut bytes[16..20], self.filter);
        copy_value(&mut bytes[20..24], self.mip_map_lod_bias);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            texture_type: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            address_u: ZeroVec::<i32>::parse_bytes(&bytes[4..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            address_v: ZeroVec::<i32>::parse_bytes(&bytes[8..12])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            address_w: ZeroVec::<i32>::parse_bytes(&bytes[12..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            filter: ZeroVec::<i32>::parse_bytes(&bytes[16..20])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            mip_map_lod_bias: ZeroVec::<f32>::parse_bytes(&bytes[20..24])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native ShaderTagMapEntry record (12 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct ShaderTagMapEntry {
    /// Native `nValueType` at byte 0.
    pub value_type: i32,
    /// Native `dwTagId` at byte 4.
    pub tag_id: u32,
    /// Native `dwValue` at byte 8.
    pub value: u32,
}

impl RecordLayout for ShaderTagMapEntry {
    const SIZE: usize = 12;
}
impl<'data> Read<'data> for ShaderTagMapEntry {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            value_type: decoder.value(offset)?,
            tag_id: decoder.value(offset + 4)?,
            value: decoder.value(offset + 8)?,
        };
        Ok(result)
    }
}
impl Write for ShaderTagMapEntry {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.value_type)?;
        encoder.value(offset + 4, self.tag_id)?;
        encoder.value(offset + 8, self.value)?;
        Ok(())
    }
}
impl AsULE for ShaderTagMapEntry {
    type ULE = <[u8; 12] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 12];
        copy_value(&mut bytes[0..4], self.value_type);
        copy_value(&mut bytes[4..8], self.tag_id);
        copy_value(&mut bytes[8..12], self.value);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            value_type: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            tag_id: ZeroVec::<u32>::parse_bytes(&bytes[4..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            value: ZeroVec::<u32>::parse_bytes(&bytes[8..12])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native RenderPass record (568 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq)]
pub struct RenderPass<'data> {
    /// Native `dwUnknown00` at byte 0.
    pub unknown00: i32,
    /// Native `dwUnknown04` at byte 4.
    pub unknown04: i32,
    /// Native `tRenderParams` at byte 8.
    pub render_params: RenderParams,
    /// Native `dwUnknown68` at byte 104.
    pub unknown68: i32,
    /// Native `dwTextureStageCount` at byte 108.
    pub texture_stage_count: i32,
    /// Native `arTextureStages` at byte 112.
    pub texture_stages: ZeroVec<'data, TextureStageParams>,
    /// Native `_pad0` at byte 120.
    pub _pad0: [u8; 8],
    /// Native `dwPassFlags` at byte 128.
    pub pass_flags: i32,
    /// Native `szEffectFile` at byte 132.
    pub effect_file: [u8; 256],
    /// Native `szVertexShaderEntry` at byte 388.
    pub vertex_shader_entry: [u8; 64],
    /// Native `szPixelShaderEntry` at byte 452.
    pub pixel_shader_entry: [u8; 64],
    /// Native `arShaderParams` at byte 516.
    pub shader_params: ZeroVec<'data, ShaderTagMapEntry>,
    /// Native `_pad1` at byte 524.
    pub _pad1: [u8; 44],
}
impl Default for RenderPass<'_> {
    fn default() -> Self {
        Self {
            unknown00: Default::default(),
            unknown04: Default::default(),
            render_params: Default::default(),
            unknown68: Default::default(),
            texture_stage_count: Default::default(),
            texture_stages: Default::default(),
            _pad0: [Default::default(); 8],
            pass_flags: Default::default(),
            effect_file: [Default::default(); 256],
            vertex_shader_entry: [Default::default(); 64],
            pixel_shader_entry: [Default::default(); 64],
            shader_params: Default::default(),
            _pad1: [Default::default(); 44],
        }
    }
}

impl RecordLayout for RenderPass<'_> {
    const SIZE: usize = 568;
}
impl<'data> Read<'data> for RenderPass<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            unknown00: decoder.value(offset)?,
            unknown04: decoder.value(offset + 4)?,
            render_params: decoder.value(offset + 8)?,
            unknown68: decoder.value(offset + 104)?,
            texture_stage_count: decoder.value(offset + 108)?,
            texture_stages: decoder.array(offset + 112, 0)?,
            _pad0: decoder.value(offset + 120)?,
            pass_flags: decoder.value(offset + 128)?,
            effect_file: decoder.value(offset + 132)?,
            vertex_shader_entry: decoder.value(offset + 388)?,
            pixel_shader_entry: decoder.value(offset + 452)?,
            shader_params: decoder.array(offset + 516, 4)?,
            _pad1: decoder.value(offset + 524)?,
        };
        Ok(result)
    }
}
impl Write for RenderPass<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.unknown00)?;
        encoder.value(offset + 4, self.unknown04)?;
        encoder.value(offset + 8, self.render_params)?;
        encoder.value(offset + 104, self.unknown68)?;
        encoder.value(offset + 108, self.texture_stage_count)?;
        encoder.array(offset + 112, &self.texture_stages, 0)?;
        encoder.value(offset + 120, self._pad0)?;
        encoder.value(offset + 128, self.pass_flags)?;
        encoder.value(offset + 132, self.effect_file)?;
        encoder.value(offset + 388, self.vertex_shader_entry)?;
        encoder.value(offset + 452, self.pixel_shader_entry)?;
        encoder.array(offset + 516, &self.shader_params, 4)?;
        encoder.value(offset + 524, self._pad1)?;
        Ok(())
    }
}

/// Native ShaderMapEntry record (12 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct ShaderMapEntry {
    /// Native `nValueType` at byte 0.
    pub value_type: i32,
    /// Native `dwTagId` at byte 4.
    pub tag_id: u32,
    /// Native `snoShader` at byte 8.
    pub shader: i32,
}

impl RecordLayout for ShaderMapEntry {
    const SIZE: usize = 12;
}
impl<'data> Read<'data> for ShaderMapEntry {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            value_type: decoder.value(offset)?,
            tag_id: decoder.value(offset + 4)?,
            shader: decoder.value(offset + 8)?,
        };
        Ok(result)
    }
}
impl Write for ShaderMapEntry {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.value_type)?;
        encoder.value(offset + 4, self.tag_id)?;
        encoder.value(offset + 8, self.shader)?;
        Ok(())
    }
}
impl AsULE for ShaderMapEntry {
    type ULE = <[u8; 12] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 12];
        copy_value(&mut bytes[0..4], self.value_type);
        copy_value(&mut bytes[4..8], self.tag_id);
        copy_value(&mut bytes[8..12], self.shader);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            value_type: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            tag_id: ZeroVec::<u32>::parse_bytes(&bytes[4..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            shader: ZeroVec::<i32>::parse_bytes(&bytes[8..12])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native ShaderMap record (32 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Default)]
pub struct ShaderMap<'data> {
    /// Native `dwSnoId` at byte 0.
    pub sno_id: i32,
    /// Native `_pad0` at byte 4.
    pub _pad0: [u8; 8],
    /// Native `dwShaderMapFlags` at byte 12.
    pub shader_map_flags: i32,
    /// Native `arShaders` at byte 16.
    pub shaders: ZeroVec<'data, ShaderMapEntry>,
    /// Native `_pad1` at byte 24.
    pub _pad1: [u8; 8],
}

impl RecordLayout for ShaderMap<'_> {
    const SIZE: usize = 32;
}
impl<'data> Read<'data> for ShaderMap<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            sno_id: decoder.value(offset)?,
            _pad0: decoder.value(offset + 4)?,
            shader_map_flags: decoder.value(offset + 12)?,
            shaders: decoder.array(offset + 16, 4)?,
            _pad1: decoder.value(offset + 24)?,
        };
        Ok(result)
    }
}
impl Write for ShaderMap<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.sno_id)?;
        encoder.value(offset + 4, self._pad0)?;
        encoder.value(offset + 12, self.shader_map_flags)?;
        encoder.array(offset + 16, &self.shaders, 4)?;
        encoder.value(offset + 24, self._pad1)?;
        Ok(())
    }
}

/// Native Shaders record (296 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq)]
pub struct Shaders<'data> {
    /// Native `dwSnoId` at byte 0.
    pub sno_id: i32,
    /// Native `_pad0` at byte 4.
    pub _pad0: [u8; 8],
    /// Native `dwShaderFlags` at byte 12.
    pub shader_flags: i32,
    /// Native `dwUnknown10` at byte 16.
    pub unknown10: i32,
    /// Native `dwRenderPassCount` at byte 20.
    pub render_pass_count: i32,
    /// Native `arRenderPasses` at byte 24.
    pub render_passes: Vec<RenderPass<'data>>,
    /// Native `_pad1` at byte 32.
    pub _pad1: [u8; 8],
    /// Native `szName` at byte 40.
    pub name: [u8; 256],
}
impl Default for Shaders<'_> {
    fn default() -> Self {
        Self {
            sno_id: Default::default(),
            _pad0: [Default::default(); 8],
            shader_flags: Default::default(),
            unknown10: Default::default(),
            render_pass_count: Default::default(),
            render_passes: Default::default(),
            _pad1: [Default::default(); 8],
            name: [Default::default(); 256],
        }
    }
}

impl RecordLayout for Shaders<'_> {
    const SIZE: usize = 296;
}
impl<'data> Read<'data> for Shaders<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            sno_id: decoder.value(offset)?,
            _pad0: decoder.value(offset + 4)?,
            shader_flags: decoder.value(offset + 12)?,
            unknown10: decoder.value(offset + 16)?,
            render_pass_count: decoder.value(offset + 20)?,
            render_passes: decoder.records(offset + 24, 0)?,
            _pad1: decoder.value(offset + 32)?,
            name: decoder.value(offset + 40)?,
        };
        decoder.count(offset + 20, result.render_passes.len())?;
        Ok(result)
    }
}
impl Write for Shaders<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.sno_id)?;
        encoder.value(offset + 4, self._pad0)?;
        encoder.value(offset + 12, self.shader_flags)?;
        encoder.value(offset + 16, self.unknown10)?;
        encoder.count(offset + 20, self.render_passes.len())?;
        encoder.records(offset + 24, &self.render_passes, 0)?;
        encoder.value(offset + 32, self._pad1)?;
        encoder.value(offset + 40, self.name)?;
        Ok(())
    }
}
