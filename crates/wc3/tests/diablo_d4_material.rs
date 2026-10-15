use wc3::model::conversion::{ExportOptions, ImportOptions, LossPolicy};
use wc3::model::formats::diablo::d4::{self, records as native};
use wc3::model::io::diablo::d4 as convert;

fn source() -> d4::Material<'static> {
    let mut source = d4::Material::default();
    source.root.sno_id = 77;
    source.root.flags = 91;
    source.root.surface = 43;
    source.root.duration = 2.5;
    source.root.gravity_magnitude = -3.;
    source.root.uber_material.shader_map = 124;
    source.root.uber_material.mat.sort_pri = 8;
    source.root.uber_material.mat_tex_list = [1, 2]
        .into_iter()
        .map(|semantic| native::MaterialTextureEntry {
            shader_tex: semantic,
            mat_tex: native::MaterialTexture {
                tex: 17,
                tex_anim_state_index: -1,
                ..Default::default()
            },
            ..Default::default()
        })
        .collect();
    source
        .root
        .uber_material
        .run_time_material_values
        .push(native::RunTimeMaterialValues {
            material_scalar_values: vec![native::RunTimeMaterialScalarValueEntry {
                value: native::RunTimeMaterialValueFloat {
                    material_value: 99,
                    value: 0.75,
                    path_val_graph_index: -1,
                    light_group: 2,
                },
            }],
            ..Default::default()
        });
    source
}
#[test]
fn standalone_materials_share_texture_identity_and_rebuild_native_resources() {
    let source = source();
    let imported = convert::import_material(&source, &ImportOptions::default()).unwrap();
    assert!(imported.report.issues.is_empty());
    assert_eq!(imported.value.resources().len(), 1);
    let (material_id, material) = imported.value.materials().next().unwrap();
    let shader = material.surface.shader.as_ref().unwrap();
    assert_eq!(shader.textures[0].image, shader.textures[1].image);
    assert_eq!(shader.asset.as_ref().unwrap().sno_id, 77);
    let output = convert::plan_material(&imported.value, material_id, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.root.sno_id, 77);
    assert_eq!(output.root.duration, 2.5);
    assert_eq!(output.root.gravity_magnitude, -3.);
    assert_eq!(output.root.uber_material.mat.sort_pri, 8);
    assert_eq!(output.root.uber_material.mat_tex_list[1].mat_tex.tex, 17);
    let encoded = output.write_files().unwrap();
    let decoded = d4::Material::read_with_payload(&encoded.root, &encoded.payload).unwrap();
    let second = convert::import_material(&decoded, &ImportOptions::default()).unwrap();
    assert_eq!(second.value.resources().len(), 1);
}
#[test]
fn material_coefficients_and_unmapped_programs_are_checked() {
    let mut invalid = source();
    invalid.root.duration = f32::NAN;
    assert!(convert::import_material(&invalid, &ImportOptions::default()).is_err());
    let mut source = source();
    source.root.uber_material.mat_tex_list[0]
        .mat_tex
        .tex_anim
        .push(native::Polymorphic::TexAnim(Box::default()));
    assert!(convert::import_material(&source, &ImportOptions::default()).is_err());
    let result = convert::import_material(
        &source,
        &ImportOptions {
            loss_policy: LossPolicy::Drop,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!result.report.issues.is_empty());
}

#[test]
fn ordered_parameter_entries_keep_duplicate_keys_and_controls() {
    let mut source = source();
    let values = &mut source.root.uber_material.run_time_material_values[0].material_scalar_values;
    let mut duplicate = values[0].clone();
    duplicate.value.value = 0.25;
    duplicate.value.path_val_graph_index = -7;
    duplicate.value.light_group = 9;
    values.push(duplicate);
    let imported = convert::import_material(&source, &ImportOptions::default()).unwrap();
    let (id, material) = imported.value.materials().next().unwrap();
    let entries = &material.surface.shader.as_ref().unwrap().values[0].scalars;
    assert_eq!(entries[0].key, entries[1].key);
    assert_eq!(entries[0].value, 0.75);
    assert_eq!(entries[1].value, 0.25);
    assert_eq!(entries[1].controls.graph.code(), -7);
    assert_eq!(entries[1].controls.light_group, 9);
    let output = convert::plan_material(&imported.value, id, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(
        output.root.uber_material.run_time_material_values,
        source.root.uber_material.run_time_material_values
    );
}
