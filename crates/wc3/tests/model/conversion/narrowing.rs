use std::borrow::Cow;
use wc3::model::common::FileKey;
use wc3::model::conversion::{self, Asset, Format, ImportOptions, OpaquePayload};
use wc3::model::mdlx::{self, mdl::Read as _, mdx::Read as _, mdx::Write as _};
use wc3::model::semantic::profiles::{Mdlx, Universal, Wmo};
use wc3::model::semantic::{Model, Resource, Texture};

#[test]
fn native_widening_and_narrowing_preserves_identities_and_borrowed_geometry() {
    let source =
        mdlx::Model::<mdlx::V800>::decode_mdl(include_str!("../../fixtures/mdl/quad_model.mdl"))
            .unwrap();
    let bytes = source.encode_mdx().unwrap();
    let source = mdlx::Model::<mdlx::V800>::decode_mdx(&bytes).unwrap();
    let mut asset = conversion::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let model_id = asset.models[0].id();
    let mesh_id = asset.models[0].meshes().next().unwrap().0;
    let positions = asset.models[0].mesh(mesh_id).unwrap().positions().clone();
    assert!(positions.is_borrowed());
    asset.opaque.push(OpaquePayload {
        format: Format::Mdlx,
        file: FileKey::Root,
        tag: *b"TEST",
        revision: None,
        count: None,
        bytes: Cow::Borrowed(b"unknown"),
    });
    let asset = asset.widen::<Universal>().try_narrow::<Mdlx>().unwrap();
    assert_eq!(asset.models[0].id(), model_id);
    let restored = asset.models[0].mesh(mesh_id).unwrap().positions();
    assert!(restored.is_borrowed());
    assert_eq!(
        restored.iter().collect::<Vec<_>>(),
        positions.iter().collect::<Vec<_>>()
    );
    assert!(matches!(asset.opaque[0].bytes, Cow::Borrowed(_)));
}

#[test]
fn resource_narrowing_moves_owned_paths_without_reallocating() {
    let mut builder = Model::<Mdlx>::builder("owned resource");
    let mut texture = Texture::<Mdlx>::file("initial.blp");
    texture.resource = Resource::Path(Cow::Owned(b"owned.blp".to_vec()));
    let Resource::Path(path) = &texture.resource else {
        unreachable!()
    };
    let pointer = path.as_ptr();
    let id = builder.add_texture(texture).unwrap();
    let asset = Asset::new(builder.finish().unwrap())
        .widen::<Universal>()
        .try_narrow::<Mdlx>()
        .unwrap();
    let Resource::Path(path) = &asset.resources().texture(id).unwrap().resource else {
        unreachable!()
    };
    assert_eq!(path.as_ptr(), pointer);
    assert!(matches!(path, Cow::Owned(_)));
}

#[test]
fn narrowing_rejects_foreign_payloads_and_missing_required_controls() {
    let asset =
        Asset::new(Model::<Mdlx>::builder("Warcraft III").finish().unwrap()).widen::<Universal>();
    assert!(asset.try_narrow::<Wmo>().is_err());
    let asset = Asset::new(
        Model::<Universal>::builder("no profile controls")
            .finish()
            .unwrap(),
    );
    assert!(asset.try_narrow::<Mdlx>().is_err());
}

#[test]
fn narrowing_checks_native_asset_cardinality() {
    let first = Model::<Mdlx>::builder("first")
        .finish()
        .unwrap()
        .widen::<Universal>();
    let second = Model::<Mdlx>::builder("second")
        .finish()
        .unwrap()
        .widen::<Universal>();
    let asset = Asset::from_models(vec![first, second], None).unwrap();
    assert!(asset.try_narrow::<Mdlx>().is_err());
}
