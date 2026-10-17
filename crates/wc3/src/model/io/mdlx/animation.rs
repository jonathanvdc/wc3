//! Typed property bindings and direct borrowed Warcraft III key arrays.
use super::curves;
use crate::model::conversion::Error;
use crate::model::formats::mdlx::{
    animation::{Track as NativeTrack, TrackValue},
    Animatable,
};
use crate::model::profiles::Mdlx;
use crate::model::{
    Animated, Channel, Clock, ClockId, CurveValue, ModelBuilder, Property, PropertyDescriptor,
    Quaternion, Texture, Track,
};

pub(super) trait Value: CurveValue {
    fn kind() -> &'static str;
    fn channel<'a>(track: Track<'a, Self, Mdlx>) -> Channel<'a, Mdlx>
    where
        Self: Sized;
    fn track<'a, 'b>(channel: &'b Channel<'a, Mdlx>) -> Option<&'b Track<'a, Self, Mdlx>>
    where
        Self: Sized;
}
macro_rules! value {
    ($($ty:ty => $variant:ident),* $(,)?) => {$ (
        impl Value for $ty {
            fn kind() -> &'static str { stringify!($variant) }
            fn channel<'a>(track: Track<'a, Self, Mdlx>) -> Channel<'a, Mdlx> { Channel::$variant(track) }
            fn track<'a, 'b>(channel: &'b Channel<'a, Mdlx>) -> Option<&'b Track<'a, Self, Mdlx>> { if let Channel::$variant(track) = channel { Some(track) } else { None } }
        }
    )*};
}
value!(f32 => Scalar, [f32; 2] => Vector2, [f32; 3] => Vector3, [f32; 4] => Vector4,
    u32 => Unsigned, i32 => Signed, Texture => Texture, Option<Texture> => OptionalTexture, Quaternion => Rotation);

pub(super) struct Decoder<'a, 'b> {
    pub builder: &'b mut ModelBuilder<'a, Mdlx>,
    pub globals: &'b [ClockId],
}
impl<'a> Decoder<'a, '_> {
    pub fn bind<T, U, D>(
        &mut self,
        source: &Animatable<'_, T>,
        owner: D::Owner,
        descriptor: D,
        map: impl Fn(T) -> U + Clone + Send + Sync + 'a,
    ) -> Result<Animated<U>, Error>
    where
        T: TrackValue + Send + Sync + 'a,
        T::ULE: Sync,
        U: Value + 'a,
        D: PropertyDescriptor<Profile = Mdlx, Value = U> + Clone,
    {
        let default = map(source.value().copied().unwrap_or_default());
        self.track(source.track(), default, owner, descriptor, map)
    }
    pub fn track<T, U, D>(
        &mut self,
        source: Option<&NativeTrack<'_, T>>,
        default: U,
        owner: D::Owner,
        descriptor: D,
        map: impl Fn(T) -> U + Clone + Send + Sync + 'a,
    ) -> Result<Animated<U>, Error>
    where
        T: TrackValue + Send + Sync + 'a,
        T::ULE: Sync,
        U: Value + 'a,
        D: PropertyDescriptor<Profile = Mdlx, Value = U> + Clone,
    {
        let Some(source) = source else {
            return Ok(Animated::constant(default));
        };
        let name = format!("{:?}", descriptor.clone().key());
        let property = Property::profile(owner, descriptor);
        self.track_property(source, default, property, &name, map.clone(), map)
    }
    pub fn track_property<T, U>(
        &mut self,
        source: &NativeTrack<'_, T>,
        default: U,
        property: Property<U, Mdlx>,
        name: &str,
        map: impl Fn(T) -> U + Clone + Send + Sync + 'a,
        tangents: impl Fn(T) -> U + Clone + Send + Sync + 'a,
    ) -> Result<Animated<U>, Error>
    where
        T: TrackValue + Send + Sync + 'a,
        T::ULE: Sync,
        U: Value + 'a,
    {
        let clock = source
            .global_sequence_id()
            .map(|index| {
                self.globals
                    .get(index as usize)
                    .copied()
                    .ok_or_else(|| Error::new(name, "shared animation clock does not exist"))
            })
            .transpose()?
            .map_or(Clock::Timeline, Clock::Shared);
        if matches!(clock, Clock::Shared(_)) {
            let curve = curves::read_maps(source, 0., map, tangents)?;
            self.builder.add_shared_channel(U::channel(Track {
                target: property.clone(),
                curve,
                clock,
            }))?;
        } else {
            let curve = curves::read_maps(source, 0., map, tangents)?;
            self.builder.add_timeline_channel(U::channel(Track {
                target: property.clone(),
                curve,
                clock,
            }))?;
        }
        Ok(Animated::new(default, property))
    }
}
