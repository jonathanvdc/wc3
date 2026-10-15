//! Native records for the explicitly supported Diablo III revisions.
//! Names follow the native field vocabulary; offset descriptors are resolved.
use super::super::codec::{copy_value, Decoder, Encoder};
use super::super::{Read, ReadError, RecordLayout, Write, WriteError};
use crate::model::common::Ownership;
use zerovec::{ule::AsULE, ZeroVec};

use super::{Bounds, Sphere, TagMapEntry, TriggerEvent};
/// Native AxialCylinder record (20 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct AxialCylinder {
    /// Native `vCenter` at byte 0.
    pub center: [f32; 3],
    /// Native `flRadius` at byte 12.
    pub radius: f32,
    /// Native `flHeight` at byte 16.
    pub height: f32,
}

impl RecordLayout for AxialCylinder {
    const SIZE: usize = 20;
}
impl<'data> Read<'data> for AxialCylinder {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            center: decoder.value(offset)?,
            radius: decoder.value(offset + 12)?,
            height: decoder.value(offset + 16)?,
        };
        Ok(result)
    }
}
impl Write for AxialCylinder {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.center)?;
        encoder.value(offset + 12, self.radius)?;
        encoder.value(offset + 16, self.height)?;
        Ok(())
    }
}
impl AsULE for AxialCylinder {
    type ULE = <[u8; 20] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 20];
        copy_value(&mut bytes[0..12], self.center);
        copy_value(&mut bytes[12..16], self.radius);
        copy_value(&mut bytes[16..20], self.height);
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
            height: ZeroVec::<f32>::parse_bytes(&bytes[16..20])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native MsgTriggeredEvent record (412 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct MsgTriggeredEvent {
    /// Native `eMessageType` at byte 0.
    pub message_type: i32,
    /// Native `tEvent` at byte 4.
    pub event: TriggerEvent,
}

impl RecordLayout for MsgTriggeredEvent {
    const SIZE: usize = 412;
}
impl<'data> Read<'data> for MsgTriggeredEvent {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            message_type: decoder.value(offset)?,
            event: decoder.value(offset + 4)?,
        };
        Ok(result)
    }
}
impl Write for MsgTriggeredEvent {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.message_type)?;
        encoder.value(offset + 4, self.event)?;
        Ok(())
    }
}
impl AsULE for MsgTriggeredEvent {
    type ULE = <[u8; 412] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 412];
        copy_value(&mut bytes[0..4], self.message_type);
        copy_value(&mut bytes[4..412], self.event);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            message_type: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
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

/// Native WeightedLook record (68 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy)]
pub struct WeightedLook {
    /// Native `szName` at byte 0.
    pub name: [u8; 64],
    /// Native `nWeight` at byte 64.
    pub weight: i32,
}
impl Default for WeightedLook {
    fn default() -> Self {
        Self {
            name: [Default::default(); 64],
            weight: Default::default(),
        }
    }
}

