use super::*;
use wc3::model::{Material, MaterialSurface, Texture};

fn constant_specular_surface() -> programmed::Standard {
    let mut layer = programmed::TextureLayer::default();
    layer.binding.sampling.flags.set_color(true);
    layer.binding.contribution.color.initial = [0.2, 0.4, 0.8, 0.5];
    layer.binding.contribution.color_type = ColorChannel::Rgba;
    layer.binding.contribution.map_alpha.initial = 1.;
    let mut surface = programmed::Standard::default();
    surface.specular.input.color = Some(layer);
    surface.specular.parameters.exponent = 30.;
    surface.specular.parameters.hdr_multiplier = 0.5;
    surface
}

#[test]
fn constant_specular_retains_tint_and_monotonic_highlight_width() {
    let mut surface = constant_specular_surface();
    let mut broad = SurfacePass::default();
    constant_specular(&surface, &mut broad);
    assert!((broad.material.reflectance - 0.4).abs() < 1e-6);
    assert_eq!(
        broad.material.specular_tint,
        Color::linear_rgb(0.25, 0.5, 1.)
    );
    assert!((broad.material.perceptual_roughness - 0.5).abs() < 1e-6);
    assert!(broad
        .diagnostics
        .iter()
        .any(|message| message.contains("approximation")));

    surface.specular.parameters.exponent = 126.;
    let mut sharp = SurfacePass::default();
    constant_specular(&surface, &mut sharp);
    assert!(sharp.material.perceptual_roughness < broad.material.perceptual_roughness);
}

#[test]
fn alpha_only_specular_is_achromatic_and_unknown_modes_are_not_reinterpreted() {
    let mut surface = constant_specular_surface();
    surface.specular.parameters.mode = SpecularMode::AlphaOnly;
    let mut pass = SurfacePass::default();
    constant_specular(&surface, &mut pass);
    assert_eq!(pass.material.reflectance, 0.25);
    assert_eq!(pass.material.specular_tint, Color::linear_rgb(1., 1., 1.));

    surface.specular.parameters.mode = SpecularMode::Unknown(99);
    let mut unknown = SurfacePass::default();
    constant_specular(&surface, &mut unknown);
    assert_eq!(unknown.material.reflectance, 0.);
    assert!(unknown
        .diagnostics
        .iter()
        .any(|message| message.contains("unresolved")));
}

#[test]
fn occlusion_only_accepts_unmodified_red_channel_sampling() {
    let mut layer = programmed::TextureLayer::default();
    layer.binding.coordinates.transform.uv_tiling.initial = [1.; 2];
    layer.binding.coordinates.transform.uv_source_related = u32::MAX;
    layer.binding.contribution.map_alpha.initial = 1.;
    assert!(simple_red_data_layer(&layer));
    layer.binding.contribution.color_type = ColorChannel::Alpha;
    assert!(!simple_red_data_layer(&layer));
    layer.binding.contribution.color_type = ColorChannel::Red;
    layer.binding.coordinates.source = UvMappingMode::ReflectSphericalEnvio;
    assert!(!simple_red_data_layer(&layer));
    layer.binding.coordinates.source = UvMappingMode::ExplicitUV0;
    layer.binding.sampling.flags.set_color_invert(true);
    assert!(!simple_red_data_layer(&layer));
}

#[test]
fn packed_normals_and_occlusion_use_independent_data_roles() {
    let mut builder = Model::builder("M3 surface roles");
    let texture = builder.add_texture(Texture::file("packed.dds")).unwrap();
    let mut layer = programmed::TextureLayer::default();
    layer.binding.image = Some(texture);
    layer.binding.coordinates.transform.uv_tiling.initial = [1.; 2];
    layer.binding.coordinates.transform.uv_source_related = u32::MAX;
    layer.binding.contribution.map_alpha.initial = 1.;
    let mut surface = programmed::Standard::default();
    surface.state.rasterization.flags.set_two_sided(true);
    surface.normals.base = Some(layer.clone());
    surface.ambient_occlusion_layer = Some(layer);
    let surface = programmed::Material::Standard(Box::new(surface));
    builder
        .add_material(Material {
            name: "native roles".into(),
            surface: MaterialSurface::Programmed(Box::new(surface.clone())),
        })
        .unwrap();
    let model = builder.finish().unwrap();
    let handle = Handle::<Image>::default();
    let images = TextureImages::resolve(&model, |_| Some(handle.clone()));
    let pass = compile_programmed(&model, &surface, &images).unwrap();
    assert_eq!(pass.material.normal_map_texture, Some(handle.clone()));
    assert_eq!(pass.material.occlusion_texture, Some(handle));
    assert_eq!(pass.state.hd.maps.x, 2);
    assert!(pass.animation.hd.is_none());
    assert!(pass.material.double_sided);
    assert_eq!(pass.material.cull_mode, None);
}
