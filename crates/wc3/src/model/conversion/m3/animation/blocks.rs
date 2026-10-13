//! Typed native block tables and their exact key encodings.
use super::*;
use std::array::from_fn;
#[derive(Default)]
pub(super) struct NativeBlocks {
    pub vectors2: Vec<nr::AnimationBlockVector2>,
    pub vectors3: Vec<nr::AnimationBlockVector3>,
    pub quaternions: Vec<nr::AnimationBlockQuaternion>,
    pub colors: Vec<nr::AnimationBlockColor>,
    pub reals: Vec<nr::AnimationBlockReal>,
    pub bytes: Vec<nr::AnimationBlockByte>,
    pub shorts: Vec<nr::AnimationBlockI16>,
    pub ushorts: Vec<nr::AnimationBlockU16>,
    pub ints: Vec<nr::AnimationBlockI32>,
    pub uints: Vec<nr::AnimationBlockU32>,
    pub flags: Vec<nr::AnimationBlockFlag>,
    pub events: Vec<nr::AnimationBlockEvent>,
    pub extents: Vec<nr::AnimationBlockExtent>,
}
impl NativeBlocks {
    pub(super) fn install<E: Envelope>(
        mut self,
        doc: &mut native::Document<'static, E>,
        controller: &mut nr::SubTrackContainer<nr::V4>,
    ) -> Result<(), Error> {
        controller.sd2v = append(doc, &self.vectors2)?.cast();
        let mut vectors = doc
            .resolve::<nr::AnimationBlockVector3>(controller.sd3v.cast())
            .map_err(read_error)?
            .iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(read_error)?;
        vectors.append(&mut self.vectors3);
        controller.sd3v = append(doc, &vectors)?.cast();
        let mut quaternions = doc
            .resolve::<nr::AnimationBlockQuaternion>(controller.sd4q.cast())
            .map_err(read_error)?
            .iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(read_error)?;
        quaternions.append(&mut self.quaternions);
        controller.sd4q = append(doc, &quaternions)?.cast();
        controller.sdcc = append(doc, &self.colors)?.cast();
        controller.sdr3 = append(doc, &self.reals)?.cast();
        controller.sdu8 = append(doc, &self.bytes)?.cast();
        controller.sds6 = append(doc, &self.shorts)?.cast();
        controller.sdu6 = append(doc, &self.ushorts)?.cast();
        controller.sds3 = append(doc, &self.ints)?.cast();
        controller.sdu3 = append(doc, &self.uints)?.cast();
        controller.sdfg = append(doc, &self.flags)?.cast();
        controller.sdev = append(doc, &self.events)?.cast();
        controller.sdmb = append(doc, &self.extents)?.cast();
        Ok(())
    }
}

