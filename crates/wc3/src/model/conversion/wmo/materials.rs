//! Native material fields and explicit resource identities.
use super::*;
type MaterialFields = Vec<(String, i64, String)>;
type MaterialOutput<D> = (Vec<nr::Material<D>>, Option<Vec<u8>>);
pub(super) fn import<D: Dialect>(
    root: &Root<'_, D>,
    model: &mut s::Model<'static>,
) -> Result<Vec<MaterialFields>, Error> {
    let rows = root
        .chunks()
        .filter_map(|chunk| {
            if let RootChunk::Materials(rows) = chunk {
                Some(collect(rows, "materials"))
            } else {
                None
            }
        })
        .collect::<Result<Vec<_>, _>>()?
        .concat();
    let mut metadata_rows = Vec::new();
    for (index, material) in rows.iter().enumerate() {
        let mut textures = Vec::new();
        let slots = if matches!(material.shader, 22..=24) {
            9
        } else {
            3
        };
        for slot in 0..slots {
            let texture = root
                .texture_reference(index, slot)
                .map_err(|error| failure(format!("materials[{index}].textures[{slot}]"), error))?;
            let mapped = match texture {
                Some(native::TextureReference::Path(path)) => {
                    let path = from_utf8(path)
                        .map_err(|error| failure("texture.path", error))?
                        .to_owned();
                    let value = s::Texture {
                        path: Cow::Owned(path),
                        ..Default::default()
                    };
                    let index = model
                        .textures
                        .iter()
                        .position(|existing| existing == &value)
                        .unwrap_or_else(|| {
                            model.textures.push(value);
                            model.textures.len() - 1
                        });
                    Some(index as u32)
                }
                Some(native::TextureReference::FileId(id)) => {
                    let value = s::Texture {
                        id: Some(id),
                        ..Default::default()
                    };
                    let index = model
                        .textures
                        .iter()
                        .position(|existing| existing == &value)
                        .unwrap_or_else(|| {
                            model.textures.push(value);
                            model.textures.len() - 1
                        });
                    Some(index as u32)
                }
                None => None,
            };
            textures.push(mapped.unwrap_or(u32::MAX));
        }
        model.materials.push(s::Material {
            name: Cow::Owned(format!("Material {index}")),
            texture: textures.first().copied().filter(|value| *value != u32::MAX),
            blend: material.blend_mode,
            flags: material.flags & 3,
        });
        metadata_rows.push(vec![
            (
                "wmo.material.flags".into(),
                i64::from(material.flags),
                String::new(),
            ),
            (
                "wmo.material.shader".into(),
                i64::from(material.shader),
                String::new(),
            ),
            (
                "wmo.material.sidn_color".into(),
                i64::from(u32::from_le_bytes(material.sidn_color)),
                String::new(),
            ),
            (
                "wmo.material.frame_sidn_color".into(),
                i64::from(u32::from_le_bytes(material.frame_sidn_color)),
                String::new(),
            ),
            (
                "wmo.material.diffuse_color".into(),
                i64::from(u32::from_le_bytes(material.diffuse_color)),
                String::new(),
            ),
            (
                "wmo.material.ground_type".into(),
                i64::from(material.ground_type),
                String::new(),
            ),
            ("wmo.material.textures".into(), 0, metadata::words(textures)),
            (
                "wmo.material.tail".into(),
                0,
                metadata::words(material.tail.texture_slots()),
            ),
        ]);
    }
    Ok(metadata_rows)
}
pub(super) fn export<D: Dialect>(
    model: &s::Model<'_>,
    native_fields: &[BTreeMap<String, (i64, String)>],
    context: &mut Context,
) -> Result<MaterialOutput<D>, Error> {
    let uses_names = model
        .textures
        .iter()
        .any(|texture| !texture.path.is_empty());
    if uses_names
        && model
            .textures
            .iter()
            .any(|texture| texture.id.is_some() && texture.path.is_empty())
    {
        return Err(Error::new(
            "textures",
            "WMO MOTX and FileDataID material addressing cannot be mixed without a resource mapping",
        ));
    }
    let mut names = vec![0];
    let mut references = Vec::new();
    for (index, texture) in model.textures.iter().enumerate() {
        if texture.replaceable != 0 {
            context.unsupported(
                format!("textures[{index}].replaceable"),
                "WMO has no Warcraft III replaceable texture category",
            )?;
        }
        if uses_names {
            let offset = u32::try_from(names.len()).map_err(|error| failure("textures", error))?;
            names.extend_from_slice(texture.path.as_bytes());
            names.push(0);
            references.push(offset);
        } else {
            references.push(texture.id.unwrap_or(0));
        }
    }
    let mut output = Vec::new();
    for (index, material) in model.materials.iter().enumerate() {
        let default = BTreeMap::new();
        let fields = native_fields.get(index).unwrap_or(&default);
        let mut value = nr::Material::<D> {
            flags: (metadata::number(fields, "wmo.material.flags", 0)? & !3) | (material.flags & 3),
            shader: metadata::number(fields, "wmo.material.shader", 0)?,
            blend_mode: material.blend,
            ..Default::default()
        };
        let mut textures = fields
            .get("wmo.material.textures")
            .map(|(_, text)| metadata::parse_words(text))
            .transpose()?
            .unwrap_or_else(|| vec![u32::MAX; 3]);
        textures.resize(textures.len().max(3), u32::MAX);
        textures[0] = material.texture.unwrap_or(u32::MAX);
        let map = |index: u32| -> Result<u32, Error> {
            if index == u32::MAX {
                return Ok(0);
            }
            references
                .get(index as usize)
                .copied()
                .ok_or_else(|| Error::new("material.texture", "texture index out of bounds"))
        };
        value.texture0 = map(textures[0])?;
        value.texture1 = map(textures[1])?;
        value.texture2 = map(textures[2])?;
        value.sidn_color = metadata::number(fields, "wmo.material.sidn_color", 0)?.to_le_bytes();
        value.frame_sidn_color =
            metadata::number(fields, "wmo.material.frame_sidn_color", 0)?.to_le_bytes();
        value.diffuse_color =
            metadata::number(fields, "wmo.material.diffuse_color", u32::MAX)?.to_le_bytes();
        value.ground_type = metadata::number(fields, "wmo.material.ground_type", 0)?;
        let tail = fields
            .get("wmo.material.tail")
            .map(|(_, text)| metadata::parse_words(text))
            .transpose()?
            .unwrap_or_default();
        for (slot, word) in tail.into_iter().enumerate() {
            value
                .tail
                .set_texture(slot, word)
                .map_err(|error| failure("material.tail", error))?;
        }
        if matches!(value.shader, 22..=24) {
            for (slot, texture) in textures.iter().enumerate().take(9).skip(3) {
                value
                    .tail
                    .set_texture(slot - 3, map(*texture)?)
                    .map_err(|error| failure("material.texture", error))?;
            }
        }
        output.push(value);
    }
    Ok((output, uses_names.then_some(names)))
}
