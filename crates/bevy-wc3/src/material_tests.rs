use super::super::textures::TextureRole;
use super::*;
use std::array::from_fn;
use wc3::model::materials::state::{BlendOperations, Lighting, Operation};
use wc3::model::materials::{m3, MaterialSurface};
use wc3::model::profiles::Unavailable;
use wc3::model::profiles::Universal;
use wc3::model::{AnimationSample, ColorSpace, Material, ModelBuilder, Texture};

#[test]
fn default_extension_preserves_base_blend_and_depth_while_authored_pass_overrides() {
    let default = ModelMaterialKey::from(&ModelMaterialExtension::default());
    assert_eq!(default.depth_write, None);
    for initial in [
        None,
        Some(BlendState::ALPHA_BLENDING),
        Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
    ] {
        let mut actual = initial;
        default.apply_blend(&mut actual);
        assert_eq!(actual, initial);
    }
    let opaque = ModelMaterialKey::from(&SurfacePass::default().state);
    let mut actual = Some(BlendState::ALPHA_BLENDING);
    opaque.apply_blend(&mut actual);
    assert_eq!(actual, None);
    assert_eq!(opaque.depth_write, Some(true));
    let alpha = particle_material(ParticleBlend::Alpha, None, false);
    let alpha = ModelMaterialKey::from(&alpha.extension);
    alpha.apply_blend(&mut actual);
    assert_eq!(actual.unwrap().color.src_factor, BlendFactor::SrcAlpha);
    assert_eq!(alpha.depth_write, Some(false));
}

#[test]
fn basic_world_shaders_keep_black_baked_light_separate_from_surface_albedo() {
    let model = ModelBuilder::new("world surface").finish().unwrap();
    for shader in [0, 4, 16] {
        let surface = wmo_shader::Material {
            shader: Some(shader),
            diffuse: [0; 4],
            ..Default::default()
        };
        let pass = compile_wmo(&model, &surface.clone().into(), &TextureImages::default()).unwrap();
        assert_eq!(pass.material.base_color, Color::WHITE);
        assert!(!pass.material.unlit);
        assert_eq!(
            pass.state.world.flags,
            UVec4::new(1, (shader == 4) as u32, 0, 0)
        );
        assert_eq!(pass.state.world.composition, UVec4::ZERO);
        assert!(!pass
            .diagnostics
            .iter()
            .any(|value| value.contains("needs its authored texture/specular combiner")));
    }
    let unsupported = wmo_shader::Material {
        shader: Some(8),
        ..Default::default()
    };
    let pass = compile_wmo(
        &model,
        &unsupported.clone().into(),
        &TextureImages::default(),
    )
    .unwrap();
    assert!(pass
        .diagnostics
        .iter()
        .any(|value| value.contains("needs its authored texture/specular combiner")));
}

