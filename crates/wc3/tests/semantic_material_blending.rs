use wc3::model::materials::mdlx;
use wc3::model::materials::state::BlendOperations;

use wc3::model::conversion::{
    Error as ConversionError, ExportOptions, ImportOptions, IssueKind, LossPolicy,
};
use wc3::model::error::Location;
use wc3::model::formats::mdlx::emitters::{Particle2FilterMode, ParticleEmitter2};
use wc3::model::formats::mdlx::materials::LayerFilterMode;
use wc3::model::formats::mdlx::scene::Node;
use wc3::model::formats::mdlx::{self as native_mdlx, mdl::Read as _, V800};
use wc3::model::io::mdlx as semantic_mdlx;
use wc3::model::materials::mdlx::AlphaAdditiveEncoding;
use wc3::model::profiles::{mdlx as profile, Mdlx};
use wc3::model::Model;

fn layer_model(selector: &str) -> Model<'static, Mdlx> {
    semantic_mdlx::decode_mdl(
        &format!(
            r#"
        Version {{ FormatVersion 800, }} Model "blend" {{}}
        Textures 1 {{ Bitmap {{ Image "white.blp", }} }}
        Materials 1 {{ Material {{ Layer {{ FilterMode {selector}, static TextureID 0, }} }} }}
    "#
        ),
        &ImportOptions::default(),
    )
    .unwrap()
    .value
}
#[test]
fn warcraft_layer_additive_selectors_share_weighted_rgb_operation_and_roundtrip_exactly() {
    for (selector, encoding, wire) in [
        (
            "Additive",
            AlphaAdditiveEncoding::Additive,
            LayerFilterMode::Additive,
        ),
        (
            "AddAlpha",
            AlphaAdditiveEncoding::AddAlpha,
            LayerFilterMode::AddAlpha,
        ),
    ] {
        let model = layer_model(selector);
        let blend = model.materials().iter().next().unwrap().1.surface.steps[0]
            .state
            .blend;
        assert_eq!(blend, mdlx::Blend::AdditiveAlpha(encoding));
        // Both declarations carry the shared source-alpha-weighted RGB operation.
        assert!(matches!(blend, BlendOperations::AdditiveAlpha(_)));
        assert_ne!(blend, mdlx::Blend::Additive);
        let output = semantic_mdlx::plan::<V800>(&model, &ExportOptions::default())
            .unwrap()
            .execute()
            .value;
        assert_eq!(
            output.materials().next().unwrap().layers[0].filter_mode,
            wire
        );
    }
}
#[test]
fn unweighted_semantic_addition_has_no_exact_warcraft_layer_encoding() {
    let mut model = layer_model("Additive");
    let (material, source) = model.materials().iter().next().unwrap();
    let mut surface = source.surface.clone();
    surface.steps[0].state.blend = mdlx::Blend::Additive;
    model
        .edit(|builder| builder.set_material_surface(material, surface))
        .unwrap();
    let error = match semantic_mdlx::plan::<V800>(&model, &ExportOptions::default()) {
        Err(error) => error,
        Ok(_) => panic!("unweighted additive exported without loss"),
    };
    assert!(
        matches!(&error, ConversionError::Requirement { requirement, .. } if requirement.contains("unweighted"))
    );
    assert_eq!(
        error.location(),
        &Location::new()
            .field("material")
            .field("layers")
            .index(0)
            .field("blend")
    );
    let plan = semantic_mdlx::plan::<V800>(
        &model,
        &ExportOptions {
            loss_policy: LossPolicy::Drop,
            ..Default::default()
        },
    )
    .unwrap();
    let issue = plan
        .report()
        .issues
        .iter()
        .find(|issue| issue.description.contains("unweighted"))
        .unwrap();
    assert_eq!(issue.path, "material.layers[0].blend");
    assert_eq!(issue.kind, IssueKind::Dropped);
}
#[test]
fn atlas_additive_preserves_its_encoding_and_rejects_material_only_selector() {
    let mut source = native_mdlx::Document::<V800>::decode_mdl(
        r#"
        Version { FormatVersion 800, } Model "particle blend" {}
        Textures 1 { Bitmap { Image "white.blp", } }
    "#,
    )
    .unwrap();
    let mut emitter = ParticleEmitter2::new(Node::new("particle", 0).unwrap());
    emitter.filter_mode = Particle2FilterMode::Additive;
    source.set_particle_emitters2([emitter]);
    source.set_pivot_points(&[[0.; 3]]);
    let mut model = semantic_mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let (id, feature) = model.features().iter().next().unwrap();
    let profile::Node::Particle2(particle) = &feature.payload else {
        panic!("atlas particle lost")
    };
    assert_eq!(
        particle.details.rendering.blend,
        mdlx::Blend::AdditiveAlpha(AlphaAdditiveEncoding::Additive)
    );
    let output = semantic_mdlx::plan::<V800>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    assert_eq!(
        output.particle_emitters2().next().unwrap().filter_mode,
        Particle2FilterMode::Additive
    );
    let mut payload = feature.payload.clone();
    let profile::Node::Particle2(particle) = &mut payload else {
        unreachable!()
    };
    particle.details.rendering.blend = mdlx::Blend::AdditiveAlpha(AlphaAdditiveEncoding::AddAlpha);
    model
        .edit(|builder| builder.set_feature(id, payload))
        .unwrap();
    assert!(semantic_mdlx::plan::<V800>(&model, &ExportOptions::default()).is_err());
}
