use wc3::model::common::IntoZeroVec;
use wc3::model::conversion::{Context, ExportOptions};
use wc3::model::io::geometry;
use wc3::model::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, ConnectivitySets, Draw,
    GeometryData, InstanceId, Material, Mesh, MeshId, MeshInstance, Model, ModelBuilder, Node,
    Topology,
};

fn fixture() -> (Model<'static>, MeshId, InstanceId) {
    let mut draft = ModelBuilder::new("seams");
    let node = draft.add_node(Node::named("placement")).unwrap();
    let first = draft
        .add_material(
            Material {
                name: "".into(),
                surface: Default::default(),
            }
            .named("first"),
        )
        .unwrap();
    let second = draft
        .add_material(
            Material {
                name: "".into(),
                surface: Default::default(),
            }
            .named("second"),
        )
        .unwrap();
    let mesh = draft
        .add_mesh(Mesh::new(GeometryData {
            positions: vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]].into_zero_vec(),
            connectivity: ConnectivitySets::new(
                Topology::Triangles(vec![[0, 1, 2], [0, 2, 3]].into_zero_vec()),
                Vec::new(),
            ),
            attributes: vec![
                Attribute {
                    semantic: AttributeSemantic::Normal,
                    domain: AttributeDomain::Face,
                    values: AttributeValues::Vector3(
                        vec![[0., 0., 1.], [0., 1., 0.]].into_zero_vec(),
                    ),
                },
                Attribute {
                    semantic: AttributeSemantic::TexCoord(0),
                    domain: AttributeDomain::Corner,
                    values: AttributeValues::Vector2(
                        vec![[0., 0.], [1., 0.], [1., 1.], [0.25, 0.], [1., 1.], [0., 1.]]
                            .into_zero_vec(),
                    ),
                },
            ]
            .try_into()
            .unwrap(),
            ..Default::default()
        }))
        .unwrap();
    let mut instance = MeshInstance::new(node, mesh);
    instance.draws = vec![
        Draw::new(draft.add_primary_subset(mesh, 0, 1).unwrap(), None),
        Draw::new(draft.add_primary_subset(mesh, 1, 1).unwrap(), None),
    ];
    let instance = draft
        .add_instance_with_materials(instance, vec![first, second])
        .unwrap();
    (draft.finish().unwrap(), mesh, instance)
}

#[test]
fn projection_preserves_face_materials_normals_and_uv_seams() {
    let (model, _, instance) = fixture();
    let draws = geometry::project(
        &model,
        instance,
        None,
        &mut Context::export(&ExportOptions::default()),
    )
    .unwrap();
    assert_eq!(draws.len(), 2);
    assert_ne!(draws[0].material, draws[1].material);
    assert_eq!(draws[0].mesh.normals().unwrap().get(0), Some([0., 0., 1.]));
    assert_eq!(draws[1].mesh.normals().unwrap().get(0), Some([0., 1., 0.]));
    assert_eq!(
        draws[1].mesh.texture_coordinates()[0].get(0),
        Some([0.25, 0.])
    );
    assert_eq!(draws[1].source_vertices.get(0), Some(0));
}

#[test]
fn editing_a_corner_keeps_the_shared_vertex_and_other_corner_unchanged() {
    let (mut model, mesh, instance) = fixture();
    let mut edited = model.mesh(mesh).unwrap().clone();
    let geometry = &mut edited.geometry;
    geometry
        .attributes
        .update(1, |attribute| {
            let AttributeValues::Vector2(coordinates) = &mut attribute.values else {
                unreachable!()
            };
            let mut values = coordinates.to_vec();
            values[3] = [0.5, 0.];
            *coordinates = values.into_zero_vec();
        })
        .unwrap();
    model.edit(|draft| draft.set_mesh(mesh, edited)).unwrap();
    assert_eq!(model.mesh(mesh).unwrap().positions().len(), 4);
    let draws = geometry::project(
        &model,
        instance,
        None,
        &mut Context::export(&ExportOptions::default()),
    )
    .unwrap();
    assert_eq!(
        draws[0].mesh.texture_coordinates()[0].get(0),
        Some([0., 0.])
    );
    assert_eq!(
        draws[1].mesh.texture_coordinates()[0].get(0),
        Some([0.5, 0.])
    );
}

#[test]
fn invalid_source_topology_cannot_replace_published_geometry() {
    let (mut model, mesh, _) = fixture();
    let mut edited = model.mesh(mesh).unwrap().clone();
    let geometry = &mut edited.geometry;
    geometry.connectivity.primary =
        Topology::Triangles(vec![[0, 1, 99], [0, 2, 3]].into_zero_vec());
    assert!(model.edit(|draft| draft.set_mesh(mesh, edited)).is_err());
    assert_eq!(model.mesh(mesh).unwrap().positions().len(), 4);
}

#[test]
fn shared_vertex_edits_update_both_draw_projections() {
    let (mut model, mesh, instance) = fixture();
    let mut edited = model.mesh(mesh).unwrap().clone();
    let geometry = &mut edited.geometry;
    let mut positions = geometry.positions.to_vec();
    positions[0] = [7., 0., 0.];
    geometry.positions = positions.into_zero_vec();
    model.edit(|draft| draft.set_mesh(mesh, edited)).unwrap();
    let draws = geometry::project(
        &model,
        instance,
        None,
        &mut Context::export(&ExportOptions::default()),
    )
    .unwrap();
    assert_eq!(draws[0].mesh.positions().get(0), Some([7., 0., 0.]));
    assert_eq!(draws[1].mesh.positions().get(0), Some([7., 0., 0.]));
}

#[test]
fn face_attributes_override_vertex_attributes_during_native_projection() {
    let (mut model, mesh, instance) = fixture();
    model
        .edit(|draft| {
            let mut edited = draft.model().mesh(mesh).unwrap().clone();
            let data = &mut edited.geometry;
            data.set_normals(vec![[1., 0., 0.]; 4].into_zero_vec());
            draft.set_mesh(mesh, edited)
        })
        .unwrap();
    let draws = geometry::project(
        &model,
        instance,
        None,
        &mut Context::export(&ExportOptions::default()),
    )
    .unwrap();
    assert_eq!(draws[0].mesh.normals().unwrap().get(0), Some([0., 0., 1.]));
    assert_eq!(draws[1].mesh.normals().unwrap().get(0), Some([0., 1., 0.]));
}
