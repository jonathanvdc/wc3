use super::*;
use crate::universal::material::compile_material_with_images;
use bevy::prelude::{Assets, Image};
use wc3::model::materials::bindings::TextureBinding;
use wc3::model::{Material, ModelBuilder};

#[test]
fn world_surface_clamps_resolve_independent_variants_without_mutating_texture() {
    let mut builder = ModelBuilder::new("world samplers");
    let mut texture = Texture::file("shared.png");
    texture.sampler.u = AddressMode::Mirror;
    let texture = builder.add_texture(texture).unwrap();
    let mut materials = Vec::new();
    for clamp_u in [false, true, true] {
        let mut surface = world::Material {
            shader: Some(0),
            ..Default::default()
        };
        surface.state.rasterization.controls.clamp_u = clamp_u;
        surface.textures.push(TextureBinding {
            image: Some(texture),
            ..Default::default()
        });
        materials.push(
            builder
                .add_material(Material {
                    name: "world".into(),
                    surface: SurfaceSet::World(Box::new(surface)),
                })
                .unwrap(),
        );
    }
    let model = builder.finish().unwrap();
    let requests = texture_requests(&model);
    assert_eq!(requests.len(), 2);
    assert_ne!(requests[0].key(), requests[1].key());
    assert_eq!(
        model.texture(texture).unwrap().sampler.u,
        AddressMode::Mirror
    );
    let mut images = Assets::<Image>::default();
    let mirror = images.add(Image::default());
    let clamp = images.add(Image::default());
    let resolved = Arc::new(TextureImages::resolve(&model, |request| {
        Some(if request.sampler.u == AddressMode::Clamp {
            clamp.clone()
        } else {
            mirror.clone()
        })
    }));
    let handles: Vec<_> = materials
        .into_iter()
        .map(|id| {
            compile_material_with_images(&model, id, resolved.clone())
                .unwrap()
                .remove(0)
                .material
                .base_color_texture
                .unwrap()
        })
        .collect();
    assert_eq!(handles, vec![mirror, clamp.clone(), clamp]);
}

#[test]
fn native_world_clamp_authority_imports_without_texture_sampler_mutation() {
    use wc3::model::formats::wmo::{
        codec::Array, records::Material as NativeMaterial, root::RootChunk, Bundle, Classic, Root,
    };
    use wc3::model::{conversion::ImportOptions, io};
    let mut material = NativeMaterial::<Classic> {
        texture0: 42,
        ..Default::default()
    };
    material.flags.set_clamp_s(true);
    let mut root = Root::<Classic>::new();
    root.header_mut().texture_count = 1;
    root.insert_chunk(
        0,
        RootChunk::Materials(Array::from_records(&[material]).unwrap()),
    )
    .unwrap();
    let bundle = Bundle::from_root(root);
    let model = io::wmo::import(&bundle, &ImportOptions::default())
        .unwrap()
        .value
        .widen();
    assert!(model
        .textures()
        .all(|(_, texture)| texture.sampler == Sampler::default()));
    assert!(texture_requests(&model)
        .iter()
        .any(|request| request.sampler.u == AddressMode::Clamp));
}
