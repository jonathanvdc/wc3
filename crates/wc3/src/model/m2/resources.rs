//! Declared supporting-file identities, independent of filesystem or archive access.
//!
//! Dependencies retain their owner because relative paths and generated sibling
//! names require that context. File IDs and arbitrary filename bytes are preserved
//! without assigning them to any particular filesystem or archive provider.
use super::chunks::{AnimationFileId, Chunk};
use super::companions::{DynamicSkeleton, Skeleton, SkeletonChunk};
use super::records::Sequence;
use super::records::TextureSource;
use super::{Dialect, Model, ModelLayout};
use crate::visit_m2_skeleton;

pub use crate::model::common::FileKey;

/// The model role a referenced file supplies.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum DependencyKind {
    /// An external mesh view, with its base-view or optional LOD index.
    Skin {
        /// Index within the base-skin or LOD-skin list.
        index: usize,
        /// Whether this is an additional LOD view rather than a base view.
        lod: bool,
    },
    /// External keys for an animation ID and variation.
    Animation {
        /// Animation ID supplied by the owning sequence or AFID record.
        animation_id: u16,
        /// Variation of that animation.
        variation_id: u16,
    },
    /// A shared skeleton or parent skeleton.
    Skeleton,
    /// Sparse bone transform overrides used by character customization.
    Bone,
    /// Physics shape, body, and constraint data.
    Physics,
    /// A model referenced by a geometry or recursive particle emitter.
    Model,
    /// An ordinary external texture, excluding replaceable texture resources.
    Texture,
}
/// A declared reference and the document that gives it meaning.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Dependency {
    /// Document containing the reference.
    pub owner: FileKey,
    /// Native or generated identity of the referenced file.
    pub key: FileKey,
    /// Role of the referenced file in the model bundle.
    pub kind: DependencyKind,
}
impl<'data, D: Dialect> Model<'data, D> {
    /// Enumerates declared references without loading or probing supporting files.
    ///
    /// Modern file-ID chunks take precedence over corresponding filename fields.
    /// Legacy skin and animation names are derived from the supplied owner. LOD
    /// skins and bone siblings with no declared count or ID list require explicit
    /// discovery by the caller and are not guessed here.
    pub fn dependencies(&self, owner: &FileKey) -> Vec<Dependency> {
        let mut result = Vec::new();
        let core = self.core();
        let extensions: Vec<_> = self.extensions().collect();
        let skin_ids = extensions.iter().find_map(|c| {
            if let Chunk::SkinIds(ids) = c {
                Some(ids)
            } else {
                None
            }
        });
        let count = core.external_skin_count() as usize;
        if let Some(ids) = skin_ids {
            for (index, id) in ids.iter().enumerate() {
                if id != 0 {
                    push(
                        &mut result,
                        owner,
                        FileKey::FileId(id),
                        DependencyKind::Skin {
                            index: if index < count { index } else { index - count },
                            lod: index >= count,
                        },
                    );
                }
            }
        } else {
            for index in 0..count {
                push(
                    &mut result,
                    owner,
                    owner.sibling(format!("{index:02}.skin")),
                    DependencyKind::Skin { index, lod: false },
                );
            }
        }
        let animation_ids: Vec<_> = extensions
            .iter()
            .filter_map(|c| {
                if let Chunk::AnimationIds(ids) = c {
                    Some(ids.as_slice())
                } else {
                    None
                }
            })
            .flatten()
            .collect();
        for entry in &animation_ids {
            animation_dependency(&mut result, owner, entry);
        }
        if D::VERSION >= 264 {
            for (index, sequence) in core.sequences.iter().enumerate() {
                if !sequence.flags.external_animation() {
                    continue;
                }
                let Some(source) = sequence_source(&core.sequences, index) else {
                    continue;
                };
                let kind = DependencyKind::Animation {
                    animation_id: source.id,
                    variation_id: source.variation_index,
                };
                if animation_ids.iter().any(|entry| {
                    entry.animation_id == source.id && entry.variation_id == source.variation_index
                }) {
                    continue;
                }
                push(
                    &mut result,
                    owner,
                    owner.sibling(format!(
                        "{:04}-{:02}.anim",
                        source.id, source.variation_index
                    )),
                    kind,
                );
            }
        }
        let texture_ids = extensions.iter().find_map(|c| {
            if let Chunk::TextureIds(ids) = c {
                Some(ids)
            } else {
                None
            }
        });
        for (index, texture) in core.textures.iter().enumerate() {
            if texture.kind != TextureSource::File {
                continue;
            }
            if let Some(id) = texture_ids
                .and_then(|ids| ids.get(index))
                .filter(|id| *id != 0)
            {
                push(
                    &mut result,
                    owner,
                    FileKey::FileId(id),
                    DependencyKind::Texture,
                );
            } else if let Some(key) = filename(texture.filename.as_bytes()) {
                push(&mut result, owner, key, DependencyKind::Texture);
            }
        }
        let recursive_ids = extensions.iter().find_map(|c| {
            if let Chunk::RecursiveParticleIds(ids) = c {
                Some(ids)
            } else {
                None
            }
        });
        let geometry_ids = extensions.iter().find_map(|c| {
            if let Chunk::GeometryParticleIds(ids) = c {
                Some(ids)
            } else {
                None
            }
        });
        for (index, particle) in core.particles.iter().enumerate() {
            for (ids, name) in [
                (recursive_ids, particle.child_model_filename.as_bytes()),
                (geometry_ids, particle.particle_model_filename.as_bytes()),
            ] {
                if let Some(id) = ids.and_then(|ids| ids.get(index)).filter(|id| *id != 0) {
                    push(
                        &mut result,
                        owner,
                        FileKey::FileId(id),
                        DependencyKind::Model,
                    );
                } else if let Some(key) = filename(name) {
                    push(&mut result, owner, key, DependencyKind::Model);
                }
            }
        }
        let mut physics = false;
        for extension in extensions {
            match extension {
                Chunk::SkeletonId(id) if *id != 0 => push(
                    &mut result,
                    owner,
                    FileKey::FileId(*id),
                    DependencyKind::Skeleton,
                ),
                Chunk::BoneIds(ids) => {
                    for id in ids.iter().filter(|id| *id != 0) {
                        push(
                            &mut result,
                            owner,
                            FileKey::FileId(id),
                            DependencyKind::Bone,
                        );
                    }
                }
                Chunk::PhysicsId(id) => {
                    physics = true;
                    if *id != 0 {
                        push(
                            &mut result,
                            owner,
                            FileKey::FileId(*id),
                            DependencyKind::Physics,
                        );
                    }
                }
                Chunk::Physics(_) => physics = true,
                _ => {}
            }
        }
        if !physics && core.flags.external_physics() {
            push(
                &mut result,
                owner,
                owner.sibling(".phys"),
                DependencyKind::Physics,
            );
        }
        result
    }
}
impl<V: ModelLayout> Skeleton<'_, V> {
    /// Enumerates parent-skeleton, animation, and bone IDs in native chunk order.
    pub fn dependencies(&self, owner: &FileKey) -> Vec<Dependency> {
        let mut result = Vec::new();
        for chunk in &self.chunks {
            match chunk {
                SkeletonChunk::Parent { file_id, .. } if *file_id != 0 => push(
                    &mut result,
                    owner,
                    FileKey::FileId(*file_id),
                    DependencyKind::Skeleton,
                ),
                SkeletonChunk::AnimationIds(ids) => {
                    for entry in ids {
                        animation_dependency(&mut result, owner, entry);
                    }
                }
                SkeletonChunk::BoneIds(ids) => {
                    for id in ids.iter().filter(|id| *id != 0) {
                        push(
                            &mut result,
                            owner,
                            FileKey::FileId(id),
                            DependencyKind::Bone,
                        );
                    }
                }
                _ => {}
            }
        }
        result
    }
}
fn animation_dependency(result: &mut Vec<Dependency>, owner: &FileKey, entry: &AnimationFileId) {
    if entry.file_id != 0 {
        push(
            result,
            owner,
            FileKey::FileId(entry.file_id),
            DependencyKind::Animation {
                animation_id: entry.animation_id,
                variation_id: entry.variation_id,
            },
        );
    }
}
fn push(result: &mut Vec<Dependency>, owner: &FileKey, key: FileKey, kind: DependencyKind) {
    let dependency = Dependency {
        owner: owner.clone(),
        key,
        kind,
    };
    if !result.contains(&dependency) {
        result.push(dependency);
    }
}
fn filename(bytes: &[u8]) -> Option<FileKey> {
    let bytes = &bytes[..bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len())];
    if bytes.is_empty() {
        return None;
    }
    Some(match String::from_utf8(bytes.to_vec()) {
        Ok(path) => FileKey::Path(path),
        Err(_) => FileKey::RawPath(bytes.to_vec()),
    })
}
fn sequence_source<V: ModelLayout>(
    sequences: &[Sequence<V>],
    index: usize,
) -> Option<&Sequence<V>> {
    let mut current_index = index;
    for _ in 0..sequences.len() {
        let current = sequences.get(current_index)?;
        if !current.flags.alias() {
            return Some(current);
        }
        current_index = current.alias_next as usize;
    }
    None
}

impl DynamicSkeleton<'_> {
    /// Enumerates native resource references using the retained skeleton layout.
    pub fn dependencies(&self, owner: &FileKey) -> Vec<Dependency> {
        visit_m2_skeleton!(self, |file| file.dependencies(owner))
    }
}
