use glam::Vec3;
use wc3::model::formats::pkb::Document;
#[path = "common/pkb.rs"]
mod native_fixture;
use native_fixture::{array, fixture, word};
const SHAPE: &[u8] = b"CParticleNodeSamplerData_Shape";
fn vector(values: [f32; 3]) -> Vec<u8> {
    values
        .into_iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}
#[test]
fn sparse_shape_view_keeps_defaults_unresolved_and_raw_authoring_coordinates() {
    let bytes = fixture(&[SHAPE], vec![(0, vec![(0, array(&[]))])], &[]);
    let document = Document::read(&bytes).unwrap();
    let shape = document.shape_sampler(1).unwrap();
    assert_eq!(shape.shape_type, None);
    assert_eq!(shape.dimensionality, None);
    assert_eq!(shape.position, None);
    assert_eq!(shape.radius, None);
    assert_eq!(shape.hemisphere, None);
    let bytes = fixture(
        &[SHAPE],
        vec![(
            0,
            vec![
                (7, [word(2), word((-3i32) as u32)].concat()),
                (9, word(3)),
                (13, vector([1., 2., 3.])),
                (14, vector([10., 20., 30.])),
                (15, word(2)),
                (16, vector([4., 5., 6.])),
                (17, word(2f32.to_bits())),
                (18, word(1f32.to_bits())),
                (20, word(3f32.to_bits())),
                (21, vec![1]),
                (22, vector([7., 8., 9.])),
            ],
        )],
        &[],
    );
    let document = Document::read(&bytes).unwrap();
    let shape = document.shape_sampler(1).unwrap();
    assert_eq!(shape.workspace, Some([2, -3]));
    assert_eq!(shape.shape_type, Some(2));
    assert_eq!(shape.dimensionality, Some(3));
    assert_eq!(shape.position, Some(Vec3::new(1., 2., 3.)));
    assert_eq!(shape.euler_degrees, Some(Vec3::new(10., 20., 30.)));
    assert_eq!(shape.nonuniform_scale, Some(Vec3::new(7., 8., 9.)));
    assert_eq!(shape.hemisphere, Some(true));
}
#[test]
fn shape_unknown_fields_and_noncanonical_booleans_fail_only_the_typed_view() {
    for fields in [vec![(21, vec![2])], vec![(63, word(0))]] {
        let bytes = fixture(&[SHAPE], vec![(0, fields)], &[]);
        let document = Document::read(&bytes).unwrap();
        assert_eq!(document.as_bytes(), bytes);
        assert!(document.shape_sampler(1).is_err());
    }
}
