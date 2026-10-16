//! Direct native M3 surfaces into the shared graph.
use crate::model::conversion::Error;
use crate::model::formats::m3::{self as native, records as nr};
use crate::model::io::m3::logical::GraphBuilder;
use crate::model::{self as sem, profiles::m3 as surface};
use native::Envelope;
use sem::{MaterialId, TextureId};
use std::borrow::Cow;
use std::collections::BTreeMap;

/// Temporary import state; no native document is retained by the finished graph.
pub(super) struct Reader<'a, 'draft> {
    pub builder: &'draft mut GraphBuilder<'a>,
    pub textures: BTreeMap<&'a str, TextureId>,
    pub materials: Vec<MaterialId>,
    pub name: Option<Cow<'a, str>>,
}
impl<'a> Reader<'a, '_> {
    fn texture(&mut self, path: &'a str) -> Result<Option<TextureId>, Error> {
        if path.is_empty() {
            return Ok(None);
        }
        if let Some(id) = self.textures.get(path) {
            return Ok(Some(*id));
        }
        let resource = self
            .builder
            .add_image_resource(sem::PathResource::Path(path.as_bytes().into()));
        let id = self.builder.add_texture(sem::Texture::new(resource))?;
        self.textures.insert(path, id);
        Ok(Some(id))
    }
    fn material(&self, index: u32) -> Result<MaterialId, Error> {
        self.materials
            .get(index as usize)
            .copied()
            .ok_or_else(|| Error::new("m3.material", "material link is outside the logical table"))
    }
}
pub(super) fn read_error(error: native::ReadError) -> Error {
    Error::new("m3.material", error.to_string())
}
pub(super) fn string<'a, E: Envelope>(
    source: &'a native::Container<'_, E>,
    reference: native::Ref<nr::targets::Character>,
) -> Result<Cow<'a, str>, Error> {
    Ok(Cow::Borrowed(
        source
            .string(reference)
            .map_err(read_error)?
            .unwrap_or_default(),
    ))
}
pub(super) fn revision<E: Envelope, T: native::Target>(
    source: &native::Container<'_, E>,
    reference: native::Ref<T>,
) -> Result<u32, native::ReadError> {
    source
        .referenced_section(reference)?
        .map(|section| section.revision())
        .ok_or_else(|| native::ReadError::invalid(0, "missing material section"))
}
pub(super) fn anim<T>(r: nr::AnimationValue<T>) -> surface::Property<T> {
    surface::Property {
        initial: r.initial_value,
        reset: r.null_value,
        binding: None,
        control: surface::AnimationControl {
            interpolation: surface::values::Interpolation::from_raw(r.interpolation.raw()),
            flags: surface::values::AnimationFlags(r.flags.bits()),
            source_id: r.animation_id,
            reserved: r.reserved,
        },
    }
}
pub(super) fn color(r: nr::AnimationValue<[u8; 4]>) -> surface::Property<[f32; 4]> {
    let rgba = |value: [u8; 4]| {
        [value[2], value[1], value[0], value[3]].map(|lane| f32::from(lane) / 255.)
    };
    let value = nr::AnimationValue {
        initial_value: rgba(r.initial_value),
        null_value: rgba(r.null_value),
        interpolation: r.interpolation,
        flags: r.flags,
        animation_id: r.animation_id,
        reserved: r.reserved,
    };
    anim(value)
}
pub(super) fn fixed_color(r: nr::AnimationValue<nr::FixedBytes<4>>) -> surface::Property<[f32; 4]> {
    color(nr::AnimationValue {
        initial_value: r.initial_value.0,
        null_value: r.null_value.0,
        interpolation: r.interpolation,
        flags: r.flags,
        animation_id: r.animation_id,
        reserved: r.reserved,
    })
}
fn identity<T>(value: T) -> T {
    value
}
pub(super) trait Slot<T> {
    fn value(&self) -> T;
}
impl<T: Default> Slot<T> for () {
    fn value(&self) -> T {
        T::default()
    }
}
macro_rules! slot {($($t:ty),*)=>{$(impl Slot<$t> for $t{fn value(&self)->Self{*self}})*};}
slot!(f32, u32, [f32; 2], [f32; 3], nr::RegionFlags);
impl<T: native::Target> Slot<native::Ref<T>> for native::Ref<T> {
    fn value(&self) -> Self {
        *self
    }
}
impl<T: Clone> Slot<nr::AnimationValue<T>> for nr::AnimationValue<T> {
    fn value(&self) -> Self {
        self.clone()
    }
}

mod compositematerial;
mod creepmaterial;
mod data_driven;
mod dispatch;
mod displacementmaterial;
mod flare;
mod hair;
mod reflectionmaterial;
mod standard;
mod stbmaterial;
mod terrainmaterial;
mod texture;
mod volumematerial;
mod volumenoisematerial;
pub(super) use dispatch::read;

pub(super) mod write;
