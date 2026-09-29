//! Construct version-specific BLP containers from encoded mipmaps.
use super::{BlpVersion, EncodeFormat};
use crate::blp::{
    Blp, Blp1, Blp1Content, Blp1Header, Blp2, Blp2Content, Blp2Header, DxtFormat, MIPMAP_SLOTS,
    PALETTE_BYTES,
};

pub(super) fn container(
    width: u32,
    height: u32,
    format: EncodeFormat,
    palette: Option<Box<[u8; PALETTE_BYTES]>>,
    mipmaps: [Option<Vec<u8>>; MIPMAP_SLOTS],
) -> Blp {
    let has_mipmaps = mipmaps[1].is_some();
    let alpha_bits = format.alpha_bits();
    match format {
        EncodeFormat::Indexed {
            version: BlpVersion::Blp1,
            ..
        }
        | EncodeFormat::Jpeg {
            version: BlpVersion::Blp1,
            ..
        } => Blp::Blp1(Blp1 {
            header: Blp1Header {
                width,
                height,
                alpha_bits: alpha_bits.into(),
                extra: 5,
                has_mipmaps: u32::from(has_mipmaps),
            },
            content: match palette {
                Some(palette) => Blp1Content::Indexed { palette },
                None => Blp1Content::Jpeg {
                    shared_header: Vec::new(),
                },
            },
            mipmaps,
        }),
        _ => Blp::Blp2(Blp2 {
            header: Blp2Header {
                width,
                height,
                alpha_bits,
                mipmap_flags: u8::from(has_mipmaps),
            },
            content: match format {
                EncodeFormat::Indexed { .. } => Blp2Content::Indexed {
                    alpha_type: alpha_bits,
                    palette: palette.expect("indexed palette"),
                },
                EncodeFormat::Jpeg { .. } => Blp2Content::Jpeg {
                    alpha_type: 0,
                    shared_header: Vec::new(),
                    unused: vec![0; PALETTE_BYTES - 4],
                },
                EncodeFormat::Dxt1 { .. } => dxt_content(DxtFormat::Dxt1),
                EncodeFormat::Dxt3 => dxt_content(DxtFormat::Dxt3),
                EncodeFormat::Dxt5 => dxt_content(DxtFormat::Dxt5),
                EncodeFormat::Bgra => Blp2Content::Bgra {
                    encoding: 3,
                    alpha_type: 0,
                    palette_region: Box::new([0; PALETTE_BYTES]),
                },
            },
            mipmaps,
        }),
    }
}

fn dxt_content(format: DxtFormat) -> Blp2Content {
    Blp2Content::Dxt {
        format,
        palette_region: Box::new([0; PALETTE_BYTES]),
    }
}
