use bevy_wc3::Wc3Model;
use wc3::model::conversion::ExportOptions;
use wc3::model::formats::mdlx::mdl::{Read as _, Write as _};
use wc3::model::formats::mdlx::{
    ConversionOptions, Document, DynamicDocument, NoExtensions, V1800,
};
use wc3::model::io;

fn native(model: &Wc3Model) -> Document<'_, V1800> {
    io::mdlx::plan::<V1800>(&model.model, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value
}

#[test]
fn classic_fixture_imports_into_semantic_model() {
    let model = Wc3Model::decode(include_bytes!("fixtures/quad_model.mdx")).unwrap();
    assert_eq!(model.source_version, 800);
    assert_eq!(model.model.meshes().count(), 1);
}

#[test]
fn mdl_and_mdx_normalize_to_the_same_model() {
    let text = include_str!("fixtures/quad_model.mdl");
    let mdl = Wc3Model::decode_mdl(text).unwrap();
    let mdx = Wc3Model::decode_mdx(include_bytes!("fixtures/quad_model.mdx")).unwrap();
    assert_eq!(mdl.source_version, 800);
    assert_eq!(
        native(&mdl).encode_mdl().unwrap(),
        native(&mdx).encode_mdl().unwrap()
    );
    for decoded in [
        Wc3Model::decode(text.as_bytes()).unwrap(),
        Wc3Model::decode_mdl(&format!("\u{feff}{text}")).unwrap(),
    ] {
        assert_eq!(
            native(&decoded).encode_mdl().unwrap(),
            native(&mdl).encode_mdl().unwrap()
        );
    }
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
    let document = native(&model);
    let emitters = document.particle_emitters2().collect::<Vec<_>>();
    assert_eq!(emitters.len(), 2);
    assert!(emitters.iter().all(|emitter| emitter.life_span > 0.0));
    assert!(!emitters[0].node.flags.model_space());
    assert!(emitters[1].node.flags.model_space());
}

#[test]
fn default_decode_and_compilation_use_the_semantic_graph() {
    use bevy_wc3::{compile_model, DecodedModel, Wc3Model};
    use wc3::model::{profiles::Mdlx, Model};
    let bytes = include_bytes!("fixtures/quad_model.mdx").to_vec();
    let decoded = DecodedModel::decode(&bytes).unwrap();
    let model: Model<'static, Mdlx> = decoded.model;
    drop(bytes);
    assert_eq!(model.meshes().count(), 1);
    let compiled = compile_model(model, |_| None, |_| None).unwrap();
    assert_eq!(compiled.lod_levels(), &[0]);
    let source = Wc3Model::decode_mdl(include_str!("fixtures/quad_model.mdl")).unwrap();
    assert_eq!(source.model.meshes().count(), 1);
}

#[test]
fn semantic_compilation_preserves_supported_feature_fixture_metadata() {
    use bevy_wc3::{compile_model, DecodedModel};
    for (name, text) in [
        ("cameras", include_str!("fixtures/cameras.mdl")),
        ("events", include_str!("fixtures/events.mdl")),
        ("light_capture", include_str!("fixtures/light_capture.mdl")),
        (
            "ribbon_capture",
            include_str!("fixtures/ribbon_capture.mdl"),
        ),
        (
            "particle_capture",
            include_str!("fixtures/particle_capture.mdl"),
        ),
        ("prem_capture", include_str!("fixtures/prem_capture.mdl")),
        ("hd_capture", include_str!("fixtures/hd_capture.mdl")),
    ] {
        let semantic =
            DecodedModel::decode(text.as_bytes()).unwrap_or_else(|error| panic!("{name}: {error}"));
        let native = DynamicDocument::<NoExtensions>::decode_mdl(text)
            .unwrap()
            .into_convert::<V1800>(&ConversionOptions::strict())
            .unwrap()
            .model;
        let semantic = compile_model(semantic.model, |_| None, |_| None)
            .unwrap_or_else(|error| panic!("{name}: {error}"));

        assert_eq!(
            semantic.sequences(),
            native.sequences().cloned().collect::<Vec<_>>()
        );
        assert!(semantic.bounds().1.is_finite());
        assert_eq!(
            semantic.texture_definitions(),
            native.textures().cloned().collect::<Vec<_>>()
        );
    }
}

#[test]
fn semantic_import_rejects_dangling_native_material_references() {
    use bevy_wc3::DecodedModel;
    let text = include_str!("fixtures/attachment_parent.mdl");
    // This native inspection fixture intentionally has geometry without materials.
    assert!(Document::<V1800>::decode_mdl(text).is_ok());
    assert!(DecodedModel::decode(text.as_bytes()).is_err());
}
