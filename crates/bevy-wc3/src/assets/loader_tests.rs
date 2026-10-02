use super::*;
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

#[test]
fn bitmap_keeps_replaceable_id_and_only_resolves_explicit_path() {
    let mut bitmap = Texture::new("").unwrap();
    bitmap.replaceable_id = 11;
    let mut requested = Vec::new();
    let binding = resolve_texture(&bitmap, &mut |path| {
        requested.push(path.to_owned());
        None
    });
    assert!(requested.is_empty());
    assert_eq!(binding.replaceable_id, 11);

    bitmap.path.set_text("Custom/Cliff.blp").unwrap();
    resolve_texture(&bitmap, &mut |path| {
        requested.push(path.to_owned());
        None
    });
    assert_eq!(requested, ["Custom/Cliff.blp"]);
}
