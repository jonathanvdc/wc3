use wc3::model::animation::ValueKeyframe;
use wc3::model::animation::{Animatable, Track};
use wc3::model::chunks::{
    BindPoseChunk, MaterialsChunk, ModelChunk, RawChunk, UnknownChunk, VersionChunk,
};
use wc3::model::emitters::PopcornEmitter;
use wc3::model::geometry::{Geoset, SkinWeights};
use wc3::model::mdl::Read as _;
use wc3::model::mdl::Write as _;
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::materials::{Layer, LayerFresnel, LayerTextureSlot, Material, ShaderType};
use wc3::model::scene::{
    Camera, CameraVariant, Light, LightFalloff, LightShadowRange, Node, NodeFlagInterpretation,
    NodeFlags,
};
use wc3::model::{
    ConversionIssueKind, ConversionOptions, DynamicModel, Model, ModelVersion, UnknownChunkPolicy,
    V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900,
};

fn sample<V: ModelVersion>() -> Model<V> {
    let mut model = Model::<V>::new();
    let mut material = Material::<V>::new();
    let mut layer = Layer::<V>::new();
    if V::NUMBER >= 1100 {
        layer
            .try_set_texture_slots(&[LayerTextureSlot {
                texture_id: Animatable::Static(4),
                texture_type: 0,
            }])
            .unwrap();
    } else {
        layer.texture_id = Animatable::Static(4);
    }
    layer.alpha = Animatable::Static(0.75);
    material.priority_plane = 12;
    material.layers = [layer].to_vec();
    model.set_materials(&[material]);
    let mut geoset = Geoset::<V>::new(&[[1.0, 2.0, 3.0]], &[[0.0, 0.0, 1.0]], &[]).unwrap();
    geoset.set_raw_unselectable(0x8000_0002);
    model.set_geosets(&[geoset]);
    let mut light = Light::<V>::new(Node::new("Lamp", 7).unwrap(), 2);
    light.node.flags.set_light(true);
    light.color = Animatable::Static([1.0, 0.5, 0.25]);
    light.intensity = Animatable::Static(2.5);
    light.attenuation_end = Animatable::Static(200.0);
    model.set_lights(&[light]);
    model.set_cameras(&[Camera::<V>::new("View").unwrap()]);
    model
}

#[test]
fn missing_popcorn_kind_is_normalized_without_changing_source_or_behavior_flags() {
    let mut source =
        Model::<V1200>::decode_mdl("Version { FormatVersion 1200, } Model \"Minimal\" {}").unwrap();
    let mut emitter = PopcornEmitter::new(Node::new("Hero_Glow", 153).unwrap(), "", "").unwrap();
    emitter.node.flags.set_unfogged(true);
    source.try_set_popcorn_emitters(&[emitter]).unwrap();
    let bytes = source.encode_mdx().unwrap();
    assert!(source.encode_mdl().is_err());

    let converted = source.normalized().unwrap();
    assert_eq!(
        converted.model.try_popcorn_emitters().unwrap()[0]
            .node
            .flags
            .bits(),
        0x21000
    );
    assert_eq!(converted.report.issues.len(), 1);
    assert_eq!(
        converted.report.issues[0].kind,
        ConversionIssueKind::Normalized
    );
    assert_eq!(
        converted.report.issues[0].path,
        "chunks[2].popcorn_emitters[0].node.flags"
    );
    let text = converted.model.encode_mdl().unwrap();
    let restored = Model::<V1200>::decode_mdl(&text).unwrap();
    assert_eq!(
        restored.try_popcorn_emitters().unwrap(),
        converted.model.try_popcorn_emitters().unwrap()
    );
    assert_eq!(source.encode_mdx().unwrap(), bytes);
    assert!(converted
        .model
        .convert::<V1200>(&ConversionOptions::strict())
        .unwrap()
        .report
        .issues
        .is_empty());
}

