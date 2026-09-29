use super::*;

#[test]
fn emitter_tracks_preserve_signed_keys_splines_globals_and_hidden_bases() {
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
    assert_eq!(record.tracks.len(), 8);
    assert_eq!(
        ParticleEmitter2::decode_mdl(&record.encode_mdl().unwrap()).unwrap(),
        record
    );
    record.speed = 10.0;
    assert!(record.encode_mdl().is_err());
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
    assert_eq!(record.tracks.len(), 6);
    assert_eq!(
        PopcornEmitter::decode_mdl(&record.encode_mdl().unwrap()).unwrap(),
        record
    );
    record.alpha = 0.0;
    assert!(record.encode_mdl().is_err());
}