#[test]
fn default_unit_binds_roles_controls_and_preserves_static_material_sharing() {
    let mut builder = ModelBuilder::new("DefaultUnit");
    let ids = from_fn::<_, 6, _>(|slot| {
        builder
            .add_file_texture(
                [
                    "map-0.png",
                    "map-1.png",
                    "map-2.png",
                    "map-3.png",
                    "map-4.png",
                    "map-5.png",
                ][slot],
            )
            .unwrap()
    });
    let mut layer = mdlx::Layer {
        shader: Some(1),
        opacity: Animated::constant(0.8),
        ..Default::default()
    };
    layer.state.blend = BlendOperations::AlphaKey(());
    layer.binding.image = Animated::constant(Some(ids[0]));
    layer.textures = ids
        .iter()
        .enumerate()
        .map(|(slot, id)| mdlx::TextureSlot {
            slot: slot as u32,
            binding: wc3::model::materials::bindings::TextureBinding {
                image: Animated::constant(Some(*id)),
                ..Default::default()
            },
        })
        .collect();
    layer.emissive_gain = Some(Animated::constant(2.));
    layer.fresnel = Some(mdlx::Fresnel {
        color: Animated::constant([0.2, 0.4, 0.8]),
        opacity: Animated::constant(0.5),
        team_color: Animated::constant(0.3),
    });
    let id = builder
        .add_material(Material {
            name: "HD".into(),
            surface: MaterialSurface::try_from(mdlx::LayeredSurface {
                layers: vec![layer],
                ..Default::default()
            })
            .unwrap(),
        })
        .unwrap();
    let model = builder.finish().unwrap();
    let mut images = Assets::<Image>::default();
    let resolved = Arc::new(TextureImages::resolve(&model, |_| {
        Some(images.add(Image::default()))
    }));
    let mut passes = compile_material_with_images(&model, id, resolved.clone()).unwrap();
    let pass = passes.remove(0);
    assert_eq!(
        pass.material.normal_map_texture.as_ref(),
        resolved.for_role(ids[1], TextureRole::Normal)
    );
    assert_ne!(
        pass.material.normal_map_texture.as_ref(),
        resolved.for_role(ids[1], TextureRole::Color)
    );
    assert_eq!(
        pass.material.metallic_roughness_texture.as_ref(),
        resolved.for_role(ids[2], TextureRole::Data)
    );
    assert_eq!(
        pass.material.occlusion_texture,
        pass.material.metallic_roughness_texture
    );
    assert_eq!(
        pass.material.emissive_texture.as_ref(),
        resolved.for_role(ids[3], TextureRole::Color)
    );
    assert_eq!(pass.material.emissive, LinearRgba::rgb(2., 2., 2.));
    assert_eq!(pass.material.alpha_mode, AlphaMode::Mask(0.75));
    assert_eq!(pass.state.hd.maps, UVec4::ONE);
    assert_eq!(pass.state.hd.fresnel, Vec4::new(0.5, 0.3, 0., 0.));
    assert_eq!(pass.state.hd.fresnel_color, Vec4::new(0.2, 0.4, 0.8, 1.));
    assert!(!pass.animation.is_animated());
    let sample = model.animation_sample(AnimationSample::default()).unwrap();
    let mut materials = Assets::default();
    let handle = materials.add(ModelMaterial {
        base: pass.material,
        extension: pass.state,
    });
    assert!(!pass
        .animation
        .apply_to_assets(&sample, &handle, &mut materials));
    let mut changed = pass.animation.clone();
    changed.hd.as_mut().unwrap().gain = Some(Animated::constant(3.));
    assert!(changed.apply_to_assets(&sample, &handle, &mut materials));
    assert!(!changed.apply_to_assets(&sample, &handle, &mut materials));
    assert_eq!(materials.get(&handle).unwrap().base.emissive.red, 3.);
}

#[test]
fn unknown_layer_shader_does_not_infer_hd_data_roles() {
    let mut builder = ModelBuilder::new("unknown shader");
    let texture = builder.add_file_texture("unknown.png").unwrap();
    let mut layer = mdlx::Layer {
        shader: Some(55),
        ..Default::default()
    };
    layer.textures.push(mdlx::TextureSlot {
        slot: 1,
        binding: wc3::model::materials::bindings::TextureBinding {
            image: Animated::constant(Some(texture)),
            ..Default::default()
        },
    });
    let id = builder
        .add_material(Material {
            name: "unknown".into(),
            surface: MaterialSurface::try_from(mdlx::LayeredSurface {
                shader: "Shader_HD_DefaultUnit".into(),
                layers: vec![layer],
                ..Default::default()
            })
            .unwrap(),
        })
        .unwrap();
    let model = builder.finish().unwrap();
    let requests = super::super::textures::texture_requests(&model);
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].role, TextureRole::Color);
    let passes =
        compile_material_with_images(&model, id, Arc::new(TextureImages::default())).unwrap();
    assert_eq!(passes[0].state.hd.maps, UVec4::ZERO);
    assert!(passes[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("extended layered shader")));
}

#[test]
fn layered_uv_rotation_and_scale_use_center_pivot() {
    let transform = layered_uv(
        [0.; 3],
        Quat::from_rotation_z(std::f32::consts::FRAC_PI_2).to_array(),
        [2., 2., 1.],
    );
    assert!((transform.transform_point2(Vec2::splat(0.5)) - Vec2::splat(0.5)).length() < 1e-6);
    assert!((transform.transform_point2(Vec2::new(1., 0.5)) - Vec2::new(0.5, 1.5)).length() < 1e-6);
}

