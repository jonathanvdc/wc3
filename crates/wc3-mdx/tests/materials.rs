use wc3_mdx::animation::{AnimationTrack, ValueKeyframe};
use wc3_mdx::animation::{LayerAlpha, LayerTextureId};
use wc3_mdx::io::{Readable, Writable};
use wc3_mdx::materials::{
    Layer, LayerShadingFlags, LayerTextureSlot, Material, MaterialRenderFlags,
};
use wc3_mdx::{AnyVersionModel, Model, ModelVersion, V1000, V1100, V1200, V1800, V800, V900};

fn sample_material<V: ModelVersion>() -> Material<V> {
    let mut layer = Layer::<V>::new();
    layer.set_filter_mode(1);
    layer.set_texture_id(2);
    layer.set_alpha(0.5);
    let mut material = Material::<V>::new();
    material.set_layers(&[layer]);
    Material::<V>::decode(&material.encode().unwrap()).unwrap()
}

#[test]
fn material_layers_round_trip_across_layouts() {
    check_version::<V800>();
    check_version::<V900>();
    check_version::<V1000>();
    check_version::<V1100>();
    check_version::<V1200>();
    check_version::<V1800>();
}

fn check_version<V: ModelVersion>() {
    let version = V::NUMBER;
    let mut material = sample_material::<V>();
    material.set_priority_plane(3);
    material.set_raw_render_mode(7);
    let mut model = Model::<V>::new();
    model.set_materials(&[material]);
    let decoded = Model::<V>::decode(&model.encode().unwrap()).unwrap();
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
        Layer::<V>::decode(&layers[0].encode().unwrap()).unwrap(),
        layers[0]
    );
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
                let model = AnyVersionModel::decode(&bytes, 800).unwrap();
                fn check<V: ModelVersion>(model: Model<V>) {
                    for material in model.materials() {
                        for layer in material.layers() {
                            layer.filter_mode();
                            layer.alpha();
                            layer.texture_slots();
                            layer.tracks();
                        }
                    }
                }
                match model {
                    AnyVersionModel::V800(model) => check(model),
                    AnyVersionModel::V900(model) => check(model),
                    AnyVersionModel::V1000(model) => check(model),
                    AnyVersionModel::V1100(model) => check(model),
                    AnyVersionModel::V1200(model) => check(model),
                    AnyVersionModel::V1800(model) => check(model),
                }
            }
        }
    }
}

#[test]
fn builds_material_with_reforged_layer() {
    let mut layer = Layer::<V1100>::new();
    layer.set_texture_id(4);
    layer.set_shader_type_id(2).unwrap();
    layer.set_fresnel_color([0.1, 0.2, 0.3]).unwrap();
    let mut material = Material::<V1100>::new();
    material.set_layers(&[layer]);
    let parsed = Material::<V1100>::decode(&material.encode().unwrap()).unwrap();
    let layers = parsed.layers();
    assert_eq!(layers[0].texture_id(), 4);
    assert_eq!(layers[0].shader_type_id(), Some(2));
    assert_eq!(layers[0].fresnel_color(), Some([0.1, 0.2, 0.3]));
    assert!(layers[0].texture_slots().is_empty());
}

#[test]
fn shader_path_round_trip_in_legacy_reforged_material() {
    let mut material = Material::<V1000>::new();
    material.set_shader("Shaders\\Unit.shader").unwrap();
    assert_eq!(
        Material::<V1000>::decode(&material.encode().unwrap())
            .unwrap()
            .shader()
            .as_deref(),
        Some("Shaders\\Unit.shader")
    );
    assert!(Material::<V1800>::new().set_shader("unused").is_err());
}

#[test]
fn reforged_layer_texture_slot_and_tracks_round_trip() {
    let mut layer = Layer::<V1800>::new();
    let texture_key = ValueKeyframe {
        frame: 100,
        value: 17u32,
    };
    let texture_track = AnimationTrack::<LayerTextureId>::linear(vec![texture_key], None).unwrap();
    layer
        .set_texture_slots(&[LayerTextureSlot {
            texture_id: 3,
            texture_type: 2,
            track: Some(texture_track.clone()),
        }])
        .unwrap();
    let alpha_track = AnimationTrack::<LayerAlpha>::linear(
        vec![ValueKeyframe {
            frame: 100,
            value: 0.5,
        }],
        None,
    )
    .unwrap()
    .into();
    layer.set_tracks(std::slice::from_ref(&alpha_track));
    let parsed = Layer::<V1800>::decode(&layer.encode().unwrap()).unwrap();
    assert_eq!(
        parsed.texture_slots()[0]
            .track
            .as_ref()
            .unwrap()
            .linear_keys()
            .unwrap()[0]
            .value,
        17
    );
    assert_eq!(parsed.tracks(), vec![alpha_track]);
}

#[test]
fn unlit_layer_flag_round_trip() {
    let mut layer = Layer::<V1800>::new();
    let mut flags = layer.shading_flags();
    flags.set(LayerShadingFlags::UNLIT, true);
    layer.set_shading_flags(flags);
    assert!(Layer::<V1800>::decode(&layer.encode().unwrap())
        .unwrap()
        .shading_flags()
        .contains(LayerShadingFlags::UNLIT));
}

#[test]
fn version_specific_fields_do_not_write_into_legacy_tracks() {
    let mut layer = Layer::<V800>::new();
    let before = layer.encode().unwrap();
    assert!(layer.set_emissive_gain(0.5).is_err());
    assert!(layer.set_fresnel_opacity(0.5).is_err());
    assert!(layer.set_shader_type_id(1).is_err());
    assert_eq!(layer.encode().unwrap(), before);
}

#[test]
fn preserves_shader_padding_and_float_bits() {
    let mut material = Material::<V1000>::new();
    material.set_layers(&[Layer::<V1000>::new()]);
    let mut bytes = material.encode().unwrap();
    bytes[20] = 0xaf;
    let layer_start = 100;
    bytes[layer_start + 24..layer_start + 28].copy_from_slice(&0x7fa1_2345u32.to_le_bytes());
    let parsed = Material::<V1000>::decode(&bytes).unwrap();
    assert_eq!(parsed.encode().unwrap(), bytes);
    assert_eq!(parsed.layers()[0].alpha().to_bits(), 0x7fa1_2345);
}
