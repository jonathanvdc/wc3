use super::*;

#[test]
fn particle2_choices_arrays_defaults_and_validation() {
    let base = "ParticleEmitter2 \"p\" { ObjectId 0, ";
    let empty = ParticleEmitter2::decode_mdl(&format!("{base}}}")).unwrap();
    assert_eq!(empty, ParticleEmitter2::new(empty.node.clone()));
    assert_eq!(empty.node.flags.bits(), 0x1000);
    for (index, filter) in ["Blend", "Additive", "Modulate", "Modulate2x", "AlphaKey"]
        .iter()
        .enumerate()
    {
        for (frame, keyword) in ["Head", "Tail", "Both"].iter().enumerate() {
            let text = format!("{base}{filter}, {keyword}, Squirt 7, PriorityPlane 4294967295, ReplaceableId 2, }}");
            let record = ParticleEmitter2::decode_mdl(&text).unwrap();
            assert_eq!(
                record.filter_mode,
                Particle2FilterMode::from_raw(index as u32)
            );
            assert_eq!(record.frames, Particle2Frames::from_raw(frame as u32));
            assert_eq!(record.squirt, 7);
            assert_eq!(
                ParticleEmitter2::decode_mdl(&record.encode_mdl().unwrap()).unwrap(),
                record
            );
        }
    }
    for body in ["Blend, Additive,", "Head, Tail,", "Both, Both,", "Alpha { 256, 0, 0 },", "Alpha { -1, 0, 0 },", "LifeSpanUVAnim { 0, 1 },", "SegmentColor {}", "SegmentColor { Color { 1, 1, 1 }, Color { 1, 1, 1 }, }", "SegmentColor { Color { 1, 1, 1 }, Color { 1, 1, 1 }, Color { 1, 1, 1 }, Color { 1, 1, 1 }, }", "static Speed 2, Speed 1 { Linear, 0: 1, }", "LifeSpan 1 { Linear, 0: 1, }"] {
        assert!(ParticleEmitter2::decode_mdl(&format!("{base}{body}}}")).is_err(), "{body}");
    }
    let mut invalid = empty.clone();
    invalid.filter_mode = Particle2FilterMode::Unknown(5);
    assert!(invalid.encode_mdl().is_err());
    invalid.filter_mode = Particle2FilterMode::Blend;
    invalid.frames = Particle2Frames::Unknown(3);
    assert!(invalid.encode_mdl().is_err());
    let flags = ParticleEmitter2::decode_mdl(&format!(
        "{base}SortPrimsFarZ, LineEmitter, Unfogged, ModelSpace, Unshaded, XYQuad, }}"
    ))
    .unwrap();
    assert_eq!(flags.node.flags.bits(), 0x1f9000);
}
