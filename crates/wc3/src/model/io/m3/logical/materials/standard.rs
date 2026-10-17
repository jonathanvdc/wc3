//! Direct logical standard decoding from native M3 fields.
use super::texture::get_layer;
use super::{
    anim, identity, native, nr, read_error, revision, string, surface, Envelope, Error, Reader,
    Slot,
};
use crate::model::materials::contributions::{
    MaskedInput, NormalContributions, PairedInputs, SpecularInputs, SurfaceContribution,
};
use crate::model::materials::m3::SpecularResponse;
use crate::model::materials::{m3::StandardRasterization, state::PassState};
pub(super) fn standard<'a, E: Envelope, D: nr::StandardMaterialDialect>(
    r: nr::StandardMaterial<D>,
    source: &'a native::Container<'_, E>,
    reader: &mut Reader<'a, '_>,
) -> Result<surface::StandardMaterialStep, Error>
where
    D::HdrEnvironmentConstant: Slot<f32>,
    D::HdrEnvironmentDiffuse: Slot<f32>,
    D::HdrEnvironmentSpecular: Slot<f32>,
    D::GlossLayer: Slot<native::Ref<nr::targets::TextureLayer>>,
    D::NormalBlend1MaskLayer: Slot<native::Ref<nr::targets::TextureLayer>>,
    D::NormalBlend2MaskLayer: Slot<native::Ref<nr::targets::TextureLayer>>,
    D::NormalBlend1Layer: Slot<native::Ref<nr::targets::TextureLayer>>,
    D::NormalBlend2Layer: Slot<native::Ref<nr::targets::TextureLayer>>,
    D::NormalBlendFactors: Slot<native::Ref<nr::targets::AnimationValueReal>>,
{
    reader.name = Some(string(source, r.name)?);
    let diffuse_layer = get_layer(source, reader, r.diffuse_layer)?;
    let decal_layer = get_layer(source, reader, r.decal_layer)?;
    let specular_layer = get_layer(source, reader, r.specular_layer)?;
    let gloss_layer = get_layer(source, reader, r.gloss_layer.value())?;
    let emissive_layer1 = get_layer(source, reader, r.emissive_layer1)?;
    let emissive_layer2 = get_layer(source, reader, r.emissive_layer2)?;
    let environment_layer = get_layer(source, reader, r.environment_layer)?;
    let environment_mask_layer = get_layer(source, reader, r.environment_mask_layer)?;
    let alpha_layer1 = get_layer(source, reader, r.alpha_layer1)?;
    let alpha_layer2 = get_layer(source, reader, r.alpha_layer2)?;
    let normal_layer = get_layer(source, reader, r.normal_layer)?;
    let height_layer = get_layer(source, reader, r.height_layer)?;
    let light_map_layer = get_layer(source, reader, r.light_map_layer)?;
    let ambient_occlusion_layer = get_layer(source, reader, r.ambient_occlusion_layer)?;
    let normal_blend1_mask_layer = get_layer(source, reader, r.normal_blend1_mask_layer.value())?;
    let normal_blend2_mask_layer = get_layer(source, reader, r.normal_blend2_mask_layer.value())?;
    let normal_blend1_layer = get_layer(source, reader, r.normal_blend1_layer.value())?;
    let normal_blend2_layer = get_layer(source, reader, r.normal_blend2_layer.value())?;
    Ok(surface::StandardMaterialStep {
        hdr_environment_constant: r.hdr_environment_constant.value(),
        hdr_environment_diffuse: r.hdr_environment_diffuse.value(),
        hdr_environment_specular: r.hdr_environment_specular.value(),
        decal_layer,
        environment_layer,
        environment_mask_layer,
        height_layer,
        light_map_layer,
        ambient_occlusion_layer,
        material_class: surface::values::MaterialClass::from_raw((r.material_class).raw()),
        parallax_height: anim(r.parallax_height),
        motion_blur_amount: anim(r.motion_blur_amount),
        diffuse: diffuse_layer,
        layer_blend: surface::values::LayerBlendOp::from_raw(r.layer_blend_mode.raw()),
        specular: SurfaceContribution {
            input: SpecularInputs {
                color: specular_layer,
                gloss: gloss_layer,
            },
            parameters: SpecularResponse {
                exponent: r.specular_exponent,
                hdr_multiplier: r.hdr_specular_multiplier,
                mode: surface::values::SpecularMode::from_raw((r.specular_mode).raw()),
            },
        },
        emissive: SurfaceContribution {
            input: PairedInputs {
                primary: SurfaceContribution {
                    input: emissive_layer1,
                    parameters: surface::values::LayerBlendOp::from_raw(
                        (r.emissive_blend_mode1).raw(),
                    ),
                },
                secondary: SurfaceContribution {
                    input: emissive_layer2,
                    parameters: surface::values::LayerBlendOp::from_raw(
                        (r.emissive_blend_mode2).raw(),
                    ),
                },
            },
            parameters: r.hdr_emissive_multiplier,
        },
        opacity: PairedInputs {
            primary: alpha_layer1,
            secondary: alpha_layer2,
        },
        normals: NormalContributions {
            base: normal_layer,
            primary: MaskedInput {
                input: normal_blend1_layer,
                mask: normal_blend1_mask_layer,
            },
            secondary: MaskedInput {
                input: normal_blend2_layer,
                mask: normal_blend2_mask_layer,
            },
            weights: identity(
                source
                    .resolve::<nr::AnimationValueReal>(r.normal_blend_factors.value().cast())
                    .map_err(read_error)?
                    .iter()
                    .map(|r| r.map(|r| anim(r.value)))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(read_error)?,
            ),
        },
        state: PassState {
            blend: surface::values::BlendMode::from_raw((r.blend_mode).raw()),
            ordering: r.priority,
            rasterization: StandardRasterization {
                additional_flags: surface::values::AdditionalMaterialFlags(
                    (r.additional_flags).bits(),
                ),
                flags: surface::values::MaterialFlags((r.flags).bits()),
                rtt_channels: surface::values::RttChannels(r.rtt_channels.bits()),
                depth_blend_falloff: r.depth_blend_falloff,
                alpha_test_threshold: r.alpha_test_threshold,
            },
        },
    })
}

pub(super) fn get_standard<'a, E: Envelope>(
    source: &'a native::Container<'_, E>,
    reader: &mut Reader<'a, '_>,
    reference: native::Ref<nr::targets::StandardMaterial>,
    index: usize,
) -> Result<surface::StandardMaterialStep, Error> {
    macro_rules! read {
        ($v:ty) => {{
            let r = source
                .resolve::<nr::StandardMaterial<$v>>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "material index out of range"))?
                .map_err(read_error)?;
            standard(r, source, reader)
        }};
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::V0),
        1 => read!(nr::V1),
        2 => read!(nr::V2),
        3 => read!(nr::V3),
        4 => read!(nr::V4),
        5 => read!(nr::V5),
        6 => read!(nr::V6),
        7 => read!(nr::V7),
        8 => read!(nr::V8),
        9 => read!(nr::V9),
        10 => read!(nr::V10),
        11 => read!(nr::V11),
        12 => read!(nr::V12),
        13 => read!(nr::V13),
        14 => read!(nr::V14),
        15 => read!(nr::V15),
        16 => read!(nr::V16),
        17 => read!(nr::V17),
        18 => read!(nr::V18),
        19 => read!(nr::V19),
        20 => read!(nr::V20),
        _ => Err(Error::new("m3.material", "unsupported standard revision")),
    }
}
