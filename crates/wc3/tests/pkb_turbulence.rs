use wc3::model::formats::pkb::Document;
#[path = "common/pkb.rs"]
#[allow(dead_code, unused_imports)]
mod native_fixture;
use native_fixture::{fixture, word};
const TYPE: &[u8] = b"CParticleNodeSamplerData_Turbulence";
#[test]
fn sparse_native_turbulence_preserves_omissions_and_integer_seed_identity() {
    let bytes = fixture(&[TYPE], vec![(0, vec![])], &[]);
    let document = Document::read(&bytes).unwrap();
    let native = document.turbulence_sampler(1).unwrap();
    assert_eq!(native.data_source, None);
    assert_eq!(native.global_scale, None);
    assert_eq!(native.octaves, None);
    assert_eq!(native.initial_seed, None);
    let bytes = fixture(
        &[TYPE],
        vec![(
            0,
            vec![
                (9, word(1)),
                (10, word(1)),
                (11, word(0.01f32.to_bits())),
                (12, word(0.01f32.to_bits())),
                (13, vec![0]),
                (14, vec![1]),
                (17, word((-2i32) as u32)),
                (18, word(0.1f32.to_bits())),
                (19, word(2)),
                (20, word(0.018f32.to_bits())),
                (21, word(0.025f32.to_bits())),
                (22, word(1)),
                (23, word(0.2f32.to_bits())),
                (24, word((-0.5f32).to_bits())),
                (25, word(0.5f32.to_bits())),
                (26, word(1f32.to_bits())),
                (27, word(2f32.to_bits())),
                (28, word(0x4269cafe)),
                (29, vec![1]),
                (30, word(1f32.to_bits())),
            ],
        )],
        &[b"external.pkvf"],
    );
    let document = Document::read(&bytes).unwrap();
    let native = document.turbulence_sampler(1).unwrap();
    assert_eq!(native.data_source, Some(1));
    assert_eq!(native.vector_field, Some(b"external.pkvf".as_slice()));
    assert_eq!(native.global_scale, Some(0.01));
    assert_eq!(native.wrapping, [Some(false), Some(true), None, None]);
    assert_eq!(native.filtering, Some(-2));
    assert_eq!(native.time_base, Some(-0.5));
    assert_eq!(native.initial_seed, Some(1_114_229_502));
    assert_eq!(native.fast_fake_flow, Some(true));
    assert_eq!(document.as_bytes(), bytes);
}
#[test]
fn invalid_typed_boolean_and_unknown_fields_preserve_the_native_document() {
    for fields in [
        vec![(29, vec![2])],
        vec![(13, vec![255])],
        vec![(31, word(0))],
        vec![(28, vec![1, 2])],
    ] {
        let bytes = fixture(&[TYPE], vec![(0, fields)], &[]);
        let document = Document::read(&bytes).unwrap();
        assert_eq!(document.as_bytes(), bytes);
        assert!(document.turbulence_sampler(1).is_err());
    }
}
