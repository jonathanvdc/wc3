use wc3_mdx::animation::GeosetColor;
use wc3_mdx::animation::{AnimationTrack, GeosetAnimation, GeosetAnimationFlags, ValueKeyframe};
use wc3_mdx::io::{Decodable, Encodable};
use wc3_mdx::Model;

#[test]
fn geoset_animation_fields_round_trip() {
    let mut animation = GeosetAnimation::new(2);
    animation.set_alpha(0.5);
    animation.set_raw_flags(7);
    animation.set_color([0.1, 0.2, 0.3]);
    let mut model = Model::<wc3_mdx::V1800>::new();
    model.set_geoset_animations(&[animation]);
    let parsed = Model::<wc3_mdx::V1800>::decode(&model.encode().unwrap(), 800).unwrap();
    let actual = &parsed.geoset_animations()[0];
    assert_eq!(actual.geoset_id(), 2);
    assert_eq!(actual.alpha(), 0.5);
    assert!(actual.flags().contains(GeosetAnimationFlags::DROP_SHADOW));
    assert!(actual.flags().contains(GeosetAnimationFlags::COLOR));
    assert_eq!(actual.raw_flags(), 7);
    assert_eq!(actual.color(), [0.1, 0.2, 0.3]);
}

#[test]
fn local_geoset_animations_round_trip_when_available() {
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
                let mut model = Model::<wc3_mdx::V1800>::decode(&bytes, 800).unwrap();
                if model.chunk(*b"GEOA").is_some() {
                    let records = model.geoset_animations();
                    model.set_geoset_animations(&records);
                    assert_eq!(model.encode().unwrap(), bytes);
                }
            }
        }
    }
}

#[test]
fn geoset_animation_color_track_round_trip() {
    let mut animation = GeosetAnimation::new(1);
    let track = AnimationTrack::<GeosetColor>::linear(
        vec![ValueKeyframe {
            frame: 25,
            value: [0.2, 0.4, 0.8],
        }],
        None,
    )
    .unwrap()
    .into();
    animation.set_tracks(std::slice::from_ref(&track));
    let parsed = GeosetAnimation::decode(&animation.encode().unwrap(), 800).unwrap();
    assert_eq!(parsed.tracks(), vec![track]);
}
