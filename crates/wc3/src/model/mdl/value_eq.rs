use crate::model::FixedText;

/// Equality for values restored by the MDL reader when a field is omitted.
///
/// Finite floats, infinities, and signed zeros compare by their bits. All NaNs
/// compare alike because MDL preserves their class, not their payload or sign.
/// Arrays compare each component using the same rules. Custom value types can
/// implement this trait to match the representation used by their codec hooks.
pub trait ValueEq {
    fn eq_mdl(&self, other: &Self) -> bool;
}

impl ValueEq for f32 {
    fn eq_mdl(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits() || (self.is_nan() && other.is_nan())
    }
}

macro_rules! exact_values {
    ($($ty:ty),* $(,)?) => { $(
        impl ValueEq for $ty {
            fn eq_mdl(&self, other: &Self) -> bool { self == other }
        }
    )* };
}
exact_values!(u8, u16, u32, i32, bool, str, String);

impl<T: ValueEq, const N: usize> ValueEq for [T; N] {
    fn eq_mdl(&self, other: &Self) -> bool {
        self.iter()
            .zip(other)
            .all(|(value, other)| value.eq_mdl(other))
    }
}

impl<const N: usize> ValueEq for FixedText<N> {
    fn eq_mdl(&self, other: &Self) -> bool {
        self == other
    }
}
