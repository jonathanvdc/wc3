use super::super::{
    compile_material_with_images, ModelMaterial, ModelMaterialExtension, ModelMaterialKey,
};
use super::layers::layer_uniform;
use super::*;
use std::sync::Arc;
use wc3::model::{AnimationSample, Material, MaterialSurface, Texture};

fn constant(color: [f32; 4]) -> programmed::TextureLayer {
    let mut layer = programmed::TextureLayer::default();
    layer.binding.sampling.flags.set_color(true);
    layer.binding.contribution.color.initial = color;
    layer.binding.contribution.color_type = ColorChannel::Rgba;
    layer.binding.contribution.map_alpha.initial = 1.;
    layer.binding.coordinates.transform.uv_tiling.initial = [1.; 2];
    layer.binding.coordinates.transform.uv_source_related = u32::MAX;
    layer
}
fn compile(surface: programmed::Standard) -> SurfacePass {
    let model = Model::builder("standard controls").finish().unwrap();
    compile_programmed(
        &model,
        &programmed::Material::Standard(Box::new(surface)),
        &TextureImages::default(),
    )
    .unwrap()
}
#[test]
fn constant_layers_keep_role_controls_without_requesting_unused_uv_streams() {
    let mut surface = programmed::Standard::default();
    let mut diffuse = constant([0.2, 0.4, 0.8, 0.5]);
    diffuse.binding.coordinates.source = UvMappingMode::TriPlanarWorld;
    diffuse.binding.contribution.map_alpha.initial = 0.25;
    surface.diffuse = Some(diffuse);
    surface.opacity.primary = Some(constant([0.3; 4]));
    surface.opacity.secondary = Some(constant([0.5; 4]));
    let pass = compile(surface);
    assert_eq!(pass.material.base_color, Color::WHITE);
    assert_eq!(
        pass.state.programmed.layers[0].color,
        Vec4::new(0.2, 0.4, 0.8, 0.5)
    );
    assert_eq!(pass.state.programmed.layers[0].controls.z, 0.25);
    assert_eq!(pass.state.programmed.layers[6].flags.x, 1);
    assert_eq!(pass.state.programmed.layers[7].flags.x, 1);
    assert_eq!(pass.fourth_coordinate, None);
    assert!(!pass.state.fourth_uv);
}
#[test]
fn uv_transform_uses_authored_radians_center_and_pretiling_offset() {
    let mut layer = constant([1.; 4]);
    layer.binding.coordinates.transform.uv_offset.initial = [0.25, -0.5];
    layer.binding.coordinates.transform.uv_angle.initial = [0., 0., std::f32::consts::FRAC_PI_2];
    layer.binding.coordinates.transform.uv_tiling.initial = [2., 3.];
    let uniform = layer_uniform(&layer, 0, None);
    let mapped = (uniform.uv * Vec3::new(0.75, 0., 1.)).truncate();
    assert!((mapped - Vec2::splat(0.5)).length() < 1e-6);
    layer.binding.coordinates.transform.uv_tiling.initial = [0.; 2];
    let collapsed = layer_uniform(&layer, 0, None);
    assert_eq!(
        (collapsed.uv * Vec3::new(0., 1., 1.)).truncate(),
        Vec2::splat(0.5)
    );
}
#[test]
fn every_regular_role_has_a_fixed_physical_image_binding() {
    let mut builder = Model::builder("standard roles");
    let texture = builder.add_texture(Texture::file("roles.dds")).unwrap();
    let mut layer = constant([1.; 4]);
    layer.binding.sampling.flags.set_color(false);
    layer.binding.image = Some(texture);
    let mut surface = programmed::Standard {
        diffuse: Some(layer.clone()),
        decal_layer: Some(layer.clone()),
        ..Default::default()
    };
    surface.specular.input.color = Some(layer.clone());
    surface.specular.input.gloss = Some(layer.clone());
    surface.emissive.input.primary.input = Some(layer.clone());
    surface.emissive.input.secondary.input = Some(layer.clone());
    surface.opacity.primary = Some(layer.clone());
    surface.opacity.secondary = Some(layer.clone());
    surface.normals.base = Some(layer.clone());
    surface.normals.primary.input = Some(layer.clone());
    surface.normals.primary.mask = Some(layer.clone());
    surface.normals.secondary.input = Some(layer.clone());
    surface.normals.secondary.mask = Some(layer.clone());
    surface.light_map_layer = Some(layer.clone());
    surface.ambient_occlusion_layer = Some(layer);
    surface.state.rasterization.flags.set_two_sided(true);
    let surface = programmed::Material::Standard(Box::new(surface));
    builder
        .add_material(Material {
            name: "roles".into(),
            surface: MaterialSurface::Programmed(Box::new(surface.clone())),
        })
        .unwrap();
    let model = builder.finish().unwrap();
    let handle = Handle::<Image>::default();
    let images = TextureImages::resolve(&model, |_| Some(handle.clone()));
    let pass = compile_programmed(&model, &surface, &images).unwrap();
    assert!(pass
        .state
        .programmed
        .layers
        .iter()
        .all(|layer| layer.flags.x == 1));
    assert_eq!(pass.material.base_color_texture, Some(handle.clone()));
    assert_eq!(pass.material.normal_map_texture, Some(handle.clone()));
    assert_eq!(pass.material.occlusion_texture, Some(handle.clone()));
    assert_eq!(pass.state.normal_secondary, Some(handle.clone()));
    assert_eq!(pass.state.normal_secondary_mask, Some(handle.clone()));
    assert_eq!(pass.state.light_map, Some(handle.clone()));
    assert_eq!(pass.state.ambient_occlusion, Some(handle));
    assert!(pass.material.double_sided);
    assert_eq!(pass.material.cull_mode, None);
    assert_eq!(pass.state.hd.maps.x, 0);
}
#[test]
fn uv3_only_requests_its_raw_stream_and_pipeline_layout() {
    let mut builder = Model::builder("fourth UV");
    let texture = builder.add_texture(Texture::file("uv.dds")).unwrap();
    let mut layer = constant([1.; 4]);
    layer.binding.image = Some(texture);
    layer.binding.sampling.flags.set_color(false);
    layer.binding.coordinates.source = UvMappingMode::ExplicitUV3;
    let surface = programmed::Standard {
        diffuse: Some(layer),
        ..Default::default()
    };
    let surface = programmed::Material::Standard(Box::new(surface));
    builder
        .add_material(Material {
            name: "UV route".into(),
            surface: MaterialSurface::Programmed(Box::new(surface.clone())),
        })
        .unwrap();
    let model = builder.finish().unwrap();
    let images = TextureImages::resolve(&model, |_| Some(Handle::default()));
    let pass = compile_programmed(&model, &surface, &images).unwrap();
    assert_eq!(pass.fourth_coordinate, Some(3));
    assert_eq!(pass.secondary_coordinate, None);
    assert_eq!(pass.third_coordinate, None);
    assert!(ModelMaterialKey::from(&pass.state).fourth_uv);
}

