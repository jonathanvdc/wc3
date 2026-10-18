use std::borrow::Cow;
use wc3::model::profiles::{Mdlx, Universal, Wmo};
use wc3::model::ModelMetadata;
use wc3::model::{MdlxImageResource, Model, Texture};

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
