//! Binary arrays backed directly by ZeroVec.
use crate::model::mdlx::{mdx, Cursor};
use zerovec::{ule::AsULE, ZeroVec};

pub(crate) trait WireValue: for<'input> mdx::Read<'input> + AsULE {}
impl WireValue for u8 {}
impl WireValue for u16 {}
impl WireValue for u32 {}
impl WireValue for i32 {}
impl WireValue for f32 {}
impl<T: WireValue + Default, const N: usize> WireValue for [T; N] {}

pub(crate) fn read_array_layout<'data, T: AsULE>(
    cursor: Cursor<'data>,
    width: usize,
) -> Result<ZeroVec<'data, T>, mdx::ReadError> {
    let malformed = || {
        mdx::ReadError::new(
            cursor.absolute_position(),
            mdx::ReadErrorKind::InvalidValue {
                field: "array layout",
            },
        )
    };
    if width != size_of::<T::ULE>() {
        return Err(malformed());
    }
    ZeroVec::parse_bytes(cursor.remaining()).map_err(|_| malformed())
}
pub(crate) fn read_array<'data, T: AsULE + for<'input> mdx::Read<'input>>(
    mut cursor: Cursor<'data>,
    width: usize,
) -> Result<ZeroVec<'data, T>, mdx::ReadError> {
    let source = cursor;
    while !cursor.remaining().is_empty() {
        let mut element = cursor.subcursor(width)?;
        element.read::<T>()?;
        element.finish()?;
    }
    read_array_layout(source, width)
}