#[test]
fn unchanged_sampling_keeps_uniforms_and_team_settings_stable() {
    let model = Model::builder("static animation").finish().unwrap();
    let surface = programmed::Standard {
        diffuse: Some(constant([0.2, 0.4, 0.8, 1.])),
        ..Default::default()
    };
    let pass = compile(surface);
    let mut material = ModelMaterial {
        base: pass.material,
        extension: pass.state,
    };
    assert!(!pass.animation.is_animated());
    assert!(pass.animation.programmed.is_none());
    let sample = model.animation_sample(AnimationSample::default()).unwrap();
    assert!(!pass.animation.apply_to_material(&sample, &mut material));
    let colors = TeamColors {
        diffuse: Vec3::new(0.2, 0.4, 0.8),
        ..Default::default()
    };
    assert!(material.extension.set_team_colors(colors).unwrap());
    assert_eq!(material.extension.team_colors(), Some(colors));
    assert!(!pass.animation.apply_to_material(&sample, &mut material));
    assert_eq!(material.extension.team_colors(), Some(colors));
    assert!(!material.extension.set_team_colors(colors).unwrap());
    let invalid = TeamColors {
        diffuse_exponent: f32::NAN,
        ..colors
    };
    assert!(material.extension.set_team_colors(invalid).is_err());
    assert_eq!(material.extension.team_colors(), Some(colors));
    assert!(!ModelMaterialExtension::default()
        .set_team_colors(colors)
        .unwrap());
}
#[test]
fn unresolved_coordinate_generators_skip_only_the_affected_role() {
    let mut builder = Model::builder("unsupported sampled coordinates");
    let texture = builder
        .add_texture(Texture::file("unsupported.dds"))
        .unwrap();
    let model = builder.finish().unwrap();
    let mut surface = programmed::Standard::default();
    let mut layer = constant([1.; 4]);
    layer.binding.sampling.flags.set_color(false);
    layer.binding.image = Some(texture);
    layer.binding.coordinates.source = UvMappingMode::TriPlanarWorld;
    surface.diffuse = Some(layer);
    surface.opacity.primary = Some(constant([0.5; 4]));
    let pass = compile_programmed(
        &model,
        &programmed::Material::Standard(Box::new(surface)),
        &TextureImages::default(),
    )
    .unwrap();
    assert_eq!(pass.state.programmed.layers[0].flags.x, 0);
    assert_eq!(pass.state.programmed.layers[6].flags.x, 1);
    assert!(pass
        .diagnostics
        .iter()
        .any(|line| line.contains("coordinate generation")));
}

