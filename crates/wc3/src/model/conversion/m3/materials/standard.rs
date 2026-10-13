//! Standard mappings for native M3 materials.
use super::{
    anim, append, append_string, get_layer, native, native_anim, native_layer_ref, nr, read_error,
    revision, string, text, wem, wr, Envelope, Error, Slot,
};
pub(in crate::model::conversion::m3) fn standard<E: Envelope, D: nr::StandardMaterialDialect>(
    r: nr::StandardMaterial<D>,
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
) -> Result<wr::M3Standard, Error>
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
    Ok(wr::M3Standard {
        name: text(doc, &string(source, r.name).map_err(read_error)?)?,
        additional_flags: wr::M3MaterialAdditionalFlag(r.additional_flags),
        flags: wr::M3MaterialFlag(r.flags),
        blend_mode: wr::M3BlendMode(r.blend_mode),
        priority: r.priority,
        rtt_channels: r.rtt_channels,
        specular_exponent: r.specular_exponent,
        depth_blend_falloff: r.depth_blend_falloff,
        alpha_test_threshold: r.alpha_test_threshold,
        hdr_specular_multiplier: r.hdr_specular_multiplier,
        hdr_emissive_multiplier: r.hdr_emissive_multiplier,
        hdr_environment_constant: r.hdr_environment_constant.value(),
        hdr_environment_diffuse: r.hdr_environment_diffuse.value(),
        hdr_environment_specular: r.hdr_environment_specular.value(),
        diffuse_layer: get_layer(source, doc, r.diffuse_layer)?,
        decal_layer: get_layer(source, doc, r.decal_layer)?,
        specular_layer: get_layer(source, doc, r.specular_layer)?,
        gloss_layer: get_layer(source, doc, r.gloss_layer.value())?,
        emissive_layer1: get_layer(source, doc, r.emissive_layer1)?,
        emissive_layer2: get_layer(source, doc, r.emissive_layer2)?,
        environment_layer: get_layer(source, doc, r.environment_layer)?,
        environment_mask_layer: get_layer(source, doc, r.environment_mask_layer)?,
        alpha_layer1: get_layer(source, doc, r.alpha_layer1)?,
        alpha_layer2: get_layer(source, doc, r.alpha_layer2)?,
        normal_layer: get_layer(source, doc, r.normal_layer)?,
        height_layer: get_layer(source, doc, r.height_layer)?,
        light_map_layer: get_layer(source, doc, r.light_map_layer)?,
        ambient_occlusion_layer: get_layer(source, doc, r.ambient_occlusion_layer)?,
        normal_blend1_mask_layer: get_layer(source, doc, r.normal_blend1_mask_layer.value())?,
        normal_blend2_mask_layer: get_layer(source, doc, r.normal_blend2_mask_layer.value())?,
        normal_blend1_layer: get_layer(source, doc, r.normal_blend1_layer.value())?,
        normal_blend2_layer: get_layer(source, doc, r.normal_blend2_layer.value())?,
        material_class: wr::M3MaterialClass(r.material_class),
        layer_blend_mode: wr::M3LayerBlendOp(r.layer_blend_mode),
        emissive_blend_mode1: wr::M3LayerBlendOp(r.emissive_blend_mode1),
        emissive_blend_mode2: wr::M3LayerBlendOp(r.emissive_blend_mode2),
        specular_mode: wr::M3SpecularMode(r.specular_mode),
        parallax_height: anim(r.parallax_height),
        motion_blur_amount: anim(r.motion_blur_amount),
        normal_blend_factors: wem::InlineList(
            source
                .resolve::<nr::AnimationValueReal>(r.normal_blend_factors.value().cast())
                .map_err(read_error)?
                .iter()
                .map(|r| r.map(|r| anim(r.value)))
                .collect::<Result<Vec<_>, _>>()
                .map_err(read_error)?,
        ),
    })
}

pub(in crate::model::conversion::m3) fn get_standard<E: Envelope>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    reference: native::Ref<nr::targets::StandardMaterial>,
    index: usize,
) -> Result<wr::M3Standard, Error> {
    macro_rules! read {
        ($v:ty) => {{
            let r = source
                .resolve::<nr::StandardMaterial<$v>>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "material index out of range"))?
                .map_err(read_error)?;
            standard(r, source, doc)
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

pub(in crate::model::conversion::m3) fn export_standard<E: Envelope>(
    r: wr::M3Standard,
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
) -> Result<nr::StandardMaterial<nr::V20>, Error> {
    Ok(nr::StandardMaterial {
        name: append_string(doc, source.string(r.name)?.unwrap_or_default())?,
        additional_flags: r.additional_flags.0,
        flags: r.flags.0,
        blend_mode: r.blend_mode.0,
        priority: r.priority,
        rtt_channels: r.rtt_channels,
        specular_exponent: r.specular_exponent,
        depth_blend_falloff: r.depth_blend_falloff,
        alpha_test_threshold: r.alpha_test_threshold,
        hdr_specular_multiplier: r.hdr_specular_multiplier,
        hdr_emissive_multiplier: r.hdr_emissive_multiplier,
        hdr_environment_constant: r.hdr_environment_constant,
        hdr_environment_diffuse: r.hdr_environment_diffuse,
        hdr_environment_specular: r.hdr_environment_specular,
        diffuse_layer: native_layer_ref(r.diffuse_layer, source, doc)?,
        decal_layer: native_layer_ref(r.decal_layer, source, doc)?,
        specular_layer: native_layer_ref(r.specular_layer, source, doc)?,
        gloss_layer: native_layer_ref(r.gloss_layer, source, doc)?,
        emissive_layer1: native_layer_ref(r.emissive_layer1, source, doc)?,
        emissive_layer2: native_layer_ref(r.emissive_layer2, source, doc)?,
        environment_layer: native_layer_ref(r.environment_layer, source, doc)?,
        environment_mask_layer: native_layer_ref(r.environment_mask_layer, source, doc)?,
        alpha_layer1: native_layer_ref(r.alpha_layer1, source, doc)?,
        alpha_layer2: native_layer_ref(r.alpha_layer2, source, doc)?,
        normal_layer: native_layer_ref(r.normal_layer, source, doc)?,
        height_layer: native_layer_ref(r.height_layer, source, doc)?,
        light_map_layer: native_layer_ref(r.light_map_layer, source, doc)?,
        ambient_occlusion_layer: native_layer_ref(r.ambient_occlusion_layer, source, doc)?,
        normal_blend1_mask_layer: native_layer_ref(r.normal_blend1_mask_layer, source, doc)?,
        normal_blend2_mask_layer: native_layer_ref(r.normal_blend2_mask_layer, source, doc)?,
        normal_blend1_layer: native_layer_ref(r.normal_blend1_layer, source, doc)?,
        normal_blend2_layer: native_layer_ref(r.normal_blend2_layer, source, doc)?,
        material_class: r.material_class.0,
        layer_blend_mode: r.layer_blend_mode.0,
        emissive_blend_mode1: r.emissive_blend_mode1.0,
        emissive_blend_mode2: r.emissive_blend_mode2.0,
        specular_mode: r.specular_mode.0,
        parallax_height: native_anim(r.parallax_height),
        motion_blur_amount: native_anim(r.motion_blur_amount),
        normal_blend_factors: append(
            doc,
            &r.normal_blend_factors
                .0
                .into_iter()
                .map(|r| nr::AnimationValueReal {
                    value: native_anim(r),
                })
                .collect::<Vec<_>>(),
        )?
        .cast(),
    })
}
