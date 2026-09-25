use wc3_mdx::Record;
use wc3_mdx::{Model, Node, Particle2Frames, ParticleEmitter2};

#[test]
fn particle_emitter2_fields_round_trip() {
    let mut emitter = ParticleEmitter2::new(Node::new("Flame", 1).unwrap());
    let mut fields = emitter.fields();
    fields.speed = 5.0;
    fields.emission_rate = 20.0;
    fields.segment_colors = [[1.0, 0.0, 0.0], [0.5, 0.5, 0.0], [0.0, 0.0, 0.0]];
    fields.alpha = [255, 128, 0];
    fields.particle_scaling = [1.0, 2.0, 3.0];
    fields.uv_animations[0] = [0, 4, 2];
    fields.texture_id = 7;
    fields.set_frames(Particle2Frames::Both);
    fields.set_squirt_enabled(true);
    assert_eq!(fields.frames(), Particle2Frames::Both);
    assert!(fields.squirt_enabled());
    emitter.set_fields(&fields);
    let mut model = Model::new(1800);
    model.set_particle_emitters2(&[emitter]);
    let parsed = Model::decode(&model.encode().unwrap(), 800).unwrap();
    assert_eq!(parsed.particle_emitters2().unwrap()[0].fields(), fields);
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
                let mut model = Model::decode(&bytes, 800).unwrap();
                if model.chunk(*b"PRE2").is_some() {
                    let mut emitters = model.particle_emitters2().unwrap();
                    for emitter in &mut emitters {
                        let fields = emitter.fields();
                        emitter.set_fields(&fields);
                        emitter.tracks();
                    }
                    model.set_particle_emitters2(&emitters);
                    assert_eq!(model.encode().unwrap(), bytes, "{}", path.display());
                }
            }
        }
    }
}
