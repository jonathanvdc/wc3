//! Native records for the explicitly supported Diablo III revisions.
//! Names follow the native field vocabulary; offset descriptors are resolved.
use super::super::codec::{copy_value, Decoder, Encoder};
use super::super::{Read, ReadError, RecordLayout, Write, WriteError};
use crate::model::common::Ownership;
use zerovec::{ule::AsULE, ZeroVec};

/// Native AABB record (24 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct Bounds {
    /// Native `vCenter` at byte 0.
    pub center: [f32; 3],
    /// Native `vHalfExtent` at byte 12.
    pub half_extent: [f32; 3],
}

impl RecordLayout for Bounds {
    const SIZE: usize = 24;
}
impl<'data> Read<'data> for Bounds {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            center: decoder.value(offset)?,
            half_extent: decoder.value(offset + 12)?,
        };
        Ok(result)
    }
}
impl Write for Bounds {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.center)?;
        encoder.value(offset + 12, self.half_extent)?;
        Ok(())
    }
}
impl AsULE for Bounds {
    type ULE = <[u8; 24] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 24];
        copy_value(&mut bytes[0..12], self.center);
        copy_value(&mut bytes[12..24], self.half_extent);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            center: ZeroVec::<[f32; 3]>::parse_bytes(&bytes[0..12])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            half_extent: ZeroVec::<[f32; 3]>::parse_bytes(&bytes[12..24])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native Sphere record (16 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct Sphere {
    /// Native `vCenter` at byte 0.
    pub center: [f32; 3],
    /// Native `flRadius` at byte 12.
    pub radius: f32,
}

impl RecordLayout for Sphere {
    const SIZE: usize = 16;
}
impl<'data> Read<'data> for Sphere {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            center: decoder.value(offset)?,
            radius: decoder.value(offset + 12)?,
        };
        Ok(result)
    }
}
impl Write for Sphere {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.center)?;
        encoder.value(offset + 12, self.radius)?;
        Ok(())
    }
}
impl AsULE for Sphere {
    type ULE = <[u8; 16] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 16];
        copy_value(&mut bytes[0..12], self.center);
        copy_value(&mut bytes[12..16], self.radius);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            center: ZeroVec::<[f32; 3]>::parse_bytes(&bytes[0..12])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            radius: ZeroVec::<f32>::parse_bytes(&bytes[12..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native TagMapEntry record (12 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct TagMapEntry {
    /// Native `nValueType` at byte 0.
    pub value_type: i32,
    /// Native `dwTagId` at byte 4.
    pub tag_id: u32,
    /// Native `dwValue` at byte 8.
    pub value: u32,
}

impl RecordLayout for TagMapEntry {
    const SIZE: usize = 12;
}
impl<'data> Read<'data> for TagMapEntry {
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
impl Write for TagMapEntry {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.value_type)?;
        encoder.value(offset + 4, self.tag_id)?;
        encoder.value(offset + 8, self.value)?;
        Ok(())
    }
}
impl AsULE for TagMapEntry {
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

/// Native TriggerConditions record (36 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct TriggerConditions {
    /// Native `nChance` at byte 0.
    pub chance: i32,
    /// Native `tmDelayMin` at byte 4.
    pub delay_min: i32,
    /// Native `tmDelayRange` at byte 8.
    pub delay_range: i32,
    /// Native `tmDelayMinAlt` at byte 12.
    pub delay_min_alt: i32,
    /// Native `tmDelayRangeAlt` at byte 16.
    pub delay_range_alt: i32,
    /// Native `flKeyValueMin` at byte 20.
    pub key_value_min: f32,
    /// Native `flKeyValueRange` at byte 24.
    pub key_value_range: f32,
    /// Native `nSubKey` at byte 28.
    pub sub_key: i32,
    /// Native `dwCondFlags` at byte 32.
    pub cond_flags: i32,
}

impl RecordLayout for TriggerConditions {
    const SIZE: usize = 36;
}
impl<'data> Read<'data> for TriggerConditions {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            chance: decoder.value(offset)?,
            delay_min: decoder.value(offset + 4)?,
            delay_range: decoder.value(offset + 8)?,
            delay_min_alt: decoder.value(offset + 12)?,
            delay_range_alt: decoder.value(offset + 16)?,
            key_value_min: decoder.value(offset + 20)?,
            key_value_range: decoder.value(offset + 24)?,
            sub_key: decoder.value(offset + 28)?,
            cond_flags: decoder.value(offset + 32)?,
        };
        Ok(result)
    }
}
impl Write for TriggerConditions {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.chance)?;
        encoder.value(offset + 4, self.delay_min)?;
        encoder.value(offset + 8, self.delay_range)?;
        encoder.value(offset + 12, self.delay_min_alt)?;
        encoder.value(offset + 16, self.delay_range_alt)?;
        encoder.value(offset + 20, self.key_value_min)?;
        encoder.value(offset + 24, self.key_value_range)?;
        encoder.value(offset + 28, self.sub_key)?;
        encoder.value(offset + 32, self.cond_flags)?;
        Ok(())
    }
}
impl AsULE for TriggerConditions {
    type ULE = <[u8; 36] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 36];
        copy_value(&mut bytes[0..4], self.chance);
        copy_value(&mut bytes[4..8], self.delay_min);
        copy_value(&mut bytes[8..12], self.delay_range);
        copy_value(&mut bytes[12..16], self.delay_min_alt);
        copy_value(&mut bytes[16..20], self.delay_range_alt);
        copy_value(&mut bytes[20..24], self.key_value_min);
        copy_value(&mut bytes[24..28], self.key_value_range);
        copy_value(&mut bytes[28..32], self.sub_key);
        copy_value(&mut bytes[32..36], self.cond_flags);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            chance: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            delay_min: ZeroVec::<i32>::parse_bytes(&bytes[4..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            delay_range: ZeroVec::<i32>::parse_bytes(&bytes[8..12])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            delay_min_alt: ZeroVec::<i32>::parse_bytes(&bytes[12..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            delay_range_alt: ZeroVec::<i32>::parse_bytes(&bytes[16..20])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            key_value_min: ZeroVec::<f32>::parse_bytes(&bytes[20..24])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            key_value_range: ZeroVec::<f32>::parse_bytes(&bytes[24..28])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            sub_key: ZeroVec::<i32>::parse_bytes(&bytes[28..32])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            cond_flags: ZeroVec::<i32>::parse_bytes(&bytes[32..36])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native SNOName record (8 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct SNOName {
    /// Native `eSnoGroup` at byte 0.
    pub sno_group: i32,
    /// Native `dwNameHandle` at byte 4.
    pub name_handle: i32,
}

impl RecordLayout for SNOName {
    const SIZE: usize = 8;
}
impl<'data> Read<'data> for SNOName {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            sno_group: decoder.value(offset)?,
            name_handle: decoder.value(offset + 4)?,
        };
        Ok(result)
    }
}
impl Write for SNOName {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.sno_group)?;
        encoder.value(offset + 4, self.name_handle)?;
        Ok(())
    }
}
impl AsULE for SNOName {
    type ULE = <[u8; 8] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 8];
        copy_value(&mut bytes[0..4], self.sno_group);
        copy_value(&mut bytes[4..8], self.name_handle);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            sno_group: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            name_handle: ZeroVec::<i32>::parse_bytes(&bytes[4..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native HardpointLink record (68 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy)]
pub struct HardpointLink {
    /// Native `szName` at byte 0.
    pub name: [u8; 64],
    /// Native `nIndex` at byte 64.
    pub index: i32,
}
impl Default for HardpointLink {
    fn default() -> Self {
        Self {
            name: [Default::default(); 64],
            index: Default::default(),
        }
    }
}

impl RecordLayout for HardpointLink {
    const SIZE: usize = 68;
}
impl<'data> Read<'data> for HardpointLink {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            name: decoder.value(offset)?,
            index: decoder.value(offset + 64)?,
        };
        Ok(result)
    }
}
impl Write for HardpointLink {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.name)?;
        encoder.value(offset + 64, self.index)?;
        Ok(())
    }
}
impl AsULE for HardpointLink {
    type ULE = <[u8; 68] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 68];
        copy_value(&mut bytes[0..64], self.name);
        copy_value(&mut bytes[64..68], self.index);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            name: ZeroVec::<[u8; 64]>::parse_bytes(&bytes[0..64])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            index: ZeroVec::<i32>::parse_bytes(&bytes[64..68])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native TriggerEvent record (408 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy)]
