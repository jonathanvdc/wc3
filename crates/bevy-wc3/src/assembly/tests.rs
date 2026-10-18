use super::plan::plan;
use super::*;
use bevy::prelude::{Assets, Image};
use std::collections::HashSet;
use wc3::model::materials::mdlx::LayeredSurface;
use wc3::model::profiles::Universal;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{
    Draw, DrawId, GeometryData, InstanceId, Material, MaterialSurface, Mesh, MeshInstance,
};

fn model() -> (Model<'static>, Vec<(InstanceId, DrawId)>) {
    let mut slot_values = SlotValues::default();
    let mut builder = Model::builder("generic assembly geometry");
    let node = builder.add_named_node("root").unwrap();
    let material = builder
        .add_material(Material {
            name: "generic".into(),
            surface: MaterialSurface::try_from(LayeredSurface::default()).unwrap(),
        })
        .unwrap();
    let mut ids = Vec::new();
    for _ in 0..2 {
        let geometry =
            GeometryData::triangles([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], [[0, 1, 2]])
                .unwrap();
        let mut mesh = Mesh::new(geometry);
        let subset = mesh.add_primary_subset(0, 1);
        let mesh = builder.add_mesh(mesh).unwrap();
        let mut instance = MeshInstance::new(node, mesh);
        let draw = Draw::new(subset, SlotConstruct::construct(&mut slot_values));
        let draw_id = draw.id();
        instance.draws.push(draw);
        let instance_id = builder
            .add_instance_with_materials(instance, vec![material])
            .unwrap();
        ids.push((instance_id, draw_id));
    }
    (
        {
            builder.import_values(&slot_values).unwrap();
            builder.finish()
        }
        .unwrap(),
        ids,
    )
}
#[test]
fn compatible_union_preserves_hidden_geometry_and_all_selection() {
    let (model, ids) = model();
    let models = [(ResourceLocator::FileDataId(1), model)]
        .into_iter()
        .collect();
    let source = Source::new(ResourceLocator::FileDataId(1));
    let mut a = PendingPart::new(source.clone(), true);
    a.source.draws.hidden_instances.insert(ids[0].0);
    let mut b = PendingPart::new(source.clone(), true);
    b.source.draws.draws = Some([ids[0].1].into_iter().collect());
    b.source.draws.hidden_instances.insert(ids[1].0);
    let assembly = PendingAssembly {
        root: source,
        parts: vec![a, b],
    };
    let result = plan(&assembly, &models, |b| *b).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].source.draws.draws.as_ref().unwrap().len(), 2);
    assert!(result[0].source.draws.hidden_instances.is_empty());
}
#[test]
fn incompatible_shared_textures_fail_before_image_provider() {
    let (model, _) = model();
    let texture = Model::<Universal>::builder("foreign")
        .add_file_texture("foreign")
        .unwrap();
    let models = [(ResourceLocator::FileDataId(1), model)]
        .into_iter()
        .collect();
    let mut source = Source::new(ResourceLocator::FileDataId(1));
    source.images.insert(
        texture.resource,
        TextureReplacement::Image(ImageResource::FileDataId(99)),
    );
    let assembly = Assembly::new(source);
    let mut calls = 0;
    let result = compile_assembly(&assembly, &models, &mut Assets::<Image>::default(), |_| {
        calls += 1;
        unreachable!()
    });
    assert!(result.is_err());
    assert_eq!(calls, 0);
}
#[test]
fn only_repeated_skinned_sources_coalesce() {
    let (a, _) = model();
    let (b, _) = model();
    let models = [
        (ResourceLocator::FileDataId(1), a),
        (ResourceLocator::FileDataId(2), b),
    ]
    .into_iter()
    .collect();
    let one = Source::new(ResourceLocator::FileDataId(1));
    let two = Source::new(ResourceLocator::FileDataId(2));
    let assembly = PendingAssembly {
        root: one.clone(),
        parts: vec![
            PendingPart::new(one.clone(), false),
            PendingPart::new(one, false),
            PendingPart::new(two, true),
        ],
    };
    assert_eq!(plan(&assembly, &models, |b| *b).unwrap().len(), 3);
}

#[test]
fn replacements_distinguish_identical_inputs_before_resolved_images_are_shared() {
    let mut slot_values = SlotValues::default();
    use wc3::model::materials::mdlx::Layer;

    let (mut model, _) = model();
    let material = model.materials().iter().next().unwrap().0;
    let first = model
        .edit(|draft| {
            let first = draft.add_file_texture("source.png")?;
            let second = first;
            let layers = [first, second].map(|image| {
                let mut layer = Layer::construct(&mut slot_values);
                layer.binding.image = slot_values.add(Some(image));
                layer
            });
            draft.set_material_surface(
                material,
                MaterialSurface::try_from(LayeredSurface {
                    layers: layers.into(),
                    ..SlotConstruct::construct(&mut slot_values)
                })?,
            )?;
            draft.import_values(&slot_values)?;
            Ok(first)
        })
        .unwrap();
    let resource = ResourceLocator::FileDataId(1);
    let models = [(resource.clone(), model)].into_iter().collect();
    let mut source = Source::new(resource.clone());
    source.images.insert(
        first.resource,
        TextureReplacement::Image(ImageResource::FileDataId(100)),
    );
    source.inputs.insert(
        TextureInput::Material {
            material,
            step: 1,
            slot: 0,
        },
        TextureReplacement::Image(ImageResource::FileDataId(200)),
    );
    let mut images = Assets::<Image>::default();
    let compiled = compile_assembly(&Assembly::new(source), &models, &mut images, |resource| {
        let ImageResource::FileDataId(value) = resource else {
            panic!("replacement was ignored")
        };
        Ok(RgbaImage::new(1, 1, vec![*value as u8, 0, 0, 255]).unwrap())
    })
    .unwrap();
    assert_eq!(images.len(), 2);
    let handles: HashSet<_> = compiled
        .draws
        .iter()
        .map(|draw| draw.material.base.base_color_texture.as_ref().unwrap().id())
        .collect();
    assert_eq!(handles.len(), 2);
    let mut shared = Assets::<Image>::default();
    compile_assembly(
        &Assembly::new(Source::new(resource)),
        &models,
        &mut shared,
        |_| Ok(RgbaImage::new(1, 1, vec![255; 4]).unwrap()),
    )
    .unwrap();
    assert_eq!(shared.len(), 1);
}

