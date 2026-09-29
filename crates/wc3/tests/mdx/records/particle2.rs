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
