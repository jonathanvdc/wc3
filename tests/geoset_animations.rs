use wc3_mdx::{GeosetAnimation, Model};

#[test]
fn geoset_animation_fields_round_trip() {
    let mut animation = GeosetAnimation::new(2);
    animation.set_alpha(0.5);
    animation.set_flags(7);
    animation.set_color([0.1, 0.2, 0.3]);
    let mut model = Model::new(1800);
    model.set_geoset_animations(&[animation]).unwrap();
    let parsed = Model::from_bytes(&model.to_bytes().unwrap()).unwrap();
    let actual = &parsed.geoset_animations().unwrap()[0];
    assert_eq!(actual.geoset_id(), 2);
    assert_eq!(actual.alpha(), 0.5);
    assert_eq!(actual.flags(), 7);
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
                let mut model = Model::from_bytes(&bytes).unwrap();
                if model.chunk(*b"GEOA").is_some() {
                    let records = model.geoset_animations().unwrap();
                    model.set_geoset_animations(&records).unwrap();
                    assert_eq!(model.to_bytes().unwrap(), bytes);
                }
            }
        }
    }
}
