use super::super::{
    compile_material_with_images, ModelMaterial, ModelMaterialExtension, ModelMaterialKey,
};
use super::layers::layer_uniform;
use super::*;
use std::sync::Arc;
use wc3::model::materials::m3;
use wc3::model::{AnimationSample, Material, MaterialStep, MaterialSurface};
use wc3::model::{Construct as SlotConstruct, Values as SlotValues};

fn constant(slot_values: &mut SlotValues, color: [f32; 4]) -> m3::TextureLayer {
    let mut layer = m3::TextureLayer::construct(slot_values);
    layer.binding.sampling.flags.set_color(true);
    slot_values
        .set(layer.binding.contribution.color.value, color)
        .unwrap();
    layer.binding.contribution.color_type = ColorChannel::Rgba;
    slot_values
        .set(layer.binding.contribution.map_alpha.value, 1.)
        .unwrap();
    slot_values
        .set(layer.binding.coordinates.transform.uv_tiling.value, [1.; 2])
        .unwrap();
    layer.binding.coordinates.transform.uv_source_related = u32::MAX;
    layer
}
fn compile(surface: m3::StandardMaterialStep, slot_values: &SlotValues) -> SurfacePass {
    let mut builder = Model::builder("standard controls");
    builder.import_values(slot_values).unwrap();
    let model = builder.finish().unwrap();
    compile_m3(
        &model,
        &m3::MaterialStep::Standard(Box::new(surface)),
        &TextureImages::default(),
    )
    .unwrap()
}
#[test]
fn constant_layers_keep_role_controls_without_requesting_unused_uv_streams() {
    let mut slot_values = SlotValues::default();
    let mut surface = m3::StandardMaterialStep::construct(&mut slot_values);
    let mut diffuse = constant(&mut slot_values, [0.2, 0.4, 0.8, 0.5]);
    diffuse.binding.coordinates.source = UvMappingMode::TriPlanarWorld;
    slot_values
        .set(diffuse.binding.contribution.map_alpha.value, 0.25)
        .unwrap();
    surface.diffuse = Some(diffuse);
    surface.opacity.primary = Some(constant(&mut slot_values, [0.3; 4]));
    surface.opacity.secondary = Some(constant(&mut slot_values, [0.5; 4]));
    let pass = compile(surface, &slot_values);
    assert_eq!(pass.material.base_color, Color::WHITE);
    assert_eq!(pass.state.m3.layers[0].color, Vec4::new(0.2, 0.4, 0.8, 0.5));
    assert_eq!(pass.state.m3.layers[0].controls.z, 0.25);
    assert_eq!(pass.state.m3.layers[6].flags.x, 1);
    assert_eq!(pass.state.m3.layers[7].flags.x, 1);
    assert_eq!(pass.fourth_coordinate, None);
    assert!(!pass.state.fourth_uv);
}
#[test]
fn uv_transform_uses_authored_radians_center_and_pretiling_offset() {
    let mut slot_values = SlotValues::default();
    let layer = constant(&mut slot_values, [1.; 4]);
    slot_values
        .set(
            layer.binding.coordinates.transform.uv_offset.value,
            [0.25, -0.5],
        )
        .unwrap();
    slot_values
        .set(
            layer.binding.coordinates.transform.uv_angle.value,
            [0., 0., std::f32::consts::FRAC_PI_2],
        )
        .unwrap();
    slot_values
        .set(
            layer.binding.coordinates.transform.uv_tiling.value,
            [2., 3.],
        )
        .unwrap();
    let uniform = layer_uniform(&layer, 0, &slot_values);
    let mapped = (uniform.uv * Vec3::new(0.75, 0., 1.)).truncate();
    assert!((mapped - Vec2::splat(0.5)).length() < 1e-6);
    slot_values
        .set(layer.binding.coordinates.transform.uv_tiling.value, [0.; 2])
        .unwrap();
    let collapsed = layer_uniform(&layer, 0, &slot_values);
    assert_eq!(
        (collapsed.uv * Vec3::new(0., 1., 1.)).truncate(),
        Vec2::splat(0.5)
    );
}
#[test]
fn every_regular_role_has_a_fixed_physical_image_binding() {
    let mut slot_values = SlotValues::default();
    let mut builder = Model::builder("standard roles");
    let texture = builder.add_file_texture("roles.dds").unwrap();
    let mut layer = constant(&mut slot_values, [1.; 4]);
    layer.binding.sampling.flags.set_color(false);
    layer.binding.image = Some(texture.resource);
    let mut surface = m3::StandardMaterialStep {
        diffuse: Some(layer.clone()),
        decal_layer: Some(layer.clone()),
        ..SlotConstruct::construct(&mut slot_values)
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
    let surface = m3::MaterialStep::Standard(Box::new(surface));
    builder
        .add_material(Material {
            name: "roles".into(),
            surface: MaterialSurface::from(surface.clone()),
        })
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let handle = Handle::<Image>::default();
    let images = TextureImages::resolve(&model, |_| Some(handle.clone()));
    let mut model = model.clone();
    model
        .edit(|builder| builder.import_values(&slot_values))
        .unwrap();
    let pass = compile_m3(&model, &surface, &images).unwrap();
    assert!(pass.state.m3.layers.iter().all(|layer| layer.flags.x == 1));
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
    let mut slot_values = SlotValues::default();
    let mut builder = Model::builder("fourth UV");
    let texture = builder.add_file_texture("uv.dds").unwrap();
    let mut layer = constant(&mut slot_values, [1.; 4]);
    layer.binding.image = Some(texture.resource);
    layer.binding.sampling.flags.set_color(false);
    layer.binding.coordinates.source = UvMappingMode::ExplicitUV3;
    let surface = m3::StandardMaterialStep {
        diffuse: Some(layer),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let surface = m3::MaterialStep::Standard(Box::new(surface));
    builder
        .add_material(Material {
            name: "UV route".into(),
            surface: MaterialSurface::from(surface.clone()),
        })
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let images = TextureImages::resolve(&model, |_| Some(Handle::default()));
    let mut model = model.clone();
    model
        .edit(|builder| builder.import_values(&slot_values))
        .unwrap();
    let pass = compile_m3(&model, &surface, &images).unwrap();
    assert_eq!(pass.fourth_coordinate, Some(3));
    assert_eq!(pass.secondary_coordinate, None);
    assert_eq!(pass.third_coordinate, None);
    assert!(ModelMaterialKey::from(&pass.state).fourth_uv);
}

#[test]
fn unchanged_sampling_keeps_uniforms_and_team_settings_stable() {
    let mut slot_values = SlotValues::default();
    let model = {
        let mut draft = Model::<wc3::model::profiles::Universal>::builder("static animation");
        draft.import_values(&slot_values).unwrap();
        draft.finish().unwrap()
    };
    let surface = m3::StandardMaterialStep {
        diffuse: Some(constant(&mut slot_values, [0.2, 0.4, 0.8, 1.])),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let pass = compile(surface, &slot_values);
    let mut material = ModelMaterial {
        base: pass.material,
        extension: pass.state,
    };
    assert!(!pass.animation.is_animated());
    assert!(pass.animation.m3.is_none());
    let sample = model.animation_sample(AnimationSample::default()).unwrap();
    assert!(!pass.animation.apply_to_material(&sample, &mut material));
    let colors = TeamColors {
        diffuse: Vec3::new(0.2, 0.4, 0.8),
        ..SlotConstruct::construct(&mut slot_values)
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
    let mut slot_values = SlotValues::default();
    let mut builder = Model::builder("unsupported sampled coordinates");
    let texture = builder.add_file_texture("unsupported.dds").unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let mut surface = m3::StandardMaterialStep::construct(&mut slot_values);
    let mut layer = constant(&mut slot_values, [1.; 4]);
    layer.binding.sampling.flags.set_color(false);
    layer.binding.image = Some(texture.resource);
    layer.binding.coordinates.source = UvMappingMode::TriPlanarWorld;
    surface.diffuse = Some(layer);
    surface.opacity.primary = Some(constant(&mut slot_values, [0.5; 4]));
    let mut model = model.clone();
    model
        .edit(|builder| builder.import_values(&slot_values))
        .unwrap();
    let pass = compile_m3(
        &model,
        &m3::MaterialStep::Standard(Box::new(surface)),
        &TextureImages::default(),
    )
    .unwrap();
    assert_eq!(pass.state.m3.layers[0].flags.x, 0);
    assert_eq!(pass.state.m3.layers[6].flags.x, 1);
    assert!(pass
        .diagnostics
        .iter()
        .any(|line| line.contains("coordinate generation")));
}

#[test]
fn alpha_test_uses_native_byte_threshold_without_losing_custom_clip_on_blend() {
    let mut slot_values = SlotValues::default();
    let mut surface = m3::StandardMaterialStep::construct(&mut slot_values);
    surface.state.rasterization.alpha_test_threshold = 128;
    let pass = compile(surface.clone(), &slot_values);
    assert_eq!(pass.state.combiner.clip.x, 128. / 255.);
    assert!(matches!(pass.material.alpha_mode, AlphaMode::Mask(_)));
    surface.state.blend = BlendOperations::Alpha;
    let transparent = compile(surface, &slot_values);
    assert_eq!(transparent.state.combiner.clip.x, 128. / 255.);
    assert_eq!(transparent.material.alpha_mode, AlphaMode::Blend);
}

#[test]
fn effect_context_enables_external_color_without_changing_authored_mesh_state() {
    let mut slot_values = SlotValues::default();
    let pass = compile(
        m3::StandardMaterialStep::construct(&mut slot_values),
        &slot_values,
    );
    assert_eq!(pass.state.m3.settings.y, 0);
    assert_eq!(pass.state.m3.settings.z, 0);
    let mut effect = pass.state.clone();
    effect.enable_effect_color();
    assert_eq!(effect.m3.settings.y, 1);
    assert_eq!(effect.m3.settings.z, 1);
    assert_eq!(pass.state.m3.settings.y, 0);
    assert_eq!(pass.state.m3.settings.z, 0);
    let mut ordinary = ModelMaterialExtension::default();
    ordinary.enable_effect_color();
    assert_eq!(ordinary.m3.settings, UVec4::ZERO);
}

#[test]
fn unresolved_images_use_white_diffuse_and_neutral_optional_roles() {
    let mut slot_values = SlotValues::default();
    let mut builder = Model::builder("unresolved roles");
    let texture = builder.add_file_texture("missing.dds").unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let mut layer = constant(&mut slot_values, [1.; 4]);
    layer.binding.sampling.flags.set_color(false);
    layer.binding.image = Some(texture.resource);
    layer.binding.coordinates.source = UvMappingMode::ExplicitUV3;
    let mut surface = m3::StandardMaterialStep {
        diffuse: Some(layer.clone()),
        ..SlotConstruct::construct(&mut slot_values)
    };
    surface.normals.base = Some(layer.clone());
    surface.opacity.primary = Some(layer.clone());
    surface.ambient_occlusion_layer = Some(layer.clone());
    let mut model = model.clone();
    model
        .edit(|builder| builder.import_values(&slot_values))
        .unwrap();
    let pass = compile_m3(
        &model,
        &m3::MaterialStep::Standard(Box::new(surface)),
        &TextureImages::default(),
    )
    .unwrap();
    assert_eq!(pass.state.m3.layers[0].flags.x, 1);
    assert_eq!(pass.state.m3.layers[0].flags.y, 0);
    for slot in [6, 8, 14] {
        assert_eq!(pass.state.m3.layers[slot].flags.x, 0);
    }
    assert!(pass.material.normal_map_texture.is_none());
    assert_eq!(pass.fourth_coordinate, None);
    assert!(pass
        .diagnostics
        .iter()
        .any(|line| line.contains("unresolved")));
    assert!(pass.animation.m3.is_none());
    let mut animation = M3Animation {
        layers: from_fn(|_| None),
        weights: Vec::new(),
    };
    animation.layers[0] = Some(layer);
    let mut uniform = pass.state.m3;
    let sample = model.animation_sample(AnimationSample::default()).unwrap();
    assert!(!animation.apply(&sample, &mut uniform));
    assert_eq!(uniform.layers[0].flags.y, 0);
}

#[test]
fn independent_image_roles_share_each_effective_native_wrap_sampler() {
    let mut slot_values = SlotValues::default();
    let mut builder = Model::builder("pooled standard sampling");
    let texture = builder.add_file_texture("wrap.dds").unwrap();
    let mut layers = Vec::new();
    for (u, v) in [
        (false, false),
        (true, false),
        (false, true),
        (true, true),
        (false, false),
    ] {
        let mut layer = constant(&mut slot_values, [1.; 4]);
        layer.binding.image = Some(texture.resource);
        layer.binding.sampling.flags.set_color(false);
        layer.binding.sampling.flags.set_uv_wrap_x(u);
        layer.binding.sampling.flags.set_uv_wrap_y(v);
        layers.push(layer);
    }
    let mut surface = m3::StandardMaterialStep {
        diffuse: Some(layers[0].clone()),
        decal_layer: Some(layers[1].clone()),
        ..SlotConstruct::construct(&mut slot_values)
    };
    surface.opacity.primary = Some(layers[2].clone());
    surface.opacity.secondary = Some(layers[3].clone());
    surface.normals.base = Some(layers[4].clone());
    let surface = m3::MaterialStep::Standard(Box::new(surface));
    builder
        .add_material(Material {
            name: "wrap states".into(),
            surface: MaterialSurface::from(surface.clone()),
        })
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let images = TextureImages::resolve(&model, |_| Some(Handle::default()));
    let mut model = model.clone();
    model
        .edit(|builder| builder.import_values(&slot_values))
        .unwrap();
    let pass = compile_m3(&model, &surface, &images).unwrap();
    assert_eq!(
        [0, 1, 6, 7, 8].map(|slot| pass.state.m3.layers[slot].route.y),
        [0, 1, 2, 3, 0]
    );
    assert_eq!(pass.state.m3_samplers.iter().flatten().count(), 4);
    assert!(pass.state.m3_samplers[4..].iter().all(Option::is_none));
}

#[test]
fn animated_role_values_sample_timeline_and_preserve_compiled_image_routes() {
    let mut slot_values = SlotValues::default();
    use wc3::model::{Channel, Clock, Curve, Time as ModelTime, Track};
    let mut builder = Model::builder("animated material roles");
    let material = builder.reserve_material();
    let property = slot_values.add(1.);
    let mut opacity = constant(&mut slot_values, [1.; 4]);
    opacity.binding.contribution.map_alpha.value = property;
    let mut surface = m3::StandardMaterialStep::construct(&mut slot_values);
    surface.opacity.primary = Some(opacity);
    builder
        .set_material(
            material,
            Material {
                name: "animated".into(),
                surface: MaterialSurface::from(m3::MaterialStep::Standard(Box::new(surface))),
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
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let pass = compile_material_with_images(&model, material, Arc::new(TextureImages::default()))
        .unwrap()
        .remove(0);
    assert!(pass.animation.is_animated());
    let mut material = ModelMaterial {
        base: pass.material,
        extension: pass.state,
    };
    material.extension.m3.layers[6].route.y = 3;
    let sample = model
        .animation_sample(AnimationSample {
            timeline_time: ModelTime::seconds(0.5).unwrap(),
            ..SlotConstruct::construct(&mut slot_values)
        })
        .unwrap();
    assert!(pass.animation.apply_to_material(&sample, &mut material));
    assert!((material.extension.m3.layers[6].controls.z - 0.5).abs() < 1e-6);
    assert_eq!(material.extension.m3.layers[6].route.y, 3);
    assert!(!pass.animation.apply_to_material(&sample, &mut material));
}

#[test]
fn native_bound_constants_keep_opaque_composite_base_and_no_animation_sidecar() {
    let mut slot_values = SlotValues::default();
    use wc3::model::conversion::{ExportOptions, ImportOptions};
    use wc3::model::formats::m3::StarCraft2;
    use wc3::model::profiles::{Universal, M3};
    use wc3::model::{io, profiles::m3 as native};
    let mut builder = Model::<M3>::builder("constant composite bindings");
    let standard = native::StandardMaterialStep {
        diffuse: Some(constant(&mut slot_values, [1.; 4])),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let leaf = builder
        .add_material(Material {
            name: "constant leaf".into(),
            surface: native::MaterialStep::Standard(Box::new(standard)),
        })
        .unwrap();
    let _composite = builder
        .add_material(Material {
            name: "constant composite".into(),
            surface: native::MaterialStep::Composite(Box::new(native::CompositeMaterialStep {
                priority: 0,
                sections: [1., 0.35]
                    .map(|weight| native::CompositeSection {
                        material: leaf,
                        map_multiplier: native::Property {
                            value: slot_values.add(weight),
                            reset: weight,
                            ..SlotConstruct::construct(&mut slot_values)
                        },
                    })
                    .into(),
            })),
        })
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let native = io::m3::plan::<StarCraft2>(&model, &ExportOptions::default())
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
        .iter()
        .find_map(|(id, material)| (material.name == "constant composite").then_some(id))
        .unwrap();
    let MaterialStep::M3(m3::MaterialStep::Composite(surface)) =
        &model.material(composite).unwrap().surface.steps[0]
    else {
        panic!("composite pass")
    };
    assert!(surface
        .sections
        .iter()
        .all(|section| model.values().contains(section.map_multiplier.value.slot())));
    let passes =
        compile_material_with_images(&model, composite, Arc::new(TextureImages::default()))
            .unwrap();
    assert_eq!(passes.len(), 2);
    assert_eq!(passes[0].material.alpha_mode, AlphaMode::Opaque);
    assert_eq!(passes[1].material.alpha_mode, AlphaMode::Blend);
    assert_eq!(passes[0].material.base_color.alpha(), 1.);
    assert_eq!(passes[1].material.base_color.alpha(), 0.35);
    assert!(passes.iter().all(|pass| !pass.animation.is_animated()));
    assert!(passes.iter().all(|pass| pass.animation.m3.is_none()));
    assert_eq!(passes[0].state.m3.layers[0].color, Vec4::ONE);
    let _ = composite;
}
