use super::super::textures::TextureRole;
use super::compile::rasterization;
use super::*;
use bevy::math::{Quat, Vec2, Vec3, Vec4};
use std::array::from_fn;
use wc3::model::materials::state::{BlendOperations, Lighting, Operation};
use wc3::model::materials::MaterialSurface;
use wc3::model::profiles::Unavailable;
use wc3::model::profiles::Universal;
use wc3::model::ModelMetadata;
use wc3::model::{AnimationSample, ColorSpace, Material, ModelBuilder, Texture};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
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
}

#[test]
fn basic_world_shaders_keep_black_baked_light_separate_from_surface_albedo() {
    let mut slot_values = SlotValues::default();
    let model = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()))
        .finish()
        .unwrap();
    for shader in [0, 4, 16] {
        let surface = wmo_shader::Material {
            shader: Some(shader),
            diffuse: [0; 4],
            ..SlotConstruct::construct(&mut slot_values)
        };
        let pass = compile_wmo(&model, &surface.clone().into(), &TextureImages::default()).unwrap();
        assert_eq!(pass.material.base_color, Color::WHITE);
        assert!(!pass.material.unlit);
        assert_eq!(
            pass.state.world.flags,
            UVec4::new(1, (shader == 4) as u32, 0, 0)
        );
        assert_eq!(pass.state.world.composition, UVec4::ZERO);
        assert!(!pass.diagnostics.iter().any(|value| value
            .message
            .contains("needs its authored texture/specular combiner")));
    }
    let unsupported = wmo_shader::Material {
        shader: Some(8),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let pass = compile_wmo(
        &model,
        &unsupported.clone().into(),
        &TextureImages::default(),
    )
    .unwrap();
    assert!(pass.diagnostics.iter().any(|value| value
        .message
        .contains("needs its authored texture/specular combiner")));
}

#[test]
fn default_unit_binds_roles_controls_and_preserves_static_material_sharing() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
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
    let mut layer = mdlx::MaterialStep::construct(&mut slot_values);
    let mut hd = mdlx::ReforgedInputs::construct(&mut slot_values);
    hd.inputs.opacity = slot_values.add(0.8);
    layer.state.blend = BlendOperations::AlphaKey(());
    hd.inputs.binding.image = slot_values.add(Some(ids[0]));
    hd.textures = ids
        .iter()
        .enumerate()
        .map(|(slot, id)| mdlx::TextureSlot {
            slot: slot as u32,
            binding: wc3::model::materials::bindings::TextureBinding {
                image: slot_values.add(Some(*id)),
                ..SlotConstruct::construct(&mut slot_values)
            },
        })
        .collect();
    hd.inputs.emissive_gain = Some(slot_values.add(2.));
    hd.fresnel = Some(mdlx::Fresnel {
        color: slot_values.add(Vec3::from_array([0.2, 0.4, 0.8])),
        opacity: slot_values.add(0.5),
        team_color: slot_values.add(0.3),
    });
    layer.shader = mdlx::Shader::ReforgedDefault(hd);
    let id = builder
        .add_material(Material {
            name: "HD".into(),
            surface: MaterialSurface::from(wc3::model::profiles::mdlx::Material {
                declaration: Default::default(),
                steps: vec![layer],
            }),
        })
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let mut images = Assets::<Image>::default();
    let resolved = Arc::new(TextureImages::resolve(&model, |_| {
        Some(images.add(Image::default()))
    }));
    let mut passes = compile_material_with_images(&model, id, resolved.clone()).unwrap();
    let pass = passes.remove(0);
    assert_eq!(
        pass.material.normal_map_texture.as_ref(),
        resolved
            .material(id, 0)
            .for_slot(1, ids[1], TextureRole::Normal)
    );
    assert_ne!(
        pass.material.normal_map_texture.as_ref(),
        resolved.for_role(ids[1], TextureRole::Color)
    );
    assert_eq!(
        pass.material.metallic_roughness_texture.as_ref(),
        resolved
            .material(id, 0)
            .for_slot(2, ids[2], TextureRole::Data)
    );
    assert_eq!(
        pass.material.occlusion_texture,
        pass.material.metallic_roughness_texture
    );
    assert_eq!(
        pass.material.emissive_texture.as_ref(),
        resolved
            .material(id, 0)
            .for_slot(3, ids[3], TextureRole::Color)
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
    changed.hd.as_mut().unwrap().gain = Some(slot_values.add(3.));
    let mut changed_model = model.clone();
    changed_model
        .edit(|builder| builder.import_values(&slot_values))
        .unwrap();
    let sample = changed_model
        .animation_sample(AnimationSample::default())
        .unwrap();
    assert!(changed.apply_to_assets(&sample, &handle, &mut materials));
    assert!(!changed.apply_to_assets(&sample, &handle, &mut materials));
    assert_eq!(materials.get(&handle).unwrap().base.emissive.red, 3.);
}

