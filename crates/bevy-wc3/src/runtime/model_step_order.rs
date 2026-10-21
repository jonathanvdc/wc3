//! Parent-first model particle evaluation, including detached ownership edges.
use bevy::prelude::*;
use std::collections::HashMap;
#[derive(Clone, Copy)]
pub(crate) enum ModelStepParent {
    /// External application/model ancestor already evaluated by the normal pass.
    Evaluated,
    /// An actor inside this owned pipeline must be projected/evaluated first.
    Pending(Entity),
    /// Missing/stale ownership never falls back to last frame's pose.
    Missing,
}
#[derive(Default)]
pub(crate) struct ModelStepOrder {
    entries: Vec<(Entity, ModelStepParent)>,
    indices: HashMap<Entity, usize>,
    parents: Vec<Option<usize>>,
    marks: Vec<u8>,
    path: Vec<usize>,
    ordered: Vec<Entity>,
    invalid: Vec<Entity>,
}
impl ModelStepOrder {
    /// Inputs include model-particle drivers and all particle-owned actor rigs.
    /// Parent is authored model ownership, not world-space Transform parenting.
    pub fn prepare(
        &mut self,
        entries: impl IntoIterator<Item = (Entity, ModelStepParent)>,
    ) -> &[Entity] {
        self.entries.clear();
        self.entries.extend(entries);
        self.entries
            .sort_unstable_by_key(|(entity, _)| entity.to_bits());
        self.indices.clear();
        for (index, (entity, _)) in self.entries.iter().enumerate() {
            self.indices.insert(*entity, index);
        }
        self.parents.clear();
        self.marks.clear();
        self.ordered.clear();
        self.invalid.clear();
        for (entity, parent) in &self.entries {
            let parent_index = match parent {
                ModelStepParent::Pending(parent) => self.indices.get(parent).copied(),
                _ => None,
            };
            let missing = matches!(parent, ModelStepParent::Missing)
                || matches!(parent, ModelStepParent::Pending(_)) && parent_index.is_none();
            self.parents.push(parent_index);
            self.marks.push(if missing { 3 } else { 0 });
            if missing {
                self.invalid.push(*entity);
            }
        }
        for start in 0..self.entries.len() {
            if self.marks[start] != 0 {
                continue;
            }
            self.path.clear();
            let mut next = Some(start);
            let mut valid = true;
            while let Some(index) = next {
                match self.marks[index] {
                    1 | 3 => {
                        valid = false;
                        break;
                    }
                    2 => break,
                    _ => {}
                }
                self.marks[index] = 1;
                self.path.push(index);
                next = self.parents[index];
            }
            for &index in self.path.iter().rev() {
                self.marks[index] = if valid { 2 } else { 3 };
                if valid {
                    self.ordered.push(self.entries[index].0);
                } else {
                    self.invalid.push(self.entries[index].0);
                }
            }
        }
        &self.ordered
    }
    pub fn invalid(&self) -> &[Entity] {
        &self.invalid
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detached_child_created_before_parent_still_evaluates_after_current_projection() {
        let mut world = World::new();
        let child = world.spawn_empty().id();
        let grandchild = world.spawn_empty().id();
        let parent = world.spawn_empty().id();
        let mut order = ModelStepOrder::default();
        let ordered = order.prepare([
            (grandchild, ModelStepParent::Pending(child)),
            (child, ModelStepParent::Pending(parent)),
            (parent, ModelStepParent::Evaluated),
        ]);
        assert_eq!(ordered, &[parent, child, grandchild]);
        let mut positions = HashMap::new();
        for &entity in ordered {
            let position = if entity == parent {
                12.
            } else if entity == child {
                positions[&parent] + 3.
            } else {
                positions[&child] + 2.
            };
            positions.insert(entity, position);
        }
        assert_eq!(positions[&grandchild], 17.);
    }
    #[test]
    fn cyclic_ownership_and_dependents_are_skipped_without_suppressing_independent_roots() {
        let mut world = World::new();
        let a = world.spawn_empty().id();
        let b = world.spawn_empty().id();
        let dependent = world.spawn_empty().id();
        let root = world.spawn_empty().id();
        let mut order = ModelStepOrder::default();
        assert_eq!(
            order.prepare([
                (a, ModelStepParent::Pending(b)),
                (b, ModelStepParent::Pending(a)),
                (dependent, ModelStepParent::Pending(a)),
                (root, ModelStepParent::Evaluated)
            ]),
            &[root]
        );
        assert_eq!(order.invalid().len(), 3);
        assert_eq!(
            order.prepare([(dependent, ModelStepParent::Evaluated)]),
            &[dependent]
        );
        assert!(order.invalid().is_empty());
    }
    #[test]
    fn missing_owner_invalidates_dependents_instead_of_treating_them_as_roots() {
        let mut world = World::new();
        let actor = world.spawn_empty().id();
        let child = world.spawn_empty().id();
        let stale = world.spawn_empty().id();
        let mut order = ModelStepOrder::default();
        assert!(order
            .prepare([
                (actor, ModelStepParent::Pending(stale)),
                (child, ModelStepParent::Pending(actor))
            ])
            .is_empty());
        assert_eq!(order.invalid().len(), 2);
        assert!(order
            .prepare([(actor, ModelStepParent::Missing)])
            .is_empty());
    }
}
