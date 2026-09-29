use wc3::model::animation::ParticleVisibility;
use wc3::model::animation::{AnimationTrack, ValueKeyframe};
use wc3::model::emitters::ParticleEmitter;
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::scene::Node;
use wc3::model::Model;

#[test]
fn classic_particle_emitter_round_trip() {
    let mut emitter = ParticleEmitter::new(Node::new("Smoke", 4).unwrap(), "smoke.mdl").unwrap();
    emitter.emission_rate = 10.0;
    emitter.gravity = -9.8;
    emitter.longitude = 0.2;
    emitter.latitude = 0.4;
    emitter.life_span = 2.0;
    emitter.initial_velocity = 5.0;
    let track = AnimationTrack::<ParticleVisibility>::linear(
        vec![ValueKeyframe {
            frame: 100,
            value: 1.0,
        }],
        None,
    )
    .unwrap()
    .into();
    emitter.tracks = (std::slice::from_ref(&track)).to_vec();
    let mut model = Model::<wc3::model::V800>::new();
    model.set_particle_emitters(&[emitter]);
    let parsed = Model::<wc3::model::V800>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    let emitter = &parsed.particle_emitters()[0];
    assert_eq!(emitter.node.name.text(), "Smoke");
    assert_eq!(emitter.path.text(), "smoke.mdl");
    assert_eq!(emitter.emission_rate, 10.0);
    assert_eq!(emitter.gravity, -9.8);
    assert_eq!(emitter.longitude, 0.2);
    assert_eq!(emitter.latitude, 0.4);
    assert_eq!(emitter.life_span, 2.0);
    assert_eq!(emitter.initial_velocity, 5.0);
    assert_eq!(emitter.tracks.as_slice(), &[track]);
}
