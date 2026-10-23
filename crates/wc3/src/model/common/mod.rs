//! Storage and binary mechanics shared by native model formats.
//!
//! [`binary`](crate::model::common::binary) supplies bounded address spaces, sequential cursors, and output
//! buffers. [`EncodedArray`](crate::model::common::EncodedArray) retains fixed records in borrowed encoded storage
//! until editing requires ownership; format adapters supply their record layouts
//! and codecs. Native codecs use [`zerovec::ZeroVec`] for unaligned scalar arrays.
//! [`PackedArray`] exposes semantic values through [`PackedValue`], retaining
//! compatible packed storage independently of glam's aligned memory layout.
//!
//! Native modules retain their own read/write traits, diagnostics, references,
//! layout selection, and validation. Sharing storage preserves each native
//! format's addressing and layout rules.
use std::array::from_fn;

mod packed;
pub(crate) use packed::packed_identity;
pub use packed::{IntoPackedArray, PackedArray, PackedValue};

mod ownership;
pub use ownership::{Borrowed, Detached, IntoOwned};
mod into_zero_vec;
pub use into_zero_vec::IntoZeroVec;
pub use zerovec::{ule::AsULE, ZeroVec};

pub mod binary;

pub use wc3_derive::{CommonPartialEq, IntoOwned, Ownership};

mod io_error;
pub use io_error::IoError;

mod resources;
pub use resources::FileKey;

mod encoded_array;
pub use encoded_array::{DecodeRecord, EncodeRecord, EncodedArray, RecordCodec};

mod record_errors;
pub(crate) use record_errors::record_errors;
mod record_primitives;
pub(crate) use record_primitives::record_primitives;

/// Builds a fixed array in order, stopping immediately at the first error.
pub fn try_array<T, E, const N: usize>(
    mut read: impl FnMut() -> Result<T, E>,
) -> Result<[T; N], E> {
    let mut error = None;
    let values = from_fn(|_| {
        if error.is_some() {
            return None;
        }
        match read() {
            Ok(value) => Some(value),
            Err(failure) => {
                error = Some(failure);
                None
            }
        }
    });
    match error {
        Some(error) => Err(error),
        None => Ok(values.map(|value| value.expect("successful array element"))),
    }
}

#[cfg(test)]
mod tests {
    use super::try_array;
    use std::cell::Cell;

    #[test]
    fn fixed_arrays_stop_on_first_failure_and_drop_initialized_values() {
        struct Failure;
        struct Value<'a>(&'a Cell<usize>);
        impl Drop for Value<'_> {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }
        let drops = Cell::new(0);
        let mut calls = 0;
        let result: Result<[Value<'_>; 4], Failure> = try_array(|| {
            calls += 1;
            if calls == 3 {
                Err(Failure)
            } else {
                Ok(Value(&drops))
            }
        });
        assert!(result.is_err());
        assert_eq!(calls, 3);
        assert_eq!(drops.get(), 2);
        let empty: Result<[(); 0], Failure> = try_array(|| panic!("empty array called reader"));
        assert!(empty.is_ok());
    }
}