#[test]
fn alpha_test_uses_native_byte_threshold_without_losing_custom_clip_on_blend() {
    let mut surface = programmed::Standard::default();
    surface.state.rasterization.alpha_test_threshold = 128;
    let pass = compile(surface.clone());
    assert_eq!(pass.state.combiner.clip.x, 128. / 255.);
    assert!(matches!(pass.material.alpha_mode, AlphaMode::Mask(_)));
    surface.state.blend = BlendOperations::Alpha;
    let transparent = compile(surface);
    assert_eq!(transparent.state.combiner.clip.x, 128. / 255.);
    assert_eq!(transparent.material.alpha_mode, AlphaMode::Blend);
}

#[test]
fn effect_context_enables_external_color_without_changing_authored_mesh_state() {
    let pass = compile(programmed::Standard::default());
    assert_eq!(pass.state.programmed.settings.y, 0);
    assert_eq!(pass.state.programmed.settings.z, 0);
    let mut effect = pass.state.clone();
    effect.enable_effect_color();
    assert_eq!(effect.programmed.settings.y, 1);
    assert_eq!(effect.programmed.settings.z, 1);
    assert_eq!(pass.state.programmed.settings.y, 0);
    assert_eq!(pass.state.programmed.settings.z, 0);
    let mut ordinary = ModelMaterialExtension::default();
    ordinary.enable_effect_color();
    assert_eq!(ordinary.programmed.settings, UVec4::ZERO);
}

