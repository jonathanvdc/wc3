use wc3::model::emitters::{Particle2Frames, ParticleEmitter2};
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::scene::Node;
use wc3::model::Model;

#[test]
fn particle_emitter2_fields_round_trip() {
    let mut emitter = ParticleEmitter2::new(Node::new("Flame", 1).unwrap());
    emitter.speed = 5.0;
    emitter.emission_rate = 20.0;
    emitter.segment_colors = [[1.0, 0.0, 0.0], [0.5, 0.5, 0.0], [0.0, 0.0, 0.0]];
    emitter.alpha = [255, 128, 0];
    emitter.particle_scaling = [1.0, 2.0, 3.0];
    emitter.uv_animations[0] = [0, 4, 2];
    emitter.texture_id = 7;
    emitter.frames = Particle2Frames::Both;
    emitter.set_squirt_enabled(true);
    assert_eq!(emitter.frames, Particle2Frames::Both);
    assert!(emitter.squirt_enabled());
    let mut model = Model::<wc3::model::V1800>::new();
    model.set_particle_emitters2(std::slice::from_ref(&emitter));
    let parsed = Model::<wc3::model::V1800>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    assert_eq!(parsed.particle_emitters2()[0], emitter);
}

#[test]
fn local_particle_emitter2_round_trip_when_available() {
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
                let mut model = Model::<wc3::model::V1800>::decode_mdx(&bytes).unwrap();
                if model.chunk(*b"PRE2").is_some() {
                    let emitters = model.particle_emitters2();
                    model.set_particle_emitters2(&emitters);
                    assert_eq!(model.encode_mdx().unwrap(), bytes, "{}", path.display());
                }
            }
        }
    }
}