#[test]
fn conflicting_popcorn_kind_is_preserved_and_still_refused_by_mdl() {
    let mut source = Model::<V1200>::new();
    let mut node = Node::new("Conflicting", 0).unwrap();
    node.flags.set_bone(true);
    let emitter = PopcornEmitter::new(node, "", "").unwrap();
    source.try_set_popcorn_emitters(&[emitter]).unwrap();
    let converted = source
        .convert::<V1200>(&ConversionOptions::strict())
        .unwrap();
    assert!(converted.report.issues.is_empty());
    let emitter = &converted.model.try_popcorn_emitters().unwrap()[0];
    assert_eq!(emitter.node.flags.bits(), 0x100);
    assert!(emitter.encode_mdl().is_err());
}

#[test]
fn kind_normalization_preserves_inheritance_and_unknown_bits() {
    let mut source = Model::<V1200>::new();
    let mut node = Node::new("UnknownFlags", 0).unwrap();
    node.flags = NodeFlags::from_bits(0x8000_0004);
    let emitter = PopcornEmitter::new(node, "", "").unwrap();
    source.try_set_popcorn_emitters(&[emitter]).unwrap();
    let converted = source
        .convert::<V1200>(&ConversionOptions::strict())
        .unwrap();
    let emitter = &converted.model.try_popcorn_emitters().unwrap()[0];
    assert_eq!(emitter.node.flags.bits(), 0x8000_1004);
    assert!(emitter.encode_mdl().is_err());
}

#[test]
fn runtime_normalization_preserves_each_version_and_matches_typed_normalization() {
    macro_rules! check {
        ($($version:ident),+) => {$(
            let mut source = sample::<$version>();
            let mut camera = Camera::<$version>::new("Noncanonical").unwrap();
            camera.variant = if $version::NUMBER >= 1200 {
                CameraVariant::Variant0
            } else {
                CameraVariant::Variant3
            };
            source.set_cameras(&[camera]);
            let original = source.encode_mdx().unwrap();
            let expected = source.normalized().unwrap();
            let dynamic = DynamicModel::$version(source);
            let normalized = dynamic.normalized().unwrap();
            assert_eq!(normalized.model.version(), $version::NUMBER);
            assert_eq!(normalized.model.encode_mdx().unwrap(), expected.model.encode_mdx().unwrap());
            assert_eq!(normalized.report, expected.report);
            assert!(normalized.report.issues.iter().any(|issue| issue.kind == ConversionIssueKind::Normalized));
            assert_eq!(dynamic.encode_mdx().unwrap(), original);
            assert!(normalized.model.normalized().unwrap().report.issues.is_empty());
        )+};
    }
    check!(V800, V900, V1000, V1100, V1200, V1300, V1400, V1600, V1800);
}

fn check_pair<S: ModelVersion, T: ModelVersion>() {
    let source = sample::<S>();
    let original = source.encode_mdx().unwrap();
    let converted = source.convert::<T>(&ConversionOptions::strict()).unwrap();
    assert_eq!(converted.model.version(), T::NUMBER);
    let bytes = converted.model.encode_mdx().unwrap();
    let decoded = Model::<T>::decode_mdx(&bytes).unwrap();
    assert_eq!(decoded.geosets()[0].raw_unselectable(), 0x8000_0002);
    let back = decoded.convert::<S>(&ConversionOptions::strict()).unwrap();
    assert_eq!(back.model.encode_mdx().unwrap(), original);
    assert_eq!(source.encode_mdx().unwrap(), original);
}

#[test]
fn default_records_convert_between_every_supported_version() {
    macro_rules! targets {
        ($source:ty) => {
            check_pair::<$source, V800>();
            check_pair::<$source, V900>();
            check_pair::<$source, V1000>();
            check_pair::<$source, V1100>();
            check_pair::<$source, V1200>();
            check_pair::<$source, V1300>();
            check_pair::<$source, V1400>();
            check_pair::<$source, V1600>();
            check_pair::<$source, V1800>();
        };
    }
    targets!(V800);
    targets!(V900);
    targets!(V1000);
    targets!(V1100);
    targets!(V1200);
    targets!(V1300);
    targets!(V1400);
    targets!(V1600);
    targets!(V1800);
}

