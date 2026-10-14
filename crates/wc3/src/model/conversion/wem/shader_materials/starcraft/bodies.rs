//! StarCraft shader inputs resolve text independently of source index sections.
use crate::model::conversion::wem::shader_materials::starcraft::{read_anim, write_anim};
use crate::model::conversion::wem::shader_materials::starcraft::{
    read_texture_layer, write_texture_layer,
};
use crate::model::conversion::wem::text;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::materials::starcraft::{
    Composite, Creep, Displacement, LensFlare, Reflection, Stb, SubFlare, Terrain, Volume,
};
use crate::model::semantic::profiles::wem::materials::starcraft::{CompositeSection, VolumeNoise};
use crate::model::semantic::TextureId;
use crate::model::wem::records::{
    M3BlendMode, M3ColorBGRA, M3Composite, M3Creep, M3Displacement, M3LensFlare, M3Reflection,
    M3Stb, M3SubFlare, M3Terrain, M3Volume,
};
use crate::model::wem::records::{
    M3CompositeSection, M3DisplacementFlags, M3ReflectionMaterialFlag, M3VolumeFalloffType,
    M3VolumeFlags, M3VolumeNoise, M3VolumeNoiseCameraMode, M3VolumeNoiseMaterialFlag,
};
use crate::model::wem::Ref;
use std::borrow::Cow;

use crate::model::semantic::profiles::m3::values::{
    BlendMode, DisplacementFlags, ReflectionFlags, VolumeCameraMode, VolumeFalloff, VolumeFlags,
    VolumeNoiseFlags,
};

