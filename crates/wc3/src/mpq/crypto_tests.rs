use super::{crypt, file_key, hash};

#[test]
fn fixed_cipher_and_filename_key_vectors() {
    let plain: Vec<u8> = [0u32, 1, 2, 3]
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect();
    let expected: Vec<u8> = [0x2b14c012u32, 0x7ef08e72, 0x3c48e5f2, 0x53ad2a73]
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect();
    let mut encrypted = plain.clone();
    crypt(&mut encrypted, 0x12345678, false);
    assert_eq!(encrypted, expected);
    let mut decrypted = expected;
    crypt(&mut decrypted, 0x12345678, true);
    assert_eq!(decrypted, plain);
    assert_eq!(hash(b"Units\\Test.bin", 1), 0x9e34bac0);
    assert_eq!(file_key(b"Units/Test.bin", 32, 100, false), 0x46dc6e18);
    assert_eq!(file_key(b"Units/Test.bin", 32, 100, true), 0x46dc6e5c);
}

#[test]
fn standard_table_keys() {
    assert_eq!(hash(b"(hash table)", 3), 0xc3af3770);
    assert_eq!(hash(b"(block table)", 3), 0xec83b3a3);
    assert_eq!(
        hash(b"units/Footman.mdx", 1),
        hash(b"UNITS\\FOOTMAN.MDX", 1)
    );
}

#[test]
fn encryption_preserves_trailing_bytes() {
    for len in 0..35 {
        let input: Vec<u8> = (0..len).collect();
        let mut bytes = input.clone();
        crypt(&mut bytes, 0xc3af3770, false);
        assert_eq!(
            &bytes[len as usize / 4 * 4..],
            &input[len as usize / 4 * 4..]
        );
        crypt(&mut bytes, 0xc3af3770, true);
        assert_eq!(bytes, input);
    }
}
