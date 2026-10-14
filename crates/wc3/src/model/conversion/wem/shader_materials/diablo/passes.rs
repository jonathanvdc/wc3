//! Diablo shader inputs retain external dependencies and independent native state.
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::materials::diablo::{
    AddressMode, BlendFactor, BlendOperation, Comparison, CullMode, FillMode, ParameterType,
    PassFlags, RenderParams, RenderState, ShaderTagValue, StencilOperation, Switch, TextureFilter,
    TextureStage,
};
use crate::model::wem::records::{
    D3PassFlags, D3RenderParams, D3RenderState, D3ShaderTagValue, D3TextureStage,
};

use crate::model::conversion::wem::text;

use crate::model::wem::{InlineList, RawDocument};
pub(super) fn read_render_params(
    _document: &RawDocument<'_>,
    value: &D3RenderParams,
) -> Result<RenderParams, Error> {
    Ok(RenderParams {
        cull_mode: CullMode::Unknown(value.dw_cull_mode),
        z_write_enable: match value.dw_z_write_enable {
            0 => Switch::Disabled,
            1 => Switch::Enabled,
            value => Switch::Unknown(value),
        },
        z_func: Comparison::Unknown(value.dw_z_func),
        depth_bias: value.fl_depth_bias,
        unclassified_control: value.fl_unknown10,
        stencil_enable: match value.dw_stencil_enable {
            0 => Switch::Disabled,
            1 => Switch::Enabled,
            value => Switch::Unknown(value),
        },
        stencil_func: Comparison::Unknown(value.dw_stencil_func),
        stencil_ref: value.dw_stencil_ref,
        stencil_pass: StencilOperation::Unknown(value.dw_stencil_pass),
        stencil_fail: StencilOperation::Unknown(value.dw_stencil_fail),
        stencil_z_fail: StencilOperation::Unknown(value.dw_stencil_z_fail),
        alpha_test_enable: match value.dw_alpha_test_enable {
            0 => Switch::Disabled,
            1 => Switch::Enabled,
            value => Switch::Unknown(value),
        },
        alpha_func: Comparison::Unknown(value.dw_alpha_func),
        alpha_ref: value.b_alpha_ref,
        alpha_to_coverage: match value.dw_alpha_to_coverage {
            0 => Switch::Disabled,
            1 => Switch::Enabled,
            value => Switch::Unknown(value),
        },
        fog_enable: match value.dw_fog_enable {
            0 => Switch::Disabled,
            1 => Switch::Enabled,
            value => Switch::Unknown(value),
        },
        fill_mode: FillMode::Unknown(value.dw_fill_mode),
        color_write_enable: match value.dw_color_write_enable {
            0 => Switch::Disabled,
            1 => Switch::Enabled,
            value => Switch::Unknown(value),
        },
        alpha_write_enable: match value.dw_alpha_write_enable {
            0 => Switch::Disabled,
            1 => Switch::Enabled,
            value => Switch::Unknown(value),
        },
        alpha_blend_enable: match value.dw_alpha_blend_enable {
            0 => Switch::Disabled,
            1 => Switch::Enabled,
            value => Switch::Unknown(value),
        },
        blend_op: BlendOperation::Unknown(value.dw_blend_op),
        src_blend: BlendFactor::Unknown(value.dw_src_blend),
        dest_blend: BlendFactor::Unknown(value.dw_dest_blend),
        constant_color: value.dw_constant_color,
    })
}
pub(super) fn write_render_params(
    _document: &mut RawDocument<'_>,
    value: &RenderParams,
) -> Result<D3RenderParams, Error> {
    Ok(D3RenderParams {
        dw_cull_mode: match value.cull_mode {
            CullMode::Unknown(value) => value,
        },
        dw_z_write_enable: match value.z_write_enable {
            Switch::Unknown(value) => value,
            Switch::Disabled => 0,
            Switch::Enabled => 1,
        },
        dw_z_func: match value.z_func {
            Comparison::Unknown(value) => value,
        },
        fl_depth_bias: value.depth_bias,
        fl_unknown10: value.unclassified_control,
        dw_stencil_enable: match value.stencil_enable {
            Switch::Unknown(value) => value,
            Switch::Disabled => 0,
            Switch::Enabled => 1,
        },
        dw_stencil_func: match value.stencil_func {
            Comparison::Unknown(value) => value,
        },
        dw_stencil_ref: value.stencil_ref,
        dw_stencil_pass: match value.stencil_pass {
            StencilOperation::Unknown(value) => value,
        },
        dw_stencil_fail: match value.stencil_fail {
            StencilOperation::Unknown(value) => value,
        },
        dw_stencil_z_fail: match value.stencil_z_fail {
            StencilOperation::Unknown(value) => value,
        },
        dw_alpha_test_enable: match value.alpha_test_enable {
            Switch::Unknown(value) => value,
            Switch::Disabled => 0,
            Switch::Enabled => 1,
        },
        dw_alpha_func: match value.alpha_func {
            Comparison::Unknown(value) => value,
        },
        b_alpha_ref: value.alpha_ref,
        dw_alpha_to_coverage: match value.alpha_to_coverage {
            Switch::Unknown(value) => value,
            Switch::Disabled => 0,
            Switch::Enabled => 1,
        },
        dw_fog_enable: match value.fog_enable {
            Switch::Unknown(value) => value,
            Switch::Disabled => 0,
            Switch::Enabled => 1,
        },
        dw_fill_mode: match value.fill_mode {
            FillMode::Unknown(value) => value,
        },
        dw_color_write_enable: match value.color_write_enable {
            Switch::Unknown(value) => value,
            Switch::Disabled => 0,
            Switch::Enabled => 1,
        },
        dw_alpha_write_enable: match value.alpha_write_enable {
            Switch::Unknown(value) => value,
            Switch::Disabled => 0,
            Switch::Enabled => 1,
        },
        dw_alpha_blend_enable: match value.alpha_blend_enable {
            Switch::Unknown(value) => value,
            Switch::Disabled => 0,
            Switch::Enabled => 1,
        },
        dw_blend_op: match value.blend_op {
            BlendOperation::Unknown(value) => value,
        },
        dw_src_blend: match value.src_blend {
            BlendFactor::Unknown(value) => value,
        },
        dw_dest_blend: match value.dest_blend {
            BlendFactor::Unknown(value) => value,
        },
        dw_constant_color: value.constant_color,
    })
}
pub(super) fn read_texture_stage(
    _document: &RawDocument<'_>,
    value: &D3TextureStage,
) -> Result<TextureStage, Error> {
    Ok(TextureStage {
        content_stage: value.content_stage,
        address_u: AddressMode::Unknown(value.dw_address_u),
        address_v: AddressMode::Unknown(value.dw_address_v),
        address_w: AddressMode::Unknown(value.dw_address_w),
        filter: TextureFilter::Unknown(value.dw_filter),
        mip_map_lod_bias: value.fl_mip_map_lod_bias,
    })
}
pub(super) fn write_texture_stage(
    _document: &mut RawDocument<'_>,
    value: &TextureStage,
) -> Result<D3TextureStage, Error> {
    Ok(D3TextureStage {
        content_stage: value.content_stage,
        dw_address_u: match value.address_u {
            AddressMode::Unknown(value) => value,
        },
        dw_address_v: match value.address_v {
            AddressMode::Unknown(value) => value,
        },
        dw_address_w: match value.address_w {
            AddressMode::Unknown(value) => value,
        },
        dw_filter: match value.filter {
            TextureFilter::Unknown(value) => value,
        },
        fl_mip_map_lod_bias: value.mip_map_lod_bias,
    })
}
pub(super) fn read_shader_tag_value(
    _document: &RawDocument<'_>,
    value: &D3ShaderTagValue,
) -> Result<ShaderTagValue, Error> {
    Ok(ShaderTagValue {
        value_type: ParameterType::Unknown(value.value_type),
        tag_id: value.tag_id,
        value: value.value,
    })
}
pub(super) fn write_shader_tag_value(
    _document: &mut RawDocument<'_>,
    value: &ShaderTagValue,
) -> Result<D3ShaderTagValue, Error> {
    Ok(D3ShaderTagValue {
        value_type: match value.value_type {
            ParameterType::Unknown(value) => value,
        },
        tag_id: value.tag_id,
        value: value.value,
    })
}
pub(super) fn read_render_state<'view>(
    document: &'view RawDocument<'_>,
    value: &D3RenderState,
) -> Result<RenderState<'view>, Error> {
    Ok(RenderState {
        unclassified_control: value.dw_unknown00,
        unclassified_control2: value.dw_unknown04,
        render_params: read_render_params(document, &value.render_params)?,
        unclassified_control3: value.dw_unknown68,
        texture_stage_count: value.texture_stage_count,
        texture_stages: value
            .texture_stages
            .0
            .iter()
            .map(|value| read_texture_stage(document, value))
            .collect::<Result<Vec<_>, Error>>()?,
        pass_flags: PassFlags::from_bits_retain(value.pass_flags.bits()),
        effect_file: text::read(document, value.effect_file)?,
        vertex_shader_entry: text::read(document, value.vertex_shader_entry)?,
        pixel_shader_entry: text::read(document, value.pixel_shader_entry)?,
        shader_params: value
            .shader_params
            .0
            .iter()
            .map(|value| read_shader_tag_value(document, value))
            .collect::<Result<Vec<_>, Error>>()?,
    })
}
pub(super) fn write_render_state(
    document: &mut RawDocument<'_>,
    value: &RenderState<'_>,
) -> Result<D3RenderState, Error> {
    Ok(D3RenderState {
        dw_unknown00: value.unclassified_control,
        dw_unknown04: value.unclassified_control2,
        render_params: write_render_params(document, &value.render_params)?,
        dw_unknown68: value.unclassified_control3,
        texture_stage_count: value.texture_stage_count,
        texture_stages: InlineList(
            value
                .texture_stages
                .iter()
                .map(|value| write_texture_stage(document, value))
                .collect::<Result<Vec<_>, Error>>()?,
        ),
        pass_flags: D3PassFlags::from(value.pass_flags.bits()),
        effect_file: text::write_bytes(document, value.effect_file.as_bytes())?,
        vertex_shader_entry: text::write_bytes(document, value.vertex_shader_entry.as_bytes())?,
        pixel_shader_entry: text::write_bytes(document, value.pixel_shader_entry.as_bytes())?,
        shader_params: InlineList(
            value
                .shader_params
                .iter()
                .map(|value| write_shader_tag_value(document, value))
                .collect::<Result<Vec<_>, Error>>()?,
        ),
    })
}