use crate::model::wem::{records, InlineList, InlineOption, RawDocument};
pub(super) fn read_displacement<'view>(
    document: &'view RawDocument<'_>,
    value: &M3Displacement,

    texture: &mut impl FnMut(Cow<'view, [u8]>) -> Result<TextureId, Error>,
) -> Result<Displacement<'view>, Error> {
    Ok(Displacement {
        name: text::read(document, value.name)?,
        unknown: value.unknown,
        strength: read_anim(&value.strength, |value| *value),
        normal_map: value
            .normal_map
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        strength_map: value
            .strength_map
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        flags: DisplacementFlags::from(value.flags.bits()),
        priority: value.priority,
    })
}
pub(super) fn write_displacement(
    document: &mut RawDocument<'_>,
    value: &Displacement<'_>,

    texture: &impl Fn(TextureId) -> Result<Ref<records::targets::Character>, Error>,
) -> Result<M3Displacement, Error> {
    Ok(M3Displacement {
        name: text::write_bytes(document, value.name.as_bytes())?,
        unknown: value.unknown,
        strength: write_anim(&value.strength, |value| *value),
        normal_map: InlineOption::from(
            value
                .normal_map
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        strength_map: InlineOption::from(
            value
                .strength_map
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        flags: M3DisplacementFlags::from_bits_retain(value.flags.bits()),
        priority: value.priority,
    })
}
pub(super) fn read_composite_section(
    _document: &RawDocument<'_>,
    value: &M3CompositeSection,
) -> Result<CompositeSection, Error> {
    Ok(CompositeSection {
        material_index: value.material_index,
        map_multiplier: read_anim(&value.map_multiplier, |value| *value),
    })
}
pub(super) fn write_composite_section(
    _document: &mut RawDocument<'_>,
    value: &CompositeSection,
) -> Result<M3CompositeSection, Error> {
    Ok(M3CompositeSection {
        material_index: value.material_index,
        map_multiplier: write_anim(&value.map_multiplier, |value| *value),
    })
}
pub(super) fn read_composite<'view>(
    document: &'view RawDocument<'_>,
    value: &M3Composite,
) -> Result<Composite<'view>, Error> {
    Ok(Composite {
        name: text::read(document, value.name)?,
        priority: value.priority,
        sections: value
            .sections
            .0
            .iter()
            .map(|value| read_composite_section(document, value))
            .collect::<Result<Vec<_>, Error>>()?,
    })
}
pub(super) fn write_composite(
    document: &mut RawDocument<'_>,
    value: &Composite<'_>,
) -> Result<M3Composite, Error> {
    Ok(M3Composite {
        name: text::write_bytes(document, value.name.as_bytes())?,
        priority: value.priority,
        sections: InlineList(
            value
                .sections
                .iter()
                .map(|value| write_composite_section(document, value))
                .collect::<Result<Vec<_>, Error>>()?,
        ),
    })
}
pub(super) fn read_terrain<'view>(
    document: &'view RawDocument<'_>,
    value: &M3Terrain,

    texture: &mut impl FnMut(Cow<'view, [u8]>) -> Result<TextureId, Error>,
) -> Result<Terrain<'view>, Error> {
    Ok(Terrain {
        name: text::read(document, value.name)?,
        terrain_map: value
            .terrain_map
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        unknown: value.unknown,
    })
}
pub(super) fn write_terrain(
    document: &mut RawDocument<'_>,
    value: &Terrain<'_>,

    texture: &impl Fn(TextureId) -> Result<Ref<records::targets::Character>, Error>,
) -> Result<M3Terrain, Error> {
    Ok(M3Terrain {
        name: text::write_bytes(document, value.name.as_bytes())?,
        terrain_map: InlineOption::from(
            value
                .terrain_map
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        unknown: value.unknown,
    })
}
pub(super) fn read_volume<'view>(
    document: &'view RawDocument<'_>,
    value: &M3Volume,

    texture: &mut impl FnMut(Cow<'view, [u8]>) -> Result<TextureId, Error>,
) -> Result<Volume<'view>, Error> {
    Ok(Volume {
        name: text::read(document, value.name)?,
        blend_mode: BlendMode::from_raw(value.blend_mode.raw()),
        falloff_type: VolumeFalloff::from_raw(value.falloff_type.raw()),
        density: read_anim(&value.density, |value| *value),
        color_map: value
            .color_map
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        noise_map1: value
            .noise_map1
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        noise_map2: value
            .noise_map2
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        alpha_threshold: value.alpha_threshold,
        flags: VolumeFlags::from(value.flags.bits()),
    })
}
pub(super) fn write_volume(
    document: &mut RawDocument<'_>,
    value: &Volume<'_>,

    texture: &impl Fn(TextureId) -> Result<Ref<records::targets::Character>, Error>,
) -> Result<M3Volume, Error> {
    Ok(M3Volume {
        name: text::write_bytes(document, value.name.as_bytes())?,
        blend_mode: M3BlendMode::from_raw(value.blend_mode.raw()),
        falloff_type: M3VolumeFalloffType::from_raw(value.falloff_type.raw()),
        density: write_anim(&value.density, |value| *value),
        color_map: InlineOption::from(
            value
                .color_map
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        noise_map1: InlineOption::from(
            value
                .noise_map1
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        noise_map2: InlineOption::from(
            value
                .noise_map2
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        alpha_threshold: value.alpha_threshold,
        flags: M3VolumeFlags::from_bits_retain(value.flags.bits()),
    })
}
pub(super) fn read_volume_noise<'view>(
    document: &'view RawDocument<'_>,
    value: &M3VolumeNoise,

    texture: &mut impl FnMut(Cow<'view, [u8]>) -> Result<TextureId, Error>,
) -> Result<VolumeNoise<'view>, Error> {
    Ok(VolumeNoise {
        name: text::read(document, value.name)?,
        falloff_type: VolumeFalloff::from_raw(value.falloff_type.raw()),
        draw_transparency: VolumeCameraMode::from_raw(value.draw_transparency.raw()),
        density: read_anim(&value.density, |value| *value),
        near_plane: read_anim(&value.near_plane, |value| *value),
        falloff: read_anim(&value.falloff, |value| *value),
        color_map: value
            .color_map
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        noise_map1: value
            .noise_map1
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        noise_map2: value
            .noise_map2
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        scroll_rate: read_anim(&value.scroll_rate, |value| *value),
        position: read_anim(&value.position, |value| *value),
        scale: read_anim(&value.scale, |value| *value),
        rotation: read_anim(&value.rotation, |value| *value),
        alpha_threshold: value.alpha_threshold,
        flags: VolumeNoiseFlags::from(value.flags.bits()),
    })
}
pub(super) fn write_volume_noise(
    document: &mut RawDocument<'_>,
    value: &VolumeNoise<'_>,

    texture: &impl Fn(TextureId) -> Result<Ref<records::targets::Character>, Error>,
) -> Result<M3VolumeNoise, Error> {
    Ok(M3VolumeNoise {
        name: text::write_bytes(document, value.name.as_bytes())?,
        falloff_type: M3VolumeFalloffType::from_raw(value.falloff_type.raw()),
        draw_transparency: M3VolumeNoiseCameraMode::from_raw(value.draw_transparency.raw()),
        density: write_anim(&value.density, |value| *value),
        near_plane: write_anim(&value.near_plane, |value| *value),
        falloff: write_anim(&value.falloff, |value| *value),
        color_map: InlineOption::from(
            value
                .color_map
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        noise_map1: InlineOption::from(
            value
                .noise_map1
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        noise_map2: InlineOption::from(
            value
                .noise_map2
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        scroll_rate: write_anim(&value.scroll_rate, |value| *value),
        position: write_anim(&value.position, |value| *value),
        scale: write_anim(&value.scale, |value| *value),
        rotation: write_anim(&value.rotation, |value| *value),
        alpha_threshold: value.alpha_threshold,
        flags: M3VolumeNoiseMaterialFlag::from_bits_retain(value.flags.bits()),
    })
}
pub(super) fn read_creep<'view>(
    document: &'view RawDocument<'_>,
    value: &M3Creep,

    texture: &mut impl FnMut(Cow<'view, [u8]>) -> Result<TextureId, Error>,
) -> Result<Creep<'view>, Error> {
    Ok(Creep {
        name: text::read(document, value.name)?,
        mask_map: value
            .mask_map
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        creep_low: value.creep_low,
    })
}
pub(super) fn write_creep(
    document: &mut RawDocument<'_>,
    value: &Creep<'_>,

    texture: &impl Fn(TextureId) -> Result<Ref<records::targets::Character>, Error>,
) -> Result<M3Creep, Error> {
    Ok(M3Creep {
        name: text::write_bytes(document, value.name.as_bytes())?,
        mask_map: InlineOption::from(
            value
                .mask_map
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        creep_low: value.creep_low,
    })
}
pub(super) fn read_stb<'view>(
    document: &'view RawDocument<'_>,
    value: &M3Stb,

    texture: &mut impl FnMut(Cow<'view, [u8]>) -> Result<TextureId, Error>,
) -> Result<Stb<'view>, Error> {
    Ok(Stb {
        name: text::read(document, value.name)?,
        diffuse_map: value
            .diffuse_map
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        normal_map: value
            .normal_map
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        specular_map: value
            .specular_map
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
    })
}
pub(super) fn write_stb(
    document: &mut RawDocument<'_>,
    value: &Stb<'_>,

    texture: &impl Fn(TextureId) -> Result<Ref<records::targets::Character>, Error>,
) -> Result<M3Stb, Error> {
    Ok(M3Stb {
        name: text::write_bytes(document, value.name.as_bytes())?,
        diffuse_map: InlineOption::from(
            value
                .diffuse_map
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        normal_map: InlineOption::from(
            value
                .normal_map
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        specular_map: InlineOption::from(
            value
                .specular_map
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
    })
}
pub(super) fn read_reflection<'view>(
    document: &'view RawDocument<'_>,
    value: &M3Reflection,

    texture: &mut impl FnMut(Cow<'view, [u8]>) -> Result<TextureId, Error>,
) -> Result<Reflection<'view>, Error> {
    Ok(Reflection {
        name: text::read(document, value.name)?,
        unknown: value.unknown,
        reflection_strength: read_anim(&value.reflection_strength, |value| *value),
        displacement_strength: read_anim(&value.displacement_strength, |value| *value),
        reflection_offset: read_anim(&value.reflection_offset, |value| *value),
        blur_angle: read_anim(&value.blur_angle, |value| *value),
        blur_distance_max: read_anim(&value.blur_distance_max, |value| *value),
        reflection_map: value
            .reflection_map
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        displacement_map: value
            .displacement_map
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        blur_map: value
            .blur_map
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        flags: ReflectionFlags::from(value.flags.bits()),
        unknown2: value.unknown2,
    })
}
pub(super) fn write_reflection(
    document: &mut RawDocument<'_>,
    value: &Reflection<'_>,

    texture: &impl Fn(TextureId) -> Result<Ref<records::targets::Character>, Error>,
) -> Result<M3Reflection, Error> {
    Ok(M3Reflection {
        name: text::write_bytes(document, value.name.as_bytes())?,
        unknown: value.unknown,
        reflection_strength: write_anim(&value.reflection_strength, |value| *value),
        displacement_strength: write_anim(&value.displacement_strength, |value| *value),
        reflection_offset: write_anim(&value.reflection_offset, |value| *value),
        blur_angle: write_anim(&value.blur_angle, |value| *value),
        blur_distance_max: write_anim(&value.blur_distance_max, |value| *value),
        reflection_map: InlineOption::from(
            value
                .reflection_map
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        displacement_map: InlineOption::from(
            value
                .displacement_map
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        blur_map: InlineOption::from(
            value
                .blur_map
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        flags: M3ReflectionMaterialFlag::from_bits_retain(value.flags.bits()),
        unknown2: value.unknown2,
    })
}
pub(super) fn read_sub_flare(
    _document: &RawDocument<'_>,
    value: &M3SubFlare,
) -> Result<SubFlare, Error> {
    Ok(SubFlare {
        index: value.index,
        position: value.position,
        size_xy: value.size_xy,
        scale_xy: value.scale_xy,
        fade_in: value.fade_in,
        fade_out: value.fade_out,
        color_alpha: [
            value.color_alpha.r,
            value.color_alpha.g,
            value.color_alpha.b,
            value.color_alpha.a,
        ],
        face_center: value.face_center,
        offset: value.offset,
    })
}
pub(super) fn write_sub_flare(
    _document: &mut RawDocument<'_>,
    value: &SubFlare,
) -> Result<M3SubFlare, Error> {
    Ok(M3SubFlare {
        index: value.index,
        position: value.position,
        size_xy: value.size_xy,
        scale_xy: value.scale_xy,
        fade_in: value.fade_in,
        fade_out: value.fade_out,
        color_alpha: M3ColorBGRA {
            r: value.color_alpha[0],
            g: value.color_alpha[1],
            b: value.color_alpha[2],
            a: value.color_alpha[3],
        },
        face_center: value.face_center,
        offset: value.offset,
    })
}
pub(super) fn read_lens_flare<'view>(
    document: &'view RawDocument<'_>,
    value: &M3LensFlare,

    texture: &mut impl FnMut(Cow<'view, [u8]>) -> Result<TextureId, Error>,
) -> Result<LensFlare<'view>, Error> {
    Ok(LensFlare {
        name: text::read(document, value.name)?,
        flare_map: value
            .flare_map
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        mask_map: value
            .mask_map
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        sub_flares: value
            .sub_flares
            .0
            .iter()
            .map(|value| read_sub_flare(document, value))
            .collect::<Result<Vec<_>, Error>>()?,
        columns: value.columns,
        rows: value.rows,
        distance_fade: value.distance_fade,
        lib_name: text::read(document, value.lib_name)?,
        intensity: read_anim(&value.intensity, |value| *value),
        color: read_anim(&value.color, |value| [value.r, value.g, value.b, value.a]),
        hdr: read_anim(&value.hdr, |value| *value),
        size: read_anim(&value.size, |value| *value),
    })
}
pub(super) fn write_lens_flare(
    document: &mut RawDocument<'_>,
    value: &LensFlare<'_>,

    texture: &impl Fn(TextureId) -> Result<Ref<records::targets::Character>, Error>,
) -> Result<M3LensFlare, Error> {
    Ok(M3LensFlare {
        name: text::write_bytes(document, value.name.as_bytes())?,
        flare_map: InlineOption::from(
            value
                .flare_map
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        mask_map: InlineOption::from(
            value
                .mask_map
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        sub_flares: InlineList(
            value
                .sub_flares
                .iter()
                .map(|value| write_sub_flare(document, value))
                .collect::<Result<Vec<_>, Error>>()?,
        ),
        columns: value.columns,
        rows: value.rows,
        distance_fade: value.distance_fade,
        lib_name: text::write_bytes(document, value.lib_name.as_bytes())?,
        intensity: write_anim(&value.intensity, |value| *value),
        color: write_anim(&value.color, |value| M3ColorBGRA {
            r: value[0],
            g: value[1],
            b: value[2],
            a: value[3],
        }),
        hdr: write_anim(&value.hdr, |value| *value),
        size: write_anim(&value.size, |value| *value),
    })
}
