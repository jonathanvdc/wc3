use wc3::model::animation::{AnimationTrack, ValueKeyframe};
use wc3::model::animation::{LightColor, LightVisibility};
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::scene::{Light, LightFalloff, LightShadowRange, Node};
use wc3::model::Model;

#[test]
fn light_fields_round_trip() {
    let mut light = Light::<wc3::model::V1200>::new(Node::new("Torch", 2).unwrap(), 1);
    light.attenuation_start = 100.0;
    light.attenuation_end = 500.0;
    light.color = [1.0, 0.5, 0.25];
    light.intensity = 2.0;
    light.ambient_color = [0.1, 0.2, 0.3];
    light.ambient_intensity = 0.5;
    let mut model = Model::<wc3::model::V1200>::new();
    model.set_lights(&[light]);
    let parsed = Model::<wc3::model::V1200>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    let light = &parsed.lights()[0];
    assert_eq!(light.node.name.text(), "Torch");
    assert_eq!(light.light_type, 1);
    assert_eq!(light.attenuation_start, 100.0);
    assert_eq!(light.attenuation_end, 500.0);
    assert_eq!(light.color, [1.0, 0.5, 0.25]);
    assert_eq!(light.intensity, 2.0);
    assert_eq!(light.ambient_color, [0.1, 0.2, 0.3]);
    assert_eq!(light.ambient_intensity, 0.5);
}

#[test]
fn light_color_track_round_trip() {
    let mut light = Light::<wc3::model::V800>::new(Node::new("Lamp", 3).unwrap(), 0);
    let track = AnimationTrack::<LightColor>::linear(
        vec![ValueKeyframe {
            frame: 250,
            value: [1.0, 0.5, 0.25],
        }],
        None,
    )
    .unwrap()
    .into();
    light.tracks = (std::slice::from_ref(&track)).to_vec();
    let parsed = Light::<wc3::model::V800>::decode_mdx(&light.encode_mdx().unwrap()).unwrap();
    assert_eq!(parsed.tracks.as_slice(), &[track]);
}

#[test]
fn extended_light_fields_and_tracks_round_trip() {
    let mut light = Light::<wc3::model::V1800>::new(Node::new("Glow", 4).unwrap(), 0);
    light.try_set_shadow_casting(true).unwrap();
    light.try_set_shadow_intensity(1.0).unwrap();
    light
        .try_set_shadow_casting_range(LightShadowRange {
            start: 2.0,
            end: 3.0,
        })
        .unwrap();
    light
        .try_set_falloff(LightFalloff {
            quadratic: 4.0,
            linear: 5.0,
            damping: 6.0,
        })
        .unwrap();
    let track = AnimationTrack::<LightVisibility>::linear(
        vec![ValueKeyframe {
            frame: 10,
            value: 1.0,
        }],
        None,
    )
    .unwrap()
    .into();
    light.tracks = (std::slice::from_ref(&track)).to_vec();
    let parsed = Light::<wc3::model::V1800>::decode_mdx(&light.encode_mdx().unwrap()).unwrap();
    assert!(parsed.try_shadow_casting().unwrap());
    assert_eq!(parsed.try_shadow_intensity().unwrap(), 1.0);
    assert_eq!(
        parsed.try_shadow_casting_range().unwrap(),
        LightShadowRange {
            start: 2.0,
            end: 3.0
        }
    );
    assert_eq!(
        parsed.falloff(),
        LightFalloff {
            quadratic: 4.0,
            linear: 5.0,
            damping: 6.0
        }
    );
    assert_eq!(parsed.tracks.as_slice(), &[track]);
}

