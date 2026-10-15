//! Root record fixtures assembled independently of the native encoders.
use wc3::model::formats::wmo::records::{
    AmbientBox, DoodadDef, FogBox, Header, Material, NewLight,
};
use wc3::model::formats::wmo::root::RootChunk;
use wc3::model::formats::wmo::{Classic, Legion, RecordLayout};

#[test]
fn fixed_record_strides_match_documented_wire_layouts() {
    assert_eq!(Header::<Classic>::size(), 64);
    assert_eq!(Header::<Legion>::size(), 64);
    assert_eq!(Material::<Classic>::size(), 64);
    assert_eq!(Material::<Legion>::size(), 64);
    assert_eq!(NewLight::size(), 184);
    assert_eq!(FogBox::size(), 140);
    assert_eq!(AmbientBox::size(), 128);
    assert_eq!(DoodadDef::size(), 40);
}

#[test]
fn doodad_packed_name_flags_transform_and_color_decode_without_copying() {
    let mut bytes = vec![0; 40];
    bytes[..4].copy_from_slice(&0x0800_0123u32.to_le_bytes());
    bytes[4..8].copy_from_slice(&2.5f32.to_le_bytes());
    bytes[28..32].copy_from_slice(&1.0f32.to_le_bytes());
    bytes[32..36].copy_from_slice(&1.5f32.to_le_bytes());
    bytes[36..40].copy_from_slice(&[10, 20, 30, 40]);
    let chunk = RootChunk::<Classic>::decode(*b"MODD", &bytes, 128).unwrap();
    let RootChunk::Doodads(records) = &chunk else {
        panic!("MODD")
    };
    assert_eq!(records.as_bytes().as_ptr(), bytes.as_ptr());
    let record = records.get(0).unwrap().unwrap();
    assert_eq!(record.name_and_flags.bits(), 0x0800_0123);
    assert_eq!(record.position, [2.5, 0.0, 0.0]);
    assert_eq!(record.rotation, [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(record.scale, 1.5);
    assert_eq!(record.color, [10, 20, 30, 40]);
    assert_eq!(chunk.encode().unwrap(), bytes);
}

#[test]
fn fixed_chunks_reject_bad_strides_and_unknown_chunks_preserve_bytes() {
    assert!(RootChunk::<Classic>::decode(*b"MOMT", &[0; 63], 400).is_err());
    assert!(RootChunk::<Classic>::decode(*b"MOSI", &[0; 5], 400).is_err());
    let bytes = [9, 8, 7, 6, 5];
    let chunk = RootChunk::<Classic>::decode(*b"ZZZZ", &bytes, 400).unwrap();
    assert_eq!(chunk.tag(), *b"ZZZZ");
    assert_eq!(chunk.encode().unwrap(), bytes);
}
