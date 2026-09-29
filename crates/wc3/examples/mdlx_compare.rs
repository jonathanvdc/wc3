//! Local converter and canonicalizer for tools/mdlx-compare. No oracle dependency.
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::exit;
use std::str::from_utf8;
use wc3::model::mdl::{Dialect, Read as _, Write as _};
use wc3::model::{visit_model, ConversionOptions, DynamicModel, Model, ModelVersion};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 4 {
        return Err("usage: mdlx_compare INPUT OUTPUT engine|hive".into());
    }
    let dialect = match args[3].as_str() {
        "engine" => Dialect::Warcraft3,
        "hive" => Dialect::HiveWorkshop,
        _ => return Err("invalid dialect".into()),
    };
    let input = fs::read(&args[1])?;
    let model = if Path::new(&args[1])
        .extension()
        .is_some_and(|ext| ext == "mdl")
    {
        let source = from_utf8(&input)?;
        DynamicModel::decode_mdl(source)
            .map_err(|error| format!("MDL parse: {}", error.diagnostic(source)))?
    } else {
        DynamicModel::decode_mdx(&input, 800).map_err(|error| format!("MDX decode: {error}"))?
    };
    // Reading text then writing it provides stable field spelling/order, float
    // formatting and defaults without ignoring IDs, flags, or track values.
    let text = visit_model!(&model, |typed| export_model(typed, dialect))?;
    fs::write(&args[2], text)?;
    Ok(())
}

fn export_model<V: ModelVersion>(
    model: &Model<V>,
    dialect: Dialect,
) -> Result<String, Box<dyn Error>> {
    let converted = model.convert::<V>(&ConversionOptions::strict())?;
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
    use wc3::model::mdx::Write as _;
    use wc3::model::scene::{Camera, CameraVariant};
    use wc3::model::V1800;

    #[test]
    fn export_normalizes_camera_without_mutating_binary_source() {
        let mut model =
            Model::<V1800>::decode_mdl("Version { FormatVersion 1800, } Model \"Minimal\" {}")
                .unwrap();
        let mut camera = Camera::<V1800>::new("Portrait").unwrap();
        camera.variant = CameraVariant::Variant0;
        model.set_cameras(&[camera]);
        let original = model.encode_mdx().unwrap();
        assert!(model.encode_mdl().is_err());
        let text = export_model(&model, Dialect::HiveWorkshop).unwrap();
        let restored = Model::<V1800>::decode_mdl(&text).unwrap();
        assert_eq!(restored.cameras()[0].variant, CameraVariant::Variant3);
        assert_eq!(model.encode_mdx().unwrap(), original);
    }
}
