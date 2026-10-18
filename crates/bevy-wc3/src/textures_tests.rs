use super::*;
use crate::material::compile_material_with_images;
use crate::material::native_fixtures::wmo;
use bevy::prelude::{Assets, Image};
use wc3::model::materials::bindings::TextureBinding;
use wc3::model::ModelMetadata;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{Material, MaterialSurface, ModelBuilder};

#[test]
fn world_surface_clamps_resolve_independent_variants_without_mutating_texture() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let mut texture = Texture::new(builder.add_image_resource(wc3::model::ImageResource::Path(
        "shared.png".as_bytes().into(),
    )));
    texture.sampler.u = AddressMode::Mirror;

    let mut materials = Vec::new();
    for clamp_u in [false, true, true] {
        let mut surface = wmo::Material {
            shader: Some(0),
            ..SlotConstruct::construct(&mut slot_values)
        };
        surface.textures.push(TextureBinding {
            image: Some({
                let mut image = texture;
                if clamp_u {
                    image.sampler.u = AddressMode::Clamp;
                }
                image
            }),
            ..SlotConstruct::construct(&mut slot_values)
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
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
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
    let mut slot_values = SlotValues::default();
    use wc3::model::formats::wmo::{
        codec::Array, records::Material as NativeMaterial, root::RootChunk, Bundle, Classic, Root,
    };
    use wc3::model::{conversion::ImportOptions, io};
    let mut material = NativeMaterial::<Classic> {
        texture0: 42,
        ..SlotConstruct::construct(&mut slot_values)
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
    let mut slot_values = SlotValues::default();
    use wc3::model::materials::mdlx::{Layer, LayeredSurface};
    use wc3::model::TextureControls;
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let first = builder.add_file_texture("shared.png").unwrap();
    let mut second = first;
    second.controls = Some(TextureControls::Mdlx(SlotConstruct::construct(
        &mut slot_values,
    )));
    let layers = [first, second].map(|image| {
        let mut layer = Layer::construct(&mut slot_values);
        layer.binding.image = slot_values.add(Some(image));
        layer
    });
    builder
        .add_material(Material {
            name: "surface".into(),
            surface: MaterialSurface::try_from(LayeredSurface {
                layers: layers.into(),
                ..SlotConstruct::construct(&mut slot_values)
            })
            .unwrap(),
        })
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
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
