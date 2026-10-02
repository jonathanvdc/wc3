const fn crypt_table() -> [u32; 1280] {
    let mut table = [0; 1280];
    let mut seed = 0x100001u32;
    let mut i = 0;
    while i < 256 {
        let mut j = 0;
        while j < 5 {
            seed = (seed * 125 + 3) % 0x2aaaab;
            let high = (seed & 0xffff) << 16;
            seed = (seed * 125 + 3) % 0x2aaaab;
            table[i + j * 256] = high | (seed & 0xffff);
            j += 1;
        }
        i += 1;
    }
    table
}

const TABLE: [u32; 1280] = crypt_table();

pub(super) fn hash(name: &[u8], kind: usize) -> u32 {
    let mut a = 0x7fed7fedu32;
    let mut b = 0xeeeeeeeeu32;
    for &byte in name {
        let ch = if byte == b'/' {
            b'\\'
        } else {
            byte.to_ascii_uppercase()
        };
        a = TABLE[kind * 256 + ch as usize] ^ a.wrapping_add(b);
        b = (ch as u32)
            .wrapping_add(a)
            .wrapping_add(b)
            .wrapping_add(b << 5)
            .wrapping_add(3);
    }
    a
}

pub(super) fn file_key(name: &[u8], offset: u32, size: u32, adjusted: bool) -> u32 {
    let base = name
        .rsplit(|&b| b == b'/' || b == b'\\')
        .next()
        .unwrap_or(name);
    let key = hash(base, 3);
    if adjusted {
        key.wrapping_add(offset) ^ size
    } else {
        key
    }
}

/// MPQ encrypts complete little-endian words; trailing bytes remain unchanged.
pub(super) fn crypt(bytes: &mut [u8], mut key: u32, decrypt: bool) {
    let mut seed = 0xeeeeeeeeu32;
    for word in bytes.chunks_exact_mut(4) {
        seed = seed.wrapping_add(TABLE[1024 + (key & 255) as usize]);
        let input = u32::from_le_bytes(word.try_into().unwrap());
        let output = input ^ key.wrapping_add(seed);
        let plain = if decrypt { output } else { input };
        word.copy_from_slice(&output.to_le_bytes());
        key = ((!key << 21).wrapping_add(0x11111111)) | (key >> 11);
        seed = plain
            .wrapping_add(seed)
            .wrapping_add(seed << 5)
            .wrapping_add(3);
    }
}

#[cfg(test)]
#[path = "crypto_tests.rs"]
mod tests;
