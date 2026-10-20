use super::super::{ModelMaterial, ModelMaterialKey};
use super::*;
use crate::material::native_fixtures::m2;
use bevy::math::Vec3;
use wc3::model::ModelMetadata;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
#[test]
fn third_input_reuses_raw_streams_with_its_own_transform() {
    let mut slot_values = SlotValues::default();
    for (sources, route, stream) in [
        ([0, 1, 0], 0, None),
        ([0, 1, 1], 2, None),
        ([0, 1, 2], 3, Some(2)),
        ([0, 1, -1], 1, Some(0)),
    ] {
        let mut shader: m2::Material<'static> = m2::Material {
            shader: 0x8012,
            textures: sources
                .into_iter()
                .map(|source| {
                    let mut input = m2::TextureUnit::construct(&mut slot_values);
                    input.coordinates.source = source;
                    input
                })
                .collect(),
            ..SlotConstruct::construct(&mut slot_values)
        };
        shader.textures[2].coordinates.transform.translation =
            slot_values.add(Vec3::from_array([0.25, 0.5, 0.]));
        let model = {
            let mut draft = Model::<wc3::model::profiles::Universal>::builder(ModelMetadata::M2(
                Default::default(),
            ));
            draft.import_values(&slot_values).unwrap();
            draft.finish().unwrap()
        };
        let mut model = model.clone();
        model
            .edit(|builder| builder.import_values(&slot_values))
            .unwrap();
        let pass = compile_m2(&model, &shader.clone().into(), &TextureImages::default()).unwrap();
        assert!(pass.diagnostics.is_empty());
        assert_eq!(pass.state.combiner.coordinates.w, route);
        assert_eq!(pass.third_coordinate, stream);
        let key = ModelMaterialKey::from(&pass.state);
        assert_eq!(key.third_uv, matches!(route, 1 | 3));
        assert_eq!(key.generated_coordinates, route == 1);
        if route == 1 {
            assert_eq!(pass.state.combiner.third_transform, Mat3::IDENTITY);
            assert!(pass.animation.third_transform.is_none());
        } else {
            assert_ne!(pass.state.combiner.third_transform, Mat3::IDENTITY);
            assert!(pass.animation.third_transform.is_some());
        }
    }
}

#[test]
fn masked_crossfade_reuses_streams_and_keeps_mask_transform() {
    let mut slot_values = SlotValues::default();
    for (sources, expected_route, expected_stream) in [
        ([0, 0, 0, 1], 3, Some(1)),
        ([0, 1, 0, 1], 2, None),
        ([0, 1, 2, 2], 3, Some(2)),
        ([0, 1, 2, 0], 0, Some(2)),
    ] {
        let mut shader: m2::Material<'static> = m2::Material {
            shader: 0x8016,
            textures: sources
                .into_iter()
                .map(|source| {
                    let mut input = m2::TextureUnit::construct(&mut slot_values);
                    input.coordinates.source = source;
                    input
                })
                .collect(),
            ..SlotConstruct::construct(&mut slot_values)
        };
        shader.textures[3].coordinates.transform.translation =
            slot_values.add(Vec3::from_array([0.25, 0.5, 0.]));
        shader.textures[3].contribution = slot_values.add(0.4);
        let model = {
            let mut draft = Model::<wc3::model::profiles::Universal>::builder(ModelMetadata::M2(
                Default::default(),
            ));
            draft.import_values(&slot_values).unwrap();
            draft.finish().unwrap()
        };
        let mut model = model.clone();
        model
            .edit(|builder| builder.import_values(&slot_values))
            .unwrap();
        let pass = compile_m2(&model, &shader.clone().into(), &TextureImages::default()).unwrap();
        assert!(pass.diagnostics.is_empty());
        assert_eq!(pass.state.combiner.mode.x, 23);
        assert_eq!(pass.state.combiner.additional_coordinates.x, expected_route);
        assert_eq!(pass.third_coordinate, expected_stream);
        assert_eq!(pass.state.combiner.weights.w, 0.4);
        assert_ne!(pass.state.combiner.fourth_transform, Mat3::IDENTITY);
        assert!(pass.animation.fourth_transform.is_some());
        assert!(pass.animation.input_weights[3].is_some());
    }
}

#[test]
fn masked_crossfade_rejects_missing_surplus_and_unrepresentable_inputs() {
    let mut slot_values = SlotValues::default();
    let model = {
        let mut draft = Model::<wc3::model::profiles::Universal>::builder(ModelMetadata::M2(
            Default::default(),
        ));
        draft.import_values(&slot_values).unwrap();
        draft.finish().unwrap()
    };
    for sources in [
        vec![0, 0, 0],
        vec![0; 5],
        vec![0; 6],
        vec![0, 1, 2, 3],
        vec![0, 0, 0, -1],
    ] {
        let shader = m2::Material {
            shader: 0x8016,
            textures: sources
                .into_iter()
                .map(|source| {
                    let mut input = m2::TextureUnit::construct(&mut slot_values);
                    input.coordinates.source = source;
                    input
                })
                .collect(),
            ..SlotConstruct::construct(&mut slot_values)
        };
        let mut model = model.clone();
        model
            .edit(|builder| builder.import_values(&slot_values))
            .unwrap();
        let pass = compile_m2(&model, &shader.clone().into(), &TextureImages::default()).unwrap();
        assert_eq!(pass.state.combiner.mode.x, 0);
        assert_eq!(pass.diagnostics.len(), 1);
    }
}

