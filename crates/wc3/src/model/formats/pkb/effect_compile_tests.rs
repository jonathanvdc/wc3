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

#[test]
fn trigger_is_a_typed_free_boolean_fraction_callback() {
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
    let args = [ValueType::Bool, ValueType::Float, ValueType::Context];
    assert_eq!(
        lowering.callback(b"trigger", &args, None, &[]).unwrap(),
        (Callback::Trigger, Some(ValueType::Int3))
    );
    assert!(lowering
        .callback(b"trigger", &args, Some(0), &[External::ParticleContext])
        .is_err());
    assert!(lowering
        .callback(
            b"trigger",
            &[ValueType::Int, ValueType::Float, ValueType::Context],
            None,
            &[]
        )
        .is_err());
}
#[test]
fn vector_random_has_distinct_unit_and_shell_free_function_abis() {
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
    for args in [
        vec![ValueType::Context],
        vec![ValueType::Float, ValueType::Float, ValueType::Context],
    ] {
        assert_eq!(
            lowering.callback(b"vrand", &args, None, &[]).unwrap(),
            (Callback::VRand, Some(ValueType::Float3))
        );
        assert!(lowering
            .callback(b"vrand", &args, Some(0), &[External::RandomContext])
            .is_err());
    }
    assert!(lowering
        .callback(b"vrand", &[ValueType::Float], None, &[])
        .is_err());
    assert!(lowering
        .callback(
            b"vrand",
            &[ValueType::Context, ValueType::Context],
            None,
            &[]
        )
        .is_err());
}
#[test]
fn inverse_callbacks_keep_distinct_point_and_direction_contracts_and_free_context() {
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
    for (name, callback) in [
        (b"xform_w2l_f_masked".as_slice(), Callback::InverseTransform),
        (b"xform_w2l_d_masked".as_slice(), Callback::InverseDirection),
    ] {
        let arguments = [ValueType::Float3, ValueType::Int, ValueType::Context];
        assert_eq!(
            lowering.callback(name, &arguments, None, &[]).unwrap(),
            (callback, Some(ValueType::Float3))
        );
        assert!(lowering
            .callback(name, &arguments, Some(0), &[External::ParticleContext])
            .is_err());
        assert!(lowering
            .callback(
                name,
                &[ValueType::Float3, ValueType::Float, ValueType::Context],
                None,
                &[]
            )
            .is_err());
    }
}

#[test]
fn scene_camera_binding_checks_native_capability_schema_and_free_signature() {
    let native = ExternalDeclaration {
        name: b"SceneContext",
        type_name: b"SceneCtx",
        native_type: NativeType(u32::MAX),
        storage_bytes: 8,
        meta_type: 2,
        attributes: 2,
        access: 1,
    };
    assert!(matches!(
        scene_external(&native).unwrap(),
        External::SceneContext
    ));
    for field in 0..6 {
        let mut invalid = native.clone();
        match field {
            0 => invalid.type_name = b"RandCtx",
            1 => invalid.native_type = NativeType(25),
            2 => invalid.storage_bytes = 4,
            3 => invalid.meta_type = 0,
            4 => invalid.attributes = 0,
            _ => invalid.access = 3,
        }
        assert!(scene_external(&invalid).is_err());
    }
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
    let arguments = [ValueType::Int, ValueType::Context];
    let (callback, result) = lowering
        .callback(b"view.position", &arguments, None, &[])
        .unwrap();
    assert_eq!(callback, Callback::ViewPosition);
    assert_eq!(result, Some(ValueType::Float3));
    assert!(lowering
        .callback(
            b"view.position",
            &[ValueType::Float, ValueType::Context],
            None,
            &[]
        )
        .is_err());
    assert!(lowering
        .callback(
            b"view.position",
            &arguments,
            Some(0),
            &[External::SceneContext]
        )
        .is_err());
    assert!(lowering
        .callback(b"scene.intersect", &[], None, &[])
        .is_err());
    assert!(lowering
        .callback(b"xform_w2w_f_masked", &[], None, &[])
        .is_err());
}

