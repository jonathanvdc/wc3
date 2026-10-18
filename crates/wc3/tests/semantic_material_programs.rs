use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
#[path = "support/materials.rs"]
mod native_fixtures;
use native_fixtures::m2::Material as M2Material;
use native_fixtures::wmo;
use wc3::model::materials::m3;
use wc3::model::materials::mdlx::{Layer, LayeredSurface};
use wc3::model::materials::program::{MaterialProgram, MaterialStep, MaterialSurface};
use wc3::model::profiles::{m2, mdlx, SurfacePayload};
use wc3::model::PropertyPayload;

#[test]
fn layered_projection_repeats_native_declarations_on_each_step() {
    let mut slot_values = SlotValues::default();
    let original = LayeredSurface {
        priority_plane: -17,
        shader: "Shader_HD_DefaultUnit".into(),
        constant_color: true,
        two_sided: true,
        sort_near_z: true,
        sort_far_z: true,
        full_resolution: true,
        layers: vec![
            Layer::construct(&mut slot_values),
            Layer::construct(&mut slot_values),
        ],
        ..SlotConstruct::construct(&mut slot_values)
    };
    let broad = MaterialSurface::try_from(original.clone()).unwrap();
    for step in &broad.steps {
        let MaterialStep::Mdlx(step) = step else {
            panic!("layered")
        };
        assert_eq!(step.declaration().priority_plane, -17);
        assert!(step.declaration().two_sided);
    }
    let canonical = LayeredSurface::try_from(broad.clone()).unwrap();
    assert!(canonical.shader.is_empty());
    assert!(canonical.layers.iter().all(|layer| layer.shader == Some(1)));
    assert_wc3_surface_eq(&MaterialSurface::try_from(canonical).unwrap(), &broad);
    let mut changed = broad;
    let MaterialStep::Mdlx(step) = &mut changed.steps[1] else {
        unreachable!()
    };
    step.declaration_mut().priority_plane = 12;
    changed.validate().unwrap();
    assert!(LayeredSurface::try_from(changed).is_err());
    assert!(MaterialSurface::try_from(LayeredSurface {
        priority_plane: 1,
        ..SlotConstruct::construct(&mut slot_values)
    })
    .is_err());
    assert!(MaterialSurface::try_from(LayeredSurface::default())
        .unwrap()
        .steps
        .is_empty());
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
        uv_scroll: [[0.2, -0.4], [0.5, 0.8]],
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
    assert!(LayeredSurface::try_from(broad.clone()).is_err());
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
fn property_selection_uses_program_ordinals_for_mdlx_and_m2() {
    let mut slot_values = SlotValues::default();
    let mut program = MaterialSurface {
        steps: vec![
            MaterialStep::Wmo(Box::new(SlotConstruct::construct(&mut slot_values))),
            MaterialStep::Mdlx(Box::new(SlotConstruct::construct(&mut slot_values))),
            MaterialStep::M2(Box::new(SlotConstruct::construct(&mut slot_values))),
        ],
    };
    let opacity = |index| {
        mdlx::properties::Target::Layer {
            index,
            field: mdlx::properties::LayerField::Opacity,
        }
        .identity()
        .to_owned()
    };
    assert!(!program.supports_property(&opacity(0)));
    assert!(program.supports_property(&opacity(1)));
    assert!(!program.supports_property(&opacity(2)));
    let model_opacity = m2::properties::Target::Opacity { step: 2 }
        .identity()
        .to_owned();
    assert!(program.supports_property(&model_opacity));
    program
        .steps
        .push(MaterialStep::M2(Box::new(SlotConstruct::construct(
            &mut slot_values,
        ))));
    assert!(program.supports_property(&model_opacity));
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
    let mut draft = Model::<Universal>::builder("mixed step animation");
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
                        MaterialStep::Mdlx(Box::new(
                            Layer {
                                opacity: layered,
                                ..SlotConstruct::construct(&mut slot_values)
                            }
                            .into(),
                        )),
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
    assert_eq!(frozen_values[first.to_layer().opacity], 0.5);
    assert_eq!(frozen_values[second.declaration().opacity], 0.75);
    assert_eq!(first.to_layer().opacity, layered);
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

#[test]
fn every_m3_shader_has_a_typed_step_and_exact_native_projection() {
    let mut slot_values = SlotValues::default();
    use wc3::model::materials::m3::MaterialStep as M3MaterialStep;
    let native = [
        M3MaterialStep::Standard(Box::new(SlotConstruct::construct(&mut slot_values))),
        M3MaterialStep::Displacement(Box::new(SlotConstruct::construct(&mut slot_values))),
        M3MaterialStep::Composite(Box::new(SlotConstruct::construct(&mut slot_values))),
        M3MaterialStep::Terrain(Box::new(SlotConstruct::construct(&mut slot_values))),
        M3MaterialStep::Volume(Box::new(SlotConstruct::construct(&mut slot_values))),
        M3MaterialStep::VolumeNoise(Box::new(SlotConstruct::construct(&mut slot_values))),
        M3MaterialStep::Hair(Box::new(SlotConstruct::construct(&mut slot_values))),
        M3MaterialStep::Creep(Box::new(SlotConstruct::construct(&mut slot_values))),
        M3MaterialStep::SplatTerrainBake(Box::new(SlotConstruct::construct(&mut slot_values))),
        M3MaterialStep::Reflection(Box::new(SlotConstruct::construct(&mut slot_values))),
        M3MaterialStep::LensFlare(Box::new(SlotConstruct::construct(&mut slot_values))),
        M3MaterialStep::DataDriven(Box::new(SlotConstruct::construct(&mut slot_values))),
    ];
    for original in native {
        let program = MaterialSurface::from(original.clone());
        assert_eq!(program.steps.len(), 1);
        assert_eq!(M3MaterialStep::try_from(program.clone()).unwrap(), original);
        let mut extended = program;
        extended
            .steps
            .push(MaterialStep::Wmo(Box::new(SlotConstruct::construct(
                &mut slot_values,
            ))));
        assert!(M3MaterialStep::try_from(extended).is_err());
    }
}

#[test]
fn diablo3_keeps_shader_inputs_and_phases() {
    let mut slot_values = SlotValues::default();
    use wc3::model::materials::diablo3::{MaterialStep as Material, RenderState};
    use wc3::model::materials::parameters::{OpaqueWord, ParameterEntry, ShaderTag};
    let mut original = Material::default();
    original.uber.shader_map.id = 991;
    original.uber.colors.diffuse = [0.2, 0.4, 0.6, 0.8];
    original.shader_params = vec![
        ParameterEntry {
            key: ShaderTag(7),
            value: OpaqueWord(0xffffffff),
            controls: SlotConstruct::construct(&mut slot_values),
        },
        ParameterEntry {
            key: ShaderTag(7),
            value: OpaqueWord(0x7fc00000),
            controls: SlotConstruct::construct(&mut slot_values),
        },
    ];
    original.opaque_passes = vec![RenderState {
        effect_file: "opaque".into(),
        ..SlotConstruct::construct(&mut slot_values)
    }];
    original.translucent_passes = vec![RenderState {
        effect_file: "translucent".into(),
        ..SlotConstruct::construct(&mut slot_values)
    }];
    let program = MaterialSurface::from(original.clone());
    let MaterialStep::Diablo3(shader) = &program.steps[0] else {
        panic!("D3 programme")
    };
    assert_eq!(shader.opaque_passes[0].effect_file, "opaque");
    assert_eq!(shader.translucent_passes[0].effect_file, "translucent");
    assert_eq!(shader.shader_params, original.shader_params);
    let restored = Material::try_from(program.clone()).unwrap();
    assert_eq!(restored.uber.shader_map.id, 991);
    assert_eq!(restored.uber.colors.diffuse, original.uber.colors.diffuse);
    assert_eq!(restored.shader_params, original.shader_params);
    let mut extended = program;
    extended
        .steps
        .push(MaterialStep::Wmo(Box::new(SlotConstruct::construct(
            &mut slot_values,
        ))));
    assert!(Material::try_from(extended).is_err());
}

#[test]
fn unresolved_and_resolved_diablo4_programmes_project_exactly() {
    let mut slot_values = SlotValues::default();
    use wc3::model::profiles::diablo::d4::{Material, MaterialStep as D4MaterialStep};
    for steps in [
        vec![],
        vec![D4MaterialStep {
            shader_map: 197,
            value_set_override: -7,
            ..SlotConstruct::construct(&mut slot_values)
        }],
    ] {
        let native = Material { steps };
        let program = MaterialSurface::from(native.clone());
        let restored = Material::try_from(program.clone()).unwrap();
        assert_eq!(restored.steps.len(), native.steps.len());
        if let Some(shader) = restored.steps.first() {
            assert_eq!(shader.shader_map, 197);
            assert_eq!(shader.value_set_override, -7);
        }
        let mut extended = program;
        extended
            .steps
            .push(MaterialStep::Wmo(Box::new(SlotConstruct::construct(
                &mut slot_values,
            ))));
        assert!(Material::try_from(extended).is_err());
    }
}

#[test]
fn unindexed_programmed_animation_fields_require_one_matching_step() {
    let mut slot_values = SlotValues::default();
    use wc3::model::profiles::m3::properties::{Field, FieldPath, Target};
    let key = Target::Field(FieldPath::root(Field::Material).child(Field::ParallaxHeight))
        .identity()
        .to_owned();
    let mut program = MaterialSurface {
        steps: vec![
            MaterialStep::M3(m3::MaterialStep::Standard(Box::new(
                SlotConstruct::construct(&mut slot_values),
            ))),
            MaterialStep::M3(m3::MaterialStep::Composite(Box::new(
                SlotConstruct::construct(&mut slot_values),
            ))),
            MaterialStep::Wmo(Box::new(SlotConstruct::construct(&mut slot_values))),
        ],
    };
    assert!(program.supports_property(&key));
    program
        .steps
        .push(MaterialStep::M3(m3::MaterialStep::Volume(Box::new(
            SlotConstruct::construct(&mut slot_values),
        ))));
    assert!(program.supports_property(&key));
    program
        .steps
        .push(MaterialStep::M3(m3::MaterialStep::Standard(Box::new(
            SlotConstruct::construct(&mut slot_values),
        ))));
    assert!(!program.supports_property(&key));
}

#[test]
fn programmed_selectors_check_optional_inputs_leaves_and_collection_bounds() {
    let mut slot_values = SlotValues::default();
    use wc3::model::materials::m3::{PropertyBinding, StandardMaterialStep, TextureLayer};
    use wc3::model::profiles::m3::properties::{Field, FieldPath, Target};
    let root = FieldPath::root(Field::Material);
    let diffuse = root.child(Field::DiffuseLayer);
    let color = Target::Field(diffuse.child(Field::Color))
        .identity()
        .to_owned();
    let mut shader = StandardMaterialStep::construct(&mut slot_values);
    let step = MaterialStep::M3(m3::MaterialStep::Standard(Box::new(shader.clone())));
    assert!(!step.supports_property(&color, 0));
    shader.diffuse = Some(TextureLayer::construct(&mut slot_values));
    shader.normals.weights = vec![PropertyBinding::construct(&mut slot_values); 2];
    let step = MaterialStep::M3(m3::MaterialStep::Standard(Box::new(shader)));
    assert!(step.supports_property(&color, 0));
    assert!(!step.supports_property(&Target::Field(diffuse).identity().to_owned(), 0));
    for index in 0..=2 {
        let key = Target::Field(root.child(Field::NormalBlendFactors).indexed(index))
            .identity()
            .to_owned();
        assert_eq!(step.supports_property(&key, 0), index < 2);
    }
    let composite = MaterialStep::M3(m3::MaterialStep::Composite(Box::new(
        SlotConstruct::construct(&mut slot_values),
    )));
    assert!(!composite.supports_property(&color, 0));
}

#[test]
fn wc3_shader_steps_interpret_selector_precedence_and_preserve_native_inputs() {
    let mut slot_values = SlotValues::default();
    use wc3::model::materials::bindings::TextureBinding;
    use wc3::model::materials::mdlx::{MaterialStep as Wc3MaterialStep, TextureSlot};
    for (name, selector, expected) in [
        ("", None, "classic"),
        ("Shader_SD_FixedFunction", None, "classic"),
        ("Shader_HD_DefaultUnit", None, "reforged"),
        ("Shader_HD_DefaultUnit", Some(0), "classic"),
        ("Shader_HD_DefaultUnit", Some(2), "classic"),
        ("Shader_SD_FixedFunction", Some(1), "reforged"),
        ("Shader_HD_DefaultUnit", Some(99), "unresolved"),
        ("Shader_HD_Crystal", None, "unresolved"),
    ] {
        let original = LayeredSurface {
            shader: name.into(),
            layers: vec![Layer {
                shader: selector,
                ..SlotConstruct::construct(&mut slot_values)
            }],
            ..SlotConstruct::construct(&mut slot_values)
        };
        let surface = MaterialSurface::try_from(original.clone()).unwrap();
        let MaterialStep::Mdlx(step) = &surface.steps[0] else {
            unreachable!()
        };
        let actual = match step.as_ref() {
            Wc3MaterialStep::Classic(_) => "classic",
            Wc3MaterialStep::ReforgedDefault(_) => "reforged",
            Wc3MaterialStep::Unresolved(_) => "unresolved",
        };
        assert_eq!(actual, expected, "{name:?}, {selector:?}");
        surface.validate().unwrap();
        let canonical = LayeredSurface::try_from(surface.clone()).unwrap();
        assert_wc3_surface_eq(
            &MaterialSurface::try_from(canonical.clone()).unwrap(),
            &surface,
        );
        if expected == "unresolved" {
            assert_eq!(canonical, original);
        } else {
            assert!(canonical.shader.is_empty());
        }
    }
    // Slot ordinals are animation destinations; duplicates and unknown roles survive.
    for selector in [Some(1), Some(99), None] {
        let original = LayeredSurface {
            layers: vec![Layer {
                shader: selector,
                textures: [2, 2, 97]
                    .map(|slot| TextureSlot {
                        slot,
                        binding: TextureBinding::construct(&mut slot_values),
                    })
                    .into(),
                ..SlotConstruct::construct(&mut slot_values)
            }],
            ..SlotConstruct::construct(&mut slot_values)
        };
        let surface = MaterialSurface::try_from(original.clone()).unwrap();
        let MaterialStep::Mdlx(step) = &surface.steps[0] else {
            unreachable!()
        };
        assert_eq!(
            matches!(step.as_ref(), Wc3MaterialStep::ReforgedDefault(_)),
            selector == Some(1)
        );
        for ordinal in 0..4 {
            let key = mdlx::properties::Target::LayerTexture {
                layer: 0,
                slot: ordinal,
            }
            .identity()
            .to_owned();
            assert_eq!(step.supports_property(&key, 0), ordinal < 3);
            assert!(!step.supports_property(&key, 1));
        }
        assert_eq!(LayeredSurface::try_from(surface).unwrap(), original);
    }
}

#[test]
fn wc3_shader_variants_derive_native_selection_and_expose_shader_specific_properties() {
    let mut slot_values = SlotValues::default();
    use wc3::model::materials::mdlx::{
        MaterialStep as Wc3MaterialStep, ReforgedDefaultMaterialStep,
    };
    use wc3::model::profiles::mdlx::properties::{LayerField, Target};
    let mut step =
        Wc3MaterialStep::ReforgedDefault(ReforgedDefaultMaterialStep::construct(&mut slot_values));
    assert_eq!(step.to_layer().shader, Some(1));
    let fresnel = Target::Layer {
        index: 0,
        field: LayerField::FresnelOpacity,
    }
    .identity()
    .to_owned();
    assert!(!step.supports_property(&fresnel, 0));
    let Wc3MaterialStep::ReforgedDefault(hd) = &mut step else {
        unreachable!()
    };
    hd.fresnel = Some(SlotConstruct::construct(&mut slot_values));
    assert!(step.supports_property(&fresnel, 0));
    let mut surface = MaterialSurface::default();
    surface.steps.push(MaterialStep::Mdlx(Box::new(step)));
    surface.validate().unwrap();
    let native = LayeredSurface::try_from(surface).unwrap();
    assert!(native.shader.is_empty());
    assert_eq!(native.layers[0].shader, Some(1));
    assert!(!Wc3MaterialStep::construct(&mut slot_values).supports_property(&fresnel, 0));
}

#[test]
fn wc3_reforged_and_unresolved_steps_freeze_shader_inputs_without_reclassification() {
    let mut slot_values = SlotValues::default();
    use wc3::model::materials::mdlx::{Fresnel, MaterialStep as Wc3MaterialStep};
    use wc3::model::profiles::Universal;
    use wc3::model::{
        AnimationSample, Curve, DynamicTrack, Material, Model, Time, TimeSource, Track,
    };
    for selector in [1, 99] {
        let mut draft = Model::<Universal>::builder("shader inputs");
        let id = draft.reserve_material();
        let _target = mdlx::properties::Layer(0);
        let opacity = slot_values.add(1f32);
        let gain = slot_values.add(1f32);
        let fresnel = slot_values.add(1f32);
        draft
            .set_material(
                id,
                Material {
                    name: "animated shader".into(),
                    surface: MaterialSurface::try_from(LayeredSurface {
                        layers: vec![Layer {
                            shader: Some(selector),
                            opacity,
                            emissive_gain: Some(gain),
                            fresnel: Some(Fresnel {
                                opacity: fresnel,
                                ..SlotConstruct::construct(&mut slot_values)
                            }),
                            ..SlotConstruct::construct(&mut slot_values)
                        }],
                        ..SlotConstruct::construct(&mut slot_values)
                    })
                    .unwrap(),
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
            matches!(step.as_ref(), Wc3MaterialStep::ReforgedDefault(_)),
            selector == 1
        );
        assert_eq!(
            matches!(step.as_ref(), Wc3MaterialStep::Unresolved(_)),
            selector == 99
        );
        let layer = step.to_layer();
        assert_eq!(frozen_values[layer.opacity], 0.5);
        assert_eq!(layer.opacity, opacity);
        let gain = layer.emissive_gain.unwrap();
        assert_eq!(frozen_values[gain], 0.5);
        assert!(frozen_values.get(gain).is_some());
        let fresnel = layer.fresnel.unwrap().opacity;
        assert_eq!(frozen_values[fresnel], 0.5);
        assert!(frozen_values.get(fresnel).is_some());
    }
}

#[test]
fn wc3_known_steps_normalize_shader_spellings_and_mix_with_unresolved_steps() {
    let mut slot_values = SlotValues::default();
    use wc3::model::materials::mdlx::MaterialStep as Wc3MaterialStep;
    let original = LayeredSurface {
        shader: "Shader_HD_DefaultUnit".into(),
        layers: [2, 1, 99]
            .map(|shader| Layer {
                shader: Some(shader),
                ..SlotConstruct::construct(&mut slot_values)
            })
            .into(),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let surface = MaterialSurface::try_from(original.clone()).unwrap();
    let canonical = LayeredSurface::try_from(surface.clone()).unwrap();
    assert_eq!(canonical.shader, original.shader);
    assert_eq!(canonical.layers[0].shader, Some(0));
    assert_eq!(canonical.layers[1].shader, Some(1));
    assert_eq!(canonical.layers[2], original.layers[2]);
    assert_wc3_surface_eq(&MaterialSurface::try_from(canonical).unwrap(), &surface);

    let mut incompatible = surface;
    let MaterialStep::Mdlx(step) = &incompatible.steps[2] else {
        unreachable!()
    };
    let Wc3MaterialStep::Unresolved(mut unresolved) = step.as_ref().clone() else {
        unreachable!()
    };
    unresolved.shader_name = "another shader".into();
    incompatible
        .steps
        .push(MaterialStep::Mdlx(Box::new(Wc3MaterialStep::Unresolved(
            unresolved,
        ))));
    assert!(LayeredSurface::try_from(incompatible).is_err());
}

fn assert_wc3_surface_eq(actual: &MaterialSurface<'_>, expected: &MaterialSurface<'_>) {
    assert_eq!(actual.steps.len(), expected.steps.len());
    for (actual, expected) in actual.steps.iter().zip(&expected.steps) {
        let (MaterialStep::Mdlx(actual), MaterialStep::Mdlx(expected)) = (actual, expected) else {
            panic!("expected WC3 steps")
        };
        assert_eq!(actual, expected);
    }
}
