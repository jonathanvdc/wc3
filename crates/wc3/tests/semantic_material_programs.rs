use glam::Vec2;
use wc3::model::ModelMetadata;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
#[path = "support/materials.rs"]
mod native_fixtures;
use native_fixtures::m2::Material as M2Material;
use native_fixtures::wmo;
use wc3::model::conversion::{ExportOptions, ImportOptions};
use wc3::model::formats::mdlx::{self as native, FormatVersion};
use wc3::model::materials::mdlx::{
    Blend, MaterialInputs, MaterialStep as MdlxStep, ReforgedInputs, Shader, UnresolvedInputs,
};
use wc3::model::materials::program::{MaterialProgram, MaterialStep, MaterialSurface};
use wc3::model::profiles::SurfacePayload;
use wc3::model::profiles::{mdlx::Material as MdlxMaterial, Mdlx};
use wc3::model::{io, Model};

#[test]
fn mdlx_profile_conversion_preserves_steps_without_interpreting_them() {
    let mut values = SlotValues::default();
    let mut first = MdlxStep::construct(&mut values);
    first.declaration.priority_plane = -17;
    first.declaration.two_sided = true;
    let mut second = first.clone();
    second.declaration.priority_plane = 12;
    let original = MdlxMaterial {
        steps: vec![first, second],
    };
    let broad = MaterialSurface::from(original.clone());
    broad.validate().unwrap();
    assert_eq!(MdlxMaterial::try_from(broad).unwrap(), original);
    let empty = MaterialSurface::from(MdlxMaterial::default());
    assert!(empty.steps.is_empty());
}

