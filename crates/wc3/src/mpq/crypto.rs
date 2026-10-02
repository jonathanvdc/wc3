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

/// lookup3 hashlittle2 over at most 264 lowercase, slash-normalized filename bytes.
/// The primary and secondary seeds are 1 and 2.
pub(super) fn jenkins(name: &[u8]) -> u64 {
    let data: Vec<u8> = name
        .iter()
        .take(0x108)
        .map(|&b| {
            if b == b'/' {
                b'\\'
            } else {
                b.to_ascii_lowercase()
            }
        })
        .collect();
    let mut a = 0xdeadbeefu32
        .wrapping_add(data.len() as u32)
        .wrapping_add(2);
    let mut b = a;
    let mut c = a.wrapping_add(1);
    let mut tail = data.as_slice();
    while tail.len() > 12 {
        a = a.wrapping_add(u32::from_le_bytes(tail[..4].try_into().unwrap()));
        b = b.wrapping_add(u32::from_le_bytes(tail[4..8].try_into().unwrap()));
        c = c.wrapping_add(u32::from_le_bytes(tail[8..12].try_into().unwrap()));
        a = a.wrapping_sub(c);
        a ^= c.rotate_left(4);
        c = c.wrapping_add(b);
        b = b.wrapping_sub(a);
        b ^= a.rotate_left(6);
        a = a.wrapping_add(c);
        c = c.wrapping_sub(b);
        c ^= b.rotate_left(8);
        b = b.wrapping_add(a);
        a = a.wrapping_sub(c);
        a ^= c.rotate_left(16);
        c = c.wrapping_add(b);
        b = b.wrapping_sub(a);
        b ^= a.rotate_left(19);
        a = a.wrapping_add(c);
        c = c.wrapping_sub(b);
        c ^= b.rotate_left(4);
        b = b.wrapping_add(a);
        tail = &tail[12..];
    }
    if !tail.is_empty() {
        for (i, &byte) in tail.iter().enumerate() {
            let n = (byte as u32) << ((i % 4) * 8);
            match i / 4 {
                0 => a = a.wrapping_add(n),
                1 => b = b.wrapping_add(n),
                _ => c = c.wrapping_add(n),
            }
        }
        c ^= b;
        c = c.wrapping_sub(b.rotate_left(14));
        a ^= c;
        a = a.wrapping_sub(c.rotate_left(11));
        b ^= a;
        b = b.wrapping_sub(a.rotate_left(25));
        c ^= b;
        c = c.wrapping_sub(b.rotate_left(16));
        a ^= c;
        a = a.wrapping_sub(c.rotate_left(4));
        b ^= a;
        b = b.wrapping_sub(a.rotate_left(14));
        c ^= b;
        c = c.wrapping_sub(b.rotate_left(24));
    }
    ((b as u64) << 32) | c as u64
}
