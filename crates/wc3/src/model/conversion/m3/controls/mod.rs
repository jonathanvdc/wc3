//! Named, editable StarCraft II controls without a generic WEM solver record.
use super::effects::metadata;
use super::{append, append_string, native, nr, read_error, revision, string, Context, Error};
use crate::model::wem::{self, records as wr, semantic as sem};
use native::{Envelope, Ref, Target};
use std::collections::BTreeMap;
mod bridge;
mod extras;
mod families;
mod mesh_sections;
mod view_volumes;
pub use bridge::Controls;
pub(super) use bridge::{attach, export, restore_divisions};
struct Fields {
    values: BTreeMap<String, i64>,
    texts: BTreeMap<String, String>,
}
impl Fields {
    fn new() -> Self {
        Self {
            values: BTreeMap::new(),
            texts: BTreeMap::new(),
        }
    }
    fn uint(&mut self, key: &str, value: u32) {
        self.values.insert(key.into(), i64::from(value));
    }
    fn float(&mut self, key: &str, value: f32) {
        self.uint(key, value.to_bits());
    }
    fn floats(&mut self, key: &str, values: impl IntoIterator<Item = f32>) {
        self.texts.insert(
            key.into(),
            values
                .into_iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    fn ints(&mut self, key: &str, values: impl IntoIterator<Item = u32>) {
        self.texts.insert(
            key.into(),
            values
                .into_iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
}
struct View {
    values: BTreeMap<String, i64>,
    texts: BTreeMap<String, String>,
}
impl View {
    fn uint(&self, key: &str) -> u32 {
        self.values.get(key).copied().unwrap_or(0) as u32
    }
    fn float(&self, key: &str) -> f32 {
        f32::from_bits(self.uint(key))
    }
    fn floats(&self, key: &str) -> Result<Vec<f32>, Error> {
        self.texts
            .get(key)
            .map(|s| {
                s.split_whitespace()
                    .map(|v| {
                        v.parse::<f32>()
                            .map_err(|_| Error::new(key, "invalid floating-point component"))
                    })
                    .collect()
            })
            .unwrap_or_else(|| Ok(Vec::new()))
    }
    fn ints(&self, key: &str) -> Result<Vec<u32>, Error> {
        self.texts
            .get(key)
            .map(|s| {
                s.split_whitespace()
                    .map(|v| {
                        v.parse::<u32>()
                            .map_err(|_| Error::new(key, "invalid integer component"))
                    })
                    .collect()
            })
            .unwrap_or_else(|| Ok(Vec::new()))
    }
    fn array<const N: usize>(&self, key: &str) -> Result<[f32; N], Error> {
        let values = self.floats(key)?;
        values
            .try_into()
            .map_err(|_| Error::new(key, "invalid vector/matrix component count"))
    }
}
fn bone(value: u32, map: &[u32], key: &str) -> Result<u16, Error> {
    u16::try_from(
        *map.get(value as usize)
            .ok_or_else(|| Error::new(key, "missing control bone"))?,
    )
    .map_err(|_| Error::new(key, "bone exceeds u16 width"))
}
trait Optional<T: Target> {
    fn reference(&self) -> Ref<T>;
}
impl<T: Target> Optional<T> for () {
    fn reference(&self) -> Ref<T> {
        Ref::default()
    }
}
impl<T: Target> Optional<T> for Ref<T> {
    fn reference(&self) -> Self {
        *self
    }
}
trait Value<T> {
    fn value(self) -> Option<T>;
}
impl<T> Value<T> for () {
    fn value(self) -> Option<T> {
        None
    }
}
impl Value<f32> for f32 {
    fn value(self) -> Option<Self> {
        Some(self)
    }
}
impl Value<[f32; 4]> for [f32; 4] {
    fn value(self) -> Option<Self> {
        Some(self)
    }
}
impl<T: Target> Value<Ref<T>> for Ref<T> {
    fn value(self) -> Option<Self> {
        Some(self)
    }
}
trait Assign<T> {
    fn assign(value: T) -> Self;
}
impl<T> Assign<T> for () {
    fn assign(_: T) -> Self {}
}
impl Assign<f32> for f32 {
    fn assign(value: Self) -> Self {
        value
    }
}
impl Assign<[f32; 4]> for [f32; 4] {
    fn assign(value: Self) -> Self {
        value
    }
}
impl<T: Target> Assign<Ref<T>> for Ref<T> {
    fn assign(value: Self) -> Self {
        value
    }
}
fn assign<T, R: Assign<T>>(value: T) -> R {
    R::assign(value)
}
