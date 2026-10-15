//! Buffer selection for borrowed and independent imports.
use std::borrow::Cow;
use zerovec::{ule::AsULE, ZeroVec};

pub(super) trait Storage<'source, 'output> {
    fn text(value: Cow<'source, str>) -> Cow<'output, str>;
    fn bytes(value: &'source [u8]) -> Cow<'output, [u8]>;
    fn array<T: AsULE>(value: ZeroVec<'source, T>) -> ZeroVec<'output, T>;
}
pub(super) struct Borrowed;
pub(super) struct Owned;
impl<'a> Storage<'a, 'a> for Borrowed {
    fn text(value: Cow<'a, str>) -> Cow<'a, str> {
        value
    }
    fn bytes(value: &'a [u8]) -> Cow<'a, [u8]> {
        Cow::Borrowed(value)
    }
    fn array<T: AsULE>(value: ZeroVec<'a, T>) -> ZeroVec<'a, T> {
        value
    }
}
impl<'source, 'output> Storage<'source, 'output> for Owned {
    fn text(value: Cow<'source, str>) -> Cow<'output, str> {
        Cow::Owned(value.into_owned())
    }
    fn bytes(value: &'source [u8]) -> Cow<'output, [u8]> {
        Cow::Owned(value.to_vec())
    }
    fn array<T: AsULE>(value: ZeroVec<'source, T>) -> ZeroVec<'output, T> {
        value.into_owned()
    }
}
