use glam::Vec3;
use wc3::model::conversion::{ExportOptions, ImportOptions};
use wc3::model::formats::mdlx::{
    self,
    animation::{TextureAnimation, ValueKeyframe},
    materials::{Layer as NativeLayer, Material as NativeMaterial, Texture as NativeTexture},
};
use wc3::model::io;
use wc3::model::profiles::Universal;
use wc3::model::AddressMode;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};

fn source() -> mdlx::Document<'static, mdlx::V800> {
    let mut source = mdlx::Document::new();
    source.set_textures([NativeTexture::new("shared.blp").unwrap()]);
    let mut material = NativeMaterial::new();
    let mut layer = NativeLayer::new();
    layer.texture_animation_id = 0;
    layer.shading_flags = mdlx::materials::LayerShadingFlags(0x0c);
    material.layers = vec![layer.clone(), layer];
    source.set_materials([material]);
    source.set_texture_animations([TextureAnimation {
        translation: Some(
            mdlx::animation::Track::linear(
                [ValueKeyframe {
                    frame: 0,
                    value: [0.25, 0., 0.],
                }],
                None,
            )
            .unwrap(),
        ),
        ..Default::default()
    }]);
    source
}

#[test]
fn shared_native_inputs_import_as_independent_values_and_export_as_interned_tables() {
    let mut slot_values = SlotValues::default();
    let source = source();
    let mut model = io::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let (id, material) = model.materials().iter().next().unwrap();
    let first = &material.surface.steps[0].shader.inputs().binding;
    let second = &material.surface.steps[1].shader.inputs().binding;
    assert_eq!(model.values()[first.image], model.values()[second.image]);
    assert_ne!(first.image, second.image);
    assert_eq!(model.dependencies().images().len(), 1);
    assert_ne!(
        first.coordinates.transform.translation,
        second.coordinates.transform.translation
    );
    assert!(model.is_animated(first.coordinates.transform.translation));
    assert!(model.is_animated(second.coordinates.transform.translation));
    let output = io::mdlx::plan::<mdlx::V800>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.textures().count(), 1);
    assert_eq!(output.texture_animations().count(), 1);
    assert_eq!(
        output.materials().next().unwrap().layers[0]
            .shading_flags
            .bits()
            & 0x0c,
        0x0c
    );

    let mut surface = material.surface.clone();
    let image_slot = surface.steps[0].shader.inputs().binding.image;
    let mut image = model.values()[image_slot].unwrap();
    image.sampler.u = AddressMode::Repeat;
    surface.steps[0]
        .shader
        .inputs_mut()
        .binding
        .coordinates
        .transform
        .scale = slot_values.add(Vec3::from_array([2., 1., 1.]));
    model
        .edit(|draft| {
            draft.import_values(&slot_values)?;
            draft.values_mut().set(image_slot, Some(image))?;
            draft.set_material_surface(id, surface)
        })
        .unwrap();
    let layers = &model.material(id).unwrap().surface.steps;
    assert_eq!(
        model.values()[layers[1].shader.inputs().binding.image]
            .unwrap()
            .sampler
            .u,
        AddressMode::Clamp
    );
    assert_eq!(
        model.values()[layers[1]
            .shader
            .inputs()
            .binding
            .coordinates
            .transform
            .scale],
        Vec3::from_array([1.; 3])
    );
    let output = io::mdlx::plan::<mdlx::V800>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.textures().count(), 2);
    assert_eq!(output.texture_animations().count(), 2);
    let layers = &output.materials().next().unwrap().layers;
    assert_ne!(
        layers[0].texture_animation_id,
        layers[1].texture_animation_id
    );
    assert_ne!(layers[0].texture_id.value(), layers[1].texture_id.value());
}

#[test]
fn animated_selection_retains_each_selected_images_sampler() {
    let mut source = source();
    let first = NativeTexture::new("shared.blp").unwrap();
    let mut second = first.clone();
    second.flags.set_wrap_width(true);
    source.set_textures([first, second]);
    let mut material = source.materials().next().unwrap().clone();
    material.layers[0].texture_id.set_track(
        mdlx::animation::Track::step(
            [
                ValueKeyframe { frame: 0, value: 0 },
                ValueKeyframe {
                    frame: 500,
                    value: 1,
                },
            ],
            None,
        )
        .unwrap(),
    );
    source.set_materials([material]);
    let model = io::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(model.dependencies().images().len(), 1);
    let output = io::mdlx::plan::<mdlx::V800>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let textures: Vec<_> = output.textures().collect();
    let layer = &output.materials().next().unwrap().layers[0];
    let track = layer.texture_id.track().unwrap();
    assert!(!textures[track.evaluate(0.).unwrap() as usize]
        .flags
        .wrap_width());
    assert!(textures[track.evaluate(500.).unwrap() as usize]
        .flags
        .wrap_width());
}

#[test]
fn duplicate_native_texture_records_share_the_resource_and_are_deduplicated_on_export() {
    let mut source = source();
    let texture = NativeTexture::new("shared.blp").unwrap();
    source.set_textures([texture.clone(), texture]);
    let mut material = source.materials().next().unwrap().clone();
    material.layers[1].texture_id = 1.into();
    source.set_materials([material]);
    let model = io::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(model.dependencies().images().len(), 1);
    let output = io::mdlx::plan::<mdlx::V800>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.textures().count(), 1);
}

#[test]
fn sampling_a_material_freezes_its_owned_coordinate_animation() {
    let mut slot_values = SlotValues::default();
    use wc3::model::profiles::mdlx::Material as MdlxMaterial;
    use wc3::model::{AnimationSample, Time};
    let mut source = source();
    source.set_sequences([mdlx::animation::Sequence::new("Sample", [0, 1000]).unwrap()]);
    let model = io::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value
        .widen::<Universal>();
    let sample = model
        .animation_sample(AnimationSample {
            clip: Some(model.clips().iter().next().unwrap().0),
            clip_time: Time::seconds(0.).unwrap(),
            ..SlotConstruct::construct(&mut slot_values)
        })
        .unwrap();
    let material = model.materials().iter().next().unwrap().1;
    let frozen_values = sample.evaluate();
    let frozen = MdlxMaterial::try_from(material.surface.clone()).unwrap();
    for layer in frozen.steps {
        assert_eq!(
            frozen_values[layer
                .shader
                .inputs()
                .binding
                .coordinates
                .transform
                .translation],
            Vec3::from_array([0.25, 0., 0.])
        );
    }
}

#[test]
fn replaceable_image_declarations_retain_filenames_and_drop_uninterpreted_flags() {
    use std::borrow::Cow;
    use wc3::model::MdlxImageResource;
    let mut source = source();
    let mut texture = NativeTexture::new("authored.blp").unwrap();
    texture.replaceable_id = 1;
    texture.flags = mdlx::materials::TextureFlags(0x8000_0001);
    source.set_textures([texture]);
    let model = io::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let texture = model.sampled_images()[0];
    assert!(matches!(model.image_resource(texture.resource),
        Some(MdlxImageResource::Replaceable { id: 1, filename: Cow::Borrowed(path) }) if *path == b"authored.blp"));
    assert_eq!(model.dependencies().images().len(), 1);
    assert_eq!(texture.sampler.u, AddressMode::Repeat);
    assert_eq!(texture.sampler.v, AddressMode::Clamp);
    let output = io::mdlx::plan::<mdlx::V800>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let texture = output.textures().next().unwrap();
    assert_eq!(texture.replaceable_id, 1);
    assert_eq!(texture.path.text(), "authored.blp");
    assert_eq!(texture.flags.bits(), 1);
}
