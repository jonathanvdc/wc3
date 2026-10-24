//! Sparse customization transforms are logical bone bindings, not saved encoded files.
use super::companion_scope::CompanionScope;
use super::*;
use crate::model::error::Location;
use crate::model::io::materialize;
use crate::model::M2ResourceLocator;
use glam::Mat4;
use std::str::from_utf8;
use zerovec::ZeroVec;
pub(super) fn import<'a, 'data: 'a, D: Dialect>(
    source: &'a Bundle<'data, D>,
    nodes: &[semantic::NodeId],
    builder: &mut GraphBuilder<'a>,
    companions: &CompanionScope,
) -> Result<Vec<profile::BoneOverride<'a>>, Error> {
    let mut values = Vec::new();
    for (key, file) in source.files().filter(|(key, _)| companions.includes(key)) {
        if let File::Bone(file) = file {
            let resource = Some(builder.add_bone_override_resource(resource(key)));
            if file.indices.len() != file.matrices.len() {
                return Err(Error::requirement(
                    Location::new().field("m2").field("bone_override"),
                    "index and matrix cardinalities differ",
                ));
            }
            values.push(profile::BoneOverride {
                resource,
                bones: file
                    .indices
                    .iter()
                    .map(|i| {
                        nodes.get(i as usize).copied().ok_or_else(|| {
                            Error::requirement(
                                Location::new().field("m2").field("bone_override"),
                                "missing bone",
                            )
                        })
                    })
                    .collect::<Result<_, _>>()?,
                transforms: materialize(file.matrices.len(), move |i| {
                    Mat4::from_cols_array(&file.matrices[i].values)
                }),
            })
        }
    }
    Ok(values)
}
pub(super) fn export<D: ExportDialect>(
    model: &Graph<'_>,
    values: &[profile::BoneOverride<'_>],
    nodes: &[semantic::NodeId],
    bundle: &mut Bundle<'static, D>,
    context: &mut Context,
) -> Result<(), Error> {
    let mut ids = Vec::new();
    for (index, value) in values.iter().enumerate() {
        let key = match &value.resource {
            Some(id) => {
                let value = model
                    .bone_override_resource(*id)
                    .expect("validated bone resource");
                if let M2ResourceLocator::FileDataId(id) = value {
                    ids.push(*id);
                }
                file_key(value)
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
                    .ok_or_else(|| {
                        Error::requirement(
                            Location::new().field("m2").field("bone_override"),
                            "missing bone",
                        )
                    })
                    .and_then(|i| {
                        u16::try_from(i).map_err(|_| {
                            Error::requirement(
                                Location::new().field("m2").field("bone_override"),
                                "index exceeds u16",
                            )
                        })
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let matrices = value
            .transforms
            .iter()
            .map(|m| native::companions::BoneMatrix {
                values: m.to_cols_array(),
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

pub(super) fn resource<'a>(key: &'a native::FileKey) -> M2ResourceLocator<'a> {
    match key {
        native::FileKey::Root => M2ResourceLocator::Root,
        native::FileKey::FileId(id) => M2ResourceLocator::FileDataId(*id),
        native::FileKey::Path(path) => M2ResourceLocator::Path(Cow::Borrowed(path.as_bytes())),
        native::FileKey::RawPath(path) => M2ResourceLocator::Path(Cow::Borrowed(path)),
        native::FileKey::Sibling { owner, suffix } => M2ResourceLocator::Relative {
            owner: Box::new(resource(owner)),
            suffix: Cow::Borrowed(suffix),
        },
    }
}
pub(super) fn file_key(value: &M2ResourceLocator<'_>) -> native::FileKey {
    match value {
        M2ResourceLocator::Root => native::FileKey::Root,
        M2ResourceLocator::FileDataId(id) => native::FileKey::FileId(*id),
        M2ResourceLocator::Path(bytes) => match from_utf8(bytes) {
            Ok(path) => native::FileKey::Path(path.to_owned()),
            Err(_) => native::FileKey::RawPath(bytes.to_vec()),
        },
        M2ResourceLocator::Relative { owner, suffix } => native::FileKey::Sibling {
            owner: Box::new(file_key(owner)),
            suffix: suffix.to_string(),
        },
    }
}
