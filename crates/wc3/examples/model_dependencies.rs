//! List direct declared file references and game-provided replaceable resources.
mod support;
use clap::Parser;
use std::collections::BTreeMap;
use std::path::PathBuf;
use support::{load, Result};
use wc3::model::resources::ResourceValue;

#[derive(Parser)]
#[command(about = "List direct model file references and replaceable resources")]
struct Args {
    /// Input MDX or MDL model.
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let model = load(&args.input)?;
    let mut files: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut replaceable = Vec::new();
    for reference in model.resources() {
        match reference.value {
            ResourceValue::Path(path) => {
                let text = path.text();
                if !text.is_empty() {
                    files
                        .entry(text.into_owned())
                        .or_default()
                        .push(reference.location.to_string());
                }
            }
            ResourceValue::ReplaceableId(id) => replaceable.push((id, reference.location)),
        }
    }
    for (path, locations) in files {
        println!("{path}\t{}", locations.join(", "));
    }
    for (id, location) in replaceable {
        println!("replaceable:{id}\t{location}");
    }
    Ok(())
}
