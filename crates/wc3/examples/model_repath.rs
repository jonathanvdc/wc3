//! Rewrite declared external paths with a literal prefix replacement.
mod support;
use clap::{builder::NonEmptyStringValueParser, Parser};
use std::convert::Infallible;
use std::path::PathBuf;
use support::{load, save, Result};
use wc3::model::resources::{ResourceEdit, ResourceValue};

#[derive(Parser)]
#[command(about = "Rewrite declared model paths with a case-sensitive prefix replacement")]
struct Args {
    /// Input MDX or MDL model.
    input: PathBuf,
    /// Output model; its .mdx or .mdl extension selects the format.
    output: PathBuf,
    /// Nonempty prefix to replace.
    #[arg(value_parser = NonEmptyStringValueParser::new())]
    from_prefix: String,
    /// Replacement prefix; may be empty.
    to_prefix: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut model = load(&args.input)?;
    let report = model.rewrite_resources(|reference| {
        let edit = match reference.value {
            ResourceValue::Path(path) => {
                let old = path.text();
                match old.strip_prefix(&args.from_prefix) {
                    Some(suffix) => ResourceEdit::SetPath(format!("{}{suffix}", args.to_prefix)),
                    None => ResourceEdit::Keep,
                }
            }
            ResourceValue::ReplaceableId(_) => ResourceEdit::Keep,
        };
        Ok::<_, Infallible>(edit)
    })?;
    for change in &report.changes {
        eprintln!("{}: {} -> {}", change.location, change.before, change.after);
    }
    save(&model, &args.output)?;
    eprintln!("Rewrote {} paths", report.changes.len());
    Ok(())
}
