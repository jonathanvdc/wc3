use wc3::model::{m2, m3, wem, wmo};
use wem::records::{BlendMode, D3Group, MaterialFlags};

#[derive(Clone, Copy, Debug, PartialEq, Eq, m2::Value, m2::Read, m2::Write, m2::RecordLayout)]
#[m2(value = u8)]
enum M2Code {
    #[m2(value = 1)]
    Known,
    #[m2(unknown)]
    Unknown(u8),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, m3::Value, m3::Read, m3::Write, m3::RecordLayout)]
#[m3(value = i16)]
enum M3Code {
    #[m3(value = -1)]
    Known,
    #[m3(unknown)]
    Unknown(i16),
}
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, wmo::Value, wmo::Read, wmo::Write, wmo::RecordLayout,
)]
#[wmo(value = u64)]
enum WmoCode {
    #[wmo(value = 1)]
    Known,
    #[wmo(unknown)]
    Unknown(u64),
}
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, wem::Value, wem::Read, wem::Write, wem::RecordLayout,
)]
#[wem(value = i64)]
enum WemCode {
    #[wem(value = -1)]
    Known,
    #[wem(unknown)]
    Unknown(i64),
}

#[test]
fn scalar_derives_round_trip_known_unknown_and_signed_values() {
    use m2::{Read as _, Write as _};
    use wmo::{Read as _, Write as _};
    for value in [M2Code::Known, M2Code::Unknown(255)] {
        let mut out = m2::codec::Encoder::new(264, Default::default());
        value.write(&mut out).unwrap();
        let bytes = out.finish();
        assert_eq!(bytes, [value.raw()]);
        assert_eq!(
            M2Code::read(&mut m2::codec::Decoder::new(
                &bytes,
                264,
                Default::default()
            ))
            .unwrap(),
            value
        );
    }
    for value in [M3Code::Known, M3Code::Unknown(i16::MIN)] {
        let mut out = m3::codec::Encoder::<m3::Md34>::new();
        out.write(&value).unwrap();
        let bytes = out.finish();
        assert_eq!(bytes, value.raw().to_le_bytes());
        assert_eq!(
            m3::codec::Decoder::<m3::Md34>::new(&bytes)
                .read::<M3Code>()
                .unwrap(),
            value
        );
        let mut out = m3::codec::Encoder::<m3::Md33>::new();
        out.write(&value).unwrap();
        assert_eq!(out.finish(), bytes);
    }
    for value in [WmoCode::Known, WmoCode::Unknown(u64::MAX)] {
        let mut out = wmo::codec::Encoder::new();
        value.write(&mut out).unwrap();
        let bytes = out.finish();
        assert_eq!(bytes, value.raw().to_le_bytes());
        assert_eq!(
            WmoCode::read(&mut wmo::codec::Decoder::new(&bytes)).unwrap(),
            value
        );
    }
    for value in [WemCode::Known, WemCode::Unknown(i64::MIN)] {
        let mut out = wem::Encoder::new(1);
        out.write(&value).unwrap();
        let bytes = out.finish();
        assert_eq!(bytes, value.raw().to_le_bytes());
        assert_eq!(
            wem::Array::<WemCode>::read(&bytes, 1, 1)
                .unwrap()
                .get(0)
                .unwrap()
                .unwrap(),
            value
        );
    }
}

#[test]
fn wem_choices_and_flags_preserve_unknown_wire_bits() {
    assert_eq!(BlendMode::from_raw(255), BlendMode::Unknown(255));
    assert_eq!(D3Group::from_raw(-1), D3Group::Unspecified);
    let mut flags = MaterialFlags::from_bits_retain(0x80000000);
    flags.set_unlit(true);
    assert!(flags.unlit());
    flags.set_unlit(false);
    assert_eq!(flags.bits(), 0x80000000);
}
