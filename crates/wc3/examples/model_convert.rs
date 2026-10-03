//! Convert model formats and optionally versions, reporting conversion losses.
mod support;
use clap::Parser;
use std::path::PathBuf;
use std::result::Result as ParseResult;
use support::{load, save, Result};
use wc3::model::{
    ConversionOptions, DynamicModel, Model, ModelVersion, V1000, V1100, V1200, V1300, V1400, V1600,
    V1800, V800, V900,
};

#[derive(Parser)]
#[command(about = "Convert MDX/MDL formats and optionally model versions")]
struct Args {
    /// Input MDX or MDL model.
    input: PathBuf,
    /// Output model; its .mdx or .mdl extension selects the format.
    output: PathBuf,
    /// Target version (800, 900, 1000, 1100, 1200, 1300, 1400, 1600, or 1800).
    #[arg(long, value_parser = parse_version)]
    version: Option<u32>,
    /// Allow version-conversion losses and report them to stderr.
    #[arg(long)]
    lossy: bool,
}

fn parse_version(value: &str) -> ParseResult<u32, String> {
    match value.parse() {
        Ok(version @ (800 | 900 | 1000 | 1100 | 1200 | 1300 | 1400 | 1600 | 1800)) => Ok(version),
        _ => Err("expected a supported model version: 800, 900, 1000, 1100, 1200, 1300, 1400, 1600, or 1800".into()),
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let options = if args.lossy {
        ConversionOptions::lossy()
    } else {
        ConversionOptions::strict()
    };
    let model = load(&args.input)?;
    macro_rules! convert {
        ($($number:literal => $version:ident),*) => {
            match args.version {
                None => model,
                $(Some($number) => DynamicModel::$version(convert::<$version>(&model, &options)?),)*
                Some(value) => return Err(format!("unsupported target version: {value}").into()),
            }
        };
    }
    let model = convert!(800 => V800, 900 => V900, 1000 => V1000, 1100 => V1100, 1200 => V1200, 1300 => V1300, 1400 => V1400, 1600 => V1600, 1800 => V1800);
    save(&model, &args.output)
}

fn convert<V: ModelVersion>(model: &DynamicModel, options: &ConversionOptions) -> Result<Model<V>> {
    let converted = model.convert::<V>(options)?;
    for issue in converted.report.issues {
        eprintln!("{:?} at {}: {}", issue.kind, issue.path, issue.description);
    }
    Ok(converted.model)
}
