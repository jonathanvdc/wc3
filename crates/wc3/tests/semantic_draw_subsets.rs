use glam::Vec3;
use wc3::model::common::IntoZeroVec;
use wc3::model::common::PackedArray;
use wc3::model::conversion::{Context, ExportOptions};
use wc3::model::error::{
    ErrorKind, GeometryError, GeometryOperation, Location, Owner, Segment, TopologyKind,
};
use wc3::model::io::geometry;
use wc3::model::ModelMetadata;
use wc3::model::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, Draw, DrawSubset, Material,
    Mesh, MeshInstance, ModelBuilder, PrimitiveRange, PrimitiveSelection, StripSelection, Topology,
};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};

fn mesh() -> Mesh<'static> {
    Mesh::triangles(
        (vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [1., 1., 0.]])
            .into_iter()
            .map(Vec3::from_array)
            .collect::<PackedArray<_>>(),
        vec![[0, 1, 2], [2, 1, 3]],
    )
}

#[test]
fn instances_share_mesh_owned_selections_with_independent_materials() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("placement").unwrap();
    let first = builder
        .add_material(Material {
            name: "".into(),
            surface: SlotConstruct::construct(&mut slot_values),
        })
        .unwrap();
    let second = builder
        .add_material(Material {
            name: "".into(),
            surface: SlotConstruct::construct(&mut slot_values),
        })
        .unwrap();
    let mut source = mesh();
    let subset = source.add_primary_subset(1, 1);
    let mesh = builder.add_mesh(source).unwrap();
    for material in [first, second] {
        let mut instance = MeshInstance::new(node, mesh);
        instance.draws.push(Draw::new(subset, None));
        builder
            .add_instance_with_materials(instance, vec![material])
            .unwrap();
    }
    let mut model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    model
        .edit(|draft| {
            let mesh_id = mesh;
            let mut mesh = draft.model().mesh(mesh_id).unwrap().clone();
            let mut selection = mesh.subset(subset).unwrap().clone();
            selection.selection.range = PrimitiveRange::Primitives { first: 0, count: 2 };
            mesh.subsets.set(subset, selection)?;
            draft.set_mesh(mesh_id, mesh)
        })
        .unwrap();
    for (id, instance) in model.instances() {
        assert_eq!(instance.draws[0].subset, subset);
        let projected = geometry::project(
            &model,
            id,
            None,
            &mut Context::export(&ExportOptions::default()),
        )
        .unwrap();
        assert_eq!(
            projected[0].material,
            model
                .material_for_draw(id, instance.draws[0].id(), None)
                .unwrap()
        );
        assert_eq!(projected[0].mesh.topology().primitive_count(), 2);
    }
}

#[test]
fn strip_selections_use_their_own_faces_and_explicit_index_counting() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("placement").unwrap();
    let material = builder
        .add_material(Material {
            name: "".into(),
            surface: SlotConstruct::construct(&mut slot_values),
        })
        .unwrap();
    let mut source = mesh();
    let strip = source
        .add_connectivity(Topology::TriangleStrip(PackedArray::from_storage(
            vec![0, 1, 2, 3].into_zero_vec(),
        )))
        .unwrap();
    let subset = source.add_subset(DrawSubset::new(PrimitiveSelection {
        connectivity: strip,
        range: PrimitiveRange::StripIndices { first: 0, count: 4 },
    }));
    let data = &mut source.geometry;
    data.attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Normal,
            domain: AttributeDomain::ConnectivityFace(strip),
            values: AttributeValues::Vector3(PackedArray::from_storage(
                vec![[0., 0., 1.], [0., 0., -1.]].into_zero_vec(),
            )),
        })
        .unwrap();
    let mesh = builder.add_mesh(source).unwrap();
    let mut instance = MeshInstance::new(node, mesh);
    instance.draws.push(Draw::new(subset, None));
    let instance = builder
        .add_instance_with_materials(instance, vec![material])
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let projected = geometry::project(
        &model,
        instance,
        None,
        &mut Context::export(&ExportOptions::default()),
    )
    .unwrap();
    assert_eq!(
        projected[0].source_vertices.to_vec(),
        vec![0, 1, 2, 2, 1, 3]
    );
    assert_eq!(
        projected[0].mesh.normals().unwrap().get(0),
        Some(Vec3::from_array([0., 0., 1.]))
    );
    assert_eq!(
        projected[0].mesh.normals().unwrap().get(3),
        Some(Vec3::from_array([0., 0., -1.]))
    );
    assert_eq!(projected[0].mesh.topology().primitive_count(), 2);
}

