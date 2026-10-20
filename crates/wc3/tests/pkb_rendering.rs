use wc3::model::formats::pkb::Document;

type Fields = Vec<(u16, Vec<u8>)>;
fn word(value: u32) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}
fn array(values: &[u32]) -> Vec<u8> {
    let mut bytes = word(values.len() as u32);
    bytes.extend(values.iter().flat_map(|value| value.to_le_bytes()));
    bytes
}
fn fixture(types: &[&[u8]], objects: Vec<(u32, Fields)>, strings: &[&[u8]]) -> Vec<u8> {
    let mut bodies = Vec::new();
    for (kind, fields) in &objects {
        let mut body = vec![0x20];
        body.extend(kind.to_le_bytes());
        body.extend((fields.len() as u16).to_le_bytes());
        for (field, value) in fields {
            body.extend(field.to_le_bytes());
            body.extend(value);
        }
        bodies.extend((body.len() as u32).to_le_bytes());
        bodies.extend(body);
    }
    let mut bytes = vec![0x11, 0x0b, 0x00, 0xca, 2, 5, 1, 1];
    for value in [
        0,
        objects.len() as u32,
        types.len() as u32,
        28 + types.len() as u32 * 8 + bodies.len() as u32,
        0,
    ] {
        bytes.extend(value.to_le_bytes());
    }
    for index in 0..types.len() as u32 {
        bytes.extend(index.to_le_bytes());
        bytes.extend(0u32.to_le_bytes());
    }
    bytes.extend(bodies);
    bytes.extend(((types.len() + strings.len()) as u32).to_le_bytes());
    for string in types.iter().chain(strings) {
        assert!(string.len() < 128);
        bytes.push(string.len() as u8);
        bytes.extend(*string);
    }
    bytes
}
const RENDERER: &[u8] = b"CLayerCompileCacheRenderer";
const INPUT: &[u8] = b"CLayerCompileCacheRendererParticleInput";
const PROPERTY: &[u8] = b"CLayerCompileCacheRendererProperty";
const NAMED: &[u8] = b"CLayerCompileCacheSampler";
const CURVE: &[u8] = b"CParticleNodeSamplerData_Curve";
const EVENTS: &[u8] = b"CParticleNodeSamplerData_EventStream";

#[test]
fn renderer_sparse_defaults_preserve_present_false_without_synthesizing_other_properties() {
    let bytes = fixture(
        &[RENDERER, INPUT, PROPERTY],
        vec![
            (0, vec![(1, array(&[2])), (2, array(&[3, 4])), (3, word(3))]),
            (1, vec![(1, word(37))]),
            (2, vec![(0, word(4))]),
            (
                2,
                vec![
                    (0, word(5)),
                    (1, word(3)),
                    (
                        2,
                        [0x7fc00001u32, u32::MAX, 7, 9]
                            .into_iter()
                            .flat_map(u32::to_le_bytes)
                            .collect(),
                    ),
                    (3, word(6)),
                ],
            ),
        ],
        &[
            b"Billboard.pkma",
            b"EnableRendering",
            b"DiffuseMap",
            b"color.dds",
        ],
    );
    let document = Document::read(&bytes).unwrap();
    let renderer = document.renderer(1).unwrap();
    assert_eq!((renderer.class, renderer.draw_order), (0, 0));
    assert_eq!(renderer.feature_set, b"Billboard.pkma");
    assert_eq!(renderer.inputs[0].semantic, 0);
    assert_eq!(renderer.inputs[0].storage_index, 37);
    assert_eq!(renderer.inputs[0].additional_name, None);
    assert_eq!(renderer.properties.len(), 2);
    assert_eq!(renderer.properties[0].name, b"EnableRendering");
    assert_eq!(renderer.properties[0].property_type, 0);
    assert_eq!(renderer.properties[0].numeric, [0; 4]);
    assert_eq!(renderer.properties[0].text, None);
    assert_eq!(renderer.properties[1].numeric, [0x7fc00001, u32::MAX, 7, 9]);
    assert_eq!(renderer.properties[1].text, Some(b"color.dds".as_slice()));
    assert_eq!(
        renderer.properties[0].name.as_ptr(),
        document.strings()[4].as_ptr()
    );
}

#[test]
fn renderer_signed_priority_and_custom_input_types_remain_lossless() {
    let bytes = fixture(
        &[RENDERER, INPUT],
        vec![
            (
                0,
                vec![
                    (0, word((-7i32) as u32)),
                    (1, array(&[2])),
                    (2, array(&[])),
                    (3, word(2)),
                    (4, word((-90i32) as u32)),
                ],
            ),
            (
                1,
                vec![(0, word(100)), (1, word(5)), (2, word(3)), (3, word(34))],
            ),
        ],
        &[b"", b"Color"],
    );
    let document = Document::read(&bytes).unwrap();
    let renderer = document.renderer(1).unwrap();
    assert_eq!((renderer.class, renderer.draw_order), (-7, -90));
    assert_eq!(renderer.inputs[0].semantic, 100);
    assert_eq!(
        renderer.inputs[0].additional_name,
        Some(b"Color".as_slice())
    );
    assert_eq!(renderer.inputs[0].additional_type, Some(34));
}

