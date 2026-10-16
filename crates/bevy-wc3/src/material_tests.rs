use super::super::textures::TextureRole;
use super::*;
use std::array::from_fn;
use wc3::model::materials::state::{BlendOperations, Lighting, Operation};
use wc3::model::materials::{programmed, SurfaceSet};
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
        let surface = world_surface::Material {
            shader: Some(shader),
            diffuse: [0; 4],
            ..Default::default()
        };
        let pass = compile_world(&model, &surface, &TextureImages::default()).unwrap();
        assert_eq!(pass.material.base_color, Color::WHITE);
        assert!(!pass.material.unlit);
        assert_eq!(
            pass.state.world.flags,
            UVec4::new(1, (shader == 4) as u32, 0, 0)
        );
        assert!(pass
            .diagnostics
            .iter()
            .any(|value| value.contains("vertex RGB supplies additive baked")));
        assert!(!pass
            .diagnostics
            .iter()
            .any(|value| value.contains("needs its authored texture/specular combiner")));
    }
    let unsupported = world_surface::Material {
        shader: Some(6),
        ..Default::default()
    };
    let pass = compile_world(&model, &unsupported, &TextureImages::default()).unwrap();
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
            .add_texture(Texture::file(
                [
                    "map-0.png",
                    "map-1.png",
                    "map-2.png",
                    "map-3.png",
                    "map-4.png",
                    "map-5.png",
                ][slot],
            ))
            .unwrap()
    });
    let mut layer = layered::Layer {
        shader: Some(1),
        opacity: Animated::constant(0.8),
        ..Default::default()
    };
    layer.state.blend = BlendOperations::AlphaKey(());
    layer.binding.image = Animated::constant(Some(ids[0]));
    layer.textures = ids
        .iter()
        .enumerate()
        .map(|(slot, id)| layered::TextureSlot {
            slot: slot as u32,
            binding: wc3::model::materials::bindings::TextureBinding {
                image: Animated::constant(Some(*id)),
                ..Default::default()
            },
        })
        .collect();
    layer.emissive_gain = Some(Animated::constant(2.));
    layer.fresnel = Some(layered::Fresnel {
        color: Animated::constant([0.2, 0.4, 0.8]),
        opacity: Animated::constant(0.5),
        team_color: Animated::constant(0.3),
    });
    let id = builder
        .add_material(Material {
            name: "HD".into(),
            surface: SurfaceSet::Layered(Box::new(layered::LayeredSurface {
                layers: vec![layer],
                ..Default::default()
            })),
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
    let texture = builder.add_texture(Texture::file("unknown.png")).unwrap();
    let mut layer = layered::Layer {
        shader: Some(55),
        ..Default::default()
    };
    layer.textures.push(layered::TextureSlot {
        slot: 1,
        binding: wc3::model::materials::bindings::TextureBinding {
            image: Animated::constant(Some(texture)),
            ..Default::default()
        },
    });
    let id = builder
        .add_material(Material {
            name: "unknown".into(),
            surface: SurfaceSet::Layered(Box::new(layered::LayeredSurface {
                shader: "Shader_HD_DefaultUnit".into(),
                layers: vec![layer],
                ..Default::default()
            })),
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
    let srgb = builder.add_texture(Texture::file("shared.png")).unwrap();
    let mut linear = Texture::file("shared.png");
    linear.color_space = ColorSpace::Linear;
    let linear = builder.add_texture(linear).unwrap();
    let layers = [srgb, linear]
        .map(|texture| {
            let mut layer = layered::Layer::default();
            layer.binding.image = Animated::constant(Some(texture));
            layer
        })
        .to_vec();
    let id = builder
        .add_material(Material {
            name: "variants".into(),
            surface: SurfaceSet::Layered(Box::new(layered::LayeredSurface {
                layers,
                ..Default::default()
            })),
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
    let id = builder.add_texture(Texture::file("shared.png")).unwrap();
    let mut layer = programmed::TextureLayer::default();
    layer.binding.image = Some(id);
    let mut standard = programmed::Standard::default();
    standard.diffuse.input = Some(layer.clone());
    standard.normals.base = Some(layer);
    let material = builder
        .add_material(Material {
            name: "standard".into(),
            surface: SurfaceSet::Programmed(Box::new(programmed::Material::Standard(Box::new(
                standard,
            )))),
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
    assert_eq!(legacy_combiner(0, 1), Some(UVec4::new(1, 0, 0, 0)));
    assert_eq!(legacy_combiner(0x10, 1), Some(UVec4::new(1, 1, 0, 0)));
    assert_eq!(legacy_combiner(4, 2), Some(UVec4::new(1, 0, 1, 1)));
    assert_eq!(legacy_combiner(0x16, 2), Some(UVec4::new(1, 1, 0, 1)));
    assert!(legacy_combiner(0x8000, 2).is_none());
    assert!(legacy_combiner(3, 2).is_none());
    assert!(legacy_combiner(0x80, 1).is_none());
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
    let operation: layered::Blend = BlendOperations::AlphaKey(());
    blend(&operation, &mut pass);
    depth(&mut pass, Operation::Enabled, Operation::Enabled);
    assert_eq!(pass.material.alpha_mode, AlphaMode::Mask(128.0 / 255.0));
    assert!(pass.diagnostics.is_empty());
}

#[test]
fn blend_and_depth_policies_remain_independent() {
    let mut pass = SurfacePass::default();
    let operation: layered::Blend = BlendOperations::Modulate2x;
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
