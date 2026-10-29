use super::*;
use crate::material::compile_material_with_images;
use crate::material::native_fixtures::wmo;
use bevy::prelude::{Assets, Image};
use wc3::model::materials::bindings::TextureBinding;
use wc3::model::{AddressMode, ModelMetadata};
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
    use wc3::model::materials::mdlx::MaterialStep as MdlxStep;
    use wc3::model::profiles::mdlx::Material as MdlxMaterial;
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let first = builder.add_file_texture("shared.png").unwrap();
    let mut second = first;
    second.resource = builder.add_image_resource(ImageResource::Replaceable {
        id: 1,
        filename: b"shared.png".as_slice().into(),
    });
    let layers = [first, second].map(|image| {
        let mut layer = MdlxStep::construct(&mut slot_values);
        layer.shader.inputs_mut().binding.image = slot_values.add(Some(image));
        layer
    });
    builder
        .add_material(Material {
            name: "surface".into(),
            surface: MaterialSurface::from(MdlxMaterial {
                declaration: Default::default(),
                steps: layers.into(),
            }),
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

#[test]
fn equal_native_bitmap_declarations_keep_independent_renderer_requests() {
    use wc3::model::conversion::ImportOptions;
    use wc3::model::formats::mdlx::materials::{
        Layer, LayerTextureSlot, Material as NativeMaterial, ShaderType, Texture as NativeTexture,
    };
    use wc3::model::formats::mdlx::{Document, V1800};
    use wc3::model::io::mdlx::import_owned;
    let mut source = Document::<V1800>::new();
    let mut texture = NativeTexture::new("").unwrap();
    texture.replaceable_id = 31;
    source.set_textures([texture.clone(), texture.clone(), texture]);
    let mut layer = Layer::new();
    layer.set_shader_type(ShaderType::HD_DEFAULT_UNIT);
    layer.set_texture_slots(&[0, 1, 2].map(|slot| LayerTextureSlot {
        texture_type: slot,
        texture_id: slot.into(),
    }));
    let mut material = NativeMaterial::new();
    material.layers.push(layer);
    source.set_materials([material]);
    let model = import_owned(&source, &ImportOptions::default())
        .unwrap()
        .value
        .widen();
    let ids = model.dependencies().images().keys().collect::<Vec<_>>();
    assert_eq!(ids.len(), 3);
    let mut images = Assets::<Image>::default();
    let handles: [_; 3] = std::array::from_fn(|_| images.add(Image::default()));
    let mut observed = Vec::new();
    let resolved = Arc::new(TextureImages::resolve(&model, |request| {
        let index = ids
            .iter()
            .position(|id| *id == request.texture.resource)
            .unwrap();
        observed.push((request.input, request.role, index));
        Some(handles[index].clone())
    }));
    for (slot, role) in [
        (0, TextureRole::Color),
        (1, TextureRole::Normal),
        (2, TextureRole::Data),
    ] {
        assert!(observed.iter().any(|(input, selected_role, index)| matches!(input, Some(TextureInput::Material { slot: selected, .. }) if *selected == slot) && *selected_role == role && *index == slot as usize));
    }
    let id = model.materials().iter().next().unwrap().0;
    let passes = compile_material_with_images(&model, id, resolved).unwrap();
    assert_eq!(
        passes[0].material.base_color_texture.as_ref(),
        Some(&handles[0])
    );
    assert_eq!(
        passes[0].material.normal_map_texture.as_ref(),
        Some(&handles[1])
    );
    assert_eq!(
        passes[0].material.metallic_roughness_texture.as_ref(),
        Some(&handles[2])
    );
}

fn scoped_texture_model(material_count: usize) -> (Model<'static>, Vec<MaterialId>, [Texture; 4]) {
    use wc3::model::materials::mdlx::MaterialStep as MdlxStep;
    use wc3::model::profiles::mdlx::Material as MdlxMaterial;

    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let first = builder.add_file_texture("shared.png").unwrap();
    let mut linear = first;
    linear.color_space = ColorSpace::Linear;
    linear.sampler.u = AddressMode::Clamp;
    let mut srgb = first;
    srgb.color_space = ColorSpace::Srgb;
    srgb.sampler.u = AddressMode::Mirror;
    let second = builder.add_file_texture("second.png").unwrap();
    let textures = [first, linear, srgb, second];
    let mut materials = Vec::new();
    for index in 0..material_count {
        let steps = textures
            .iter()
            .map(|&texture| {
                let mut step: MdlxStep = builder.construct();
                step.shader.inputs_mut().binding.image = builder.add_value(Some(texture));
                step
            })
            .collect();
        materials.push(
            builder
                .add_material(Material {
                    name: format!("surface {index}").into(),
                    surface: MaterialSurface::from(MdlxMaterial {
                        declaration: Default::default(),
                        steps,
                    }),
                })
                .unwrap(),
        );
    }
    (builder.finish().unwrap(), materials, textures)
}

#[test]
fn scoped_lookup_and_color_maps_match_flat_request_interpretations() {
    let (model, materials, textures) = scoped_texture_model(3);
    let mut images = Assets::<Image>::default();
    let mut flat = HashMap::new();
    let mut observed = Vec::new();
    let resolved = TextureImages::resolve(&model, |request| {
        let image = images.add(Image::default());
        observed.push(request.key());
        flat.insert(request.key(), image.clone());
        Some(image)
    });
    assert_eq!(
        observed,
        texture_requests(&model)
            .iter()
            .map(TextureRequest::key)
            .collect::<Vec<_>>()
    );

    for &material in &materials {
        for step in 0..4 {
            let scoped = resolved.material(material, step);
            let input = Some(TextureInput::Material {
                material,
                step,
                slot: 0,
            });
            let expected = flat
                .iter()
                .filter(|(key, _)| key.input == input)
                .map(|(key, image)| {
                    (
                        Texture {
                            resource: key.resource,
                            color_space: key.declared,
                            sampler: key.sampler,
                        },
                        image.clone(),
                    )
                })
                .collect::<HashMap<_, _>>();
            assert_eq!(*scoped.color_images(), expected);
            for texture in textures {
                for role in [TextureRole::Color, TextureRole::Normal, TextureRole::Data] {
                    let key = TextureRequestKey {
                        input,
                        resource: texture.resource,
                        declared: texture.color_space,
                        color_space: TextureColorSpace::selected(texture.color_space, role),
                        sampler: texture.sampler,
                    };
                    assert_eq!(scoped.for_role(texture, role), flat.get(&key));
                    assert!(scoped.for_slot(1, texture, role).is_none());
                }
            }
        }
    }
    let expected = flat
        .iter()
        .filter(|(key, _)| key.input.is_none())
        .map(|(key, image)| {
            (
                Texture {
                    resource: key.resource,
                    color_space: key.declared,
                    sampler: key.sampler,
                },
                image.clone(),
            )
        })
        .collect::<HashMap<_, _>>();
    assert_eq!(*resolved.color_images(), expected);
}

#[test]
fn image_remapping_updates_scoped_animation_maps_without_changing_clones() {
    let (model, materials, textures) = scoped_texture_model(2);
    let mut images = Assets::<Image>::default();
    let old = images.add(Image::default());
    let replacement = images.add(Image::default());
    let original = TextureImages::resolve(&model, |_| Some(old.clone()));
    let mut mapped = original.material(materials[0], 0);
    // Construct the animation map before mapping to catch stale derived storage.
    let previous_colors = mapped.color_images();
    let mut calls = 0;
    mapped.map_images(&mut TextureImageRemapper::default(), &mut |image| {
        assert_eq!(image, &old);
        calls += 1;
        Some(replacement.clone())
    });
    assert_eq!(calls, texture_requests(&model).len());
    assert_eq!(previous_colors.get(&textures[0]), Some(&old));
    for &material in &materials {
        for (step, texture) in textures.into_iter().enumerate() {
            let remapped = mapped.material(material, step as u32);
            assert_eq!(
                remapped.for_role(texture, TextureRole::Color),
                Some(&replacement)
            );
            assert_eq!(remapped.color_images().get(&texture), Some(&replacement));
            assert_eq!(
                original
                    .material(material, step as u32)
                    .for_role(texture, TextureRole::Color),
                Some(&old)
            );
        }
    }
    mapped.map_images(&mut TextureImageRemapper::default(), &mut |_| None);
    assert!(mapped.color_images().is_empty());
    assert!(mapped.for_role(textures[0], TextureRole::Color).is_none());
    assert_eq!(
        original
            .material(materials[0], 0)
            .for_role(textures[0], TextureRole::Color),
        Some(&old)
    );
}

#[test]
fn selected_images_filter_resources_independently_of_sampler_and_declaration() {
    let (model, materials, textures) = scoped_texture_model(2);
    let mut selected = textures[0];
    selected.sampler.v = AddressMode::Clamp;
    selected.color_space = ColorSpace::Srgb;
    let selected = HashSet::from([selected]);
    let mut images = Assets::<Image>::default();
    let image = images.add(Image::default());
    let mut observed = Vec::new();
    let resolved = TextureImages::resolve_selected(&model, Some(&selected), |request| {
        observed.push(request.key());
        Some(image.clone())
    });
    let expected = texture_requests(&model)
        .into_iter()
        .filter(|request| request.texture.resource == textures[0].resource)
        .map(|request| request.key())
        .collect::<Vec<_>>();
    assert_eq!(observed, expected);
    for &material in &materials {
        for (step, texture) in textures.into_iter().enumerate() {
            assert_eq!(
                resolved
                    .material(material, step as u32)
                    .for_role(texture, TextureRole::Color),
                (step < 3).then_some(&image)
            );
        }
    }
    let empty = TextureImages::resolve_selected(&model, Some(&HashSet::new()), |_| {
        panic!("empty selection must not resolve requests")
    });
    assert!(empty.color_images().is_empty());
}

#[test]
fn animated_declarations_and_data_roles_keep_distinct_interpretations_in_one_scope() {
    use wc3::model::materials::mdlx::{
        MaterialStep as MdlxStep, ReforgedInputs, Shader, TextureSlot,
    };
    use wc3::model::profiles::mdlx::Material as MdlxMaterial;
    use wc3::model::{Curve, DynamicTrack, Time, TimeSource, Track};

    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let unspecified = builder.add_file_texture("shared.png").unwrap();
    let srgb = Texture {
        color_space: ColorSpace::Srgb,
        ..unspecified
    };
    let linear = Texture {
        color_space: ColorSpace::Linear,
        ..unspecified
    };
    let mut step: MdlxStep = builder.construct();
    let mut hd: ReforgedInputs = builder.construct();
    hd.inputs.binding.image = builder.add_value(Some(unspecified));
    let target = hd.inputs.binding.image;
    hd.textures = [1, 2]
        .map(|slot| TextureSlot {
            slot,
            binding: TextureBinding {
                image: builder.add_value(Some(unspecified)),
                ..builder.construct()
            },
        })
        .into();
    step.shader = Shader::ReforgedDefault(hd);
    let material = builder
        .add_material(Material {
            name: "animated HD".into(),
            surface: MaterialSurface::from(MdlxMaterial {
                declaration: Default::default(),
                steps: vec![step],
            }),
        })
        .unwrap();
    builder
        .add_global_track(DynamicTrack::OptionalTexture(Track {
            target,
            clock: TimeSource::Animation,
            curve: Curve::step(
                [
                    Time::default(),
                    Time::seconds(1.).unwrap(),
                    Time::seconds(2.).unwrap(),
                ],
                [Some(unspecified), Some(srgb), Some(linear)],
            )
            .unwrap(),
        }))
        .unwrap();
    let model = builder.finish().unwrap();
    let mut images = Assets::<Image>::default();
    let mut flat = HashMap::new();
    let resolved = TextureImages::resolve(&model, |request| {
        let image = images.add(Image::default());
        flat.insert(request.key(), image.clone());
        Some(image)
    });
    let scoped = resolved.material(material, 0);
    let colors = scoped.color_images();
    assert_eq!(colors.len(), 3);
    for texture in [unspecified, srgb, linear] {
        assert_eq!(
            colors.get(&texture),
            scoped.for_role(texture, TextureRole::Color)
        );
    }
    assert_ne!(
        resolved.for_role(unspecified, TextureRole::Color),
        resolved.for_role(unspecified, TextureRole::Data)
    );
    assert!(resolved.slot(1).color_images().contains_key(&srgb));
    assert!(scoped.slot(1).color_images().is_empty());
    for request in texture_requests(&model) {
        let scope = match request.input {
            None => resolved.clone(),
            Some(TextureInput::Material {
                material,
                step,
                slot,
            }) => resolved.material(material, step).slot(slot),
            _ => unreachable!(),
        };
        assert_eq!(
            scope.get_with_sampler(
                request.texture,
                request.texture.color_space,
                request.role,
                request.sampler
            ),
            flat.get(&request.key())
        );
    }
}

#[test]
fn repeated_animated_textures_resolve_once_per_input_in_source_order() {
    use wc3::model::materials::mdlx::MaterialStep as MdlxStep;
    use wc3::model::profiles::mdlx::Material as MdlxMaterial;
    use wc3::model::{Curve, DynamicTrack, Time, TimeSource, Track};

    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let texture = builder.add_file_texture("repeated.png").unwrap();
    let mut step: MdlxStep = builder.construct();
    step.shader.inputs_mut().binding.image = builder.add_value(Some(texture));
    let target = step.shader.inputs().binding.image;
    let mut materials = Vec::new();
    for _ in 0..2 {
        materials.push(
            builder
                .add_material(Material {
                    name: "repeated".into(),
                    surface: MaterialSurface::from(MdlxMaterial {
                        declaration: Default::default(),
                        steps: vec![step.clone(), step.clone()],
                    }),
                })
                .unwrap(),
        );
    }
    builder
        .add_global_track(DynamicTrack::OptionalTexture(Track {
            target,
            clock: TimeSource::Animation,
            curve: Curve::step(
                [
                    Time::default(),
                    Time::seconds(1.).unwrap(),
                    Time::seconds(2.).unwrap(),
                ],
                [Some(texture); 3],
            )
            .unwrap(),
        }))
        .unwrap();
    let model = builder.finish().unwrap();
    let mut images = Assets::<Image>::default();
    let mut inputs = Vec::new();
    let resolved =
        TextureImages::resolve_selected(&model, Some(&HashSet::from([texture])), |request| {
            assert_eq!(request.texture, texture);
            inputs.push(request.input);
            Some(images.add(Image::default()))
        });
    let mut expected = vec![None];
    for &material in &materials {
        for step in 0..2 {
            expected.push(Some(TextureInput::Material {
                material,
                step,
                slot: 0,
            }));
        }
    }
    assert_eq!(
        inputs, expected,
        "repeated animation keys must not duplicate resolver work or reorder inputs"
    );
    let handles = materials
        .iter()
        .flat_map(|&material| (0..2).map(move |step| (material, step)))
        .map(|(material, step)| {
            resolved
                .material(material, step)
                .for_role(texture, TextureRole::Color)
                .unwrap()
                .id()
        })
        .collect::<HashSet<_>>();
    assert_eq!(
        handles.len(),
        4,
        "the same resource in distinct inputs may resolve independently"
    );
    assert_eq!(
        images.len(),
        5,
        "one unscoped resolution and four scoped resolutions"
    );
}

#[test]
fn remapping_shared_views_visits_each_table_once_and_keeps_sources_alive() {
    let (model, materials, _textures) = scoped_texture_model(2);
    let mut assets = Assets::<Image>::default();
    let mut flat = HashMap::new();
    let original = TextureImages::resolve(&model, |request| {
        let image = assets.add(Image::default());
        flat.insert(request.key(), image.clone());
        Some(image)
    });
    let weak_source = Arc::downgrade(&original.variants);
    let mut views = materials
        .iter()
        .flat_map(|&material| (0..2).map(move |step| (material, step)))
        .map(|(material, step)| original.material(material, step))
        .collect::<Vec<_>>();
    // Independent resolutions with equal handles still own independent tables.
    let mut independent =
        TextureImages::resolve(&model, |request| flat.get(&request.key()).cloned());
    let mut replacements = HashMap::new();
    for (index, image) in flat.values().enumerate() {
        let replacement = (index % 2 == 0).then(|| assets.add(Image::default()));
        replacements.insert(image.id(), replacement);
    }
    let mut remapper = TextureImageRemapper::default();
    let mut calls = 0;
    let mut substitute = |image: &Handle<Image>| {
        calls += 1;
        replacements[&image.id()].clone()
    };
    for view in &mut views {
        view.map_images(&mut remapper, &mut substitute);
    }
    assert_eq!(calls, flat.len());
    assert!(views.iter().all(|view| view.shares_storage(&views[0])));
    for request in texture_requests(&model) {
        let expected = replacements[&flat[&request.key()].id()].as_ref();
        let view = TextureImages {
            input: request.input,
            variants: views[0].variants.clone(),
        };
        assert_eq!(
            view.get_with_sampler(
                request.texture,
                request.texture.color_space,
                request.role,
                request.sampler
            ),
            expected
        );
    }
    let mut independent_calls = 0;
    independent.map_images(&mut remapper, &mut |image| {
        independent_calls += 1;
        replacements[&image.id()].clone()
    });
    assert_eq!(independent_calls, flat.len());
    assert!(!independent.shares_storage(&views[0]));
    drop(original);
    assert!(
        weak_source.upgrade().is_some(),
        "cache retains source ownership for its pointer keys"
    );
    drop(remapper);
    assert!(weak_source.upgrade().is_none());
    let old_mapped = views[0].clone();
    views[0].map_images(&mut TextureImageRemapper::default(), &mut |_| None);
    assert!(views[0].variants.values().all(HashMap::is_empty));
    assert!(old_mapped
        .variants
        .values()
        .any(|variants| !variants.is_empty()));
}
