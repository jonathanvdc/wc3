use wc3::model::animation::{GlobalSequence, Sequence};
use wc3::model::chunks::{
    BindPoseChunk, CamerasChunk, FaceFxChunk, GlidersChunk, GlobalSequencesChunk, ModelChunk,
    ModelInfoChunk, ParticleEmitters2Chunk, PopcornEmittersChunk, RawChunk, SequencesChunk,
    UnknownChunk, VersionChunk,
};
use wc3::model::emitters::{ParticleEmitter2, PopcornEmitter};
use wc3::model::geometry::BindPoseMatrix;
use wc3::model::mdl::{Read as _, Write as _};
use wc3::model::mdx::{Read as _, Write as _};
use wc3::model::scene::{Camera, CameraVariant, FaceFx, Glider, ModelInfo, Node};
use wc3::model::{
    mdl, DynamicModel, Model, ModelVersion, V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800,
    V900,
};

const QUAD: &str = include_str!("fixtures/mdl/quad_model.mdl");
const CANONICAL: &str = include_str!("fixtures/mdl/quad_model.canonical.mdl");
const BINARY: &[u8] = include_bytes!("fixtures/mdl/quad_model.mdx");
const MINIMAL: &str = "Version { FormatVersion 800, } Model \"Minimal\" {}";
fn roundtrip<V: ModelVersion>(model: &Model<V>) {
    let text = model.encode_mdl().unwrap();
    let decoded = Model::<V>::decode_mdl(&text).unwrap_or_else(|error| panic!("{error}: {text}"));
    let wire = decoded.encode_mdx().unwrap();
    assert_eq!(
        Model::<V>::decode_mdx(&wire).unwrap().encode_mdl().unwrap(),
        text
    );
    assert_eq!(decoded.encode_mdl().unwrap(), text);
}
#[test]
fn specification_quad_roundtrips_independent_text_and_binary_fixtures() {
    let model = Model::<V800>::decode_mdl(QUAD).unwrap();
    assert_eq!(model.encode_mdl().unwrap(), CANONICAL);
    assert_eq!(model.encode_mdx().unwrap(), BINARY);
    let decoded = Model::<V800>::decode_mdx(BINARY).unwrap();
    assert_eq!(decoded.encode_mdl().unwrap(), CANONICAL);
    assert_eq!(
        Model::<V800>::decode_mdl(CANONICAL)
            .unwrap()
            .encode_mdx()
            .unwrap(),
        BINARY
    );
    assert_eq!(model.bones()[0].node.object_id, 0);
    assert_eq!(model.geosets()[0].material_id, 0);
    assert_eq!(
        DynamicModel::decode_mdl(QUAD)
            .unwrap()
            .encode_mdl()
            .unwrap(),
        CANONICAL
    );
}
#[test]
fn typed_and_dynamic_models_cover_every_supported_version() {
    macro_rules! check { ($($version:ty),*) => { $( {
        let source = QUAD.replace("FormatVersion 800", &format!("FormatVersion {}", <$version>::NUMBER));
        let model = Model::<$version>::decode_mdl(&source).unwrap();
        roundtrip(&model);
        let text = model.encode_mdl().unwrap();
        assert_eq!(Model::<$version>::decode_mdl(&text).unwrap().encode_mdx().unwrap(), model.encode_mdx().unwrap());
        let dynamic = DynamicModel::decode_mdl(&source).unwrap();
        assert_eq!(dynamic.version(), <$version>::NUMBER);
        assert_eq!(dynamic.encode_mdl().unwrap(), text);
    } )* }; }
    check!(V800, V900, V1000, V1100, V1200, V1300, V1400, V1600, V1800);
    let error = Model::<V900>::decode_mdl(MINIMAL).unwrap_err();
    assert_eq!(
        error.kind,
        mdl::ReadErrorKind::VersionMismatch {
            expected: 900,
            actual: 800
        }
    );
    assert!(error.to_string().contains("expected format version 900"));
    let source = MINIMAL.replace("800", "1500");
    let error = DynamicModel::decode_mdl(&source).unwrap_err();
    assert_eq!(
        error.kind,
        mdl::ReadErrorKind::UnsupportedVersion { version: 1500 }
    );
}
#[test]
fn noncanonical_input_preserves_record_order_ids_and_canonical_block_order() {
    let source = r#"Version { FormatVersion 800, }
        Glider { GeosetId 8, }
        Helper "second" { ObjectId 7, Parent 3, }
        Textures 0 {}
        GlobalSequences 2 { Duration 1000, Duration 2000, }
        Helper "first" { ObjectId 3, }
        Bone "bone" { ObjectId 12, GeosetId Multiple, }
        Glider { GeosetId 8, }
        Model "Order" { BoundsRadius -0.0, BlendTime 150, }
        PivotPoints 1 { { 0.0, 0.0, 0.0 }, }
        Glider { GeosetId 2, }
    "#;
    let model = Model::<V800>::decode_mdl(source).unwrap();
    assert_eq!(
        model
            .helpers()
            .iter()
            .map(|node| node.object_id)
            .collect::<Vec<_>>(),
        [7, 3]
    );
    assert_eq!(
        model
            .gliders()
            .iter()
            .map(|entry| entry.geoset_id)
            .collect::<Vec<_>>(),
        [8, 8, 2]
    );
    assert_eq!(
        model
            .chunks
            .iter()
            .filter(|chunk| chunk.tag() == *b"HELP")
            .count(),
        1
    );
    let text = model.encode_mdl().unwrap();
    assert!(!text.contains("Textures"));
    assert!(text.find("Model ").unwrap() < text.find("GlobalSequences").unwrap());
    assert!(text.find("Bone ").unwrap() < text.find("Helper ").unwrap());
    assert!(text.find("PivotPoints").unwrap() < text.find("Glider").unwrap());
    assert!(text.find("BlendTime").unwrap() < text.find("MinimumExtent").unwrap());
    assert!(text.contains("BoundsRadius -0.0,"));
    roundtrip(&model);
}
#[test]
fn available_record_codecs_dispatch_through_model_io() {
    let source = r#"Version { FormatVersion 900, } Model "Records" {}
        TextureAnims 1 { TVertexAnim { Translation 0 { Linear, } } }
        GeosetAnim { GeosetId 0, }
        Light "light" { ObjectId 0, Ambient, }
        Attachment "attachment" { ObjectId 1, }
        ParticleEmitter "particle" { ObjectId 2, }
        RibbonEmitter "ribbon" { ObjectId 3, }
        EventObject "event" { ObjectId 4, EventTrack 2 { -5, 10, } }
        CollisionShape "collision" { ObjectId 5, Sphere, Vertices 1 { { 0.0, 0.0, 0.0 }, } BoundsRadius 1.0, }
        FaceFX "Face" { Path "face.facefx", }
        BindPose { Matrices 1 { { 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0 }, } }
    "#;
    let model = Model::<V900>::decode_mdl(source).unwrap();
    assert_eq!(model.texture_animations().len(), 1);
    assert_eq!(model.lights().len(), 1);
    assert_eq!(model.event_objects()[0].frames.as_slice(), [-5, 10]);
    assert_eq!(model.face_fx()[0].path.text(), "face.facefx");
    assert_eq!(model.bind_poses().len(), 1);
    roundtrip(&model);
    for name in ["FaceFX \"Face\" {}", "BindPose { Matrices 0 {} }"] {
        assert_eq!(
            Model::<V800>::decode_mdl(&format!("{MINIMAL} {name}"))
                .unwrap_err()
                .kind,
            mdl::ReadErrorKind::UnsupportedField
        );
    }
}
#[test]
fn malformed_and_duplicate_top_level_blocks_retain_diagnostics() {
    for source in [
        "",
        "Model \"a\" {} Version { FormatVersion 800, }",
        "Version {} Model \"a\" {}",
        "Version { FormatVersion 800, }",
        "Version { FormatVersion 800, FormatVersion 800, } Model \"a\" {}",
    ] {
        assert!(Model::<V800>::decode_mdl(source).is_err());
    }
    for block in ["Version { FormatVersion 800, }", "Model \"Other\" {}"] {
        assert_eq!(
            Model::<V800>::decode_mdl(&format!("{MINIMAL} {block}"))
                .unwrap_err()
                .kind,
            mdl::ReadErrorKind::DuplicateField
        );
    }
    for name in [
        "Sequences",
        "GlobalSequences",
        "Textures",
        "Materials",
        "TextureAnims",
        "PivotPoints",
    ] {
        let source = format!("{MINIMAL} {name} 0 {{}} {name} 0 {{}}");
        let error = Model::<V800>::decode_mdl(&source).unwrap_err();
        assert_eq!(error.kind, mdl::ReadErrorKind::DuplicateField);
        assert_eq!(&source[error.span.start..error.span.end], name);
        assert!(Model::<V800>::decode_mdl(&format!("{MINIMAL} {name} 1 {{}}")).is_err());
    }
    let source = "Version { FormatVersion 900, } Model \"a\" {} BindPose { Matrices 0 {} } BindPose { Matrices 0 {} }";
    assert_eq!(
        Model::<V900>::decode_mdl(source).unwrap_err().kind,
        mdl::ReadErrorKind::DuplicateField
    );
    for name in ["BlendColors", "ComponentSkin", "Unknown"] {
        let source = format!("{MINIMAL} {name} {{}}");
        let error = Model::<V800>::decode_mdl(&source).unwrap_err();
        assert_eq!(error.kind, mdl::ReadErrorKind::UnknownField);
        assert_eq!(&source[error.span.start..error.span.end], name);
    }
    assert_eq!(
        Model::<V800>::decode_mdl(&format!("{MINIMAL} ParticleEmitterPopcorn \"a\" {{}}"))
            .unwrap_err()
            .kind,
        mdl::ReadErrorKind::UnsupportedField
    );
    assert_eq!(
        Model::<V800>::decode_mdl(&format!("{MINIMAL} ,"))
            .unwrap_err()
            .kind,
        mdl::ReadErrorKind::TrailingInput
    );
}
#[test]
fn writers_merge_known_collections_in_chunk_order_and_omit_empty_ones() {
    let mut model =
        Model::<V900>::decode_mdl("Version { FormatVersion 900, } Model \"Merged\" {}").unwrap();
    model
        .chunks
        .push(SequencesChunk::new(vec![Sequence::new("One", [0, 10]).unwrap()]).into());
    model.chunks.push(GlobalSequencesChunk::new(vec![]).into());
    model
        .chunks
        .push(SequencesChunk::new(vec![Sequence::new("Two", [11, 20]).unwrap()]).into());
    model
        .chunks
        .push(GlidersChunk::new(vec![Glider { geoset_id: 4 }, Glider { geoset_id: 4 }]).into());
    model
        .chunks
        .push(GlidersChunk::new(vec![Glider { geoset_id: 1 }]).into());
    model
        .chunks
        .push(GlobalSequencesChunk::new(vec![GlobalSequence(25)]).into());
    model
        .chunks
        .push(GlobalSequencesChunk::new(vec![GlobalSequence(50)]).into());
    model
        .chunks
        .push(BindPoseChunk::new(vec![BindPoseMatrix([1.0; 12])]).into());
    model
        .chunks
        .push(BindPoseChunk::new(vec![BindPoseMatrix([2.0; 12])]).into());
    model.chunks.push(CamerasChunk::<V900>::new(vec![]).into());
    let text = model.encode_mdl().unwrap();
    assert_eq!(text.matches("Sequences 2").count(), 2); // includes GlobalSequences
    assert_eq!(text.matches("BindPose {").count(), 1);
    assert!(text.contains("Matrices 2"));
    let parsed = Model::<V900>::decode_mdl(&text).unwrap();
    assert_eq!(
        parsed
            .sequences()
            .iter()
            .map(|sequence| sequence.name.text().into_owned())
            .collect::<Vec<_>>(),
        ["One", "Two"]
    );
    assert_eq!(
        parsed
            .gliders()
            .iter()
            .map(|entry| entry.geoset_id)
            .collect::<Vec<_>>(),
        [4, 4, 1]
    );
    assert_eq!(
        parsed.bind_poses(),
        [BindPoseMatrix([1.0; 12]), BindPoseMatrix([2.0; 12])]
    );
    roundtrip(&model);
}
#[test]
fn writer_rejects_missing_duplicate_extended_opaque_and_unsupported_chunks() {
    let mut model = Model::<V800>::new();
    assert!(model.encode_mdl().is_err());
    model.set_model_info(&ModelInfo::new("a").unwrap());
    model.chunks.retain(|chunk| chunk.tag() != *b"VERS");
    assert!(model.encode_mdl().is_err());
    let base = Model::<V800>::decode_mdl(MINIMAL).unwrap();
    let mut model = base.clone();
    model.chunks.push(VersionChunk::<V800>::new().into());
    assert!(model.encode_mdl().is_err());
    let mut model = base.clone();
    model
        .chunks
        .push(ModelInfoChunk::new(ModelInfo::new("other").unwrap(), vec![]).into());
    assert!(model.encode_mdl().is_err());
    let mut model = base.clone();
    let mut version = VersionChunk::<V800>::new();
    version.extension.push(1);
    model.chunks[0] = version.into();
    assert!(model.encode_mdl().is_err());
    let mut model = base.clone();
    model.chunks[1] = ModelInfoChunk::new(ModelInfo::new("a").unwrap(), vec![1]).into();
    assert!(model.encode_mdl().is_err());
    let mut model = base.clone();
    model.chunks.push(
        UnknownChunk::<V800>::new(RawChunk::new(*b"FUTR", vec![]))
            .map(ModelChunk::Unknown)
            .unwrap(),
    );
    assert!(model.encode_mdl().is_err());
    let mut model = base.clone();
    model
        .chunks
        .push(FaceFxChunk::new(vec![FaceFx::new("Face", "face.facefx").unwrap()]).into());
    assert!(model.encode_mdl().is_err());
    let mut model = base.clone();
    model
        .chunks
        .push(BindPoseChunk::new(vec![BindPoseMatrix([0.0; 12])]).into());
    assert!(model.encode_mdl().is_err());
    let mut model = base.clone();
    let mut camera = Camera::<V800>::new("camera").unwrap();
    camera.variant = CameraVariant::Variant1([0; 12]);
    model
        .chunks
        .push(CamerasChunk::<V800>::new(vec![camera]).into());
    assert!(model.encode_mdl().is_err());
    let mut model = base.clone();
    model.chunks.push(
        ParticleEmitters2Chunk::new(vec![ParticleEmitter2::new(Node::new("p2", 0).unwrap())])
            .into(),
    );
    assert!(model.encode_mdl().is_err());
    let mut model = base;
    model.chunks.push(
        PopcornEmittersChunk::new(vec![PopcornEmitter::new(
            Node::new("pop", 0).unwrap(),
            "fx.pkfx",
            "Always=on",
        )
        .unwrap()])
        .into(),
    );
    assert!(model.encode_mdl().is_err());
}

