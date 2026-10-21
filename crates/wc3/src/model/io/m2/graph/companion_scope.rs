//! Native companion ownership within one semantic model import.
use crate::model::formats::m2::{Bundle, Dependency, DependencyKind, Dialect, File, FileKey};
use std::collections::{HashMap, HashSet, VecDeque};

pub(super) struct CompanionScope {
    child_only: HashSet<FileKey>,
}
impl CompanionScope {
    pub fn new<D: Dialect>(source: &Bundle<'_, D>) -> Self {
        let dependencies = source.dependencies();
        let documents = source
            .files()
            .filter_map(|(key, file)| matches!(file, File::Document(_)).then_some(key.clone()));
        Self::from_dependencies(&source.key, documents, &dependencies)
    }
    fn from_dependencies(
        root: &FileKey,
        documents: impl IntoIterator<Item = FileKey>,
        dependencies: &[Dependency],
    ) -> Self {
        let mut skeletons: HashMap<&FileKey, Vec<&FileKey>> = HashMap::new();
        for dependency in dependencies {
            if matches!(dependency.kind, DependencyKind::Skeleton) {
                skeletons
                    .entry(&dependency.owner)
                    .or_default()
                    .push(&dependency.key);
            }
        }
        let closure = |seeds: Vec<FileKey>| {
            let mut owners: HashSet<FileKey> = seeds.into_iter().collect();
            let mut pending: VecDeque<FileKey> = owners.iter().cloned().collect();
            while let Some(owner) = pending.pop_front() {
                if let Some(targets) = skeletons.get(&owner) {
                    for &target in targets {
                        if owners.insert(target.clone()) {
                            pending.push_back(target.clone());
                        }
                    }
                }
            }
            owners
        };
        let root_owners = closure(vec![root.clone()]);
        let child_owners = closure(documents.into_iter().collect());
        let mut root_keys = HashSet::new();
        let mut child_keys = HashSet::new();
        for dependency in dependencies {
            if matches!(
                dependency.kind,
                DependencyKind::Bone | DependencyKind::Physics
            ) {
                if root_owners.contains(&dependency.owner) {
                    root_keys.insert(dependency.key.clone());
                }
                if child_owners.contains(&dependency.owner) {
                    child_keys.insert(dependency.key.clone());
                }
            }
        }
        child_keys.retain(|key| !root_keys.contains(key));
        Self {
            child_only: child_keys,
        }
    }
    /// Undeclared optional root companions retain their established behavior;
    /// only documents explicitly owned exclusively by child trees are excluded.
    pub fn includes(&self, key: &FileKey) -> bool {
        !self.child_only.contains(key)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn root_skeleton_closure_and_shared_keys_win_over_child_ownership() {
        let key = FileKey::FileId;
        let dependency = |owner, target, kind| Dependency {
            owner: key(owner),
            key: key(target),
            kind,
        };
        let scope = CompanionScope::from_dependencies(
            &key(1),
            [key(2)],
            &[
                dependency(1, 10, DependencyKind::Skeleton),
                dependency(10, 11, DependencyKind::Skeleton),
                dependency(11, 10, DependencyKind::Skeleton),
                dependency(11, 77, DependencyKind::Bone),
                dependency(2, 20, DependencyKind::Skeleton),
                dependency(20, 88, DependencyKind::Bone),
                dependency(20, 89, DependencyKind::Physics),
                dependency(2, 77, DependencyKind::Bone),
            ],
        );
        assert!(scope.includes(&key(77)));
        assert!(!scope.includes(&key(88)));
        assert!(!scope.includes(&key(89)));
        assert!(
            scope.includes(&key(99)),
            "manually attached optional root companion remains supported"
        );
    }
}
