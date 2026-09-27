use wc3_mdx::animation::ParticleVisibility;
use wc3_mdx::animation::{AnimationTrack, ValueKeyframe};
use wc3_mdx::emitters::ParticleEmitter;
use wc3_mdx::io::{Encodable, Readable};
use wc3_mdx::scene::Node;
use wc3_mdx::Model;

#[test]
fn classic_particle_emitter_round_trip() {
    let mut emitter = ParticleEmitter::new(Node::new("Smoke", 4).unwrap(), "smoke.mdl").unwrap();
    emitter.set_emission_rate(10.0);
    emitter.set_gravity(-9.8);
    emitter.set_longitude(0.2);
    emitter.set_latitude(0.4);
    emitter.set_life_span(2.0);
    emitter.set_initial_velocity(5.0);
    let track = AnimationTrack::<ParticleVisibility>::linear(
        vec![ValueKeyframe {
            frame: 100,
            value: 1.0,
        }],
        None,
    )
    .unwrap()
    .into();
    emitter.set_tracks(std::slice::from_ref(&track));
    let mut model = Model::<wc3_mdx::V800>::new();
    model.set_particle_emitters(&[emitter]);
    let parsed = Model::<wc3_mdx::V800>::decode(&model.encode().unwrap()).unwrap();
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
