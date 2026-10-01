use super::*;
#[test]
fn model_paths_preserve_locations_and_support_mdl_fallback() {
    let model = Path::new("units/parent.mdx");
    assert_eq!(
        model_paths(model, "Effects\\Spark.MDL"),
        [
            "units/Effects/Spark.MDL",
            "units/Effects/Spark.mdx",
            "Effects/Spark.MDL",
            "Effects/Spark.mdx",
        ]
    );
    assert_eq!(
        model_paths(model, "Spark.mdx"),
        ["units/Spark.mdx", "Spark.mdx"]
    );
    assert_eq!(
        model_paths(Path::new("parent.mdl"), "Spark.mdl"),
        ["Spark.mdl", "Spark.mdx"]
    );
    for path in ["", "../secret.mdl", "/secret.mdx", "Spark.tga"] {
        assert!(model_paths(model, path).is_empty());
    }
}

#[test]
fn texture_paths_try_model_then_asset_root() {
    let model = Path::new("units/footman.mdx");
    assert_eq!(
        texture_paths(model, "body.blp"),
        ["units/body.blp", "body.blp"]
    );
    assert_eq!(
        texture_paths(model, "Textures\\armor.blp"),
        ["units/Textures/armor.blp", "Textures/armor.blp"]
    );
    assert!(texture_paths(model, "../secret.blp").is_empty());
}

#[test]
fn texture_sampler_uses_wrap_flags_per_axis() {
    let mut flags = TextureFlags::default();
    flags.set_wrap_width(true);
    let ImageSampler::Descriptor(sampler) = texture_sampler(flags) else {
        panic!("expected a texture sampler descriptor");
    };
    assert_eq!(sampler.address_mode_u, ImageAddressMode::Repeat);
    assert_eq!(sampler.address_mode_v, ImageAddressMode::ClampToEdge);

    flags.set_wrap_width(false);
    flags.set_wrap_height(true);
    let ImageSampler::Descriptor(sampler) = texture_sampler(flags) else {
        panic!("expected a texture sampler descriptor");
    };
    assert_eq!(sampler.address_mode_u, ImageAddressMode::ClampToEdge);
    assert_eq!(sampler.address_mode_v, ImageAddressMode::Repeat);
}

#[test]
fn replaceable_bitmap_has_no_guessed_path() {
    let mut bitmap = Texture::new("").unwrap();
    bitmap.replaceable_id = 31;
    let mut requested = Vec::new();
    let binding = resolve_texture(&bitmap, &mut |path| {
        requested.push(path.to_owned());
        None
    });
    assert!(requested.is_empty());
    assert_eq!(binding.replaceable_id, 31);
}
