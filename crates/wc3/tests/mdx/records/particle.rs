use wc3::model::animation::{Animatable, Track};

use wc3::model::animation::ValueKeyframe;
use wc3::model::emitters::ParticleEmitter;
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::scene::Node;
use wc3::model::Model;

#[test]
fn classic_particle_emitter_round_trip() {
    let mut emitter = ParticleEmitter::new(Node::new("Smoke", 4).unwrap(), "smoke.mdl").unwrap();
    emitter.emission_rate = Animatable::Static(10.0);
    emitter.gravity = Animatable::Static(-9.8);
    emitter.longitude = Animatable::Static(0.2);
    emitter.latitude = Animatable::Static(0.4);
    emitter.life_span = Animatable::Static(2.0);
    emitter.initial_velocity = Animatable::Static(5.0);
    let track = Track::<f32>::linear(
        vec![ValueKeyframe {
            frame: 100,
            value: 1.0,
        }],
        None,
    )
    .unwrap();
    emitter.visibility = Some(track.clone());
    let mut model = Model::<wc3::model::V800>::new();
    model.set_particle_emitters(&[emitter]);
    let parsed = Model::<wc3::model::V800>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    let emitter = &parsed.particle_emitters()[0];
    assert_eq!(emitter.node.name.text(), "Smoke");
    assert_eq!(emitter.path.text(), "smoke.mdl");
    assert_eq!(emitter.emission_rate, Animatable::Static(10.0));
    assert_eq!(emitter.gravity, Animatable::Static(-9.8));
    assert_eq!(emitter.longitude, Animatable::Static(0.2));
    assert_eq!(emitter.latitude, Animatable::Static(0.4));
    assert_eq!(emitter.life_span, Animatable::Static(2.0));
    assert_eq!(emitter.initial_velocity, Animatable::Static(5.0));
    assert_eq!(emitter.visibility.as_ref(), Some(&track));
}
