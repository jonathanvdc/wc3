//! Named shading choices map without numerical discriminator tables.
use crate::model::semantic::profiles::wem::materials;
use crate::model::wem::records;
pub(super) fn read_blend_mode(value: records::BlendMode) -> materials::BlendMode {
    match value {
        records::BlendMode::Opaque => materials::BlendMode::Opaque,
        records::BlendMode::AlphaKey => materials::BlendMode::AlphaKey,
        records::BlendMode::AlphaBlend => materials::BlendMode::AlphaBlend,
        records::BlendMode::Additive => materials::BlendMode::Additive,
        records::BlendMode::AdditiveAlpha => materials::BlendMode::AdditiveAlpha,
        records::BlendMode::Modulate => materials::BlendMode::Modulate,
        records::BlendMode::Modulate2x => materials::BlendMode::Modulate2x,
        records::BlendMode::BlendAdd => materials::BlendMode::BlendAdd,
        records::BlendMode::Transparent => materials::BlendMode::Transparent,
        records::BlendMode::PremultipliedAlpha => materials::BlendMode::PremultipliedAlpha,
        records::BlendMode::Unknown(code) => materials::BlendMode::Unknown(code),
    }
}
pub(super) fn write_blend_mode(value: materials::BlendMode) -> records::BlendMode {
    match value {
        materials::BlendMode::Opaque => records::BlendMode::Opaque,
        materials::BlendMode::AlphaKey => records::BlendMode::AlphaKey,
        materials::BlendMode::AlphaBlend => records::BlendMode::AlphaBlend,
        materials::BlendMode::Additive => records::BlendMode::Additive,
        materials::BlendMode::AdditiveAlpha => records::BlendMode::AdditiveAlpha,
        materials::BlendMode::Modulate => records::BlendMode::Modulate,
        materials::BlendMode::Modulate2x => records::BlendMode::Modulate2x,
        materials::BlendMode::BlendAdd => records::BlendMode::BlendAdd,
        materials::BlendMode::Transparent => records::BlendMode::Transparent,
        materials::BlendMode::PremultipliedAlpha => records::BlendMode::PremultipliedAlpha,
        materials::BlendMode::Unknown(code) => records::BlendMode::Unknown(code),
    }
}
pub(super) fn read_cull_mode(value: records::CullMode) -> materials::CullMode {
    match value {
        records::CullMode::Back => materials::CullMode::Back,
        records::CullMode::Front => materials::CullMode::Front,
        records::CullMode::None => materials::CullMode::None,
        records::CullMode::Unknown(code) => materials::CullMode::Unknown(code),
    }
}
pub(super) fn write_cull_mode(value: materials::CullMode) -> records::CullMode {
    match value {
        materials::CullMode::Back => records::CullMode::Back,
        materials::CullMode::Front => records::CullMode::Front,
        materials::CullMode::None => records::CullMode::None,
        materials::CullMode::Unknown(code) => records::CullMode::Unknown(code),
    }
}
pub(super) fn read_uv_mapping_mode(value: records::UVMappingMode) -> materials::UVMappingMode {
    match value {
        records::UVMappingMode::ExplicitUV => materials::UVMappingMode::ExplicitUV,
        records::UVMappingMode::EnvSphere => materials::UVMappingMode::EnvSphere,
        records::UVMappingMode::EnvCube => materials::UVMappingMode::EnvCube,
        records::UVMappingMode::Unknown(code) => materials::UVMappingMode::Unknown(code),
    }
}
pub(super) fn write_uv_mapping_mode(value: materials::UVMappingMode) -> records::UVMappingMode {
    match value {
        materials::UVMappingMode::ExplicitUV => records::UVMappingMode::ExplicitUV,
        materials::UVMappingMode::EnvSphere => records::UVMappingMode::EnvSphere,
        materials::UVMappingMode::EnvCube => records::UVMappingMode::EnvCube,
        materials::UVMappingMode::Unknown(code) => records::UVMappingMode::Unknown(code),
    }
}
pub(super) fn read_legacy_slot(value: records::LegacySlot) -> materials::LegacySlot {
    match value {
        records::LegacySlot::Diffuse => materials::LegacySlot::Diffuse,
        records::LegacySlot::Normal => materials::LegacySlot::Normal,
        records::LegacySlot::Specular => materials::LegacySlot::Specular,
        records::LegacySlot::Gloss => materials::LegacySlot::Gloss,
        records::LegacySlot::Emissive => materials::LegacySlot::Emissive,
        records::LegacySlot::Environment => materials::LegacySlot::Environment,
        records::LegacySlot::AmbientOcclusion => materials::LegacySlot::AmbientOcclusion,
        records::LegacySlot::Height => materials::LegacySlot::Height,
        records::LegacySlot::Lightmap => materials::LegacySlot::Lightmap,
        records::LegacySlot::Detail => materials::LegacySlot::Detail,
        records::LegacySlot::Count => materials::LegacySlot::Count,
        records::LegacySlot::Unknown(code) => materials::LegacySlot::Unknown(code),
    }
}
pub(super) fn write_legacy_slot(value: materials::LegacySlot) -> records::LegacySlot {
    match value {
        materials::LegacySlot::Diffuse => records::LegacySlot::Diffuse,
        materials::LegacySlot::Normal => records::LegacySlot::Normal,
        materials::LegacySlot::Specular => records::LegacySlot::Specular,
        materials::LegacySlot::Gloss => records::LegacySlot::Gloss,
        materials::LegacySlot::Emissive => records::LegacySlot::Emissive,
        materials::LegacySlot::Environment => records::LegacySlot::Environment,
        materials::LegacySlot::AmbientOcclusion => records::LegacySlot::AmbientOcclusion,
        materials::LegacySlot::Height => records::LegacySlot::Height,
        materials::LegacySlot::Lightmap => records::LegacySlot::Lightmap,
        materials::LegacySlot::Detail => records::LegacySlot::Detail,
        materials::LegacySlot::Count => records::LegacySlot::Count,
        materials::LegacySlot::Unknown(code) => records::LegacySlot::Unknown(code),
    }
}
pub(super) fn read_surface_channel(value: records::SurfaceChannel) -> materials::SurfaceChannel {
    match value {
        records::SurfaceChannel::Color => materials::SurfaceChannel::Color,
        records::SurfaceChannel::Emissive => materials::SurfaceChannel::Emissive,
        records::SurfaceChannel::Specular => materials::SurfaceChannel::Specular,
        records::SurfaceChannel::Normal => materials::SurfaceChannel::Normal,
        records::SurfaceChannel::AmbientOcclusion => materials::SurfaceChannel::AmbientOcclusion,
        records::SurfaceChannel::Environment => materials::SurfaceChannel::Environment,
        records::SurfaceChannel::Coverage => materials::SurfaceChannel::Coverage,
        records::SurfaceChannel::Gloss => materials::SurfaceChannel::Gloss,
        records::SurfaceChannel::Count => materials::SurfaceChannel::Count,
        records::SurfaceChannel::Unknown(code) => materials::SurfaceChannel::Unknown(code),
    }
}
pub(super) fn write_surface_channel(value: materials::SurfaceChannel) -> records::SurfaceChannel {
    match value {
        materials::SurfaceChannel::Color => records::SurfaceChannel::Color,
        materials::SurfaceChannel::Emissive => records::SurfaceChannel::Emissive,
        materials::SurfaceChannel::Specular => records::SurfaceChannel::Specular,
        materials::SurfaceChannel::Normal => records::SurfaceChannel::Normal,
        materials::SurfaceChannel::AmbientOcclusion => records::SurfaceChannel::AmbientOcclusion,
        materials::SurfaceChannel::Environment => records::SurfaceChannel::Environment,
        materials::SurfaceChannel::Coverage => records::SurfaceChannel::Coverage,
        materials::SurfaceChannel::Gloss => records::SurfaceChannel::Gloss,
        materials::SurfaceChannel::Count => records::SurfaceChannel::Count,
        materials::SurfaceChannel::Unknown(code) => records::SurfaceChannel::Unknown(code),
    }
}
pub(super) fn read_composite_op(value: records::CompositeOp) -> materials::CompositeOp {
    match value {
        records::CompositeOp::Set => materials::CompositeOp::Set,
        records::CompositeOp::AlphaKey => materials::CompositeOp::AlphaKey,
        records::CompositeOp::AlphaBlend => materials::CompositeOp::AlphaBlend,
        records::CompositeOp::Add => materials::CompositeOp::Add,
        records::CompositeOp::AddAlpha => materials::CompositeOp::AddAlpha,
        records::CompositeOp::Modulate => materials::CompositeOp::Modulate,
        records::CompositeOp::Modulate2x => materials::CompositeOp::Modulate2x,
        records::CompositeOp::Count => materials::CompositeOp::Count,
        records::CompositeOp::Unknown(code) => materials::CompositeOp::Unknown(code),
    }
}
pub(super) fn write_composite_op(value: materials::CompositeOp) -> records::CompositeOp {
    match value {
        materials::CompositeOp::Set => records::CompositeOp::Set,
        materials::CompositeOp::AlphaKey => records::CompositeOp::AlphaKey,
        materials::CompositeOp::AlphaBlend => records::CompositeOp::AlphaBlend,
        materials::CompositeOp::Add => records::CompositeOp::Add,
        materials::CompositeOp::AddAlpha => records::CompositeOp::AddAlpha,
        materials::CompositeOp::Modulate => records::CompositeOp::Modulate,
        materials::CompositeOp::Modulate2x => records::CompositeOp::Modulate2x,
        materials::CompositeOp::Count => records::CompositeOp::Count,
        materials::CompositeOp::Unknown(code) => records::CompositeOp::Unknown(code),
    }
}
pub(super) fn read_combiner_op(value: records::CombinerOp) -> materials::CombinerOp {
    match value {
        records::CombinerOp::Opaque => materials::CombinerOp::Opaque,
        records::CombinerOp::Mod => materials::CombinerOp::Mod,
        records::CombinerOp::Mod2x => materials::CombinerOp::Mod2x,
        records::CombinerOp::Add => materials::CombinerOp::Add,
        records::CombinerOp::Decal => materials::CombinerOp::Decal,
        records::CombinerOp::Fade => materials::CombinerOp::Fade,
        records::CombinerOp::Pass => materials::CombinerOp::Pass,
        records::CombinerOp::AddAlpha => materials::CombinerOp::AddAlpha,
        records::CombinerOp::MaskedMod => materials::CombinerOp::MaskedMod,
        records::CombinerOp::MaskedMod2x => materials::CombinerOp::MaskedMod2x,
        records::CombinerOp::Count => materials::CombinerOp::Count,
        records::CombinerOp::Unknown(code) => materials::CombinerOp::Unknown(code),
    }
}
pub(super) fn write_combiner_op(value: materials::CombinerOp) -> records::CombinerOp {
    match value {
        materials::CombinerOp::Opaque => records::CombinerOp::Opaque,
        materials::CombinerOp::Mod => records::CombinerOp::Mod,
        materials::CombinerOp::Mod2x => records::CombinerOp::Mod2x,
        materials::CombinerOp::Add => records::CombinerOp::Add,
        materials::CombinerOp::Decal => records::CombinerOp::Decal,
        materials::CombinerOp::Fade => records::CombinerOp::Fade,
        materials::CombinerOp::Pass => records::CombinerOp::Pass,
        materials::CombinerOp::AddAlpha => records::CombinerOp::AddAlpha,
        materials::CombinerOp::MaskedMod => records::CombinerOp::MaskedMod,
        materials::CombinerOp::MaskedMod2x => records::CombinerOp::MaskedMod2x,
        materials::CombinerOp::Count => records::CombinerOp::Count,
        materials::CombinerOp::Unknown(code) => records::CombinerOp::Unknown(code),
    }
}
pub(super) fn read_pbr_slot(value: records::PbrSlot) -> materials::PbrSlot {
    match value {
        records::PbrSlot::BaseColor => materials::PbrSlot::BaseColor,
        records::PbrSlot::Normal => materials::PbrSlot::Normal,
        records::PbrSlot::Orm => materials::PbrSlot::Orm,
        records::PbrSlot::Metallic => materials::PbrSlot::Metallic,
        records::PbrSlot::Roughness => materials::PbrSlot::Roughness,
        records::PbrSlot::AmbientOcclusion => materials::PbrSlot::AmbientOcclusion,
        records::PbrSlot::Emissive => materials::PbrSlot::Emissive,
        records::PbrSlot::Environment => materials::PbrSlot::Environment,
        records::PbrSlot::TeamColorMask => materials::PbrSlot::TeamColorMask,
        records::PbrSlot::Count => materials::PbrSlot::Count,
        records::PbrSlot::Unknown(code) => materials::PbrSlot::Unknown(code),
    }
}
pub(super) fn write_pbr_slot(value: materials::PbrSlot) -> records::PbrSlot {
    match value {
        materials::PbrSlot::BaseColor => records::PbrSlot::BaseColor,
        materials::PbrSlot::Normal => records::PbrSlot::Normal,
        materials::PbrSlot::Orm => records::PbrSlot::Orm,
        materials::PbrSlot::Metallic => records::PbrSlot::Metallic,
        materials::PbrSlot::Roughness => records::PbrSlot::Roughness,
        materials::PbrSlot::AmbientOcclusion => records::PbrSlot::AmbientOcclusion,
        materials::PbrSlot::Emissive => records::PbrSlot::Emissive,
        materials::PbrSlot::Environment => records::PbrSlot::Environment,
        materials::PbrSlot::TeamColorMask => records::PbrSlot::TeamColorMask,
        materials::PbrSlot::Count => records::PbrSlot::Count,
        materials::PbrSlot::Unknown(code) => records::PbrSlot::Unknown(code),
    }
}