#[test]
fn shader_transition_is_explicit_and_source_is_unchanged() {
    let mut material = Material::<V1000>::new();
    material.set_shader("Shaders\\Unit.shader").unwrap();
    let mut model = Model::<V1000>::new();
    model.set_materials(&[material]);
    let original = model.encode_mdx().unwrap();
    let error = model
        .convert::<V1100>(&ConversionOptions::strict())
        .unwrap_err();
    assert_eq!(error.path, "chunks[1].materials[0].shader");
    assert_eq!(error.source_version, 1000);
    assert_eq!(error.target_version, 1100);
    let result = model.convert::<V1100>(&ConversionOptions::lossy()).unwrap();
    assert!(result
        .report
        .issues
        .iter()
        .any(|issue| issue.path == error.path && issue.kind == ConversionIssueKind::Dropped));
    assert_eq!(model.encode_mdx().unwrap(), original);
    Model::<V1100>::decode_mdx(&result.model.encode_mdx().unwrap()).unwrap();
}

#[test]
fn texture_slots_including_animations_require_loss_permission() {
    let mut layer = Layer::<V1100>::new();
    layer.set_texture_slots(&[LayerTextureSlot {
        texture_id: Animatable::Both {
            value: 8,
            track: Track::<u32>::step(
                vec![ValueKeyframe {
                    frame: 20,
                    value: 9,
                }],
                None,
            )
            .unwrap(),
        },
        texture_type: 2,
    }]);
    assert!(layer
        .convert::<V1000>(&ConversionOptions::strict())
        .is_err());
    let result = layer.convert::<V1000>(&ConversionOptions::lossy()).unwrap();
    assert!(result
        .report
        .issues
        .iter()
        .any(|issue| issue.path == "record.texture_slots"
            && issue.kind == ConversionIssueKind::Dropped));
    assert_eq!(result.model.texture_id, layer.texture_id);
}

#[test]
fn tracks_are_checked_even_when_static_fields_are_neutral() {
    let mut layer = Layer::<V900>::new();
    let track = Track::<f32>::linear(
        vec![ValueKeyframe {
            frame: 100,
            value: 2.0,
        }],
        Some(3),
    )
    .unwrap();
    layer.set_emissive_gain(Animatable::Both { value: 1.0, track });
    let error = layer
        .convert::<V800>(&ConversionOptions::strict())
        .unwrap_err();
    assert_eq!(error.path, "record.emissive_gain");
    let lossy = layer.convert::<V800>(&ConversionOptions::lossy()).unwrap();
    assert!(lossy.model.try_emissive_gain().is_err());
    let mut light = Light::<V1800>::new(Node::new("Lamp", 1).unwrap(), 0);
    let mut falloff = light.falloff();
    falloff.damping.set_track(Track::constant(1.0));
    light.set_falloff(falloff);
    assert_eq!(
        light
            .convert::<V1400>(&ConversionOptions::strict())
            .unwrap_err()
            .path,
        "record.falloff"
    );
    assert!(light
        .convert::<V1400>(&ConversionOptions::lossy())
        .unwrap()
        .model
        .falloff()
        .damping
        .track()
        .is_none());
}

#[test]
fn skin_narrowing_checks_indices_and_never_truncates_them() {
    let mut geoset = Geoset::<V1800>::new(&[[0.0; 3]], &[[0.0; 3]], &[]).unwrap();
    geoset
        .set_skin_weights(Some(&[SkinWeights {
            bone_indices: [256, 0, 0, 0],
            weights: [255, 0, 0, 0],
        }]))
        .unwrap();
    geoset.set_tangents(Some(&[[1.0, 0.0, 0.0, 1.0]]));
    let error = geoset
        .convert::<V1300>(&ConversionOptions::strict())
        .unwrap_err();
    assert_eq!(error.path, "record.extra_sections[1]");
    let lossy = geoset
        .convert::<V1300>(&ConversionOptions::lossy())
        .unwrap();
    assert_eq!(lossy.model.skin_weights(), None);
    assert_eq!(lossy.model.tangents(), geoset.tangents());
    Geoset::<V1300>::decode_mdx(&lossy.model.encode_mdx().unwrap()).unwrap();
    let wide = geoset
        .convert::<V1400>(&ConversionOptions::strict())
        .unwrap();
    assert_eq!(wide.model.skin_weights(), geoset.skin_weights());
    geoset
        .set_skin_weights(Some(&[SkinWeights {
            bone_indices: [255, 0, 0, 0],
            weights: [255, 0, 0, 0],
        }]))
        .unwrap();
    let narrow = geoset
        .convert::<V900>(&ConversionOptions::strict())
        .unwrap();
    let decoded = Geoset::<V900>::decode_mdx(&narrow.model.encode_mdx().unwrap()).unwrap();
    assert_eq!(decoded.skin_weights(), geoset.skin_weights());
}

