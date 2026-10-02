#[cfg(all(feature = "mpq-encode", not(feature = "mpq-decode")))]
use std::io::Write;
#[cfg(feature = "mpq-decode")]
use std::io::{self, Read, Write as IoWrite};

#[cfg(feature = "mpq-decode")]
use super::{adpcm, huffman};
use super::{Compression, Error};

/// MPQ sector checksums are Adler-32 over decrypted, encoded bytes, seeded zero.
pub(super) fn sector_checksum(bytes: &[u8]) -> u32 {
    let mut a = 0u32;
    let mut b = 0u32;
    for chunk in bytes.chunks(5552) {
        for &byte in chunk {
            a += byte as u32;
            b += a;
        }
        a %= 65521;
        b %= 65521;
    }
    (b << 16) | a
}

pub(super) fn decode(input: &[u8], expected: usize, implode: bool) -> Result<Vec<u8>, Error> {
    #[cfg(not(feature = "mpq-decode"))]
    {
        let _ = (input, expected, implode);
        Err(Error::FeatureDisabled("mpq-decode"))
    }
    #[cfg(feature = "mpq-decode")]
    {
        use bzip2::read::BzDecoder;
        use explode::ExplodeReader;
        use flate2::read::ZlibDecoder;

        let (mask, data) = if implode {
            (8, input)
        } else {
            let (&mask, data) = input
                .split_first()
                .ok_or(Error::InvalidArchive("missing compression mask"))?;
            (mask, data)
        };
        // 0x12 identifies LZMA, not a zlib/bzip2 combination.
        if mask == 0 || mask & 4 != 0 || mask & 0xc0 == 0xc0 {
            return Err(Error::UnsupportedCompression(mask));
        }
        let mut output = data.to_vec();
        if mask == 0x12 {
            output = lzma(data, expected)?;
        } else {
            // Reverse of MPQ's compression order.
            if mask & 0x10 != 0 {
                output = bounded(BzDecoder::new(output.as_slice()), expected)?;
            }
            if mask & 8 != 0 {
                output = bounded(ExplodeReader::new(output.as_slice()), expected)?;
            }
            if mask & 2 != 0 {
                output = bounded(ZlibDecoder::new(output.as_slice()), expected)?;
            }
            if mask & 1 != 0 {
                output = huffman::decode(&output, expected)?;
            }
            if mask & 0xc0 != 0 {
                output = adpcm::decode(&output, if mask & 0x80 != 0 { 2 } else { 1 }, expected)?;
            }
            if mask & 0x20 != 0 {
                output = sparse(&output, expected)?;
            }
        }
        if output.len() != expected {
            return Err(Error::InvalidArchive("decompressed length mismatch"));
        }
        Ok(output)
    }
}

#[cfg(feature = "mpq-decode")]
fn lzma(input: &[u8], limit: usize) -> Result<Vec<u8>, Error> {
    use lzma_rs::{decompress::Options, lzma_decompress_with_options};

    // MPQ adds a filter byte before the standard LZMA-alone header.
    if input.len() < 15 || input[0] != 0 {
        return Err(Error::InvalidArchive("invalid MPQ LZMA header"));
    }
    let size = u64::from_le_bytes(input[6..14].try_into().unwrap());
    if size != u64::MAX && size != limit as u64 {
        return Err(Error::InvalidArchive("LZMA decompressed length mismatch"));
    }
    let mut output = LimitedOutput {
        bytes: Vec::new(),
        limit,
    };
    let options = Options {
        // Bound the dynamically allocated dictionary, independently of the header.
        memlimit: Some(limit.max(4096)),
        ..Options::default()
    };
    lzma_decompress_with_options(&mut &input[1..], &mut output, &options)
        .map_err(|_| Error::InvalidArchive("invalid or oversized LZMA stream"))?;
    Ok(output.bytes)
}

#[cfg(feature = "mpq-decode")]
struct LimitedOutput {
    bytes: Vec<u8>,
    limit: usize,
}

#[cfg(feature = "mpq-decode")]
impl IoWrite for LimitedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit - self.bytes.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "decompression exceeds declared size",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "mpq-decode")]
fn bounded(reader: impl Read, limit: usize) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    reader.take(limit as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(Error::InvalidArchive("decompression exceeds declared size"));
    }
    Ok(bytes)
}

#[cfg(feature = "mpq-decode")]
fn sparse(input: &[u8], limit: usize) -> Result<Vec<u8>, Error> {
    if input.len() < 4 {
        return Err(Error::InvalidArchive("sparse header"));
    }
    let size = u32::from_be_bytes(input[..4].try_into().unwrap()) as usize;
    if size > limit {
        return Err(Error::InvalidArchive("sparse output size"));
    }
    let mut output = Vec::with_capacity(size);
    let mut input = &input[4..];
    while output.len() < size {
        let (&tag, rest) = input
            .split_first()
            .ok_or(Error::InvalidArchive("truncated sparse stream"))?;
        input = rest;
        let count = if tag & 0x80 != 0 {
            (tag & 0x7f) as usize + 1
        } else {
            tag as usize + 3
        };
        let count = count.min(size - output.len());
        if tag & 0x80 != 0 {
            let chunk = input
                .get(..count)
                .ok_or(Error::InvalidArchive("truncated sparse literals"))?;
            output.extend_from_slice(chunk);
            input = &input[count..];
        } else {
            output.resize(output.len() + count, 0);
        }
    }
    Ok(output)
}

pub(super) fn check_encoder(compression: Compression) -> Result<(), Error> {
    if compression == Compression::Stored {
        return Ok(());
    }
    #[cfg(feature = "mpq-encode")]
    {
        Ok(())
    }
    #[cfg(not(feature = "mpq-encode"))]
    {
        Err(Error::FeatureDisabled("mpq-encode"))
    }
}

pub(super) fn encode(input: &[u8], compression: Compression) -> Result<Vec<u8>, Error> {
    if compression == Compression::Stored {
        return Ok(input.to_vec());
    }
    #[cfg(not(feature = "mpq-encode"))]
    {
        Err(Error::FeatureDisabled("mpq-encode"))
    }
    #[cfg(feature = "mpq-encode")]
    {
        use bzip2::write::BzEncoder;
        use bzip2::Compression as BzCompression;
        use flate2::write::ZlibEncoder;
        use flate2::Compression as ZlibCompression;
        let output = match compression {
            Compression::Stored => unreachable!(),
            Compression::Zlib => {
                let mut encoder = ZlibEncoder::new(vec![2], ZlibCompression::default());
                encoder.write_all(input)?;
                encoder.finish()?
            }
            Compression::Bzip2 => {
                let mut encoder = BzEncoder::new(vec![0x10], BzCompression::default());
                encoder.write_all(input)?;
                encoder.finish()?
            }
        };
        // MPQ identifies raw sectors by equality with the expected decoded size.
        Ok(if output.len() < input.len() {
            output
        } else {
            input.to_vec()
        })
    }
}

#[cfg(all(test, feature = "mpq-decode"))]
#[path = "codec_tests.rs"]
mod tests;
