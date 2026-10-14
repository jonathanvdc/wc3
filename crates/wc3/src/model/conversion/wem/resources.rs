//! Resource identities, authored paths, and image roles remain separate authorities.
use crate::model::conversion::wem::text;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::{
    SupplementalTextureFlags, TextureControls, TextureSlot,
};
use crate::model::semantic::profiles::Profile;
use crate::model::semantic::{AddressMode, ColorSpace, Resource, Sampler, Texture};
use crate::model::wem::{records, RawDocument};

pub(super) fn read<'view, P>(
    document: &'view RawDocument<'_>,
    value: &records::TextureRecord,
) -> Result<Texture<'view, P>, Error>
where
    P: Profile<Texture<'view> = TextureControls<'view>>,
{
    let resource = match &value.key {
        records::TextureKey::None => Resource::Unspecified,
        records::TextureKey::Path(key) => Resource::Path(text::read_bytes(document, key.value)?),
        records::TextureKey::FileDataId(key) => Resource::FileDataId(key.value),
        records::TextureKey::SnoId(key) => Resource::Sno {
            group: i32::from_ne_bytes(key.group.to_ne_bytes()),
            id: key.id,
        },
    };
    let slot = match value.slot_type {
        records::TextureSlot::Diffuse => TextureSlot::Diffuse,
        records::TextureSlot::Normal => TextureSlot::Normal,
        records::TextureSlot::Specular => TextureSlot::Specular,
        records::TextureSlot::Gloss => TextureSlot::Gloss,
        records::TextureSlot::Emissive => TextureSlot::Emissive,
        records::TextureSlot::Environment => TextureSlot::Environment,
        records::TextureSlot::AmbientOcclusion => TextureSlot::AmbientOcclusion,
        records::TextureSlot::Height => TextureSlot::Height,
        records::TextureSlot::Lightmap => TextureSlot::Lightmap,
        records::TextureSlot::Detail => TextureSlot::Detail,
        records::TextureSlot::Count => TextureSlot::Count,
        records::TextureSlot::Unknown(code) => TextureSlot::Unknown(code),
    };
    Ok(Texture {
        resource,
        color_space: match value.declared_space {
            records::ColorSpace::Auto => ColorSpace::Unspecified,
            records::ColorSpace::Linear => ColorSpace::Linear,
            records::ColorSpace::Srgb => ColorSpace::Srgb,
            records::ColorSpace::Unknown(code) => ColorSpace::Unknown(code),
        },
        sampler: Sampler {
            u: if value.flags.wrap_u() {
                AddressMode::Repeat
            } else {
                AddressMode::Clamp
            },
            v: if value.flags.wrap_v() {
                AddressMode::Repeat
            } else {
                AddressMode::Clamp
            },
        },
        controls: TextureControls {
            path: if value.path.entries == 0 {
                None
            } else {
                Some(text::read_bytes(document, value.path)?)
            },
            replaceable: (value.replaceable_id != 0).then_some(value.replaceable_id),
            slot,
            flags: SupplementalTextureFlags::from_bits_retain(value.flags.bits()),
        },
    })
}
pub(super) fn write<'data, P>(
    document: &mut RawDocument<'_>,
    texture: &Texture<'data, P>,
) -> Result<records::TextureRecord, Error>
where
    P: Profile<Texture<'data> = TextureControls<'data>>,
{
    let key = match &texture.resource {
        Resource::Unspecified => records::TextureKey::None,
        Resource::Path(bytes) => records::TextureKey::Path(records::TexturePath {
            value: text::write_bytes(document, bytes)?,
        }),
        Resource::FileDataId(value) => {
            records::TextureKey::FileDataId(records::TextureFileDataId { value: *value })
        }
        Resource::Sno { group, id } => records::TextureKey::SnoId(records::TextureSnoId {
            group: u32::from_ne_bytes(group.to_ne_bytes()),
            id: *id,
        }),
        _ => {
            return Err(Error::new(
                "wem.texture.resource",
                "this identity requires an explicit resource translation",
            ));
        }
    };
    let controls = &texture.controls;
    let slot_type = match controls.slot {
        TextureSlot::Diffuse => records::TextureSlot::Diffuse,
        TextureSlot::Normal => records::TextureSlot::Normal,
        TextureSlot::Specular => records::TextureSlot::Specular,
        TextureSlot::Gloss => records::TextureSlot::Gloss,
        TextureSlot::Emissive => records::TextureSlot::Emissive,
        TextureSlot::Environment => records::TextureSlot::Environment,
        TextureSlot::AmbientOcclusion => records::TextureSlot::AmbientOcclusion,
        TextureSlot::Height => records::TextureSlot::Height,
        TextureSlot::Lightmap => records::TextureSlot::Lightmap,
        TextureSlot::Detail => records::TextureSlot::Detail,
        TextureSlot::Count => records::TextureSlot::Count,
        TextureSlot::Unknown(code) => records::TextureSlot::Unknown(code),
    };
    if !matches!(texture.sampler.u, AddressMode::Repeat | AddressMode::Clamp)
        || !matches!(texture.sampler.v, AddressMode::Repeat | AddressMode::Clamp)
    {
        return Err(Error::new(
            "wem.texture.sampler",
            "image flags represent repeat or clamp addressing",
        ));
    }
    let mut flags = records::TextureFlags::from_bits_retain(controls.flags.bits());
    flags.set_wrap_u(texture.sampler.u == AddressMode::Repeat);
    flags.set_wrap_v(texture.sampler.v == AddressMode::Repeat);
    Ok(records::TextureRecord {
        key,
        path: match &controls.path {
            Some(bytes) => text::write_bytes(document, bytes)?,
            None => Default::default(),
        },
        flags,
        replaceable_id: controls.replaceable.unwrap_or(0),
        slot_type,
        declared_space: match texture.color_space {
            ColorSpace::Unspecified => records::ColorSpace::Auto,
            ColorSpace::Srgb => records::ColorSpace::Srgb,
            ColorSpace::Linear => records::ColorSpace::Linear,
            ColorSpace::Unknown(code) => records::ColorSpace::Unknown(code),
        },
    })
}
