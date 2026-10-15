//! Mechanical standard surface encoding.
use super::{append, native_anim, nr, surface, Envelope, Error, Writer};
pub(super) fn encode<E: Envelope>(
    name: &str,
    r: &surface::Standard,
    writer: &mut Writer<'_, '_, '_, '_, E>,
) -> Result<nr::StandardMaterial<nr::V20>, Error> {
    Ok(nr::StandardMaterial {
        name: writer.string(name)?,
        additional_flags: nr::AdditionalMaterialFlags(
            r.state.rasterization.additional_flags.bits(),
        ),
        flags: nr::MaterialFlags(r.state.rasterization.flags.bits()),
        blend_mode: nr::BlendMode::from_raw(r.state.blend.raw()),
        priority: r.state.ordering,
        rtt_channels: nr::RttChannels(r.state.rasterization.rtt_channels.bits()),
        specular_exponent: r.specular.parameters.exponent,
        depth_blend_falloff: r.state.rasterization.depth_blend_falloff,
        alpha_test_threshold: r.state.rasterization.alpha_test_threshold,
        hdr_specular_multiplier: r.specular.parameters.hdr_multiplier,
        hdr_emissive_multiplier: r.emissive.parameters,
        hdr_environment_constant: r.hdr_environment_constant,
        hdr_environment_diffuse: r.hdr_environment_diffuse,
        hdr_environment_specular: r.hdr_environment_specular,
        diffuse_layer: writer.layer(r.diffuse.input.as_ref())?,
        decal_layer: writer.layer(r.decal_layer.as_ref())?,
        specular_layer: writer.layer(r.specular.input.color.as_ref())?,
        gloss_layer: writer.layer(r.specular.input.gloss.as_ref())?,
        emissive_layer1: writer.layer(r.emissive.input.primary.input.as_ref())?,
        emissive_layer2: writer.layer(r.emissive.input.secondary.input.as_ref())?,
        environment_layer: writer.layer(r.environment_layer.as_ref())?,
        environment_mask_layer: writer.layer(r.environment_mask_layer.as_ref())?,
        alpha_layer1: writer.layer(r.opacity.primary.as_ref())?,
        alpha_layer2: writer.layer(r.opacity.secondary.as_ref())?,
        normal_layer: writer.layer(r.normals.base.as_ref())?,
        height_layer: writer.layer(r.height_layer.as_ref())?,
        light_map_layer: writer.layer(r.light_map_layer.as_ref())?,
        ambient_occlusion_layer: writer.layer(r.ambient_occlusion_layer.as_ref())?,
        normal_blend1_mask_layer: writer.layer(r.normals.primary.mask.as_ref())?,
        normal_blend2_mask_layer: writer.layer(r.normals.secondary.mask.as_ref())?,
        normal_blend1_layer: writer.layer(r.normals.primary.input.as_ref())?,
        normal_blend2_layer: writer.layer(r.normals.secondary.input.as_ref())?,
        material_class: nr::MaterialClass::from_raw(r.material_class.raw()),
        layer_blend_mode: nr::LayerBlendOp::from_raw(r.diffuse.parameters.raw()),
        emissive_blend_mode1: nr::LayerBlendOp::from_raw(r.emissive.input.primary.parameters.raw()),
        emissive_blend_mode2: nr::LayerBlendOp::from_raw(
            r.emissive.input.secondary.parameters.raw(),
        ),
        specular_mode: nr::SpecularMode::from_raw(r.specular.parameters.mode.raw()),
        parallax_height: native_anim(&r.parallax_height, writer.animation_ids),
        motion_blur_amount: native_anim(&r.motion_blur_amount, writer.animation_ids),
        normal_blend_factors: append(
            writer.document,
            &r.normals
                .weights
                .iter()
                .map(|r| nr::AnimationValueReal {
                    value: native_anim(r, writer.animation_ids),
                })
                .collect::<Vec<_>>(),
        )?
        .cast(),
    })
}
