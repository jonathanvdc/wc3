use glam::Vec3;
use wc3::model::conversion::{ExportOptions, ImportOptions};
use wc3::model::formats::mdlx::animation::Sequence;
use wc3::model::formats::mdlx::geometry::{Geoset, GeosetExtent};
use wc3::model::formats::mdlx::materials::{Layer, Material, Texture};
use wc3::model::formats::mdlx::scene::Node;
use wc3::model::formats::mdlx::{Document, V800};
use wc3::model::profiles::mdlx::Extent;
use wc3::model::{io, Bounds};
fn empty() -> GeosetExtent {
    GeosetExtent {
        bounds_radius: 0.,
        minimum: [1.; 3],
        maximum: [-1.; 3],
    }
}
fn bounded() -> GeosetExtent {
    GeosetExtent {
        bounds_radius: 4.,
        minimum: [-2.; 3],
        maximum: [2.; 3],
    }
}
fn fixture(
    sequence_extent: GeosetExtent,
    mesh_extents: Vec<GeosetExtent>,
) -> Document<'static, V800> {
    let mut document = Document::new();
    let mut sequence = Sequence::new("Stand", [0, 1000]).unwrap();
    sequence.extent = sequence_extent;
    document.set_sequences([sequence]);
    document.set_textures([Texture::new("texture.blp").unwrap()]);
    let mut material = Material::new();
    material.layers.push(Layer::new());
    document.set_materials([material]);
    document.set_helpers([Node::new("root", 0).unwrap()]);
    document.set_pivot_points(&[[0.; 3]]);
    let mut geoset = Geoset::new([[0.; 3]], [[0., 0., 1.]], [0, 0, 0]).unwrap();
    geoset.sequence_extents = mesh_extents.into_iter().collect();
    document.set_geosets([geoset]);
    document
}

#[test]
fn native_empty_extents_remain_distinct_from_finite_point_bounds_and_roundtrip() {
    let expected = vec![empty(), bounded(), empty()];
    let document = fixture(empty(), expected.clone());
    let model = io::mdlx::import_owned(&document, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(
        model.clips().iter().next().unwrap().1.properties.extent,
        Extent::Empty
    );
    let mesh = model.meshes().iter().next().unwrap().1;
    assert_eq!(mesh.properties.sequence_bounds[0].1, Extent::Empty);
    assert_eq!(
        mesh.properties.unassigned_bounds,
        vec![
            Extent::Bounds(Bounds {
                minimum: Vec3::from_array([-2.; 3]),
                maximum: Vec3::from_array([2.; 3]),
                sphere_radius: 4.
            }),
            Extent::Empty
        ]
    );
    let output = io::mdlx::export::<V800>(&model, &ExportOptions::default())
        .unwrap()
        .value;
    assert_eq!(output.sequences().next().unwrap().extent, empty());
    assert_eq!(
        output.geosets().next().unwrap().sequence_extents.to_vec(),
        expected
    );
    let point = fixture(GeosetExtent::default(), vec![GeosetExtent::default()]);
    let point_model = io::mdlx::import_owned(&point, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(
        point_model
            .clips()
            .iter()
            .next()
            .unwrap()
            .1
            .properties
            .extent,
        Extent::Bounds(Bounds::default())
    );
}

#[test]
fn noncanonical_inversions_and_nonfinite_extents_remain_invalid() {
    for invalid in [
        GeosetExtent {
            minimum: [2.; 3],
            ..empty()
        },
        GeosetExtent {
            bounds_radius: 1.,
            ..empty()
        },
        GeosetExtent {
            bounds_radius: f32::NAN,
            ..bounded()
        },
        GeosetExtent {
            minimum: [f32::INFINITY; 3],
            ..bounded()
        },
    ] {
        let sequence = fixture(invalid, vec![bounded()]);
        assert!(io::mdlx::import_owned(&sequence, &ImportOptions::default()).is_err());
        let mesh = fixture(bounded(), vec![invalid]);
        assert!(io::mdlx::import_owned(&mesh, &ImportOptions::default()).is_err());
    }
}
