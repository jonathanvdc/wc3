use super::*;
use wc3::model::profiles::Unavailable;
use wc3::model::{AnimationSample, ColorSpace, Material, ModelBuilder, Texture};

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
    let add = particle_material(ParticleBlend::Add, None, false);
    let alpha_add = particle_material(ParticleBlend::AddAlphaKey, None, false);
    assert_eq!(add.extension.blend, UniversalBlend::Additive);
    assert_eq!(alpha_add.extension.blend, UniversalBlend::AdditiveAlpha);
    assert_eq!(alpha_add.extension.combiner.clip.x, 1.0 / 255.0);
    assert!(!UniversalMaterialKey::from(&add.extension).depth_write);
    let cutout = particle_material(ParticleBlend::AlphaKey, None, true);
    assert!(UniversalMaterialKey::from(&cutout.extension).depth_write);
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
    let state = UniversalMaterialKey::from(&pass.state);
    assert_eq!(state.blend, UniversalBlend::Modulate2x);
    assert!(!state.depth_test);
    assert!(state.depth_write);
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
