use super::*;
use crate::material::compile_material_with_images;
use crate::material::native_fixtures::wmo;
use bevy::prelude::{Assets, Image};
use wc3::model::materials::bindings::TextureBinding;
use wc3::model::{Material, MaterialSurface, ModelBuilder};

#[test]
fn m3_layers_resolve_axis_wrap_variants_for_color_normals_and_occlusion() {
    let mut builder = ModelBuilder::new("M3 layer samplers");
    let mut source = Texture::new(builder.add_image_resource(wc3::model::ImageResource::Path(
        "shared.dds".as_bytes().into(),
    )));
    source.sampler = Sampler {
        u: AddressMode::Mirror,
        v: AddressMode::Mirror,
    };
    let texture = source;
    let mut materials = Vec::new();
    for (wrap_u, wrap_v) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut layer = m3::TextureLayer::default();
        layer.binding.image = Some(texture.resource);
        layer.binding.coordinates.transform.uv_tiling.initial = [1.; 2];
        layer.binding.coordinates.transform.uv_source_related = u32::MAX;
        layer.binding.contribution.map_alpha.initial = 1.;
        layer.binding.sampling.flags.set_uv_wrap_x(wrap_u);
        layer.binding.sampling.flags.set_uv_wrap_y(wrap_v);
        let mut surface = m3::StandardMaterialStep {
            diffuse: Some(layer.clone()),
            ..Default::default()
        };
        surface.normals.base = Some(layer.clone());
        surface.ambient_occlusion_layer = Some(layer);
        materials.push((
            builder
                .add_material(Material {
                    name: "M3 independent layer wrapping".into(),
                    surface: MaterialSurface::from(m3::MaterialStep::Standard(Box::new(surface))),
                })
                .unwrap(),
            Sampler {
                u: if wrap_u {
                    AddressMode::Repeat
                } else {
                    AddressMode::Clamp
                },
                v: if wrap_v {
                    AddressMode::Repeat
                } else {
                    AddressMode::Clamp
                },
            },
        ));
    }
    let model = builder.finish().unwrap();
    let mut assets = Assets::<Image>::default();
    let resolved = Arc::new(TextureImages::resolve(&model, |_| {
        Some(assets.add(Image::default()))
    }));
    for (material, sampler) in materials {
        let pass = compile_material_with_images(&model, material, resolved.clone())
            .unwrap()
            .remove(0);
        let scoped = resolved.material(material, 0);
        let normal = scoped.slot(8);
        let diffuse = scoped.slot(0);
        let data = normal
            .get_with_sampler(
                texture,
                ColorSpace::Unspecified,
                TextureRole::Normal,
                sampler,
            )
            .unwrap();
        let color = diffuse
            .get_with_sampler(
                texture,
                ColorSpace::Unspecified,
                TextureRole::Color,
                sampler,
            )
            .unwrap();
        assert_eq!(pass.material.base_color_texture.as_ref(), Some(color));
        assert_eq!(pass.material.normal_map_texture.as_ref(), Some(data));
        assert!(resolved
            .get_with_sampler(texture, ColorSpace::Unspecified, TextureRole::Data, sampler)
            .is_some());
        assert!(!pass
            .diagnostics
            .iter()
            .any(|line| line.contains("is unresolved")));
        assert_ne!(color, data);
    }
    assert_eq!(
        texture.sampler,
        Sampler {
            u: AddressMode::Mirror,
            v: AddressMode::Mirror,
        }
    );
}

#[test]
fn world_surface_clamps_resolve_independent_variants_without_mutating_texture() {
    let mut builder = ModelBuilder::new("world samplers");
    let mut texture = Texture::new(builder.add_image_resource(wc3::model::ImageResource::Path(
        "shared.png".as_bytes().into(),
    )));
    texture.sampler.u = AddressMode::Mirror;

    let mut materials = Vec::new();
    for clamp_u in [false, true, true] {
        let mut surface = wmo::Material {
            shader: Some(0),
            ..Default::default()
        };
        surface.textures.push(TextureBinding {
            image: Some({
                let mut image = texture;
                if clamp_u {
                    image.sampler.u = AddressMode::Clamp;
                }
                image
            }),
            ..Default::default()
        });
        materials.push(
            builder
                .add_material(Material {
                    name: "world".into(),
                    surface: MaterialSurface::from(surface),
                })
                .unwrap(),
        );
    }
    let model = builder.finish().unwrap();
    let requests = texture_requests(&model);
    assert_eq!(
        requests
            .iter()
            .map(|request| (request.texture, request.color_space, request.sampler))
            .collect::<HashSet<_>>()
            .len(),
        2
    );
    assert_ne!(requests[0].key(), requests[1].key());
    assert_eq!(texture.sampler.u, AddressMode::Mirror);
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
        .widen::<wc3::model::profiles::Universal>();
    assert!(model
        .materials()
        .iter()
        .all(|(_, material)| material.surface.steps.iter().all(|step| {
            let MaterialStep::Wmo(surface) = step else {
                return true;
            };
            surface
                .inputs()
                .iter()
                .all(|input| input.sampling.u == AddressMode::Clamp)
        })));
    assert!(texture_requests(&model)
        .iter()
        .any(|request| request.sampler.u == AddressMode::Clamp));
}

#[test]
fn image_requests_retain_native_input_metadata_for_replacement_selection() {
    use wc3::model::materials::mdlx::{Layer, LayeredSurface};
    use wc3::model::{Animated, TextureControls};
    let mut builder = ModelBuilder::new("shared decoded image");
    let first = builder.add_file_texture("shared.png").unwrap();
    let mut second = first;
    second.controls = Some(TextureControls::Mdlx(Default::default()));
    let layers = [first, second].map(|image| {
        let mut layer = Layer::default();
        layer.binding.image = Animated::constant(Some(image));
        layer
    });
    builder
        .add_material(Material {
            name: "surface".into(),
            surface: MaterialSurface::try_from(LayeredSurface {
                layers: layers.into(),
                ..Default::default()
            })
            .unwrap(),
        })
        .unwrap();
    let model = builder.finish().unwrap();
    let requests = texture_requests(&model);
    assert_eq!(
        requests
            .iter()
            .map(|request| (request.texture, request.color_space, request.sampler))
            .collect::<HashSet<_>>()
            .len(),
        2
    );
    assert_ne!(requests[0].key(), requests[1].key());
}
