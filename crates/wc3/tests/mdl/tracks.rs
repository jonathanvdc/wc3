use wc3::model::animation::{Interpolation, TangentKeyframe, TextureAnimation, ValueKeyframe};
use wc3::model::animation::{Track, TrackValue};
use wc3::model::mdl::{self, Read as _, ReadErrorKind, Write as _};
use wc3::model::{Vec3, Vec4};

fn roundtrip<K>(value: Track<K>)
where
    K: TrackValue + mdl::Read + mdl::Write,
{
    let text = value.encode_mdl().unwrap();
    let decoded = Track::<K>::decode_mdl(&text).unwrap();
    assert_eq!(decoded.interpolation(), value.interpolation());
    assert_eq!(decoded.global_sequence_id(), value.global_sequence_id());
    assert_eq!(decoded.step_keys(), value.step_keys());
    assert_eq!(decoded.linear_keys(), value.linear_keys());
    assert_eq!(decoded.hermite_keys(), value.hermite_keys());
    assert_eq!(decoded.bezier_keys(), value.bezier_keys());
}

#[test]
fn all_modes_and_value_shapes_roundtrip() {
    roundtrip(
        Track::<f32>::step(
            vec![ValueKeyframe {
                frame: 0,
                value: -0.0,
            }],
            None,
        )
        .unwrap(),
    );
    roundtrip(
        Track::<u32>::linear(
            vec![ValueKeyframe {
                frame: i32::MAX,
                value: 3,
            }],
            Some(7),
        )
        .unwrap(),
    );
    roundtrip(
        Track::<Vec3>::hermite(
            vec![TangentKeyframe {
                frame: 100,
                value: [1.0, 2.0, 3.0],
                in_tangent: [0.0; 3],
                out_tangent: [4.0; 3],
            }],
            None,
        )
        .unwrap(),
    );
    roundtrip(
        Track::<Vec4>::bezier(
            vec![TangentKeyframe {
                frame: 200,
                value: [0.0, 0.0, 0.0, 1.0],
                in_tangent: [0.0; 4],
                out_tangent: [1.0; 4],
            }],
            Some(0),
        )
        .unwrap(),
    );
    roundtrip(Track::<f32>::linear(Vec::new(), None).unwrap());
}

#[test]
fn canonical_tangent_output() {
    let track = Track::<f32>::decode_mdl(
        "Track 1 { Bezier, GlobalSeqId 0, 10: 0.5, InTan 0.25, OutTan 0.75, }",
    )
    .unwrap();
    assert_eq!(
        track.encode_mdl().unwrap(),
        "Track 1 {\n\tBezier,\n\tGlobalSeqId 0,\n\t10: 0.5,\n\t\tInTan 0.25,\n\t\tOutTan 0.75,\n}\n"
    );
    assert_eq!(
        Track::<f32>::decode_mdl("Track 0 { GlobalSeqId 4, DontInterp, }")
            .unwrap()
            .interpolation(),
        Interpolation::Step
    );
}

#[test]
fn rejects_malformed_tracks() {
    for source in [
        "Track 0 { }",
        "Track 0 { Linear, Linear, }",
        "Track 0 { Linear, Bezier, }",
        "Track 0 { Linear, GlobalSeqId 0, GlobalSeqId 1, }",
        "Track 0 { Linear, GlobalSeqId 4294967295, }",
        "Track 0 { Unknown, }",
        "Track 1 { Hermite, 0: 1, }",
        "Track 1 { Bezier, 0: 1, InTan 1, }",
        "Track 1 { Linear, 0: 1, InTan 1, OutTan 1, }",
        "Track 1 { Linear, 0 1, }",
        "Track 1 { Linear, 0: 1 }",
        "Track 0 { Linear,",
    ] {
        assert!(Track::<f32>::decode_mdl(source).is_err(), "{source}");
    }
    for (source, expected, actual) in [
        ("Track 1 { Linear, }", 1, 0),
        ("Track 0 { Linear, 0: 1, }", 0, 1),
        ("Track 4294967295 { Linear, }", u32::MAX as usize, 0),
    ] {
        assert_eq!(
            Track::<f32>::decode_mdl(source).unwrap_err().kind,
            ReadErrorKind::CountMismatch { expected, actual }
        );
    }
}

#[test]
fn texture_animation_dispatch_and_roundtrip() {
    let source = "TVertexAnim { Translation 1 { Linear, 0: { 1, 2, 3 }, } Rotation 0 { DontInterp, } Scaling 0 { Hermite, } }";
    let animation = TextureAnimation::decode_mdl(source).unwrap();
    assert!(animation.translation.is_some());
    assert!(animation.rotation.is_some());
    assert!(animation.scaling.is_some());
    assert_eq!(
        TextureAnimation::decode_mdl(&animation.encode_mdl().unwrap()).unwrap(),
        animation
    );
    assert_eq!(
        TextureAnimation::new().encode_mdl().unwrap(),
        "TVertexAnim {\n}\n"
    );
    assert!(TextureAnimation::decode_mdl("TVertexAnim { Track 0 { Linear, } }").is_err());
    assert!(TextureAnimation::decode_mdl(
        "TVertexAnim { Translation 0 { Linear, } Translation 0 { Linear, } }"
    )
    .is_err());
}