#[test]
fn foreign_subset_and_connectivity_identities_are_rejected() {
    let mut slot_values = SlotValues::default();
    let mut owner = mesh();
    let subset = owner.add_primary_subset(0, 1);
    let foreign_connectivity = owner.subsets.primary();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("placement").unwrap();
    let material = builder
        .add_material(Material {
            name: "".into(),
            surface: SlotConstruct::construct(&mut slot_values),
        })
        .unwrap();
    let other = builder.add_mesh(mesh()).unwrap();
    let mut instance = MeshInstance::new(node, other);
    instance.draws.push(Draw::new(subset, None));
    builder
        .add_instance_with_materials(instance, vec![material])
        .unwrap();
    assert!({
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .is_err());
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let mut other = mesh();
    other.add_subset(DrawSubset::new(PrimitiveSelection {
        connectivity: foreign_connectivity,
        range: PrimitiveRange::Primitives { first: 0, count: 1 },
    }));
    builder.add_mesh(other).unwrap();
    assert!({
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .is_err());
}

#[test]
fn range_domains_and_connectivity_attribute_cardinality_are_checked() {
    let slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let mut source = mesh();
    source.add_subset(DrawSubset::new(PrimitiveSelection {
        connectivity: source.subsets.primary(),
        range: PrimitiveRange::StripIndices { first: 0, count: 3 },
    }));
    builder.add_mesh(source).unwrap();
    assert!({
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .is_err());
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let mut source = mesh();
    let connectivity = source.subsets.primary();
    let data = &mut source.geometry;
    data.attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Normal,
            domain: AttributeDomain::ConnectivityFace(connectivity),
            values: AttributeValues::Vector3(PackedArray::from_storage(
                vec![[0.; 3]].into_zero_vec(),
            )),
        })
        .unwrap();
    builder.add_mesh(source).unwrap();
    assert!({
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .is_err());
}

#[test]
fn supplemental_strip_failure_identifies_the_actual_selection_range() {
    let mut source = mesh();
    let selection = PrimitiveSelection {
        connectivity: source.subsets.primary(),
        range: PrimitiveRange::Primitives { first: 0, count: 1 },
    };
    let mut definition = DrawSubset::new(selection);
    definition.strip = Some(StripSelection {
        selection,
        control: 0,
    });
    let subset = source.add_subset(definition);
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let mesh = builder.add_mesh(source).unwrap();
    let error = builder.finish().unwrap_err();
    assert!(
        matches!(error.kind(), ErrorKind::Geometry(GeometryError::SelectionDomain {
        topology: TopologyKind::TriangleStrip, range
    }) if *range == selection.range)
    );
    assert_eq!(
        error.location(),
        &Location::for_owner(subset)
            .field("strip")
            .field("selection")
            .field("range")
            .with_owner(mesh)
    );
}

#[test]
fn constrained_profiles_reject_unsupported_subset_bindings() {
    let slot_values = SlotValues::default();
    use wc3::model::profiles::Mdlx;
    use wc3::model::{MdlxGeometry, Model};
    let geometry = MdlxGeometry::triangles(
        (vec![[0.; 3]; 3])
            .into_iter()
            .map(Vec3::from_array)
            .collect::<PackedArray<_>>(),
        vec![[0, 1, 2]],
    )
    .unwrap();
    let mut mesh = Mesh::<Mdlx>::new(geometry);
    let subset = mesh.add_primary_subset(0, 1);
    let mut definition = mesh.subset(subset).unwrap().clone();
    definition.vertex_window = Some([0, 2]);
    mesh.subsets.set(subset, definition).unwrap();
    let mut builder = Model::<Mdlx>::builder("");
    builder.add_mesh(mesh).unwrap();
    builder.import_values(&slot_values).unwrap();
    let error = builder.finish().unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Geometry(GeometryError::UnsupportedOperation {
            operation: GeometryOperation::VertexWindows
        })
    ));
    assert!(error
        .location()
        .segments()
        .iter()
        .any(|segment| matches!(segment, Segment::Owner(Owner::DrawSubset(id)) if *id == subset)));
}