#[test]
fn single_step_projection_preserves_shader_identity_and_native_state() {
    let mut slot_values = SlotValues::default();
    let shader = M2Material {
        shader: 0xf123,
        label: "authored programme".into(),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let restored = M2Material::try_from(MaterialSurface::from(shader)).unwrap();
    assert_eq!(restored.shader, 0xf123);
    assert_eq!(restored.label, "authored programme");

    let mut world = wmo::Material {
        shader: Some(0xffff1234),
        uv_scroll: [Vec2::from_array([0.2, -0.4]), Vec2::from_array([0.5, 0.8])],
        legacy_controls: Some(0xfedcba98),
        ..SlotConstruct::construct(&mut slot_values)
    };
    world.state.rasterization.controls.exterior_lit = true;
    assert_eq!(
        wmo::Material::try_from(MaterialSurface::from(world.clone())).unwrap(),
        world
    );
}

#[test]
fn mixed_steps_are_valid_but_native_projection_cannot_drop_them() {
    let mut slot_values = SlotValues::default();
    let program = MaterialSurface {
        steps: vec![
            MaterialStep::Mdlx(Box::new(SlotConstruct::construct(&mut slot_values))),
            MaterialStep::M2(Box::new(SlotConstruct::construct(&mut slot_values))),
            MaterialStep::Wmo(Box::new(SlotConstruct::construct(&mut slot_values))),
        ],
    };
    program.validate().unwrap();
    let broad = program;
    assert!(MdlxMaterial::try_from(broad.clone()).is_err());
    assert!(M2Material::try_from(broad.clone()).is_err());
    assert!(wmo::Material::try_from(broad).is_err());
}

#[test]
fn native_single_step_projection_rejects_empty_and_repeated_steps() {
    let mut slot_values = SlotValues::default();
    for steps in [
        vec![],
        vec![
            MaterialStep::Wmo(Box::new(SlotConstruct::construct(&mut slot_values))),
            MaterialStep::Wmo(Box::new(SlotConstruct::construct(&mut slot_values))),
        ],
    ] {
        let broad = MaterialSurface { steps };
        assert!(wmo::Material::try_from(broad.clone()).is_err());
        assert!(M2Material::try_from(broad).is_err());
    }
}

#[test]
fn an_empty_program_does_not_require_a_default_step_type() {
    struct NoDefault;
    let program: MaterialProgram<NoDefault> = MaterialProgram::default();
    assert!(program.steps.is_empty());
}

#[test]
fn a_mixed_program_samples_each_shader_without_changing_step_order() {
    let mut slot_values = SlotValues::default();
    use wc3::model::profiles::Universal;
    use wc3::model::{
        AnimationSample, Curve, DynamicTrack, Material, Model, Time, TimeSource, Track,
    };
    let mut draft = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let id = draft.reserve_material();
    let layered = slot_values.add(1f32);
    let shader = slot_values.add(1f32);
    draft
        .set_material(
            id,
            Material {
                name: "mixed".into(),
                surface: MaterialSurface {
                    steps: vec![
                        MaterialStep::Mdlx(Box::new(MdlxStep {
                            shader: Shader::Classic(MaterialInputs {
                                opacity: layered,
                                ..SlotConstruct::construct(&mut slot_values)
                            }),
                            ..SlotConstruct::construct(&mut slot_values)
                        })),
                        MaterialStep::M2(Box::new(
                            M2Material {
                                opacity: shader,
                                ..SlotConstruct::construct(&mut slot_values)
                            }
                            .into(),
                        )),
                        MaterialStep::Wmo(Box::new(SlotConstruct::construct(&mut slot_values))),
                    ],
                },
            },
        )
        .unwrap();
    for (target, values) in [(layered, [0., 1.]), (shader, [0.5, 1.])] {
        draft
            .add_global_track(DynamicTrack::Scalar(Track {
                target,
                clock: TimeSource::Animation,
                curve: Curve::linear(
                    [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()],
                    values,
                )
                .unwrap(),
            }))
            .unwrap();
    }
    let mut model = {
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .unwrap();
    let sample = model
        .animation_sample(AnimationSample {
            clip_time: Time::seconds(0.5).unwrap(),
            ..SlotConstruct::construct(&mut slot_values)
        })
        .unwrap();
    let source = &model.material(id).unwrap().surface;
    let frozen_values = sample.evaluate();
    let frozen = source.clone();
    let MaterialStep::Mdlx(first) = &frozen.steps[0] else {
        panic!("layered")
    };
    let MaterialStep::M2(second) = &frozen.steps[1] else {
        panic!("model shader")
    };
    assert_eq!(frozen_values[first.shader.inputs().opacity], 0.5);
    assert_eq!(frozen_values[second.declaration().opacity], 0.75);
    assert_eq!(first.shader.inputs().opacity, layered);
    assert_eq!(second.declaration().opacity, shader);
    assert!(matches!(frozen.steps[2], MaterialStep::Wmo(_)));
    // Appending a step preserves the existing indexed animation targets.
    assert!(model
        .edit(|draft| {
            let edit_result = {
                let mut material = draft.model().material(id).unwrap().clone();
                let program = &mut material.surface;
                program
                    .steps
                    .push(MaterialStep::M2(Box::new(SlotConstruct::construct(
                        &mut slot_values,
                    ))));
                draft.set_material(id, material)
            };
            draft.import_values(&slot_values)?;
            edit_result
        })
        .is_ok());
}

fn native_model<V: FormatVersion>(
    name: &str,
    selectors: &[Option<u32>],
    slots: bool,
) -> Model<'static, Mdlx> {
    let mut document = native::Document::<V>::new();
    document.set_textures([native::materials::Texture::new("shared.blp").unwrap()]);
    let mut material = native::materials::Material::<V>::new();
    if !name.is_empty() {
        material.try_set_shader(name).unwrap();
    }
    for selector in selectors {
        let mut layer = native::materials::Layer::<V>::new();
        if let Some(selector) = selector {
            layer
                .try_set_shader_type(native::materials::ShaderType::new(*selector))
                .unwrap();
        }
        if slots {
            layer
                .try_set_texture_slots(&[2, 2, 97].map(|slot| {
                    native::materials::LayerTextureSlot {
                        texture_type: slot,
                        texture_id: 0.into(),
                    }
                }))
                .unwrap();
        }
        material.layers.push(layer);
    }
    document.set_materials([material]);
    io::mdlx::import_owned(&document, &ImportOptions::default())
        .unwrap()
        .value
}

fn export<V: FormatVersion>(
    model: &Model<'_, Mdlx>,
) -> Result<native::Document<'static, V>, wc3::model::conversion::Error> {
    // Material test inputs own their text, and the native output is consumed here.
    io::mdlx::plan::<V>(model, &ExportOptions::default())?
        .execute()
        .map(|output| output.value.into_owned())
}

