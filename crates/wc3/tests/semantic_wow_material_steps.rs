use wc3::model::materials::bindings::TextureBinding;
use wc3::model::materials::{m2, wmo};
use wc3::model::ModelMetadata;

use wc3::model::profiles::wmo::WorldSystem;
use wc3::model::profiles::{Universal, Wmo, M2};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{MaterialProgram, MaterialStep, Model};

#[test]
fn m2_table_contracts_project_to_stable_typed_equations() {
    let mut slot_values = SlotValues::default();
    for index in 0..36 {
        let count = match index {
            3 | 9 | 17 | 18 | 20 | 24 | 26 | 28 | 30 | 31 => 3,
            19 | 25 | 32 | 34 => 1,
            22 => 4,
            _ => 2,
        };
        let step = m2::MaterialStep::from_native(
            0x8000 + index,
            m2::MaterialDeclaration::construct(&mut slot_values),
            (0..count)
                .map(|ordinal| m2::TextureUnit {
                    contribution: slot_values.add(ordinal as f32 + 0.25),
                    ..SlotConstruct::construct(&mut slot_values)
                })
                .collect(),
            m2::GuildPalette::default(),
        );
        assert_eq!(
            matches!(step, m2::MaterialStep::Unresolved(_)),
            matches!(index, 29..=31),
            "table entry {index}"
        );
        step.validate().unwrap();
        let reconstructed = m2::MaterialStep::from_native(
            step.native_shader(),
            step.declaration().clone(),
            step.inputs().into_iter().cloned().collect(),
            step.guild_palette().unwrap_or_default(),
        );
        assert_eq!(reconstructed, step, "table entry {index}");
        if !matches!(step, m2::MaterialStep::Unresolved(_)) {
            for ordinal in 0..count {
                assert_eq!(
                    slot_values[step.inputs()[ordinal].contribution],
                    ordinal as f32 + 0.25
                );
            }
        }
    }
}

#[test]
fn m2_legacy_equations_normalize_flags_and_retain_unsampled_bindings() {
    let mut slot_values = SlotValues::default();
    for shader in [0, 0x10, 0x11, 4, 0x16, 3, 0x13, 7, 0x17, 0x88, 0x4011] {
        for count in [1, 2, 4] {
            let step = m2::MaterialStep::from_native(
                shader,
                SlotConstruct::construct(&mut slot_values),
                vec![SlotConstruct::construct(&mut slot_values); count],
                SlotConstruct::construct(&mut slot_values),
            );
            assert_eq!(
                matches!(step, m2::MaterialStep::Unresolved(_)),
                count == 1 && shader & 0x4080 == 0x4000
            );
            assert_eq!(step.inputs().len(), count);
            let roundtrip = m2::MaterialStep::from_native(
                step.native_shader(),
                step.declaration().clone(),
                step.inputs().into_iter().cloned().collect(),
                SlotConstruct::construct(&mut slot_values),
            );
            assert_eq!(roundtrip, step, "selector {shader:#x}, count {count}");
        }
    }
}

#[test]
fn m2_unresolved_layouts_retain_raw_selection_and_bindings() {
    let mut slot_values = SlotValues::default();
    for (shader, count) in [
        (0x801d, 2),
        (0x801e, 3),
        (0x801f, 3),
        (0x801c, 6),
        (0xffff, 2),
        (0x8000, 1),
    ] {
        let step = m2::MaterialStep::from_native(
            shader,
            SlotConstruct::construct(&mut slot_values),
            vec![SlotConstruct::construct(&mut slot_values); count],
            SlotConstruct::construct(&mut slot_values),
        );
        let m2::MaterialStep::Unresolved(value) = step else {
            panic!("expected unresolved")
        };
        assert_eq!(value.shader, shader);
        assert_eq!(value.textures.len(), count);
    }
}

