//! Texture inputs resolve document indices into shared resource identities.
use crate::model::conversion::wem::materials::values::{
    read_uv_mapping_mode, write_uv_mapping_mode,
};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::materials::Input;
use crate::model::semantic::{AddressMode, ColorSpace, Sampler, TextureId};
use crate::model::wem::records;
use std::collections::BTreeMap;
pub(super) fn read(value: &records::TextureInput, textures: &[TextureId]) -> Result<Input, Error> {
    let texture = if value.texture == u32::MAX {
        None
    } else {
        Some(*textures.get(value.texture as usize).ok_or_else(|| {
            Error::new(
                "wem.material.texture",
                "resource index exceeds the document texture table",
            )
        })?)
    };
    Ok(Input {
        texture,
        constant: value.constant,
        uv_set: value.uv_set,
        mapping: read_uv_mapping_mode(value.mapping),
        color_space: match value.color_space {
            records::ColorSpace::Auto => ColorSpace::Unspecified,
            records::ColorSpace::Linear => ColorSpace::Linear,
            records::ColorSpace::Srgb => ColorSpace::Srgb,
            records::ColorSpace::Unknown(code) => ColorSpace::Unknown(code),
        },
        sampler: Sampler {
            u: read_wrap(value.wrap_u),
            v: read_wrap(value.wrap_v),
        },
        uv_transform: value.uv_transform.m,
        weight: value.weight,
    })
}
fn read_wrap(value: records::WrapMode) -> AddressMode {
    match value {
        records::WrapMode::Clamp => AddressMode::Clamp,
        records::WrapMode::Repeat => AddressMode::Repeat,
        records::WrapMode::Mirror => AddressMode::Mirror,
        records::WrapMode::Unknown(code) => AddressMode::Unknown(code),
    }
}
fn write_wrap(value: AddressMode) -> records::WrapMode {
    match value {
        AddressMode::Clamp => records::WrapMode::Clamp,
        AddressMode::Repeat => records::WrapMode::Repeat,
        AddressMode::Mirror => records::WrapMode::Mirror,
        AddressMode::Unknown(code) => records::WrapMode::Unknown(code),
    }
}
pub(super) fn write(
    value: &Input,
    textures: &BTreeMap<TextureId, u32>,
) -> Result<records::TextureInput, Error> {
    Ok(records::TextureInput {
        texture: match value.texture {
            None => u32::MAX,
            Some(id) => *textures.get(&id).ok_or_else(|| {
                Error::new(
                    "wem.material.texture",
                    "resource identity is absent from the output table",
                )
            })?,
        },
        constant: value.constant,
        uv_set: value.uv_set,
        mapping: write_uv_mapping_mode(value.mapping),
        color_space: match value.color_space {
            ColorSpace::Unspecified => records::ColorSpace::Auto,
            ColorSpace::Linear => records::ColorSpace::Linear,
            ColorSpace::Srgb => records::ColorSpace::Srgb,
            ColorSpace::Unknown(code) => records::ColorSpace::Unknown(code),
        },
        wrap_u: write_wrap(value.sampler.u),
        wrap_v: write_wrap(value.sampler.v),
        uv_transform: records::Matrix3x2f {
            m: value.uv_transform,
        },
        weight: value.weight,
    })
}
