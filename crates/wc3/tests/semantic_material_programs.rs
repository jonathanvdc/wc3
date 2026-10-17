use wc3::model::materials::m2::MaterialStep as M2MaterialStep;
use wc3::model::materials::m3;
use wc3::model::materials::mdlx::{Layer, LayeredSurface};
use wc3::model::materials::program::{MaterialProgram, MaterialStep, MaterialSurface};
use wc3::model::materials::wmo;
use wc3::model::profiles::{m2, mdlx, SurfacePayload};
use wc3::model::PropertyPayload;

#[test]
fn layered_projection_repeats_native_declarations_on_each_step() {
    let original = LayeredSurface {
        priority_plane: -17,
        shader: "Shader_HD_DefaultUnit".into(),
        constant_color: true,
        two_sided: true,
        sort_near_z: true,
        sort_far_z: true,
        full_resolution: true,
        layers: vec![Layer::default(), Layer::default()],
        ..Default::default()
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
        ..Default::default()
    })
    .is_err());
    assert!(MaterialSurface::try_from(LayeredSurface::default())
        .unwrap()
        .steps
        .is_empty());
}

#[test]
fn single_step_projection_preserves_shader_identity_and_native_state() {
    let shader = M2MaterialStep {
        shader: 0xf123,
        geoset: 19,
        layer: 7,
        label: "authored programme".into(),
        ..Default::default()
    };
    let restored = M2MaterialStep::try_from(MaterialSurface::from(shader)).unwrap();
    assert_eq!(restored.shader, 0xf123);
    assert_eq!(restored.geoset, 19);
    assert_eq!(restored.layer, 7);
    assert_eq!(restored.label, "authored programme");

    let mut world = wmo::MaterialStep {
        shader: Some(0xffff1234),
        uv_scroll: [[0.2, -0.4], [0.5, 0.8]],
        legacy_controls: Some(0xfedcba98),
        ..Default::default()
    };
    world.state.rasterization.controls.exterior_lit = true;
    assert_eq!(
        wmo::MaterialStep::try_from(MaterialSurface::from(world.clone())).unwrap(),
        world
    );
}

#[test]
fn mixed_steps_are_valid_but_native_projection_cannot_drop_them() {
    let program = MaterialSurface {
        steps: vec![
            MaterialStep::Mdlx(Box::default()),
            MaterialStep::M2(Box::default()),
            MaterialStep::Wmo(Box::default()),
        ],
    };
    program.validate().unwrap();
    let broad = program;
    assert!(LayeredSurface::try_from(broad.clone()).is_err());
    assert!(M2MaterialStep::try_from(broad.clone()).is_err());
    assert!(wmo::MaterialStep::try_from(broad).is_err());
}

#[test]
fn native_single_step_projection_rejects_empty_and_repeated_steps() {
    for steps in [
        vec![],
        vec![
            MaterialStep::Wmo(Box::default()),
            MaterialStep::Wmo(Box::default()),
        ],
    ] {
        let broad = MaterialSurface { steps };
        assert!(wmo::MaterialStep::try_from(broad.clone()).is_err());
        assert!(M2MaterialStep::try_from(broad).is_err());
    }
}