#[test]
fn same_resource_texture_variants_keep_independent_image_handles() {
    let mut builder = ModelBuilder::<Universal>::new("texture variants");
    let srgb = builder.add_file_texture("shared.png").unwrap();
    let mut linear = Texture::new(builder.add_image_resource(wc3::model::ImageResource::Path(
        "shared.png".as_bytes().into(),
    )));
    linear.color_space = ColorSpace::Linear;
    let linear = builder.add_texture(linear).unwrap();
    let layers = [srgb, linear]
        .map(|texture| {
            let mut layer = mdlx::Layer::default();
            layer.binding.image = Animated::constant(Some(texture));
            layer
        })
        .to_vec();
    let id = builder
        .add_material(Material {
            name: "variants".into(),
            surface: MaterialSurface::try_from(mdlx::LayeredSurface {
                layers,
                ..Default::default()
            })
            .unwrap(),
        })
        .unwrap();
    let model = builder.finish().unwrap();
    let mut assets = Assets::<Image>::default();
    let first = assets.add(Image::default());
    let second = assets.add(Image::default());
    let map = Arc::new(TextureImages::from_colors(
        &model,
        HashMap::from([(srgb, first.clone()), (linear, second.clone())]),
    ));
    let passes = compile_material_with_images(&model, id, map).unwrap();
    assert_eq!(passes[0].material.base_color_texture, Some(first));
    assert_eq!(passes[1].material.base_color_texture, Some(second));
}

#[test]
fn m3_normal_and_diffuse_uses_of_one_image_select_distinct_interpretations() {
    let mut builder = ModelBuilder::<Universal>::new("normal interpretation");
    let id = builder.add_file_texture("shared.png").unwrap();
    let mut layer = m3::TextureLayer::default();
    layer.binding.image = Some(id);
    let mut standard = m3::StandardMaterialStep {
        diffuse: Some(layer.clone()),
        ..Default::default()
    };
    standard.normals.base = Some(layer);
    let material = builder
        .add_material(Material {
            name: "standard".into(),
            surface: MaterialSurface::from(m3::MaterialStep::Standard(Box::new(standard))),
        })
        .unwrap();
    let model = builder.finish().unwrap();
    let mut assets = Assets::<Image>::default();
    let color = assets.add(Image::default());
    let normal = assets.add(Image::default());
    let images = TextureImages::resolve(&model, |request| {
        Some(if request.color_space.is_srgb() {
            color.clone()
        } else {
            normal.clone()
        })
    });
    let passes = compile_material_with_images(&model, material, Arc::new(images)).unwrap();
    assert_eq!(passes[0].material.base_color_texture, Some(color));
    assert_eq!(passes[0].material.normal_map_texture, Some(normal));
}

#[test]
fn particle_blend_separates_unweighted_and_opacity_weighted_addition() {
    let add = particle_material(ParticleBlend::NoAlphaAdd, None, false);
    let alpha_add = particle_material(ParticleBlend::Add, None, false);
    assert_eq!(add.extension.blend, Some(Blend::Additive));
    assert_eq!(alpha_add.extension.blend, Some(Blend::AdditiveAlpha));
    assert_eq!(alpha_add.extension.combiner.clip.x, 0.0);
    assert!(!ModelMaterialKey::from(&add.extension).depth_write.unwrap());
    let cutout = particle_material(ParticleBlend::AlphaKey, None, true);
    assert_eq!(
        ModelMaterialKey::from(&cutout.extension).depth_write,
        Some(true)
    );
    assert!(!cutout.base.unlit);
}

#[test]
fn legacy_selectors_keep_texture_alpha_roles_independent() {
    use crate::material::native_fixtures::m2 as m2_materials;
    let combiner = |shader, count| {
        let step = m2_materials::Material {
            shader,
            textures: vec![Default::default(); count],
            ..Default::default()
        }
        .into();
        super::m2::program(&step).map(|program| program.mode)
    };
    assert_eq!(combiner(0, 1), Some(UVec4::new(1, 0, 0, 0)));
    assert_eq!(combiner(0x10, 1), Some(UVec4::new(1, 1, 0, 0)));
    assert_eq!(combiner(4, 2), Some(UVec4::new(1, 0, 1, 1)));
    assert_eq!(combiner(0x16, 2), Some(UVec4::new(1, 1, 0, 1)));
    assert_eq!(combiner(3, 2), Some(UVec4::new(16, 0, 0, 0)));
    assert!(combiner(0xffff, 2).is_none());
}

#[test]
fn sampling_static_bindings_does_not_request_repeated_asset_writes() {
    let model = ModelBuilder::<Universal>::new("material sampling")
        .finish()
        .unwrap();
    let sampled = model.animation_sample(AnimationSample::default()).unwrap();
    let mut material = StandardMaterial::default();
    let animation = SurfaceAnimation {
        color: Some(Animated::constant([0.25, 0.5, 0.75])),
        opacity: Some(Animated::constant(0.6)),
        weight: Some(Animated::constant(0.5)),
        ..Default::default()
    };
    assert!(animation.apply(&sampled, &mut material));
    assert!(!animation.apply(&sampled, &mut material));
    assert_eq!(
        material.base_color,
        Color::linear_rgba(0.25, 0.5, 0.75, 0.3)
    );
}

