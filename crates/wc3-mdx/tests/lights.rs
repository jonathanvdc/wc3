use wc3_mdx::animation::{AnimationTrack, ValueKeyframe};
use wc3_mdx::animation::{LightColor, LightVisibility};
use wc3_mdx::io::{Readable, Writable};
use wc3_mdx::scene::{Light, Node};
use wc3_mdx::Model;

#[test]
fn light_fields_round_trip() {
    let mut light = Light::<wc3_mdx::V1200>::new(Node::new("Torch", 2).unwrap(), 1);
    light.set_attenuation_start(100.0);
    light.set_attenuation_end(500.0);
    light.set_color([1.0, 0.5, 0.25]);
    light.set_intensity(2.0);
    light.set_ambient_color([0.1, 0.2, 0.3]);
    light.set_ambient_intensity(0.5);
    let mut model = Model::<wc3_mdx::V1200>::new();
    model.set_lights(&[light]);
    let parsed = Model::<wc3_mdx::V1200>::decode(&model.encode().unwrap()).unwrap();
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
    let mut light = Light::<wc3_mdx::V800>::new(Node::new("Lamp", 3).unwrap(), 0);
    let track = AnimationTrack::<LightColor>::linear(
        vec![ValueKeyframe {
            frame: 250,
            value: [1.0, 0.5, 0.25],
        }],
        None,
    )
    .unwrap()
    .into();
    light.set_tracks(std::slice::from_ref(&track));
    let parsed = Light::<wc3_mdx::V800>::decode(&light.encode().unwrap()).unwrap();
    assert_eq!(parsed.tracks(), &[track]);
}

#[test]
fn extended_light_fields_and_tracks_round_trip() {
    let mut light = Light::<wc3_mdx::V1800>::new(Node::new("Glow", 4).unwrap(), 0);
    let words = [1, 2, 3, 4, 5, 6, 7];
    light.set_extended_words(words);
    let track = AnimationTrack::<LightVisibility>::linear(
        vec![ValueKeyframe {
            frame: 10,
            value: 1.0,
        }],
        None,
    )
    .unwrap()
    .into();
    light.set_tracks(std::slice::from_ref(&track));
    let parsed = Light::<wc3_mdx::V1800>::decode(&light.encode().unwrap()).unwrap();
    assert_eq!(parsed.extended_words(), words);
    assert_eq!(parsed.tracks(), &[track]);
}
