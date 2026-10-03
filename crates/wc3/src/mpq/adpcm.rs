//! Blizzard ADPCM decoding (16-bit little-endian PCM).
//! Tables adapted from stormlib-rs 0.0.3; Copyright (c) 2026 notOrrytrout.
//! See licenses/stormlib-rs-MIT.txt.

use super::Error;

const NEXT_STEP_TABLE: [i32; 32] = [
    -1, 0, -1, 4, -1, 2, -1, 6, -1, 1, -1, 5, -1, 3, -1, 7, -1, 1, -1, 5, -1, 3, -1, 7, -1, 2, -1,
    4, -1, 6, -1, 8,
];

const STEP_SIZE_TABLE: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66,
    73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449,
    494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272,
    2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493,
    10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767,
];

pub(super) fn decode(input: &[u8], channels: usize, limit: usize) -> Result<Vec<u8>, Error> {
    let header_len = 2 + channels * 2;
    if input.len() < header_len || input[0] != 0 || input[1] > 31 {
        return Err(Error::InvalidArchive("invalid ADPCM header"));
    }
    if header_len - 2 > limit {
        return Err(Error::InvalidArchive("ADPCM output exceeds declared size"));
    }
    let shift = input[1];
    let mut samples = [0i32; 2];
    let mut indices = [44i32; 2];
    let mut output = Vec::new();
    for (channel, bytes) in input[2..header_len].as_chunks::<2>().0.iter().enumerate() {
        samples[channel] = i16::from_le_bytes(*bytes) as i32;
        output.extend_from_slice(bytes);
    }
    let mut channel = channels - 1;
    for &code in &input[header_len..] {
        channel = (channel + 1) % channels;
        match code {
            0x80 => indices[channel] = (indices[channel] - 1).max(0),
            0x81 => {
                indices[channel] = (indices[channel] + 8).min(88);
                channel = (channel + 1) % channels;
                continue;
            }
            _ => {
                let step = STEP_SIZE_TABLE[indices[channel] as usize];
                let mut difference = step >> shift;
                for bit in 0..6 {
                    if code & (1 << bit) != 0 {
                        difference += step >> bit;
                    }
                }
                let difference = if code & 0x40 != 0 {
                    -difference
                } else {
                    difference
                };
                samples[channel] = (samples[channel] + difference).clamp(-32768, 32767);
                indices[channel] =
                    (indices[channel] + NEXT_STEP_TABLE[(code & 31) as usize]).clamp(0, 88);
            }
        }
        if limit - output.len() < 2 {
            return Err(Error::InvalidArchive("ADPCM output exceeds declared size"));
        }
        output.extend_from_slice(&(samples[channel] as i16).to_le_bytes());
    }
    Ok(output)
}