#[test]
fn unresolved_images_use_white_diffuse_and_neutral_optional_roles() {
    let mut builder = Model::builder("unresolved roles");
    let texture = builder.add_texture(Texture::file("missing.dds")).unwrap();
    let model = builder.finish().unwrap();
    let mut layer = constant([1.; 4]);
    layer.binding.sampling.flags.set_color(false);
    layer.binding.image = Some(texture);
    layer.binding.coordinates.source = UvMappingMode::ExplicitUV3;
    let mut surface = programmed::Standard {
        diffuse: Some(layer.clone()),
        ..Default::default()
    };
    surface.normals.base = Some(layer.clone());
    surface.opacity.primary = Some(layer.clone());
    surface.ambient_occlusion_layer = Some(layer.clone());
    let pass = compile_programmed(
        &model,
        &programmed::Material::Standard(Box::new(surface)),
        &TextureImages::default(),
    )
    .unwrap();
    assert_eq!(pass.state.programmed.layers[0].flags.x, 1);
    assert_eq!(pass.state.programmed.layers[0].flags.y, 0);
    for slot in [6, 8, 14] {
        assert_eq!(pass.state.programmed.layers[slot].flags.x, 0);
    }
    assert!(pass.material.normal_map_texture.is_none());
    assert_eq!(pass.fourth_coordinate, None);
    assert!(pass
        .diagnostics
        .iter()
        .any(|line| line.contains("unresolved")));
    assert!(pass.animation.programmed.is_none());
    let mut animation = ProgrammedAnimation {
        layers: from_fn(|_| None),
        weights: Vec::new(),
    };
    animation.layers[0] = Some(layer);
    let mut uniform = pass.state.programmed;
    let sample = model.animation_sample(AnimationSample::default()).unwrap();
    assert!(!animation.apply(&sample, &mut uniform));
    assert_eq!(uniform.layers[0].flags.y, 0);
}

#[test]
fn independent_image_roles_share_each_effective_native_wrap_sampler() {
    let mut builder = Model::builder("pooled standard sampling");
    let texture = builder.add_texture(Texture::file("wrap.dds")).unwrap();
    let mut layers = Vec::new();
    for (u, v) in [
        (false, false),
        (true, false),
        (false, true),
        (true, true),
        (false, false),
    ] {
        let mut layer = constant([1.; 4]);
        layer.binding.image = Some(texture);
        layer.binding.sampling.flags.set_color(false);
        layer.binding.sampling.flags.set_uv_wrap_x(u);
        layer.binding.sampling.flags.set_uv_wrap_y(v);
        layers.push(layer);
    }
    let mut surface = programmed::Standard {
        diffuse: Some(layers[0].clone()),
        decal_layer: Some(layers[1].clone()),
        ..Default::default()
    };
    surface.opacity.primary = Some(layers[2].clone());
    surface.opacity.secondary = Some(layers[3].clone());
    surface.normals.base = Some(layers[4].clone());
    let surface = programmed::Material::Standard(Box::new(surface));
    builder
        .add_material(Material {
            name: "wrap states".into(),
            surface: MaterialSurface::Programmed(Box::new(surface.clone())),
        })
        .unwrap();
    let model = builder.finish().unwrap();
    let images = TextureImages::resolve(&model, |_| Some(Handle::default()));
    let pass = compile_programmed(&model, &surface, &images).unwrap();
    assert_eq!(
        [0, 1, 6, 7, 8].map(|slot| pass.state.programmed.layers[slot].route.y),
        [0, 1, 2, 3, 0]
    );
    assert_eq!(pass.state.programmed_samplers.iter().flatten().count(), 4);
    assert!(pass.state.programmed_samplers[4..]
        .iter()
        .all(Option::is_none));
}

