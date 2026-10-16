//! Current ordinary transform chains for pre-propagation camera inputs.
use super::{CameraBinding, ModelNode};
use bevy::prelude::*;

pub(super) type HierarchyData = (
    &'static Transform,
    Option<&'static ChildOf>,
    Has<ModelNode>,
    Has<CameraBinding>,
);
/// Bound leaf cameras can supply their already evaluated local pose. Bound or
/// animated ancestors would introduce ordering/circular dependencies.
pub(super) fn current_world(
    leaf: Entity,
    hierarchy: &Query<HierarchyData>,
    scratch: &mut Vec<(Entity, Transform)>,
    allow_bound_leaf: bool,
) -> Result<GlobalTransform, &'static str> {
    scratch.clear();
    let mut next = Some(leaf);
    while let Some(entity) = next {
        if scratch.iter().any(|(seen, _)| *seen == entity) {
            return Err("cyclic transform hierarchy");
        }
        let (transform, parent, animated, bound) = hierarchy
            .get(entity)
            .map_err(|_| "missing hierarchy transform")?;
        if animated || (bound && !(allow_bound_leaf && entity == leaf)) {
            return Err("hierarchy depends on an animated node or another bound camera");
        }
        scratch.push((entity, *transform));
        next = parent.map(ChildOf::parent);
    }
    let mut result = GlobalTransform::IDENTITY;
    for (_, transform) in scratch.iter().rev() {
        result = result.mul_transform(*transform);
    }
    if !result.affine().is_finite() {
        return Err("nonfinite transform hierarchy");
    }
    Ok(result)
}
