#![allow(dead_code)]
use mdl::{Read as _, Write as _};
use std::result::Result as StdResult;
use std::{error::Error, fs, path::Path, str::from_utf8};
use wc3::model::NoExtensions;
use wc3::model::{mdl, DynamicModel};

pub type Result<T> = StdResult<T, Box<dyn Error>>;

pub fn load(path: &Path) -> Result<DynamicModel> {
    let bytes = fs::read(path)?;
    if bytes.starts_with(b"MDLX") {
        Ok(DynamicModel::<NoExtensions>::decode_mdx(&bytes, 800)?)
    } else {
        let source = from_utf8(&bytes)?;
        DynamicModel::<NoExtensions>::decode_mdl(source)
            .map_err(|error| error.diagnostic(source).to_string().into())
    }
}

pub fn save(model: &DynamicModel, path: &Path) -> Result<()> {
    match path.extension().and_then(|value| value.to_str()) {
        Some(extension) if extension.eq_ignore_ascii_case("mdx") => {
            fs::write(path, model.encode_mdx()?)?
        }
        Some(extension) if extension.eq_ignore_ascii_case("mdl") => {
            fs::write(path, model.encode_mdl()?)?
        }
        _ => return Err("output must have an .mdx or .mdl extension".into()),
    }
    Ok(())
}
