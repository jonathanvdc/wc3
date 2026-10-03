//! Decode a selected BLP mipmap and export it as PNG.
mod support;
use clap::{value_parser, Parser};
use image::{codecs::png::PngEncoder, ExtendedColorType, ImageEncoder};
use std::path::PathBuf;
use std::{
    fs,
    io::{BufWriter, Write as IoWrite},
};
use support::Result;
use wc3::blp::BlpRef;

#[derive(Parser)]
#[command(about = "Inspect a BLP texture and export a mipmap as PNG")]
struct Args {
    /// Input BLP1 or BLP2 texture.
    input: PathBuf,
    /// Output PNG file.
    output: PathBuf,
    /// Mipmap level to decode.
    #[arg(long, default_value_t = 0, value_parser = value_parser!(u8).range(0..16))]
    mip: u8,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let level = usize::from(args.mip);
    let bytes = fs::read(&args.input)?;
    let blp = BlpRef::read(&bytes)?;
    match &blp {
        BlpRef::Blp1(value) => println!(
            "BLP1: {:?}\nMip levels: {}",
            value.header,
            value.mipmaps.iter().flatten().count()
        ),
        BlpRef::Blp2(value) => println!(
            "BLP2: {:?}\nMip levels: {}",
            value.header,
            value.mipmaps.iter().flatten().count()
        ),
    }
    let pixels = blp.decode_mip(level)?;
    println!("Mip {level}: {}x{}", pixels.width(), pixels.height());
    let mut output = BufWriter::new(fs::File::create(&args.output)?);
    PngEncoder::new(&mut output).write_image(
        pixels.as_raw(),
        pixels.width(),
        pixels.height(),
        ExtendedColorType::Rgba8,
    )?;
    output.flush()?;
    Ok(())
}