pub struct TriggerEvent {
    /// Native `eTriggerType` at byte 0.
    pub trigger_type: i32,
    /// Native `tConditions` at byte 4.
    pub conditions: TriggerConditions,
    /// Native `ePayloadKind` at byte 40.
    pub payload_kind: i32,
    /// Native `tPayload` at byte 44.
    pub payload: SNOName,
    /// Native `dwUnknown34` at byte 52.
    pub unknown34: i32,
    /// Native `dwFlags38` at byte 56.
    pub flags38: i32,
    /// Native `dwUnknown3C` at byte 60.
    pub unknown3_c: i32,
    /// Native `dwUnknown40` at byte 64.
    pub unknown40: i32,
    /// Native `tHardpoint0` at byte 68.
    pub hardpoint0: HardpointLink,
    /// Native `tHardpoint1` at byte 136.
    pub hardpoint1: HardpointLink,
    /// Native `szLookName` at byte 204.
    pub look_name: [u8; 64],
    /// Native `szConstraintName` at byte 268.
    pub constraint_name: [u8; 64],
    /// Native `dwUnknown14C` at byte 332.
    pub unknown14_c: i32,
    /// Native `flUnknown150` at byte 336.
    pub unknown150: f32,
    /// Native `dwUnknown154` at byte 340.
    pub unknown154: i32,
    /// Native `dwUnknown158` at byte 344.
    pub unknown158: i32,
    /// Native `dwFlags15C` at byte 348.
    pub flags15_c: i32,
    /// Native `dwUnknown160` at byte 352.
    pub unknown160: i32,
    /// Native `dwUnknown164` at byte 356.
    pub unknown164: i32,
    /// Native `flUnknown168` at byte 360.
    pub unknown168: f32,
    /// Native `flUnknown16C` at byte 364.
    pub unknown16_c: f32,
    /// Native `dwUnknown170` at byte 368.
    pub unknown170: i32,
    /// Native `flUnknown174` at byte 372.
    pub unknown174: f32,
    /// Native `dwUnknown178` at byte 376.
    pub unknown178: i32,
    /// Native `flVelocity17C` at byte 380.
    pub velocity17_c: f32,
    /// Native `dwUnknown180` at byte 384.
    pub unknown180: i32,
    /// Native `tmDuration` at byte 388.
    pub duration: i32,
    /// Native `dwColor188` at byte 392.
    pub color188: u32,
    /// Native `tmColor188Time` at byte 396.
    pub color188_time: i32,
    /// Native `dwColor190` at byte 400.
    pub color190: u32,
    /// Native `tmColor190Time` at byte 404.
    pub color190_time: i32,
}
impl Default for TriggerEvent {
    fn default() -> Self {
        Self {
            trigger_type: Default::default(),
            conditions: Default::default(),
            payload_kind: Default::default(),
            payload: Default::default(),
            unknown34: Default::default(),
            flags38: Default::default(),
            unknown3_c: Default::default(),
            unknown40: Default::default(),
            hardpoint0: Default::default(),
            hardpoint1: Default::default(),
            look_name: [Default::default(); 64],
            constraint_name: [Default::default(); 64],
            unknown14_c: Default::default(),
            unknown150: Default::default(),
            unknown154: Default::default(),
            unknown158: Default::default(),
            flags15_c: Default::default(),
            unknown160: Default::default(),
            unknown164: Default::default(),
            unknown168: Default::default(),
            unknown16_c: Default::default(),
            unknown170: Default::default(),
            unknown174: Default::default(),
            unknown178: Default::default(),
            velocity17_c: Default::default(),
            unknown180: Default::default(),
            duration: Default::default(),
            color188: Default::default(),
            color188_time: Default::default(),
            color190: Default::default(),
            color190_time: Default::default(),
        }
    }
}

