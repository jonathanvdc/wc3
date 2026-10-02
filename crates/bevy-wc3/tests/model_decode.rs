use bevy_wc3::Wc3Model;
use wc3::model::mdl::Write as _;

#[test]
fn classic_fixture_converts_to_runtime_version() {
    let model = Wc3Model::decode(include_bytes!("fixtures/quad_model.mdx")).unwrap();
    assert_eq!(model.source_version, 800);
    assert_eq!(model.model.version(), 1800);
    assert!(!model.model.geosets().is_empty());
}

#[test]
fn mdl_and_mdx_normalize_to_the_same_model() {
    let text = include_str!("fixtures/quad_model.mdl");
    let mdl = Wc3Model::decode_mdl(text).unwrap();
    let mdx = Wc3Model::decode_mdx(include_bytes!("fixtures/quad_model.mdx")).unwrap();
    assert_eq!(mdl.source_version, 800);
    assert_eq!(
        mdl.model.encode_mdl().unwrap(),
        mdx.model.encode_mdl().unwrap()
    );
    assert_eq!(
        Wc3Model::decode(text.as_bytes())
            .unwrap()
            .model
            .encode_mdl()
            .unwrap(),
        mdl.model.encode_mdl().unwrap()
    );
    assert_eq!(
        Wc3Model::decode_mdl(&format!("\u{feff}{text}"))
            .unwrap()
            .model
            .encode_mdl()
            .unwrap(),
        mdl.model.encode_mdl().unwrap()
    );
}

#[test]
fn malformed_mdl_and_non_utf8_source_report_errors() {
    assert!(Wc3Model::decode_mdl("Version { FormatVersion").is_err());
    let error = Wc3Model::decode(&[0xff]).err().unwrap();
    assert!(error.to_string().contains("UTF-8"));
}

#[test]
fn synthetic_capture_fixture_contains_live_particle_emitters() {
    let model = Wc3Model::decode_mdl(include_str!("fixtures/particle_capture.mdl")).unwrap();
    let emitters = model.model.particle_emitters2();
    assert_eq!(emitters.len(), 2);
    assert!(emitters.iter().all(|emitter| emitter.life_span > 0.0));
    assert!(!emitters[0].node.flags.model_space());
    assert!(emitters[1].node.flags.model_space());
}