#[test]
fn native_import_interprets_selector_precedence_before_widening() {
    for (name, selector, expected) in [
        ("", None, "classic"),
        ("Shader_SD_FixedFunction", None, "classic"),
        ("Shader_HD_DefaultUnit", None, "reforged"),
        ("", Some(0), "classic"),
        ("", Some(2), "classic"),
        ("", Some(1), "reforged"),
        ("", Some(99), "unresolved"),
        ("Shader_HD_Crystal", None, "unresolved"),
    ] {
        let model = if selector.is_some() {
            native_model::<native::V1800>(name, &[selector], false)
        } else {
            native_model::<native::V900>(name, &[None], false)
        };
        let original = &model.materials().iter().next().unwrap().1.surface;
        let shader = &original.steps[0].shader;
        let actual = match shader {
            Shader::Classic(_) => "classic",
            Shader::ReforgedDefault(_) => "reforged",
            Shader::Unresolved(_) => "unresolved",
        };
        assert_eq!(actual, expected, "{name:?}, {selector:?}");
        let broad = MaterialSurface::from(original.clone());
        assert_eq!(MdlxMaterial::try_from(broad).unwrap(), *original);
        let restored = if selector.is_some() {
            let output = export::<native::V1800>(&model).unwrap();
            io::mdlx::import_owned(&output, &ImportOptions::default())
                .unwrap()
                .value
        } else {
            let output = export::<native::V900>(&model).unwrap();
            io::mdlx::import_owned(&output, &ImportOptions::default())
                .unwrap()
                .value
        };
        let restored = &restored.materials().iter().next().unwrap().1.surface;
        assert_eq!(
            std::mem::discriminant(&restored.steps[0].shader),
            std::mem::discriminant(shader)
        );
        if let Shader::Unresolved(inputs) = &restored.steps[0].shader {
            assert_eq!(inputs.shader_name, name);
        }
    }
}

#[test]
fn native_import_preserves_duplicate_and_unknown_texture_roles() {
    for selector in [1, 99, 0] {
        let model = native_model::<native::V1800>("", &[Some(selector)], true);
        if selector == 1 {
            assert!(export::<native::V1000>(&model).is_err());
        }
        let material = &model.materials().iter().next().unwrap().1.surface;
        assert_eq!(
            matches!(material.steps[0].shader, Shader::ReforgedDefault(_)),
            selector == 1
        );
        assert_eq!(
            material.steps[0]
                .shader
                .textures()
                .iter()
                .map(|slot| slot.slot)
                .collect::<Vec<_>>(),
            [2, 2, 97]
        );
        let output = export::<native::V1800>(&model).unwrap();
        assert_eq!(
            output.materials().next().unwrap().layers[0]
                .try_texture_slots()
                .unwrap()
                .iter()
                .map(|slot| slot.texture_type)
                .collect::<Vec<_>>(),
            [2, 2, 97]
        );
    }
}

