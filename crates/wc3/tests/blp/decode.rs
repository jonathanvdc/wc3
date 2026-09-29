use std::{env, fs, path::Path};
use wc3::blp::{
    Blp1ContentRef, Blp1Header, Blp1Ref, Blp2ContentRef, Blp2Header, Blp2Ref, BlpDecoder, BlpRef,
    DxtFormat,
};

#[test]
fn image_decoder_integrates_with_dynamic_image() {
    use image::{ColorType, DynamicImage, ImageDecoder};

    let palette = [0; 1024];
    let blp = blp2(
        &[3, 2, 1, 4],
        Blp2ContentRef::Bgra {
            encoding: 3,
            alpha_type: 8,
            palette_region: &palette,
        },
        8,
        1,
        1,
    );
    let bytes = blp.write().unwrap();
    let decoder = BlpDecoder::new(&bytes).unwrap();
    assert_eq!(decoder.dimensions(), (1, 1));
    assert_eq!(decoder.color_type(), ColorType::Rgba8);
    assert_eq!(
        DynamicImage::from_decoder(decoder)
            .unwrap()
            .to_rgba8()
            .into_raw(),
        [1, 2, 3, 4]
    );
    assert!(BlpDecoder::with_mip(&bytes, 1).is_err());
}

fn blp1<'a>(
    data: &'a [u8],
    content: Blp1ContentRef<'a>,
    alpha_bits: u32,
    width: u32,
) -> Blp1Ref<'a> {
    let mut mipmaps = [None; 16];
    mipmaps[0] = Some(data);
    Blp1Ref {
        header: Blp1Header {
            width,
            height: 1,
            alpha_bits,
            extra: 5,
            has_mipmaps: 0,
        },
        content,
        mipmaps,
    }
}

fn blp2<'a>(
    data: &'a [u8],
    content: Blp2ContentRef<'a>,
    alpha_bits: u8,
    width: u32,
    height: u32,
) -> Blp2Ref<'a> {
    let mut mipmaps = [None; 16];
    mipmaps[0] = Some(data);
    Blp2Ref {
        header: Blp2Header {
            width,
            height,
            alpha_bits,
            mipmap_flags: 0,
        },
        content,
        mipmaps,
    }
}

#[test]
fn bgra_swaps_channels_and_preserves_alpha() {
    let palette = [0; 1024];
    let blp = blp2(
        &[3, 2, 1, 4, 30, 20, 10, 40],
        Blp2ContentRef::Bgra {
            encoding: 3,
            alpha_type: 8,
            palette_region: &palette,
        },
        8,
        2,
        1,
    );
    assert_eq!(
        blp.decode_mip(0).unwrap().into_raw(),
        [1, 2, 3, 4, 10, 20, 30, 40]
    );
    assert!(blp.decode_mip(1).is_err());
}

#[test]
fn indexed_alpha_depths_and_palette_order() {
    let mut palette = [0; 1024];
    palette[..8].copy_from_slice(&[3, 2, 1, 0, 30, 20, 10, 0]);
    let content = Blp1ContentRef::Indexed { palette: &palette };
    for (bits, data, alpha) in [
        (0, vec![0, 1, 0], [255, 255, 255]),
        (1, vec![0, 1, 0, 0b101], [255, 0, 255]),
        (4, vec![0, 1, 0, 0x80, 0x0f], [0, 136, 255]),
        (8, vec![0, 1, 0, 7, 128, 255], [7, 128, 255]),
    ] {
        let image = blp1(&data, content, bits, 3).decode_mip(0).unwrap();
        assert_eq!(image.get_pixel(0, 0).0, [1, 2, 3, alpha[0]]);
        assert_eq!(image.get_pixel(1, 0).0, [10, 20, 30, alpha[1]]);
        assert_eq!(image.get_pixel(2, 0).0, [1, 2, 3, alpha[2]]);
    }
    let blp = blp2(
        &[1, 0x0f],
        Blp2ContentRef::Indexed {
            alpha_type: 4,
            palette: &palette,
        },
        4,
        1,
        1,
    );
    assert_eq!(
        blp.decode_mip(0).unwrap().get_pixel(0, 0).0,
        [10, 20, 30, 255]
    );
}

#[test]
fn dxt1_color_and_binary_alpha() {
    let palette = [0; 1024];
    let block = [0, 0, 0xff, 0xff, 0b11_10_01_00, 0, 0, 0];
    let blp = blp2(
        &block,
        Blp2ContentRef::Dxt {
            format: DxtFormat::Dxt1,
            palette_region: &palette,
        },
        1,
        4,
        4,
    );
    let image = blp.decode_mip(0).unwrap();
    assert_eq!(image.get_pixel(0, 0).0, [0, 0, 0, 255]);
    assert_eq!(image.get_pixel(1, 0).0, [255, 255, 255, 255]);
    assert_eq!(image.get_pixel(2, 0).0, [127, 127, 127, 255]);
    assert_eq!(image.get_pixel(3, 0).0, [0, 0, 0, 0]);
}

#[test]
fn dxt3_and_dxt5_decode_alpha_and_clip_edge_blocks() {
    let palette = [0; 1024];
    let mut bc2 = [0; 16];
    bc2[0] = 0xf0;
    bc2[8..12].copy_from_slice(&[0, 0xf8, 0, 0]);
    let blp = blp2(
        &bc2,
        Blp2ContentRef::Dxt {
            format: DxtFormat::Dxt3,
            palette_region: &palette,
        },
        4,
        2,
        1,
    );
    let image = blp.decode_mip(0).unwrap();
    assert_eq!(image.get_pixel(0, 0).0, [255, 0, 0, 0]);
    assert_eq!(image.get_pixel(1, 0).0, [255, 0, 0, 255]);

    let mut bc3 = [0; 16];
    bc3[..3].copy_from_slice(&[0, 255, 0b111_110]);
    bc3[8..12].copy_from_slice(&[0, 0xf8, 0, 0]);
    let blp = blp2(
        &bc3,
        Blp2ContentRef::Dxt {
            format: DxtFormat::Dxt5,
            palette_region: &palette,
        },
        8,
        2,
        1,
    );
    let image = blp.decode_mip(0).unwrap();
    assert_eq!(image.get_pixel(0, 0).0, [255, 0, 0, 0]);
    assert_eq!(image.get_pixel(1, 0).0, [255, 0, 0, 255]);
}

#[test]
#[ignore = "requires WC3_FIXTURES"]
fn local_blp_corpus_decodes_all_present_mips() {
    fn visit(path: &Path, count: &mut usize) {
        let Ok(entries) = fs::read_dir(path) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit(&path, count);
            } else if path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("blp"))
            {
                let bytes = fs::read(&path).unwrap();
                let blp = BlpRef::read(&bytes)
                    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
                for level in 0..16 {
                    let present = match &blp {
                        BlpRef::Blp1(value) => value.mipmaps[level].is_some(),
                        BlpRef::Blp2(value) => value.mipmaps[level].is_some(),
                    };
                    if present {
                        blp.decode_mip(level).unwrap_or_else(|error| {
                            panic!("{} mip {level}: {error}", path.display())
                        });
                    }
                }
                *count += 1;
            }
        }
    }
    let directory = env::var("WC3_FIXTURES").expect("set WC3_FIXTURES to a model directory");
    let mut count = 0;
    visit(Path::new(&directory), &mut count);
    assert!(count > 0, "WC3_FIXTURES contains no BLP files");
}