#[test]
fn wmo_equations_retain_spare_bindings_and_keep_special_paths_unresolved() {
    let mut slot_values = SlotValues::default();
    for shader in 0..26 {
        let step = wmo::MaterialStep::from_native(
            Some(shader),
            SlotConstruct::construct(&mut slot_values),
            vec![SlotConstruct::construct(&mut slot_values); 3],
        );
        let known = matches!(shader,0 | 3..=6 | 9 | 11..=13 | 15 | 17..=20);
        assert_eq!(
            !matches!(step, wmo::MaterialStep::Unresolved(_)),
            known,
            "shader {shader}"
        );
        assert_eq!(step.native_shader(), Some(shader));
        let roundtrip = wmo::MaterialStep::from_native(
            step.native_shader(),
            step.declaration().clone(),
            step.inputs().into_iter().cloned().collect(),
        );
        assert_eq!(roundtrip, step, "shader {shader}");
    }
    let mut draft = Model::<Wmo>::builder(wc3::model::profiles::wmo::ModelMetadata::default());
    let texture = draft.add_file_texture("retained.blp").unwrap();
    let mut inputs = vec![SlotConstruct::construct(&mut slot_values); 3];
    inputs[2] = TextureBinding {
        image: Some(texture.resource),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let step =
        wmo::MaterialStep::from_native(Some(0), SlotConstruct::construct(&mut slot_values), inputs);
    let wmo::MaterialStep::Diffuse(value) = &step else {
        panic!("diffuse")
    };
    assert_eq!(value.unused_inputs.len(), 2);
    assert_eq!(value.unused_inputs[1].image, Some(texture.resource));
    assert_eq!(step.inputs()[2].image, Some(texture.resource));
}

#[test]
fn native_profiles_use_the_same_shader_specific_steps_as_universal() {
    let mut slot_values = SlotValues::default();
    use wc3::model::Material;
    let mut draft = Model::<M2>::builder("typed M2");
    let program = MaterialProgram {
        declaration: Default::default(),
        steps: vec![
            m2::MaterialStep::MaskedDualCrossfade(Box::new(SlotConstruct::construct(
                &mut slot_values,
            ))),
            m2::MaterialStep::Diffuse(SlotConstruct::construct(&mut slot_values)),
        ],
    };
    let id = draft
        .add_material(Material {
            name: "crossfade".into(),
            surface: program.clone(),
        })
        .unwrap();
    let broad = {
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .unwrap()
    .widen::<Universal>();
    let narrow = broad.try_narrow::<M2>().unwrap();
    assert_eq!(narrow.material(id).unwrap().surface, program);

    let mut draft = Model::<Wmo>::builder(wc3::model::profiles::wmo::ModelMetadata::default());
    draft.add_system(WorldSystem::default()).unwrap();
    let step = wmo::MaterialStep::LayeredDiffuse(SlotConstruct::construct(&mut slot_values));
    let id = draft
        .add_material(Material {
            name: "layers".into(),
            surface: step.clone(),
        })
        .unwrap();
    let broad = {
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .unwrap()
    .widen::<Universal>();
    let narrow = broad.try_narrow::<Wmo>().unwrap();
    assert_eq!(narrow.material(id).unwrap().surface, step);
}

#[test]
fn named_crossfade_inputs_keep_native_animation_ordinals_when_frozen() {
    let mut slot_values = SlotValues::default();

    use wc3::model::{AnimationSample, Curve, DynamicTrack, Material, Time, TimeSource, Track};
    let mut draft = Model::<Universal>::builder(ModelMetadata::M2(Default::default()));
    let id = draft.reserve_material();
    let target = slot_values.add(0f32);
    let mut payload = m2::MaskedDualCrossfadeMaterialStep::construct(&mut slot_values);
    payload.first.contribution = target;
    draft
        .set_material(
            id,
            Material {
                name: "crossfade".into(),
                surface: m2::MaterialStep::MaskedDualCrossfade(Box::new(payload)).into(),
            },
        )
        .unwrap();
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
    let model = {
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .unwrap();
    let sample = model
        .animation_sample(AnimationSample {
            clip_time: Time::seconds(0.75).unwrap(),
            ..SlotConstruct::construct(&mut slot_values)
        })
        .unwrap();
    let frozen_values = sample.evaluate();
    let frozen = model.material(id).unwrap().surface.clone();
    let MaterialStep::M2(step) = &frozen.steps[0] else {
        panic!("M2")
    };
    let m2::MaterialStep::MaskedDualCrossfade(value) = step.as_ref() else {
        panic!("masked crossfade")
    };
    assert_eq!(frozen_values[value.first.contribution], 0.75);
    assert_eq!(value.first.contribution, target);
    assert_eq!(frozen_values[step.inputs()[1].contribution], 0.75);
}

#[test]
fn single_input_uv1_keeps_unresolved_transform_ownership_and_native_selection() {
    let mut slot_values = SlotValues::default();
    let step = m2::MaterialStep::from_native(
        0x4010,
        SlotConstruct::construct(&mut slot_values),
        vec![SlotConstruct::construct(&mut slot_values)],
        SlotConstruct::construct(&mut slot_values),
    );
    let m2::MaterialStep::Unresolved(value) = &step else {
        panic!("UV1 ownership must remain unresolved")
    };
    assert_eq!(step.native_shader(), 0x4010);
    assert!(matches!(
        value.partial_equation(),
        Some(m2::MaterialStep::Diffuse(_))
    ));
}

#[test]
fn unresolved_coordinate_selectors_are_not_replaced_by_environment_routes() {
    let mut slot_values = SlotValues::default();
    let mut input = m2::TextureUnit::construct(&mut slot_values);
    input.coordinates.source = -2;
    let step = m2::MaterialStep::from_native(
        0x8000,
        SlotConstruct::construct(&mut slot_values),
        vec![SlotConstruct::construct(&mut slot_values), input],
        SlotConstruct::construct(&mut slot_values),
    );
    let m2::MaterialStep::Unresolved(value) = step else {
        panic!("unknown coordinates")
    };
    assert_eq!(value.shader, 0x8000);
    assert_eq!(value.textures[1].coordinates.source, -2);
}
