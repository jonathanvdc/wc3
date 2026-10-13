//! Native key families decoded directly into their typed WEM representations.
use super::*;
pub(super) struct KeyData {
    pub kind: wr::AttrType,
    pub times: Vec<wr::Real>,
    pub values: Vec<u8>,
    pub flags: u32,
    pub end: u32,
    pub family: u32,
}
pub(super) fn read_keys<E: Envelope>(
    doc: &native::Document<'_, E>,
    controller: &nr::SubTrackContainer<nr::V4>,
    selector: u32,
    start: u32,
    context: &mut Context,
) -> Result<Option<KeyData>, Error> {
    let index = (selector & 0xffff) as usize;
    let family = selector >> 16;
    macro_rules! keys {
        ($block:ty,$reference:expr,$key:ty,$kind:expr,$convert:expr) => {{
            let blocks = doc
                .resolve::<$block>($reference.cast())
                .map_err(read_error)?;
            let block = blocks
                .get(index)
                .ok_or_else(|| Error::new("m3.animation.selector", "missing key block"))?
                .map_err(read_error)?;
            let times: Vec<_> = doc
                .resolve::<nr::I32>(block.timestamps.cast())
                .map_err(read_error)?
                .iter()
                .map(|row| {
                    row.map(|row| wr::Real {
                        value: (row.value as f64 - f64::from(start)) as f32 / 1000.,
                    })
                })
                .collect::<Result<_, _>>()
                .map_err(read_error)?;
            let rows = doc.resolve::<$key>(block.keys.cast()).map_err(read_error)?;
            if rows.len() != times.len() {
                return Err(Error::new(
                    "m3.animation.keys",
                    "timestamp/value count differs",
                ));
            }
            let mut values = Vec::new();
            for row in rows.iter() {
                let row = row.map_err(read_error)?;
                values.extend(($convert)(row));
            }
            Some(KeyData {
                kind: $kind,
                times,
                values,
                flags: block.flags,
                end: block.end_frame,
                family,
            })
        }};
    }
    Ok(match family {
        1 => keys!(
            nr::AnimationBlockVector2,
            controller.sd2v,
            nr::Vector2,
            wr::AttrType::F32X2,
            |row: nr::Vector2| row
                .value
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect::<Vec<_>>()
        ),
        2 => keys!(
            nr::AnimationBlockVector3,
            controller.sd3v,
            nr::Vector3,
            wr::AttrType::F32X3,
            |row: nr::Vector3| row
                .value
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect::<Vec<_>>()
        ),
        3 => keys!(
            nr::AnimationBlockQuaternion,
            controller.sd4q,
            nr::Quaternion,
            wr::AttrType::QUAT,
            |row: nr::Quaternion| row
                .value
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect::<Vec<_>>()
        ),
        4 => keys!(
            nr::AnimationBlockColor,
            controller.sdcc,
            nr::Color,
            wr::AttrType::U8X4,
            |row: nr::Color| row.value.to_vec()
        ),
        5 => keys!(
            nr::AnimationBlockReal,
            controller.sdr3,
            nr::Real,
            wr::AttrType::F32,
            |row: nr::Real| row.value.to_le_bytes().to_vec()
        ),
        6 => keys!(
            nr::AnimationBlockByte,
            controller.sdu8,
            nr::Byte,
            wr::AttrType::U32,
            |row: nr::Byte| u32::from(row.value).to_le_bytes().to_vec()
        ),
        7 => keys!(
            nr::AnimationBlockI16,
            controller.sds6,
            nr::I16,
            wr::AttrType::I32,
            |row: nr::I16| i32::from(row.value).to_le_bytes().to_vec()
        ),
        8 => keys!(
            nr::AnimationBlockU16,
            controller.sdu6,
            nr::U16,
            wr::AttrType::U16,
            |row: nr::U16| row.value.to_le_bytes().to_vec()
        ),
        9 => keys!(
            nr::AnimationBlockI32,
            controller.sds3,
            nr::I32,
            wr::AttrType::I32,
            |row: nr::I32| row.value.to_le_bytes().to_vec()
        ),
        10 => keys!(
            nr::AnimationBlockU32,
            controller.sdu3,
            nr::U32,
            wr::AttrType::U32,
            |row: nr::U32| row.value.to_le_bytes().to_vec()
        ),
        11 => keys!(
            nr::AnimationBlockFlag,
            controller.sdfg,
            nr::Flag,
            wr::AttrType::U32,
            |row: nr::Flag| row.value.to_le_bytes().to_vec()
        ),
        _ => {
            context.unsupported(
                "m3.animation.selector",
                "event and extent families require their structured semantic records",
            )?;
            None
        }
    })
}
