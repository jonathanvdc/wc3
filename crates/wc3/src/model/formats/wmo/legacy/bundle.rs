//! External group assembly for version-16 roots.
use super::{Group16, Root16};
use crate::model::{common::Ownership, formats::wmo};
use std::collections::BTreeMap;
use wmo::{Dependency, DependencyKind, EncodedBundle, FileKey, ReadError, Validate, WriteError};

#[derive(Clone, Debug, PartialEq, Ownership)]
struct ResolvedGroup16<'data> {
    key: FileKey,
    group: Group16<'data>,
}
/// A version-16 root connected to its native 32-byte-batch group documents.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct Bundle16<'data> {
    /// Root document declaring the external groups and shared resources.
    pub root: Root16<'data>,
    key: FileKey,
    groups: BTreeMap<usize, ResolvedGroup16<'data>>,
}
impl<'data> Bundle16<'data> {
    /// Starts a partial assembly without loading external resources.
    pub fn from_root(root: Root16<'data>) -> Self {
        Self {
            root,
            key: FileKey::Root,
            groups: BTreeMap::new(),
        }
    }
    /// Assigns the root identity used to derive group filenames.
    pub fn with_key(mut self, key: FileKey) -> Self {
        self.key = key;
        self
    }
    /// Enumerates native group names and external assets.
    pub fn dependencies(&self) -> Result<Vec<Dependency>, ReadError> {
        self.root.dependencies(&self.key)
    }
    /// Registers a native group under its declared filename identity.
    pub fn insert_group(&mut self, index: usize, key: FileKey, group: Group16<'data>) {
        self.groups.insert(index, ResolvedGroup16 { key, group });
    }
    /// Borrows a loaded geometry group.
    pub fn group(&self, index: usize) -> Option<&Group16<'data>> {
        self.groups.get(&index).map(|entry| &entry.group)
    }
    /// Edits a loaded geometry group.
    pub fn group_mut(&mut self, index: usize) -> Option<&mut Group16<'data>> {
        self.groups.get_mut(&index).map(|entry| &mut entry.group)
    }
    /// Decodes the root and requests its external version-16 groups.
    pub fn decode(
        data: &'data [u8],
        key: FileKey,
        mut resolve: impl FnMut(&Dependency) -> Result<Option<&'data [u8]>, ReadError>,
    ) -> Result<Self, ReadError> {
        let mut bundle = Self::from_root(Root16::decode(data)?).with_key(key);
        for dependency in bundle.dependencies()? {
            if let DependencyKind::Group { index, lod: 0 } = dependency.kind {
                if let Some(bytes) = resolve(&dependency)? {
                    bundle.insert_group(index, dependency.key, Group16::decode(bytes)?);
                }
            }
        }
        Ok(bundle)
    }
    fn validate_identities(&self) -> Result<(), ReadError> {
        for (index, entry) in &self.groups {
            if *index >= self.root.group_count()
                || entry.key != self.key.sibling(format!("_{index:03}.wmo"))
            {
                return Err(ReadError::invalid(
                    0,
                    "version-16 group identity differs from root",
                ));
            }
        }
        Ok(())
    }
    /// Checks completeness, native group geometry, and shared root references.
    pub fn validate_complete(&self) -> Result<(), ReadError> {
        self.root.validate()?;
        self.validate_identities()?;
        for index in 0..self.root.group_count() {
            let group = self
                .group(index)
                .ok_or_else(|| ReadError::invalid(0, "missing version-16 group"))?;
            group.validate()?;
            self.root.validate_group_references(group)?;
        }
        Ok(())
    }
    /// Writes partial or complete assemblies while keeping filename identities coherent.
    pub fn encode(&self) -> Result<EncodedBundle, WriteError> {
        self.validate_identities()
            .map_err(|_| WriteError::invalid("version-16 group identity differs from root"))?;
        let groups = self
            .groups
            .values()
            .map(|entry| Ok((entry.key.clone(), entry.group.encode()?)))
            .collect::<Result<_, WriteError>>()?;
        Ok(EncodedBundle {
            root: self.root.encode()?,
            groups,
        })
    }
}