#[test]
fn shared_primary_and_secondary_coordinates_avoid_duplicate_vertex_streams() {
    let mut slot_values = SlotValues::default();
    let model = {
        let mut draft = Model::<wc3::model::profiles::Universal>::builder(ModelMetadata::M2(
            Default::default(),
        ));
        draft.import_values(&slot_values).unwrap();
        draft.finish().unwrap()
    };
    for (shader, sources, secondary) in [
        (0x8005, vec![0, 0], None),
        (0x8016, vec![0, 0, 0, 1], None),
        (0x8000, vec![0, 0], Some(0)),
        (0x800f, vec![0, 1], Some(1)),
    ] {
        let surface: m2::Material<'static> = m2::Material {
            shader,
            textures: sources
                .into_iter()
                .map(|source| {
                    let mut input = m2::TextureUnit::construct(&mut slot_values);
                    input.coordinates.source = source;
                    input
                })
                .collect(),
            ..SlotConstruct::construct(&mut slot_values)
        };
        let mut model = model.clone();
        model
            .edit(|builder| builder.import_values(&slot_values))
            .unwrap();
        let pass = compile_m2(&model, &surface.clone().into(), &TextureImages::default()).unwrap();
        assert_eq!(pass.secondary_coordinate, secondary);
    }
}

#[test]
fn all_m2_texture_inputs_share_centered_pretranslation_in_rest_and_sampled_states() {
    use bevy::math::{Mat2, Quat, Vec2};
    use std::array::from_fn;
    use wc3::model::profiles::Universal;
    use wc3::model::{AnimationProgram, AnimationSample};

    let mut values = SlotValues::default();
    let shader: m2::Material<'static> = m2::Material {
        shader: 0x8016,
        textures: [0, 1, 0, 1]
            .into_iter()
            .enumerate()
            .map(|(index, source)| {
                let mut input = m2::TextureUnit::construct(&mut values);
                input.coordinates.source = source;
                let factor = index as f32 + 1.;
                input.coordinates.transform.translation =
                    values.add(Vec3::new(0.12 * factor, -0.07 * factor, 0.));
                input.coordinates.transform.rotation =
                    values.add(Quat::from_rotation_z(0.3 * factor));
                input.coordinates.transform.scale =
                    values.add(Vec3::new(1.2 + 0.1 * factor, 0.7, 1.));
                input
            })
            .collect(),
        ..SlotConstruct::construct(&mut values)
    };
    let mut builder = Model::<Universal>::builder(ModelMetadata::M2(Default::default()));
    builder.import_values(&values).unwrap();
    let model = builder.finish().unwrap();
    let pass = compile_m2(&model, &shader.clone().into(), &TextureImages::default()).unwrap();
    let uv = Vec2::new(0.18, 0.83);
    let expected = from_fn::<_, 4, _>(|index| {
        let factor = index as f32 + 1.;
        Vec2::splat(0.5)
            + Mat2::from_angle(0.3 * factor)
                * (Vec2::new(1.2 + 0.1 * factor, 0.7)
                    * (uv + Vec2::new(0.12 * factor, -0.07 * factor) - Vec2::splat(0.5)))
    });
    let rest = [
        pass.material.uv_transform,
        Affine2::from_mat3(pass.state.combiner.uv_transform),
        Affine2::from_mat3(pass.state.combiner.third_transform),
        Affine2::from_mat3(pass.state.combiner.fourth_transform),
    ];
    for (transform, expected) in rest.into_iter().zip(expected) {
        assert!((transform.transform_point2(uv) - expected).length() < 1e-6);
    }
    let mut material = ModelMaterial {
        base: pass.material.clone(),
        extension: pass.state.clone(),
    };
    material.base.uv_transform = Affine2::IDENTITY;
    material.extension.combiner.uv_transform = Mat3::IDENTITY;
    material.extension.combiner.third_transform = Mat3::IDENTITY;
    material.extension.combiner.fourth_transform = Mat3::IDENTITY;
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(AnimationSample::default()).unwrap();
    assert!(pass.animation.apply_to_material(&sample, &mut material));
    let sampled = [
        material.base.uv_transform,
        Affine2::from_mat3(material.extension.combiner.uv_transform),
        Affine2::from_mat3(material.extension.combiner.third_transform),
        Affine2::from_mat3(material.extension.combiner.fourth_transform),
    ];
    for (transform, expected) in sampled.into_iter().zip(expected) {
        assert!((transform.transform_point2(uv) - expected).length() < 1e-6);
    }
}
