use wc3::model::animation::ValueKeyframe;
use wc3::model::animation::{Animatable, Track};

use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::materials::{
    Layer, LayerFilterMode, LayerShadingFlags, LayerTextureSlot, Material, MaterialRenderFlags,
    ShaderType,
};
use wc3::model::{
    Model, ModelVersion, V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900,
};

fn sample_material<V: ModelVersion>() -> Material<V> {
    let mut layer = Layer::<V>::new();
    layer.filter_mode = LayerFilterMode::Transparent;
    layer.texture_id = Animatable::Static(2);
    layer.alpha = Animatable::Static(0.5);
    let mut material = Material::<V>::new();
    material.layers = [layer].to_vec();
    Material::<V>::decode_mdx(&material.encode_mdx().unwrap()).unwrap()
}

#[test]
fn material_layers_round_trip_across_layouts() {
    check_version::<V800>();
    check_version::<V900>();
    check_version::<V1000>();
    check_version::<V1100>();
    check_version::<V1200>();
    check_version::<V1300>();
    check_version::<V1400>();
    check_version::<V1600>();
    check_version::<V1800>();
}

fn check_version<V: ModelVersion>() {
    let version = V::NUMBER;
    let mut material = sample_material::<V>();
    material.priority_plane = 3;
    material.render_mode = MaterialRenderFlags(7);
    let mut model = Model::<V>::new();
    model.set_materials(&[material]);
    let decoded = Model::<V>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    let materials = decoded.materials();
    assert_eq!(materials[0].version(), version);
    assert_eq!(materials[0].priority_plane, 3);
    assert_eq!(materials[0].render_mode.bits(), 7);
    assert!(materials[0].render_mode.constant_color());
    let layers = materials[0].layers.as_slice();
    assert_eq!(layers[0].version(), version);
    assert_eq!(layers[0].filter_mode, LayerFilterMode::Transparent);
    assert_eq!(layers[0].texture_id, Animatable::Static(2));
    assert_eq!(layers[0].alpha, Animatable::Static(0.5));
    assert_eq!(layers[0].shading_flags, LayerShadingFlags::default());
    assert_eq!(
        Layer::<V>::decode_mdx(&layers[0].encode_mdx().unwrap()).unwrap(),
        layers[0]
    );
}

#[test]
fn builds_material_with_reforged_layer() {
    let mut layer = Layer::<V1100>::new();
    layer.texture_id = Animatable::Static(4);
    layer.set_shader_type(ShaderType::new(2));
    layer.set_fresnel_color(Animatable::Static([0.1, 0.2, 0.3]));
    let mut material = Material::<V1100>::new();
    material.layers = [layer].to_vec();
    let parsed = Material::<V1100>::decode_mdx(&material.encode_mdx().unwrap()).unwrap();
    let layers = parsed.layers.as_slice();
    assert_eq!(layers[0].texture_id, Animatable::Static(4));
    assert_eq!(layers[0].shader_type(), ShaderType::SD_FIXED_FUNCTION);
    assert_eq!(
        layers[0].fresnel_color(),
        Animatable::Static([0.1, 0.2, 0.3])
    );
    assert!(layers[0].texture_slots().is_empty());
}

#[test]
fn shader_path_round_trip_in_legacy_reforged_material() {
    let mut material = Material::<V1000>::new();
    material.set_shader("Shaders\\Unit.shader").unwrap();
    assert_eq!(
        Material::<V1000>::decode_mdx(&material.encode_mdx().unwrap())
            .unwrap()
            .shader()
            .as_ref(),
        "Shaders\\Unit.shader"
    );
    assert!(Material::<V1800>::new().try_set_shader("unused").is_err());
}

#[test]
fn reforged_layer_texture_slot_and_tracks_round_trip() {
    let mut layer = Layer::<V1800>::new();
    let texture_key = ValueKeyframe {
        frame: 100,
        value: 17u32,
    };
    let texture_track = Track::<u32>::linear(vec![texture_key], None).unwrap();
    layer.set_texture_slots(&[LayerTextureSlot {
        texture_id: Animatable::Both {
            value: 3,
            track: texture_track.clone(),
        },
        texture_type: 2,
    }]);
    let alpha_track = Track::<f32>::linear(
        vec![ValueKeyframe {
            frame: 100,
            value: 0.5,
        }],
        None,
    )
    .unwrap();
    layer.alpha.set_track(alpha_track.clone());
    let parsed = Layer::<V1800>::decode_mdx(&layer.encode_mdx().unwrap()).unwrap();
    assert_eq!(
        parsed.texture_slots()[0]
            .texture_id
            .track()
            .unwrap()
            .linear_keys()
            .unwrap()[0]
            .value,
        17
    );
    assert_eq!(parsed.alpha.track(), Some(&alpha_track));
}

