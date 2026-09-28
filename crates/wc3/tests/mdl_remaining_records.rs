use wc3::model::emitters::{Particle2FilterMode, Particle2Frames};
use wc3::model::emitters::{ParticleEmitter2, PopcornEmitter};
use wc3::model::mdl::{Read as _, Write as _};
use wc3::model::mdx::{Read as _, Write as _};
use wc3::model::scene::{Camera, CameraTrack, CameraVariant};
use wc3::model::{
    mdl, DynamicModel, Model, ModelVersion, V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800,
    V900,
};

const SMOKE: &str = include_str!("fixtures/mdl/smoke.mdl");
const POPCORN: &str = include_str!("fixtures/mdl/popcorn_fire.mdl");
const CAMERA: &str = include_str!("fixtures/mdl/portrait.mdl");

#[test]
fn spec_examples_match_independently_packed_binary_records() {
    // MDX fixtures were packed from the attached specification, independently
    // of these codecs. Smoke uses unequal length/width to check their order.
    let smoke = ParticleEmitter2::decode_mdl(SMOKE).unwrap();
    let wire = include_bytes!("fixtures/mdl/smoke.mdx");
    assert_eq!(smoke.encode_mdx().unwrap(), wire);
    assert_eq!(ParticleEmitter2::decode_mdx(wire).unwrap(), smoke);
    assert_eq!(smoke.length, 16.0);
    assert_eq!(smoke.width, 17.0);
    assert_eq!(
        ParticleEmitter2::decode_mdl(&smoke.encode_mdl().unwrap()).unwrap(),
        smoke
    );
    let popcorn = PopcornEmitter::decode_mdl(POPCORN).unwrap();
    let wire = include_bytes!("fixtures/mdl/popcorn_fire.mdx");
    assert_eq!(popcorn.encode_mdx().unwrap(), wire);
    assert_eq!(PopcornEmitter::decode_mdx(wire).unwrap(), popcorn);
    assert_eq!(
        PopcornEmitter::decode_mdl(&popcorn.encode_mdl().unwrap()).unwrap(),
        popcorn
    );
    let camera = Camera::<V800>::decode_mdl(CAMERA).unwrap();
    let wire = include_bytes!("fixtures/mdl/portrait.mdx");
    assert_eq!(camera.encode_mdx().unwrap(), wire);
    assert_eq!(Camera::<V800>::decode_mdx(wire).unwrap(), camera);
    let canonical = camera.encode_mdl().unwrap();
    assert!(canonical.contains("FocusDistanceKeys 1"));
    assert_eq!(Camera::<V800>::decode_mdl(&canonical).unwrap(), camera);
}

#[test]
fn particle2_choices_arrays_defaults_and_validation() {
    let base = "ParticleEmitter2 \"p\" { ObjectId 0, ";
    let empty = ParticleEmitter2::decode_mdl(&format!("{base}}}")).unwrap();
    assert_eq!(empty, ParticleEmitter2::new(empty.node().clone()));
    assert_eq!(empty.node().flags().bits(), 0x1000);
    for (index, filter) in ["Blend", "Additive", "Modulate", "Modulate2x", "AlphaKey"]
        .iter()
        .enumerate()
    {
        for (frame, keyword) in ["Head", "Tail", "Both"].iter().enumerate() {
            let text = format!("{base}{filter}, {keyword}, Squirt 7, PriorityPlane 4294967295, ReplaceableId 2, }}");
            let record = ParticleEmitter2::decode_mdl(&text).unwrap();
            assert_eq!(
                record.filter_mode,
                Particle2FilterMode::from_raw(index as u32)
            );
            assert_eq!(record.frames, Particle2Frames::from_raw(frame as u32));
            assert_eq!(record.squirt, 7);
            assert_eq!(
                ParticleEmitter2::decode_mdl(&record.encode_mdl().unwrap()).unwrap(),
                record
            );
        }
    }
    for body in ["Blend, Additive,", "Head, Tail,", "Both, Both,", "Alpha { 256, 0, 0 },", "Alpha { -1, 0, 0 },", "LifeSpanUVAnim { 0, 1 },", "SegmentColor {}", "SegmentColor { Color { 1, 1, 1 }, Color { 1, 1, 1 }, }", "SegmentColor { Color { 1, 1, 1 }, Color { 1, 1, 1 }, Color { 1, 1, 1 }, Color { 1, 1, 1 }, }", "static Speed 2, Speed 1 { Linear, 0: 1, }", "LifeSpan 1 { Linear, 0: 1, }"] {
        assert!(ParticleEmitter2::decode_mdl(&format!("{base}{body}}}")).is_err(), "{body}");
    }
    let mut invalid = empty.clone();
    invalid.filter_mode = Particle2FilterMode::Unknown(5);
    assert!(invalid.encode_mdl().is_err());
    invalid.filter_mode = Particle2FilterMode::Blend;
    invalid.frames = Particle2Frames::Unknown(3);
    assert!(invalid.encode_mdl().is_err());
    let flags = ParticleEmitter2::decode_mdl(&format!(
        "{base}SortPrimsFarZ, LineEmitter, Unfogged, ModelSpace, Unshaded, XYQuad, }}"
    ))
    .unwrap();
    assert_eq!(flags.node().flags().bits(), 0x1f9000);
}

