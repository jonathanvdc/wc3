use super::super::ModelMaterialKey;
use super::*;
use wc3::model::Animated;

#[test]
fn third_input_reuses_raw_streams_with_its_own_transform() {
    for (sources, route, stream) in [
        ([0, 1, 0], 0, None),
        ([0, 1, 1], 2, None),
        ([0, 1, 2], 3, Some(2)),
        ([0, 1, -1], 1, Some(0)),
    ] {
        let mut shader: model_shader::ShaderPass<'static> = model_shader::ShaderPass {
            shader: 0x8012,
            textures: sources
                .into_iter()
                .map(|source| {
                    let mut input = model_shader::TextureUnit::default();
                    input.coordinates.source = source;
                    input
                })
                .collect(),
            ..Default::default()
        };
        shader.textures[2].coordinates.transform.translation = Animated::constant([0.25, 0.5, 0.]);
        let model = Model::builder("coordinate routes").finish().unwrap();
        let pass = compile_model_shader(&model, &shader, &TextureImages::default()).unwrap();
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
    for (sources, expected_route, expected_stream) in [
        ([0, 0, 0, 1], 3, Some(1)),
        ([0, 1, 0, 1], 2, None),
        ([0, 1, 2, 2], 3, Some(2)),
        ([0, 1, 2, 0], 0, Some(2)),
    ] {
        let mut shader: model_shader::ShaderPass<'static> = model_shader::ShaderPass {
            shader: 0x8016,
            textures: sources
                .into_iter()
                .map(|source| {
                    let mut input = model_shader::TextureUnit::default();
                    input.coordinates.source = source;
                    input
                })
                .collect(),
            ..Default::default()
        };
        shader.textures[3].coordinates.transform.translation = Animated::constant([0.25, 0.5, 0.]);
        shader.textures[3].contribution = Animated::constant(0.4);
        let model = Model::builder("masked crossfade routes").finish().unwrap();
        let pass = compile_model_shader(&model, &shader, &TextureImages::default()).unwrap();
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
    let model = Model::builder("invalid crossfade").finish().unwrap();
    for sources in [
        vec![0, 0, 0],
        vec![0; 5],
        vec![0; 6],
        vec![0, 1, 2, 3],
        vec![0, 0, 0, -1],
    ] {
        let shader = model_shader::ShaderPass {
            shader: 0x8016,
            textures: sources
                .into_iter()
                .map(|source| {
                    let mut input = model_shader::TextureUnit::default();
                    input.coordinates.source = source;
                    input
                })
                .collect(),
            ..Default::default()
        };
        let pass = compile_model_shader(&model, &shader, &TextureImages::default()).unwrap();
        assert_eq!(pass.state.combiner.mode.x, 0);
        assert_eq!(pass.diagnostics.len(), 1);
    }
}

#[test]
fn shared_primary_and_secondary_coordinates_avoid_duplicate_vertex_streams() {
    let model = Model::builder("shared UV streams").finish().unwrap();
    for (shader, sources, secondary) in [
        (0x8005, vec![0, 0], None),
        (0x8016, vec![0, 0, 0, 1], None),
        (0x8000, vec![0, 0], Some(0)),
        (0x800f, vec![0, 1], Some(1)),
    ] {
        let surface: model_shader::ShaderPass<'static> = model_shader::ShaderPass {
            shader,
            textures: sources
                .into_iter()
                .map(|source| {
                    let mut input = model_shader::TextureUnit::default();
                    input.coordinates.source = source;
                    input
                })
                .collect(),
            ..Default::default()
        };
        let pass = compile_model_shader(&model, &surface, &TextureImages::default()).unwrap();
        assert_eq!(pass.secondary_coordinate, secondary);
    }
}
