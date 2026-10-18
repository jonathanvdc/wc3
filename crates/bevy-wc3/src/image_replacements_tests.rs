use super::*;
use crate::{compile_model, TextureColorSpace};
use bevy::asset::uuid::Uuid;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::render::render_resource::TextureFormat;
use wc3::model::{ColorSpace, ModelBuilder, ModelMetadata, Sampler};

fn binding() -> (PreparedModel, ImageBinding) {
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let resource = builder.add_image_resource(ImageResource::replaceable(1));
    let model = builder.finish().unwrap();
    let compiled = compile_model(&model, |_| None).unwrap();
    let prepared = compiled.prepare(
        &mut Assets::<Mesh>::default(),
        &mut Assets::<SkinnedMeshInverseBindposes>::default(),
    );
    let binding = ImageBinding {
        key: TextureRequestKey {
            input: None,
            resource,
            declared: ColorSpace::Unspecified,
            color_space: TextureColorSpace::Srgb,
            sampler: Sampler::default(),
        },
        resource: ImageResource::replaceable(1),
        role: TextureRole::Color,
        symbol: Handle::from(Uuid::new_v4()),
        default: None,
    };
    (prepared, binding)
}

#[test]
fn precise_requests_override_resources_and_categories() {
    let (_, binding) = binding();
    let category = Handle::from(Uuid::new_v4());
    let resource = Handle::from(Uuid::new_v4());
    let exact = Handle::from(Uuid::new_v4());
    let mut replacements = ImageReplacements::replaceable(1, category.clone());
    assert_eq!(replacements.source(&binding), Some(&category));
    replacements
        .resources
        .insert(binding.key.resource, resource.clone());
    assert_eq!(replacements.source(&binding), Some(&resource));
    replacements.requests.insert(binding.key, exact.clone());
    assert_eq!(replacements.source(&binding), Some(&exact));
    replacements.requests.clear();
    assert_eq!(replacements.source(&binding), Some(&resource));
    replacements.resources.clear();
    assert_eq!(replacements.source(&binding), Some(&category));
}

#[test]
fn one_source_retains_independent_transfer_and_sampler_variants() {
    let (mut prepared, color) = binding();
    let mut data = color.clone();
    data.key.color_space = TextureColorSpace::Linear;
    data.key.sampler = Sampler {
        u: AddressMode::Clamp,
        v: AddressMode::Mirror,
    };
    data.role = TextureRole::Data;
    data.symbol = Handle::from(Uuid::new_v4());
    prepared.image_template = None;
    prepared.image_bindings = vec![color.clone(), data.clone()].into();
    let mut images = Assets::<Image>::default();
    let mut source = Image::default();
    source.texture_descriptor.format = TextureFormat::Rgba8UnormSrgb;
    let source = images.add(source);
    let replacements = ImageReplacements::replaceable(1, source.clone());
    let mut cache = ReplacementImages::default();
    cache.bind(&prepared, &replacements, &mut images).unwrap();
    assert_eq!(cache.variants.len(), 2);
    for (key, handle) in &cache.variants {
        let variant = images.get(handle).unwrap();
        assert_ne!(handle, &source);
        assert_eq!(
            variant.texture_descriptor.format.is_srgb(),
            key.color_space.is_srgb()
        );
        let ImageSampler::Descriptor(sampler) = &variant.sampler else {
            panic!("expected explicit sampler")
        };
        if key.color_space == TextureColorSpace::Linear {
            assert_eq!(sampler.address_mode_u, ImageAddressMode::ClampToEdge);
            assert_eq!(sampler.address_mode_v, ImageAddressMode::MirrorRepeat);
        } else {
            assert_eq!(sampler.address_mode_u, ImageAddressMode::Repeat);
        }
    }
    assert_eq!(
        images.get(&source).unwrap().texture_descriptor.format,
        TextureFormat::Rgba8UnormSrgb
    );
    cache.bind(&prepared, &replacements, &mut images).unwrap();
    assert_eq!(cache.variants.len(), 2);
}
