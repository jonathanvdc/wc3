use super::super::{Callback, EffectCompileOptions, EffectSampler, External, Lowering};
use super::*;
use crate::model::scene::popcorn::turbulence::TemporalCache;
use crate::model::scene::popcorn::ValueType;
use glam::Vec3;
use std::collections::HashMap;
#[path = "../../../../tests/common/pkb.rs"]
#[allow(dead_code, unused_imports)]
mod native_fixture;
use native_fixture::{fixture, word};
const TYPE: &[u8] = b"CParticleNodeSamplerData_Turbulence";
#[test]
fn sparse_defaults_keep_numeric_seed_and_actual_small_wavelengths() {
    let bytes = fixture(&[TYPE], vec![(0, vec![])], &[]);
    let document = Document::read(&bytes).unwrap();
    let native = document.turbulence_sampler(1).unwrap();
    assert_eq!(native.initial_seed, None);
    let implicit = prepare(&document, 1).unwrap();
    let bytes = fixture(&[TYPE], vec![(0, vec![(28, word(0x4269cafe))])], &[]);
    let document = Document::read(&bytes).unwrap();
    assert_eq!(
        document.turbulence_sampler(1).unwrap().initial_seed,
        Some(1_114_229_502)
    );
    let explicit = prepare(&document, 1).unwrap();
    let mut cache = TemporalCache::default();
    let point = Vec3::new(0.1, 0.2, 0.3);
    assert_eq!(
        implicit
            .sample(point, cache.phase(0, &implicit, 0.).unwrap())
            .unwrap(),
        explicit
            .sample(point, cache.phase(1, &explicit, 0.).unwrap())
            .unwrap()
    );
    let fields = vec![
        (11, word(0.01f32.to_bits())),
        (12, word(0.01f32.to_bits())),
        (18, word(0.1f32.to_bits())),
        (20, word(0.018f32.to_bits())),
        (21, word(0.025f32.to_bits())),
    ];
    let bytes = fixture(&[TYPE], vec![(0, fields)], &[]);
    let document = Document::read(&bytes).unwrap();
    let prepared = prepare(&document, 1).unwrap();
    assert!(prepared
        .sample(point, cache.phase(2, &prepared, 0.).unwrap())
        .unwrap()
        .is_finite());
}
#[test]
fn lowering_binds_the_resource_family_and_checked_scene_signature_once() {
    let bytes = fixture(&[TYPE], vec![(0, vec![])], &[]);
    let document = Document::read(&bytes).unwrap();
    let options = EffectCompileOptions::default();
    let mut lowering = Lowering {
        document: &document,
        options: &options,
        samplers: vec![],
        attributes: vec![],
        attribute_slots: HashMap::new(),
        curve_widths: HashMap::new(),
    };
    let index = lowering.sampler(1, 9).unwrap();
    assert!(matches!(
        lowering.samplers[index],
        EffectSampler::Turbulence(_)
    ));
    let externals = [External::Sampler(index)];
    assert_eq!(
        lowering
            .callback(
                b"sample",
                &[ValueType::Float3, ValueType::Context],
                Some(0),
                &externals
            )
            .unwrap(),
        (Callback::TurbulenceSample, Some(ValueType::Float3))
    );
    assert!(lowering
        .callback(
            b"sample",
            &[ValueType::Float, ValueType::Context],
            Some(0),
            &externals
        )
        .is_err());
    assert!(lowering
        .callback(b"sample", &[ValueType::Float3], Some(0), &externals)
        .is_err());
}
#[test]
fn unsupported_external_and_extra_flow_controls_are_not_zero_stubs() {
    for fields in [
        vec![(9, word(1))],
        vec![(9, word(2))],
        vec![(26, word(1f32.to_bits()))],
        vec![(27, word(1f32.to_bits()))],
        vec![(29, vec![1])],
        vec![(22, word(3))],
        vec![(19, word(25))],
    ] {
        let bytes = fixture(&[TYPE], vec![(0, fields)], &[]);
        let document = Document::read(&bytes).unwrap();
        assert!(prepare(&document, 1).is_err());
    }
}
#[test]
#[ignore = "requires the local retained five-resource native corpus"]
fn actual_five_native_turbulence_records_lower_without_defaulting_the_borrowed_view() {
    use std::fs::read;
    use std::path::Path;
    let base =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/dependencies/files/war3.w3mod");
    for (path, links) in [
        (
            "_hd.w3mod/abilities/spells/items/aifb/aifbtarget--2e5e4b3c03a9.pkb",
            &[154u32][..],
        ),
        (
            "_de.w3mod/abilities/spells/items/aifb/aifbtarget--3eebdf06876d.pkb",
            &[154u32][..],
        ),
        (
            "_de.w3mod/abilities/spells/undead/relentlesscleave/relentlesscleave--bf573fa235a8.pkb",
            &[242u32, 1111, 1386][..],
        ),
    ] {
        let bytes = read(base.join(path)).unwrap();
        let document = Document::read(&bytes).unwrap();
        for &link in links {
            let native = document.turbulence_sampler(link).unwrap();
            assert_eq!(native.data_source, None);
            assert_eq!(native.initial_seed, None);
            match link {
                154 => {
                    assert_eq!(native.strength, Some(0.01));
                    assert_eq!(native.wavelength, Some(1.));
                }
                242 => {
                    assert_eq!(native.global_scale, Some(0.01));
                    assert_eq!(native.wavelength, Some(0.1));
                    assert_eq!(native.lacunarity, Some(0.018));
                    assert_eq!(native.gain, Some(0.025));
                }
                1111 => assert_eq!(native.time_scale, Some(0.2)),
                1386 => {
                    assert_eq!(native.strength, None);
                    assert_eq!(native.time_scale, None);
                }
                _ => unreachable!(),
            }
            let prepared = prepare(&document, link).unwrap();
            let mut cache = TemporalCache::default();
            for time in [0., 0.5, 1.] {
                assert!(prepared
                    .sample(
                        Vec3::new(0.1, -0.2, 0.3),
                        cache.phase(0, &prepared, time).unwrap()
                    )
                    .unwrap()
                    .is_finite());
            }
        }
    }
}