#[test]
fn opaque_chunks_have_an_independent_policy() {
    let mut model = Model::<V800>::new();
    model.chunks.push(ModelChunk::Unknown(
        UnknownChunk::new(RawChunk::new(*b"FUTR", vec![1, 2, 3])).unwrap(),
    ));
    assert!(model.convert::<V900>(&ConversionOptions::lossy()).is_err());
    let options = ConversionOptions {
        unknown_chunks: UnknownChunkPolicy::Preserve,
        ..ConversionOptions::strict()
    };
    let preserved = model.convert::<V900>(&options).unwrap();
    assert_eq!(
        preserved.report.issues[0].kind,
        ConversionIssueKind::PreservedUnknown
    );
    let ModelChunk::Unknown(chunk) = &preserved.model.chunks[1] else {
        panic!("expected opaque chunk")
    };
    assert_eq!(chunk.raw().data, [1, 2, 3]);
    let options = ConversionOptions {
        unknown_chunks: UnknownChunkPolicy::Drop,
        ..ConversionOptions::strict()
    };
    let dropped = model.convert::<V900>(&options).unwrap();
    assert_eq!(dropped.model.chunks.len(), 1);
    assert_eq!(dropped.report.issues[0].kind, ConversionIssueKind::Dropped);
    assert_eq!(
        model
            .convert::<V800>(&ConversionOptions::strict())
            .unwrap()
            .model
            .encode_mdx()
            .unwrap(),
        model.encode_mdx().unwrap()
    );
}

#[test]
fn unsupported_chunks_are_rejected_or_reported_and_removed() {
    let mut model = Model::<V900>::new();
    model.chunks.push(BindPoseChunk::new(vec![]).into());
    assert_eq!(
        model
            .convert::<V800>(&ConversionOptions::strict())
            .unwrap_err()
            .path,
        "chunks[1]"
    );
    let dropped = model.convert::<V800>(&ConversionOptions::lossy()).unwrap();
    assert_eq!(dropped.model.chunks.len(), 1);
    assert_eq!(dropped.report.issues[0].kind, ConversionIssueKind::Dropped);
}

#[test]
fn chunk_order_duplicates_version_extensions_and_raw_names_are_preserved() {
    let mut model = sample::<V900>();
    let mut version = VersionChunk::<V900>::new();
    version.extension = vec![9, 8, 7];
    model.chunks.push(version.into());
    model.chunks.push(model.chunks[1].clone());
    // Deliberately preserve non-UTF8 shader bytes and trailing padding.
    let mut material_bytes = Material::<V900>::new().encode_mdx().unwrap();
    material_bytes[12] = 255;
    material_bytes[91] = 42;
    let material = Material::<V900>::decode_mdx(&material_bytes).unwrap();
    model
        .chunks
        .push(MaterialsChunk::new(vec![material]).into());
    let converted = model
        .convert::<V1000>(&ConversionOptions::strict())
        .unwrap();
    assert_eq!(
        model.chunks.iter().map(ModelChunk::tag).collect::<Vec<_>>(),
        converted
            .model
            .chunks
            .iter()
            .map(ModelChunk::tag)
            .collect::<Vec<_>>()
    );
    let back = converted
        .model
        .convert::<V900>(&ConversionOptions::strict())
        .unwrap();
    assert_eq!(
        back.model.encode_mdx().unwrap(),
        model.encode_mdx().unwrap()
    );
}

