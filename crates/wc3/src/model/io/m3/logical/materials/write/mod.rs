//! Mechanical material encoding from authored graph surfaces.
use super::super::animation::AnimationIds;
use crate::model::conversion::{Context, Error};
use crate::model::formats::m3::{self as native, records as nr};
use crate::model::io::m3::logical::Graph;
use crate::model::{
    profiles::{m3 as surface, M3},
    MaterialId, Resource, Resources, TextureId,
};
use native::{Envelope, Ref};
use std::collections::BTreeMap;
use std::str::from_utf8;
pub(in crate::model::io::m3::logical) struct Writer<'model, 'source, 'out, 'draft, E: Envelope> {
    pub model: &'model Graph<'source>,
    pub resources: &'model Resources<'source, M3>,
    pub document: &'draft mut native::Container<'out, E>,
    pub animation_ids: &'draft AnimationIds,
    pub context: &'draft mut Context,
    pub materials: BTreeMap<MaterialId, u32>,
}
impl<'model, 'source, 'out, 'draft, E: Envelope> Writer<'model, 'source, 'out, 'draft, E> {
    fn string(&mut self, text: &str) -> Result<Ref<nr::targets::Character>, Error> {
        if text.as_bytes().contains(&0) {
            return Err(Error::new(
                "m3.string",
                "native text cannot contain embedded NUL",
            ));
        }
        let mut bytes: Vec<_> = text.bytes().map(|value| nr::Character { value }).collect();
        bytes.push(nr::Character { value: 0 });
        Ok(append(self.document, &bytes)?.cast())
    }
    fn texture(&self, id: TextureId) -> Result<&'model str, Error> {
        let texture = self
            .resources
            .texture(id)
            .ok_or_else(|| Error::new("m3.texture", "texture handle is missing"))?;
        match &texture.resource {
            Resource::Path(bytes) => from_utf8(bytes)
                .map_err(|_| Error::new("m3.texture.path", "M3 texture names must be UTF-8")),
            _ => Err(Error::new(
                "m3.texture.resource",
                "M3 texture layer requires a path resource",
            )),
        }
    }
    fn layer(
        &mut self,
        layer: Option<&surface::TextureLayer>,
    ) -> Result<Ref<nr::targets::TextureLayer>, Error> {
        let Some(layer) = layer else {
            return Ok(Ref::default());
        };
        let row = texture::encode(layer, self)?;
        Ok(append(self.document, &[row])?.cast())
    }
    fn material(&self, id: MaterialId) -> Result<u32, Error> {
        self.materials
            .get(&id)
            .copied()
            .ok_or_else(|| Error::new("m3.material.link", "material has no encoded table entry"))
    }
}
fn append<E: Envelope, T: native::Record + native::Write<E> + native::RecordLayout<E>>(
    document: &mut native::Container<'_, E>,
    rows: &[T],
) -> Result<Ref<T>, Error> {
    if rows.is_empty() {
        return Ok(Ref::default());
    }
    let count = u32::try_from(rows.len())
        .map_err(|_| Error::new("m3.array", "array exceeds native count capacity"))?;
    let id = document
        .append(rows)
        .map_err(|e| Error::new("m3.write", e.to_string()))?;
    Ok(Ref::from_section(count, id.index()))
}
fn native_anim<T: Clone>(r: &surface::Property<T>, ids: &AnimationIds) -> nr::AnimationValue<T> {
    let mut interpolation = r.control.interpolation;
    let mut flags = r.control.flags.bits();
    let mut id = r.control.source_id;
    if let Some(mapped) = r
        .binding
        .as_ref()
        .and_then(|binding| ids.get(binding.target()))
    {
        id = mapped.id;
        interpolation = mapped.interpolation;
        if mapped.keyed {
            flags |= 6
        }
    }
    nr::AnimationValue {
        interpolation: nr::Interpolation::from_raw(interpolation.raw()),
        flags: nr::AnimationFlags(flags),
        animation_id: id,
        initial_value: r.initial.clone(),
        null_value: r.reset.clone(),
        reserved: r.control.reserved,
    }
}
fn native_color<E: Envelope>(
    r: &surface::Property<[f32; 4]>,
    writer: &mut Writer<'_, '_, '_, '_, E>,
) -> Result<nr::AnimationValue<[u8; 4]>, Error> {
    let initial = quantize_color(r.initial, writer.context)?;
    let reset = quantize_color(r.reset, writer.context)?;
    let value = native_anim(r, writer.animation_ids);
    Ok(nr::AnimationValue {
        interpolation: value.interpolation,
        flags: value.flags,
        animation_id: value.animation_id,
        initial_value: initial,
        null_value: reset,
        reserved: value.reserved,
    })
}
mod texture;

mod compositematerial;
mod creepmaterial;
mod displacementmaterial;
mod reflectionmaterial;
mod standard;
mod stbmaterial;
mod terrainmaterial;
mod volumematerial;
mod volumenoisematerial;

fn quantize_color(color: [f32; 4], context: &mut Context) -> Result<[u8; 4], Error> {
    let mut rgba = [0; 4];
    for (index, value) in color.into_iter().enumerate() {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(Error::new(
                "m3.material.color",
                "color is outside the native unorm range",
            ));
        }
        let lane = (value * 255.0).round() as u8;
        context.approximate(
            "m3.material.color",
            (f32::from(lane) / 255.0 - value).abs(),
            "color requires native byte quantization",
        )?;
        rgba[index] = lane
    }
    Ok([rgba[2], rgba[1], rgba[0], rgba[3]])
}
mod data_driven;
mod flare;
mod hair;
pub(in crate::model::io::m3::logical) mod tables;