#[test]
fn signed_frames_preserve_mdx_bits_and_mdl_spelling() {
    use wc3::model::mdx::{Read as _, Write as _};
    for frame in [i32::MIN, -3600, -1, 0, i32::MAX] {
        let source = format!("Track 1 {{ Linear, {frame}: 0.5, }}");
        let track = Track::<f32>::decode_mdl(&source).unwrap();
        assert_eq!(track.linear_keys().unwrap()[0].frame, frame);
        assert!(track.encode_mdl().unwrap().contains(&format!("{frame}:")));
        let bytes = track.encode_mdx().unwrap();
        assert_eq!(&bytes[12..16], &frame.to_le_bytes());
        let decoded = Track::<f32>::decode_mdx(&bytes).unwrap();
        assert_eq!(decoded, track);
    }
    for frame in ["-2147483649", "2147483648", "4294967295"] {
        assert!(Track::<f32>::decode_mdl(&format!("Track 1 {{ Linear, {frame}: 0.5, }}")).is_err());
    }
    let tangent = Track::<Vec4>::decode_mdl(
        "Track 1 { Hermite, -3600: { 0, 0, 0, 1 }, InTan { 0, 0, 0, 1 }, OutTan { 0, 0, 0, 1 }, }",
    )
    .unwrap();
    assert_eq!(tangent.hermite_keys().unwrap()[0].frame, -3600);
    roundtrip(tangent);
}

#[test]
fn spec_property_spellings_and_camera_target_context() {
    use wc3::model::emitters::ParticleEmitter;
    use wc3::model::scene::{Camera, Light};
    use wc3::model::V800;
    let light = Light::<V800>::decode_mdl(
        "Light \"L\" { ObjectId 0, Ambient, AmbColor 0 { Linear, } AmbIntensity 0 { Linear, } }",
    )
    .unwrap();
    let text = light.encode_mdl().unwrap();
    assert!(text.contains("AmbColor 0"));
    assert!(text.contains("AmbIntensity 0"));
    let particle = ParticleEmitter::decode_mdl(
        "ParticleEmitter \"P\" { ObjectId 0, InitVelocity 0 { Linear, } }",
    )
    .unwrap();
    assert!(particle.encode_mdl().unwrap().contains("InitVelocity 0"));
    assert!(ParticleEmitter::decode_mdl(
        "ParticleEmitter \"P\" { ObjectId 0, Speed 0 { Linear, } }"
    )
    .is_err());
    let camera = Camera::<V800>::decode_mdl("Camera \"C\" { FieldOfView 1, FarClip 2, Translation 1 { Linear, 0: { 1, 2, 3 }, } Target { Translation 1 { Linear, 0: { 4, 5, 6 }, } } }").unwrap();
    assert_ne!(camera.translation, camera.target_translation);
    assert_eq!(
        Camera::<V800>::decode_mdl(&camera.encode_mdl().unwrap()).unwrap(),
        camera
    );
}

#[test]
fn texture_anims_spec_container_transcodes_without_losing_tracks() {
    use wc3::model::mdl::{Parser, Writer};
    use wc3::model::mdx::{Read as _, Write as _};
    let source = r#"TextureAnims 2 {
        TVertexAnim {
            Translation 2 {
                Linear,
                0: { 0.0, 0.0, 0.0 },
                1000: { 1.0, 0.0, 0.0 },
            }
        }
        TVertexAnim {
            Scaling 1 { DontInterp, -3600: { 1.0, 2.0, 3.0 }, }
            Rotation 1 {
                Hermite,
                GlobalSeqId 0,
                -3600: { 0.0, 0.0, 0.0, 1.0 },
                    InTan { 0.0, 0.0, 0.0, 1.0 },
                    OutTan { 0.0, 0.0, 0.0, 1.0 },
            }
            Translation 0 { Bezier, }
        }
    }"#;
    let mut parser = Parser::new(source);
    parser.expect_ident("TextureAnims").unwrap();
    let animations = parser
        .counted::<TextureAnimation>()
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    parser.finish().unwrap();
    let mut writer = Writer::new(Vec::new());
    writer.counted("TextureAnims", animations.iter()).unwrap();
    let output = String::from_utf8(writer.finish().unwrap()).unwrap();
    assert!(output.contains("\t\t\t\tInTan"));
    let mut parser = Parser::new(&output);
    parser.expect_ident("TextureAnims").unwrap();
    let decoded = parser
        .counted::<TextureAnimation>()
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    parser.finish().unwrap();
    assert_eq!(decoded, animations);
    for animation in animations {
        let bytes = animation.encode_mdx().unwrap();
        let binary = TextureAnimation::decode_mdx(&bytes).unwrap();
        let text = binary.encode_mdl().unwrap();
        let decoded = TextureAnimation::decode_mdl(&text).unwrap();
        assert_eq!(decoded.encode_mdx().unwrap(), bytes);
    }
}

#[test]
fn texture_animation_has_no_static_transform_fields() {
    for property in [
        "static Translation { 0.0, 0.0, 0.0 },",
        "static Track { 0.0, 0.0, 0.0, 1.0 },",
        "static Scaling { 1.0, 1.0, 1.0 },",
    ] {
        let source = format!("TVertexAnim {{ {property} }}");
        let error = TextureAnimation::decode_mdl(&source).unwrap_err();
        assert_eq!(error.kind, ReadErrorKind::UnknownField);
        assert_eq!(&source[error.span.start..error.span.end], "static");
    }
    assert!(TextureAnimation::decode_mdl("TVertexAnim { }")
        .unwrap()
        .translation
        .is_none());
    for name in ["Translation", "Rotation", "Scaling"] {
        let source = format!("TVertexAnim {{ {name} 0 {{ Linear, }} {name} 0 {{ Linear, }} }}");
        assert_eq!(
            TextureAnimation::decode_mdl(&source).unwrap_err().kind,
            ReadErrorKind::DuplicateField
        );
    }
}
