//! Export a selected Diablo TEX image without choosing a storage provider.
use clap::{Parser, ValueEnum};
use image::{codecs::png::PngEncoder, ExtendedColorType, ImageEncoder};
use std::{
    error::Error,
    fs,
    io::{BufWriter, Write as _},
    path::PathBuf,
};
use wc3::tex::{d3, d4};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Game {
    D3,
    D4,
}
#[derive(Parser)]
#[command(about = "Export a supported Diablo III or IV TEX mip image as PNG")]
struct Args {
    /// Native texture document including its SNO envelope.
    input: PathBuf,
    /// Destination PNG file.
    output: PathBuf,
    /// Resource namespace.
    #[arg(long, value_enum)]
    game: Game,
    /// Mip level; D4 ranges require an explicit level in the original texture.
    #[arg(long, default_value_t = 0)]
    mip: u32,
    /// D3 cubemap face index.
    #[arg(long, default_value_t = 0)]
    face: usize,
    /// D4 payload file chosen by the caller, with any SNO header removed.
    #[arg(long)]
    payload: Option<PathBuf>,
    /// Native D4 serialized pixel-range slot.
    #[arg(long, default_value_t = 0)]
    range: usize,
}
fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let bytes = fs::read(args.input)?;
    let pixels = match args.game {
        Game::D3 => {
            if args.payload.is_some() {
                return Err("--payload applies to D4 textures".into());
            }
            let texture = d3::Texture::<d3::V47>::read(&bytes)?;
            println!(
                "D3 V47: {}x{}, format {}",
                texture.width(),
                texture.height(),
                texture.format()
            );
            texture.decode_mip(usize::try_from(args.mip)?, args.face)?
        }
        Game::D4 => {
            if args.face != 0 {
                return Err("D4 selects a native payload range instead of --face".into());
            }
            let texture = d4::Texture::<d4::HashF9cd83e7>::read(&bytes)?;
            let path = args
                .payload
                .ok_or("D4 requires --payload with a header-stripped pixel payload")?;
            let payload = fs::read(path)?;
            println!(
                "D4 F9CD83E7: {}x{}, format {}",
                texture.width(),
                texture.height(),
                texture.format()
            );
            texture.decode_range(args.range, args.mip, &payload)?
        }
    };
    let mut output = BufWriter::new(fs::File::create(args.output)?);
    PngEncoder::new(&mut output).write_image(
        pixels.as_raw(),
        pixels.width(),
        pixels.height(),
        ExtendedColorType::Rgba8,
    )?;
    output.flush()?;
    Ok(())
}