#[test]
fn input_replacement_survives_sampler_edits_and_animated_selection() {
    let mut slot_values = SlotValues::default();
    use bevy::image::{ImageAddressMode, ImageSampler};
    use wc3::model::materials::mdlx::Layer;

    use wc3::model::{AddressMode, AnimationSample, Channel, Curve, Time, TimeSource, Track};
    let (mut model, _) = model();
    let material = model.materials().iter().next().unwrap().0;
    let time = |value| Time::seconds(value).unwrap();
    let first = model
        .edit(|draft| {
            let edit_result = (|| {
                let first = draft.add_file_texture("first.png")?;
                let mut second = draft.add_file_texture("second.png")?;
                second.sampler.u = AddressMode::Mirror;
                let layers = (0..2)
                    .map(|_index| {
                        let mut layer = Layer::construct(&mut slot_values);
                        layer.binding.image = slot_values.add(Some(first));
                        layer.binding.coordinates.transform.translation =
                            slot_values.add([0.25, 0., 0.]);
                        layer
                    })
                    .collect::<Vec<_>>();
                let targets: Vec<_> = layers.iter().map(|layer| layer.binding.image).collect();
                draft.set_material_surface(
                    material,
                    MaterialSurface::try_from(LayeredSurface {
                        layers,
                        ..SlotConstruct::construct(&mut slot_values)
                    })?,
                )?;
                for target in targets {
                    draft.add_global_channel(Channel::OptionalTexture(Track {
                        target,
                        clock: TimeSource::Animation,
                        curve: Curve::step([time(0.), time(1.)], [Some(first), Some(second)])?,
                    }))?;
                }
                Ok(first)
            })();
            draft.import_values(&slot_values)?;
            edit_result
        })
        .unwrap();
    let resource = ResourceLocator::FileDataId(1);
    let mut source = Source::new(resource.clone());
    source.images.insert(
        first.resource,
        TextureReplacement::Image(ImageResource::FileDataId(100)),
    );
    source.inputs.insert(
        TextureInput::Material {
            material,
            step: 1,
            slot: 0,
        },
        TextureReplacement::Image(ImageResource::FileDataId(200)),
    );
    let models = [(resource, model)].into_iter().collect::<Models<'_>>();
    let mut images = Assets::<Image>::default();
    let compiled = compile_assembly(&Assembly::new(source), &models, &mut images, |resource| {
        let value = match resource {
            ImageResource::FileDataId(value) => *value as u8,
            ImageResource::Path(path) if path.as_ref() == b"second.png" => 50,
            _ => panic!("unexpected image source {resource:?}"),
        };
        Ok(RgbaImage::new(1, 1, vec![value, 0, 0, 255]).unwrap())
    })
    .unwrap();
    let model = models.values().next().unwrap();
    let sample = model
        .animation_sample(AnimationSample {
            clip_time: time(1.),
            ..SlotConstruct::construct(&mut slot_values)
        })
        .unwrap();
    for draw in &compiled.draws {
        let mut material = draw.material.clone();
        draw.animation.apply_to_material(&sample, &mut material);
        let image = images
            .get(material.base.base_color_texture.as_ref().unwrap())
            .unwrap();
        assert_eq!(
            image.data.as_ref().unwrap()[0],
            if draw.pass == 1 { 200 } else { 50 }
        );
        let ImageSampler::Descriptor(sampler) = &image.sampler else {
            panic!("missing sampler")
        };
        assert_eq!(sampler.address_mode_u, ImageAddressMode::MirrorRepeat);
        assert_eq!(material.base.uv_transform.translation.x, 0.25);
    }
}

#[test]
fn absent_input_is_rejected_before_resolving_images() {
    let (model, _) = model();
    let material = model.materials().iter().next().unwrap().0;
    let resource = ResourceLocator::FileDataId(1);
    let models = [(resource.clone(), model)].into_iter().collect();
    let mut source = Source::new(resource);
    source.inputs.insert(
        TextureInput::Material {
            material,
            step: 99,
            slot: 0,
        },
        TextureReplacement::Image(ImageResource::FileDataId(100)),
    );
    let result = compile_assembly(
        &Assembly::new(source),
        &models,
        &mut Assets::<Image>::default(),
        |_| panic!("invalid input reached image resolution"),
    );
    assert!(result.err().unwrap().0.contains("absent image input"));
}
