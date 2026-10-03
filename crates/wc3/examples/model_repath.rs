//! Rewrite declared external paths with a literal prefix replacement.
mod support;
use clap::{builder::NonEmptyStringValueParser, Parser};
use std::path::PathBuf;
use support::{load, paths, save, Result};
use wc3::model::visit_model;

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
    let mut count = 0;
    visit_model!(&mut model, |typed| paths(typed, |location, path| {
        let old = path.text().into_owned();
        if let Some(suffix) = old.strip_prefix(&args.from_prefix) {
            let new = format!("{}{suffix}", args.to_prefix);
            path.set_text(&new)?;
            eprintln!("{location}: {old} -> {new}");
            count += 1;
        }
        Ok(())
    }))?;
    save(&model, &args.output)?;
    eprintln!("Rewrote {count} paths");
    Ok(())
}
