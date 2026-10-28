use super::*;
use crate::model::common::{Borrowed, IntoPackedArray};
use crate::model::ids::Scope;
use crate::model::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, ConnectivitySets,
    ExplicitBinding, GeometryData, GeometryInfluences, GeometrySchema, M2Geometry, MdlxGeometry,
    TriangleProperties, UniversalBinding, WmoGeometry,
};
use glam::Mat4;
use std::borrow::Cow;

type ExtraVertices<'data> = VertexData<
    'data,
    [PackedArray<'data, Vec2>; 2],
    (),
    (),
    PackedArray<'data, Vec3>,
    ValueGroups<'data, u32>,
>;
type ExtraStreams<'data> = GeometryStreams<'data, ExtraVertices<'data>, JointInfluences<'data>>;

#[test]
fn generic_stream_components_detach_hidden_lifetimes_and_move_owned_buffers() {
    let owned: ExtraStreams<'static> = {
        let normals = [Vec3::Z, Vec3::Z, Vec3::Z].into_packed_array();
        let uv = [Vec2::ZERO, Vec2::X, Vec2::Y].into_packed_array();
        let extras = ValueGroups::from_rows([vec![1u32, 2], vec![], vec![3]]);
        let positions = [Vec3::ZERO, Vec3::X, Vec3::Y].into_packed_array();
        let positions_pointer = positions.as_bytes().as_ptr();
        let triangles = [[0, 1, 2]].into_packed_array();
        let vertices = VertexData {
            positions,
            normals: normals.as_borrowed(),
            tangents: (),
            colors: (),
            texture_coordinates: [uv.as_borrowed(), uv.as_borrowed()],
            extras: extras.as_borrowed(),
        };
        let vertices: ExtraVertices<'static> = vertices.into_owned();
        assert_eq!(vertices.positions.as_bytes().as_ptr(), positions_pointer);
        assert!(vertices.normals.is_owned());
        assert!(vertices
            .texture_coordinates
            .iter()
            .all(PackedArray::is_owned));
        assert!(vertices.extras.values().is_owned());
        let streams = GeometryStreams {
            vertices,
            topology: Topology::Triangles(triangles.as_borrowed()),
            influences: JointInfluences::MatrixGroups(extras.as_borrowed()),
        };
        streams.into_owned()
    };
    assert_eq!(owned.positions.to_vec(), [Vec3::ZERO, Vec3::X, Vec3::Y]);
    assert_eq!(owned.normals.to_vec(), [Vec3::Z; 3]);
    assert_eq!(
        owned.texture_coordinates[1].to_vec(),
        [Vec2::ZERO, Vec2::X, Vec2::Y]
    );
    assert_eq!(owned.extras.to_vec(), [vec![1, 2], vec![], vec![3]]);
    let JointInfluences::MatrixGroups(groups) = owned.influences else {
        panic!("matrix groups")
    };
    assert_eq!(groups.offsets().to_vec(), [0, 2, 2, 3]);
    let Topology::Triangles(triangles) = owned.topology else {
        panic!("triangles")
    };
    assert_eq!(triangles.to_vec(), [[0, 1, 2]]);
}

#[test]
fn shared_geometry_keeps_domains_topology_order_and_authored_influence_form() {
    let connectivity = Scope::default().allocate();
    let material = Scope::default().allocate();
    let owned: GeometryData<'static> = {
        let name = String::from("named mask");
        let positions = [Vec3::ZERO, Vec3::X, Vec3::Y].into_packed_array();
        let corners = [2, 0, 1].into_packed_array();
        let offsets = [0, 3].into_packed_array();
        let lines = [[0, 2]].into_packed_array();
        let strip = [2, 1, 0].into_packed_array();
        let values = [f32::from_bits(0x8000_0000)].into_packed_array();
        let faces = [TriangleProperties {
            material: Some(material),
            render: true,
            ..Default::default()
        }]
        .into_packed_array();
        let groups = ValueGroups::from_rows([vec![Influence {
            joint: 7,
            weight: 1.,
        }]]);
        let mut geometry = GeometryData {
            positions: positions.as_borrowed(),
            connectivity: ConnectivitySets::new(
                Topology::Mixed(vec![
                    Topology::Polygons {
                        corners: corners.as_borrowed(),
                        offsets: offsets.as_borrowed(),
                    },
                    Topology::Lines(lines.as_borrowed()),
                ]),
                vec![Topology::TriangleStrip(strip.as_borrowed())],
            ),
            influences: GeometryInfluences::Variable(groups.as_borrowed()),
            ..Default::default()
        };
        geometry
            .attributes
            .insert(Attribute {
                semantic: AttributeSemantic::Named(Cow::Borrowed(&name)),
                domain: AttributeDomain::ConnectivityFace(connectivity),
                values: AttributeValues::Scalar(values.as_borrowed()),
            })
            .unwrap();
        geometry
            .attributes
            .insert(Attribute {
                semantic: AttributeSemantic::Named(Cow::Borrowed("face links")),
                domain: AttributeDomain::Face,
                values: AttributeValues::TriangleProperties(faces.as_borrowed()),
            })
            .unwrap();
        geometry.into_owned()
    };
    assert_eq!(
        owned.attributes[0].domain,
        AttributeDomain::ConnectivityFace(connectivity)
    );
    assert_eq!(
        owned.attributes[0].semantic,
        AttributeSemantic::Named(Cow::Borrowed("named mask"))
    );
    let AttributeValues::Scalar(values) = &owned.attributes[0].values else {
        panic!("scalar")
    };
    assert_eq!(values.get(0).unwrap().to_bits(), 0x8000_0000);
    let AttributeValues::TriangleProperties(faces) = &owned.attributes[1].values else {
        panic!("face links")
    };
    assert_eq!(faces.get(0).unwrap().material, Some(material));
    let GeometryInfluences::Variable(groups) = &owned.influences else {
        panic!("variable weights")
    };
    assert_eq!(
        groups.values().get(0),
        Some(Influence {
            joint: 7,
            weight: 1.
        })
    );
    let Topology::Mixed(primary) = &owned.connectivity.primary else {
        panic!("mixed")
    };
    assert!(
        matches!(&primary[0], Topology::Polygons { corners, offsets } if corners.to_vec() == [2, 0, 1] && offsets.to_vec() == [0, 3])
    );
    assert!(matches!(&primary[1], Topology::Lines(lines) if lines.to_vec() == [[0, 2]]));
    assert!(
        matches!(&owned.connectivity.additional()[0], Topology::TriangleStrip(indices) if indices.to_vec() == [2, 1, 0])
    );
}

