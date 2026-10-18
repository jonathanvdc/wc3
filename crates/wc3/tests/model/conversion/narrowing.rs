use std::borrow::Cow;
use wc3::model::conversion::ImportOptions;
use wc3::model::formats::mdlx::{self, mdl::Read as _, mdx::Read as _, mdx::Write as _};
use wc3::model::io;
use wc3::model::profiles::{Mdlx, Universal, Wmo};
use wc3::model::ModelMetadata;
use wc3::model::{MdlxImageResource, Model, Texture};

#[test]
fn native_widening_and_narrowing_preserves_identities_and_borrowed_geometry() {
    let source =
        mdlx::Document::<mdlx::V800>::decode_mdl(include_str!("../../fixtures/mdl/quad_model.mdl"))
            .unwrap();
    let bytes = source.encode_mdx().unwrap();
    let source = mdlx::Document::<mdlx::V800>::decode_mdx(&bytes).unwrap();
    let asset = io::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let mesh_id = asset.meshes().iter().next().unwrap().0;
    let positions = asset.mesh(mesh_id).unwrap().positions().clone();
    assert!(!positions.is_owned());

    let asset = asset.widen::<Universal>().try_narrow::<Mdlx>().unwrap();
    let restored = asset.mesh(mesh_id).unwrap().positions();
    assert!(!restored.is_owned());
    assert_eq!(
        restored.iter().collect::<Vec<_>>(),
        positions.iter().collect::<Vec<_>>()
    );
}

#[test]
fn resource_narrowing_moves_owned_paths_without_reallocating() {
    let mut builder = Model::<Mdlx>::builder("owned resource");
    let path: Cow<'_, [u8]> = Cow::Owned(b"owned.blp".to_vec());
    let pointer = path.as_ptr();
    let resource = builder.add_image_resource(MdlxImageResource::Path(path));
    let id = Texture::new(resource);
    let asset = builder
        .finish()
        .unwrap()
        .widen::<Universal>()
        .try_narrow::<Mdlx>()
        .unwrap();
    let MdlxImageResource::Path(path) = asset.image_resource(id.resource).unwrap() else {
        unreachable!()
    };
    assert_eq!(path.as_ptr(), pointer);
    assert!(matches!(path, Cow::Owned(_)));
}

#[test]
fn narrowing_rejects_foreign_metadata() {
    let asset = Model::<Mdlx>::builder("Warcraft III")
        .finish()
        .unwrap()
        .widen::<Universal>();
    assert!(asset.try_narrow::<Wmo>().is_err());
    let asset = Model::<Universal>::builder(ModelMetadata::M2(Default::default()))
        .finish()
        .unwrap();
    assert!(asset.try_narrow::<Mdlx>().is_err());
}
