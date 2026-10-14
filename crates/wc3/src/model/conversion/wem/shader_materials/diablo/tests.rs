use crate::model::conversion::wem::shader_materials::diablo::{read, write, Encoded};
use crate::model::conversion::wem::text;
use crate::model::semantic::profiles::wem::materials::diablo::Switch;
use crate::model::wem::records::{
    D3AssetRef, D3Group, D3Material, D3PassFlags, D3RenderParams, D3RenderState, D3ShaderTagValue,
    D3TextureEntry, D3TextureStage, D3UberMaterial, D3UvFlags,
};

use crate::model::wem::{records, InlineList, RawDocument};
use std::borrow::Cow;

#[test]
fn absent_and_explicit_empty_variant_options_select_distinct_native_schemas() {
    let source = RawDocument::new();
    let old = D3Material::<records::V1>::default();
    let graph = read(&source, &old).unwrap();
    assert!(graph.variant_flags.is_none());
    let mut output = RawDocument::new();
    let encoded = write(&mut output, &graph, 18).unwrap();
    assert!(matches!(encoded, Encoded::Unflagged(_)));
    let reference = encoded.append(&mut output).unwrap();
    assert_eq!(
        output
            .referenced_section(reference)
            .unwrap()
            .unwrap()
            .revision(),
        1
    );
    let authored = D3Material::<records::V2>::default();
    let graph = read(&source, &authored).unwrap();
    assert_eq!(graph.variant_flags.unwrap().bits(), 0);
    let encoded = write(&mut output, &graph, 19).unwrap();
    assert!(matches!(encoded, Encoded::Flagged(_)));
    let reference = encoded.append(&mut output).unwrap();
    assert_eq!(
        output
            .referenced_section(reference)
            .unwrap()
            .unwrap()
            .revision(),
        2
    );
}

#[test]
fn native_pass_text_borrows_and_signed_options_and_external_dependencies_survive_edits() {
    let mut source = RawDocument::new();
    let effect = text::write_bytes(&mut source, b"shaders/independent.fx").unwrap();
    let pass = D3RenderState {
        texture_stage_count: 1,
        texture_stages: InlineList(vec![D3TextureStage {
            dw_address_u: -7,
            dw_filter: 93,
            fl_mip_map_lod_bias: 0.5,
            ..Default::default()
        }]),
        pass_flags: D3PassFlags::from_bits_retain(i32::MIN),
        effect_file: effect,
        render_params: D3RenderParams {
            dw_cull_mode: -3,
            dw_z_write_enable: 17,
            fl_depth_bias: -0.25,
            ..Default::default()
        },
        shader_params: InlineList(vec![D3ShaderTagValue {
            value_type: -13,
            tag_id: 0xffff_ffff,
            value: 0x8000_0001,
        }]),
        ..Default::default()
    };
    let original = D3Material::<records::V2> {
        source_version: 20,
        variant_flags: i32::MIN.into(),
        base_material: D3AssetRef {
            id: -23,
            group: D3Group::Unknown(-24),
        },
        uber: D3UberMaterial {
            textures: InlineList(vec![D3TextureEntry {
                texture: D3AssetRef {
                    id: 17,
                    group: D3Group::Textures,
                },
                uv_flags: D3UvFlags::from_bits_retain(i32::MIN),
                ..Default::default()
            }]),
            ..Default::default()
        },
        opaque_passes: InlineList(vec![pass]),
        ..Default::default()
    };
    let mut graph = read(&source, &original).unwrap();
    graph.validate().unwrap();
    assert!(matches!(
        graph.opaque_passes[0].effect_file,
        Cow::Borrowed("shaders/independent.fx")
    ));
    assert_eq!(
        graph.opaque_passes[0].render_params.z_write_enable,
        Switch::Unknown(17)
    );
    graph.opaque_passes[0].render_params.depth_bias = -0.75;
    let mut destination = RawDocument::new();
    let Encoded::Flagged(output) =
        write(&mut destination, &graph, original.source_version).unwrap()
    else {
        unreachable!()
    };
    assert_eq!(output.variant_flags.bits(), i32::MIN);
    assert_eq!(output.base_material, original.base_material);
    assert_eq!(output.uber, original.uber);
    assert_eq!(output.opaque_passes.0[0].render_params.fl_depth_bias, -0.75);
    assert_eq!(
        output.opaque_passes.0[0].render_params.dw_z_write_enable,
        17
    );
    assert_eq!(
        output.opaque_passes.0[0].shader_params,
        original.opaque_passes.0[0].shader_params
    );
    assert_eq!(output.opaque_passes.0[0].pass_flags.bits(), i32::MIN);
    assert_eq!(
        destination
            .string(output.opaque_passes.0[0].effect_file)
            .unwrap(),
        Some("shaders/independent.fx")
    );
}