#[test]
fn cutout_preserves_depth_and_uses_the_documented_threshold() {
    let mut pass = SurfacePass::default();
    let operation: mdlx::Blend = BlendOperations::AlphaKey(());
    blend(&operation, &mut pass);
    depth(&mut pass, Operation::Enabled, Operation::Enabled);
    assert_eq!(pass.material.alpha_mode, AlphaMode::Mask(128.0 / 255.0));
    assert!(pass.diagnostics.is_empty());
}

#[test]
fn blend_and_depth_policies_remain_independent() {
    let mut pass = SurfacePass::default();
    let operation: mdlx::Blend = BlendOperations::Modulate2x;
    blend(&operation, &mut pass);
    depth(&mut pass, Operation::Disabled, Operation::Enabled);
    assert_eq!(pass.material.alpha_mode, AlphaMode::Blend);
    assert!(pass.diagnostics.is_empty());
    let state = ModelMaterialKey::from(&pass.state);
    assert_eq!(state.blend, Some(Blend::Modulate2x));
    assert!(!state.depth_test);
    assert_eq!(state.depth_write, Some(true));
}

#[test]
fn unlit_and_unfogged_are_independent() {
    let mut pass = SurfacePass::default();
    rasterization(
        &mut pass,
        true,
        Lighting {
            unlit: true,
            unfogged: false,
        },
    );
    assert!(pass.material.unlit);
    assert!(pass.material.fog_enabled);
    assert!(pass.material.double_sided);
    assert_eq!(pass.material.cull_mode, None);
}

#[test]
fn unknown_blending_does_not_silently_claim_opaque_support() {
    let mut pass = SurfacePass::default();
    let operation: BlendOperations<Unavailable> = BlendOperations::Unknown(999);
    blend(&operation, &mut pass);
    assert_eq!(pass.material.alpha_mode, AlphaMode::Blend);
    assert!(pass.diagnostics[0].contains("999"));
}

#[test]
fn live_guild_palette_updates_uniforms_without_changing_pipeline_identity() {
    let palette = GuildPalette {
        background: [0.2, 0.4, 0.8],
        emblem: [2., 0.5, 0.1],
        border: [0.1, 0.2, 0.3],
    };
    let mut material = ModelMaterialExtension::default();
    assert_eq!(material.guild_palette(), None);
    assert!(!material.set_guild_palette(palette).unwrap());
    for mode in 24..=26 {
        material.combiner.mode.x = mode;
        let before = ModelMaterialKey::from(&material);
        assert_eq!(material.guild_palette(), Some(GuildPalette::default()));
        assert!(material.set_guild_palette(palette).unwrap());
        assert_eq!(material.guild_palette(), Some(palette));
        let after = ModelMaterialKey::from(&material);
        assert_eq!(before.generated_coordinates, after.generated_coordinates);
        assert_eq!(before.third_uv, after.third_uv);
        assert_eq!(before.blend, after.blend);
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut invalid_palette = palette;
            invalid_palette.border[2] = invalid;
            assert!(material.set_guild_palette(invalid_palette).is_err());
            assert_eq!(material.guild_palette(), Some(palette));
        }
        material.set_guild_palette(GuildPalette::default()).unwrap();
    }
}

