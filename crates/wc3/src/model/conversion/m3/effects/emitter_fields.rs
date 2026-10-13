//! Typed optional revision fields and native emitter property headers.
use super::metadata::{self, header, native_property};
use super::{native, nr, wr, Context, Error};
use std::collections::BTreeMap;
pub(super) trait Slot<T> {
    fn slot(self) -> Option<T>;
}
pub(super) trait Assign<T> {
    const PRESENT: bool;
    fn assign(value: T) -> Self;
}
impl<T> Slot<T> for () {
    fn slot(self) -> Option<T> {
        None
    }
}
impl<T> Assign<T> for () {
    const PRESENT: bool = false;
    fn assign(_: T) -> Self {}
}
macro_rules! scalar{($($t:ty),*)=>{$(impl Slot<$t> for $t{fn slot(self)->Option<Self>{Some(self)}}impl Assign<$t> for $t{const PRESENT:bool=true;fn assign(value:Self)->Self{value}})*};}
scalar!(f32, u32, i32);
impl<T> Slot<nr::AnimationValue<T>> for nr::AnimationValue<T> {
    fn slot(self) -> Option<Self> {
        Some(self)
    }
}
impl<T> Assign<nr::AnimationValue<T>> for nr::AnimationValue<T> {
    const PRESENT: bool = true;
    fn assign(value: Self) -> Self {
        value
    }
}
impl<T: native::Target> Slot<native::Ref<T>> for native::Ref<T> {
    fn slot(self) -> Option<Self> {
        Some(self)
    }
}
impl<T: native::Target> Assign<native::Ref<T>> for native::Ref<T> {
    const PRESENT: bool = true;
    fn assign(value: Self) -> Self {
        value
    }
}
pub(super) fn property<T: Copy>(
    values: &mut BTreeMap<String, i64>,
    key: &str,
    value: nr::AnimationValue<T>,
) -> wr::Sc2Property<T> {
    metadata::property(values, key, value)
}
pub(super) fn color(
    values: &mut BTreeMap<String, i64>,
    key: &str,
    value: nr::AnimationValue<[u8; 4]>,
) -> wr::Sc2Property<[f32; 4]> {
    header(values, key, &value);
    wr::Sc2Property {
        init_value: value.initial_value.map(|v| f32::from(v) / 255.),
        null_value: value.null_value.map(|v| f32::from(v) / 255.),
    }
}
pub(super) fn fixed_color(
    values: &mut BTreeMap<String, i64>,
    key: &str,
    value: nr::AnimationValue<nr::FixedBytes<4>>,
) -> wr::Sc2Property<[f32; 4]> {
    header(values, key, &value);
    wr::Sc2Property {
        init_value: value.initial_value.0.map(|v| f32::from(v) / 255.),
        null_value: value.null_value.0.map(|v| f32::from(v) / 255.),
    }
}
pub(super) fn squirt(
    values: &mut BTreeMap<String, i64>,
    key: &str,
    value: nr::AnimationValue<u16>,
) -> wr::Sc2Property<u32> {
    header(values, key, &value);
    wr::Sc2Property {
        init_value: u32::from(value.initial_value),
        null_value: u32::from(value.null_value),
    }
}
fn packed_color(value: [f32; 4], path: &str, context: &mut Context) -> Result<[u8; 4], Error> {
    let mut result = [0; 4];
    for (index, value) in value.into_iter().enumerate() {
        if !value.is_finite() {
            return Err(Error::new(path, "non-finite particle color"));
        }
        let encoded = (value * 255.).round().clamp(0., 255.) as u8;
        context.approximate(
            format!("{path}[{index}]"),
            (f32::from(encoded) / 255. - value).abs(),
            "particle color requires byte quantization",
        )?;
        result[index] = encoded;
    }
    Ok(result)
}
pub(super) fn native_color(
    values: &BTreeMap<String, i64>,
    key: &str,
    value: wr::Sc2Property<[f32; 4]>,
    context: &mut Context,
) -> Result<nr::AnimationValue<[u8; 4]>, Error> {
    let prop = wr::Sc2Property {
        init_value: packed_color(value.init_value, key, context)?,
        null_value: packed_color(value.null_value, key, context)?,
    };
    Ok(native_property(values, key, prop))
}
pub(super) fn native_fixed_color(
    values: &BTreeMap<String, i64>,
    key: &str,
    value: wr::Sc2Property<[f32; 4]>,
    context: &mut Context,
) -> Result<nr::AnimationValue<nr::FixedBytes<4>>, Error> {
    let prop = wr::Sc2Property {
        init_value: nr::FixedBytes(packed_color(value.init_value, key, context)?),
        null_value: nr::FixedBytes(packed_color(value.null_value, key, context)?),
    };
    Ok(native_property(values, key, prop))
}
pub(super) fn native_squirt(
    values: &BTreeMap<String, i64>,
    key: &str,
    value: wr::Sc2Property<u32>,
    _: &mut Context,
) -> Result<nr::AnimationValue<u16>, Error> {
    let prop = wr::Sc2Property {
        init_value: u16::try_from(value.init_value)
            .map_err(|_| Error::new(key, "squirt count exceeds native width"))?,
        null_value: u16::try_from(value.null_value)
            .map_err(|_| Error::new(key, "squirt reset count exceeds native width"))?,
    };
    Ok(native_property(values, key, prop))
}