#[test]
fn unlit_layer_flag_round_trip() {
    let mut layer = Layer::<V1800>::new();
    let mut flags = layer.shading_flags;
    flags.set_unlit(true);
    layer.shading_flags = flags;
    assert!(Layer::<V1800>::decode_mdx(&layer.encode_mdx().unwrap())
        .unwrap()
        .shading_flags
        .unlit());
}

#[test]
fn version_specific_fields_do_not_write_into_legacy_tracks() {
    let mut layer = Layer::<V800>::new();
    let before = layer.encode_mdx().unwrap();
    assert!(layer
        .try_set_emissive_gain(Animatable::Static(0.5))
        .is_err());
    assert!(layer
        .try_set_fresnel_opacity(Animatable::Static(0.5))
        .is_err());
    assert!(layer.try_set_shader_type(ShaderType::new(1)).is_err());
    assert_eq!(layer.encode_mdx().unwrap(), before);
}

#[test]
fn preserves_shader_padding_and_float_bits() {
    let mut material = Material::<V1000>::new();
    material.layers = [Layer::<V1000>::new()].to_vec();
    let mut bytes = material.encode_mdx().unwrap();
    bytes[20] = 0xaf;
    let layer_start = 100;
    bytes[layer_start + 24..layer_start + 28].copy_from_slice(&0x7fa1_2345u32.to_le_bytes());
    let parsed = Material::<V1000>::decode_mdx(&bytes).unwrap();
    assert_eq!(parsed.encode_mdx().unwrap(), bytes);
    assert_eq!(
        parsed.layers[0].alpha.value().unwrap().to_bits(),
        0x7fa1_2345
    );
}

#[test]
fn independent_layer_fields_preserve_wire_order_across_versions() {
    fn check<V: ModelVersion>() {
        use wc3::model::materials::LayerFresnel;

        let mut layer = Layer::<V>::new();
        let fresnel = LayerFresnel {
            color: Animatable::Static([0.1, 0.2, 0.3]),
            opacity: Animatable::Static(0.4),
            team_color: Animatable::Static(0.5),
        };
        let slot = LayerTextureSlot {
            texture_id: Animatable::Static(7),
            texture_type: 2,
        };
        let defaults = layer.encode_mdx().unwrap();
        let has_gain = V::NUMBER >= 900;
        let has_fresnel = V::NUMBER >= 1000;
        let has_slots = V::NUMBER >= 1100;
        assert_eq!(layer.try_emissive_gain().is_ok(), has_gain);
        assert_eq!(
            layer.try_set_emissive_gain(Animatable::Static(1.5)).is_ok(),
            has_gain
        );
        assert_eq!(layer.try_fresnel().is_ok(), has_fresnel);
        assert_eq!(layer.try_set_fresnel(fresnel.clone()).is_ok(), has_fresnel);
        assert_eq!(layer.try_fresnel_color().is_ok(), has_fresnel);
        assert_eq!(layer.try_fresnel_opacity().is_ok(), has_fresnel);
        assert_eq!(layer.try_fresnel_team_color().is_ok(), has_fresnel);
        assert_eq!(layer.try_shader_type().is_ok(), has_slots);
        assert_eq!(
            layer.try_set_shader_type(ShaderType::new(3)).is_ok(),
            has_slots
        );
        assert_eq!(layer.try_texture_slots().is_ok(), has_slots);
        assert_eq!(layer.try_set_texture_slots(&[slot]).is_ok(), has_slots);
        if !has_gain {
            assert_eq!(layer.encode_mdx().unwrap(), defaults);
        }

        let mut expected = vec![0; 4];
        for value in [0u32, 0, 0, u32::MAX, 0, 1.0f32.to_bits()] {
            expected.extend(value.to_le_bytes());
        }
        if has_gain {
            expected.extend(1.5f32.to_le_bytes());
            assert_eq!(layer.try_emissive_gain().unwrap(), Animatable::Static(1.5));
        }
        if has_fresnel {
            for value in [0.1f32, 0.2, 0.3, 0.4, 0.5] {
                expected.extend(value.to_le_bytes());
            }
            assert_eq!(layer.try_fresnel().unwrap(), fresnel);
        }
        if has_slots {
            for value in [3u32, 1, 7, 2] {
                expected.extend(value.to_le_bytes());
            }
        }
        let size = expected.len() as u32;
        expected[..4].copy_from_slice(&size.to_le_bytes());
        assert_eq!(layer.encode_mdx().unwrap(), expected);
        assert_eq!(Layer::<V>::decode_mdx(&expected).unwrap(), layer);
    }

    check::<V800>();
    check::<V900>();
    check::<V1000>();
    check::<V1100>();
    check::<V1200>();
    check::<V1300>();
    check::<V1400>();
    check::<V1600>();
    check::<V1800>();
}