#[test]
fn intermediate_light_layouts_follow_version_gates() {
    use wc3::model::{V1300, V1400, V1600};

    let base = Light::<wc3::model::V1200>::new(Node::new("Lamp", 1).unwrap(), 0)
        .encode_mdx()
        .unwrap();
    let v1300 = Light::<V1300>::new(Node::new("Lamp", 1).unwrap(), 0)
        .encode_mdx()
        .unwrap();
    let v1400 = Light::<V1400>::new(Node::new("Lamp", 1).unwrap(), 0)
        .encode_mdx()
        .unwrap();
    let v1600 = Light::<V1600>::new(Node::new("Lamp", 1).unwrap(), 0)
        .encode_mdx()
        .unwrap();
    assert_eq!(v1300.len(), base.len() + 12);
    assert_eq!(v1400.len(), v1300.len());
    assert_eq!(v1600.len(), v1300.len() + 12);
    assert_eq!(
        Light::<V1300>::decode_mdx(&v1300)
            .unwrap()
            .encode_mdx()
            .unwrap(),
        v1300
    );
    assert_eq!(
        Light::<V1400>::decode_mdx(&v1400)
            .unwrap()
            .encode_mdx()
            .unwrap(),
        v1400
    );
    assert_eq!(
        Light::<V1600>::decode_mdx(&v1600)
            .unwrap()
            .encode_mdx()
            .unwrap(),
        v1600
    );
}

#[test]
fn shadow_casting_and_falloff_round_trip() {
    use wc3::model::{V1300, V1600};

    let mut old = Light::<V1300>::new(Node::new("Old", 1).unwrap(), 0);
    assert_eq!(old.falloff(), LightFalloff::default());
    assert!(old
        .try_set_falloff(LightFalloff {
            quadratic: 1.0,
            linear: 2.0,
            damping: 3.0
        })
        .is_err());

    let mut light = Light::<V1600>::new(Node::new("New", 2).unwrap(), 0);
    assert_eq!(light.falloff(), LightFalloff::default());
    light.try_set_shadow_casting(true).unwrap();
    light
        .try_set_falloff(LightFalloff {
            quadratic: 1.0,
            linear: 2.0,
            damping: 3.0,
        })
        .unwrap();
    let decoded = Light::<V1600>::decode_mdx(&light.encode_mdx().unwrap()).unwrap();
    assert!(decoded.try_shadow_casting().unwrap());
    assert_eq!(
        decoded.falloff(),
        LightFalloff {
            quadratic: 1.0,
            linear: 2.0,
            damping: 3.0
        }
    );
}

#[test]
fn infallible_light_accessors_cover_supported_versions() {
    use wc3::model::{
        SupportsLightFalloff, SupportsLightShadowCasting, SupportsLightShadowIntensity, V1200,
        V1300, V1400, V1600, V1800,
    };

    fn check_intensity<V: SupportsLightShadowIntensity>() {
        let mut light = Light::<V>::new(Node::new("Lamp", 1).unwrap(), 0);
        light.set_shadow_intensity(0.5);
        let decoded = Light::<V>::decode_mdx(&light.encode_mdx().unwrap()).unwrap();
        assert_eq!(decoded.shadow_intensity(), 0.5);
    }
    fn check_casting<V: SupportsLightShadowCasting>() {
        let mut light = Light::<V>::new(Node::new("Lamp", 1).unwrap(), 0);
        let range = LightShadowRange {
            start: 10.0,
            end: 100.0,
        };
        light.set_shadow_casting(true);
        light.set_shadow_casting_range(range);
        let decoded = Light::<V>::decode_mdx(&light.encode_mdx().unwrap()).unwrap();
        assert!(decoded.shadow_casting());
        assert_eq!(decoded.shadow_casting_range(), range);
    }
    fn check_falloff<V: SupportsLightFalloff>() {
        let mut light = Light::<V>::new(Node::new("Lamp", 1).unwrap(), 0);
        let falloff = LightFalloff {
            quadratic: 0.1,
            linear: 0.2,
            damping: 0.3,
        };
        light.set_falloff(falloff);
        let decoded = Light::<V>::decode_mdx(&light.encode_mdx().unwrap()).unwrap();
        assert_eq!(decoded.falloff(), falloff);
    }
    check_intensity::<V1200>();
    check_intensity::<V1300>();
    check_intensity::<V1400>();
    check_intensity::<V1600>();
    check_intensity::<V1800>();
    check_casting::<V1300>();
    check_casting::<V1400>();
    check_casting::<V1600>();
    check_casting::<V1800>();
    check_falloff::<V1600>();
    check_falloff::<V1800>();
}
