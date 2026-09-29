use super::*;

#[test]
fn spec_examples_match_independently_packed_binary_records() {
    // MDX fixtures were packed from the attached specification, independently
    // of these codecs. Smoke uses unequal length/width to check their order.
    let smoke = ParticleEmitter2::decode_mdl(SMOKE).unwrap();
    let wire = include_bytes!("../../../fixtures/mdl/smoke.mdx");
    assert_eq!(smoke.encode_mdx().unwrap(), wire);
    assert_eq!(ParticleEmitter2::decode_mdx(wire).unwrap(), smoke);
    assert_eq!(smoke.length, 16.0);
    assert_eq!(smoke.width, 17.0);
    assert_eq!(
        ParticleEmitter2::decode_mdl(&smoke.encode_mdl().unwrap()).unwrap(),
        smoke
    );
    let popcorn = PopcornEmitter::decode_mdl(POPCORN).unwrap();
    let wire = include_bytes!("../../../fixtures/mdl/popcorn_fire.mdx");
    assert_eq!(popcorn.encode_mdx().unwrap(), wire);
    assert_eq!(PopcornEmitter::decode_mdx(wire).unwrap(), popcorn);
    assert_eq!(
        PopcornEmitter::decode_mdl(&popcorn.encode_mdl().unwrap()).unwrap(),
        popcorn
    );
    let camera = Camera::<V800>::decode_mdl(CAMERA).unwrap();
    let wire = include_bytes!("../../../fixtures/mdl/portrait.mdx");
    assert_eq!(camera.encode_mdx().unwrap(), wire);
    assert_eq!(Camera::<V800>::decode_mdx(wire).unwrap(), camera);
    let canonical = camera.encode_mdl().unwrap();
    assert!(canonical.contains("FocusDistanceKeys 1"));
    assert_eq!(Camera::<V800>::decode_mdl(&canonical).unwrap(), camera);
}