#[test]
fn constrained_profiles_and_explicit_binding_outlive_borrowed_sources() {
    let (mdlx, m2, wmo, binding): (
        MdlxGeometry<'static>,
        M2Geometry<'static>,
        WmoGeometry<'static>,
        UniversalBinding<'static>,
    ) = {
        let positions = [Vec3::ZERO, Vec3::X, Vec3::Y].into_packed_array();
        let normals = [Vec3::Z; 3].into_packed_array();
        let tangents = [Vec4::X; 3].into_packed_array();
        let uv = [Vec2::ZERO, Vec2::X, Vec2::Y].into_packed_array();
        let colors = [Vec4::ONE; 3].into_packed_array();
        let triangles = [[0, 1, 2]].into_packed_array();
        let strip = [0, 1, 2].into_packed_array();
        let slots = [
            Influence {
                joint: 2,
                weight: 1.,
            },
            Influence {
                joint: 9,
                weight: 0.,
            },
            Influence {
                joint: 4,
                weight: 0.,
            },
            Influence {
                joint: 5,
                weight: 0.,
            },
        ];
        let influences = [slots; 3].into_packed_array();
        let matrix = Mat4::from_translation(Vec3::new(1., 2., 3.));
        let matrices = [matrix].into_packed_array();
        let mdlx = MdlxGeometry::triangles(positions.as_borrowed(), triangles.as_borrowed())
            .unwrap()
            .with_normals(normals.as_borrowed())
            .unwrap()
            .with_tangents(tangents.as_borrowed())
            .unwrap()
            .with_texture_coordinates(vec![uv.as_borrowed()])
            .unwrap()
            .with_influences(JointInfluences::Four(influences.as_borrowed()))
            .unwrap();
        let m2 = M2Geometry::new(
            positions.as_borrowed(),
            normals.as_borrowed(),
            [uv.as_borrowed(), uv.as_borrowed()],
            triangles.as_borrowed(),
            influences.as_borrowed(),
        )
        .unwrap();
        let wmo = WmoGeometry::triangles(positions.as_borrowed(), triangles.as_borrowed())
            .unwrap()
            .with_normals(normals.as_borrowed())
            .unwrap()
            .with_texture_coordinates(vec![uv.as_borrowed(), uv.as_borrowed()])
            .unwrap()
            .with_color_sets(vec![colors.as_borrowed(), colors.as_borrowed()])
            .unwrap()
            .with_lightmap_coordinates(uv.as_borrowed())
            .unwrap()
            .with_strip_indices(strip.as_borrowed())
            .unwrap()
            .with_tangent_vectors(tangents.as_borrowed())
            .unwrap();
        let binding =
            UniversalBinding::Explicit(ExplicitBinding::new(matrices.as_borrowed()).unwrap());
        (
            mdlx.into_owned(),
            m2.into_owned(),
            wmo.into_owned(),
            binding.into_owned(),
        )
    };
    fn check_geometry(geometry: &impl GeometrySchema<'static>) {
        assert!(geometry.positions().is_owned());
        assert_eq!(
            geometry.positions().to_vec(),
            [Vec3::ZERO, Vec3::X, Vec3::Y]
        );
        assert!(geometry.normals().unwrap().is_owned());
    }
    check_geometry(&mdlx);
    check_geometry(&m2);
    check_geometry(&wmo);
    mdlx.validate().unwrap();
    m2.validate().unwrap();
    wmo.validate().unwrap();
    assert_eq!(
        m2.influences().joint_weights(0).unwrap().get(1).unwrap(),
        Influence {
            joint: 9,
            weight: 0.
        }
    );
    assert_eq!(wmo.color_sets().len(), 2);
    assert_eq!(
        wmo.lightmap_coordinates().to_vec(),
        [Vec2::ZERO, Vec2::X, Vec2::Y]
    );
    assert_eq!(wmo.strip_indices().to_vec(), [0, 1, 2]);
    let UniversalBinding::Explicit(binding) = binding else {
        panic!("explicit binding")
    };
    assert_eq!(
        binding.values().get(0),
        Some(Mat4::from_translation(Vec3::new(1., 2., 3.)))
    );
    assert!(binding.values().is_owned());
}
