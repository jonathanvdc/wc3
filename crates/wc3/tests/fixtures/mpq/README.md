These tiny archives were generated with [StormLib 9.40](https://github.com/ladislav-zezula/StormLib)
using `generate.cpp`. They contain only synthetic data:

- `Units\Encrypted.bin`: 17,003 bytes, byte `i` is `i % 7`, zlib compressed,
  encrypted with the offset-adjusted key, in 4,096-byte sectors.
- `stored.bin`: the seven bytes `payload`, stored without compression.

The archives use on-disk versions 1, 2, and 3 (MPQ v2, v3, and v4).
V3/V4 contain both classic and HET/BET tables. V4 enables 1,024-byte raw chunks.
There are no listfiles, attributes, signatures, or game assets.

Build `generate.cpp` against StormLib, then run `generate PATH_TO_THIS_DIRECTORY`
to regenerate the fixtures. `generate verify ARCHIVE` independently checks an
archive containing the encrypted payload above, including raw file and HET/BET
MD5s. StormLib is needed for regeneration, not for running the Rust tests.