#[test]
fn reforged_shader_derives_native_selection_and_preserves_fresnel() {
    let mut builder = Model::<Mdlx>::builder(wc3::model::profiles::mdlx::ModelMetadata::default());
    let values = builder.values_mut();
    let mut hd = ReforgedInputs::construct(values);
    hd.fresnel = Some(SlotConstruct::construct(values));
    let step = MdlxStep {
        shader: Shader::ReforgedDefault(hd),
        ..SlotConstruct::construct(values)
    };
    builder
        .add_material(wc3::model::Material {
            name: "HD".into(),
            surface: MdlxMaterial { steps: vec![step] },
        })
        .unwrap();
    let model = builder.finish().unwrap();
    let output = export::<native::V1800>(&model).unwrap();
    let material = output.materials().next().unwrap();
    assert_eq!(material.layers[0].try_shader_type().unwrap().id(), 1);
    assert!(material.layers[0].try_fresnel().is_ok());
    let output = export::<native::V1000>(&model).unwrap();
    assert_eq!(
        output.materials().next().unwrap().try_shader().unwrap(),
        "Shader_HD_DefaultUnit"
    );
}

#[test]
fn wc3_reforged_and_unresolved_steps_freeze_shader_inputs_without_reclassification() {
    let mut slot_values = SlotValues::default();
    use wc3::model::materials::mdlx::Fresnel;
    use wc3::model::profiles::Universal;
    use wc3::model::{
        AnimationSample, Curve, DynamicTrack, Material, Model, Time, TimeSource, Track,
    };
    for selector in [1, 99] {
        let mut draft = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
        let id = draft.reserve_material();
        let opacity = slot_values.add(1f32);
        let gain = slot_values.add(1f32);
        let fresnel = slot_values.add(1f32);
        draft
            .set_material(
                id,
                Material {
                    name: "animated shader".into(),
                    surface: {
                        let inputs = MaterialInputs {
                            opacity,
                            emissive_gain: Some(gain),
                            ..SlotConstruct::construct(&mut slot_values)
                        };
                        let fresnel = Some(Fresnel {
                            opacity: fresnel,
                            ..SlotConstruct::construct(&mut slot_values)
                        });
                        let shader = if selector == 1 {
                            Shader::ReforgedDefault(ReforgedInputs {
                                inputs,
                                fresnel,
                                textures: vec![],
                            })
                        } else {
                            Shader::Unresolved(UnresolvedInputs {
                                inputs,
                                fresnel,
                                selector: Some(selector),
                                ..SlotConstruct::construct(&mut slot_values)
                            })
                        };
                        MaterialSurface::from(MdlxMaterial {
                            steps: vec![MdlxStep {
                                shader,
                                ..SlotConstruct::construct(&mut slot_values)
                            }],
                        })
                    },
                },
            )
            .unwrap();
        for target in [opacity, gain, fresnel] {
            draft
                .add_global_track(DynamicTrack::Scalar(Track {
                    target,
                    clock: TimeSource::Animation,
                    curve: Curve::linear(
                        [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()],
                        [0., 1.],
                    )
                    .unwrap(),
                }))
                .unwrap();
        }
        let model = {
            draft.import_values(&slot_values).unwrap();
            draft.finish()
        }
        .unwrap();
        let sample = model
            .animation_sample(AnimationSample {
                clip_time: Time::seconds(0.5).unwrap(),
                ..SlotConstruct::construct(&mut slot_values)
            })
            .unwrap();
        let frozen_values = sample.evaluate();
        let surface = model.material(id).unwrap().surface.clone();
        let MaterialStep::Mdlx(step) = &surface.steps[0] else {
            unreachable!()
        };
        assert_eq!(
            matches!(step.shader, Shader::ReforgedDefault(_)),
            selector == 1
        );
        assert_eq!(matches!(step.shader, Shader::Unresolved(_)), selector == 99);
        let layer = step.shader.inputs();
        assert_eq!(frozen_values[layer.opacity], 0.5);
        assert_eq!(layer.opacity, opacity);
        let gain = layer.emissive_gain.unwrap();
        assert_eq!(frozen_values[gain], 0.5);
        assert!(frozen_values.get(gain).is_some());
        let fresnel = step.shader.fresnel().unwrap().opacity;
        assert_eq!(frozen_values[fresnel], 0.5);
        assert!(frozen_values.get(fresnel).is_some());
    }
}