#[test]
fn emitter_tracks_preserve_signed_keys_splines_globals_and_hidden_bases() {
    let mut body = String::new();
    for name in [
        "Speed",
        "Variation",
        "Latitude",
        "Gravity",
        "EmissionRate",
        "Length",
        "Width",
        "Visibility",
    ] {
        body.push_str(&format!(
            "{name} 1 {{ Hermite, GlobalSeqId 3, -7: 2, InTan 1, OutTan 3, }}"
        ));
    }
    let mut record =
        ParticleEmitter2::decode_mdl(&format!("ParticleEmitter2 \"p\" {{ ObjectId 0, {body} }}"))
            .unwrap();
    assert_eq!(record.tracks().len(), 8);
    assert_eq!(
        ParticleEmitter2::decode_mdl(&record.encode_mdl().unwrap()).unwrap(),
        record
    );
    record.speed = 10.0;
    assert!(record.encode_mdl().is_err());
    let mut body = String::new();
    for name in ["LifeSpan", "EmissionRate", "Speed", "Alpha", "Visibility"] {
        body.push_str(&format!(
            "{name} 1 {{ Bezier, GlobalSeqId 2, -3: 1, InTan 2, OutTan 3, }}"
        ));
    }
    body.push_str("Color 1 { Hermite, -1: { 1, 2, 3 }, InTan { 4, 5, 6 }, OutTan { 7, 8, 9 }, }");
    let mut record = PopcornEmitter::decode_mdl(&format!(
        "ParticleEmitterPopcorn \"p\" {{ ObjectId 0, {body} }}"
    ))
    .unwrap();
    assert_eq!(record.tracks().len(), 6);
    assert_eq!(
        PopcornEmitter::decode_mdl(&record.encode_mdl().unwrap()).unwrap(),
        record
    );
    record.set_alpha(0.0);
    assert!(record.encode_mdl().is_err());
}

#[test]
fn popcorn_defaults_flags_and_literal_strings() {
    let record = PopcornEmitter::decode_mdl("ParticleEmitterPopcorn \"p\" { ObjectId 0, SortPrimsFarZ, Unshaded, Unfogged, PopcornScaling, Path \"a\\b.pkfx\", AnimVisibilityGuide \"Always=on,\nDeath=off\", }").unwrap();
    assert_eq!(record.node().flags().bits(), 0x79000);
    assert_eq!(record.life_span(), 1.0);
    assert_eq!(record.color(), [1.0; 3]);
    assert_eq!(record.visibility_guide(), "Always=on,\nDeath=off");
    assert_eq!(
        PopcornEmitter::decode_mdl(&record.encode_mdl().unwrap()).unwrap(),
        record
    );
    assert!(
        PopcornEmitter::decode_mdl("ParticleEmitterPopcorn \"p\" { ObjectId 0, XYQuad, }").is_err()
    );
    assert!(PopcornEmitter::decode_mdl(
        "ParticleEmitterPopcorn \"p\" { ObjectId 0, static Alpha 1, Alpha 0 {} }"
    )
    .is_err());
}

