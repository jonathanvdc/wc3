use super::*;
use crate::material::{Blend, ModelMaterialKey};
use bevy::prelude::{AlphaMode, Assets};
use naga::front::wgsl::parse_str;
use naga::valid::{Capabilities, ValidationFlags, Validator};

#[test]
fn additional_layers_preserve_material_blending_and_request_third_uv_layout() {
    let mut images = Assets::<Image>::default();
    let primary = images.add(Image::default());
    let secondary = images.add(Image::default());
    let tertiary = images.add(Image::default());
    for (composition, mode) in [
        (ParticleComposition::TwoColor, 1),
        (ParticleComposition::ThreeColor, 2),
    ] {
        let material = particle_material_layers(
            ParticleBlend::BlendAdd,
            [
                Some(primary.clone()),
                Some(secondary.clone()),
                Some(tertiary.clone()),
            ],
            true,
            composition,
        );
        assert_eq!(material.base.base_color_texture, Some(primary.clone()));
        assert_eq!(material.extension.secondary, Some(secondary.clone()));
        assert_eq!(material.extension.tertiary, Some(tertiary.clone()));
        assert_eq!(material.extension.particle.mode.x, mode);
        assert_eq!(material.extension.blend, Some(Blend::Premultiplied));
        assert_eq!(material.base.alpha_mode, AlphaMode::Premultiplied);
        assert!(!material.base.unlit);
        assert!(ModelMaterialKey::from(&material.extension).third_uv);
    }
    let simple = particle_material(ParticleBlend::Alpha, None, false);
    assert!(!ModelMaterialKey::from(&simple.extension).third_uv);
}

#[test]
fn particle_composition_shader_validates_independently() {
    let source = include_str!("../shaders/particle_composition.wgsl")
        .lines()
        .filter(|line| !line.starts_with("#define_import_path"))
        .collect::<Vec<_>>()
        .join("\n")
        .replace("#{MATERIAL_BIND_GROUP}", "2");
    let module =
        parse_str(&source).unwrap_or_else(|error| panic!("{}", error.emit_to_string(&source)));
    Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&module)
        .unwrap_or_else(|error| panic!("{error:?}"));
}
