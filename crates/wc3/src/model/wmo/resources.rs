//! Resource identities and string offsets without filesystem or archive policy.
use super::codec::Array;
use super::group::GroupChunk;
use super::root::RootChunk;
use super::{Dialect, FileKey, Group, GroupKey, ReadError, Root, WriteError};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::str::from_utf8;

/// A resource role declared by a root document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DependencyKind {
    /// Group geometry at an index and LOD level; zero is the base view.
    Group {
        /// Index in the root group-info table.
        index: usize,
        /// Zero for the base view; subsequent values identify LOD rows.
        lod: usize,
    },
    /// Material texture at a native slot.
    Texture {
        /// Index in the root material table.
        material: usize,
        /// Texture slot used by the material shader.
        slot: usize,
    },
    /// M2 placed as a doodad.
    Doodad {
        /// Placement index in MODD.
        index: usize,
    },
    /// Skybox M2 model.
    Skybox,
    /// Cookie texture referenced by an animated light in a resolved group.
    GroupLightCookie {
        /// Group index and LOD level within the root.
        group: GroupKey,
        /// Index in the group's MOP2 animated-light table.
        index: usize,
    },
    /// Light cookie texture.
    LightCookie {
        /// Index in the MNLD light table.
        index: usize,
    },
}
/// A resource reference plus its semantic owner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dependency {
    /// Role within the root document.
    pub kind: DependencyKind,
    /// Path or file-data identity supplied to the caller's resolver.
    pub key: FileKey,
}
/// A native texture reference resolved against the root's actual string table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextureReference<'view> {
    /// Native filename bytes, without the encoded terminator.
    Path(&'view [u8]),
    /// Game file-data ID.
    FileId(u32),
}
/// Reads a terminated string at a byte offset without assuming UTF-8.
pub fn string_at(data: &[u8], offset: u32) -> Result<&[u8], ReadError> {
    let start = usize::try_from(offset).map_err(|_| ReadError::invalid(0, "string offset"))?;
    let remaining = data
        .get(start..)
        .ok_or_else(|| ReadError::invalid(start, "string offset"))?;
    let end = remaining
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| ReadError::invalid(start, "unterminated string"))?;
    Ok(&remaining[..end])
}
fn path_key(bytes: &[u8]) -> FileKey {
    match from_utf8(bytes) {
        Ok(path) => FileKey::Path(path.to_owned()),
        Err(_) => FileKey::RawPath(bytes.to_vec()),
    }
}
impl<'data, D: Dialect> Root<'data, D> {
    /// Resolves a material texture slot; empty references return `None`.
    ///
    /// MOTX presence selects filename offsets, including transitional modern
    /// files. Without MOTX, slots carry file-data IDs. Historical trailing color
    /// and runtime words are not enumerated as resources for ordinary shaders.
    pub fn texture_reference(
        &self,
        material: usize,
        slot: usize,
    ) -> Result<Option<TextureReference<'_>>, ReadError> {
        let materials = self
            .chunks()
            .find_map(|chunk| match chunk {
                RootChunk::Materials(values) => Some(values),
                _ => None,
            })
            .ok_or_else(|| ReadError::invalid(0, "missing material table"))?;
        let material = materials
            .get(material)
            .ok_or_else(|| ReadError::invalid(0, "material index"))??;
        let slots = if matches!(material.shader, 22..=24) {
            9
        } else {
            3
        };
        if slot >= slots {
            return Err(ReadError::invalid(0, "material texture slot"));
        }
        let value = material.textures()[slot];
        if let Some(names) = self.chunks().find_map(|chunk| match chunk {
            RootChunk::TextureNames(names) => Some(names.as_bytes()),
            _ => None,
        }) {
            let name = string_at(names, value)?;
            Ok((!name.is_empty()).then_some(TextureReference::Path(name)))
        } else {
            Ok((value != 0).then_some(TextureReference::FileId(value)))
        }
    }
    /// Enumerates base and available LOD group identities plus external assets.
    pub fn dependencies(&self, owner: &FileKey) -> Result<Vec<Dependency>, ReadError> {
        let mut result = Vec::new();
        let count = self.group_count();
        if count != 0
            && !self
                .chunks()
                .any(|chunk| matches!(chunk, RootChunk::Groups(_) | RootChunk::Groups2(_)))
        {
            return Err(ReadError::invalid(0, "missing group information table"));
        }
        if let Some(ids) = self.chunks().find_map(|chunk| match chunk {
            RootChunk::GroupFileIds(ids) => Some(ids),
            _ => None,
        }) {
            if count != self.header().group_count as usize {
                return Err(ReadError::invalid(
                    0,
                    "GFID stride differs from group table",
                ));
            }
            if count == 0 {
                if !ids.is_empty() {
                    return Err(ReadError::invalid(0, "GFID without groups"));
                }
            } else {
                if ids.len() < count || ids.len() % count != 0 {
                    return Err(ReadError::invalid(0, "GFID row dimensions"));
                }
                for (position, id) in ids.iter().enumerate() {
                    let id = id?;
                    if id != 0 {
                        result.push(Dependency {
                            kind: DependencyKind::Group {
                                index: position % count,
                                lod: position / count,
                            },
                            key: FileKey::FileId(id),
                        });
                    }
                }
            }
        } else {
            for index in 0..count {
                result.push(Dependency {
                    kind: DependencyKind::Group { index, lod: 0 },
                    key: owner.sibling(format!("_{index:03}.wmo")),
                });
            }
        }
        let materials = self.chunks().find_map(|chunk| match chunk {
            RootChunk::Materials(materials) => Some(materials),
            _ => None,
        });
        if let Some(materials) = materials {
            for (index, material) in materials.iter().enumerate() {
                let material = material?;
                let slots = if matches!(material.shader, 22..=24) {
                    9
                } else {
                    3
                };
                for slot in 0..slots {
                    if let Some(reference) = self.texture_reference(index, slot)? {
                        let key = match reference {
                            TextureReference::Path(path) => path_key(path),
                            TextureReference::FileId(id) => FileKey::FileId(id),
                        };
                        result.push(Dependency {
                            kind: DependencyKind::Texture {
                                material: index,
                                slot,
                            },
                            key,
                        });
                    }
                }
            }
        }
        let doodads = self.chunks().find_map(|chunk| match chunk {
            RootChunk::Doodads(values) => Some(values),
            _ => None,
        });
        let names = self.chunks().find_map(|chunk| match chunk {
            RootChunk::DoodadNames(values) => Some(values.as_bytes()),
            _ => None,
        });
        let ids = self.chunks().find_map(|chunk| match chunk {
            RootChunk::DoodadFileIds(values) => Some(values),
            _ => None,
        });
        if let Some(doodads) = doodads {
            for (index, doodad) in doodads.iter().enumerate() {
                let reference = doodad?.name_index();
                let key = if let Some(names) = names {
                    let name = string_at(names, reference)?;
                    if name.is_empty() {
                        continue;
                    }
                    path_key(name)
                } else if let Some(ids) = ids {
                    let id = ids
                        .get(reference as usize)
                        .ok_or_else(|| ReadError::invalid(0, "MODI index"))??;
                    if id == 0 {
                        continue;
                    }
                    FileKey::FileId(id)
                } else {
                    return Err(ReadError::invalid(0, "missing doodad resource table"));
                };
                result.push(Dependency {
                    kind: DependencyKind::Doodad { index },
                    key,
                });
            }
        }
        for chunk in self.chunks() {
            match chunk {
                RootChunk::SkyboxFileId(id) if *id != 0 => result.push(Dependency {
                    kind: DependencyKind::Skybox,
                    key: FileKey::FileId(*id),
                }),
                RootChunk::SkyboxName(name) if !name.is_empty() => {
                    let path = string_at(name.as_bytes(), 0)?;
                    if !path.is_empty() {
                        result.push(Dependency {
                            kind: DependencyKind::Skybox,
                            key: path_key(path),
                        });
                    }
                }
                RootChunk::NewLights(lights) => {
                    for (index, light) in lights.iter().enumerate() {
                        let id = light?.cookie_file_id;
                        if id != 0 {
                            result.push(Dependency {
                                kind: DependencyKind::LightCookie { index },
                                key: FileKey::FileId(id),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(result)
    }
    /// Rewrites MOTX paths and all material offsets that reference them.
    ///
    /// The callback receives native filename bytes. Edited tables are rebuilt
    /// deterministically; unused original strings remain in the table. File-ID
    /// materials are edited through their native fields instead.
    pub fn rewrite_texture_paths(
        &mut self,
        mut rewrite: impl FnMut(&[u8]) -> Vec<u8>,
    ) -> Result<(), WriteError> {
        let names = self
            .chunks()
            .find_map(|chunk| match chunk {
                RootChunk::TextureNames(values) => Some(values.as_bytes()),
                _ => None,
            })
            .ok_or_else(|| WriteError::invalid("missing MOTX"))?;
        let mut bytes = names.to_vec();
        let mut offsets = BTreeMap::new();
        for chunk in self.chunks() {
            if let RootChunk::Materials(materials) = chunk {
                for material in materials.iter() {
                    let material = material.map_err(|_| WriteError::invalid("material record"))?;
                    let slots = if matches!(material.shader, 22..=24) {
                        9
                    } else {
                        3
                    };
                    for old in &material.textures()[..slots] {
                        if offsets.contains_key(old) {
                            continue;
                        }
                        let name = string_at(names, *old)
                            .map_err(|_| WriteError::invalid("MOTX offset"))?;
                        if name.is_empty() {
                            offsets.insert(*old, *old);
                            continue;
                        }
                        let edited = rewrite(name);
                        if edited.contains(&0) {
                            return Err(WriteError::invalid("embedded filename terminator"));
                        }
                        if edited == name {
                            offsets.insert(*old, *old);
                            continue;
                        }
                        let offset = u32::try_from(bytes.len())
                            .map_err(|_| WriteError::invalid("MOTX size"))?;
                        bytes.extend_from_slice(&edited);
                        bytes.push(0);
                        offsets.insert(*old, offset);
                    }
                }
            }
        }
        // All fallible string transformations precede mutation.
        for chunk in self.chunks_mut() {
            match chunk {
                RootChunk::TextureNames(values) => *values = Array::from_records(&bytes)?,
                RootChunk::Materials(materials) => {
                    for index in 0..materials.len() {
                        let mut material = materials
                            .get(index)
                            .ok_or_else(|| WriteError::invalid("material index"))?
                            .map_err(|_| WriteError::invalid("material record"))?;
                        let slots = if matches!(material.shader, 22..=24) {
                            9
                        } else {
                            3
                        };
                        let old = material.textures();
                        for slot in 0..slots {
                            material.set_texture(slot, offsets[&old[slot]])?;
                        }
                        materials.set(index, &material)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
/// Mutable string-table bytes for callers editing group and doodad names.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StringBlock<'data> {
    /// Native bytes, including terminators and unused entries.
    pub bytes: Cow<'data, [u8]>,
}
impl StringBlock<'_> {
    /// Borrows one native string without a text-encoding assumption.
    pub fn get(&self, offset: u32) -> Result<&[u8], ReadError> {
        string_at(&self.bytes, offset)
    }
    /// Appends a string and returns its native byte offset.
    pub fn append(&mut self, value: &[u8]) -> Result<u32, WriteError> {
        if value.contains(&0) {
            return Err(WriteError::invalid("embedded string terminator"));
        }
        let offset = u32::try_from(self.bytes.len())
            .map_err(|_| WriteError::invalid("string block size"))?;
        let bytes = self.bytes.to_mut();
        bytes.extend_from_slice(value);
        bytes.push(0);
        Ok(offset)
    }
}

impl<D: Dialect> Group<'_, D> {
    /// Reports typed group-local cookie textures without loading game resources.
    pub fn dependencies(&self, group: GroupKey) -> Result<Vec<Dependency>, ReadError> {
        let mut dependencies = Vec::new();
        for chunk in self.chunks() {
            if let GroupChunk::PointLightAnimations(lights) = chunk {
                for (index, light) in lights.iter().enumerate() {
                    let id = light?.texture_file_id;
                    if id != 0 {
                        dependencies.push(Dependency {
                            kind: DependencyKind::GroupLightCookie { group, index },
                            key: FileKey::FileId(id),
                        });
                    }
                }
            }
        }
        Ok(dependencies)
    }
}