#[test]
fn native_export_checks_shared_declarations_and_shader_names() {
    let mut model = native_model::<native::V1800>("", &[Some(2), Some(1), Some(99)], false);
    let output = export::<native::V1800>(&model).unwrap();
    let material = output.materials().next().unwrap();
    assert!(material.try_shader().unwrap_or_default().is_empty());
    assert_eq!(
        material
            .layers
            .iter()
            .map(|layer| layer.try_shader_type().unwrap().id())
            .collect::<Vec<_>>(),
        [0, 1, 99]
    );
    let id = model.materials().iter().next().unwrap().0;
    model
        .edit(|builder| {
            let mut material = builder.model().material(id).unwrap().clone();
            let mut other = material.surface.steps[2].clone();
            let Shader::Unresolved(inputs) = &mut other.shader else {
                unreachable!()
            };
            inputs.shader_name = "another shader".into();
            material.surface.steps.push(other);
            builder.set_material(id, material)
        })
        .unwrap();
    assert!(export::<native::V1800>(&model)
        .unwrap_err()
        .description
        .contains("different material shader names"));
    model
        .edit(|builder| {
            let mut material = builder.model().material(id).unwrap().clone();
            material.surface.steps.pop();
            material.surface.steps[1].declaration.priority_plane += 1;
            builder.set_material(id, material)
        })
        .unwrap();
    assert!(export::<native::V1800>(&model)
        .unwrap_err()
        .description
        .contains("different material-wide draw declarations"));
}

#[test]
fn additional_texture_slots_roundtrip_their_own_images_and_animation() {
    use native::animation::{Animatable, Sequence, Track, ValueKeyframe};
    use native::materials::{Layer, LayerTextureSlot, Material, ShaderType, Texture};
    for selector in [1, 99] {
        let mut document = native::Document::<native::V1800>::new();
        document.set_textures(
            ["primary.blp", "normal.blp", "animated.blp"].map(|name| Texture::new(name).unwrap()),
        );
        document.set_sequences([Sequence::new("slots", [0, 1000]).unwrap()]);
        let mut material = Material::new();
        let mut layer = Layer::<native::V1800>::new();
        layer
            .try_set_shader_type(ShaderType::new(selector))
            .unwrap();
        let mut animated = Animatable::from(1);
        animated.set_track(
            Track::step(
                [
                    ValueKeyframe { frame: 0, value: 1 },
                    ValueKeyframe {
                        frame: 1000,
                        value: 2,
                    },
                ],
                None,
            )
            .unwrap(),
        );
        layer
            .try_set_texture_slots(&[LayerTextureSlot {
                texture_type: 1,
                texture_id: animated,
            }])
            .unwrap();
        material.layers.push(layer);
        document.set_materials([material]);
        let model = io::mdlx::import_owned(&document, &ImportOptions::default())
            .unwrap()
            .value;
        let output = export::<native::V1800>(&model).unwrap();
        let material = output.materials().next().unwrap();
        let layer = &material.layers[0];
        let textures: Vec<_> = output.textures().collect();
        let primary = *layer.texture_id.value().unwrap();
        let slot = &layer.try_texture_slots().unwrap()[0];
        let track = slot.texture_id.track().unwrap();
        let start = track.evaluate(0.).unwrap();
        let end = track.evaluate(1000.).unwrap();
        assert_ne!(primary, start);
        assert_ne!(start, end);
        assert_eq!(textures[primary as usize].path.text(), "primary.blp");
        assert_eq!(textures[start as usize].path.text(), "normal.blp");
        assert_eq!(textures[end as usize].path.text(), "animated.blp");
    }
}

