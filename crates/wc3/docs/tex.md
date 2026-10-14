# Diablo texture containers

The `wc3::tex` module separates native storage from optional pixel conversion, as `wc3::blp` does. `d3::Texture<'a, d3::V47>` reads Diablo III revision 47, whose root contains up to sixty mip entries and optional atlas frames. Encoded mip payloads and frame hashes borrow their input through `Cow`; named header fields can be edited without copying them. `as_ref` creates borrowed views and `into_owned` produces storage independent of the input.

`d4::Texture<'a, d4::HashF9cd83e7>` reads the established 120-byte Diablo IV TextureDefinition layout. Its metadata contains serialized external pixel ranges rather than embedded mip data. Parsing does not open files or assume a storage provider. Each `PayloadRange` resolves against a caller-selected, header-stripped payload; `payload_body` checks a full SNO payload header when an extractor retains it. Callers select the correct high or low payload and native slot explicitly. The codec does not guess a streaming split from offset resets, and it does not assume that hash F9CD83E6 has the same layout.

Writers rebuild offsets for arrays and embedded D3 pixels while retaining reserved native fields. Serialized runtime pointer slots must be zero and are regenerated as zero. D4 external pixel ranges retain their offsets because they identify independent resources; writing metadata never silently rewrites or discards those resources. D4 atlas frames and spherical-harmonic coefficients retain typed borrowed arrays. Known and unknown native pixel format identifiers survive container reads and writes independently of pixel conversion support.

## Pixel conversion

The `tex-decode` feature converts supported mip payloads to `image::RgbaImage`. D3 supports BGRA8, A4R4G4B4, luminance, alpha, BC1, BC2, BC3, and BC5. D4 supports established BGRA8/RGBA8, R8, alpha, BC1 through BC5, and BC7 identifiers, including their known alternate and sRGB identifiers. Decoding preserves numerical channel values; it does not linearize sRGB. HDR BC6H and float pixel formats return an unsupported-format error rather than losing their range through an implicit RGBA8 conversion.

The `tex-encode` feature encodes BGRA8, packed A4R4G4B4, luminance, alpha, and BC1/2/3 for D3; D4 additionally accepts RGBA8 and writes native 256-byte row padding. Encoded D3 BC fields use the native planar shuffle and per-face prefix. Block compression uses `squish`; extended BC decoding and BC2 alpha expansion use `texture2ddecoder`. No new compression algorithm is implemented here. Encoding a mip returns bytes for explicit attachment rather than generating an entire mip chain or choosing a D4 streaming policy.

Pixel conversion validates dimensions, byte counts, arithmetic, and allocations independently from container parsing. Native containers can therefore preserve unfamiliar formats without claiming that their pixels can be decoded. D4 volume storage is retained as metadata but its pixel conversion is rejected. Cubemap faces can be selected individually; conversion does not infer cube orientation or claim game-rendering fidelity.

Compatibility with a real-game asset corpus and game-rendering fidelity remain unverified.

## Native block storage

The D3 block transformation permutes bytes between native field planes and
standard BC blocks. Compression and decompression remain library operations.