#[test]
fn nested_composites_expand_ordered_passes_and_multiply_opacity_once() {
    let mut builder = ModelBuilder::<Universal>::new("nested composite");
    let leaf = builder
        .add_material(Material {
            name: "leaf".into(),
            surface: MaterialSurface::from(m3::MaterialStep::Standard(Box::default())),
        })
        .unwrap();
    let inner = builder
        .add_material(Material {
            name: "inner".into(),
            surface: MaterialSurface::from(m3::MaterialStep::Composite(Box::new(
                m3::CompositeMaterialStep {
                    priority: 3,
                    sections: vec![m3::CompositeSection {
                        material: leaf,
                        map_multiplier: m3::Property {
                            initial: 0.5,
                            ..Default::default()
                        },
                    }],
                },
            ))),
        })
        .unwrap();
    let outer = builder
        .add_material(Material {
            name: "outer".into(),
            surface: MaterialSurface::from(m3::MaterialStep::Composite(Box::new(
                m3::CompositeMaterialStep {
                    priority: 7,
                    sections: vec![
                        m3::CompositeSection {
                            material: inner,
                            map_multiplier: m3::Property {
                                initial: 0.25,
                                ..Default::default()
                            },
                        },
                        m3::CompositeSection {
                            material: leaf,
                            map_multiplier: m3::Property {
                                initial: 1.,
                                ..Default::default()
                            },
                        },
                    ],
                },
            ))),
        })
        .unwrap();
    let model = builder.finish().unwrap();
    let passes =
        compile_material_with_images(&model, outer, Arc::new(TextureImages::default())).unwrap();
    assert_eq!(passes.len(), 2);
    assert_eq!(passes[0].material.base_color.alpha(), 0.125);
    assert_eq!(passes[0].material.alpha_mode, AlphaMode::Blend);
    assert_eq!(passes[1].material.base_color.alpha(), 1.);
    assert_eq!(passes[1].material.alpha_mode, AlphaMode::Opaque);
    assert!(passes.iter().all(|pass| pass.material.depth_bias == 7.));
    let sample = model.animation_sample(AnimationSample::default()).unwrap();
    let mut passes = passes.into_iter();
    let pass = passes.next().unwrap();
    let animation =
        pass.animation
            .with_mesh(pass.material.base_color, pass.material.alpha_mode, None);
    let mut material = ModelMaterial {
        base: pass.material,
        extension: pass.state,
    };
    animation.apply_to_material(&sample, &mut material);
    assert_eq!(material.base.base_color.alpha(), 0.125);
    assert!(!animation.apply_to_material(&sample, &mut material));
}

#[test]
fn m3_shader_family_is_static_and_uniform_edits_reuse_its_pipeline() {
    let ordinary = ModelMaterialExtension::default();
    let ordinary_key = ModelMaterialKey::from(&ordinary);
    assert!(!ordinary_key.m3);
    let mut standard = ordinary;
    standard.m3.settings.x = 1;
    let standard_key = ModelMaterialKey::from(&standard);
    assert!(standard_key.m3);
    assert_ne!(ordinary_key, standard_key);
    standard.m3.diffuse_team = Vec4::new(0.2, 0.4, 0.8, 1.);
    standard.m3.layers[0].controls.z = 0.25;
    standard.enable_effect_color();
    assert_eq!(ModelMaterialKey::from(&standard), standard_key);
}

#[test]
fn mixed_m3_external_and_composite_steps_keep_their_order_and_weights() {
    use wc3::model::materials::{diablo3, diablo4};
    use wc3::model::MaterialStep;

    let mut builder = ModelBuilder::new("mixed shader execution");
    let mut external = diablo3::MaterialStep::default();
    external.uber.colors.diffuse = [0.2, 0.4, 0.6, 0.8];
    let child = builder
        .add_material(Material {
            name: "external child".into(),
            surface: external.into(),
        })
        .unwrap();
    let id = builder
        .add_material(Material {
            name: "mixed stack".into(),
            surface: MaterialSurface {
                steps: vec![
                    MaterialStep::M3(m3::MaterialStep::Standard(Box::default())),
                    MaterialStep::M3(m3::MaterialStep::Composite(Box::new(
                        m3::CompositeMaterialStep {
                            priority: 13,
                            sections: vec![m3::CompositeSection {
                                material: child,
                                map_multiplier: m3::Property {
                                    initial: 0.5,
                                    ..Default::default()
                                },
                            }],
                        },
                    ))),
                    MaterialStep::Diablo4(Box::new(diablo4::MaterialStep {
                        shader_map: 42,
                        ..Default::default()
                    })),
                    MaterialStep::M3(m3::MaterialStep::Volume(Box::default())),
                ],
            },
        })
        .unwrap();
    let model = builder.finish().unwrap();
    let passes =
        compile_material_with_images(&model, id, Arc::new(TextureImages::default())).unwrap();
    assert_eq!(passes.len(), 4);
    assert_eq!(passes[0].state.m3.settings.x, 1);
    assert!((passes[1].material.base_color.alpha() - 0.4).abs() < 1e-6);
    assert_eq!(passes[1].material.depth_bias, 13.);
    assert_eq!(passes[1].animation.pass_weights.len(), 1);
    assert!(passes[2]
        .diagnostics
        .iter()
        .any(|v| v.contains("shader map 42")));
    assert!(passes[3].diagnostics.iter().any(|v| v.contains("Volume")));
}

