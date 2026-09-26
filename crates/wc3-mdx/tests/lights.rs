use wc3_mdx::animation::{AnimationTrack, Keyframe, TrackTag};
use wc3_mdx::io::{Decodable, Encodable};
use wc3_mdx::scene::{Light, Node};
use wc3_mdx::Model;

#[test]
fn light_fields_round_trip() {
    let mut light = Light::new(Node::new("Torch", 2).unwrap(), 1);
    light.set_attenuation_start(100.0);
    light.set_attenuation_end(500.0);
    light.set_color([1.0, 0.5, 0.25]);
    light.set_intensity(2.0);
    light.set_ambient_color([0.1, 0.2, 0.3]);
    light.set_ambient_intensity(0.5);
    let mut model = Model::new(1200);
    model.set_lights(&[light]);
    let parsed = Model::decode(&model.encode().unwrap(), 800).unwrap();
    let light = &parsed.lights()[0];
    assert_eq!(light.node().name(), "Torch");
    assert_eq!(light.light_type(), 1);
    assert_eq!(light.attenuation_start(), 100.0);
    assert_eq!(light.attenuation_end(), 500.0);
    assert_eq!(light.color(), [1.0, 0.5, 0.25]);
    assert_eq!(light.intensity(), 2.0);
    assert_eq!(light.ambient_color(), [0.1, 0.2, 0.3]);
    assert_eq!(light.ambient_intensity(), 0.5);
}

#[test]
fn light_color_track_round_trip() {
    let mut light = Light::new(Node::new("Lamp", 3).unwrap(), 0);
    let track = AnimationTrack {
        tag: TrackTag::LightColor,
        interpolation: 1,
        global_sequence_id: u32::MAX,
        keyframes: vec![Keyframe {
            frame: 250,
            value: vec![1.0, 0.5, 0.25],
            in_tangent: None,
            out_tangent: None,
        }],
    };
    light.set_tracks(std::slice::from_ref(&track)).unwrap();
    let parsed = Light::decode(&light.encode().unwrap(), 800).unwrap();
    assert_eq!(parsed.tracks(), &[track]);
}

#[test]
fn extended_light_fields_and_tracks_round_trip() {
    let mut light = Light::new_for_version(Node::new("Glow", 4).unwrap(), 0, 1800);
    let words = [1, 2, 3, 4, 5, 6, 7];
    light.set_extended_words(words).unwrap();
    let track = AnimationTrack {
        tag: TrackTag::LightVisibility,
        interpolation: 1,
        global_sequence_id: u32::MAX,
        keyframes: vec![Keyframe {
            frame: 10,
            value: vec![1.0],
            in_tangent: None,
            out_tangent: None,
        }],
    };
    light.set_tracks(std::slice::from_ref(&track)).unwrap();
    let parsed = Light::decode(&light.encode().unwrap(), 800).unwrap();
    assert_eq!(parsed.extended_words(), Some(words));
    assert_eq!(parsed.tracks(), &[track]);
}
