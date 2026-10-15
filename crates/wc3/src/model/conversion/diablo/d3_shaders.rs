//! Native shader and shader-map resources use the same pass vocabulary as WEM.
use super::{i32_count, name, text};
use crate::model::conversion::{Conversion, Error, ExportOptions, ImportOptions, Plan};
use crate::model::diablo::d3::records as native;
use crate::model::diablo::d3::{Shader, ShaderMapDocument, V150, V26};
use crate::model::semantic::profiles::diablo::d3::{
    AddressMode, AssetRef, BlendFactor, BlendOperation, Comparison, CullMode, FillMode, Group,
    ParameterType, PassFlags, RenderParams, RenderState, ShaderAsset, ShaderMapAsset,
    ShaderMapEntry, ShaderTagValue, StencilOperation, Switch, TextureFilter, TextureStage,
};
use std::borrow::Cow;
use zerovec::ZeroVec;
fn switch(v: i32) -> Switch {
    match v {
        0 => Switch::Disabled,
        1 => Switch::Enabled,
        v => Switch::Unknown(v),
    }
}
fn native_switch(v: Switch) -> i32 {
    match v {
        Switch::Disabled => 0,
        Switch::Enabled => 1,
        Switch::Unknown(v) => v,
    }
}
fn params(v: &native::RenderParams) -> RenderParams {
    RenderParams {
        cull_mode: CullMode(v.cull_mode),
        z_write_enable: switch(v.z_write_enable),
        z_func: Comparison(v.z_func),
        depth_bias: v.depth_bias,
        unclassified_control: v.unknown10,
        stencil_enable: switch(v.stencil_enable),
        stencil_func: Comparison(v.stencil_func),
        stencil_ref: v.stencil_ref,
        stencil_pass: StencilOperation(v.stencil_pass),
        stencil_fail: StencilOperation(v.stencil_fail),
        stencil_z_fail: StencilOperation(v.stencil_z_fail),
        alpha_test_enable: switch(v.alpha_test_enable),
        alpha_func: Comparison(v.alpha_func),
        alpha_ref: v.b_alpha_ref,
        alpha_to_coverage: switch(v.alpha_to_coverage),
        fog_enable: switch(v.fog_enable),
        fill_mode: FillMode(v.fill_mode),
        color_write_enable: switch(v.color_write_enable),
        alpha_write_enable: switch(v.alpha_write_enable),
        alpha_blend_enable: switch(v.alpha_blend_enable),
        blend_op: BlendOperation(v.blend_op),
        src_blend: BlendFactor(v.src_blend),
        dest_blend: BlendFactor(v.dest_blend),
        constant_color: v.constant_color,
    }
}
fn native_params(v: &RenderParams) -> native::RenderParams {
    native::RenderParams {
        cull_mode: v.cull_mode.0,
        z_write_enable: native_switch(v.z_write_enable),
        z_func: v.z_func.0,
        depth_bias: v.depth_bias,
        unknown10: v.unclassified_control,
        stencil_enable: native_switch(v.stencil_enable),
        stencil_func: v.stencil_func.0,
        stencil_ref: v.stencil_ref,
        stencil_pass: v.stencil_pass.0,
        stencil_fail: v.stencil_fail.0,
        stencil_z_fail: v.stencil_z_fail.0,
        alpha_test_enable: native_switch(v.alpha_test_enable),
        alpha_func: v.alpha_func.0,
        b_alpha_ref: v.alpha_ref,
        alpha_to_coverage: native_switch(v.alpha_to_coverage),
        fog_enable: native_switch(v.fog_enable),
        fill_mode: v.fill_mode.0,
        color_write_enable: native_switch(v.color_write_enable),
        alpha_write_enable: native_switch(v.alpha_write_enable),
        alpha_blend_enable: native_switch(v.alpha_blend_enable),
        blend_op: v.blend_op.0,
        src_blend: v.src_blend.0,
        dest_blend: v.dest_blend.0,
        constant_color: v.constant_color,
        ..Default::default()
    }
}
fn pass<'a>(v: &'a native::RenderPass<'_>) -> Result<RenderState<'a>, Error> {
    Ok(RenderState {
        unclassified_control: v.unknown00,
        unclassified_control2: v.unknown04,
        render_params: params(&v.render_params),
        unclassified_control3: v.unknown68,
        texture_stage_count: v.texture_stage_count,
        texture_stages: v
            .texture_stages
            .iter()
            .map(|s| TextureStage {
                content_stage: s.texture_type,
                address_u: AddressMode(s.address_u),
                address_v: AddressMode(s.address_v),
                address_w: AddressMode(s.address_w),
                filter: TextureFilter(s.filter),
                mip_map_lod_bias: s.mip_map_lod_bias,
            })
            .collect(),
        pass_flags: PassFlags::from_bits_retain(v.pass_flags),
        effect_file: Cow::Borrowed(text(&v.effect_file)?),
        vertex_shader_entry: Cow::Borrowed(text(&v.vertex_shader_entry)?),
        pixel_shader_entry: Cow::Borrowed(text(&v.pixel_shader_entry)?),
        shader_params: v
            .shader_params
            .iter()
            .map(|t| ShaderTagValue {
                value_type: ParameterType(t.value_type),
                tag_id: t.tag_id,
                value: t.value,
            })
            .collect(),
    })
}
fn native_pass(v: &RenderState<'_>) -> Result<native::RenderPass<'static>, Error> {
    let stages = v
        .texture_stages
        .iter()
        .map(|s| native::TextureStageParams {
            texture_type: s.content_stage,
            address_u: s.address_u.0,
            address_v: s.address_v.0,
            address_w: s.address_w.0,
            filter: s.filter.0,
            mip_map_lod_bias: s.mip_map_lod_bias,
        })
        .collect::<Vec<_>>();
    let tags = v
        .shader_params
        .iter()
        .map(|t| native::ShaderTagMapEntry {
            value_type: t.value_type.0,
            tag_id: t.tag_id,
            value: t.value,
        })
        .collect::<Vec<_>>();
    Ok(native::RenderPass {
        unknown00: v.unclassified_control,
        unknown04: v.unclassified_control2,
        render_params: native_params(&v.render_params),
        unknown68: v.unclassified_control3,
        texture_stage_count: v.texture_stage_count,
        texture_stages: ZeroVec::alloc_from_slice(&stages),
        pass_flags: v.pass_flags.bits(),
        effect_file: name(&v.effect_file)?,
        vertex_shader_entry: name(&v.vertex_shader_entry)?,
        pixel_shader_entry: name(&v.pixel_shader_entry)?,
        shader_params: ZeroVec::alloc_from_slice(&tags),
        ..Default::default()
    })
}
/// Import revision 150 shader declarations, preserving pass order and external names.
pub fn import_shader<'a>(
    source: &'a Shader<'_, V150>,
    _options: &ImportOptions,
) -> Result<Conversion<ShaderAsset<'a>>, Error> {
    let r = &source.root;
    let value = ShaderAsset {
        sno_id: r.sno_id,
        flags: r.shader_flags,
        unclassified_control: r.unknown10,
        name: Cow::Borrowed(text(&r.name)?),
        passes: r
            .render_passes
            .iter()
            .map(pass)
            .collect::<Result<Vec<_>, Error>>()?,
    };
    value.validate()?;
    Ok(Conversion {
        value,
        report: Default::default(),
    })
}
/// Plan revision 150 shader output from independently authored pass declarations.
pub fn plan_shader(
    value: &ShaderAsset<'_>,
    _options: &ExportOptions,
) -> Result<Plan<Shader<'static, V150>>, Error> {
    value.validate()?;
    let passes = value
        .passes
        .iter()
        .map(native_pass)
        .collect::<Result<Vec<_>, Error>>()?;
    let root = native::Shaders {
        sno_id: value.sno_id,
        shader_flags: value.flags,
        unknown10: value.unclassified_control,
        name: name(&value.name)?,
        render_pass_count: i32_count(passes.len())?,
        render_passes: passes,
        ..Default::default()
    };
    Ok(Plan::new(Shader::<V150>::new(root), Default::default()))
}
/// Import revision 26 shader-map tags with explicit external shader references.
pub fn import_shader_map(
    source: &ShaderMapDocument<'_, V26>,
    _options: &ImportOptions,
) -> Result<Conversion<ShaderMapAsset>, Error> {
    let r = &source.root;
    Ok(Conversion {
        value: ShaderMapAsset {
            sno_id: r.sno_id,
            flags: r.shader_map_flags,
            entries: r
                .shaders
                .iter()
                .map(|v| ShaderMapEntry {
                    value_type: ParameterType(v.value_type),
                    tag_id: v.tag_id,
                    shader: AssetRef {
                        id: v.shader,
                        group: Group::Shaders,
                    },
                })
                .collect(),
        },
        report: Default::default(),
    })
}
/// Plan revision 26 shader-map output, checking dependency groups explicitly.
pub fn plan_shader_map(
    value: &ShaderMapAsset,
    _options: &ExportOptions,
) -> Result<Plan<ShaderMapDocument<'static, V26>>, Error> {
    let entries = value
        .entries
        .iter()
        .map(|v| {
            if v.shader.group != Group::Shaders {
                return Err(Error::new(
                    "d3.shader_map.shader",
                    "dependency belongs to a different SNO group",
                ));
            }
            Ok(native::ShaderMapEntry {
                value_type: v.value_type.0,
                tag_id: v.tag_id,
                shader: v.shader.id,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(Plan::new(
        ShaderMapDocument::<V26>::new(native::ShaderMap {
            sno_id: value.sno_id,
            shader_map_flags: value.flags,
            shaders: ZeroVec::alloc_from_slice(&entries),
            ..Default::default()
        }),
        Default::default(),
    ))
}
