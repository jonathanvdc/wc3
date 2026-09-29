use super::*;
use wc3::model::animation::Animatable;

#[test]
fn constructors_use_documented_defaults_without_changing_decoded_values() {
    let light = Light::<V800>::new(Node::new("Light", 0).unwrap(), 0);
    assert_eq!(light.color, Animatable::Static([1.0; 3]));
    assert_eq!(light.ambient_color, Animatable::Static([1.0; 3]));
    assert_eq!(
        light.falloff(),
        LightFalloff {
            quadratic: Animatable::Static(0.0005),
            linear: Animatable::Static(0.0),
            damping: Animatable::Static(0.00001)
        }
    );
    let mut emitter = PopcornEmitter::new(
        Node::new("Effect", 1).unwrap(),
        "effect.pkfx",
        "Always=on\r\nDeath=off",
    )
    .unwrap();
    assert_eq!(emitter.life_span, Animatable::Static(1.0));
    assert_eq!(emitter.emission_rate, Animatable::Static(1.0));
    assert_eq!(emitter.speed, Animatable::Static(1.0));
    assert_eq!(emitter.alpha, Animatable::Static(1.0));
    assert_eq!(emitter.color, Animatable::Static([1.0; 3]));
    emitter.life_span = Animatable::Static(0.0);
    emitter.color = Animatable::Static([0.0; 3]);
    let decoded = PopcornEmitter::decode_mdx(&emitter.encode_mdx().unwrap()).unwrap();
    assert_eq!(decoded.life_span, Animatable::Static(0.0));
    assert_eq!(decoded.color, Animatable::Static([0.0; 3]));
    assert_eq!(decoded.visibility_guide.text(), "Always=on\r\nDeath=off");
    assert_eq!(
        ParticleEmitter2::new(Node::new("p", 0).unwrap()).priority_plane,
        0u32
    );
}