#[test]
fn unknown_layer_shader_does_not_infer_hd_data_roles() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let texture = builder.add_file_texture("unknown.png").unwrap();
    let mut layer = mdlx::MaterialStep::construct(&mut slot_values);
    let mut unresolved = mdlx::UnresolvedInputs::construct(&mut slot_values);
    unresolved.selector = Some(55);
    unresolved.textures.push(mdlx::TextureSlot {
        slot: 1,
        binding: wc3::model::materials::bindings::TextureBinding {
            image: slot_values.add(Some(texture)),
            ..SlotConstruct::construct(&mut slot_values)
        },
    });
    layer.shader = mdlx::Shader::Unresolved(unresolved);
    let id = builder
        .add_material(Material {
            name: "unknown".into(),
            surface: MaterialSurface::from(wc3::model::profiles::mdlx::Material {
                declaration: mdlx::MaterialDeclaration {
                    shader_name: "Shader_HD_DefaultUnit".into(),
                    ..Default::default()
                },
                steps: vec![layer],
            }),
        })
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let requests = super::super::textures::texture_requests(&model);
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].role, TextureRole::Color);
    let passes =
        compile_material_with_images(&model, id, Arc::new(TextureImages::default())).unwrap();
    assert_eq!(passes[0].state.hd.maps, UVec4::ZERO);
    assert!(passes[0]
        .diagnostics
        .iter()
        .any(|message| message.message.contains("extended layered shader")));
}

#[test]
fn layered_uv_rotation_and_scale_use_center_pivot() {
    let transform = layered_uv(
        Vec3::ZERO,
        Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        Vec3::new(2., 2., 1.),
    );
    assert!((transform.transform_point2(Vec2::splat(0.5)) - Vec2::splat(0.5)).length() < 1e-6);
    assert!((transform.transform_point2(Vec2::new(1., 0.5)) - Vec2::new(0.5, 1.5)).length() < 1e-6);
}

#[test]
fn same_resource_texture_variants_keep_independent_image_handles() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::<Universal>::new(ModelMetadata::Mdlx(Default::default()));
    let srgb = builder.add_file_texture("shared.png").unwrap();
    let mut linear = Texture::new(builder.add_image_resource(wc3::model::ImageResource::Path(
        "shared.png".as_bytes().into(),
    )));
    linear.color_space = ColorSpace::Linear;

    let layers = [srgb, linear]
        .map(|texture| {
            let mut layer = mdlx::MaterialStep::construct(&mut slot_values);
            layer.shader.inputs_mut().binding.image = slot_values.add(Some(texture));
            layer
        })
        .to_vec();
    let id = builder
        .add_material(Material {
            name: "variants".into(),
            surface: MaterialSurface::from(wc3::model::profiles::mdlx::Material {
                declaration: Default::default(),
                steps: layers,
            }),
        })
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
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
fn legacy_selectors_keep_texture_alpha_roles_independent() {
    let mut slot_values = SlotValues::default();
    use crate::material::native_fixtures::m2 as m2_materials;
    let mut combiner = |shader, count| {
        let step = m2_materials::Material {
            shader,
            textures: vec![SlotConstruct::construct(&mut slot_values); count],
            ..SlotConstruct::construct(&mut slot_values)
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
    let mut slot_values = SlotValues::default();
    let mut material = StandardMaterial::default();
    let animation = SurfaceAnimation {
        color: Some(slot_values.add(Vec3::from_array([0.25, 0.5, 0.75]))),
        opacity: Some(slot_values.add(0.6)),
        weight: Some(slot_values.add(0.5)),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let model = {
        let mut builder = ModelBuilder::<Universal>::new(ModelMetadata::Mdlx(Default::default()));
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let sampled = model.animation_sample(AnimationSample::default()).unwrap();
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
    assert!(pass.diagnostics[0].message.contains("999"));
}

#[test]
fn live_guild_palette_updates_uniforms_without_changing_pipeline_identity() {
    let palette = GuildPalette {
        background: Vec3::from_array([0.2, 0.4, 0.8]),
        emblem: Vec3::from_array([2., 0.5, 0.1]),
        border: Vec3::from_array([0.1, 0.2, 0.3]),
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
fn m2_material_program_compiles_steps_in_order_with_independent_controls() {
    let mut slot_values = SlotValues::default();
    use wc3::model::materials::m2::{DiffuseMaterialStep, MaterialStep as M2Step};
    use wc3::model::profiles::m2::Material as M2Material;
    use wc3::model::MaterialProgram;

    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let mut first = DiffuseMaterialStep::construct(&mut slot_values);
    first.declaration.color = slot_values.add(Vec3::from_array([1., 0., 0.]));
    first.declaration.opacity = slot_values.add(0.25);
    first.declaration.state.ordering = 3;
    first.diffuse.contribution = slot_values.add(1.);
    let mut second = DiffuseMaterialStep::construct(&mut slot_values);
    second.declaration.color = slot_values.add(Vec3::from_array([0., 1., 0.]));
    second.declaration.opacity = slot_values.add(0.75);
    second.declaration.state.ordering = -2;
    second.diffuse.contribution = slot_values.add(1.);
    let program: M2Material<'_> = MaterialProgram {
        declaration: Default::default(),
        steps: vec![M2Step::Diffuse(first), M2Step::Diffuse(second)],
    };
    let id = builder
        .add_material(Material {
            name: "two steps".into(),
            surface: program.into(),
        })
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
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
