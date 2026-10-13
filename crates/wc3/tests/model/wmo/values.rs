use wc3::model::wmo::{
    self,
    codec::{Decoder, Encoder},
    records::{BatchFlags, BlendMode, DoodadReference, FlickerMode, LightKind},
    Read as _, Write as _,
};
#[test]
fn scalar_codes_preserve_unknown_values_including_signed_codes() {
    let unknown = BlendMode::Unknown(u32::MAX);
    let mut encoded = Encoder::new();
    unknown.write(&mut encoded).unwrap();
    let bytes = encoded.finish();
    assert_eq!(BlendMode::read(&mut Decoder::new(&bytes)).unwrap(), unknown);
    assert_eq!(LightKind::from_raw(1), LightKind::Spot);
    assert_eq!(FlickerMode::from_raw(-99), FlickerMode::Unknown(-99));
    assert_eq!(FlickerMode::Unknown(-99).raw(), -99);
    assert_eq!(<BlendMode as wmo::RecordLayout>::size(), 4);
}
#[test]
fn changing_named_flags_preserves_other_bits_and_packed_identity() {
    let mut flags = BatchFlags(0x80);
    flags.set_large_material(true);
    assert_eq!(flags.bits(), 0x82);
    assert!(flags.large_material());
    let mut reference = DoodadReference(0x8000_1234);
    let mut flags = reference.flags();
    flags.set_has_light_reference(true);
    reference.set_flags(flags);
    assert_eq!(reference.name_index(), 0x1234);
    assert_eq!(reference.bits(), 0x8400_1234);
    let mut encoded = Encoder::new();
    reference.write(&mut encoded).unwrap();
    let bytes = encoded.finish();
    assert_eq!(
        DoodadReference::read(&mut Decoder::new(&bytes)).unwrap(),
        reference
    );
}
