//! Root/group assembly with caller-provided resource identities and buffers.
use super::{
    Dependency, DependencyKind, Dialect, FileKey, Group, ReadError, Root, Validate, WriteError,
};
use crate::model::common::Ownership;
use std::collections::BTreeMap;

/// A group index and geometry level in the owning root.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GroupKey {
    /// Index in the root group-info table.
    pub index: usize,
    /// Zero for base geometry, larger values for LOD rows.
    pub lod: usize,
}
/// Native group document and the identity used to resolve it.
#[derive(Clone, Debug, PartialEq, Ownership)]
struct ResolvedGroup<'data, D: Dialect> {
    key: FileKey,
    group: Group<'data, D>,
}
/// A root connected to independently editable base and LOD group documents.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct Bundle<'data, D: Dialect> {
    /// Root document with shared records and external references.
    pub root: Root<'data, D>,
    key: FileKey,
    groups: BTreeMap<GroupKey, ResolvedGroup<'data, D>>,
}
/// Relocated root bytes and supporting group outputs.
#[derive(Clone, Debug, PartialEq)]
pub struct EncodedBundle {
    /// Encoded root file.
    pub root: Vec<u8>,
    /// Group files keyed by their original caller-visible identities.
    pub groups: BTreeMap<FileKey, Vec<u8>>,
}
impl<'data, D: Dialect> Bundle<'data, D> {
    /// Starts a partial assembly, without eagerly resolving external resources.
    pub fn from_root(root: Root<'data, D>) -> Self {
        Self {
            root,
            key: FileKey::Root,
            groups: BTreeMap::new(),
        }
    }
    /// Assigns the identity used to derive legacy group filenames.
    pub fn with_key(mut self, key: FileKey) -> Self {
        self.key = key;
        self
    }
    /// Returns declared dependencies, including unloaded textures and doodad M2s.
    pub fn dependencies(&self) -> Result<Vec<Dependency>, ReadError> {
        {
            let mut dependencies = self.root.dependencies(&self.key)?;
            for (key, value) in &self.groups {
                dependencies.extend(value.group.dependencies(*key)?);
            }
            Ok(dependencies)
        }
    }
    /// Registers a base or LOD group under its physical document identity.
    pub fn insert_group(&mut self, group: GroupKey, key: FileKey, document: Group<'data, D>) {
        self.groups.insert(
            group,
            ResolvedGroup {
                key,
                group: document,
            },
        );
    }
    /// Borrows a resolved group.
    pub fn group(&self, key: GroupKey) -> Option<&Group<'data, D>> {
        self.groups.get(&key).map(|value| &value.group)
    }
    /// Returns the caller-visible resource identity of a resolved group.
    pub fn group_file_key(&self, key: GroupKey) -> Option<&FileKey> {
        self.groups.get(&key).map(|value| &value.key)
    }
    /// Edits a resolved group in its native dialect.
    pub fn group_mut(&mut self, key: GroupKey) -> Option<&mut Group<'data, D>> {
        self.groups.get_mut(&key).map(|value| &mut value.group)
    }
    /// Reads the root and requests only supporting WMO groups from the resolver.
    ///
    /// Missing files produce a partial assembly; `validate_complete` distinguishes
    /// a usable complete model. Returned slices must outlive the bundle.
    pub fn decode(
        data: &'data [u8],
        key: FileKey,
        mut resolve: impl FnMut(&Dependency) -> Result<Option<&'data [u8]>, ReadError>,
    ) -> Result<Self, ReadError> {
        let mut bundle = Self::from_root(Root::decode(data)?).with_key(key);
        for dependency in bundle.dependencies()? {
            if let DependencyKind::Group { index, lod } = dependency.kind {
                if let Some(bytes) = resolve(&dependency)? {
                    bundle.insert_group(
                        GroupKey { index, lod },
                        dependency.key,
                        Group::decode(bytes)?,
                    );
                }
            }
        }
        Ok(bundle)
    }
    fn validate_group_identities(&self) -> Result<(), ReadError> {
        let declarations = self
            .dependencies()?
            .into_iter()
            .filter_map(|dependency| match dependency.kind {
                DependencyKind::Group { index, lod } => {
                    Some((GroupKey { index, lod }, dependency.key))
                }
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();
        for (group, value) in &self.groups {
            if declarations.get(group) != Some(&value.key) {
                return Err(ReadError::invalid(
                    0,
                    "group identity differs from root reference",
                ));
            }
        }
        Ok(())
    }
    /// Checks local document invariants and the root/group reference graph.
    pub fn validate_complete(&self) -> Result<(), ReadError> {
        self.root.validate()?;
        self.validate_group_identities()?;
        for index in 0..self.root.group_count() {
            if self.group(GroupKey { index, lod: 0 }).is_none() {
                return Err(ReadError::invalid(0, "missing base group"));
            }
        }
        for (key, value) in &self.groups {
            if key.index >= self.root.group_count() {
                return Err(ReadError::invalid(0, "group index outside root"));
            }
            value.group.validate()?;
            self.validate_group_references(&value.group)?;
        }
        for dependency in self.dependencies()? {
            if let DependencyKind::Group { index, lod } = dependency.kind {
                if self.group(GroupKey { index, lod }).is_none() {
                    return Err(ReadError::invalid(0, "missing declared LOD group"));
                }
            }
        }
        Ok(())
    }
    /// Encodes native documents and rejects conflicting outputs for aliased identities.
    pub fn encode(&self) -> Result<EncodedBundle, WriteError> {
        self.validate_group_identities()
            .map_err(|_| WriteError::invalid("group identity differs from root reference"))?;
        let mut groups = BTreeMap::new();
        for value in self.groups.values() {
            let bytes = value.group.encode()?;
            if let Some(previous) = groups.get(&value.key) {
                if previous != &bytes {
                    return Err(WriteError::invalid(
                        "incompatible groups share file identity",
                    ));
                }
            } else {
                groups.insert(value.key.clone(), bytes);
            }
        }
        Ok(EncodedBundle {
            root: self.root.encode()?,
            groups,
        })
    }
}
