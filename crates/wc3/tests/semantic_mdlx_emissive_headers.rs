use wc3::model::conversion::{ExportOptions, ImportOptions};
use wc3::model::formats::mdlx::animation::{Animatable, Sequence, Track, ValueKeyframe};
use wc3::model::formats::mdlx::materials::{Layer, Material, Texture};
use wc3::model::formats::mdlx::{Document, V1000};
use wc3::model::io;

fn fixture(header: f32, key: Option<f32>) -> Document<'static, V1000> {
    let mut document = Document::new();
    document.set_textures([Texture::new("texture.blp").unwrap()]);
    document.set_sequences([Sequence::new("Stand", [0, 1000]).unwrap()]);
    let mut gain = Animatable::Static(header);
    if let Some(value) = key {
        gain.set_track(
            Track::step(
                [
                    ValueKeyframe { frame: 0, value },
                    ValueKeyframe { frame: 1000, value },
                ],
                None,
            )
            .unwrap(),
        );
    }
    let mut layer = Layer::new();
    layer.set_emissive_gain(gain);
    let mut material = Material::new();
    material.layers.push(layer);
    document.set_materials([material]);
    document
}

#[test]
fn unspecified_emissive_header_uses_unity_and_retains_authored_track() {
    for key in [None, Some(2.)] {
        let document = fixture(f32::NAN, key);
        let model = io::mdlx::import_owned(&document, &ImportOptions::default())
            .unwrap()
            .value;
        let shader = &model.materials().iter().next().unwrap().1.surface.steps[0].shader;
        let gain = shader.inputs().emissive_gain.unwrap();
        assert_eq!(model.values()[gain], 1.);
        assert_eq!(model.is_animated(gain), key.is_some());
        let output = io::mdlx::export::<V1000>(&model, &ExportOptions::default())
            .unwrap()
            .value;
        let material = output.materials().next().unwrap();
        let gain = material.layers[0].emissive_gain();
        assert_eq!(gain.value(), Some(&1.));
        assert_eq!(gain.track().is_some(), key.is_some());
        if let Some(value) = key {
            let track = gain.track().unwrap();
            assert!(track
                .step_keys()
                .unwrap()
                .iter()
                .all(|key| key.value == value));
        }
    }
}

#[test]
fn infinite_emissive_headers_and_nonfinite_active_keys_are_not_unspecified() {
    assert!(
        io::mdlx::import_owned(&fixture(f32::INFINITY, None), &ImportOptions::default()).is_err()
    );
    for key in [f32::NAN, f32::INFINITY] {
        assert!(
            io::mdlx::import_owned(&fixture(f32::NAN, Some(key)), &ImportOptions::default())
                .is_err()
        );
    }
}
