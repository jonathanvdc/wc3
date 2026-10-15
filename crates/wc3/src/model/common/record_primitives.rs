//! Primitive implementations for native record traits, preserving format namespaces.
macro_rules! record_primitives {
    ($generics:tt, $args:tt, $read_args:tt, $decoder:ty, $encoder:ty; $($ty:ty),* $(,)?) => {$(
        record_primitives!(@one $generics, $args, $read_args, $decoder, $encoder; $ty);
    )*};
    (@one [$($generics:tt)*], [$($args:tt)*], [$($read_args:tt)*], $decoder:ty, $encoder:ty; $ty:ty) => {
        impl<$($generics)*> Read<$($read_args)*> for $ty {
            fn read(input: &mut $decoder) -> Result<Self, ReadError> {
                Ok(Self::from_le_bytes(input.bytes(size_of::<Self>())?.try_into().unwrap()))
            }
        }
        impl<$($generics)*> Write<$($args)*> for $ty {
            fn write(&self, output: &mut $encoder) -> Result<(), WriteError> {
                output.bytes(&self.to_le_bytes())
            }
        }
        impl<$($generics)*> RecordLayout<$($args)*> for $ty {
            fn size() -> usize { size_of::<Self>() }
        }
        impl Validate for $ty {
            fn validate(&self) -> Result<(), ReadError> { Ok(()) }
        }
    };
}
pub(crate) use record_primitives;