#[test]
fn modern_classic_layers_normalize_only_neutral_extension_storage() {
    use native::animation::{Animatable, Track, ValueKeyframe};
    use native::materials::{Layer, LayerTextureSlot, Material, ShaderType, Texture};
    for selector in [0, 2] {
        for variant in 0..6 {
            let mut document = native::Document::<native::V1800>::new();
            document
                .set_textures(["base.blp", "selected.blp"].map(|name| Texture::new(name).unwrap()));
            let mut layer = Layer::new();
            layer.set_shader_type(ShaderType::new(selector));
            let mut texture = Animatable::Static(1);
            texture.set_track(
                Track::step(
                    [
                        ValueKeyframe { frame: 0, value: 0 },
                        ValueKeyframe {
                            frame: 1000,
                            value: 1,
                        },
                    ],
                    None,
                )
                .unwrap(),
            );
            layer.set_texture_slots(&[LayerTextureSlot {
                texture_type: 0,
                texture_id: texture.clone(),
            }]);
            match variant {
                1 => {
                    let mut fresnel = layer.fresnel().clone();
                    fresnel.opacity = Animatable::Static(0.5);
                    layer.set_fresnel(fresnel);
                }
                2 => {
                    let mut fresnel = layer.fresnel().clone();
                    fresnel.opacity.set_track(
                        Track::step(
                            [
                                ValueKeyframe {
                                    frame: 0,
                                    value: 0.0,
                                },
                                ValueKeyframe {
                                    frame: 1000,
                                    value: 0.0,
                                },
                            ],
                            None,
                        )
                        .unwrap(),
                    );
                    layer.set_fresnel(fresnel);
                }
                3 => layer.set_texture_slots(&[LayerTextureSlot {
                    texture_type: 1,
                    texture_id: texture.clone(),
                }]),
                4 => layer.texture_id = Animatable::Static(1),
                5 => layer.set_texture_slots(&[
                    LayerTextureSlot {
                        texture_type: 0,
                        texture_id: texture.clone(),
                    },
                    LayerTextureSlot {
                        texture_type: 0,
                        texture_id: texture.clone(),
                    },
                ]),
                _ => {}
            }
            let mut material = Material::new();
            material.layers.push(layer);
            document.set_materials([material]);
            let model = io::mdlx::import_owned(&document, &ImportOptions::default())
                .unwrap()
                .value;
            let shader = &model.materials().iter().next().unwrap().1.surface.steps[0].shader;
            assert_eq!(
                matches!(shader, Shader::Classic(_)),
                variant == 0,
                "selector {selector}, variant {variant}"
            );
            let output = export::<native::V1800>(&model).unwrap();
            let material = output.materials().next().unwrap();
            let exported = &material.layers[0];
            if variant == 0 {
                assert_eq!(exported.texture_id, Animatable::Static(0));
                assert_eq!(exported.texture_slots().len(), 1);
                assert_eq!(exported.texture_slots()[0].texture_type, 0);
                let track = exported.texture_slots()[0].texture_id.track().unwrap();
                let textures = output.textures().collect::<Vec<_>>();
                assert_eq!(
                    textures[track.evaluate(0.).unwrap() as usize].path.text(),
                    "base.blp"
                );
                assert_eq!(
                    textures[track.evaluate(1000.).unwrap() as usize]
                        .path
                        .text(),
                    "selected.blp"
                );
                let restored = io::mdlx::import_owned(&output, &ImportOptions::default())
                    .unwrap()
                    .value;
                assert!(matches!(
                    restored.materials().iter().next().unwrap().1.surface.steps[0].shader,
                    Shader::Classic(_)
                ));
            } else {
                assert!(matches!(shader, Shader::Unresolved(_)));
                assert_eq!(
                    exported.fresnel(),
                    document.materials().next().unwrap().layers[0].fresnel()
                );
                assert_eq!(
                    exported.texture_slots().len(),
                    document.materials().next().unwrap().layers[0]
                        .texture_slots()
                        .len()
                );
            }
        }
    }
}

