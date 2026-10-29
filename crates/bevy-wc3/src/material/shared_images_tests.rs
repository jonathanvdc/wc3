//! Differential material sampling across shared, scoped image substitutions.
use super::*;
use crate::textures::{TextureImageRemapper, TextureInput, TextureRequestKey};
use bevy::asset::AssetId;
use wc3::model::materials::bindings::TextureBinding;
use wc3::model::materials::state::{BlendOperations, Operation};
use wc3::model::profiles::mdlx::Material as MdlxMaterial;
use wc3::model::{
    AddressMode, AnimationSample, ColorSpace, Curve, DynamicTrack, Material, MaterialId,
    MaterialSurface, Model, ModelBuilder, ModelMetadata, Time, TimeSource, Track,
};

fn animated_surfaces() -> (Model<'static>, MaterialId) {
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let unspecified = builder.add_file_texture("shared.png").unwrap();
    let mut linear = unspecified;
    linear.color_space = ColorSpace::Linear;
    linear.sampler.u = AddressMode::Clamp;
    linear.sampler.v = AddressMode::Mirror;
    let mut explicit_srgb = unspecified;
    explicit_srgb.color_space = ColorSpace::Srgb;
    let choices = [Some(unspecified), Some(linear), Some(explicit_srgb), None];
    let mut steps = Vec::new();
    for family in 0..3 {
        let mut step: mdlx::MaterialStep = builder.construct();
        step.state.blend = match family {
            0 => BlendOperations::Alpha,
            1 => BlendOperations::AlphaKey(()),
            _ => BlendOperations::Modulate2x,
        };
        step.state.rasterization.faces = family != 0;
        step.state.rasterization.depth.test = Operation::Disabled;
        let binding = |builder: &mut ModelBuilder<'static>| {
            let image = builder.add_value(Some(unspecified));
            builder
                .add_global_track(DynamicTrack::OptionalTexture(Track {
                    target: image,
                    clock: TimeSource::Animation,
                    curve: Curve::step(
                        [0., 1., 2., 3.].map(|time| Time::seconds(time).unwrap()),
                        choices,
                    )
                    .unwrap(),
                }))
                .unwrap();
            image
        };
        if family == 0 {
            let inputs = step.shader.inputs_mut();
            inputs.binding.image = binding(&mut builder);
            inputs.opacity = builder.add_value(0.8);
        } else {
            let mut inputs: mdlx::ReforgedInputs = builder.construct();
            inputs.inputs.binding.image = binding(&mut builder);
            inputs.inputs.opacity = builder.add_value(0.8);
            inputs.inputs.emissive_gain = Some(builder.add_value(1.75));
            inputs.textures = (1..6)
                .map(|slot| mdlx::TextureSlot {
                    slot,
                    binding: TextureBinding {
                        image: binding(&mut builder),
                        ..builder.construct()
                    },
                })
                .collect();
            step.shader = if family == 1 {
                mdlx::Shader::ReforgedDefault(inputs)
            } else {
                mdlx::Shader::ReforgedCrystal(inputs)
            };
        }
        steps.push(step);
    }
    let material = builder
        .add_material(Material {
            name: "ordered animated families".into(),
            surface: MaterialSurface::from(MdlxMaterial {
                declaration: mdlx::MaterialDeclaration {
                    priority_plane: 7,
                    ..Default::default()
                },
                steps,
            }),
        })
        .unwrap();
    (builder.finish().unwrap(), material)
}

fn compiled(pass: &SurfacePass) -> ModelMaterial {
    ModelMaterial {
        base: pass.material.clone(),
        extension: pass.state.clone(),
    }
}

fn effective_images(animation: &SurfaceAnimation) -> &TextureImages {
    animation
        .hd
        .as_ref()
        .map(|hd| &hd.images)
        .or(animation.images.as_ref())
        .unwrap()
}

#[test]
fn shared_image_substitution_matches_fresh_compilation_at_every_animated_selection() {
    let (model, material_id) = animated_surfaces();
    let mut assets = Assets::<Image>::default();
    let mut substitutions = HashMap::<AssetId<Image>, Option<Handle<Image>>>::new();
    let mut fresh = HashMap::<TextureRequestKey, Option<Handle<Image>>>::new();
    let images = Arc::new(TextureImages::resolve(&model, |request| {
        let symbol = assets.add(Image::default());
        // Remove specific scoped interpretations while retaining the same image
        // in other roles, samplers, declarations, and material steps.
        let removed = matches!(
            request.input,
            Some(TextureInput::Material {
                step: 1,
                slot: 2,
                ..
            })
        ) || (request.texture.color_space == ColorSpace::Linear
            && matches!(request.input, Some(TextureInput::Material { step: 0, .. })));
        let replacement = (!removed).then(|| assets.add(Image::default()));
        fresh.insert(request.key(), replacement.clone());
        substitutions.insert(symbol.id(), replacement);
        Some(symbol)
    }));
    let mut passes = compile_material_with_images(&model, material_id, images.clone()).unwrap();
    let originals = passes
        .iter()
        .map(|pass| (compiled(pass), pass.animation.clone()))
        .collect::<Vec<_>>();
    let expected = compile_material_with_images(
        &model,
        material_id,
        Arc::new(TextureImages::resolve(&model, |request| {
            fresh[&request.key()].clone()
        })),
    )
    .unwrap();
    assert_eq!(passes.len(), 3);
    let mut remapper = TextureImageRemapper::default();
    let mut substitute = |image: &Handle<Image>| substitutions[&image.id()].clone();
    for pass in &mut passes {
        let mut material = compiled(pass);
        map_material_images(&mut material, &mut substitute);
        pass.material = material.base;
        pass.state = material.extension;
        pass.animation.map_images(&mut remapper, &mut substitute);
    }
    for (index, pass) in passes.iter().enumerate() {
        assert_eq!(pass.animation.images.is_some(), index == 0);
        let effective = effective_images(&pass.animation);
        assert!(effective.shares_storage(effective_images(&passes[0].animation)));
        assert!(!effective.shares_storage(&images));
    }
    for time in [0., 0.5, 1., 1.5, 2., 2.5, 3., 3.5] {
        let sample = model
            .animation_sample(AnimationSample {
                clip_time: Time::seconds(time).unwrap(),
                ..Default::default()
            })
            .unwrap();
        for (index, (actual, expected)) in passes.iter().zip(&expected).enumerate() {
            let mut actual_material = compiled(actual);
            let mut expected_material = compiled(expected);
            actual
                .animation
                .apply_to_material(&sample, &mut actual_material);
            expected
                .animation
                .apply_to_material(&sample, &mut expected_material);
            // Compare the complete prepared material, including texture handles,
            // HD presence flags, alpha, blending, depth and rasterization fields.
            assert_eq!(
                format!("{actual_material:?}"),
                format!("{expected_material:?}"),
                "step {index} at {time} seconds"
            );
            assert_eq!(actual_material.base.depth_bias, 7.);
            assert!(!actual_material.extension.depth_test);
            assert_eq!(actual_material.base.double_sided, index != 0);
            let (original, animation) = &originals[index];
            let mut original = original.clone();
            animation.apply_to_material(&sample, &mut original);
            if time < 3. {
                let handle = original.base.base_color_texture.as_ref().unwrap();
                assert!(substitutions.contains_key(&handle.id()));
                assert_ne!(
                    original.base.base_color_texture,
                    actual_material.base.base_color_texture
                );
            } else {
                assert!(actual_material.base.base_color_texture.is_none());
            }
        }
    }
}
