//! Export the deterministic benchmark models and texture for GPU captures.
use mdl::Write as _;
use mdx::Write as _;
use std::{env, error::Error, fs, path::PathBuf};
use wc3::blp::{Blp, EncodeFormat, EncodeOptions};
use wc3::model::formats::mdlx::{mdl, mdx};

#[path = "../../../benchmarks/support/model.rs"]
mod models;
#[path = "../../../benchmarks/support/texture.rs"]
mod textures;

fn main() -> Result<(), Box<dyn Error>> {
    let output = env::args()
        .nth(1)
        .ok_or("usage: benchmark_fixtures OUTPUT_DIRECTORY")?;
    let output = PathBuf::from(output);
    fs::create_dir_all(output.join("synthetic"))?;
    let blp = Blp::encode_image(
        &textures::texture(256),
        EncodeOptions {
            format: EncodeFormat::Bgra,
            mipmaps: true,
            ..Default::default()
        },
    )?;
    fs::write(output.join("synthetic/diffuse.blp"), blp.write()?)?;
    for profile in models::profiles() {
        let classic = models::classic(profile);
        let modern = models::modern(profile);
        for (version, bytes, text) in [
            (800, classic.encode_mdx()?, classic.encode_mdl()?),
            (1800, modern.encode_mdx()?, modern.encode_mdl()?),
        ] {
            let name = format!("{}-{version}", profile.name);
            fs::write(output.join(format!("{name}.mdx")), bytes)?;
            fs::write(output.join(format!("{name}.mdl")), text)?;
        }
    }
    println!("{}", output.display());
    Ok(())
}
