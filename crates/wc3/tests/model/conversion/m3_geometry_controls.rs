use super::*;
use sem::GeometrySchema as _;
#[test]
fn draw_controls_and_uv_transforms_are_editable_native_fields() {
    let original = asset();
    let source = conversion::m3::plan::<m3::StarCraft2>(&original, &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = source.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let mut document = source.model.into_document();
    let mut division = document
        .resolve::<nr::MeshDivision<nr::V2>>(root.divisions.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let mut region = document
        .resolve::<nr::Region<nr::V5>>(division.regions.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let mut batch = document
        .resolve::<nr::Batch<nr::V1>>(division.batches.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    region.index = 41;
    region.unknown = 0xdeadbeef;
    region.unknown_byte = 7;
    region.flags = nr::RegionFlags(0x1234);
    region.uv_scale = 32.;
    region.uv_offset = 0.5;
    batch.flags = nr::BatchFlags(0x321);
    batch.priority_plane = 9;
    batch.color_index = 17;
    batch.visibility_bone = 0;
    division.instances = 3;
    document
        .replace(
            document.section_id(division.regions.index).unwrap(),
            &[region],
        )
        .unwrap();
    document
        .replace(
            document.section_id(division.batches.index).unwrap(),
            &[batch],
        )
        .unwrap();
    document
        .replace(
            document.section_id(root.divisions.index).unwrap(),
            &[division],
        )
        .unwrap();
    let source = m3::Bundle::from_model(m3::Model::from_document(document).unwrap());
    let mut imported = conversion::m3::import(&source, &Default::default())
        .unwrap()
        .value;
    let (id, mesh) = imported.models[0].meshes().next().unwrap();
    assert_eq!(mesh.texture_coordinates()[0].get(1), Some([2.5, 0.5]));
    let mut mesh = mesh.clone();
    let sem::M3Geometry::Display(display) = &mesh.geometry else {
        panic!("display geometry")
    };
    let mut coordinates = display.texture_coordinates()[0].iter().collect::<Vec<_>>();
    coordinates[1] = [1.5, 0.5];
    mesh.geometry = sem::M3Geometry::Display(
        display
            .clone()
            .with_texture_coordinates(vec![sem::Values::owned(coordinates)])
            .unwrap(),
    );
    imported
        .edit_model(0, |builder| builder.set_mesh(id, mesh))
        .unwrap();
    let result = conversion::m3::plan::<m3::StarCraft2>(&imported, &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = result.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let division = result
        .model
        .document()
        .resolve::<nr::MeshDivision<nr::V2>>(root.divisions.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let region = result
        .model
        .document()
        .resolve::<nr::Region<nr::V5>>(division.regions.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let batch = result
        .model
        .document()
        .resolve::<nr::Batch<nr::V1>>(division.batches.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(
        (
            region.index,
            region.unknown,
            region.unknown_byte,
            region.flags.bits()
        ),
        (41, 0xdeadbeef, 7, 0x1234)
    );
    assert_eq!((region.uv_scale, region.uv_offset), (32., 0.5));
    assert_eq!(
        (
            batch.flags.bits(),
            batch.priority_plane,
            batch.color_index,
            batch.visibility_bone
        ),
        (0x321, 9, 17, 0)
    );
    assert_eq!(division.instances, 3);
    let reread = conversion::m3::import(&result, &Default::default()).unwrap();
    assert_eq!(
        reread.value.models[0]
            .meshes()
            .next()
            .unwrap()
            .1
            .texture_coordinates()[0]
            .get(1),
        Some([1.5, 0.5])
    );
}

#[test]
fn undrawn_regions_retain_definition_binding_and_detail_level() {
    let asset = asset();
    let source = conversion::m3::plan::<m3::StarCraft2>(&asset, &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root_id = source.model.root_id();
    let mut root = source.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let mut division = source
        .model
        .document()
        .resolve::<nr::MeshDivision<nr::V2>>(root.divisions.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    division.batches = m3::Ref::default();
    division.instances = 7;
    let mut document = source.model.into_document();
    let divisions = document
        .append(&[nr::MeshDivision::<nr::V2>::default(), division])
        .unwrap();
    root.divisions = m3::Ref::from_section(2, divisions.index());
    document.replace(root_id, &[root]).unwrap();
    let source = m3::Bundle::from_model(m3::Model::from_document(document).unwrap());
    let mut asset = conversion::m3::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(asset.models[0].instances().count(), 0);
    let (id, definition) = asset.models[0].meshes().next().unwrap();
    assert!(definition.skin().is_some());
    let placement = definition
        .controls
        .as_ref()
        .unwrap()
        .undrawn
        .as_ref()
        .unwrap();
    assert_eq!(placement.detail_level, 1);
    assert_eq!(placement.instance_count, 7);
    let mut edited = definition.clone();
    let sem::M3Geometry::Display(display) = &edited.geometry else {
        unreachable!()
    };
    edited.geometry = sem::M3Geometry::Display(
        display
            .clone()
            .with_texture_coordinates(vec![sem::Values::owned(vec![[0.5, 0.]; 3])])
            .unwrap(),
    );
    asset
        .edit_model(0, |builder| builder.set_mesh(id, edited))
        .unwrap();
    let rebuilt = conversion::m3::plan::<m3::StarCraft2>(&asset, &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = rebuilt.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let divisions = rebuilt
        .model
        .document()
        .resolve::<nr::MeshDivision<nr::V2>>(root.divisions.cast())
        .unwrap();
    assert_eq!(divisions.len(), 2);
    let division = divisions.get(1).unwrap().unwrap();
    assert_eq!(division.instances, 7);
    assert_eq!(division.batches.entries, 0);
    assert_eq!(division.regions.entries, 1);
    let encoded = rebuilt.write().unwrap();
    let source = m3::Bundle::from_model(m3::Model::<m3::Md34>::read(&encoded.model).unwrap());
    let imported = conversion::m3::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let (_, mesh) = imported.models[0].meshes().next().unwrap();
    assert!(mesh.skin().is_some());
    assert_eq!(mesh.texture_coordinates()[0].get(0), Some([0.5, 0.]));
    assert_eq!(imported.models[0].instances().count(), 0);
}

#[test]
fn unselected_bounds_definitions_are_emitted_without_invented_draws() {
    let asset = asset();
    let source = conversion::m3::plan::<m3::StarCraft2>(&asset, &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = source.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let division_id = source
        .model
        .document()
        .section_id(root.divisions.index)
        .unwrap();
    let mut division = source
        .model
        .document()
        .resolve::<nr::MeshDivision<nr::V2>>(root.divisions.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let mut document = source.model.into_document();
    let bounds = document
        .append(&[
            nr::MeshSection::<nr::V1>::default(),
            nr::MeshSection::<nr::V1> {
                node_index: 0,
                bounds: nr::AnimationValue {
                    animation_id: 42,
                    initial_value: nr::Extent {
                        maximum: [3.; 3],
                        radius: 5.,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        ])
        .unwrap();
    division.msec = m3::Ref::from_section(2, bounds.index());
    document.replace(division_id, &[division]).unwrap();
    let source = m3::Bundle::from_model(m3::Model::from_document(document).unwrap());
    let imported = conversion::m3::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(
        imported.models[0]
            .features()
            .filter(|(_, feature)| matches!(feature.payload, surface::Feature::Extent(_)))
            .count(),
        2
    );
    let output = conversion::m3::plan::<m3::StarCraft2>(&imported, &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = output.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let division = output
        .model
        .document()
        .resolve::<nr::MeshDivision<nr::V2>>(root.divisions.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(division.batches.entries, 1);
    assert_eq!(division.msec.entries, 2);
    let bounds = output
        .model
        .document()
        .resolve::<nr::MeshSection<nr::V1>>(division.msec.cast())
        .unwrap()
        .get(1)
        .unwrap()
        .unwrap();
    assert_eq!(bounds.bounds.initial_value.maximum, [3.; 3]);
    assert_eq!(bounds.bounds.initial_value.radius, 5.);
    assert_eq!(bounds.bounds.animation_id, 42);
}
