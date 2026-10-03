#![allow(dead_code)]
use mdl::{Read as _, Write as _};
use std::result::Result as StdResult;
use std::{error::Error, fs, path::Path, str::from_utf8};
use wc3::model::chunks::ModelChunk;
use wc3::model::{mdl, DynamicModel, FixedText, Model, ModelVersion};

pub type Result<T> = StdResult<T, Box<dyn Error>>;

pub fn load(path: &Path) -> Result<DynamicModel> {
    let bytes = fs::read(path)?;
    if bytes.starts_with(b"MDLX") {
        Ok(DynamicModel::decode_mdx(&bytes, 800)?)
    } else {
        let source = from_utf8(&bytes)?;
        DynamicModel::decode_mdl(source)
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

// Visit the actual chunks so duplicate chunks and Reforged records are retained.
pub fn paths<V: ModelVersion>(
    model: &mut Model<V>,
    mut visit: impl FnMut(String, &mut FixedText<260>) -> Result<()>,
) -> Result<()> {
    for (chunk_index, chunk) in model.chunks.iter_mut().enumerate() {
        let prefix = format!("chunk[{chunk_index}]");
        macro_rules! records {
            ($chunk:expr, $kind:literal) => {
                for (index, record) in $chunk.records.iter_mut().enumerate() {
                    visit(
                        format!("{prefix}.{}[{index}].path", $kind),
                        &mut record.path,
                    )?;
                }
            };
        }
        match chunk {
            ModelChunk::ModelInfo(chunk) => visit(
                format!("{prefix}.animation_file_name"),
                &mut chunk.info.animation_file_name,
            )?,
            ModelChunk::Textures(chunk) => records!(chunk, "textures"),
            ModelChunk::Attachments(chunk) => records!(chunk, "attachments"),
            ModelChunk::ParticleEmitters(chunk) => records!(chunk, "particle_emitters"),
            ModelChunk::PopcornEmitters(chunk) => records!(chunk, "popcorn_emitters"),
            ModelChunk::FaceFx(chunk) => records!(chunk, "face_fx"),
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wc3::model::chunks::TexturesChunk;
    use wc3::model::materials::Texture;
    use wc3::model::{visit_model, V800};

    #[test]
    fn path_edits_retain_duplicate_chunks_and_replaceable_ids() {
        let mut texture = Texture::new("Textures\\Shared.blp").unwrap();
        texture.replaceable_id = 1;
        let mut model = Model::<V800>::new();
        model
            .chunks
            .push(TexturesChunk::new(vec![texture.clone()]).into());
        model.chunks.push(TexturesChunk::new(vec![texture]).into());
        let mut locations = Vec::new();
        paths(&mut model, |location, path| {
            locations.push(location);
            path.set_text("Custom\\Shared.blp")?;
            Ok(())
        })
        .unwrap();
        assert_eq!(
            locations,
            ["chunk[1].textures[0].path", "chunk[2].textures[0].path"]
        );
        assert_eq!(model.chunks.len(), 3);
        for texture in model.textures() {
            assert_eq!(texture.path.text(), "Custom\\Shared.blp");
            assert_eq!(texture.replaceable_id, 1);
        }
    }

    #[test]
    fn visits_reforged_popcorn_paths() {
        let source = concat!(
            "Version { FormatVersion 1200, } Model \"Test\" {}\n",
            include_str!("../../tests/fixtures/mdl/popcorn_fire.mdl")
        );
        let mut model = DynamicModel::decode_mdl(source).unwrap();
        let mut found = Vec::new();
        visit_model!(&mut model, |typed| paths(typed, |location, path| {
            if !path.text().is_empty() {
                found.push((location, path.text().into_owned()));
            }
            Ok(())
        }))
        .unwrap();
        assert_eq!(found.len(), 1);
        assert!(found[0].0.ends_with("popcorn_emitters[0].path"));
        assert_eq!(found[0].1, "SharedFX\\Hero_Glow\\Hero_Glow.pkfx");
    }
}