impl RecordLayout for WeightedLook {
    const SIZE: usize = 68;
}
impl<'data> Read<'data> for WeightedLook {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            name: decoder.value(offset)?,
            weight: decoder.value(offset + 64)?,
        };
        Ok(result)
    }
}
impl Write for WeightedLook {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.name)?;
        encoder.value(offset + 64, self.weight)?;
        Ok(())
    }
}
impl AsULE for WeightedLook {
    type ULE = <[u8; 68] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 68];
        copy_value(&mut bytes[0..64], self.name);
        copy_value(&mut bytes[64..68], self.weight);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            name: ZeroVec::<[u8; 64]>::parse_bytes(&bytes[0..64])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            weight: ZeroVec::<i32>::parse_bytes(&bytes[64..68])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native ActorCollisionFlags record (16 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct ActorCollisionFlags {
    /// Native `dwFlags00` at byte 0.
    pub flags00: i32,
    /// Native `dwFlags04` at byte 4.
    pub flags04: i32,
    /// Native `dwFlags08` at byte 8.
    pub flags08: i32,
    /// Native `dwFlags0C` at byte 12.
    pub flags0_c: i32,
}

impl RecordLayout for ActorCollisionFlags {
    const SIZE: usize = 16;
}
impl<'data> Read<'data> for ActorCollisionFlags {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            flags00: decoder.value(offset)?,
            flags04: decoder.value(offset + 4)?,
            flags08: decoder.value(offset + 8)?,
            flags0_c: decoder.value(offset + 12)?,
        };
        Ok(result)
    }
}
impl Write for ActorCollisionFlags {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.flags00)?;
        encoder.value(offset + 4, self.flags04)?;
        encoder.value(offset + 8, self.flags08)?;
        encoder.value(offset + 12, self.flags0_c)?;
        Ok(())
    }
}
impl AsULE for ActorCollisionFlags {
    type ULE = <[u8; 16] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 16];
        copy_value(&mut bytes[0..4], self.flags00);
        copy_value(&mut bytes[4..8], self.flags04);
        copy_value(&mut bytes[8..12], self.flags08);
        copy_value(&mut bytes[12..16], self.flags0_c);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            flags00: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            flags04: ZeroVec::<i32>::parse_bytes(&bytes[4..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            flags08: ZeroVec::<i32>::parse_bytes(&bytes[8..12])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            flags0_c: ZeroVec::<i32>::parse_bytes(&bytes[12..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native ActorCollisionData record (68 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct ActorCollisionData {
    /// Native `tFlags` at byte 0.
    pub flags: ActorCollisionFlags,
    /// Native `nCollisionEnabled` at byte 16.
    pub collision_enabled: i32,
    /// Native `tCylinder` at byte 20.
    pub cylinder: AxialCylinder,
    /// Native `tBounds` at byte 40.
    pub bounds: Bounds,
    /// Native `flUnknown40` at byte 64.
    pub unknown40: f32,
}

impl RecordLayout for ActorCollisionData {
    const SIZE: usize = 68;
}
impl<'data> Read<'data> for ActorCollisionData {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            flags: decoder.value(offset)?,
            collision_enabled: decoder.value(offset + 16)?,
            cylinder: decoder.value(offset + 20)?,
            bounds: decoder.value(offset + 40)?,
            unknown40: decoder.value(offset + 64)?,
        };
        Ok(result)
    }
}
impl Write for ActorCollisionData {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.flags)?;
        encoder.value(offset + 16, self.collision_enabled)?;
        encoder.value(offset + 20, self.cylinder)?;
        encoder.value(offset + 40, self.bounds)?;
        encoder.value(offset + 64, self.unknown40)?;
        Ok(())
    }
}
impl AsULE for ActorCollisionData {
    type ULE = <[u8; 68] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 68];
        copy_value(&mut bytes[0..16], self.flags);
        copy_value(&mut bytes[16..20], self.collision_enabled);
        copy_value(&mut bytes[20..40], self.cylinder);
        copy_value(&mut bytes[40..64], self.bounds);
        copy_value(&mut bytes[64..68], self.unknown40);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            flags: ZeroVec::<ActorCollisionFlags>::parse_bytes(&bytes[0..16])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            collision_enabled: ZeroVec::<i32>::parse_bytes(&bytes[16..20])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            cylinder: ZeroVec::<AxialCylinder>::parse_bytes(&bytes[20..40])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            bounds: ZeroVec::<Bounds>::parse_bytes(&bytes[40..64])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            unknown40: ZeroVec::<f32>::parse_bytes(&bytes[64..68])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native InventoryImages record (8 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Copy, Default)]
pub struct InventoryImages {
    /// Native `dwIconName0` at byte 0.
    pub icon_name0: i32,
    /// Native `dwIconName1` at byte 4.
    pub icon_name1: i32,
}

impl RecordLayout for InventoryImages {
    const SIZE: usize = 8;
}
impl<'data> Read<'data> for InventoryImages {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            icon_name0: decoder.value(offset)?,
            icon_name1: decoder.value(offset + 4)?,
        };
        Ok(result)
    }
}
impl Write for InventoryImages {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.icon_name0)?;
        encoder.value(offset + 4, self.icon_name1)?;
        Ok(())
    }
}
impl AsULE for InventoryImages {
    type ULE = <[u8; 8] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0u8; 8];
        copy_value(&mut bytes[0..4], self.icon_name0);
        copy_value(&mut bytes[4..8], self.icon_name1);
        bytes.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let bytes = &value;
        Self {
            icon_name0: ZeroVec::<i32>::parse_bytes(&bytes[0..4])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
            icon_name1: ZeroVec::<i32>::parse_bytes(&bytes[4..8])
                .expect("fixed native value")
                .get(0)
                .expect("one value"),
        }
    }
}

