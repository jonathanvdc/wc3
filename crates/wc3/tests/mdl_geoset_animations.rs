use wc3::model::animation::{
    AnimationTrack, GeosetAlpha, GeosetAnimation, GeosetAnimationFlags, GeosetColor, GeosetTrack,
};
use wc3::model::mdl::{MdlWriter, Read as _, ReadErrorKind, Write as _};
use wc3::model::mdx::{Read as _, Write as _};

fn roundtrip(source: &str) -> GeosetAnimation {
    let value = GeosetAnimation::decode_mdl(source).unwrap();
    let bytes = value.encode_mdx().unwrap();
    let binary = GeosetAnimation::decode_mdx(&bytes).unwrap();
    let output = binary.encode_mdl().unwrap();
    let decoded = GeosetAnimation::decode_mdl(&output).unwrap();
    assert_eq!(decoded.encode_mdx().unwrap(), bytes);
    value
}

#[test]
fn spec_static_example_and_omitted_defaults() {
    let source =
        "GeosetAnim { static Alpha 1.0, DropShadow, GeosetId 0, static Color { 1.0, 1.0, 1.0 }, }";
    let value = roundtrip(source);
    assert_eq!(value.flags().bits(), 3);
    assert!(value.tracks().is_empty());
    assert_eq!(value.encode_mdl().unwrap(), "GeosetAnim {\n\tstatic Alpha 1.0,\n\tDropShadow,\n\tGeosetId 0,\n\tstatic Color { 1.0, 1.0, 1.0 },\n}\n");
    let omitted = roundtrip("GeosetAnim { GeosetId 2, }");
    assert_eq!(omitted.alpha(), 1.0);
    assert_eq!(omitted.color(), [1.0; 3]);
    assert_eq!(omitted.flags().bits(), 0);
    assert!(!omitted.encode_mdl().unwrap().contains("Color"));
    let nondefault = roundtrip(
        "GeosetAnim { static Color { 0.25, 0.5, 0.75 }, GeosetId 3, static Alpha -0.0, }",
    );
    assert_eq!(nondefault.alpha().to_bits(), (-0.0f32).to_bits());
    assert_eq!(nondefault.color(), [0.25, 0.5, 0.75]);
}

#[test]
fn animated_and_mixed_properties_restore_defaults_and_flags() {
    let source = "GeosetAnim { Color 1 { Hermite, -3600: { 0.25, 0.5, 0.75 }, InTan { 0, 0, 0 }, OutTan { 1, 1, 1 }, } GeosetId 1, Alpha 1 { Linear, GlobalSeqId 0, 0: 0.5, } }";
    let value = roundtrip(source);
    assert_eq!(value.alpha(), 1.0);
    assert_eq!(value.color(), [1.0; 3]);
    assert!(value.flags().color());
    assert!(matches!(value.tracks()[0], GeosetTrack::Color(_)));
    let output = value.encode_mdl().unwrap();
    assert!(!output.contains("static Alpha"));
    assert!(!output.contains("static Color"));
    roundtrip("GeosetAnim { GeosetId 1, static Alpha 0.25, Color 0 { Bezier, } }");
    roundtrip(
        "GeosetAnim { GeosetId 1, Alpha 0 { DontInterp, } static Color { 0.25, 0.5, 0.75 }, }",
    );
}

#[test]
fn static_and_animated_share_duplicate_checks() {
    for name in ["Alpha", "Color"] {
        let value = if name == "Alpha" { "1" } else { "{ 1, 1, 1 }" };
        let fixed = format!("static {name} {value},");
        let track = format!("{name} 0 {{ Linear, }}");
        for (first, second) in [
            (&fixed, &fixed),
            (&fixed, &track),
            (&track, &fixed),
            (&track, &track),
        ] {
            let source = format!("GeosetAnim {{ GeosetId 0, {first} {second} }}");
            let error = GeosetAnimation::decode_mdl(&source).unwrap_err();
            assert_eq!(error.kind, ReadErrorKind::DuplicateField);
            assert_eq!(&source[error.span.start..error.span.end], name);
        }
    }
    for source in [
        "GeosetAnim { GeosetId 0, DropShadow, DropShadow, }",
        "GeosetAnim { GeosetId 0, GeosetId 1, }",
    ] {
        assert_eq!(
            GeosetAnimation::decode_mdl(source).unwrap_err().kind,
            ReadErrorKind::DuplicateField
        );
    }
}

#[test]
fn rejects_invalid_fields_and_static_framing() {
    for source in [
        "GeosetAnim { }",
        "GeosetAnim { GeosetId 0, static GeosetId 1, }",
        "GeosetAnim { GeosetId 0, static DropShadow, }",
        "GeosetAnim { GeosetId 0, static Alpha 1 }",
        "GeosetAnim { GeosetId 0, static Color { 1, 1 }, }",
        "GeosetAnim { GeosetId 0, static 1, }",
        "GeosetAnim { GeosetId 0, Unknown, }",
        "GeosetAnim { GeosetId 0, } trailing",
    ] {
        assert!(GeosetAnimation::decode_mdl(source).is_err(), "{source}");
    }
}

fn refuses_before_output(value: &GeosetAnimation) {
    let mut bytes = Vec::new();
    let mut writer = MdlWriter::new(&mut bytes);
    assert!(writer.write(value).is_err());
    assert!(writer.finish().unwrap().is_empty());
}

#[test]
fn refuses_binary_data_that_text_would_discard() {
    let alpha = AnimationTrack::<GeosetAlpha>::linear(Vec::new(), None).unwrap();
    let color = AnimationTrack::<GeosetColor>::linear(Vec::new(), None).unwrap();
    let mut value = GeosetAnimation::new(0);
    value.set_flags(GeosetAnimationFlags(4));
    refuses_before_output(&value);
    value.set_flags(GeosetAnimationFlags(0));
    value.set_color([0.0; 3]);
    refuses_before_output(&value);
    value.set_color([1.0; 3]);
    value.set_tracks(&[color.clone().into()]);
    refuses_before_output(&value);
    value.set_flags(GeosetAnimationFlags(2));
    value.set_color([0.5; 3]);
    refuses_before_output(&value);
    value.set_color([1.0; 3]);
    value.set_tracks(&[alpha.clone().into()]);
    value.set_alpha(-0.0);
    refuses_before_output(&value);
    value.set_alpha(1.0);
    value.set_tracks(&[alpha.clone().into(), alpha.into()]);
    refuses_before_output(&value);
    value.set_tracks(&[color.clone().into(), color.into()]);
    refuses_before_output(&value);
}
