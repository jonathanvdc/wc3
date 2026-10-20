use super::*;
const FIXTURE: &[u8] = include_bytes!("../../../../tests/fixtures/pkb/sparse_curve.pkb");
#[test]
fn omitted_native_curve_width_comes_from_unanimous_typed_calls() {
    let document = Document::read(FIXTURE).unwrap();
    assert_eq!(document.curve_sampler(27).unwrap().value_type, None);
    let program = document
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap();
    assert!(matches!(
        program.inner.samplers[0],
        EffectSampler::Curve {
            kind: ValueType::Float,
            ..
        }
    ));
}
fn changed_returns(banks: &[u8]) -> Vec<u8> {
    let mut bytes = FIXTURE.to_vec();
    let positions: Vec<_> = bytes
        .windows(4)
        .enumerate()
        .filter(|(_, w)| *w == [0, 0, 1, 0x20])
        .map(|(i, _)| i)
        .collect();
    assert_eq!(positions.len(), 2);
    for (index, bank) in banks.iter().enumerate() {
        bytes[positions[index] + 3] = *bank;
    }
    bytes
}
#[test]
fn conflicting_uses_of_one_sparse_curve_fail_with_call_location() {
    let bytes = changed_returns(&[0x20, 0x22]);
    let document = Document::read(&bytes).unwrap();
    let error = document
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap_err();
    assert!(error.message.contains("disagree"));
    assert_eq!(error.layer, Some(EffectLayerId(0)));
    assert_eq!(error.stage, Some(0));
    assert!(error.offset.is_some());
}
#[test]
fn incorrect_component_footprints_are_not_reinterpreted_as_vectors() {
    let bytes = changed_returns(&[0x22, 0x22]);
    let document = Document::read(&bytes).unwrap();
    let error = document
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap_err();
    assert!(error.message.contains("incompatible lengths"));
}

#[test]
fn color_callback_contracts_are_typed_free_functions_and_unknown_symbols_fail() {
    let document = Document::read(FIXTURE).unwrap();
    let options = EffectCompileOptions::default();
    let lowering = Lowering {
        document: &document,
        options: &options,
        samplers: vec![],
        attributes: vec![],
        attribute_slots: HashMap::new(),
        curve_widths: HashMap::new(),
    };
    for symbol in [b"hsv2rgb".as_slice(), b"rgb2hsv"] {
        let (_, result) = lowering
            .callback(symbol, &[ValueType::Float4], None, &[])
            .unwrap();
        assert_eq!(result, Some(ValueType::Float4));
        assert!(lowering
            .callback(symbol, &[ValueType::Float3], None, &[])
            .is_err());
        assert!(lowering
            .callback(
                symbol,
                &[ValueType::Float4],
                Some(0),
                &[External::ParticleContext]
            )
            .is_err());
    }
    assert!(lowering.callback(b"unimplemented", &[], None, &[]).is_err());
}

#[test]
fn typed_boolean_defaults_use_integer_storage_and_canonical_false_when_absent() {
    assert_eq!(
        attribute_default(ValueType::Bool, None, None).unwrap(),
        Value::from(false)
    );
    for word in [1, u32::MAX] {
        assert_eq!(
            attribute_default(ValueType::Bool, None, Some([word, 0, 0, 0])).unwrap(),
            Value::from(true)
        );
    }
    assert!(attribute_default(ValueType::Bool, None, Some([2, 0, 0, 0])).is_err());
    assert!(attribute_default(ValueType::Bool, Some([1f32.to_bits(), 0, 0, 0]), None).is_err());
    assert_eq!(
        attribute_default(
            ValueType::Float,
            Some([2f32.to_bits(), 0, 0, 0]),
            Some([99, 0, 0, 0])
        )
        .unwrap(),
        Value::from(2f32)
    );
}
#[test]
fn orientation_callback_signatures_preserve_quaternion_types_and_reject_bound_receivers() {
    let document = Document::read(FIXTURE).unwrap();
    let options = EffectCompileOptions::default();
    let lowering = Lowering {
        document: &document,
        options: &options,
        samplers: vec![],
        attributes: vec![],
        attribute_slots: HashMap::new(),
        curve_widths: HashMap::new(),
    };
    let (_, result) = lowering
        .callback(b"scene.orientation_f_norm", &[ValueType::Float3], None, &[])
        .unwrap();
    assert_eq!(result, Some(ValueType::Quaternion));
    for (symbol, args) in [
        (
            b"rotate".as_slice(),
            vec![ValueType::Float3, ValueType::Quaternion],
        ),
        (b"orientation_axisSide", vec![ValueType::Quaternion]),
        (b"effect.axisSide", vec![ValueType::Context]),
        (b"effect.axisUp", vec![ValueType::Context]),
        (
            b"radians.rotate",
            vec![ValueType::Float3, ValueType::Float3, ValueType::Float],
        ),
    ] {
        let (_, result) = lowering.callback(symbol, &args, None, &[]).unwrap();
        assert_eq!(result, Some(ValueType::Float3));
        assert!(lowering
            .callback(symbol, &args, Some(0), &[External::EffectContext])
            .is_err());
    }
    assert!(lowering
        .callback(b"orientation_axisSide", &[ValueType::Float4], None, &[])
        .is_err());
}
