use wc3_mdx::animation::{AnimationTrack, Keyframe};
use wc3_mdx::io::{Decodable, Encodable};
use wc3_mdx::materials::{
    Layer, LayerShadingFlags, LayerTextureSlot, Material, MaterialRenderFlags,
};
use wc3_mdx::Model;

fn sample_material(version: u32) -> Material {
    let mut layer = Layer::new(version);
    layer.set_filter_mode(1);
    layer.set_texture_id(2);
    layer.set_alpha(0.5);
    let mut material = Material::new(version);
    material.set_layers(&[layer]).unwrap();
    Material::decode(&material.encode().unwrap(), version).unwrap()
}

#[test]
fn material_layers_round_trip_across_layouts() {
    for version in [800, 900, 1000, 1100, 1200, 1800] {
        let mut material = sample_material(version);
        material.set_priority_plane(3);
        material.set_raw_render_mode(7);
        let mut model = Model::new(version);
        model.set_materials(&[material]).unwrap();
        let decoded = Model::decode(&model.encode().unwrap(), 800).unwrap();
        let materials = decoded.materials();
        assert_eq!(materials[0].version(), version);
        assert_eq!(materials[0].priority_plane(), 3);
        assert_eq!(materials[0].raw_render_mode(), 7);
        assert!(materials[0]
            .render_mode()
            .contains(MaterialRenderFlags::CONSTANT_COLOR));
        let layers = materials[0].layers();
        assert_eq!(layers[0].version(), version);
        assert_eq!(layers[0].filter_mode(), 1);
        assert_eq!(layers[0].texture_id(), 2);
        assert_eq!(layers[0].alpha(), 0.5);
        assert_eq!(layers[0].shading_flags(), LayerShadingFlags::default());
        assert_eq!(
            Layer::decode(&layers[0].encode().unwrap(), version).unwrap(),
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
                let model = Model::decode(&bytes, 800).unwrap();
                for material in model.materials() {
                    for layer in material.layers() {
                        layer.filter_mode();
                        layer.alpha();
                        layer.texture_slots();
                        layer.tracks();
                    }
                }
            }
        }
    }
}

#[test]
fn builds_material_with_reforged_layer() {
    let mut layer = Layer::new(1100);
    layer.set_texture_id(4);
    layer.set_shader_type_id(2).unwrap();
    layer.set_fresnel_color([0.1, 0.2, 0.3]).unwrap();
    let mut material = Material::new(1100);
    material.set_layers(&[layer]).unwrap();
    let parsed = Material::decode(&material.encode().unwrap(), 1100).unwrap();
    let layers = parsed.layers();
    assert_eq!(layers[0].texture_id(), 4);
    assert_eq!(layers[0].shader_type_id(), Some(2));
    assert_eq!(layers[0].fresnel_color(), Some([0.1, 0.2, 0.3]));
    assert!(layers[0].texture_slots().is_empty());
}

#[test]
fn shader_path_round_trip_in_legacy_reforged_material() {
    let mut material = Material::new(1000);
    material.set_shader("Shaders\\Unit.shader").unwrap();
    assert_eq!(
        Material::decode(&material.encode().unwrap(), 1000)
            .unwrap()
            .shader()
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
    let parsed = Layer::decode(&layer.encode().unwrap(), 1800).unwrap();
    assert_eq!(
        parsed.texture_slots()[0].track.as_ref().unwrap().keyframes[0].integer_value(),
        Some(17)
    );
    assert_eq!(parsed.tracks(), vec![alpha_track]);
}

#[test]
fn unlit_layer_flag_round_trip() {
    let mut layer = Layer::new(1800);
    let mut flags = layer.shading_flags();
    flags.set(LayerShadingFlags::UNLIT, true);
    layer.set_shading_flags(flags);
    assert!(Layer::decode(&layer.encode().unwrap(), 1800)
        .unwrap()
        .shading_flags()
        .contains(LayerShadingFlags::UNLIT));
}

#[test]
fn version_specific_fields_do_not_write_into_legacy_tracks() {
    let mut layer = Layer::new(800);
    let before = layer.encode().unwrap();
    assert!(layer.set_emissive_gain(0.5).is_err());
    assert!(layer.set_fresnel_opacity(0.5).is_err());
    assert!(layer.set_shader_type_id(1).is_err());
    assert_eq!(layer.encode().unwrap(), before);
}

#[test]
fn preserves_shader_padding_and_float_bits() {
    let mut material = Material::new(1000);
    material.set_layers(&[Layer::new(1000)]).unwrap();
    let mut bytes = material.encode().unwrap();
    bytes[20] = 0xaf;
    let layer_start = 100;
    bytes[layer_start + 24..layer_start + 28].copy_from_slice(&0x7fa1_2345u32.to_le_bytes());
    let parsed = Material::decode(&bytes, 1000).unwrap();
    assert_eq!(parsed.encode().unwrap(), bytes);
    assert_eq!(parsed.layers()[0].alpha().to_bits(), 0x7fa1_2345);
}
