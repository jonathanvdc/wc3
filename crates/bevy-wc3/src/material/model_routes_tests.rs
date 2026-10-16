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
