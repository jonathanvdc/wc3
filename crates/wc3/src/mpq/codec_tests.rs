use super::{decode, Error};

use super::super::codec_test_vectors::{
    HUFFMAN, LZMA_EOS, LZMA_SIZED, MONO, MONO_CHAIN, MONO_PCM, RAW, STEREO, STEREO_CHAIN,
    STEREO_PCM,
};

fn sector(mask: u8, data: &[u8]) -> Vec<u8> {
    let mut bytes = vec![mask];
    bytes.extend_from_slice(data);
    bytes
}

#[test]
fn huffman_distributions() {
    for data in HUFFMAN {
        assert_eq!(decode(&sector(1, data), RAW.len(), false).unwrap(), RAW);
        assert!(decode(&sector(1, data), RAW.len() - 1, false).is_err());
        assert!(decode(&sector(1, data), RAW.len() + 1, false).is_err());
        assert!(decode(&sector(1, &data[..data.len() / 2]), RAW.len(), false).is_err());
    }
}

#[test]
fn adpcm_and_huffman_chains() {
    for (mask, data, expected) in [
        (0x40, MONO, MONO_PCM),
        (0x80, STEREO, STEREO_PCM),
        (0x41, MONO_CHAIN, MONO_PCM),
        (0x81, STEREO_CHAIN, STEREO_PCM),
    ] {
        assert_eq!(
            decode(&sector(mask, data), expected.len(), false).unwrap(),
            expected
        );
        assert!(decode(&sector(mask, data), expected.len() - 1, false).is_err());
        assert!(decode(&sector(mask, data), expected.len() + 1, false).is_err());
        assert!(decode(
            &sector(mask, &data[..data.len() / 2]),
            expected.len(),
            false
        )
        .is_err());
    }
}

#[test]
fn lzma_mpq_headers_and_size_limits() {
    for data in [LZMA_EOS, LZMA_SIZED] {
        assert_eq!(decode(&sector(0x12, data), RAW.len(), false).unwrap(), RAW);
        assert!(decode(&sector(0x12, data), RAW.len() - 1, false).is_err());
        assert!(decode(&sector(0x12, data), RAW.len() + 1, false).is_err());
        for end in 0..data.len() / 2 {
            assert!(decode(&sector(0x12, &data[..end]), RAW.len(), false).is_err());
        }
    }
    let mut filtered = LZMA_SIZED.to_vec();
    filtered[0] = 1;
    assert!(decode(&sector(0x12, &filtered), RAW.len(), false).is_err());
}

#[test]
fn malformed_headers_and_masks() {
    for mask in [0, 4, 0xc0, 0xff] {
        assert!(matches!(
            decode(&[mask], 16, false),
            Err(Error::UnsupportedCompression(_))
        ));
    }
    for mask in [1, 0x40, 0x80, 0x12] {
        assert!(decode(&[mask], 16, false).is_err());
    }
    for shift in [32, 255] {
        assert!(decode(&[0x40, 0, shift, 0, 0, 1], 4, false).is_err());
    }
    assert!(decode(&[1, 9, 0], 4, false).is_err());
    assert!(decode(&[0x80, 0, 4, 0, 0], 4, false).is_err());
}

#[test]
fn adpcm_control_bytes_and_saturation() {
    // Step changes do not emit a sample or advance the stereo channel.
    let bytes = [
        0x80, 0, 0, 0xff, 0x7f, 0, 0x80, 0x81, 0x3f, 0x81, 0x7f, 0x80, 0x80,
    ];
    let expected = [
        0xff, 0x7f, 0, 0x80, 0xff, 0x7f, 0, 0x80, 0xff, 0x7f, 0, 0x80,
    ];
    assert_eq!(decode(&bytes, expected.len(), false).unwrap(), expected);
}

#[test]
fn lzma_known_size_without_end_marker() {
    use lzma_rs::{
        compress::{Options, UnpackedSize},
        lzma_compress_with_options,
    };

    let mut encoded = vec![0x12, 0];
    let options = Options {
        unpacked_size: UnpackedSize::WriteToHeader(Some(RAW.len() as u64)),
    };
    lzma_compress_with_options(&mut &RAW[..], &mut encoded, &options).unwrap();
    assert_eq!(decode(&encoded, RAW.len(), false).unwrap(), RAW);
    assert!(decode(&encoded[..encoded.len() - 4], RAW.len(), false).is_err());
}

#[test]
fn zlib_huffman_chain() {
    use flate2::{write::ZlibEncoder, Compression};
    use std::io::Write;

    let mut encoder = ZlibEncoder::new(vec![0x43], Compression::default());
    encoder.write_all(MONO_CHAIN).unwrap();
    let encoded = encoder.finish().unwrap();
    assert_eq!(decode(&encoded, MONO_PCM.len(), false).unwrap(), MONO_PCM);
}
