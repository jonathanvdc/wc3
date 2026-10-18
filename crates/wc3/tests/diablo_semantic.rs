use wc3::model::common::{IntoZeroVec, ZeroVec};
use wc3::model::profiles::diablo::d3::{self, Geometry, NodeRole, VertexExtras, VertexStreams, D3};
use wc3::model::profiles::Universal;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{Mesh, Model, SnoNamespace, SnoResource};

fn triangle<'a>(positions: &'a [[f32; 3]]) -> Geometry<'a> {
    Geometry::new(
        VertexStreams {
            positions: ZeroVec::alloc_from_slice(positions),
            normals: vec![[0., 0., 1.]; 3].into_zero_vec(),
            colors: vec![[1.; 4]; 3].into_zero_vec(),
            texture_coordinates: [
                vec![[0.; 2]; 3].into_zero_vec(),
                vec![[1.; 2]; 3].into_zero_vec(),
            ],
            tangents: vec![[1., 0., 0.]; 3].into_zero_vec(),
            extras: VertexExtras {
                auxiliary_colors: vec![[0.; 4]; 3].into_zero_vec(),
                binormals: vec![[0., -1., 0.]; 3].into_zero_vec(),
                direction_auxiliary: ZeroVec::default(),
                cloth_vertices: vec![0; 3].into_zero_vec(),
            },
        },
        vec![[0, 1, 2]],
        Vec::new(),
    )
    .unwrap()
}

#[test]
fn widening_and_narrowing_preserve_geometry_identity_and_borrowed_arrays() {
    let mut slot_values = SlotValues::default();
    let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let mut builder =
        Model::<D3>::builder(wc3::model::profiles::diablo::d3::ModelMetadata::default());
    let material = builder
        .add_material(wc3::model::Material {
            name: "".into(),
            surface: d3::Material::default(),
        })
        .unwrap();
    let mut appearance = d3::Appearance::default();
    let slot = appearance.add_slot(d3::Slot::named("surface"));
    appearance.add_variant(slot, material).unwrap();
    builder.set_appearance(appearance);
    let id = builder
        .add_mesh(
            Mesh::new(triangle(&positions)).with_controls(d3::MeshControls {
                material_slot: Some(slot),
                ..SlotConstruct::construct(&mut slot_values)
            }),
        )
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let wider = model.widen::<Universal>();
    assert_eq!(wider.mesh(id).unwrap().positions().to_vec(), positions);
    let restored = wider.try_narrow::<D3>().unwrap();
    assert_eq!(restored.mesh(id).unwrap().positions().to_vec(), positions);
    assert_eq!(
        restored
            .mesh(id)
            .unwrap()
            .geometry
            .streams()
            .extras
            .binormals
            .get(0),
        Some([0., -1., 0.])
    );
}

#[test]
fn independent_binormals_and_auxiliary_colors_are_validated() {
    let positions = [[0.; 3]; 3];
    let geometry = triangle(&positions);
    let mut streams = geometry.streams().clone();
    streams.extras.binormals = vec![[f32::NAN, 0., 0.]; 3].into_zero_vec();
    assert!(geometry.clone().with_streams(streams).is_err());
    let mut streams = geometry.streams().clone();
    streams.extras.auxiliary_colors = vec![[0.; 4]; 2].into_zero_vec();
    assert!(geometry.with_streams(streams).is_err());
}

#[test]
fn source_namespaces_do_not_alias_and_profiles_require_their_game() {
    let slot_values = SlotValues::default();
    let d3 = SnoResource::Sno {
        namespace: SnoNamespace::Diablo3,
        group: 44,
        id: 7,
    };
    let d4 = SnoResource::Sno {
        namespace: SnoNamespace::Diablo4,
        group: 44,
        id: 7,
    };
    assert_ne!(d3, d4);
    let mut builder =
        Model::<D3>::builder(wc3::model::profiles::diablo::d3::ModelMetadata::default());
    let _resource = builder.add_image_resource(d4);
    assert!({
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .is_err());
}

#[test]
fn nonuniform_bone_transforms_fail_checked_publication() {
    let mut slot_values = SlotValues::default();
    let mut builder =
        Model::<D3>::builder(wc3::model::profiles::diablo::d3::ModelMetadata::default());
    let mut node = builder.new_node("bone");
    node.controls = NodeRole::Bone(Box::new(SlotConstruct::construct(&mut slot_values)));
    builder
        .values_mut()
        .set(node.transform.scale, [1., 2., 1.])
        .unwrap();
    builder.add_node(node).unwrap();
    assert!({
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .is_err());
}

#[test]
fn d4_resource_publication_requires_the_correct_game_namespace() {
    let slot_values = SlotValues::default();
    use wc3::model::profiles::D4;
    for (namespace, valid) in [
        (SnoNamespace::Diablo4, true),
        (SnoNamespace::Diablo3, false),
    ] {
        let mut builder =
            Model::<D4>::builder(wc3::model::profiles::diablo::d4::ModelMetadata::default());
        let _resource = builder.add_image_resource(SnoResource::Sno {
            namespace,
            group: 44,
            id: 12,
        });
        assert_eq!(
            {
                builder.import_values(&slot_values).unwrap();
                builder.finish()
            }
            .is_ok(),
            valid
        );
    }
}

#[test]
fn d4_chunks_and_look_hashes_have_model_scoped_ownership() {
    let mut slot_values = SlotValues::default();
    use wc3::model::profiles::diablo::d4::{self, Chunk, D4};
    use wc3::model::GeometryData;
    let mut foreign = d4::ModelMetadata::default();
    let foreign_id = foreign.add_chunk(Chunk::default());
    let mut own = d4::ModelMetadata::default();
    own.add_chunk(Chunk::default());
    let mut builder =
        Model::<D4>::builder(wc3::model::profiles::diablo::d4::ModelMetadata::default());
    builder.set_metadata(own);
    let geometry = d4::Geometry::new(GeometryData::default()).unwrap();
    builder
        .add_mesh(Mesh::new(geometry).with_controls(d4::MeshControls {
            chunk: Some(foreign_id),
            ..SlotConstruct::construct(&mut slot_values)
        }))
        .unwrap();
    assert!({
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .is_err());

    let mut a = d4::Appearance::default();
    let slot_a = a.add_slot(d4::Slot::new(1, 0, 0));
    let mut b = d4::Appearance::default();
    let slot_b = b.add_slot(d4::Slot::new(1, 0, 0));
    let mut builder =
        Model::<D4>::builder(wc3::model::profiles::diablo::d4::ModelMetadata::default());
    let material = builder
        .add_material(wc3::model::Material {
            name: "".into(),
            surface: d4::Material::default(),
        })
        .unwrap();
    let foreign_variant = a.add_variant(slot_a, material).unwrap();
    b.add_variant(slot_b, material).unwrap();
    assert!(b.add_look(9, vec![(slot_b, foreign_variant)]).is_err());
    assert!(b.add_look(9, vec![(slot_a, foreign_variant)]).is_err());
}