#[test]
fn property_selection_uses_program_ordinals_and_rejects_ambiguous_model_destinations() {
    let mut program = MaterialSurface {
        steps: vec![
            MaterialStep::Wmo(Box::default()),
            MaterialStep::Mdlx(Box::default()),
            MaterialStep::M2(Box::default()),
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
    let model_opacity = m2::properties::Target::Opacity.identity().to_owned();
    assert!(program.supports_property(&model_opacity));
    program.steps.push(MaterialStep::M2(Box::default()));
    assert!(!program.supports_property(&model_opacity));
}

#[test]
fn an_empty_program_does_not_require_a_default_step_type() {
    struct NoDefault;
    let program: MaterialProgram<NoDefault> = MaterialProgram::default();
    assert!(program.steps.is_empty());
}

#[test]
fn a_mixed_program_samples_each_shader_without_changing_step_order() {
    use wc3::model::profiles::Universal;
    use wc3::model::{
        Animated, AnimationSample, Channel, Clock, Curve, Material, Model, Time, Track,
    };
    let mut draft = Model::<Universal>::builder("mixed step animation");
    let id = draft.reserve_material();
    let layered = mdlx::properties::Layer(0)
        .opacity()
        .bind(id)
        .widen::<Universal>();
    let shader = m2::properties::OPACITY.bind(id).widen::<Universal>();
    draft
        .set_material(
            id,
            Material {
                name: "mixed".into(),
                surface: MaterialSurface {
                    steps: vec![
                        MaterialStep::Mdlx(Box::new(
                            Layer {
                                opacity: Animated::new(1., layered.clone()),
                                ..Default::default()
                            }
                            .into(),
                        )),
                        MaterialStep::M2(Box::new(M2MaterialStep {
                            opacity: Animated::new(1., shader.clone()),
                            ..Default::default()
                        })),
                        MaterialStep::Wmo(Box::default()),
                    ],
                },
            },
        )
        .unwrap();
    for (target, values) in [(layered, [0., 1.]), (shader, [0.5, 1.])] {
        draft
            .add_timeline_channel(Channel::Scalar(Track {
                target,
                clock: Clock::Timeline,
                curve: Curve::linear(
                    [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()],
                    values,
                )
                .unwrap(),
            }))
            .unwrap();
    }
    let mut model = draft.finish().unwrap();
    let sample = model
        .animation_sample(AnimationSample {
            timeline_time: Time::seconds(0.5).unwrap(),
            ..Default::default()
        })
        .unwrap();
    let source = &model.material(id).unwrap().surface;
    let frozen = source.sampled(&sample);
    let MaterialStep::Mdlx(first) = &frozen.steps[0] else {
        panic!("layered")
    };
    let MaterialStep::M2(second) = &frozen.steps[1] else {
        panic!("model shader")
    };
    assert_eq!(first.to_layer().opacity.default, 0.5);
    assert_eq!(second.opacity.default, 0.75);
    assert!(first.to_layer().opacity.binding.is_none());
    assert!(second.opacity.binding.is_none());
    assert!(matches!(frozen.steps[2], MaterialStep::Wmo(_)));
    // Existing model-shader channels cannot become ambiguous through an edit.
    assert!(model
        .edit(|draft| {
            let mut material = draft.model().material(id).unwrap().clone();
            let program = &mut material.surface;
            program.steps.push(MaterialStep::M2(Box::default()));
            draft.set_material(id, material)
        })
        .is_err());
}

#[test]
fn every_m3_shader_has_a_typed_step_and_exact_native_projection() {
    use wc3::model::materials::m3::MaterialStep as M3MaterialStep;
    let native = [
        M3MaterialStep::Standard(Box::default()),
        M3MaterialStep::Displacement(Box::default()),
        M3MaterialStep::Composite(Box::default()),
        M3MaterialStep::Terrain(Box::default()),
        M3MaterialStep::Volume(Box::default()),
        M3MaterialStep::VolumeNoise(Box::default()),
        M3MaterialStep::Hair(Box::default()),
        M3MaterialStep::Creep(Box::default()),
        M3MaterialStep::SplatTerrainBake(Box::default()),
        M3MaterialStep::Reflection(Box::default()),
        M3MaterialStep::LensFlare(Box::default()),
        M3MaterialStep::DataDriven(Box::default()),
    ];
    for original in native {
        let program = MaterialSurface::from(original.clone());
        assert_eq!(program.steps.len(), 1);
        assert_eq!(M3MaterialStep::try_from(program.clone()).unwrap(), original);
        let mut extended = program;
        extended.steps.push(MaterialStep::Wmo(Box::default()));
        assert!(M3MaterialStep::try_from(extended).is_err());
    }
}

#[test]
fn diablo3_keeps_shader_inputs_and_phases() {
    use wc3::model::materials::diablo3::{MaterialStep as Material, RenderState};
    use wc3::model::materials::parameters::{OpaqueWord, ParameterEntry, ShaderTag};
    let mut original = Material::default();
    original.uber.shader_map.id = 991;
    original.uber.colors.diffuse = [0.2, 0.4, 0.6, 0.8];
    original.shader_params = vec![
        ParameterEntry {
            key: ShaderTag(7),
            value: OpaqueWord(0xffffffff),
            controls: Default::default(),
        },
        ParameterEntry {
            key: ShaderTag(7),
            value: OpaqueWord(0x7fc00000),
            controls: Default::default(),
        },
    ];
    original.opaque_passes = vec![RenderState {
        effect_file: "opaque".into(),
        ..Default::default()
    }];
    original.translucent_passes = vec![RenderState {
        effect_file: "translucent".into(),
        ..Default::default()
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
    extended.steps.push(MaterialStep::Wmo(Box::default()));
    assert!(Material::try_from(extended).is_err());
}

#[test]
fn unresolved_and_resolved_diablo4_programmes_project_exactly() {
    use wc3::model::profiles::diablo::d4::{Material, MaterialStep as D4MaterialStep};
    for steps in [
        vec![],
        vec![D4MaterialStep {
            shader_map: 197,
            value_set_override: -7,
            ..Default::default()
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
        extended.steps.push(MaterialStep::Wmo(Box::default()));
        assert!(Material::try_from(extended).is_err());
    }
}

#[test]
fn unindexed_programmed_animation_fields_require_one_matching_step() {
    use wc3::model::profiles::m3::properties::{Field, FieldPath, Target};
    let key = Target::Field(FieldPath::root(Field::Material).child(Field::ParallaxHeight))
        .identity()
        .to_owned();
    let mut program = MaterialSurface {
        steps: vec![
            MaterialStep::M3(m3::MaterialStep::Standard(Box::default())),
            MaterialStep::M3(m3::MaterialStep::Composite(Box::default())),
            MaterialStep::Wmo(Box::default()),
        ],
    };
    assert!(program.supports_property(&key));
    program
        .steps
        .push(MaterialStep::M3(m3::MaterialStep::Volume(Box::default())));
    assert!(program.supports_property(&key));
    program
        .steps
        .push(MaterialStep::M3(m3::MaterialStep::Standard(Box::default())));
    assert!(!program.supports_property(&key));
}

#[test]
fn programmed_selectors_check_optional_inputs_leaves_and_collection_bounds() {
    use wc3::model::materials::m3::{Property, StandardMaterialStep, TextureLayer};
    use wc3::model::profiles::m3::properties::{Field, FieldPath, Target};
    let root = FieldPath::root(Field::Material);
    let diffuse = root.child(Field::DiffuseLayer);
    let color = Target::Field(diffuse.child(Field::Color))
        .identity()
        .to_owned();
    let mut shader = StandardMaterialStep::default();
    let step = MaterialStep::M3(m3::MaterialStep::Standard(Box::new(shader.clone())));
    assert!(!step.supports_property(&color, 0));
    shader.diffuse = Some(TextureLayer::default());
    shader.normals.weights = vec![Property::default(); 2];
    let step = MaterialStep::M3(m3::MaterialStep::Standard(Box::new(shader)));
    assert!(step.supports_property(&color, 0));
    assert!(!step.supports_property(&Target::Field(diffuse).identity().to_owned(), 0));
    for index in 0..=2 {
        let key = Target::Field(root.child(Field::NormalBlendFactors).indexed(index))
            .identity()
            .to_owned();
        assert_eq!(step.supports_property(&key, 0), index < 2);
    }
    let composite = MaterialStep::M3(m3::MaterialStep::Composite(Box::default()));
    assert!(!composite.supports_property(&color, 0));
}

#[test]
fn wc3_shader_steps_interpret_selector_precedence_and_preserve_native_inputs() {
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
                ..Default::default()
            }],
            ..Default::default()
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
                        binding: TextureBinding::default(),
                    })
                    .into(),
                ..Default::default()
            }],
            ..Default::default()
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
    use wc3::model::materials::mdlx::{
        MaterialStep as Wc3MaterialStep, ReforgedDefaultMaterialStep,
    };
    use wc3::model::profiles::mdlx::properties::{LayerField, Target};
    let mut step = Wc3MaterialStep::ReforgedDefault(ReforgedDefaultMaterialStep::default());
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
    hd.fresnel = Some(Default::default());
    assert!(step.supports_property(&fresnel, 0));
    let mut surface = MaterialSurface::default();
    surface.steps.push(MaterialStep::Mdlx(Box::new(step)));
    surface.validate().unwrap();
    let native = LayeredSurface::try_from(surface).unwrap();
    assert!(native.shader.is_empty());
    assert_eq!(native.layers[0].shader, Some(1));
    assert!(!Wc3MaterialStep::default().supports_property(&fresnel, 0));
}

