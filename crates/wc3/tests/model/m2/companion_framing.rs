//! Companion framing preserves borrowed payloads and file-relative diagnostics.
use std::borrow::Cow;
use wc3::model::common::binary::ChunkHeader;
use wc3::model::formats::m2::codec::Decoder;
use wc3::model::formats::m2::companions::{AnimationFile, BoneFile, Skeleton, SkeletonChunk};
use wc3::model::formats::m2::records::ModelFlags;
use wc3::model::formats::m2::{self, V274};

fn frame(tag: [u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut bytes = ChunkHeader {
        tag,
        size: payload.len() as u32,
    }
    .to_bytes()
    .to_vec();
    bytes.extend_from_slice(payload);
    bytes
}

#[test]
fn animation_preserves_order_duplicates_and_borrowed_unknown_payloads() {
    let mut bytes = frame(*b"TEST", &[1, 2, 3]);
    bytes.extend(frame(*b"AFM2", &[4, 5]));
    bytes.extend(frame(*b"AFM2", &[6]));
    let animation = AnimationFile::read(&bytes, true).unwrap();
    let AnimationFile::Chunked(streams) = &animation else {
        panic!("framed animation")
    };
    assert_eq!(streams.len(), 3);
    assert!(matches!(streams[0].data, Cow::Borrowed(_)));
    assert_eq!(streams[0].data.as_ptr(), bytes[8..].as_ptr());
    assert_eq!(animation.stream(*b"AFM2"), Some([4, 5].as_slice()));
    assert_eq!(animation.write().unwrap(), bytes);

    bytes.extend([0, 1]);
    assert_eq!(AnimationFile::read(&bytes, true).unwrap_err().offset, 30);
    assert_eq!(
        AnimationFile::read(&bytes, false).unwrap().write().unwrap(),
        bytes
    );
}

#[test]
fn skeleton_payload_errors_locate_the_chunk_in_the_whole_file() {
    for (tag, payload) in [
        (*b"SKA1", &[][..]),
        (*b"SKB1", &[][..]),
        (*b"SKS1", &[][..]),
        (*b"BFID", &[0][..]),
        (*b"AFID", &[0][..]),
    ] {
        let mut bytes = frame(*b"TEST", &[1, 2, 3]);
        bytes.extend(frame(tag, payload));
        let error = Skeleton::<V274>::read(&bytes, ModelFlags::default()).unwrap_err();
        assert_eq!(error.offset, 19, "{tag:?}");

        let mut decoder = Decoder::new(&bytes, 274, ModelFlags::default()).with_origin(100);
        let nested = <Skeleton<V274> as m2::Read>::read(&mut decoder).unwrap_err();
        assert_eq!(nested.offset, 119, "{tag:?}");
    }
}

#[test]
fn skeleton_offset_bounds_remain_payload_relative_and_errors_are_absolute() {
    let mut payload = vec![0; 16];
    payload[4..8].copy_from_slice(&1u32.to_le_bytes()); // name count
    payload[8..12].copy_from_slice(&16u32.to_le_bytes()); // missing name byte
    let mut bytes = frame(*b"TEST", &[1, 2, 3]);
    bytes.extend(frame(*b"SKL1", &payload));
    let error = Skeleton::<V274>::read(&bytes, ModelFlags::default()).unwrap_err();
    assert_eq!(error.offset, 35);
}

#[test]
fn skeleton_retains_unknown_payloads_and_rejects_truncated_final_framing() {
    let mut bytes = frame(*b"TEST", &[1, 2, 3]);
    let skeleton = Skeleton::<V274>::read(&bytes, ModelFlags::default()).unwrap();
    let SkeletonChunk::Unknown { data, .. } = &skeleton.chunks[0] else {
        panic!("unknown chunk")
    };
    assert!(matches!(data, Cow::Borrowed(_)));
    assert_eq!(data.as_ptr(), bytes[8..].as_ptr());
    assert_eq!(skeleton.write().unwrap(), bytes);
    bytes.extend([0, 1]);
    assert_eq!(
        Skeleton::<V274>::read(&bytes, ModelFlags::default())
            .unwrap_err()
            .offset,
        11
    );
}

#[test]
fn bone_requires_exactly_two_ordered_frames_and_reports_payload_origins() {
    let mut empty = 1u32.to_le_bytes().to_vec();
    empty.extend(frame(*b"BIDA", &[]));
    empty.extend(frame(*b"BOMT", &[]));
    assert_eq!(BoneFile::read(&empty).unwrap().write().unwrap(), empty);
    let mut extra = empty.clone();
    extra.extend(frame(*b"TEST", &[]));
    assert!(BoneFile::read(&extra).is_err());
    let mut truncated = empty.clone();
    truncated.extend([0, 1]);
    assert_eq!(BoneFile::read(&truncated).unwrap_err().offset, 20);
    let mut reversed = 1u32.to_le_bytes().to_vec();
    reversed.extend(frame(*b"BOMT", &[]));
    reversed.extend(frame(*b"BIDA", &[]));
    assert!(BoneFile::read(&reversed).is_err());

    let mut bad_indices = 1u32.to_le_bytes().to_vec();
    bad_indices.extend(frame(*b"BIDA", &[0]));
    bad_indices.extend(frame(*b"BOMT", &[]));
    assert_eq!(BoneFile::read(&bad_indices).unwrap_err().offset, 12);
    let mut bad_matrix = 1u32.to_le_bytes().to_vec();
    bad_matrix.extend(frame(*b"BIDA", &[]));
    bad_matrix.extend(frame(*b"BOMT", &[0]));
    assert_eq!(BoneFile::read(&bad_matrix).unwrap_err().offset, 20);

    let mut decoder = Decoder::new(&bad_matrix, 0, ModelFlags::default()).with_origin(100);
    assert_eq!(
        <BoneFile as m2::Read>::read(&mut decoder)
            .unwrap_err()
            .offset,
        120
    );
}
