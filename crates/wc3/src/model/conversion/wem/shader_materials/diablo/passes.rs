//! Diablo shader inputs retain external dependencies and independent native state.
use crate::model::conversion::Error;
use crate::model::semantic::profiles::diablo::d3::{
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
        cull_mode: CullMode(value.dw_cull_mode),
        z_write_enable: match value.dw_z_write_enable {
            0 => Switch::Disabled,
            1 => Switch::Enabled,
            value => Switch::Unknown(value),
        },
        z_func: Comparison(value.dw_z_func),
        depth_bias: value.fl_depth_bias,
        unclassified_control: value.fl_unknown10,
        stencil_enable: match value.dw_stencil_enable {
            0 => Switch::Disabled,
            1 => Switch::Enabled,
            value => Switch::Unknown(value),
        },
        stencil_func: Comparison(value.dw_stencil_func),
        stencil_ref: value.dw_stencil_ref,
        stencil_pass: StencilOperation(value.dw_stencil_pass),
        stencil_fail: StencilOperation(value.dw_stencil_fail),
        stencil_z_fail: StencilOperation(value.dw_stencil_z_fail),
        alpha_test_enable: match value.dw_alpha_test_enable {
            0 => Switch::Disabled,
            1 => Switch::Enabled,
            value => Switch::Unknown(value),
        },
        alpha_func: Comparison(value.dw_alpha_func),
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
        fill_mode: FillMode(value.dw_fill_mode),
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
        blend_op: BlendOperation(value.dw_blend_op),
        src_blend: BlendFactor(value.dw_src_blend),
        dest_blend: BlendFactor(value.dw_dest_blend),
        constant_color: value.dw_constant_color,
    })
}
pub(super) fn write_render_params(
    _document: &mut RawDocument<'_>,
    value: &RenderParams,
) -> Result<D3RenderParams, Error> {
    Ok(D3RenderParams {
        dw_cull_mode: value.cull_mode.0,
        dw_z_write_enable: match value.z_write_enable {
            Switch::Unknown(value) => value,
            Switch::Disabled => 0,
            Switch::Enabled => 1,
        },
        dw_z_func: value.z_func.0,
        fl_depth_bias: value.depth_bias,
        fl_unknown10: value.unclassified_control,
        dw_stencil_enable: match value.stencil_enable {
            Switch::Unknown(value) => value,
            Switch::Disabled => 0,
            Switch::Enabled => 1,
        },
        dw_stencil_func: value.stencil_func.0,
        dw_stencil_ref: value.stencil_ref,
        dw_stencil_pass: value.stencil_pass.0,
        dw_stencil_fail: value.stencil_fail.0,
        dw_stencil_z_fail: value.stencil_z_fail.0,
        dw_alpha_test_enable: match value.alpha_test_enable {
            Switch::Unknown(value) => value,
            Switch::Disabled => 0,
            Switch::Enabled => 1,
        },
        dw_alpha_func: value.alpha_func.0,
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
        dw_fill_mode: value.fill_mode.0,
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
        dw_blend_op: value.blend_op.0,
        dw_src_blend: value.src_blend.0,
        dw_dest_blend: value.dest_blend.0,
        dw_constant_color: value.constant_color,
    })
}
pub(super) fn read_texture_stage(
    _document: &RawDocument<'_>,
    value: &D3TextureStage,
) -> Result<TextureStage, Error> {
    Ok(TextureStage {
        content_stage: value.content_stage,
        address_u: AddressMode(value.dw_address_u),
        address_v: AddressMode(value.dw_address_v),
        address_w: AddressMode(value.dw_address_w),
        filter: TextureFilter(value.dw_filter),
        mip_map_lod_bias: value.fl_mip_map_lod_bias,
    })
}
pub(super) fn write_texture_stage(
    _document: &mut RawDocument<'_>,
    value: &TextureStage,
) -> Result<D3TextureStage, Error> {
    Ok(D3TextureStage {
        content_stage: value.content_stage,
        dw_address_u: value.address_u.0,
        dw_address_v: value.address_v.0,
        dw_address_w: value.address_w.0,
        dw_filter: value.filter.0,
        fl_mip_map_lod_bias: value.mip_map_lod_bias,
    })
}
pub(super) fn read_shader_tag_value(
    _document: &RawDocument<'_>,
    value: &D3ShaderTagValue,
) -> Result<ShaderTagValue, Error> {
    Ok(ShaderTagValue {
        value_type: ParameterType(value.value_type),
        tag_id: value.tag_id,
        value: value.value,
    })
}
pub(super) fn write_shader_tag_value(
    _document: &mut RawDocument<'_>,
    value: &ShaderTagValue,
) -> Result<D3ShaderTagValue, Error> {
    Ok(D3ShaderTagValue {
        value_type: value.value_type.0,
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
