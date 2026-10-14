//! World of Warcraft particle snapshots map directly to their authored parameters.
use super::{common::Field, read_asset, write_asset};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::actors::wow::{
    EmitterShape, Particle, ParticleBlend, ParticleFlags,
};
use crate::model::semantic::{Animated, TextureId, Values};
use crate::model::wem::{self, records, RawDocument, Record, RecordLayout, Ref, Target};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::mem::size_of;
use zerovec::{ule::AsULE, ZeroVec};
impl Field<[f32; 2]> for [f32; 2] {
    fn read(&self) -> Option<[f32; 2]> {
        Some(*self)
    }
    fn write(value: Option<[f32; 2]>) -> Result<Self, Error> {
        value.ok_or_else(|| {
            Error::new(
                "wem.actor.capability",
                "selected vocabulary requires an undeclared vector control",
            )
        })
    }
}
impl<T: Target> Field<Ref<T>> for Ref<T> {
    fn read(&self) -> Option<Ref<T>> {
        Some(*self)
    }
    fn write(value: Option<Ref<T>>) -> Result<Self, Error> {
        value.ok_or_else(|| {
            Error::new(
                "wem.actor.capability",
                "selected vocabulary requires an undeclared array control",
            )
        })
    }
}
impl Field<records::AssetKey> for records::AssetKey {
    fn read(&self) -> Option<records::AssetKey> {
        Some(self.clone())
    }
    fn write(value: Option<records::AssetKey>) -> Result<Self, Error> {
        value.ok_or_else(|| {
            Error::new(
                "wem.actor.capability",
                "selected vocabulary requires an undeclared asset control",
            )
        })
    }
}
trait Primitive:
    Record + RecordLayout + for<'record> wem::Read<'record> + wem::Write + Send + Sync + 'static
{
    type Value: Clone + Send + Sync + AsULE + 'static;
    fn make(value: Self::Value) -> Self;
}
macro_rules! primitives {($($record:ident:$value:ty),*)=>{$(impl Primitive for records::$record {type Value=$value;fn make(value:Self::Value)->Self{Self{value}}})*};}
primitives!(Real:f32,Vector2:[f32;2],Vector3:[f32;3],U32:u32);
fn values<'view, R: Primitive, T: Target>(
    document: &'view RawDocument<'_>,
    reference: Ref<T>,
) -> Result<Values<'view, R::Value>, Error>
where
    <R::Value as AsULE>::ULE: Send + Sync,
{
    let Some(section) = document
        .referenced_section(reference)
        .map_err(|error| Error::new("wem.actor.array", error.to_string()))?
    else {
        return Ok(Values::owned(Vec::new()));
    };
    if section.tag() != R::TAG || section.revision() != R::VERSION {
        return Err(Error::new(
            "wem.actor.array",
            "unexpected primitive array layout",
        ));
    }
    let width = size_of::<<R::Value as AsULE>::ULE>();
    let len = (reference.entries as usize)
        .checked_mul(width)
        .ok_or_else(|| Error::new("wem.actor.array", "array span overflow"))?;
    let bytes = section
        .bytes()
        .get(..len)
        .ok_or_else(|| Error::new("wem.actor.array", "array span exceeds section"))?;
    Ok(Values::from_zero_vec(ZeroVec::parse_bytes(bytes).map_err(
        |error| Error::new("wem.actor.array", error.to_string()),
    )?))
}

fn put<'data, R: Primitive, T: Target>(
    document: &mut RawDocument<'data>,
    values: &Values<'data, R::Value>,
) -> Result<Ref<T>, Error>
where
    <R::Value as AsULE>::ULE: Send + Sync,
{
    if values.is_empty() {
        return Ok(Ref::default());
    }
    let count = u32::try_from(values.len())
        .map_err(|_| Error::new("wem.actor.array", "count exceeds u32"))?;
    if let Some(bytes) = values.borrowed_canonical_bytes() {
        let section = document
            .append_data(R::TAG, R::VERSION, count, Cow::Borrowed(bytes))
            .map_err(|error| Error::new("wem.actor.array", error.to_string()))?;
        return Ok(Ref::from_section(count, section.index()));
    }
    let records = values.iter().map(R::make).collect::<Vec<_>>();
    let section = document
        .append(&records)
        .map_err(|error| Error::new("wem.actor.array", error.to_string()))?;
    Ok(Ref::from_section(count, section.index()))
}
fn texture(index: u32, textures: &[TextureId]) -> Result<Option<TextureId>, Error> {
    if index == u32::MAX {
        Ok(None)
    } else {
        textures
            .get(index as usize)
            .copied()
            .map(Some)
            .ok_or_else(|| {
                Error::new(
                    "wem.actor.texture",
                    "texture is absent from the shared arena",
                )
            })
    }
}
fn texture_index(
    value: Option<TextureId>,
    textures: &BTreeMap<TextureId, u32>,
) -> Result<u32, Error> {
    match value {
        None => Ok(u32::MAX),
        Some(id) => textures
            .get(&id)
            .copied()
            .ok_or_else(|| Error::new("wem.actor.texture", "texture belongs to another arena")),
    }
}

mod read;
mod write;
pub(in crate::model::conversion::wem) use read::read;
pub(in crate::model::conversion::wem) use write::write;
#[cfg(test)]
#[path = "tests.rs"]
mod tests;

mod properties;
pub(in crate::model::conversion::wem) use properties::{bind_property, property_code, Bound};