#[test]
fn camera_nested_tracks_aliases_required_fields_and_variants() {
    let base = "Camera \"c\" { FieldOfView 1, FarClip 100, ";
    for body in [
        "DOFDistance 1, FocusDistanceKeys 0 { DontInterp, }",
        "FocalLength 1, FocalLength 2,",
        "FStop 1, FStopKeys 0 { DontInterp, }",
        "Target { Translation 0 { DontInterp, } Translation 0 { DontInterp, } }",
        "Target {} Target {}",
        "Rotation 0 { DontInterp, } Rotation 0 { DontInterp, }",
    ] {
        assert!(
            Camera::<V800>::decode_mdl(&format!("{base}{body}}}")).is_err(),
            "{body}"
        );
    }
    assert!(Camera::<V800>::decode_mdl("Camera \"c\" { FarClip 1, }").is_err());
    assert!(Camera::<V800>::decode_mdl("Camera \"c\" { FieldOfView 1, }").is_err());
    let record = Camera::<V800>::decode_mdl(&format!("{base} Visibility 1 {{ Linear, -2: 1, }} Target {{ Position {{ 1, 2, 3 }}, Translation 1 {{ Hermite, GlobalSeqId 4, -5: {{ 1, 2, 3 }}, InTan {{ 4, 5, 6 }}, OutTan {{ 7, 8, 9 }}, }} }} FStop 2.8, FocalLength 50, DOFDistance 180, Rotation 1 {{ Bezier, -3: 1, InTan 2, OutTan 3, }} Translation 1 {{ Linear, -1: {{ 3, 2, 1 }}, }} }}")).unwrap();
    assert_eq!(record.tracks().len(), 7);
    assert_eq!(
        Camera::<V800>::decode_mdl(&record.encode_mdl().unwrap()).unwrap(),
        record
    );
    assert_eq!(
        Camera::<V800>::decode_mdx(&record.encode_mdx().unwrap()).unwrap(),
        record
    );
    assert!(matches!(
        record.tracks()[5],
        CameraTrack::TargetTranslation(_)
    ));
    let mut invalid = record.clone();
    let mut tracks = record.tracks().to_vec();
    tracks.reverse();
    invalid.set_tracks(&tracks);
    assert!(invalid.encode_mdl().is_err());
    invalid = record;
    invalid.set_variant(CameraVariant::Variant1([0; 12]));
    assert!(invalid.encode_mdl().is_err());
}

fn whole_model<V: ModelVersion>() {
    let popcorn = if V::NUMBER >= 900 { POPCORN } else { "" };
    let source = format!(
        "Version {{ FormatVersion {}, }} Model \"All\" {{}} {SMOKE} {CAMERA} {popcorn}",
        V::NUMBER
    );
    let record = Model::<V>::decode_mdl(&source).unwrap();
    let text = record.encode_mdl().unwrap();
    assert_eq!(
        DynamicModel::decode_mdl(&text)
            .unwrap()
            .encode_mdl()
            .unwrap(),
        text
    );
    let decoded = Model::<V>::decode_mdl(&text).unwrap();
    assert_eq!(decoded.cameras().len(), 1);
    assert_eq!(decoded.particle_emitters2().len(), 1);
    assert_eq!(
        Model::<V>::decode_mdx(&decoded.encode_mdx().unwrap())
            .unwrap()
            .encode_mdl()
            .unwrap(),
        text
    );
}
#[test]
fn model_io_supports_all_three_with_popcorn_version_gate() {
    whole_model::<V800>();
    whole_model::<V900>();
    whole_model::<V1000>();
    whole_model::<V1100>();
    whole_model::<V1200>();
    whole_model::<V1300>();
    whole_model::<V1400>();
    whole_model::<V1600>();
    whole_model::<V1800>();
    assert_eq!(
        Model::<V800>::decode_mdl(&format!(
            "Version {{ FormatVersion 800, }} Model \"a\" {{}} {POPCORN}"
        ))
        .unwrap_err()
        .kind,
        mdl::ReadErrorKind::UnsupportedField
    );
}
