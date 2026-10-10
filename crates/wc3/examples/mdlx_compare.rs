//! Local converter and canonicalizer for tools/mdlx-compare. No oracle dependency.
use clap::{Parser, ValueEnum};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::exit;
use std::str::from_utf8;
use wc3::model::mdl::{Dialect, Read as _, Write as _};
use wc3::model::{visit_model, ConversionOptions, DynamicModel, Model, ModelDialect, V1200};
use wc3::model::{ModelVersion, NoExtensions};

#[derive(Parser)]
#[command(about = "Convert and canonicalize a model for mdlx-compare")]
struct Args {
    /// Input MDX or MDL model.
    input: PathBuf,
    /// Output canonical MDL file.
    output: PathBuf,
    /// Output MDL dialect.
    #[arg(value_enum)]
    dialect: OutputDialect,
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputDialect {
    Engine,
    Hive,
}

fn main() {
    if let Err(error) = run(Args::parse()) {
        eprintln!("{error}");
        exit(1);
    }
}

fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let dialect = match args.dialect {
        OutputDialect::Engine => Dialect::Warcraft3,
        OutputDialect::Hive => Dialect::HiveWorkshop,
    };
    let input = fs::read(&args.input)?;
    let model = if args.input.extension().is_some_and(|ext| ext == "mdl") {
        let source = from_utf8(&input)?;
        DynamicModel::<NoExtensions>::decode_mdl(source)
            .map_err(|error| format!("MDL parse: {}", error.diagnostic(source)))?
    } else {
        DynamicModel::<NoExtensions>::decode_mdx(&input, 800)
            .map_err(|error| format!("MDX decode: {error}"))?
    };
    // Reading text then writing it provides stable field spelling/order, float
    // formatting and defaults without ignoring IDs, flags, or track values.
    let text = visit_model!(&model, |typed| export_model(typed, dialect))?;
    fs::write(&args.output, text)?;
    Ok(())
}

fn export_model<V: ModelDialect<Extension = NoExtensions>>(
    model: &Model<V>,
    dialect: Dialect,
) -> Result<String, Box<dyn Error>> {
    if matches!(V::Version::NUMBER, 900 | 1000) {
        export_to::<V, V1200>(model, dialect)
    } else {
        export_to::<V, V>(model, dialect)
    }
}

fn export_to<
    S: ModelDialect<Extension = NoExtensions>,
    T: ModelDialect<Extension = NoExtensions>,
>(
    model: &Model<S>,
    dialect: Dialect,
) -> Result<String, Box<dyn Error>> {
    let converted = model.convert::<T>(&ConversionOptions::strict())?;
    for issue in &converted.report.issues {
        eprintln!(
            "conversion {:?} at {}: {}",
            issue.kind, issue.path, issue.description
        );
    }
    converted
        .model
        .encode_mdl_with_dialect(dialect)
        .map_err(|error| format!("MDL write: {error}").into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wc3::model::V1000;

    #[test]
    fn export_upgrades_old_reforged_versions_to_oracle_target() {
        let model =
            Model::<V1000>::decode_mdl("Version { FormatVersion 1000, } Model \"Minimal\" {}")
                .unwrap();
        let text = export_model(&model, Dialect::HiveWorkshop).unwrap();
        let restored = DynamicModel::<NoExtensions>::decode_mdl(&text).unwrap();
        assert_eq!(restored.version(), 1200);
        assert_eq!(model.version(), 1000);
    }

    #[test]
    fn export_does_not_discard_unsupported_material_shader_during_upgrade() {
        let model = Model::<V1000>::decode_mdl(
            "Version { FormatVersion 1000, } Model \"Minimal\" {} Materials 1 { Material { Shader \"UnsupportedShader\", } }",
        ).unwrap();
        let error = export_model(&model, Dialect::HiveWorkshop).unwrap_err();
        assert!(error.to_string().contains("shader"));
    }
}
