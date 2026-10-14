use wc3::model::conversion::Asset;
use wc3::model::semantic::profiles::Universal;
use wc3::model::semantic::{
    Animated, Material, MaterialSurface, ModelBuilder, PbrSurface, Resource, Resources, Texture,
    TextureInput, TextureRole,
};

#[test]
fn shared_texture_edits_publish_once_for_every_model() {
    let mut library = Resources::builder();
    let texture = library.add_texture(Texture::file("original.blp"));
    let library = library.finish();
    let build = |name| {
        let mut draft = ModelBuilder::new(name);
        draft.set_resources(library.clone());
        draft
            .add_material(Material {
                name: "surface".into(),
                surface: MaterialSurface::Generic(Box::new(PbrSurface {
                    inputs: vec![TextureInput {
                        role: TextureRole::BaseColor,
                        texture: Animated::constant(Some(texture)),
                        uv_set: 0,
                        sampler: None,
                    }],
                    ..Default::default()
                })),
            })
            .unwrap();
        draft.finish().unwrap()
    };
    let mut asset = Asset::from_models(vec![build("first"), build("second")], None).unwrap();
    asset
        .edit_resources(|draft| draft.set_texture(texture, Texture::file("edited.blp")))
        .unwrap();
    for index in 0..asset.models.len() {
        let model = asset.models.view(index).unwrap();
        assert_eq!(
            model.texture(texture).unwrap().resource,
            Resource::Path("edited.blp".as_bytes().into())
        );
        assert!(std::ptr::eq(model.resources(), asset.resources()));
    }
    assert!(asset
        .edit_resources(|draft| {
            draft.remove_texture(texture);
            Ok(())
        })
        .is_err());
    assert_eq!(asset.resources().len(), 1);
    assert!((0..asset.models.len()).all(|index| asset
        .models
        .view(index)
        .unwrap()
        .texture(texture)
        .is_some()));
}

#[test]
fn unanimated_texture_references_are_checked_at_publication() {
    let mut foreign = Resources::<Universal>::builder();
    let texture = foreign.add_texture(Texture::file("foreign.blp"));
    let mut draft = ModelBuilder::new("invalid");
    draft
        .add_material(Material {
            name: "surface".into(),
            surface: MaterialSurface::Generic(Box::new(PbrSurface {
                inputs: vec![TextureInput {
                    role: TextureRole::BaseColor,
                    texture: Animated::constant(Some(texture)),
                    uv_set: 0,
                    sampler: None,
                }],
                ..Default::default()
            })),
        })
        .unwrap();
    assert!(draft.finish().is_err());
}

#[test]
fn collection_builder_resolves_shared_resources_only_when_published() {
    use wc3::model::semantic::{ModelCollection, Multiple};
    let mut collection = ModelCollection::<Universal, Multiple>::builder();
    let first = collection.add_model("first");
    let second = collection.add_model("second");
    let texture = collection
        .resources_mut()
        .add_texture(Texture::file("shared.blp"));
    for index in [first, second] {
        collection
            .model_mut(index)
            .unwrap()
            .add_material(Material::pbr().base_color(texture).unwrap())
            .unwrap();
    }
    let collection = collection.finish().unwrap();
    assert_eq!(collection.len(), 2);
    assert_eq!(collection.resources().len(), 1);
    let first = collection.view(first).unwrap();
    let second = collection.view(second).unwrap();
    assert!(std::ptr::eq(first.resources(), second.resources()));
    assert!(first.texture(texture).is_some());
    assert!(second.texture(texture).is_some());
}

#[test]
fn native_profiles_publish_declared_bounds_and_reject_clearing_them() {
    use wc3::model::semantic::profiles::{Universal, M2};
    use wc3::model::semantic::{Bounds, Model};
    let mut native = Model::<M2>::builder("required bounds").finish().unwrap();
    assert_eq!(native.bounds(), Some(Bounds::default()));
    assert!(native
        .edit(|draft| {
            draft.set_bounds(None);
            Ok(())
        })
        .is_err());
    assert_eq!(native.bounds(), Some(Bounds::default()));
    let universal = Model::<Universal>::builder("unspecified bounds")
        .finish()
        .unwrap();
    assert_eq!(universal.bounds(), None);
}
