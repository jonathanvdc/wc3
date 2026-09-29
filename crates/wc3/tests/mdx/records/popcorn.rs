use wc3::model::animation::Animatable;
use wc3::model::emitters::PopcornEmitter;
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::scene::Node;
use wc3::model::Model;

#[test]
fn popcorn_fixed_fields_round_trip() {
    let mut emitter =
        PopcornEmitter::new(Node::new("Spark", 3).unwrap(), "spark.mdx", "Stand").unwrap();
    emitter.life_span = Animatable::Static(1.0);
    emitter.emission_rate = Animatable::Static(20.0);
    emitter.speed = Animatable::Static(3.0);
    emitter.color = Animatable::Static([1.0, 0.5, 0.25]);
    emitter.alpha = Animatable::Static(0.75);
    emitter.replaceable_id = 1;
    let mut model = Model::<wc3::model::V1800>::new();
    model.set_popcorn_emitters(&[emitter]);
    let parsed = Model::<wc3::model::V1800>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    let emitter = &parsed.popcorn_emitters()[0];
    assert_eq!(emitter.node.name.text(), "Spark");
    assert_eq!(emitter.path.text(), "spark.mdx");
    assert_eq!(emitter.visibility_guide.text(), "Stand");
    assert_eq!(emitter.life_span, Animatable::Static(1.0));
    assert_eq!(emitter.emission_rate, Animatable::Static(20.0));
    assert_eq!(emitter.speed, Animatable::Static(3.0));
    assert_eq!(emitter.color, Animatable::Static([1.0, 0.5, 0.25]));
    assert_eq!(emitter.alpha, Animatable::Static(0.75));
    assert_eq!(emitter.replaceable_id, 1);
}