/// Native Actor record (880 fixed bytes before referenced arrays).
#[derive(Clone, Debug, Ownership, PartialEq, Default)]
pub struct ActorRecord<'data> {
    /// Native `dwSnoId` at byte 0.
    pub sno_id: i32,
    /// Native `_pad0` at byte 4.
    pub _pad0: [u8; 8],
    /// Native `dwFlags` at byte 12.
    pub flags: i32,
    /// Native `eActorType` at byte 16.
    pub actor_type: i32,
    /// Native `snoAppearance` at byte 20.
    pub appearance: i32,
    /// Native `snoPhysMesh` at byte 24.
    pub phys_mesh: i32,
    /// Native `tBoundCylinder` at byte 28.
    pub bound_cylinder: AxialCylinder,
    /// Native `tBoundSphere` at byte 48.
    pub bound_sphere: Sphere,
    /// Native `tBoundBox` at byte 64.
    pub bound_box: Bounds,
    /// Native `arTagMap` at byte 88.
    pub tag_map: ZeroVec<'data, TagMapEntry>,
    /// Native `_pad1` at byte 96.
    pub _pad1: [u8; 8],
    /// Native `snoAnimSet` at byte 104.
    pub anim_set: i32,
    /// Native `snoMonster` at byte 108.
    pub monster: i32,
    /// Native `arMsgTriggeredEvents` at byte 112.
    pub msg_triggered_events: ZeroVec<'data, MsgTriggeredEvent>,
    /// Native `dwMsgTriggeredEventCount` at byte 120.
    pub msg_triggered_event_count: i32,
    /// Native `_pad2` at byte 124.
    pub _pad2: [u8; 12],
    /// Native `vDisplacement` at byte 136.
    pub displacement: [f32; 3],
    /// Native `tLook0` at byte 148.
    pub look0: WeightedLook,
    /// Native `tLook1` at byte 216.
    pub look1: WeightedLook,
    /// Native `tLook2` at byte 284.
    pub look2: WeightedLook,
    /// Native `tLook3` at byte 352.
    pub look3: WeightedLook,
    /// Native `tLook4` at byte 420.
    pub look4: WeightedLook,
    /// Native `tLook5` at byte 488.
    pub look5: WeightedLook,
    /// Native `tLook6` at byte 556.
    pub look6: WeightedLook,
    /// Native `tLook7` at byte 624.
    pub look7: WeightedLook,
    /// Native `snoPhysics` at byte 692.
    pub physics: i32,
    /// Native `dwFlags2B8` at byte 696.
    pub flags2_b8: i32,
    /// Native `dwUnknown2BC` at byte 700.
    pub unknown2_bc: i32,
    /// Native `flCloth2C0` at byte 704.
    pub cloth2_c0: f32,
    /// Native `flClothForceScale` at byte 708.
    pub cloth_force_scale: f32,
    /// Native `flClothDamping` at byte 712.
    pub cloth_damping: f32,
    /// Native `tCollision` at byte 716.
    pub collision: ActorCollisionData,
    /// Native `tInventoryIcon0` at byte 784.
    pub inventory_icon0: InventoryImages,
    /// Native `tInventoryIcon1` at byte 792.
    pub inventory_icon1: InventoryImages,
    /// Native `tInventoryIcon2` at byte 800.
    pub inventory_icon2: InventoryImages,
    /// Native `tInventoryIcon3` at byte 808.
    pub inventory_icon3: InventoryImages,
    /// Native `tInventoryIcon4` at byte 816.
    pub inventory_icon4: InventoryImages,
    /// Native `tInventoryIcon5` at byte 824.
    pub inventory_icon5: InventoryImages,
    /// Native `tInventoryIcon6` at byte 832.
    pub inventory_icon6: InventoryImages,
    /// Native `dwInventoryIconOverride` at byte 840.
    pub inventory_icon_override: i32,
    /// Native `_pad3` at byte 844.
    pub _pad3: [u8; 4],
    /// Native `_pad4` at byte 848.
    pub _pad4: [u8; 8],
    /// Native `szCastingNotes` at byte 856.
    pub casting_notes: ZeroVec<'data, u8>,
    /// Native `_pad5` at byte 864.
    pub _pad5: [u8; 8],
    /// Native `szVoiceOverRole` at byte 872.
    pub voice_over_role: ZeroVec<'data, u8>,
}

