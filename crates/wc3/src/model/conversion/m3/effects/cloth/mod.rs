//! Cloth cages, proxy bindings, capsules, and solver controls as active WEM physics.
use super::emitter_fields::{Assign, Slot};
use super::{
    metadata, native, nr, read_error, revision, sem, wem, wr, Context, Envelope, Error, Ref,
};
use crate::model::conversion::m3::append;
use std::collections::BTreeMap;
use wem::{Bool, InlineOption};
mod decode;
mod encode;
mod export;
mod import;
pub(in crate::model::conversion::m3) use export::export;
pub(in crate::model::conversion::m3) use import::attach;
struct SourceCloth {
    record: nr::ClothPhysics<nr::V4>,
    values: BTreeMap<String, i64>,
}
impl<const N: usize> Slot<nr::FixedBytes<N>> for nr::FixedBytes<N> {
    fn slot(self) -> Option<Self> {
        Some(self)
    }
}
impl<const N: usize> Assign<nr::FixedBytes<N>> for nr::FixedBytes<N> {
    const PRESENT: bool = true;
    fn assign(value: Self) -> Self {
        value
    }
}
impl Slot<[f32; 3]> for [f32; 3] {
    fn slot(self) -> Option<Self> {
        Some(self)
    }
}
impl Assign<[f32; 3]> for [f32; 3] {
    const PRESENT: bool = true;
    fn assign(value: Self) -> Self {
        value
    }
}
fn numbers<T: std::fmt::Display>(values: impl IntoIterator<Item = T>) -> String {
    values
        .into_iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
fn parse<T: std::str::FromStr>(value: &str, key: &str) -> Result<Vec<T>, Error>
where
    T::Err: std::fmt::Display,
{
    if value.is_empty() {
        return Ok(Vec::new());
    }
    value
        .split(',')
        .map(|value| {
            value
                .parse()
                .map_err(|e| Error::new(key, format!("invalid array value: {e}")))
        })
        .collect()
}
fn text_fields(
    doc: &wem::RawDocument<'_>,
    bag: &wr::NativeBag,
) -> Result<BTreeMap<String, String>, Error> {
    doc.resolve::<wr::NativeEntry<wr::V2>>(bag.entries.cast())?
        .iter()
        .map(|row| {
            let row = row?;
            Ok((
                doc.string(row.name)?.unwrap_or_default().to_owned(),
                doc.string(row.text)?.unwrap_or_default().to_owned(),
            ))
        })
        .collect()
}
#[cfg(test)]
#[path = "tests.rs"]
mod tests;

trait ClothReference {
    fn cloth_reference(&self) -> Ref<nr::targets::ClothPhysics>;
}
impl ClothReference for () {
    fn cloth_reference(&self) -> Ref<nr::targets::ClothPhysics> {
        Ref::default()
    }
}
impl ClothReference for Ref<nr::targets::ClothPhysics> {
    fn cloth_reference(&self) -> Self {
        *self
    }
}
