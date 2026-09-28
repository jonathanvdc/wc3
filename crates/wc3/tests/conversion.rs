use wc3::model::animation::{
    AnimationTrack, LayerEmissiveGain, LayerTextureId, LightDamping, ValueKeyframe,
};
use wc3::model::chunks::{
    BindPoseChunk, MaterialsChunk, ModelChunk, RawChunk, UnknownChunk, VersionChunk,
};
use wc3::model::geometry::{Geoset, SkinWeights};
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::materials::{Layer, LayerFresnel, LayerTextureSlot, Material};
use wc3::model::scene::{Camera, CameraVariant, Light, LightFalloff, LightShadowRange, Node};
use wc3::model::{
    ConversionIssueKind, ConversionOptions, DynamicModel, Model, ModelVersion, UnknownChunkPolicy,
    V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900,
};

fn sample<V: ModelVersion>() -> Model<V> {
    let mut model = Model::<V>::new();
    let mut material = Material::<V>::new();
    let mut layer = Layer::<V>::new();
    layer.set_texture_id(4);
    layer.set_alpha(0.75);
    material.set_priority_plane(12);
    material.set_layers(&[layer]);
    model.set_materials(&[material]);
    let mut geoset = Geoset::<V>::new(&[[1.0, 2.0, 3.0]], &[[0.0, 0.0, 1.0]], &[]).unwrap();
    geoset.set_raw_unselectable(0x8000_0002);
    model.set_geosets(&[geoset]);
    let mut light = Light::<V>::new(Node::new("Lamp", 7).unwrap(), 2);
    light.set_color([1.0, 0.5, 0.25]);
    light.set_intensity(2.5);
    light.set_attenuation_end(200.0);
    model.set_lights(&[light]);
    model.set_cameras(&[Camera::<V>::new("View").unwrap()]);
    model
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
        texture_id: 8,
        texture_type: 2,
        track: Some(
            AnimationTrack::<LayerTextureId>::step(
                vec![ValueKeyframe {
                    frame: 20,
                    value: 9,
                }],
                None,
            )
            .unwrap(),
        ),
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
    assert_eq!(result.model.texture_id(), layer.texture_id());
}

#[test]
fn tracks_are_checked_even_when_static_fields_are_neutral() {
    let mut layer = Layer::<V900>::new();
    let track = AnimationTrack::<LayerEmissiveGain>::linear(
        vec![ValueKeyframe {
            frame: 100,
            value: 2.0,
        }],
        Some(3),
    )
    .unwrap()
    .into();
    layer.set_tracks(&[track]);
    let error = layer
        .convert::<V800>(&ConversionOptions::strict())
        .unwrap_err();
    assert_eq!(error.path, "record.tracks[0]");
    let lossy = layer.convert::<V800>(&ConversionOptions::lossy()).unwrap();
    assert!(lossy.model.tracks().is_empty());
    let mut light = Light::<V1800>::new(Node::new("Lamp", 1).unwrap(), 0);
    light.set_tracks(&[AnimationTrack::<LightDamping>::step(
        vec![ValueKeyframe {
            frame: 0,
            value: 1.0,
        }],
        None,
    )
    .unwrap()
    .into()]);
    assert_eq!(
        light
            .convert::<V1400>(&ConversionOptions::strict())
            .unwrap_err()
            .path,
        "record.tracks[0]"
    );
    assert!(light
        .convert::<V1400>(&ConversionOptions::lossy())
        .unwrap()
        .model
        .tracks()
        .is_empty());
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
    model.push(ModelChunk::Unknown(
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
    let ModelChunk::Unknown(chunk) = &preserved.model.chunks()[1] else {
        panic!("expected opaque chunk")
    };
    assert_eq!(chunk.raw().data, [1, 2, 3]);
    let options = ConversionOptions {
        unknown_chunks: UnknownChunkPolicy::Drop,
        ..ConversionOptions::strict()
    };
    let dropped = model.convert::<V900>(&options).unwrap();
    assert_eq!(dropped.model.chunks().len(), 1);
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
    model.push(BindPoseChunk::new(vec![]).into());
    assert_eq!(
        model
            .convert::<V800>(&ConversionOptions::strict())
            .unwrap_err()
            .path,
        "chunks[1]"
    );
    let dropped = model.convert::<V800>(&ConversionOptions::lossy()).unwrap();
    assert_eq!(dropped.model.chunks().len(), 1);
    assert_eq!(dropped.report.issues[0].kind, ConversionIssueKind::Dropped);
}

#[test]
fn chunk_order_duplicates_version_extensions_and_raw_names_are_preserved() {
    let mut model = sample::<V900>();
    let mut version = VersionChunk::<V900>::new();
    version.extension = vec![9, 8, 7];
    model.push(version.into());
    model.push(model.chunks()[1].clone());
    // Deliberately preserve non-UTF8 shader bytes and trailing padding.
    let mut material_bytes = Material::<V900>::new().encode_mdx().unwrap();
    material_bytes[12] = 255;
    material_bytes[91] = 42;
    let material = Material::<V900>::decode_mdx(&material_bytes).unwrap();
    model.push(MaterialsChunk::new(vec![material]).into());
    let converted = model
        .convert::<V1000>(&ConversionOptions::strict())
        .unwrap();
    assert_eq!(
        model
            .chunks()
            .iter()
            .map(ModelChunk::tag)
            .collect::<Vec<_>>(),
        converted
            .model
            .chunks()
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
    let mut camera = Camera::<V1800>::new("Camera").unwrap();
    camera.set_variant(CameraVariant::Variant2([37; 12]));
    let result = camera
        .convert::<V800>(&ConversionOptions::strict())
        .unwrap();
    assert_eq!(
        result.model.encode_mdx().unwrap(),
        camera.encode_mdx().unwrap()
    );
}

#[test]
fn absent_version_is_inserted_and_dynamic_sources_convert() {
    let source = Model::<V800>::decode_mdx(b"MDLX").unwrap();
    let converted = source
        .convert::<V900>(&ConversionOptions::strict())
        .unwrap();
    assert_eq!(converted.model.stored_version(), Some(900));
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
    layer.set_emissive_gain(2.5);
    layer.set_fresnel(LayerFresnel {
        color: [0.25, 0.5, 1.0],
        opacity: 0.75,
        team_color: 0.25,
    });
    layer.set_shader_type_id(3);
    layer.set_texture_slots(&[LayerTextureSlot {
        texture_id: 2,
        texture_type: 1,
        track: Some(
            AnimationTrack::<LayerTextureId>::linear(
                vec![ValueKeyframe {
                    frame: 200,
                    value: 5,
                }],
                Some(2),
            )
            .unwrap(),
        ),
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
        start: 1.0,
        end: 10.0,
    });
    light.set_falloff(LightFalloff {
        quadratic: 1.0,
        linear: 2.0,
        damping: 3.0,
    });
    // A noncanonical nonzero shadow flag must retain its original bits.
    let shadow_offset = 4 + light.node().encode_mdx().unwrap().len() + 4;
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
