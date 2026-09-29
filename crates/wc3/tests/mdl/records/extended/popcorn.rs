use super::*;
use wc3::model::animation::Animatable;

#[test]
fn popcorn_defaults_flags_and_literal_strings() {
    let record = PopcornEmitter::decode_mdl("ParticleEmitterPopcorn \"p\" { ObjectId 0, SortPrimsFarZ, Unshaded, Unfogged, PopcornScaling, Path \"a\\b.pkfx\", AnimVisibilityGuide \"Always=on,\nDeath=off\", }").unwrap();
    assert_eq!(record.node.flags.bits(), 0x79000);
    assert_eq!(record.life_span, Animatable::Static(1.0));
    assert_eq!(record.color, Animatable::Static([1.0; 3]));
    assert_eq!(record.visibility_guide.text(), "Always=on,\nDeath=off");
    assert_eq!(
        PopcornEmitter::decode_mdl(&record.encode_mdl().unwrap()).unwrap(),
        record
    );
    assert!(
        PopcornEmitter::decode_mdl("ParticleEmitterPopcorn \"p\" { ObjectId 0, XYQuad, }").is_err()
    );
    assert!(PopcornEmitter::decode_mdl(
        "ParticleEmitterPopcorn \"p\" { ObjectId 0, static Alpha 1, Alpha 0 {} }"
    )
    .is_err());
}
