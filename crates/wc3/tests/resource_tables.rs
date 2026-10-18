use std::borrow::Cow;
use wc3::model::profiles::{Mdlx, Universal, M2};
use wc3::model::ModelMetadata;
use wc3::model::{
    Attachment, ChildModel, ImageResource, M2ImageResource, MdlxImageResource, Model, NodeFeature,
    PathResource, PlacementAppearance, ResourceLocator, Texture,
};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};

fn use_texture(draft: &mut wc3::model::ModelBuilder<'_, Universal>, texture: Texture) {
    let mut slot_values = SlotValues::default();
    use wc3::model::materials::mdlx::MaterialStep as MdlxStep;
    use wc3::model::profiles::mdlx::Material as MdlxMaterial;
    use wc3::model::{Material, MaterialSurface};
    let mut layer = MdlxStep::construct(&mut slot_values);
    layer.shader.inputs_mut().binding.image = slot_values.add(Some(texture));
    draft.import_values(&slot_values).unwrap();
    draft
        .add_material(Material {
            name: "input".into(),
            surface: MaterialSurface::from(MdlxMaterial { steps: vec![layer] }),
        })
        .unwrap();
}

#[test]
fn image_sharing_is_explicit_and_edits_are_transactional() {
    let slot_values = SlotValues::default();
    let mut draft = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let image = draft.add_image_resource(ImageResource::Path(b"body.blp".as_slice().into()));
    let first = Texture::new(image);
    let second = Texture::new(image);
    use_texture(&mut draft, first);
    use_texture(&mut draft, second);
    let independent = draft.add_image_resource(ImageResource::Path(b"body.blp".as_slice().into()));
    assert_ne!(image, independent);
    let mut model = {
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .unwrap();
    let original = model.clone();
    model
        .edit(|draft| {
            draft.set_image_resource(image, ImageResource::Path(b"edited.blp".as_slice().into()))
        })
        .unwrap();
    assert_eq!(first.resource, image);
    assert_eq!(second.resource, image);
    assert_ne!(model.image_resource(image), original.image_resource(image));
    assert_eq!(
        model.image_resource(independent),
        original.image_resource(independent)
    );
    assert!(model
        .edit(|draft| {
            draft.remove_image_resource(image);
            Ok(())
        })
        .is_err());
    assert!(model.image_resource(image).is_some());
}

#[test]
fn resource_handles_require_the_owning_table_and_removed_slots_are_not_reused() {
    let slot_values = SlotValues::default();
    let mut foreign = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let image = foreign.add_image_resource(ImageResource::Path(b"foreign.blp".as_slice().into()));
    let mut draft = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    use_texture(&mut draft, Texture::new(image));
    assert!({
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .is_err());

    let mut draft = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let removed = draft.add_image_resource(ImageResource::Path(b"old.blp".as_slice().into()));
    draft.remove_image_resource(removed);
    let replacement = draft.add_image_resource(ImageResource::Path(b"new.blp".as_slice().into()));
    assert_ne!(removed, replacement);
    use_texture(&mut draft, Texture::new(removed));
    assert!({
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .is_err());
}

#[test]
fn child_model_handles_are_checked_at_publication() {
    let slot_values = SlotValues::default();
    let mut foreign = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let resource =
        foreign.add_model_resource(ResourceLocator::Path(b"child.mdx".as_slice().into()));
    let mut draft = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let node = draft.add_named_node("child").unwrap();
    draft
        .add_feature(
            node,
            NodeFeature::Attachment(Attachment::Child(ChildModel {
                resource,
                socket: None,
                appearance: PlacementAppearance::Unmodified,
            })),
        )
        .unwrap();
    assert!({
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .is_err());
}

#[test]
fn profile_conversion_preserves_unreferenced_declarations_and_borrowed_paths() {
    let slot_values = SlotValues::default();
    let bytes = b"unreferenced.mdx";
    let mut draft = Model::<Mdlx>::builder("borrowed");
    let id = draft.add_model_resource(PathResource::Path(Cow::Borrowed(bytes)));
    let model = {
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .unwrap()
    .widen::<Universal>();
    assert!(
        matches!(model.model_resource(id), Some(ResourceLocator::Path(Cow::Borrowed(path))) if path.as_ptr() == bytes.as_ptr())
    );
    let model = model.try_narrow::<Mdlx>().unwrap();
    assert!(
        matches!(model.model_resource(id), Some(PathResource::Path(Cow::Borrowed(path))) if path.as_ptr() == bytes.as_ptr())
    );
}

#[test]
fn narrowing_checks_unused_resources_and_nested_companion_addresses() {
    let mut model = Model::<Mdlx>::empty("unused").unwrap().widen::<Universal>();
    model
        .edit(|draft| {
            draft.add_model_resource(ResourceLocator::FileDataId(123));
            Ok(())
        })
        .unwrap();
    assert!(model.try_narrow::<Mdlx>().is_err());

    let mut model = Model::<M2>::empty("companion")
        .unwrap()
        .widen::<Universal>();
    model
        .edit(|draft| {
            draft.add_physics_resource(ResourceLocator::Relative {
                owner: Box::new(ResourceLocator::Named {
                    namespace: "custom".into(),
                    name: "body".into(),
                }),
                suffix: ".phys".into(),
            });
            Ok(())
        })
        .unwrap();
    assert!(model.try_narrow::<M2>().is_err());
}

#[test]
fn unused_resource_values_are_validated() {
    let slot_values = SlotValues::default();
    let mut draft = Model::<M2>::builder("invalid image");
    draft.add_image_resource(M2ImageResource::FileDataId(0));
    assert!({
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .is_err());
}

#[test]
fn resource_particle_modes_round_trip_with_present_and_absent_dependencies() {
    use wc3::model::conversion::{ExportOptions, ImportOptions};
    use wc3::model::formats::mdlx::{self, mdl::Read as _};
    use wc3::model::{io, MdlxParticle1Resource, ParticleEmitter};

    for path in ["", "particles/child.mdx"] {
        for model_resource in [false, true] {
            for text_resource in [false, true] {
                let mut source = mdlx::Document::<mdlx::V800>::decode_mdl(include_str!(
                    "fixtures/mdl/quad_model.mdl"
                ))
                .unwrap();
                let mut particle = mdlx::emitters::ParticleEmitter::new(
                    mdlx::scene::Node::new("resource particle", 1).unwrap(),
                    path,
                )
                .unwrap();
                particle.node.flags.set_emitter_uses_mdl(model_resource);
                particle.node.flags.set_emitter_uses_tga(text_resource);
                source.set_particle_emitters([particle]);
                source.set_pivot_points(&[[0.; 3], [0.; 3]]);
                let model = io::mdlx::import(&source, &ImportOptions::default())
                    .unwrap()
                    .value
                    .widen::<Universal>();
                let details = model
                    .features()
                    .iter()
                    .find_map(|(_, feature)| {
                        let NodeFeature::Particle(ParticleEmitter::MdlxEmitter1(value)) =
                            &feature.payload
                        else {
                            return None;
                        };
                        let details = &value.details;
                        Some(details)
                    })
                    .unwrap();
                assert_eq!(
                    matches!(details.resource, MdlxParticle1Resource::Model(_)),
                    model_resource
                );
                assert_eq!(details.text_resource, text_resource);
                let absent = match details.resource {
                    MdlxParticle1Resource::Model(id) => id.is_none(),
                    MdlxParticle1Resource::Image(id) => id.is_none(),
                };
                assert_eq!(absent, path.is_empty());
                let model = model.try_narrow::<Mdlx>().unwrap();
                let output = io::mdlx::plan::<mdlx::V800>(&model, &ExportOptions::default())
                    .unwrap()
                    .execute()
                    .unwrap()
                    .value;
                let particle = output.particle_emitters().next().unwrap();
                assert_eq!(particle.flags().emitter_uses_mdl(), model_resource);
                assert_eq!(particle.flags().emitter_uses_tga(), text_resource);
                assert_eq!(particle.path.text(), path);
            }
        }
    }
}

#[test]
fn typed_collections_preserve_order_and_reject_foreign_identities() {
    let slot_values = SlotValues::default();
    let mut draft = Model::<Mdlx>::builder("tables");
    let first = draft.add_named_node("first").unwrap();
    let second = draft.add_named_node("second").unwrap();
    let image = draft.add_image_resource(MdlxImageResource::Path(b"body.blp".as_slice().into()));
    let mut foreign = Model::<Mdlx>::builder("foreign");
    let foreign_node = foreign.add_named_node("foreign").unwrap();
    let foreign_image =
        foreign.add_image_resource(MdlxImageResource::Path(b"body.blp".as_slice().into()));
    let model = {
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .unwrap();
    assert_eq!(model.nodes().len(), 2);
    assert_eq!(model.nodes().keys().collect::<Vec<_>>(), [first, second]);
    assert_eq!(
        model
            .nodes()
            .iter()
            .rev()
            .map(|(id, _)| id)
            .collect::<Vec<_>>(),
        [second, first]
    );
    assert_eq!(
        model
            .nodes()
            .values()
            .map(|node| node.name.as_ref())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
    assert!(model.nodes().get(foreign_node).is_none());
    assert!(!model.nodes().contains_key(foreign_node));
    assert!(model.dependencies().images().get(foreign_image).is_none());
    assert!(model.meshes().is_empty());
    assert!(model.dependencies().physics().is_empty());
    let nodes: Vec<_> = model.nodes().into_iter().map(|(id, _)| id).collect();
    assert_eq!(nodes, [first, second]);

    let model = model.widen::<Universal>().try_narrow::<Mdlx>().unwrap();
    assert_eq!(model.nodes().keys().collect::<Vec<_>>(), nodes);
    assert_eq!(model.nodes().get(first).unwrap().name, "first");
    assert!(model.dependencies().images().contains_key(image));
}
