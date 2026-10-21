//! Sparse current-frame transforms for pre-propagation model-particle placement.
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// Exact model-pose owner; public ModelNode remains source identity only.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct NodePoseSource {
    pub rig: Entity,
    pub index: usize,
}

/// Query adapter validates `rig.nodes[index] == entity` before returning a pose.
/// A node's posed matrix maps model coordinates into its owning rig basis; its
/// informational Transform and ordinary ChildOf chain must not also be applied.
pub(crate) trait CurrentTransformSource {
    fn edge(&self, entity: Entity) -> Option<(Option<Entity>, Mat4)>;
}

#[derive(Default)]
pub(crate) struct CurrentAffines {
    resolved: HashMap<Entity, Option<Mat4>>,
    path: Vec<(Entity, Mat4)>,
    visiting: HashSet<Entity>,
}
impl CurrentAffines {
    /// Called once per early effect step; allocation capacity survives frames.
    pub fn begin_frame(&mut self) {
        self.resolved.clear();
    }
    pub fn resolve(
        &mut self,
        entity: Entity,
        source: &impl CurrentTransformSource,
    ) -> Option<Mat4> {
        self.path.clear();
        self.visiting.clear();
        let mut current = entity;
        let mut result = loop {
            if let Some(result) = self.resolved.get(&current) {
                break *result;
            }
            if self.path.len() == 4096 || !self.visiting.insert(current) {
                break None;
            }
            let Some((parent, local)) = source.edge(current) else {
                self.resolved.insert(current, None);
                break None;
            };
            if !local.is_finite() {
                self.resolved.insert(current, None);
                break None;
            }
            self.path.push((current, local));
            match parent {
                Some(parent) => current = parent,
                None => break Some(Mat4::IDENTITY),
            }
        };
        for (entity, local) in self.path.drain(..).rev() {
            result = result.and_then(|parent| {
                let world = parent * local;
                world.is_finite().then_some(world)
            });
            self.resolved.insert(entity, result);
        }
        result
    }
}

#[cfg(test)]
#[path = "current_affine_tests.rs"]
mod tests;
