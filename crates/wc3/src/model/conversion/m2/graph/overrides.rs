//! Sparse customization transforms are logical bone bindings, not saved encoded files.
use super::*;
pub(super) fn import<'a, 'data: 'a, D: Dialect>(
    source: &'a Bundle<'data, D>,
    nodes: &[semantic::NodeId],
) -> Result<Vec<profile::BoneOverride<'a>>, Error> {
    let mut values = Vec::new();
    for (key, file) in source.files() {
        if let File::Bone(file) = file {
            let resource = Some(resource(key));
            if file.indices.len() != file.matrices.len() {
                return Err(Error::new(
                    "m2.bone_override",
                    "index and matrix cardinalities differ",
                ));
            }
            values.push(profile::BoneOverride {
                resource,
                bones: file
                    .indices
                    .iter()
                    .map(|i| {
                        nodes
                            .get(i as usize)
                            .copied()
                            .ok_or_else(|| Error::new("m2.bone_override", "missing bone"))
                    })
                    .collect::<Result<_, _>>()?,
                transforms: Values::view(file.matrices.len(), move |i| {
                    std::array::from_fn(|row| {
                        std::array::from_fn(|column| file.matrices[i].values[row * 4 + column])
                    })
                }),
            })
        }
    }
    Ok(values)
}
pub(super) fn export<D: ExportDialect>(
    values: &[profile::BoneOverride<'_>],
    nodes: &[semantic::NodeId],
    bundle: &mut Bundle<'static, D>,
    context: &mut Context,
) -> Result<(), Error> {
    let mut ids = Vec::new();
    for (index, value) in values.iter().enumerate() {
        let key = match &value.resource {
            Some(value) => {
                if let Resource::FileDataId(id) = value {
                    ids.push(*id);
                }
                file_key(value)?
            }
            None => bundle.key.sibling(format!("{index:02}.bone")),
        };
        let indices = value
            .bones
            .iter()
            .map(|id| {
                nodes
                    .iter()
                    .position(|n| n == id)
                    .ok_or_else(|| Error::new("m2.bone_override", "missing bone"))
                    .and_then(|i| {
                        u16::try_from(i)
                            .map_err(|_| Error::new("m2.bone_override", "index exceeds u16"))
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let matrices = value
            .transforms
            .iter()
            .map(|m| native::companions::BoneMatrix {
                values: std::array::from_fn(|i| m[i / 4][i % 4]),
            })
            .collect();
        bundle.insert(
            key,
            File::Bone(native::companions::BoneFile {
                version: 1,
                indices: ZeroVec::alloc_from_slice(&indices),
                matrices,
            }),
        );
    }
    D::bone_ids(&mut bundle.model, ids, context)?;
    Ok(())
}

pub(super) fn resource<'a>(key: &'a native::FileKey) -> Resource<'a> {
    match key {
        native::FileKey::Root => Resource::Root,
        native::FileKey::FileId(id) => Resource::FileDataId(*id),
        native::FileKey::Path(path) => Resource::Path(Cow::Borrowed(path.as_bytes())),
        native::FileKey::RawPath(path) => Resource::Path(Cow::Borrowed(path)),
        native::FileKey::Sibling { owner, suffix } => Resource::Relative {
            owner: Box::new(resource(owner)),
            suffix: Cow::Borrowed(suffix),
        },
    }
}
pub(super) fn file_key(value: &Resource<'_>) -> Result<native::FileKey, Error> {
    Ok(match value {
        Resource::Root => native::FileKey::Root,
        Resource::FileDataId(id) => native::FileKey::FileId(*id),
        Resource::Path(bytes) => match std::str::from_utf8(bytes) {
            Ok(path) => native::FileKey::Path(path.to_owned()),
            Err(_) => native::FileKey::RawPath(bytes.to_vec()),
        },
        Resource::Relative { owner, suffix } => native::FileKey::Sibling {
            owner: Box::new(file_key(owner)?),
            suffix: suffix.to_string(),
        },
        _ => {
            return Err(Error::new(
                "m2.resource",
                "resource namespace has no companion identity",
            ));
        }
    })
}
