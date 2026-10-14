//! StarCraft shader inputs resolve text independently of source index sections.
use crate::model::conversion::wem::shader_materials::starcraft::{read_anim, write_anim};
use crate::model::conversion::wem::shader_materials::starcraft::{
    read_texture_layer, write_texture_layer,
};
use crate::model::conversion::wem::text;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::materials::starcraft::Standard;
use crate::model::semantic::TextureId;
use crate::model::wem::records::{
    M3BlendMode, M3LayerBlendOp, M3MaterialAdditionalFlag, M3MaterialClass, M3MaterialFlag,
    M3SpecularMode, M3Standard,
};
use crate::model::wem::Ref;
use std::borrow::Cow;

use crate::model::semantic::profiles::m3::values::{
    AdditionalMaterialFlags, BlendMode, LayerBlendOp, MaterialClass, MaterialFlags, RttChannels,
    SpecularMode,
};

use crate::model::wem::{records, InlineList, InlineOption, RawDocument};
pub(super) fn read_standard<'view>(
    document: &'view RawDocument<'_>,
    value: &M3Standard,

    texture: &mut impl FnMut(Cow<'view, [u8]>) -> Result<TextureId, Error>,
) -> Result<Standard<'view>, Error> {
    Ok(Standard {
        name: text::read(document, value.name)?,
        additional_flags: AdditionalMaterialFlags::from(value.additional_flags.bits()),
        flags: MaterialFlags::from(value.flags.bits()),
        blend_mode: BlendMode::from_raw(value.blend_mode.raw()),
        priority: value.priority,
        rtt_channels: RttChannels::from(value.rtt_channels.bits()),
        specular_exponent: value.specular_exponent,
        depth_blend_falloff: value.depth_blend_falloff,
        alpha_test_threshold: value.alpha_test_threshold,
        hdr_specular_multiplier: value.hdr_specular_multiplier,
        hdr_emissive_multiplier: value.hdr_emissive_multiplier,
        hdr_environment_constant: value.hdr_environment_constant,
        hdr_environment_diffuse: value.hdr_environment_diffuse,
        hdr_environment_specular: value.hdr_environment_specular,
        diffuse_layer: value
            .diffuse_layer
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        decal_layer: value
            .decal_layer
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        specular_layer: value
            .specular_layer
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        gloss_layer: value
            .gloss_layer
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        emissive_layer1: value
            .emissive_layer1
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        emissive_layer2: value
            .emissive_layer2
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        environment_layer: value
            .environment_layer
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        environment_mask_layer: value
            .environment_mask_layer
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        alpha_layer1: value
            .alpha_layer1
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        alpha_layer2: value
            .alpha_layer2
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        normal_layer: value
            .normal_layer
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        height_layer: value
            .height_layer
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        light_map_layer: value
            .light_map_layer
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        ambient_occlusion_layer: value
            .ambient_occlusion_layer
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        normal_blend1_mask_layer: value
            .normal_blend1_mask_layer
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        normal_blend2_mask_layer: value
            .normal_blend2_mask_layer
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        normal_blend1_layer: value
            .normal_blend1_layer
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        normal_blend2_layer: value
            .normal_blend2_layer
            .value
            .as_ref()
            .map(|value| read_texture_layer(document, value, texture))
            .transpose()?,
        material_class: MaterialClass::from_raw(value.material_class.raw()),
        layer_blend_mode: LayerBlendOp::from_raw(value.layer_blend_mode.raw()),
        emissive_blend_mode1: LayerBlendOp::from_raw(value.emissive_blend_mode1.raw()),
        emissive_blend_mode2: LayerBlendOp::from_raw(value.emissive_blend_mode2.raw()),
        specular_mode: SpecularMode::from_raw(value.specular_mode.raw()),
        parallax_height: read_anim(&value.parallax_height, |value| *value),
        motion_blur_amount: read_anim(&value.motion_blur_amount, |value| *value),
        normal_blend_factors: value
            .normal_blend_factors
            .0
            .iter()
            .map(|value| Ok::<_, Error>(read_anim(value, |value| *value)))
            .collect::<Result<Vec<_>, Error>>()?,
    })
}
pub(super) fn write_standard(
    document: &mut RawDocument<'_>,
    value: &Standard<'_>,

    texture: &impl Fn(TextureId) -> Result<Ref<records::targets::Character>, Error>,
) -> Result<M3Standard, Error> {
    Ok(M3Standard {
        name: text::write_bytes(document, value.name.as_bytes())?,
        additional_flags: M3MaterialAdditionalFlag::from_bits_retain(value.additional_flags.bits()),
        flags: M3MaterialFlag::from_bits_retain(value.flags.bits()),
        blend_mode: M3BlendMode::from_raw(value.blend_mode.raw()),
        priority: value.priority,
        rtt_channels: records::RttChannels::from_bits_retain(value.rtt_channels.bits()),
        specular_exponent: value.specular_exponent,
        depth_blend_falloff: value.depth_blend_falloff,
        alpha_test_threshold: value.alpha_test_threshold,
        hdr_specular_multiplier: value.hdr_specular_multiplier,
        hdr_emissive_multiplier: value.hdr_emissive_multiplier,
        hdr_environment_constant: value.hdr_environment_constant,
        hdr_environment_diffuse: value.hdr_environment_diffuse,
        hdr_environment_specular: value.hdr_environment_specular,
        diffuse_layer: InlineOption::from(
            value
                .diffuse_layer
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        decal_layer: InlineOption::from(
            value
                .decal_layer
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        specular_layer: InlineOption::from(
            value
                .specular_layer
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        gloss_layer: InlineOption::from(
            value
                .gloss_layer
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        emissive_layer1: InlineOption::from(
            value
                .emissive_layer1
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        emissive_layer2: InlineOption::from(
            value
                .emissive_layer2
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        environment_layer: InlineOption::from(
            value
                .environment_layer
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        environment_mask_layer: InlineOption::from(
            value
                .environment_mask_layer
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        alpha_layer1: InlineOption::from(
            value
                .alpha_layer1
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        alpha_layer2: InlineOption::from(
            value
                .alpha_layer2
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        normal_layer: InlineOption::from(
            value
                .normal_layer
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        height_layer: InlineOption::from(
            value
                .height_layer
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        light_map_layer: InlineOption::from(
            value
                .light_map_layer
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        ambient_occlusion_layer: InlineOption::from(
            value
                .ambient_occlusion_layer
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        normal_blend1_mask_layer: InlineOption::from(
            value
                .normal_blend1_mask_layer
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        normal_blend2_mask_layer: InlineOption::from(
            value
                .normal_blend2_mask_layer
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        normal_blend1_layer: InlineOption::from(
            value
                .normal_blend1_layer
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        normal_blend2_layer: InlineOption::from(
            value
                .normal_blend2_layer
                .as_ref()
                .map(|value| write_texture_layer(document, value, texture))
                .transpose()?,
        ),
        material_class: M3MaterialClass::from_raw(value.material_class.raw()),
        layer_blend_mode: M3LayerBlendOp::from_raw(value.layer_blend_mode.raw()),
        emissive_blend_mode1: M3LayerBlendOp::from_raw(value.emissive_blend_mode1.raw()),
        emissive_blend_mode2: M3LayerBlendOp::from_raw(value.emissive_blend_mode2.raw()),
        specular_mode: M3SpecularMode::from_raw(value.specular_mode.raw()),
        parallax_height: write_anim(&value.parallax_height, |value| *value),
        motion_blur_amount: write_anim(&value.motion_blur_amount, |value| *value),
        normal_blend_factors: InlineList(
            value
                .normal_blend_factors
                .iter()
                .map(|value| Ok::<_, Error>(write_anim(value, |value| *value)))
                .collect::<Result<Vec<_>, Error>>()?,
        ),
    })
}