#[test]
fn equal_native_bitmap_declarations_retain_independent_identities_and_animation() {
    use native::animation::{Animatable, Sequence, Track, ValueKeyframe};
    use native::materials::{Layer, LayerTextureSlot, Material, ShaderType, Texture};
    use wc3::model::DynamicTrack;
    let mut document = native::Document::<native::V1800>::new();
    let mut texture = Texture::new("").unwrap();
    texture.replaceable_id = 31;
    document.set_textures([texture.clone(), texture.clone(), texture]);
    document.set_sequences([Sequence::new("Stand", [0, 1000]).unwrap()]);
    let mut diffuse = Animatable::Static(0);
    diffuse.set_track(
        Track::step(
            [
                ValueKeyframe { frame: 0, value: 0 },
                ValueKeyframe {
                    frame: 1000,
                    value: 1,
                },
            ],
            None,
        )
        .unwrap(),
    );
    let mut layer = Layer::new();
    layer.set_shader_type(ShaderType::HD_DEFAULT_UNIT);
    layer.set_texture_slots(&[
        LayerTextureSlot {
            texture_type: 0,
            texture_id: diffuse,
        },
        LayerTextureSlot {
            texture_type: 1,
            texture_id: Animatable::Static(1),
        },
        LayerTextureSlot {
            texture_type: 2,
            texture_id: Animatable::Static(2),
        },
    ]);
    let mut material = Material::new();
    material.layers.push(layer);
    document.set_materials([material]);
    let model = io::mdlx::import_owned(&document, &ImportOptions::default())
        .unwrap()
        .value;
    let images = model.dependencies().images().iter().collect::<Vec<_>>();
    assert_eq!(images.len(), 3);
    assert_eq!(images[0].1, images[1].1);
    assert_eq!(images[1].1, images[2].1);
    assert_ne!(images[0].0, images[1].0);
    assert_ne!(images[1].0, images[2].0);
    let shader = &model.materials().iter().next().unwrap().1.surface.steps[0].shader;
    for (slot, image) in shader.textures().iter().zip(&images) {
        assert_eq!(
            model.values()[slot.binding.image].unwrap().resource,
            image.0
        );
    }
    let target = shader.textures()[0].binding.image.slot();
    let channel = model
        .tracks()
        .find(|track| track.target() == target)
        .unwrap();
    let DynamicTrack::OptionalTexture(track) = channel else {
        panic!("expected image selections")
    };
    let selected = track.curve.values();
    assert_eq!(selected.get(0).unwrap().unwrap().resource, images[0].0);
    assert_eq!(selected.get(1).unwrap().unwrap().resource, images[1].0);
}