#[test]
fn fresnel_convenience_accessors_preserve_other_components() {
    use wc3::model::materials::LayerFresnel;

    let mut layer = Layer::<V1000>::new();
    assert_eq!(layer.fresnel(), LayerFresnel::default());
    layer.set_fresnel(LayerFresnel {
        color: Animatable::Static([0.1, 0.2, 0.3]),
        opacity: Animatable::Static(0.4),
        team_color: Animatable::Static(0.5),
    });
    layer.set_fresnel_color(Animatable::Static([0.6, 0.7, 0.8]));
    layer.set_fresnel_opacity(Animatable::Static(0.9));
    layer.set_fresnel_team_color(Animatable::Static(1.0));
    assert_eq!(layer.fresnel_color(), Animatable::Static([0.6, 0.7, 0.8]));
    assert_eq!(layer.fresnel_opacity(), Animatable::Static(0.9));
    assert_eq!(layer.fresnel_team_color(), Animatable::Static(1.0));
    assert_eq!(layer.try_fresnel().unwrap(), layer.fresnel());

    let mut legacy = Layer::<V900>::new();
    let before = legacy.encode_mdx().unwrap();
    assert!(legacy
        .try_set_fresnel_color(Animatable::Static([0.0; 3]))
        .is_err());
    assert!(legacy
        .try_set_fresnel_opacity(Animatable::Static(1.0))
        .is_err());
    assert!(legacy
        .try_set_fresnel_team_color(Animatable::Static(1.0))
        .is_err());
    assert_eq!(legacy.encode_mdx().unwrap(), before);
}

#[test]
fn infallible_layer_accessors_cover_supported_versions() {
    use wc3::model::materials::LayerFresnel;
    use wc3::model::{
        SupportsEmissiveGain, SupportsFresnel, SupportsLayerShaderTypeId, SupportsLayerTextureSlots,
    };

    fn check_gain<V: SupportsEmissiveGain>() {
        let mut layer = Layer::<V>::new();
        assert_eq!(layer.emissive_gain(), Animatable::Static(1.0));
        let encoded = layer.encode_mdx().unwrap();
        assert_eq!(&encoded[28..32], &1.0f32.to_le_bytes());
        assert_eq!(
            Layer::<V>::decode_mdx(&encoded).unwrap().emissive_gain(),
            Animatable::Static(1.0)
        );
        layer.set_emissive_gain(Animatable::Static(1.5));
        assert_eq!(layer.emissive_gain(), Animatable::Static(1.5));
        assert_eq!(layer.try_emissive_gain().unwrap(), layer.emissive_gain());
    }
    fn check_fresnel<V: SupportsFresnel>() {
        let mut layer = Layer::<V>::new();
        layer.set_fresnel(LayerFresnel::default());
        layer.set_fresnel_color(Animatable::Static([0.1, 0.2, 0.3]));
        layer.set_fresnel_opacity(Animatable::Static(0.4));
        layer.set_fresnel_team_color(Animatable::Static(0.5));
        assert_eq!(layer.fresnel_color(), Animatable::Static([0.1, 0.2, 0.3]));
        assert_eq!(layer.fresnel_opacity(), Animatable::Static(0.4));
        assert_eq!(layer.fresnel_team_color(), Animatable::Static(0.5));
        assert_eq!(layer.try_fresnel().unwrap(), layer.fresnel());
    }
    fn check_slots<V: SupportsLayerShaderTypeId + SupportsLayerTextureSlots>() {
        let mut layer = Layer::<V>::new();
        layer.set_shader_type(ShaderType::new(2));
        layer.set_texture_slots(&[LayerTextureSlot {
            texture_id: Animatable::Static(7),
            texture_type: 3,
        }]);
        assert_eq!(layer.shader_type(), ShaderType::SD_FIXED_FUNCTION);
        assert_eq!(layer.texture_slots()[0].texture_id, Animatable::Static(7));
        assert_eq!(layer.try_shader_type().unwrap(), layer.shader_type());
        assert_eq!(layer.try_texture_slots().unwrap(), layer.texture_slots());
    }

    check_gain::<V900>();
    check_gain::<V1000>();
    check_gain::<V1100>();
    check_gain::<V1200>();
    check_gain::<V1300>();
    check_gain::<V1400>();
    check_gain::<V1600>();
    check_gain::<V1800>();
    check_fresnel::<V1000>();
    check_fresnel::<V1100>();
    check_fresnel::<V1200>();
    check_fresnel::<V1300>();
    check_fresnel::<V1400>();
    check_fresnel::<V1600>();
    check_fresnel::<V1800>();
    check_slots::<V1100>();
    check_slots::<V1200>();
    check_slots::<V1300>();
    check_slots::<V1400>();
    check_slots::<V1600>();
    check_slots::<V1800>();
}