impl RecordLayout for ActorRecord<'_> {
    const SIZE: usize = 880;
}
impl<'data> Read<'data> for ActorRecord<'data> {
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError> {
        decoder.range(offset, Self::SIZE)?;
        let result = Self {
            sno_id: decoder.value(offset)?,
            _pad0: decoder.value(offset + 4)?,
            flags: decoder.value(offset + 12)?,
            actor_type: decoder.value(offset + 16)?,
            appearance: decoder.value(offset + 20)?,
            phys_mesh: decoder.value(offset + 24)?,
            bound_cylinder: decoder.value(offset + 28)?,
            bound_sphere: decoder.value(offset + 48)?,
            bound_box: decoder.value(offset + 64)?,
            tag_map: decoder.array(offset + 88, 4)?,
            _pad1: decoder.value(offset + 96)?,
            anim_set: decoder.value(offset + 104)?,
            monster: decoder.value(offset + 108)?,
            msg_triggered_events: decoder.array(offset + 112, 0)?,
            msg_triggered_event_count: decoder.value(offset + 120)?,
            _pad2: decoder.value(offset + 124)?,
            displacement: decoder.value(offset + 136)?,
            look0: decoder.value(offset + 148)?,
            look1: decoder.value(offset + 216)?,
            look2: decoder.value(offset + 284)?,
            look3: decoder.value(offset + 352)?,
            look4: decoder.value(offset + 420)?,
            look5: decoder.value(offset + 488)?,
            look6: decoder.value(offset + 556)?,
            look7: decoder.value(offset + 624)?,
            physics: decoder.value(offset + 692)?,
            flags2_b8: decoder.value(offset + 696)?,
            unknown2_bc: decoder.value(offset + 700)?,
            cloth2_c0: decoder.value(offset + 704)?,
            cloth_force_scale: decoder.value(offset + 708)?,
            cloth_damping: decoder.value(offset + 712)?,
            collision: decoder.value(offset + 716)?,
            inventory_icon0: decoder.value(offset + 784)?,
            inventory_icon1: decoder.value(offset + 792)?,
            inventory_icon2: decoder.value(offset + 800)?,
            inventory_icon3: decoder.value(offset + 808)?,
            inventory_icon4: decoder.value(offset + 816)?,
            inventory_icon5: decoder.value(offset + 824)?,
            inventory_icon6: decoder.value(offset + 832)?,
            inventory_icon_override: decoder.value(offset + 840)?,
            _pad3: decoder.value(offset + 844)?,
            _pad4: decoder.value(offset + 848)?,
            casting_notes: decoder.array(offset + 856, 0)?,
            _pad5: decoder.value(offset + 864)?,
            voice_over_role: decoder.array(offset + 872, 0)?,
        };
        decoder.count(offset + 120, result.msg_triggered_events.len())?;
        Ok(result)
    }
}
impl Write for ActorRecord<'_> {
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError> {
        encoder.value(offset, self.sno_id)?;
        encoder.value(offset + 4, self._pad0)?;
        encoder.value(offset + 12, self.flags)?;
        encoder.value(offset + 16, self.actor_type)?;
        encoder.value(offset + 20, self.appearance)?;
        encoder.value(offset + 24, self.phys_mesh)?;
        encoder.value(offset + 28, self.bound_cylinder)?;
        encoder.value(offset + 48, self.bound_sphere)?;
        encoder.value(offset + 64, self.bound_box)?;
        encoder.array(offset + 88, &self.tag_map, 4)?;
        encoder.value(offset + 96, self._pad1)?;
        encoder.value(offset + 104, self.anim_set)?;
        encoder.value(offset + 108, self.monster)?;
        encoder.array(offset + 112, &self.msg_triggered_events, 0)?;
        encoder.count(offset + 120, self.msg_triggered_events.len())?;
        encoder.value(offset + 124, self._pad2)?;
        encoder.value(offset + 136, self.displacement)?;
        encoder.value(offset + 148, self.look0)?;
        encoder.value(offset + 216, self.look1)?;
        encoder.value(offset + 284, self.look2)?;
        encoder.value(offset + 352, self.look3)?;
        encoder.value(offset + 420, self.look4)?;
        encoder.value(offset + 488, self.look5)?;
        encoder.value(offset + 556, self.look6)?;
        encoder.value(offset + 624, self.look7)?;
        encoder.value(offset + 692, self.physics)?;
        encoder.value(offset + 696, self.flags2_b8)?;
        encoder.value(offset + 700, self.unknown2_bc)?;
        encoder.value(offset + 704, self.cloth2_c0)?;
        encoder.value(offset + 708, self.cloth_force_scale)?;
        encoder.value(offset + 712, self.cloth_damping)?;
        encoder.value(offset + 716, self.collision)?;
        encoder.value(offset + 784, self.inventory_icon0)?;
        encoder.value(offset + 792, self.inventory_icon1)?;
        encoder.value(offset + 800, self.inventory_icon2)?;
        encoder.value(offset + 808, self.inventory_icon3)?;
        encoder.value(offset + 816, self.inventory_icon4)?;
        encoder.value(offset + 824, self.inventory_icon5)?;
        encoder.value(offset + 832, self.inventory_icon6)?;
        encoder.value(offset + 840, self.inventory_icon_override)?;
        encoder.value(offset + 844, self._pad3)?;
        encoder.value(offset + 848, self._pad4)?;
        encoder.array(offset + 856, &self.casting_notes, 0)?;
        encoder.value(offset + 864, self._pad5)?;
        encoder.array(offset + 872, &self.voice_over_role, 0)?;
        Ok(())
    }
}