#[test]
fn animated_role_values_sample_timeline_and_preserve_compiled_image_routes() {
    use wc3::model::{Channel, Clock, Curve, Time as ModelTime, Track};
    let mut builder = Model::builder("animated material roles");
    let material = builder.reserve_material();
    let property = material.opacity();
    let mut opacity = constant([1.; 4]);
    opacity.binding.contribution.map_alpha.binding = Some(property.clone().into());
    let mut surface = programmed::Standard::default();
    surface.opacity.primary = Some(opacity);
    builder
        .set_material(
            material,
            Material {
                name: "animated".into(),
                surface: MaterialSurface::Programmed(Box::new(programmed::Material::Standard(
                    Box::new(surface),
                ))),
            },
        )
        .unwrap();
    builder
        .add_timeline_channel(Channel::Scalar(Track {
            target: property,
            clock: Clock::Timeline,
            curve: Curve::linear(
                [
                    ModelTime::seconds(0.).unwrap(),
                    ModelTime::seconds(1.).unwrap(),
                ],
                [0.2, 0.8],
            )
            .unwrap(),
        }))
        .unwrap();
    let model = builder.finish().unwrap();
    let pass = compile_material_with_images(&model, material, Arc::new(TextureImages::default()))
        .unwrap()
        .remove(0);
    assert!(pass.animation.is_animated());
    let mut material = ModelMaterial {
        base: pass.material,
        extension: pass.state,
    };
    material.extension.programmed.layers[6].route.y = 3;
    let sample = model
        .animation_sample(AnimationSample {
            timeline_time: ModelTime::seconds(0.5).unwrap(),
            ..Default::default()
        })
        .unwrap();
    assert!(pass.animation.apply_to_material(&sample, &mut material));
    assert!((material.extension.programmed.layers[6].controls.z - 0.5).abs() < 1e-6);
    assert_eq!(material.extension.programmed.layers[6].route.y, 3);
    assert!(!pass.animation.apply_to_material(&sample, &mut material));
}

#[test]
fn native_bound_constants_keep_opaque_composite_base_and_no_animation_sidecar() {
    use wc3::model::conversion::{ExportOptions, ImportOptions};
    use wc3::model::formats::m3;
    use wc3::model::profiles::{Universal, M3};
    use wc3::model::{io, profiles::m3 as native};
    let mut builder = Model::<M3>::builder("constant composite bindings");
    let standard = native::Standard {
        diffuse: Some(constant([1.; 4])),
        ..Default::default()
    };
    let leaf = builder
        .add_material(Material {
            name: "constant leaf".into(),
            surface: native::Material::Standard(Box::new(standard)),
        })
        .unwrap();
    let _composite = builder
        .add_material(Material {
            name: "constant composite".into(),
            surface: native::Material::Composite(Box::new(native::Composite {
                priority: 0,
                sections: [1., 0.35]
                    .map(|weight| native::CompositeSection {
                        material: leaf,
                        map_multiplier: native::Property {
                            initial: weight,
                            reset: weight,
                            ..Default::default()
                        },
                    })
                    .into(),
            })),
        })
        .unwrap();
    let model = builder.finish().unwrap();
    let native = io::m3::plan::<m3::StarCraft2>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let model = io::m3::import(&native, &ImportOptions::default())
        .unwrap()
        .value
        .widen::<Universal>();
    let composite = model
        .materials()
        .find_map(|(id, material)| (material.name == "constant composite").then_some(id))
        .unwrap();
    let MaterialSurface::Programmed(surface) = &model.material(composite).unwrap().surface else {
        panic!("programmed surface")
    };
    let programmed::Material::Composite(surface) = &**surface else {
        panic!("composite")
    };
    assert!(surface
        .sections
        .iter()
        .all(|section| section.map_multiplier.binding.is_some()));
    let passes =
        compile_material_with_images(&model, composite, Arc::new(TextureImages::default()))
            .unwrap();
    assert_eq!(passes.len(), 2);
    assert_eq!(passes[0].material.alpha_mode, AlphaMode::Opaque);
    assert_eq!(passes[1].material.alpha_mode, AlphaMode::Blend);
    assert_eq!(passes[0].material.base_color.alpha(), 1.);
    assert_eq!(passes[1].material.base_color.alpha(), 0.35);
    assert!(passes.iter().all(|pass| !pass.animation.is_animated()));
    assert!(passes
        .iter()
        .all(|pass| pass.animation.programmed.is_none()));
    assert_eq!(passes[0].state.programmed.layers[0].color, Vec4::ONE);
    let _ = composite;
}
