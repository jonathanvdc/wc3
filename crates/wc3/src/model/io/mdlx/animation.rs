//! Slot allocation and direct borrowed Warcraft III key arrays.
use super::curves;
use crate::model::conversion::Error;
use crate::model::formats::mdlx::{
    animation::{Track as NativeTrack, TrackValue},
    Animatable,
};
use crate::model::profiles::Mdlx;
use crate::model::{
    ClockId, CurveValue, DynamicTrack, ModelBuilder, Property, PropertyValue, Quat, Texture,
    TimeSource, Track,
};
use glam::{Vec2, Vec3, Vec4};

pub(super) trait Value: CurveValue + PropertyValue {
    fn kind() -> &'static str;
    fn channel<'a>(track: Track<'a, Self>) -> DynamicTrack<'a>
    where
        Self: Sized;
    fn track<'a, 'b>(channel: &'b DynamicTrack<'a>) -> Option<&'b Track<'a, Self>>
    where
        Self: Sized;
}
macro_rules! value {
    ($($ty:ty => $variant:ident),* $(,)?) => {$ (
        impl Value for $ty {
            fn kind() -> &'static str { stringify!($variant) }
            fn channel<'a>(track: Track<'a, Self>) -> DynamicTrack<'a> { DynamicTrack::$variant(track) }
            fn track<'a, 'b>(channel: &'b DynamicTrack<'a>) -> Option<&'b Track<'a, Self>> { if let DynamicTrack::$variant(track) = channel { Some(track) } else { None } }
        }
    )*};
}
value!(f32 => Scalar, Vec2 => Vector2, Vec3 => Vector3, Vec4 => Vector4,
    u32 => Unsigned, i32 => Signed, Texture => Texture, Option<Texture> => OptionalTexture, Quat => Rotation);

pub(super) struct Decoder<'a, 'b> {
    pub builder: &'b mut ModelBuilder<'a, Mdlx>,
    pub globals: &'b [ClockId],
}
impl<'a> Decoder<'a, '_> {
    pub fn bind<T, U>(
        &mut self,
        source: &Animatable<'_, T>,
        name: &str,
        map: impl Fn(T) -> U + Clone + Send + Sync + 'a,
    ) -> Result<Property<U>, Error>
    where
        T: TrackValue + Send + Sync + 'a,
        T::ULE: Sync,
        U: Value + 'a,
    {
        let default = map(source.value().copied().unwrap_or_default());
        self.track(source.track(), default, name, map)
    }
    pub fn track<T, U>(
        &mut self,
        source: Option<&NativeTrack<'_, T>>,
        default: U,
        name: &str,
        map: impl Fn(T) -> U + Clone + Send + Sync + 'a,
    ) -> Result<Property<U>, Error>
    where
        T: TrackValue + Send + Sync + 'a,
        T::ULE: Sync,
        U: Value + 'a,
    {
        let Some(source) = source else {
            return Ok(self.builder.add_value(default));
        };
        self.track_property(source, default, name, map.clone(), map)
    }
    pub fn track_property<T, U>(
        &mut self,
        source: &NativeTrack<'_, T>,
        default: U,
        name: &str,
        map: impl Fn(T) -> U + Clone + Send + Sync + 'a,
        tangents: impl Fn(T) -> U + Clone + Send + Sync + 'a,
    ) -> Result<Property<U>, Error>
    where
        T: TrackValue + Send + Sync + 'a,
        T::ULE: Sync,
        U: Value + 'a,
    {
        let slot = self.builder.add_value(default);
        let clock = source
            .global_sequence_id()
            .map(|index| {
                self.globals
                    .get(index as usize)
                    .copied()
                    .ok_or_else(|| Error::new(name, "shared animation clock does not exist"))
            })
            .transpose()?
            .map_or(TimeSource::Animation, TimeSource::Shared);
        let curve = curves::read_maps(source, 0., map, tangents)?;
        self.builder.add_global_track(U::channel(Track {
            target: slot,
            curve,
            clock,
        }))?;
        Ok(slot)
    }
}