#[test]
fn legacy_default_unit_imports_one_surface_with_animated_texture_roles() {
    use native::mdl::Read as _;
    use wc3::model::{AnimationSample, Time};

    let source = native::Document::<native::V1000>::decode_mdl(
        r#"
Version { FormatVersion 1000, }
Model "Legacy HD roles" { }
Sequences 1 { Anim "Stand" { Interval { 0, 1000 }, } }
Textures 7 {
    Bitmap { Image "diffuse.dds", }
    Bitmap { Image "normal.dds", }
    Bitmap { Image "orm.dds", }
    Bitmap { Image "emissive.dds", }
    Bitmap { Image "team.dds", }
    Bitmap { Image "environment.dds", }
    Bitmap { Image "normal_alt.dds", }
}
Materials 1 {
    Material {
        Shader "Shader_HD_DefaultUnit",
        Layer { FilterMode Transparent, TwoSided, static TextureID 0, static Alpha 0.8, }
        Layer {
            FilterMode None,
            TextureID 2 { DontInterp, 0: 1, 500: 6, }
            Alpha 2 { Linear, 0: 0.0, 1000: 1.0, }
        }
        Layer { FilterMode None, static TextureID 2, }
        Layer { FilterMode None, static TextureID 3, }
        Layer { FilterMode None, static TextureID 4, }
        Layer { FilterMode None, static TextureID 5, }
    }
}
"#,
    )
    .unwrap();
    let model = io::mdlx::import_owned(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let surface = &model.materials().iter().next().unwrap().1.surface;
    assert_eq!(surface.steps.len(), 1);
    let step = &surface.steps[0];
    assert!(step.state.rasterization.faces);
    assert_eq!(step.state.blend, Blend::AlphaKey(()));
    let Shader::ReforgedDefault(inputs) = &step.shader else {
        panic!("expected interpreted HD surface");
    };
    assert_eq!(
        inputs
            .textures
            .iter()
            .map(|slot| slot.slot)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4, 5]
    );
    let textures: Vec<_> = model
        .dependencies()
        .images()
        .iter()
        .map(|(id, _)| id)
        .collect();
    assert_eq!(
        model.values()[inputs.inputs.binding.image].map(|texture| texture.resource),
        Some(textures[0])
    );
    assert_eq!(model.values()[inputs.inputs.opacity], 0.8);
    let mut sample_values = SlotValues::default();
    let sample = model
        .animation_sample(AnimationSample {
            clip: model.clips().iter().next().map(|(id, _)| id),
            clip_time: Time::seconds(0.75).unwrap(),
            ..SlotConstruct::construct(&mut sample_values)
        })
        .unwrap();
    let evaluated = sample.evaluate();
    assert_eq!(
        evaluated[inputs.textures[0].binding.image].map(|texture| texture.resource),
        Some(textures[6])
    );
    for slot in &inputs.textures[1..] {
        assert_eq!(
            evaluated[slot.binding.image].map(|texture| texture.resource),
            Some(textures[slot.slot as usize])
        );
    }
    let output = export::<native::V1000>(&model).unwrap();
    let layers = &output.materials().next().unwrap().layers;
    assert_eq!(layers.len(), 6);
    let output_textures: Vec<_> = output.textures().collect();
    let track = layers[1].texture_id.track().unwrap();
    for (time, expected) in [(0., "normal.dds"), (750., "normal_alt.dds")] {
        let index = track.evaluate(time).unwrap() as usize;
        assert_eq!(output_textures[index].path.text(), expected);
    }
    for version in [1000, 1100] {
        let restored = if version == 1000 {
            let output = export::<native::V1000>(&model).unwrap();
            io::mdlx::import_owned(&output, &ImportOptions::default())
                .unwrap()
                .value
        } else {
            let output = export::<native::V1100>(&model).unwrap();
            assert_eq!(output.materials().next().unwrap().layers.len(), 1);
            io::mdlx::import_owned(&output, &ImportOptions::default())
                .unwrap()
                .value
        };
        let surface = &restored.materials().iter().next().unwrap().1.surface;
        assert_eq!(surface.steps.len(), 1);
        assert_eq!(surface.steps[0].shader.textures().len(), 5);
    }
}

#[test]
fn legacy_layer_roles_do_not_collapse_classic_or_modern_passes() {
    for (version, name) in [(1000, ""), (1100, "Shader_HD_DefaultUnit")] {
        let model = if version == 1000 {
            native_model::<native::V1000>(name, &[None; 6], false)
        } else {
            native_model::<native::V1100>("", &[Some(1); 6], false)
        };
        assert_eq!(
            model
                .materials()
                .iter()
                .next()
                .unwrap()
                .1
                .surface
                .steps
                .len(),
            6
        );
        if version == 1100 {
            assert!(export::<native::V1000>(&model).is_err());
        }
    }
}

#[test]
fn version_900_uses_legacy_roles_without_fresnel_storage() {
    let model = native_model::<native::V900>("Shader_HD_DefaultUnit", &[None; 6], false);
    let surface = &model.materials().iter().next().unwrap().1.surface;
    assert_eq!(surface.steps.len(), 1);
    let Shader::ReforgedDefault(inputs) = &surface.steps[0].shader else {
        panic!("expected interpreted HD surface");
    };
    assert!(inputs.fresnel.is_none());
    assert_eq!(inputs.textures.len(), 5);
    let output = export::<native::V900>(&model).unwrap();
    assert_eq!(output.materials().next().unwrap().layers.len(), 6);
}