#[test]
fn camera_variants_preserve_their_self_describing_layout_and_opaque_bytes() {
    for variant in [
        CameraVariant::Variant1([19; 12]),
        CameraVariant::Variant2([37; 12]),
        CameraVariant::Unknown(7),
    ] {
        let mut camera = Camera::<V1800>::new("Camera").unwrap();
        camera.variant = variant;
        let bytes = camera.encode_mdx().unwrap();
        for options in [ConversionOptions::strict(), ConversionOptions::lossy()] {
            let classic = camera.convert::<V800>(&options).unwrap();
            let modern = camera.convert::<V1800>(&options).unwrap();
            assert_eq!(classic.model.variant, variant);
            assert_eq!(modern.model.variant, variant);
            assert_eq!(classic.model.encode_mdx().unwrap(), bytes);
            assert_eq!(modern.model.encode_mdx().unwrap(), bytes);
            assert!(classic.report.issues.is_empty());
            assert!(modern.report.issues.is_empty());
        }
    }
}

#[test]
fn camera_conversion_normalizes_equivalent_variants_to_each_target_layout() {
    fn check<S: ModelVersion, T: ModelVersion>() {
        for variant in [CameraVariant::Variant0, CameraVariant::Variant3] {
            let mut camera = Camera::<S>::new("Animated Camera").unwrap();
            camera.variant = variant;
            camera.position = [-0.0, 2.0, 3.0];
            camera.target_position = [4.0, 5.0, 6.0];
            camera.field_of_view = 0.9;
            camera.focus_distance = Some(Track::constant(180.0));
            let original = camera.encode_mdx().unwrap();
            let converted = camera.convert::<T>(&ConversionOptions::strict()).unwrap();
            let expected = if T::NUMBER >= 1200 {
                CameraVariant::Variant3
            } else {
                CameraVariant::Variant0
            };
            assert_eq!(converted.model.variant, expected);
            // Only the high variant byte changes; all fields and tracks survive.
            let mut expected_bytes = original.clone();
            expected_bytes[3] = expected.value();
            assert_eq!(converted.model.encode_mdx().unwrap(), expected_bytes);
            assert_eq!(
                Camera::<T>::decode_mdx(&expected_bytes).unwrap().variant,
                expected
            );
            assert!(converted.model.encode_mdl().is_ok());
            assert_eq!(camera.encode_mdx().unwrap(), original);
            if variant == expected {
                assert!(converted.report.issues.is_empty());
            } else {
                assert_eq!(converted.report.issues.len(), 1);
                assert_eq!(converted.report.issues[0].path, "record.variant");
                assert_eq!(
                    converted.report.issues[0].kind,
                    ConversionIssueKind::Normalized
                );
            }
        }
    }
    macro_rules! targets {
        ($source:ty) => {
            check::<$source, V800>();
            check::<$source, V900>();
            check::<$source, V1000>();
            check::<$source, V1100>();
            check::<$source, V1200>();
            check::<$source, V1300>();
            check::<$source, V1400>();
            check::<$source, V1600>();
            check::<$source, V1800>();
        };
    }
    targets!(V800);
    targets!(V1800);
}

#[test]
fn model_camera_upgrade_downgrade_and_same_version_normalization_are_reported() {
    let mut source = Model::<V800>::new();
    source.set_cameras(&[Camera::<V800>::new("View").unwrap()]);
    let bytes = source.encode_mdx().unwrap();
    let upgraded = source
        .convert::<V1800>(&ConversionOptions::strict())
        .unwrap();
    assert_eq!(upgraded.model.cameras()[0].variant, CameraVariant::Variant3);
    assert_eq!(
        upgraded.report.issues[0].path,
        "chunks[1].cameras[0].variant"
    );
    assert_eq!(
        upgraded.report.issues[0].kind,
        ConversionIssueKind::Normalized
    );
    let downgraded = upgraded
        .model
        .convert::<V800>(&ConversionOptions::strict())
        .unwrap();
    assert_eq!(
        downgraded.model.cameras()[0].variant,
        CameraVariant::Variant0
    );
    assert_eq!(downgraded.model.encode_mdx().unwrap(), bytes);
    assert_eq!(
        downgraded.report.issues[0].kind,
        ConversionIssueKind::Normalized
    );
    assert_eq!(source.encode_mdx().unwrap(), bytes);

    let mut noncanonical = Model::<V1800>::new();
    let mut camera = Camera::<V1800>::new("Portrait").unwrap();
    camera.variant = CameraVariant::Variant0;
    noncanonical.set_cameras(&[camera]);
    let result = DynamicModel::V1800(noncanonical)
        .convert::<V1800>(&ConversionOptions::strict())
        .unwrap();
    assert_eq!(result.model.cameras()[0].variant, CameraVariant::Variant3);
    assert_eq!(
        result.report.issues[0].kind,
        ConversionIssueKind::Normalized
    );
}

