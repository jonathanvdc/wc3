use wc3::model::materials::layered::{Layer, LayeredSurface};
use wc3::model::profiles::Universal;
use wc3::model::{
    Animated, Material, MaterialSurface, ModelBuilder, Resource, Resources, Texture, TextureId,
};

fn textured_material(texture: TextureId) -> Material<'static> {
    let mut layer = Layer::default();
    layer.binding.image = Animated::constant(Some(texture));
    Material {
        name: "surface".into(),
        surface: MaterialSurface::Layered(Box::new(LayeredSurface {
            layers: vec![layer],
            ..Default::default()
        })),
    }
}

#[test]
fn model_resource_edits_are_transactional_and_clones_are_independent() {
    let mut draft = ModelBuilder::new("first");
    let texture = draft.add_texture(Texture::file("original.blp")).unwrap();
    draft.add_material(textured_material(texture)).unwrap();
    let mut model = draft.finish().unwrap();
    let original = model.clone();
    model
        .edit_resources(|draft| draft.set_texture(texture, Texture::file("edited.blp")))
        .unwrap();
    assert_eq!(
        model.texture(texture).unwrap().resource,
        Resource::Path("edited.blp".as_bytes().into())
    );
    assert_eq!(
        original.texture(texture).unwrap().resource,
        Resource::Path("original.blp".as_bytes().into())
    );
    assert!(model
        .edit_resources(|draft| {
            draft.remove_texture(texture);
            Ok(())
        })
        .is_err());
    assert_eq!(model.resources().len(), 1);
    assert_eq!(
        model.texture(texture).unwrap().resource,
        Resource::Path("edited.blp".as_bytes().into())
    );
}

#[test]
fn unanimated_texture_references_are_checked_at_publication() {
    let mut foreign = Resources::<Universal>::builder();
    let texture = foreign.add_texture(Texture::file("foreign.blp"));
    let mut draft = ModelBuilder::new("invalid");
    draft.add_material(textured_material(texture)).unwrap();
    assert!(draft.finish().is_err());
}

#[test]
fn draft_model_exposes_its_owned_resources_before_publication() {
    let mut draft = ModelBuilder::new("draft resources");
    let texture = draft.add_texture(Texture::file("model.blp")).unwrap();
    assert!(draft.model().texture(texture).is_some());
    draft.add_material(textured_material(texture)).unwrap();
    let model = draft.finish().unwrap();
    assert!(model.texture(texture).is_some());
    model.validate().unwrap();
}

#[test]
fn native_profiles_publish_declared_bounds_and_reject_clearing_them() {
    use wc3::model::profiles::{Universal, M2};
    use wc3::model::{Bounds, Model};
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

#[test]
fn metadata_edits_validate_and_preserve_the_published_model_on_failure() {
    use wc3::model::profiles::{mdlx::ModelMetadata, Mdlx};
    use wc3::model::Model;
    let mut draft = Model::<Mdlx>::builder("metadata");
    draft.set_metadata(ModelMetadata {
        blend_time: 0.25,
        ..Default::default()
    });
    let mut model = draft.finish().unwrap();
    let identity = model.id();
    assert_eq!(model.metadata().blend_time, 0.25);
    assert!(model
        .edit(|draft| {
            draft.set_metadata(ModelMetadata {
                blend_time: f64::NAN,
                ..Default::default()
            });
            Ok(())
        })
        .is_err());
    assert_eq!(model.metadata().blend_time, 0.25);
    assert_eq!(model.id(), identity);
    model
        .edit(|draft| {
            let mut metadata = draft.model().metadata().clone();
            metadata.blend_time = 0.5;
            draft.set_metadata(metadata);
            Ok(())
        })
        .unwrap();
    let restored = model.widen::<Universal>().try_narrow::<Mdlx>().unwrap();
    assert_eq!(restored.metadata().blend_time, 0.5);
    assert_eq!(restored.id(), identity);
}
