use wc3::model::m2::{
    codec::{Decoder, Encoder},
    records::{
        BlendMode, Interpolation, Material, MaterialFlags, ModelFlags, ParticleGenerator,
        PhysicsShapeKind, TextureFlags,
    },
    Read as _, Write as _, ZeroVec,
};

#[test]
fn interpolation_and_unknown_wire_codes_keep_their_native_values() {
    assert_eq!(Interpolation::Hermite.raw(), 2);
    assert_eq!(Interpolation::Bezier.raw(), 3);
    let values = [
        Interpolation::Hermite,
        Interpolation::Bezier,
        Interpolation::Unknown(0xff31),
    ];
    let mut encoder = Encoder::new(264, ModelFlags::default());
    for value in values {
        value.write(&mut encoder).unwrap();
    }
    let bytes = encoder.finish();
    assert_eq!(bytes, [2, 0, 3, 0, 0x31, 0xff]);
    let mut decoder = Decoder::new(&bytes, 264, ModelFlags::default());
    for expected in values {
        assert_eq!(Interpolation::read(&mut decoder).unwrap(), expected);
    }
    let values =
        ZeroVec::alloc_from_slice(&[ParticleGenerator::Plane, ParticleGenerator::Unknown(0xffac)]);
    assert_eq!(values.as_bytes(), [1, 0, 0xac, 0xff]);
    assert_eq!(
        ZeroVec::<ParticleGenerator>::parse_bytes(values.as_bytes())
            .unwrap()
            .get(1),
        Some(ParticleGenerator::Unknown(0xffac))
    );
}

#[test]
fn flag_accessors_and_unaligned_material_arrays_retain_unknown_bits() {
    let mut texture = TextureFlags::from_bits_retain(0x8000_0000);
    texture.set_wrap_u(true);
    texture.set_wrap_v(true);
    assert!(texture.wrap_u() && texture.wrap_v());
    assert_eq!(texture.bits(), 0x8000_0003);
    texture.set_wrap_u(false);
    assert_eq!(texture.bits(), 0x8000_0002);
    let values = ZeroVec::alloc_from_slice(&[Material {
        flags: MaterialFlags::from_bits_retain(0x8001),
        blending_mode: BlendMode::Unknown(0x8123),
    }]);
    assert_eq!(values.as_bytes(), [1, 0x80, 0x23, 0x81]);
    let parsed = ZeroVec::<Material>::parse_bytes(values.as_bytes()).unwrap();
    let material = parsed.get(0).unwrap();
    assert!(material.flags.unlit());
    assert_eq!(material.flags.bits(), 0x8001);
    assert_eq!(material.blending_mode, BlendMode::Unknown(0x8123));
    let mut encoder = Encoder::new(264, ModelFlags::from_bits_retain(0x8000_0200));
    assert!(encoder.flags.particle_multi_texture());
    encoder.flags.set_particle_multi_texture(false);
    assert_eq!(encoder.flags.bits(), 0x8000_0000);
    assert_eq!(
        PhysicsShapeKind::from_raw(77),
        PhysicsShapeKind::Unknown(77)
    );
}
