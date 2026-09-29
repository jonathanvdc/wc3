use super::*;

#[test]
fn emitter_tracks_preserve_keys_and_override_bases() {
    let mut body = String::new();
    for name in [
        "Speed",
        "Variation",
        "Latitude",
        "Gravity",
        "EmissionRate",
        "Length",
        "Width",
        "Visibility",
    ] {
        body.push_str(&format!(
            "{name} 1 {{ Hermite, GlobalSeqId 3, -7: 2, InTan 1, OutTan 3, }}"
        ));
    }
    let mut record =
        ParticleEmitter2::decode_mdl(&format!("ParticleEmitter2 \"p\" {{ ObjectId 0, {body} }}"))
            .unwrap();
    assert!(
        record.speed.track().is_some()
            && record.variation.track().is_some()
            && record.latitude.track().is_some()
            && record.gravity.track().is_some()
            && record.emission_rate.track().is_some()
            && record.length.track().is_some()
            && record.width.track().is_some()
            && record.visibility.is_some()
    );
    assert_eq!(
        ParticleEmitter2::decode_mdl(&record.encode_mdl().unwrap()).unwrap(),
        record
    );
    record.speed.set_value(10.0);
    let decoded = ParticleEmitter2::decode_mdl(&record.encode_mdl().unwrap()).unwrap();
    assert_eq!(decoded.speed.value(), Some(&0.0));
    assert_eq!(decoded.speed.track(), record.speed.track());
    let mut body = String::new();
    for name in ["LifeSpan", "EmissionRate", "Speed", "Alpha", "Visibility"] {
        body.push_str(&format!(
            "{name} 1 {{ Bezier, GlobalSeqId 2, -3: 1, InTan 2, OutTan 3, }}"
        ));
    }
    body.push_str("Color 1 { Hermite, -1: { 1, 2, 3 }, InTan { 4, 5, 6 }, OutTan { 7, 8, 9 }, }");
    let mut record = PopcornEmitter::decode_mdl(&format!(
        "ParticleEmitterPopcorn \"p\" {{ ObjectId 0, {body} }}"
    ))
    .unwrap();
    assert!(
        record.life_span.track().is_some()
            && record.emission_rate.track().is_some()
            && record.speed.track().is_some()
            && record.alpha.track().is_some()
            && record.color.track().is_some()
            && record.visibility.is_some()
    );
    let decoded = PopcornEmitter::decode_mdl(&record.encode_mdl().unwrap()).unwrap();
    assert_eq!(decoded, record);
    record.alpha.set_value(0.0);
    let decoded = PopcornEmitter::decode_mdl(&record.encode_mdl().unwrap()).unwrap();
    assert_eq!(decoded.alpha.value(), Some(&1.0));
    assert_eq!(decoded.alpha.track(), record.alpha.track());
}