#[test]
fn wc3_reforged_and_unresolved_steps_freeze_shader_inputs_without_reclassification() {
    use wc3::model::materials::mdlx::{Fresnel, MaterialStep as Wc3MaterialStep};
    use wc3::model::profiles::Universal;
    use wc3::model::{
        Animated, AnimationSample, Channel, Clock, Curve, Material, Model, Time, Track,
    };
    for selector in [1, 99] {
        let mut draft = Model::<Universal>::builder("shader inputs");
        let id = draft.reserve_material();
        let target = mdlx::properties::Layer(0);
        let opacity = target.opacity().bind(id).widen::<Universal>();
        let gain = target.emissive_gain().bind(id).widen::<Universal>();
        let fresnel = target.fresnel_opacity().bind(id).widen::<Universal>();
        draft
            .set_material(
                id,
                Material {
                    name: "animated shader".into(),
                    surface: MaterialSurface::try_from(LayeredSurface {
                        layers: vec![Layer {
                            shader: Some(selector),
                            opacity: Animated::new(1., opacity.clone()),
                            emissive_gain: Some(Animated::new(1., gain.clone())),
                            fresnel: Some(Fresnel {
                                opacity: Animated::new(1., fresnel.clone()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    })
                    .unwrap(),
                },
            )
            .unwrap();
        for target in [opacity, gain, fresnel] {
            draft
                .add_timeline_channel(Channel::Scalar(Track {
                    target,
                    clock: Clock::Timeline,
                    curve: Curve::linear(
                        [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()],
                        [0., 1.],
                    )
                    .unwrap(),
                }))
                .unwrap();
        }
        let model = draft.finish().unwrap();
        let sample = model
            .animation_sample(AnimationSample {
                timeline_time: Time::seconds(0.5).unwrap(),
                ..Default::default()
            })
            .unwrap();
        let surface = model.material(id).unwrap().surface.sampled(&sample);
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
        assert_eq!(layer.opacity.default, 0.5);
        assert!(layer.opacity.binding.is_none());
        let gain = layer.emissive_gain.unwrap();
        assert_eq!(gain.default, 0.5);
        assert!(gain.binding.is_none());
        let fresnel = layer.fresnel.unwrap().opacity;
        assert_eq!(fresnel.default, 0.5);
        assert!(fresnel.binding.is_none());
    }
}

#[test]
fn wc3_known_steps_normalize_shader_spellings_and_mix_with_unresolved_steps() {
    use wc3::model::materials::mdlx::MaterialStep as Wc3MaterialStep;
    let original = LayeredSurface {
        shader: "Shader_HD_DefaultUnit".into(),
        layers: [2, 1, 99]
            .map(|shader| Layer {
                shader: Some(shader),
                ..Default::default()
            })
            .into(),
        ..Default::default()
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