#[test]
fn spatial_identity_uses_full_name_and_payload_schema_not_alias_or_writer_flags() {
    use crate::model::formats::pkb::SpatialPayload;
    let mut native = SpatialLayer {
        name: b"Original/Keyed",
        local_name: b"write",
        flags: Some(1),
        cell_size: Some(0.75),
        payload: vec![SpatialPayload {
            name: b"Position",
            native_type: NativeType(33),
            flags: 0x1c00,
        }],
    };
    let mut resources = Vec::new();
    assert_eq!(spatial_resource(&native, &mut resources).unwrap(), 0);
    native.local_name = b"reader";
    native.payload[0].flags = 0;
    assert_eq!(spatial_resource(&native, &mut resources).unwrap(), 0);
    assert_eq!(resources.len(), 1);
    let mut conflict = native.clone();
    conflict.cell_size = Some(1.);
    assert!(spatial_resource(&conflict, &mut resources).is_err());
    let mut conflict = native.clone();
    conflict.payload[0].name = b"Other";
    assert!(spatial_resource(&conflict, &mut resources).is_err());
    let mut unsupported = native.clone();
    unsupported.payload[0].native_type = NativeType(34);
    assert!(spatial_resource(&unsupported, &mut resources).is_err());
    let mut unsupported = native.clone();
    unsupported.flags = Some(2);
    assert!(spatial_resource(&unsupported, &mut resources).is_err());
    native.name = b"Other/Keyed";
    assert_eq!(spatial_resource(&native, &mut resources).unwrap(), 1);
}
#[test]
fn original_native_spatial_schema_retains_sparse_payload_controls_and_checked_abi() {
    let document =
        Document::read(include_bytes!("../../../../tests/fixtures/pkb/spatial.pkb")).unwrap();
    let resources: Vec<_> = document
        .objects()
        .iter()
        .enumerate()
        .filter(|(_, object)| {
            document.strings()[document.types()[object.type_index as usize].name as usize]
                == b"CLayerCompileCacheSpatialLayer"
        })
        .map(|(index, _)| document.spatial_layer(index as u32 + 1).unwrap())
        .collect();
    assert_eq!(resources.len(), 3);
    assert_eq!(resources[0].payload[0].flags, 0x1c00);
    assert_eq!(resources[1].payload[0].flags, 0);
    assert_eq!(resources[0].name, resources[1].name);
    assert_ne!(resources[0].local_name, resources[1].local_name);
    let options = EffectCompileOptions::default();
    let lowering = Lowering {
        document: &document,
        options: &options,
        samplers: vec![],
        attributes: vec![],
        attribute_slots: HashMap::new(),
        curve_widths: HashMap::new(),
    };
    use ValueType::{Context, Float, Float3, Int};
    for (name, args, expected) in [
        (b"allocatePayload".as_slice(), vec![Context], Some(Int)),
        (
            b"appendPayload".as_slice(),
            vec![Int, Float3, Int, Context],
            Some(Int),
        ),
        (b"insert".as_slice(), vec![Int, Float3, Context], None),
        (
            b"closestF3".as_slice(),
            vec![Float3, Float, Int, Int, Int, Int, Context],
            Some(Float3),
        ),
    ] {
        assert_eq!(
            lowering
                .callback(name, &args, Some(0), &[External::Spatial(0)])
                .unwrap()
                .1,
            expected
        );
        assert!(lowering.callback(name, &args, None, &[]).is_err());
        assert!(lowering
            .callback(name, &args, Some(0), &[External::Sampler(0)])
            .is_err());
    }
}

const REDUNDANT: &[u8] = include_bytes!("../../../../tests/fixtures/pkb/redundant-stage.pkb");

#[test]
fn redundant_unknown_stage_does_not_change_selected_evolution_or_run_twice() {
    use crate::model::scene::popcorn::{EffectInstance, EffectLimits, SceneInputs};
    let document = Document::read(REDUNDANT).unwrap();
    let extra = document
        .compiler_blobs()
        .map(Result::unwrap)
        .find(|blob| blob.identifier == 97)
        .unwrap();
    let varying = document
        .compiler_blobs()
        .map(Result::unwrap)
        .find(|blob| blob.identifier == 5)
        .unwrap();
    assert_eq!(extra.bytes, varying.bytes);
    assert_ne!(extra.externals, varying.externals);
    assert_ne!(extra.functions, varying.functions);
    assert_ne!(extra.entrypoint, varying.entrypoint);
    assert!(same_programme(&document, &extra, &varying).unwrap());
    for (evolution, expected) in [
        (EffectEvolution::Physics, 5.),
        (EffectEvolution::Fixed, 5.),
        (EffectEvolution::Varying, 6.),
    ] {
        let program = document
            .compile_effect(
                1,
                &EffectCompileOptions {
                    evolution,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(program
            .limitations()
            .contains(&EffectLimitation::RedundantProgramme));
        let mut instance = program
            .instantiate(
                EffectLimits {
                    step_seconds: 0.01,
                    ..Default::default()
                },
                1,
            )
            .unwrap();
        instance.start(&program, 0, SceneInputs::default()).unwrap();
        let size = |instance: &EffectInstance| {
            let particle = instance.particles(EffectLayerId(0)).next().unwrap();
            f32::from_bits(particle.field(EffectFieldId(3)).unwrap().words()[0])
        };
        assert_eq!(size(&instance), 3.);
        instance
            .advance(&program, 0.01, SceneInputs::default())
            .unwrap();
        assert_eq!(size(&instance), expected);
    }
}

#[test]
fn unknown_nonduplicate_stage_still_rejects_with_its_native_identifier() {
    let document = Document::read(REDUNDANT).unwrap();
    let extra = document
        .compiler_blobs()
        .map(Result::unwrap)
        .find(|blob| blob.identifier == 97)
        .unwrap();
    let offset = extra.bytes.as_ptr() as usize - REDUNDANT.as_ptr() as usize;
    let mut changed = REDUNDANT.to_vec();
    // Change only the unknown programme's increment constant, leaving a complete
    // valid script and all metadata equal to its original supported counterpart.
    changed[offset + 36 + 64..offset + 36 + 68].copy_from_slice(&4f32.to_bits().to_le_bytes());
    let document = Document::read(&changed).unwrap();
    let failure = document
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap_err();
    assert!(failure
        .message
        .contains("unsupported native particle programme stage"));
    assert_eq!(failure.stage, Some(97));
}

#[test]
fn duplicate_admission_requires_resolved_external_function_and_entry_metadata() {
    let document = Document::read(REDUNDANT).unwrap();
    let extra = document
        .compiler_blobs()
        .map(Result::unwrap)
        .find(|blob| blob.identifier == 97)
        .unwrap();
    let varying = document
        .compiler_blobs()
        .map(Result::unwrap)
        .find(|blob| blob.identifier == 5)
        .unwrap();
    for field in 0..4 {
        let mut different = extra.clone();
        match field {
            0 => different.externals[0] = different.externals[1],
            1 => different.functions[0] = different.functions[1],
            2 => different.entrypoint = 0,
            _ => different.class_name = Some(u32::MAX),
        }
        assert!(!same_programme(&document, &different, &varying).unwrap());
    }
}