#[test]
fn absent_version_is_inserted_and_dynamic_sources_convert() {
    let source = Model::<V800>::decode_mdx(b"MDLX").unwrap();
    let converted = source
        .convert::<V900>(&ConversionOptions::strict())
        .unwrap();
    assert!(matches!(
        converted.model.chunk(*b"VERS"),
        Some(ModelChunk::Version(_))
    ));
    assert_eq!(converted.model.version(), 900);
    assert_eq!(converted.report.issues[0].path, "VERS");
    let dynamic = DynamicModel::V800(sample::<V800>());
    let converted = dynamic
        .convert::<V1800>(&ConversionOptions::strict())
        .unwrap();
    assert_eq!(converted.model.version(), 1800);
    assert!(converted
        .report
        .issues
        .iter()
        .any(|issue| issue.kind == ConversionIssueKind::Initialized));
}

#[test]
fn populated_versioned_fields_preserve_exact_storage_when_supported() {
    let mut layer = Layer::<V1800>::new();
    layer.set_emissive_gain(Animatable::Static(2.5));
    layer.set_fresnel(LayerFresnel {
        color: Animatable::Static([0.25, 0.5, 1.0]),
        opacity: Animatable::Static(0.75),
        team_color: Animatable::Static(0.25),
    });
    layer.set_shader_type(ShaderType::new(3));
    layer.set_texture_slots(&[LayerTextureSlot {
        texture_id: Animatable::Both {
            value: 2,
            track: Track::<u32>::linear(
                vec![ValueKeyframe {
                    frame: 200,
                    value: 5,
                }],
                Some(2),
            )
            .unwrap(),
        },
        texture_type: 1,
    }]);
    let converted = layer
        .convert::<V1600>(&ConversionOptions::strict())
        .unwrap();
    assert!(converted.report.issues.is_empty());
    assert_eq!(
        converted.model.encode_mdx().unwrap(),
        layer.encode_mdx().unwrap()
    );

    let mut light = Light::<V1800>::new(Node::new("Shadow", 2).unwrap(), 1);
    light.set_shadow_intensity(0.5);
    light.set_shadow_casting_range(LightShadowRange {
        start: Animatable::Static(1.0),
        end: Animatable::Static(10.0),
    });
    light.set_falloff(LightFalloff {
        quadratic: Animatable::Static(1.0),
        linear: Animatable::Static(2.0),
        damping: Animatable::Static(3.0),
    });
    // A noncanonical nonzero shadow flag must retain its original bits.
    let shadow_offset = 4 + light.node.encode_mdx().unwrap().len() + 4;
    let mut bytes = light.encode_mdx().unwrap();
    bytes[shadow_offset..shadow_offset + 4].copy_from_slice(&0x8000_0002u32.to_le_bytes());
    let light = Light::<V1800>::decode_mdx(&bytes).unwrap();
    let converted = light
        .convert::<V1600>(&ConversionOptions::strict())
        .unwrap();
    assert_eq!(converted.model.encode_mdx().unwrap(), bytes);
    assert!(converted.report.issues.is_empty());
    assert_eq!(
        light
            .convert::<V1400>(&ConversionOptions::strict())
            .unwrap_err()
            .path,
        "record.falloff"
    );
    assert_eq!(
        light
            .convert::<V1200>(&ConversionOptions::strict())
            .unwrap_err()
            .path,
        "record.shadow_casting"
    );
}

#[test]
fn strict_omission_of_neutral_defaults_is_reported() {
    let layer = Layer::<V900>::new();
    let converted = layer.convert::<V800>(&ConversionOptions::strict()).unwrap();
    assert_eq!(converted.report.issues.len(), 1);
    assert_eq!(
        converted.report.issues[0].kind,
        ConversionIssueKind::OmittedDefault
    );
    assert_eq!(converted.report.issues[0].path, "record.emissive_gain");
}