#[test]
fn layered_passes_compile_independent_culling_priority_and_sorting() {
    use wc3::model::materials::mdlx::{
        ClassicMaterialStep, MaterialDeclaration, MaterialStep as MdlxMaterialStep,
    };
    use wc3::model::materials::program::MaterialStep;
    let mut builder = ModelBuilder::new("independent layered settings");
    let id = builder
        .add_material(Material {
            name: "mixed".into(),
            surface: MaterialSurface {
                steps: vec![
                    MaterialStep::Mdlx(Box::new(MdlxMaterialStep::Classic(ClassicMaterialStep {
                        declaration: MaterialDeclaration {
                            priority_plane: -3,
                            two_sided: true,
                            sort_far_z: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    }))),
                    MaterialStep::M3(m3::MaterialStep::Standard(Box::default())),
                    MaterialStep::Mdlx(Box::new(MdlxMaterialStep::Classic(ClassicMaterialStep {
                        declaration: MaterialDeclaration {
                            priority_plane: 9,
                            sort_near_z: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    }))),
                ],
            },
        })
        .unwrap();
    let model = builder.finish().unwrap();
    let passes =
        compile_material_with_images(&model, id, Arc::new(TextureImages::default())).unwrap();
    assert_eq!(passes.len(), 3);
    assert_eq!(passes[0].material.depth_bias, -3.);
    assert!(passes[0].material.cull_mode.is_none());
    assert!(passes[0].sort_far && !passes[0].sort_near);
    assert_eq!(passes[2].material.depth_bias, 9.);
    assert!(passes[2].material.cull_mode.is_some());
    assert!(passes[2].sort_near && !passes[2].sort_far);
}

#[test]
fn unresolved_diablo_placeholder_uses_appearance_variant_ownership() {
    use wc3::model::appearance::{diablo4, UniversalAppearance};
    let mut builder = ModelBuilder::new("unresolved Diablo selection");
    let linked = builder
        .add_material(Material {
            name: "external".into(),
            surface: Default::default(),
        })
        .unwrap();
    let empty = builder
        .add_material(Material {
            name: "empty".into(),
            surface: Default::default(),
        })
        .unwrap();
    let mut appearance = diablo4::Appearance::default();
    let slot = appearance.add_slot(diablo4::Slot::default());
    appearance.add_variant(slot, linked).unwrap();
    builder.set_appearance(UniversalAppearance::D4(appearance));
    let model = builder.finish().unwrap();
    let images = Arc::new(TextureImages::default());
    let passes = compile_material_with_images(&model, linked, images.clone()).unwrap();
    assert_eq!(passes.len(), 1);
    assert!(passes[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("external material shader is unresolved")));
    assert!(compile_material_with_images(&model, empty, images)
        .unwrap()
        .is_empty());
}

#[test]
fn m2_material_program_compiles_steps_in_order_with_independent_controls() {
    use wc3::model::materials::m2::{DiffuseMaterialStep, MaterialStep as M2Step};
    use wc3::model::profiles::m2::Material as M2Material;
    use wc3::model::MaterialProgram;

    let mut builder = ModelBuilder::new("M2 layered material");
    let mut first = DiffuseMaterialStep::default();
    first.declaration.color = Animated::constant([1., 0., 0.]);
    first.declaration.opacity = Animated::constant(0.25);
    first.declaration.state.ordering = 3;
    first.diffuse.contribution = Animated::constant(1.);
    let mut second = DiffuseMaterialStep::default();
    second.declaration.color = Animated::constant([0., 1., 0.]);
    second.declaration.opacity = Animated::constant(0.75);
    second.declaration.state.ordering = -2;
    second.diffuse.contribution = Animated::constant(1.);
    let program: M2Material<'_> = MaterialProgram {
        steps: vec![M2Step::Diffuse(first), M2Step::Diffuse(second)],
    };
    let id = builder
        .add_material(Material {
            name: "two steps".into(),
            surface: program.into(),
        })
        .unwrap();
    let model = builder.finish().unwrap();
    let passes =
        compile_material_with_images(&model, id, Arc::new(TextureImages::default())).unwrap();
    assert_eq!(passes.len(), 2);
    assert_eq!(
        passes[0].material.base_color.to_linear().to_f32_array(),
        [1., 0., 0., 0.25]
    );
    assert_eq!(
        passes[1].material.base_color.to_linear().to_f32_array(),
        [0., 1., 0., 0.75]
    );
    assert_eq!(passes[0].material.depth_bias, 3.);
    assert_eq!(passes[1].material.depth_bias, -2.);
}
