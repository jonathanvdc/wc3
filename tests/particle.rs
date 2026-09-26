use wc3_mdx::{Decodable, Encodable};
use wc3_mdx::{AnimationTrack, Keyframe, Model, Node, ParticleEmitter};

#[test]
fn classic_particle_emitter_round_trip() {
    let mut emitter = ParticleEmitter::new(Node::new("Smoke", 4).unwrap(), "smoke.mdl").unwrap();
    emitter.set_emission_rate(10.0);
    emitter.set_gravity(-9.8);
    emitter.set_longitude(0.2);
    emitter.set_latitude(0.4);
    emitter.set_life_span(2.0);
    emitter.set_initial_velocity(5.0);
    let track = AnimationTrack {
        tag: *b"KPEV",
        interpolation: 1,
        global_sequence_id: u32::MAX,
        keyframes: vec![Keyframe {
            frame: 100,
            value: vec![1.0],
            in_tangent: None,
            out_tangent: None,
        }],
    };
    emitter.set_tracks(std::slice::from_ref(&track)).unwrap();
    let mut model = Model::new(800);
    model.set_particle_emitters(&[emitter]);
    let parsed = Model::decode(&model.encode().unwrap(), 800).unwrap();
    let emitter = &parsed.particle_emitters()[0];
    assert_eq!(emitter.node().name(), "Smoke");
    assert_eq!(emitter.path(), "smoke.mdl");
    assert_eq!(emitter.emission_rate(), 10.0);
    assert_eq!(emitter.gravity(), -9.8);
    assert_eq!(emitter.longitude(), 0.2);
    assert_eq!(emitter.latitude(), 0.4);
    assert_eq!(emitter.life_span(), 2.0);
    assert_eq!(emitter.initial_velocity(), 5.0);
    assert_eq!(emitter.tracks(), &[track]);
}
