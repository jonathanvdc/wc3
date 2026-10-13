//! Common material values presented alongside native profile bodies.
use super::{native, nr, read_error, revision, string, Error};
use crate::model::conversion::Context;
use crate::model::wem::semantic as sem;
use native::{Envelope, Ref};
use std::borrow::Cow;
pub(in crate::model::conversion::m3) fn import_semantic<E: Envelope>(
    source: &native::Model<'_, E>,
    model: &mut sem::Model<'static>,
    context: &mut Context,
) -> Result<(), Error> {
    let root = source.root_record().map_err(read_error)?;
    let document = source.document();
    let hair = native::visit_m3_root!(root, |r| r.hair_materials);
    if hair.entries != 0 {
        context.unsupported(
            "m3.hair_materials",
            "WEM has no native hair-material body; populated hair shaders cannot be represented",
        )?;
    }
    let (maps_ref, standard_ref) =
        native::visit_m3_root!(root, |r| (r.material_maps, r.standard_materials));
    let maps = document
        .resolve::<nr::MaterialMap>(maps_ref.cast())
        .map_err(read_error)?
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(read_error)?;
    let mut standard = Vec::new();
    macro_rules! materials {
        ($v:ty) => {
            for row in document
                .resolve::<nr::StandardMaterial<$v>>(standard_ref.cast())
                .map_err(read_error)?
                .iter()
            {
                let m = row.map_err(read_error)?;
                standard.push((m.name, m.flags, m.blend_mode, m.diffuse_layer));
            }
        };
    }
    if standard_ref.entries != 0 {
        match revision(document, standard_ref).map_err(read_error)? {
            0 => materials!(nr::V0),
            1 => materials!(nr::V1),
            2 => materials!(nr::V2),
            3 => materials!(nr::V3),
            4 => materials!(nr::V4),
            5 => materials!(nr::V5),
            6 => materials!(nr::V6),
            7 => materials!(nr::V7),
            8 => materials!(nr::V8),
            9 => materials!(nr::V9),
            10 => materials!(nr::V10),
            11 => materials!(nr::V11),
            12 => materials!(nr::V12),
            13 => materials!(nr::V13),
            14 => materials!(nr::V14),
            15 => materials!(nr::V15),
            16 => materials!(nr::V16),
            17 => materials!(nr::V17),
            18 => materials!(nr::V18),
            19 => materials!(nr::V19),
            20 => materials!(nr::V20),
            _ => {
                return Err(Error::new(
                    "m3.materials",
                    "unsupported standard material revision",
                ));
            }
        }
    }
    model.materials.clear();
    for (index, map) in maps.into_iter().enumerate() {
        let path = format!("m3.materials[{index}]");
        if map.material_type != nr::MaterialKind::Standard {
            if matches!(
                map.material_type,
                nr::MaterialKind::Hair | nr::MaterialKind::Unknown(_)
            ) {
                context.unsupported(&path, "native shader family has no WEM body mapping")?;
            }
            model.materials.push(sem::Material::default());
            continue;
        }
        let (name, flags, blend, diffuse) = *standard
            .get(map.material_index as usize)
            .ok_or_else(|| Error::new(&path, "material map index out of range"))?;
        let texture = if diffuse.entries != 0 {
            let path = texture_path(document, diffuse).map_err(read_error)?;
            let texture = model.textures.len() as u32;
            model.textures.push(sem::Texture {
                path: Cow::Owned(path),
                ..Default::default()
            });
            Some(texture)
        } else {
            None
        };
        model.materials.push(sem::Material {
            name: Cow::Owned(string(document, name).map_err(read_error)?),
            flags: super::render_state::flags(flags),
            blend: super::render_state::blend(blend),
            texture,
        });
    }
    Ok(())
}
fn texture_path<E: Envelope>(
    document: &native::Document<'_, E>,
    reference: Ref<nr::targets::TextureLayer>,
) -> Result<String, native::ReadError> {
    macro_rules! read {
        ($v:ty) => {{
            let row = document
                .resolve::<nr::TextureLayer<$v>>(reference.cast())?
                .get(0)
                .ok_or_else(|| native::ReadError::invalid(0, "empty diffuse layer"))??;
            string(document, row.texture_path)
        }};
    }
    match revision(document, reference)? {
        0 => read!(nr::V0),
        1 => read!(nr::V1),
        2 => read!(nr::V2),
        3 => read!(nr::V3),
        4 => read!(nr::V4),
        5 => read!(nr::V5),
        6 => read!(nr::V6),
        7 => read!(nr::V7),
        8 => read!(nr::V8),
        9 => read!(nr::V9),
        10 => read!(nr::V10),
        11 => read!(nr::V11),
        12 => read!(nr::V12),
        13 => read!(nr::V13),
        14 => read!(nr::V14),
        15 => read!(nr::V15),
        16 => read!(nr::V16),
        17 => read!(nr::V17),
        18 => read!(nr::V18),
        19 => read!(nr::V19),
        20 => read!(nr::V20),
        21 => read!(nr::V21),
        22 => read!(nr::V22),
        23 => read!(nr::V23),
        24 => read!(nr::V24),
        25 => read!(nr::V25),
        26 => read!(nr::V26),
        _ => Err(native::ReadError::invalid(
            0,
            "unsupported texture layer revision",
        )),
    }
}