#[test]
fn curve_arrays_borrow_and_sparse_value_type_stays_absent() {
    let bytes = fixture(
        &[CURVE],
        vec![(
            0,
            vec![
                (0, [word(1), word(42), word(1)].concat()),
                (7, [word((-2i32) as u32), word(8)].concat()),
                (16, array(&[0.0f32.to_bits(), 1.0f32.to_bits()])),
                (17, array(&[0.25f32.to_bits(), 0.75f32.to_bits()])),
            ],
        )],
        &[b"Editor name"],
    );
    let document = Document::read(&bytes).unwrap();
    let curve = document.curve_sampler(1).unwrap();
    assert_eq!(curve.names, vec![(42, b"Editor name".as_slice())]);
    assert_eq!(curve.workspace, Some([-2, 8]));
    assert_eq!(curve.value_type, None);
    assert_eq!(curve.interpolator, None);
    assert_eq!(curve.quality, None);
    assert_eq!(curve.times.iter().collect::<Vec<_>>(), vec![0., 1.]);
    assert_eq!(curve.values.iter().collect::<Vec<_>>(), vec![0.25, 0.75]);
    assert!(curve.tangents.is_empty());
    assert!(!curve.times.is_owned());
    assert!(!curve.values.is_owned());
    assert_eq!(document.as_bytes(), bytes);
}

#[test]
fn explicit_curve_metadata_and_tangents_are_not_reinterpreted() {
    let bytes = fixture(
        &[CURVE],
        vec![(
            0,
            vec![
                (9, word(4)),
                (10, word(99)),
                (14, word(2)),
                (16, array(&[0])),
                (17, array(&[0x7fc00001])),
                (18, array(&[1, 2])),
            ],
        )],
        &[],
    );
    let document = Document::read(&bytes).unwrap();
    let curve = document.curve_sampler(1).unwrap();
    assert_eq!(curve.value_type, Some(4));
    assert_eq!(curve.interpolator, Some(99));
    assert_eq!(curve.quality, Some(2));
    assert_eq!(curve.values.get(0).unwrap().to_bits(), 0x7fc00001);
    assert_eq!(
        curve.tangents.iter().map(f32::to_bits).collect::<Vec<_>>(),
        vec![1, 2]
    );
}

#[test]
fn named_unknown_sampler_payloads_remain_checked_raw_object_links() {
    let bytes = fixture(
        &[NAMED, b"FutureShape"],
        vec![
            (0, vec![(0, word(2)), (1, word(2)), (3, word(9))]),
            (1, vec![(60, vec![1, 2, 3, 4, 5])]),
        ],
        &[b"__sampler_0"],
    );
    let document = Document::read(&bytes).unwrap();
    let sampler = document.named_sampler(1).unwrap();
    assert_eq!(sampler.name, b"__sampler_0");
    assert_eq!(sampler.sampler_type, 9);
    assert_eq!(sampler.usage, 0);
    assert_eq!(sampler.sampler, 2);
    assert_eq!(
        document.object(sampler.sampler).unwrap().fields,
        &[60, 0, 1, 2, 3, 4, 5]
    );
    assert!(document.curve_sampler(sampler.sampler).is_err());
    assert_eq!(document.as_bytes(), bytes);
}

#[test]
fn event_stream_times_borrow_without_applying_interval_policy() {
    let bytes = fixture(
        &[EVENTS],
        vec![(0, vec![(10, array(&[0., 0.5, 1.].map(f32::to_bits)))])],
        &[],
    );
    let document = Document::read(&bytes).unwrap();
    let stream = document.event_stream_sampler(1).unwrap();
    assert_eq!(stream.times.iter().collect::<Vec<_>>(), vec![0., 0.5, 1.]);
    assert!(!stream.times.is_owned());
}

#[test]
fn invalid_links_unknown_duplicate_and_truncated_fields_fail_only_the_requested_view() {
    let cases = [
        fixture(
            &[RENDERER, PROPERTY],
            vec![
                (0, vec![(1, array(&[2])), (2, array(&[])), (3, word(2))]),
                (1, vec![(0, word(2))]),
            ],
            &[b"x"],
        ),
        fixture(
            &[PROPERTY],
            vec![(0, vec![(0, word(1)), (0, word(1))])],
            &[b"x"],
        ),
        fixture(
            &[PROPERTY],
            vec![(0, vec![(0, word(1)), (4, word(7))])],
            &[b"x"],
        ),
        fixture(
            &[CURVE],
            vec![(0, vec![(16, [word(2), word(0)].concat()), (17, array(&[]))])],
            &[],
        ),
        fixture(
            &[NAMED],
            vec![(0, vec![(0, word(1)), (1, word(0)), (3, word(3))])],
            &[b"s"],
        ),
    ];
    for (index, bytes) in cases.iter().enumerate() {
        let document = Document::read(bytes).unwrap();
        let failed = match index {
            0 => document.renderer(1).is_err(),
            1 | 2 => document.renderer_property(1).is_err(),
            3 => document.curve_sampler(1).is_err(),
            _ => document.named_sampler(1).is_err(),
        };
        assert!(failed, "case {index}");
        assert_eq!(document.as_bytes(), bytes);
    }
}