#[test]
fn small_derived_record_codecs_keep_required_headers_counts_and_defaults() {
    let glider = Glider::decode_mdl("Glider { GeosetId 7, }").unwrap();
    assert_eq!(glider.encode_mdl().unwrap(), "Glider {\n\tGeosetId 7,\n}\n");
    assert!(Glider::decode_mdl("Glider {}").is_err());
    assert!(Glider::decode_mdl("Glider { GeosetId 1, GeosetId 2, }").is_err());
    let face = FaceFx::decode_mdl("FaceFX \"Face\" {}").unwrap();
    assert_eq!(face.path.text(), "");
    assert_eq!(
        face.encode_mdl().unwrap(),
        "FaceFX \"Face\" {\n\tPath \"\",\n}\n"
    );
    assert!(FaceFx::decode_mdl("FaceFX {}").is_err());
    assert!(BindPoseChunk::decode_mdl("BindPose {}").is_err());
    assert!(BindPoseChunk::decode_mdl("BindPose { Matrices 1 { { 0.0, 0.0 }, } }").is_err());
    assert!(BindPoseChunk::decode_mdl("BindPose { Matrices 0 {} Matrices 0 {} }").is_err());
    assert_eq!(
        VersionChunk::<V800>::decode_mdl("Version { FormatVersion 800, }")
            .unwrap()
            .encode_mdl()
            .unwrap(),
        "Version {\n\tFormatVersion 800,\n}\n"
    );
}
