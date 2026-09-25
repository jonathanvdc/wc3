use wc3_mdx::{
    AnimationTrack, Keyframe, Layer, LayerShadingFlags, LayerTextureSlot, Material,
    MaterialRenderFlags, Model,
};

fn sample_material(version: u32) -> Material {
    let mut layer = Layer::new(version);
    layer.set_filter_mode(1).unwrap();
    layer.set_texture_id(2).unwrap();
    layer.set_alpha(0.5).unwrap();
    let mut material = Material::new(version);
    material.set_layers(&[layer]).unwrap();
    Material::from_bytes(version, material.as_bytes()).unwrap()
}

#[test]
fn material_layers_round_trip_across_layouts() {
    for version in [800, 900, 1000, 1100, 1200, 1800] {
        let mut material = sample_material(version);
        material.set_priority_plane(3).unwrap();
        material.set_raw_render_mode(7).unwrap();
        let mut model = Model::new(version);
        model.set_materials(&[material]).unwrap();
        let decoded = Model::from_bytes(&model.to_bytes().unwrap()).unwrap();
        let materials = decoded.materials().unwrap();
        assert_eq!(materials[0].version(), version);
        assert_eq!(materials[0].priority_plane().unwrap(), 3);
        assert_eq!(materials[0].raw_render_mode().unwrap(), 7);
        assert!(materials[0]
            .render_mode()
            .unwrap()
            .contains(MaterialRenderFlags::CONSTANT_COLOR));
        let layers = materials[0].layers().unwrap();
        assert_eq!(layers[0].version(), version);
        assert_eq!(layers[0].filter_mode().unwrap(), 1);
        assert_eq!(layers[0].texture_id().unwrap(), 2);
        assert_eq!(layers[0].alpha().unwrap(), 0.5);
        assert_eq!(
            layers[0].shading_flags().unwrap(),
            LayerShadingFlags::default()
        );
        assert_eq!(
            Layer::from_bytes(version, layers[0].as_bytes()).unwrap(),
            layers[0]
        );
    }
}

#[test]
fn local_material_layers_are_bounded_when_available() {
    let Ok(directory) = std::env::var("WC3_MDX_FIXTURES") else {
        return;
    };
    let mut pending = vec![std::path::PathBuf::from(directory)];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "mdx") {
                let bytes = std::fs::read(&path).unwrap();
                let model = Model::from_bytes(&bytes).unwrap();
                for material in model.materials().unwrap() {
                    for layer in material.layers().unwrap() {
                        layer.filter_mode().unwrap();
                        layer.alpha().unwrap();
                        layer.texture_slots().unwrap();
                        layer.tracks().unwrap();
                    }
                }
            }
        }
    }
}

#[test]
fn builds_material_with_reforged_layer() {
    let mut layer = Layer::new(1100);
    layer.set_texture_id(4).unwrap();
    layer.set_shader_type_id(2).unwrap();
    layer.set_fresnel_color([0.1, 0.2, 0.3]).unwrap();
    let mut material = Material::new(1100);
    material.set_layers(&[layer]).unwrap();
    let parsed = Material::from_bytes(1100, material.as_bytes()).unwrap();
    let layers = parsed.layers().unwrap();
    assert_eq!(layers[0].texture_id().unwrap(), 4);
    assert_eq!(layers[0].shader_type_id().unwrap(), 2);
    assert_eq!(layers[0].fresnel_color().unwrap(), [0.1, 0.2, 0.3]);
    assert!(layers[0].texture_slots().unwrap().is_empty());
}

#[test]
fn shader_path_round_trip_in_legacy_reforged_material() {
    let mut material = Material::new(1000);
    material.set_shader("Shaders\\Unit.shader").unwrap();
    assert_eq!(
        Material::from_bytes(1000, material.as_bytes())
            .unwrap()
            .shader()
            .unwrap()
            .as_deref(),
        Some("Shaders\\Unit.shader")
    );
    assert!(Material::new(1800).set_shader("unused").is_err());
}

#[test]
fn reforged_layer_texture_slot_and_tracks_round_trip() {
    let mut layer = Layer::new(1800);
    let mut texture_key = Keyframe {
        frame: 100,
        value: vec![0.0],
        in_tangent: None,
        out_tangent: None,
    };
    texture_key.set_integer_value(17);
    let texture_track = AnimationTrack {
        tag: *b"KMTF",
        interpolation: 1,
        global_sequence_id: u32::MAX,
        keyframes: vec![texture_key],
    };
    layer
        .set_texture_slots(&[LayerTextureSlot {
            texture_id: 3,
            texture_type: 2,
            track: Some(texture_track.clone()),
        }])
        .unwrap();
    let alpha_track = AnimationTrack {
        tag: *b"KMTA",
        interpolation: 1,
        global_sequence_id: u32::MAX,
        keyframes: vec![Keyframe {
            frame: 100,
            value: vec![0.5],
            in_tangent: None,
            out_tangent: None,
        }],
    };
    layer
        .set_tracks(std::slice::from_ref(&alpha_track))
        .unwrap();
    let parsed = Layer::from_bytes(1800, layer.as_bytes()).unwrap();
    assert_eq!(
        parsed.texture_slots().unwrap()[0]
            .track
            .as_ref()
            .unwrap()
            .keyframes[0]
            .integer_value(),
        Some(17)
    );
    assert_eq!(parsed.tracks().unwrap(), vec![alpha_track]);
}

#[test]
fn unlit_layer_flag_round_trip() {
    let mut layer = Layer::new(1800);
    let mut flags = layer.shading_flags().unwrap();
    flags.set(LayerShadingFlags::UNLIT, true);
    layer.set_shading_flags(flags).unwrap();
    assert!(Layer::from_bytes(1800, layer.as_bytes())
        .unwrap()
        .shading_flags()
        .unwrap()
        .contains(LayerShadingFlags::UNLIT));
}

#[test]
fn version_specific_fields_do_not_write_into_legacy_tracks() {
    let mut layer = Layer::new(800);
    let before = layer.as_bytes().to_vec();
    assert!(layer.set_emissive_gain(0.5).is_err());
    assert!(layer.set_fresnel_opacity(0.5).is_err());
    assert!(layer.set_shader_type_id(1).is_err());
    assert_eq!(layer.as_bytes(), before);
}
