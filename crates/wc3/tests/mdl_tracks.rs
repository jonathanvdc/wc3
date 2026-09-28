use wc3::model::animation::{
    AnimationTrack, Interpolation, LayerAlpha, LayerTextureId, NodeRotation, NodeTranslation,
    TangentKeyframe, TextureAnimation, TextureAnimationTrack, TextureTranslation, TrackKind,
    ValueKeyframe,
};
use wc3::model::mdl::{self, Read as _, ReadErrorKind, Write as _};

fn roundtrip<K: TrackKind>(value: AnimationTrack<K>)
where
    K::Value: mdl::Read + mdl::Write,
{
    let text = value.encode_mdl().unwrap();
    let decoded = AnimationTrack::<K>::decode_mdl(&text).unwrap();
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
        AnimationTrack::<LayerAlpha>::step(
            vec![ValueKeyframe {
                frame: 0,
                value: -0.0,
            }],
            None,
        )
        .unwrap(),
    );
    roundtrip(
        AnimationTrack::<LayerTextureId>::linear(
            vec![ValueKeyframe {
                frame: i32::MAX,
                value: 3,
            }],
            Some(7),
        )
        .unwrap(),
    );
    roundtrip(
        AnimationTrack::<NodeTranslation>::hermite(
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
        AnimationTrack::<NodeRotation>::bezier(
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
    roundtrip(AnimationTrack::<LayerAlpha>::linear(Vec::new(), None).unwrap());
}

#[test]
fn canonical_tangent_output() {
    let track = AnimationTrack::<LayerAlpha>::decode_mdl(
        "Alpha 1 { Bezier, GlobalSeqId 0, 10: 0.5, InTan 0.25, OutTan 0.75, }",
    )
    .unwrap();
    assert_eq!(
        track.encode_mdl().unwrap(),
        "Alpha 1 {\n\tBezier,\n\tGlobalSeqId 0,\n\t10: 0.5,\n\t\tInTan 0.25,\n\t\tOutTan 0.75,\n}\n"
    );
    assert_eq!(
        AnimationTrack::<LayerAlpha>::decode_mdl("Alpha 0 { GlobalSeqId 4, DontInterp, }")
            .unwrap()
            .interpolation(),
        Interpolation::Step
    );
}

#[test]
fn rejects_malformed_tracks() {
    for source in [
        "Alpha 0 { }",
        "Alpha 0 { Linear, Linear, }",
        "Alpha 0 { Linear, Bezier, }",
        "Alpha 0 { Linear, GlobalSeqId 0, GlobalSeqId 1, }",
        "Alpha 0 { Linear, GlobalSeqId 4294967295, }",
        "Alpha 0 { Unknown, }",
        "Alpha 1 { Hermite, 0: 1, }",
        "Alpha 1 { Bezier, 0: 1, InTan 1, }",
        "Alpha 1 { Linear, 0: 1, InTan 1, OutTan 1, }",
        "Alpha 1 { Linear, 0 1, }",
        "Alpha 1 { Linear, 0: 1 }",
        "Alpha 0 { Linear,",
    ] {
        assert!(
            AnimationTrack::<LayerAlpha>::decode_mdl(source).is_err(),
            "{source}"
        );
    }
    for (source, expected, actual) in [
        ("Alpha 1 { Linear, }", 1, 0),
        ("Alpha 0 { Linear, 0: 1, }", 0, 1),
        ("Alpha 4294967295 { Linear, }", u32::MAX as usize, 0),
    ] {
        assert_eq!(
            AnimationTrack::<LayerAlpha>::decode_mdl(source)
                .unwrap_err()
                .kind,
            ReadErrorKind::CountMismatch { expected, actual }
        );
    }
}

#[test]
fn texture_animation_dispatch_and_roundtrip() {
    let source = "TVertexAnim { Translation 1 { Linear, 0: { 1, 2, 3 }, } Rotation 0 { DontInterp, } Scaling 0 { Hermite, } }";
    let animation = TextureAnimation::decode_mdl(source).unwrap();
    assert!(matches!(
        animation.tracks[0],
        TextureAnimationTrack::Translation(_)
    ));
    assert!(matches!(
        animation.tracks[1],
        TextureAnimationTrack::Rotation(_)
    ));
    assert!(matches!(
        animation.tracks[2],
        TextureAnimationTrack::Scaling(_)
    ));
    assert_eq!(
        TextureAnimation::decode_mdl(&animation.encode_mdl().unwrap()).unwrap(),
        animation
    );
    assert_eq!(
        TextureAnimation::new().encode_mdl().unwrap(),
        "TVertexAnim {\n}\n"
    );
    assert!(TextureAnimation::decode_mdl("TVertexAnim { Alpha 0 { Linear, } }").is_err());
    assert!(TextureAnimation::decode_mdl(
        "TVertexAnim { Translation 0 { Linear, } Translation 0 { Linear, } }"
    )
    .is_err());
    let track = AnimationTrack::<TextureTranslation>::step(Vec::new(), None).unwrap();
    let mut duplicate = TextureAnimation::new();
    duplicate.tracks = (&[track.clone().into(), track.into()]).to_vec();
    assert!(duplicate.encode_mdl().is_err());
}

#[test]
fn signed_frames_preserve_mdx_bits_and_mdl_spelling() {
    use wc3::model::mdx::{Read as _, Write as _};
    for frame in [i32::MIN, -3600, -1, 0, i32::MAX] {
        let source = format!("Alpha 1 {{ Linear, {frame}: 0.5, }}");
        let track = AnimationTrack::<LayerAlpha>::decode_mdl(&source).unwrap();
        assert_eq!(track.linear_keys().unwrap()[0].frame, frame);
        assert!(track.encode_mdl().unwrap().contains(&format!("{frame}:")));
        let bytes = track.encode_mdx().unwrap();
        assert_eq!(&bytes[16..20], &frame.to_le_bytes());
        let decoded = AnimationTrack::<LayerAlpha>::decode_mdx(&bytes).unwrap();
        assert_eq!(decoded, track);
    }
    for frame in ["-2147483649", "2147483648", "4294967295"] {
        assert!(AnimationTrack::<LayerAlpha>::decode_mdl(&format!(
            "Alpha 1 {{ Linear, {frame}: 0.5, }}"
        ))
        .is_err());
    }
    let tangent = AnimationTrack::<NodeRotation>::decode_mdl("Rotation 1 { Hermite, -3600: { 0, 0, 0, 1 }, InTan { 0, 0, 0, 1 }, OutTan { 0, 0, 0, 1 }, }").unwrap();
    assert_eq!(tangent.hermite_keys().unwrap()[0].frame, -3600);
    roundtrip(tangent);
}

#[test]
fn spec_property_spellings() {
    use wc3::model::animation::{LightAmbientColor, LightAmbientIntensity, ParticleSpeed};
    let color = AnimationTrack::<LightAmbientColor>::decode_mdl("AmbColor 0 { Linear, }").unwrap();
    assert!(color.encode_mdl().unwrap().starts_with("AmbColor "));
    let intensity =
        AnimationTrack::<LightAmbientIntensity>::decode_mdl("AmbIntensity 0 { Linear, }").unwrap();
    assert!(intensity.encode_mdl().unwrap().starts_with("AmbIntensity "));
    let velocity =
        AnimationTrack::<ParticleSpeed>::decode_mdl("InitVelocity 0 { Linear, }").unwrap();
    assert!(velocity.encode_mdl().unwrap().starts_with("InitVelocity "));
    assert!(AnimationTrack::<LightAmbientColor>::decode_mdl("AmbientColor 0 { Linear, }").is_err());
    assert!(
        AnimationTrack::<LightAmbientIntensity>::decode_mdl("AmbientIntensity 0 { Linear, }")
            .is_err()
    );
    assert!(AnimationTrack::<ParticleSpeed>::decode_mdl("Speed 0 { Linear, }").is_err());
}

#[test]
fn camera_translation_dispatch_requires_context() {
    use wc3::model::mdl::{MdlWriter, Parser};
    use wc3::model::scene::CameraTrack;
    let source = "Translation 1 { Linear, -3600: { 1, 2, 3 }, }";
    let eye = CameraTrack::decode_mdl(source).unwrap();
    assert!(matches!(eye, CameraTrack::Translation(_)));
    assert_eq!(
        CameraTrack::decode_mdl(&eye.encode_mdl().unwrap()).unwrap(),
        eye
    );
    let mut parser = Parser::new(source);
    let target = CameraTrack::read_mdl_target(&mut parser).unwrap();
    parser.finish().unwrap();
    assert!(matches!(target, CameraTrack::TargetTranslation(_)));
    assert!(target.encode_mdl().is_err());
    let mut writer = MdlWriter::new(Vec::new());
    writer.begin_block("Target").unwrap();
    writer.property("Position", &[0.0f32; 3]).unwrap();
    target.write_mdl_target(&mut writer).unwrap();
    writer.end_block().unwrap();
    let text = String::from_utf8(writer.finish().unwrap()).unwrap();
    let mut parser = Parser::new(&text);
    parser.expect_ident("Target").unwrap();
    let mut block = parser.begin_block().unwrap();
    assert_eq!(block.next_field().unwrap().unwrap().name, "Position");
    assert_eq!(block.read_property::<[f32; 3]>().unwrap(), [0.0; 3]);
    assert_eq!(CameraTrack::read_mdl_target(&mut block).unwrap(), target);
    block.finish().unwrap();
    parser.finish().unwrap();
    let mut writer = MdlWriter::new(Vec::new());
    assert!(eye.write_mdl_target(&mut writer).is_err());
}

#[test]
fn texture_anims_spec_container_transcodes_without_losing_tracks() {
    use wc3::model::mdl::{MdlWriter, Parser};
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
    let mut writer = MdlWriter::new(Vec::new());
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
        assert_eq!(
            TextureAnimation::decode_mdl(&text)
                .unwrap()
                .encode_mdx()
                .unwrap(),
            bytes
        );
    }
}

#[test]
fn texture_animation_has_no_static_transform_fields() {
    for property in [
        "static Translation { 0.0, 0.0, 0.0 },",
        "static Rotation { 0.0, 0.0, 0.0, 1.0 },",
        "static Scaling { 1.0, 1.0, 1.0 },",
    ] {
        let source = format!("TVertexAnim {{ {property} }}");
        let error = TextureAnimation::decode_mdl(&source).unwrap_err();
        assert_eq!(error.kind, ReadErrorKind::UnknownField);
        assert_eq!(&source[error.span.start..error.span.end], "static");
    }
    assert!(TextureAnimation::decode_mdl("TVertexAnim { }")
        .unwrap()
        .tracks
        .is_empty());
    for name in ["Translation", "Rotation", "Scaling"] {
        let source = format!("TVertexAnim {{ {name} 0 {{ Linear, }} {name} 0 {{ Linear, }} }}");
        assert_eq!(
            TextureAnimation::decode_mdl(&source).unwrap_err().kind,
            ReadErrorKind::DuplicateField
        );
    }
}
