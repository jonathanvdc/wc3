//! StarCraft shader inputs resolve text independently of source index sections.
use crate::model::conversion::Error;
use crate::model::semantic::profiles::m3::values::{AnimationFlags, Interpolation, MaterialKind};
use crate::model::semantic::profiles::m3::AnimationControl;
use crate::model::semantic::profiles::wem::materials::starcraft::{
    Body, ExternalAnimation, StarcraftMaterial,
};
use crate::model::semantic::TextureId;
use crate::model::wem::records::M3MaterialDialect;
use crate::model::wem::records::M3MaterialKind;
use crate::model::wem::records::{M3AnimRef, M3AnimationFlags, M3Body, M3Material};

use crate::model::wem::Ref;
use crate::model::wem::{records, InlineOption, RawDocument};
use std::borrow::Cow;
use std::marker::PhantomData;
mod layer;
use layer::{read_texture_layer, write_texture_layer};

mod standard;
use standard::{read_standard, write_standard};
mod bodies;
use bodies::{
    read_composite, read_creep, read_displacement, read_lens_flare, read_reflection, read_stb,
    read_terrain, read_volume, read_volume_noise, write_composite, write_creep, write_displacement,
    write_lens_flare, write_reflection, write_stb, write_terrain, write_volume, write_volume_noise,
};
mod data_driven;
use data_driven::{
    read_data_driven, read_data_driven_properties, write_data_driven, write_data_driven_properties,
};
fn read_anim<T, U>(value: &M3AnimRef<T>, map: impl Fn(&T) -> U) -> ExternalAnimation<U> {
    ExternalAnimation {
        initial: map(&value.init_value),
        reset: map(&value.null_value),
        control: AnimationControl {
            interpolation: Interpolation::from_raw(value.interp_type.raw()),
            flags: AnimationFlags::from(value.flags.bits()),
            source_id: value.anim_id,
            reserved: value.unused,
        },
    }
}
fn write_anim<T, U>(value: &ExternalAnimation<T>, map: impl Fn(&T) -> U) -> M3AnimRef<U> {
    M3AnimRef {
        init_value: map(&value.initial),
        null_value: map(&value.reset),
        interp_type: records::NativeInterpolation16::from_raw(value.control.interpolation.raw()),
        flags: M3AnimationFlags::from_bits_retain(value.control.flags.bits()),
        anim_id: value.control.source_id,
        unused: value.control.reserved,
    }
}
pub(in crate::model::conversion::wem) fn read<
    'view,
    D: M3MaterialDialect,
    A: records::NativeLayout,
>(
    document: &'view RawDocument<'_>,
    value: &M3Material<D, A>,

    texture: &mut impl FnMut(Cow<'view, [u8]>) -> Result<TextureId, Error>,
) -> Result<StarcraftMaterial<'view>, Error> {
    let body = match &value.body.body {
        M3Body::Standard(value) => {
            Body::Standard(Box::new(read_standard(document, value, texture)?))
        }
        M3Body::Displacement(value) => {
            Body::Displacement(Box::new(read_displacement(document, value, texture)?))
        }
        M3Body::Composite(value) => Body::Composite(Box::new(read_composite(document, value)?)),
        M3Body::Terrain(value) => Body::Terrain(Box::new(read_terrain(document, value, texture)?)),
        M3Body::Volume(value) => Body::Volume(Box::new(read_volume(document, value, texture)?)),
        M3Body::Creep(value) => Body::Creep(Box::new(read_creep(document, value, texture)?)),
        M3Body::VolumeNoise(value) => {
            Body::VolumeNoise(Box::new(read_volume_noise(document, value, texture)?))
        }
        M3Body::Stb(value) => Body::Stb(Box::new(read_stb(document, value, texture)?)),
        M3Body::Reflection(value) => {
            Body::Reflection(Box::new(read_reflection(document, value, texture)?))
        }
        M3Body::LensFlare(value) => {
            Body::LensFlare(Box::new(read_lens_flare(document, value, texture)?))
        }
        M3Body::DataDriven(value) => Body::DataDriven(Box::new(read_data_driven(document, value)?)),
    };
    Ok(StarcraftMaterial {
        declared_family: MaterialKind::from_raw(value.kind.raw()),
        authored_data_driven: value.authored_data_driven.get(),
        body,
        data_driven_properties: value
            .data_driven_properties
            .value
            .as_ref()
            .map(|value| read_data_driven_properties(document, value))
            .transpose()?,
        restored_standard: value
            .restored_standard
            .value
            .as_ref()
            .map(|value| read_standard(document, value, texture))
            .transpose()?,
    })
}
pub(in crate::model::conversion::wem) fn write(
    document: &mut RawDocument<'_>,
    value: &StarcraftMaterial<'_>,
    source_version: u32,

    texture: &impl Fn(TextureId) -> Result<Ref<records::targets::Character>, Error>,
) -> Result<M3Material<records::V19, records::Abi64>, Error> {
    let body = match &value.body {
        Body::Standard(value) => {
            M3Body::Standard(Box::new(write_standard(document, value, texture)?))
        }
        Body::Displacement(value) => {
            M3Body::Displacement(Box::new(write_displacement(document, value, texture)?))
        }
        Body::Composite(value) => M3Body::Composite(Box::new(write_composite(document, value)?)),
        Body::Terrain(value) => M3Body::Terrain(Box::new(write_terrain(document, value, texture)?)),
        Body::Volume(value) => M3Body::Volume(Box::new(write_volume(document, value, texture)?)),
        Body::Creep(value) => M3Body::Creep(Box::new(write_creep(document, value, texture)?)),
        Body::VolumeNoise(value) => {
            M3Body::VolumeNoise(Box::new(write_volume_noise(document, value, texture)?))
        }
        Body::Stb(value) => M3Body::Stb(Box::new(write_stb(document, value, texture)?)),
        Body::Reflection(value) => {
            M3Body::Reflection(Box::new(write_reflection(document, value, texture)?))
        }
        Body::LensFlare(value) => {
            M3Body::LensFlare(Box::new(write_lens_flare(document, value, texture)?))
        }
        Body::DataDriven(value) => {
            M3Body::DataDriven(Box::new(write_data_driven(document, value)?))
        }
    };
    Ok(M3Material {
        source_version,
        kind: M3MaterialKind::from_raw(value.declared_family.raw()),
        authored_data_driven: value.authored_data_driven.into(),
        body: records::NativeM3Body {
            body,
            abi: PhantomData,
        },
        data_driven_properties: InlineOption::from(
            value
                .data_driven_properties
                .as_ref()
                .map(|value| write_data_driven_properties(document, value))
                .transpose()?,
        ),
        restored_standard: InlineOption::from(
            value
                .restored_standard
                .as_ref()
                .map(|value| write_standard(document, value, texture))
                .transpose()?,
        ),
    })
}

#[cfg(test)]
mod tests;