impl RecordLayout for TriggerEvent {
    const SIZE: usize = 408;
}
impl<'data> Read<'data> for TriggerEvent {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            trigger_type: decoder.value(offset)?,
            conditions: decoder.value(offset + 4)?,
            payload_kind: decoder.value(offset + 40)?,
            payload: decoder.value(offset + 44)?,
            unknown34: decoder.value(offset + 52)?,
            flags38: decoder.value(offset + 56)?,
            unknown3_c: decoder.value(offset + 60)?,
            unknown40: decoder.value(offset + 64)?,
            hardpoint0: decoder.value(offset + 68)?,
            hardpoint1: decoder.value(offset + 136)?,
            look_name: decoder.value(offset + 204)?,
            constraint_name: decoder.value(offset + 268)?,
            unknown14_c: decoder.value(offset + 332)?,
            unknown150: decoder.value(offset + 336)?,
            unknown154: decoder.value(offset + 340)?,
            unknown158: decoder.value(offset + 344)?,
            flags15_c: decoder.value(offset + 348)?,
            unknown160: decoder.value(offset + 352)?,
            unknown164: decoder.value(offset + 356)?,
            unknown168: decoder.value(offset + 360)?,
            unknown16_c: decoder.value(offset + 364)?,
            unknown170: decoder.value(offset + 368)?,
            unknown174: decoder.value(offset + 372)?,
            unknown178: decoder.value(offset + 376)?,
            velocity17_c: decoder.value(offset + 380)?,
            unknown180: decoder.value(offset + 384)?,
            duration: decoder.value(offset + 388)?,
            color188: decoder.value(offset + 392)?,
            color188_time: decoder.value(offset + 396)?,
            color190: decoder.value(offset + 400)?,
            color190_time: decoder.value(offset + 404)?,
        };
        Ok(result)
    }
}
impl Write for TriggerEvent {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.trigger_type)?;
        encoder.value(offset + 4, self.conditions)?;
        encoder.value(offset + 40, self.payload_kind)?;
        encoder.value(offset + 44, self.payload)?;
        encoder.value(offset + 52, self.unknown34)?;
        encoder.value(offset + 56, self.flags38)?;
        encoder.value(offset + 60, self.unknown3_c)?;
        encoder.value(offset + 64, self.unknown40)?;
        encoder.value(offset + 68, self.hardpoint0)?;
        encoder.value(offset + 136, self.hardpoint1)?;
        encoder.value(offset + 204, self.look_name)?;
        encoder.value(offset + 268, self.constraint_name)?;
        encoder.value(offset + 332, self.unknown14_c)?;
        encoder.value(offset + 336, self.unknown150)?;
        encoder.value(offset + 340, self.unknown154)?;
        encoder.value(offset + 344, self.unknown158)?;
        encoder.value(offset + 348, self.flags15_c)?;
        encoder.value(offset + 352, self.unknown160)?;
        encoder.value(offset + 356, self.unknown164)?;
        encoder.value(offset + 360, self.unknown168)?;
        encoder.value(offset + 364, self.unknown16_c)?;
        encoder.value(offset + 368, self.unknown170)?;
        encoder.value(offset + 372, self.unknown174)?;
        encoder.value(offset + 376, self.unknown178)?;
        encoder.value(offset + 380, self.velocity17_c)?;
        encoder.value(offset + 384, self.unknown180)?;
        encoder.value(offset + 388, self.duration)?;
        encoder.value(offset + 392, self.color188)?;
        encoder.value(offset + 396, self.color188_time)?;
        encoder.value(offset + 400, self.color190)?;
        encoder.value(offset + 404, self.color190_time)?;
        Ok(())
    }
}
impl AsULE for TriggerEvent {
    type ULE = <[u8; 408] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 408];
        copy_value(&mut bytes[0..4], self.trigger_type);
        copy_value(&mut bytes[4..40], self.conditions);
        copy_value(&mut bytes[40..44], self.payload_kind);
        copy_value(&mut bytes[44..52], self.payload);
        copy_value(&mut bytes[52..56], self.unknown34);
        copy_value(&mut bytes[56..60], self.flags38);
        copy_value(&mut bytes[60..64], self.unknown3_c);
        copy_value(&mut bytes[64..68], self.unknown40);
        copy_value(&mut bytes[68..136], self.hardpoint0);
        copy_value(&mut bytes[136..204], self.hardpoint1);
        copy_value(&mut bytes[204..268], self.look_name);
        copy_value(&mut bytes[268..332], self.constraint_name);
        copy_value(&mut bytes[332..336], self.unknown14_c);
        copy_value(&mut bytes[336..340], self.unknown150);
        copy_value(&mut bytes[340..344], self.unknown154);
        copy_value(&mut bytes[344..348], self.unknown158);
        copy_value(&mut bytes[348..352], self.flags15_c);
        copy_value(&mut bytes[352..356], self.unknown160);
        copy_value(&mut bytes[356..360], self.unknown164);
        copy_value(&mut bytes[360..364], self.unknown168);
        copy_value(&mut bytes[364..368], self.unknown16_c);
        copy_value(&mut bytes[368..372], self.unknown170);
        copy_value(&mut bytes[372..376], self.unknown174);
        copy_value(&mut bytes[376..380], self.unknown178);
        copy_value(&mut bytes[380..384], self.velocity17_c);
        copy_value(&mut bytes[384..388], self.unknown180);
        copy_value(&mut bytes[388..392], self.duration);
        copy_value(&mut bytes[392..396], self.color188);
        copy_value(&mut bytes[396..400], self.color188_time);
        copy_value(&mut bytes[400..404], self.color190);
        copy_value(&mut bytes[404..408], self.color190_time);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            trigger_type: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            conditions: ZeroVec::<TriggerConditions>::parse_bytes(&bytes[4..40])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            payload_kind: ZeroVec::<i32>::parse_bytes(&bytes[40..44])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            payload: ZeroVec::<SNOName>::parse_bytes(&bytes[44..52])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown34: ZeroVec::<i32>::parse_bytes(&bytes[52..56])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            flags38: ZeroVec::<i32>::parse_bytes(&bytes[56..60])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown3_c: ZeroVec::<i32>::parse_bytes(&bytes[60..64])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown40: ZeroVec::<i32>::parse_bytes(&bytes[64..68])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            hardpoint0: ZeroVec::<HardpointLink>::parse_bytes(&bytes[68..136])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            hardpoint1: ZeroVec::<HardpointLink>::parse_bytes(&bytes[136..204])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            look_name: ZeroVec::<[u8; 64]>::parse_bytes(&bytes[204..268])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            constraint_name: ZeroVec::<[u8; 64]>::parse_bytes(&bytes[268..332])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown14_c: ZeroVec::<i32>::parse_bytes(&bytes[332..336])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown150: ZeroVec::<f32>::parse_bytes(&bytes[336..340])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown154: ZeroVec::<i32>::parse_bytes(&bytes[340..344])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown158: ZeroVec::<i32>::parse_bytes(&bytes[344..348])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            flags15_c: ZeroVec::<i32>::parse_bytes(&bytes[348..352])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown160: ZeroVec::<i32>::parse_bytes(&bytes[352..356])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown164: ZeroVec::<i32>::parse_bytes(&bytes[356..360])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown168: ZeroVec::<f32>::parse_bytes(&bytes[360..364])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown16_c: ZeroVec::<f32>::parse_bytes(&bytes[364..368])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown170: ZeroVec::<i32>::parse_bytes(&bytes[368..372])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown174: ZeroVec::<f32>::parse_bytes(&bytes[372..376])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown178: ZeroVec::<i32>::parse_bytes(&bytes[376..380])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            velocity17_c: ZeroVec::<f32>::parse_bytes(&bytes[380..384])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown180: ZeroVec::<i32>::parse_bytes(&bytes[384..388])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            duration: ZeroVec::<i32>::parse_bytes(&bytes[388..392])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            color188: ZeroVec::<u32>::parse_bytes(&bytes[392..396])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            color188_time: ZeroVec::<i32>::parse_bytes(&bytes[396..400])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            color190: ZeroVec::<u32>::parse_bytes(&bytes[400..404])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            color190_time: ZeroVec::<i32>::parse_bytes(&bytes[404..408])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}