pub(super) struct BlockInput {
    pub family: u32,
    pub kind: wr::AttrType,
    pub times: Vec<nr::I32>,
    pub data: Vec<u8>,
    pub flags: u32,
    pub end_frame: u32,
    pub vector_base: usize,
    pub quaternion_base: usize,
}
impl NativeBlocks {
    pub(super) fn add<E: Envelope>(
        &mut self,
        doc: &mut native::Document<'static, E>,
        input: BlockInput,
    ) -> Result<u32, Error> {
        let BlockInput {
            family,
            kind,
            times,
            data,
            flags,
            end_frame,
            vector_base,
            quaternion_base,
        } = input;
        let timestamps = append(doc, &times)?.cast();
        let selector = match family {
            1 => {
                if kind != wr::AttrType::F32X2 {
                    return Err(Error::new(
                        "animation.type",
                        "edited channel differs from source family",
                    ));
                }
                if data.len() != times.len() * 8 {
                    return Err(Error::new(
                        "animation.keys",
                        "value byte count differs from times",
                    ));
                }
                let mut keys = Vec::new();
                for bytes in data.as_chunks::<8>().0.iter() {
                    keys.push(nr::Vector2 {
                        value: from_fn(|index| {
                            f32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
                        }),
                    });
                }
                let keys = append(doc, &keys)?.cast();
                let index = self.vectors2.len();
                self.vectors2.push(nr::AnimationBlockVector2 {
                    timestamps,
                    flags,
                    end_frame,
                    keys,
                });
                (1u32 << 16)
                    | u32::try_from(index)
                        .map_err(|_| Error::new("animation.blocks", "too many blocks"))?
            }
            2 => {
                if kind != wr::AttrType::F32X3 {
                    return Err(Error::new(
                        "animation.type",
                        "edited channel differs from source family",
                    ));
                }
                if data.len() != times.len() * 12 {
                    return Err(Error::new(
                        "animation.keys",
                        "value byte count differs from times",
                    ));
                }
                let mut keys = Vec::new();
                for bytes in data.as_chunks::<12>().0.iter() {
                    keys.push(nr::Vector3 {
                        value: from_fn(|index| {
                            f32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
                        }),
                    });
                }
                let keys = append(doc, &keys)?.cast();
                let index = vector_base + self.vectors3.len();
                self.vectors3.push(nr::AnimationBlockVector3 {
                    timestamps,
                    flags,
                    end_frame,
                    keys,
                });
                (2u32 << 16)
                    | u32::try_from(index)
                        .map_err(|_| Error::new("animation.blocks", "too many blocks"))?
            }
            3 => {
                if kind != wr::AttrType::QUAT && kind != wr::AttrType::F32X4 {
                    return Err(Error::new(
                        "animation.type",
                        "edited channel differs from source family",
                    ));
                }
                if data.len() != times.len() * 16 {
                    return Err(Error::new(
                        "animation.keys",
                        "value byte count differs from times",
                    ));
                }
                let mut keys = Vec::new();
                for bytes in data.as_chunks::<16>().0.iter() {
                    keys.push(nr::Quaternion {
                        value: from_fn(|index| {
                            f32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
                        }),
                    });
                }
                let keys = append(doc, &keys)?.cast();
                let index = quaternion_base + self.quaternions.len();
                self.quaternions.push(nr::AnimationBlockQuaternion {
                    timestamps,
                    flags,
                    end_frame,
                    keys,
                });
                (3u32 << 16)
                    | u32::try_from(index)
                        .map_err(|_| Error::new("animation.blocks", "too many blocks"))?
            }
            4 => {
                if kind != wr::AttrType::U8X4 {
                    return Err(Error::new(
                        "animation.type",
                        "edited channel differs from source family",
                    ));
                }
                if data.len() != times.len() * 4 {
                    return Err(Error::new(
                        "animation.keys",
                        "value byte count differs from times",
                    ));
                }
                let mut keys = Vec::new();
                for bytes in data.as_chunks::<4>().0.iter() {
                    keys.push(nr::Color { value: *bytes });
                }
                let keys = append(doc, &keys)?.cast();
                let index = self.colors.len();
                self.colors.push(nr::AnimationBlockColor {
                    timestamps,
                    flags,
                    end_frame,
                    keys,
                });
                (4u32 << 16)
                    | u32::try_from(index)
                        .map_err(|_| Error::new("animation.blocks", "too many blocks"))?
            }
            5 => {
                if kind != wr::AttrType::F32 {
                    return Err(Error::new(
                        "animation.type",
                        "edited channel differs from source family",
                    ));
                }
                if data.len() != times.len() * 4 {
                    return Err(Error::new(
                        "animation.keys",
                        "value byte count differs from times",
                    ));
                }
                let mut keys = Vec::new();
                for bytes in data.as_chunks::<4>().0.iter() {
                    keys.push(nr::Real {
                        value: f32::from_le_bytes(*bytes),
                    });
                }
                let keys = append(doc, &keys)?.cast();
                let index = self.reals.len();
                self.reals.push(nr::AnimationBlockReal {
                    timestamps,
                    flags,
                    end_frame,
                    keys,
                });
                (5u32 << 16)
                    | u32::try_from(index)
                        .map_err(|_| Error::new("animation.blocks", "too many blocks"))?
            }
            6 => {
                if kind != wr::AttrType::U32 {
                    return Err(Error::new(
                        "animation.type",
                        "edited channel differs from source family",
                    ));
                }
                if data.len() != times.len() * 4 {
                    return Err(Error::new(
                        "animation.keys",
                        "value byte count differs from times",
                    ));
                }
                let mut keys = Vec::new();
                for bytes in data.as_chunks::<4>().0.iter() {
                    keys.push(nr::Byte {
                        value: u8::try_from(u32::from_le_bytes(*bytes)).map_err(|_| {
                            Error::new(
                                "animation.value",
                                "edited integer exceeds source key family",
                            )
                        })?,
                    });
                }
                let keys = append(doc, &keys)?.cast();
                let index = self.bytes.len();
                self.bytes.push(nr::AnimationBlockByte {
                    timestamps,
                    flags,
                    end_frame,
                    keys,
                });
                (6u32 << 16)
                    | u32::try_from(index)
                        .map_err(|_| Error::new("animation.blocks", "too many blocks"))?
            }
            7 => {
                if kind != wr::AttrType::I32 {
                    return Err(Error::new(
                        "animation.type",
                        "edited channel differs from source family",
                    ));
                }
                if data.len() != times.len() * 4 {
                    return Err(Error::new(
                        "animation.keys",
                        "value byte count differs from times",
                    ));
                }
                let mut keys = Vec::new();
                for bytes in data.as_chunks::<4>().0.iter() {
                    keys.push(nr::I16 {
                        value: i16::try_from(i32::from_le_bytes(*bytes)).map_err(|_| {
                            Error::new(
                                "animation.value",
                                "edited integer exceeds source key family",
                            )
                        })?,
                    });
                }
                let keys = append(doc, &keys)?.cast();
                let index = self.shorts.len();
                self.shorts.push(nr::AnimationBlockI16 {
                    timestamps,
                    flags,
                    end_frame,
                    keys,
                });
                (7u32 << 16)
                    | u32::try_from(index)
                        .map_err(|_| Error::new("animation.blocks", "too many blocks"))?
            }
            8 => {
                if kind != wr::AttrType::U16 {
                    return Err(Error::new(
                        "animation.type",
                        "edited channel differs from source family",
                    ));
                }
                if data.len() != times.len() * 2 {
                    return Err(Error::new(
                        "animation.keys",
                        "value byte count differs from times",
                    ));
                }
                let mut keys = Vec::new();
                for bytes in data.as_chunks::<2>().0.iter() {
                    keys.push(nr::U16 {
                        value: u16::from_le_bytes(*bytes),
                    });
                }
                let keys = append(doc, &keys)?.cast();
                let index = self.ushorts.len();
                self.ushorts.push(nr::AnimationBlockU16 {
                    timestamps,
                    flags,
                    end_frame,
                    keys,
                });
                (8u32 << 16)
                    | u32::try_from(index)
                        .map_err(|_| Error::new("animation.blocks", "too many blocks"))?
            }
            9 => {
                if kind != wr::AttrType::I32 {
                    return Err(Error::new(
                        "animation.type",
                        "edited channel differs from source family",
                    ));
                }
                if data.len() != times.len() * 4 {
                    return Err(Error::new(
                        "animation.keys",
                        "value byte count differs from times",
                    ));
                }
                let mut keys = Vec::new();
                for bytes in data.as_chunks::<4>().0.iter() {
                    keys.push(nr::I32 {
                        value: i32::from_le_bytes(*bytes),
                    });
                }
                let keys = append(doc, &keys)?.cast();
                let index = self.ints.len();
                self.ints.push(nr::AnimationBlockI32 {
                    timestamps,
                    flags,
                    end_frame,
                    keys,
                });
                (9u32 << 16)
                    | u32::try_from(index)
                        .map_err(|_| Error::new("animation.blocks", "too many blocks"))?
            }
            10 => {
                if kind != wr::AttrType::U32 {
                    return Err(Error::new(
                        "animation.type",
                        "edited channel differs from source family",
                    ));
                }
                if data.len() != times.len() * 4 {
                    return Err(Error::new(
                        "animation.keys",
                        "value byte count differs from times",
                    ));
                }
                let mut keys = Vec::new();
                for bytes in data.as_chunks::<4>().0.iter() {
                    keys.push(nr::U32 {
                        value: u32::from_le_bytes(*bytes),
                    });
                }
                let keys = append(doc, &keys)?.cast();
                let index = self.uints.len();
                self.uints.push(nr::AnimationBlockU32 {
                    timestamps,
                    flags,
                    end_frame,
                    keys,
                });
                (10u32 << 16)
                    | u32::try_from(index)
                        .map_err(|_| Error::new("animation.blocks", "too many blocks"))?
            }
            11 => {
                if kind != wr::AttrType::U32 {
                    return Err(Error::new(
                        "animation.type",
                        "edited channel differs from source family",
                    ));
                }
                if data.len() != times.len() * 4 {
                    return Err(Error::new(
                        "animation.keys",
                        "value byte count differs from times",
                    ));
                }
                let mut keys = Vec::new();
                for bytes in data.as_chunks::<4>().0.iter() {
                    keys.push(nr::Flag {
                        value: u32::from_le_bytes(*bytes),
                    });
                }
                let keys = append(doc, &keys)?.cast();
                let index = self.flags.len();
                self.flags.push(nr::AnimationBlockFlag {
                    timestamps,
                    flags,
                    end_frame,
                    keys,
                });
                (11u32 << 16)
                    | u32::try_from(index)
                        .map_err(|_| Error::new("animation.blocks", "too many blocks"))?
            }
            _ => {
                return Err(Error::new(
                    "animation.family",
                    "unsupported native control key family",
                ));
            }
        };
        Ok(selector)
    }
}
