//! Rest-pose bounds in the application's Bevy coordinate frame.
use super::mesh_attributes::{EXTRA_JOINT_INDEX, EXTRA_JOINT_WEIGHT};
use super::pose::{evaluate_pose, EvaluatedPose};
use super::{CompiledUniversalModel, RuntimeSkin};
use bevy::camera::primitives::Aabb;
use bevy::mesh::{MeshVertexAttribute, VertexAttributeValues};
use bevy::prelude::*;
use std::sync::Arc;
use wc3::model::Bounds;

fn include(point: Vec3, min: &mut Vec3, max: &mut Vec3) {
    if point.is_finite() {
        *min = min.min(point);
        *max = max.max(point);
    }
}
fn skin_row(
    mesh: &Mesh,
    vertex: usize,
    indices: MeshVertexAttribute,
    weights: MeshVertexAttribute,
) -> Option<(&[u16; 4], &[f32; 4])> {
    let VertexAttributeValues::Uint16x4(indices) = mesh.attribute(indices)? else {
        return None;
    };
    let VertexAttributeValues::Float32x4(weights) = mesh.attribute(weights)? else {
        return None;
    };
    Some((indices.get(vertex)?, weights.get(vertex)?))
}
fn skinned(
    mesh: &Mesh,
    vertex: usize,
    point: Vec3,
    skin: &RuntimeSkin<Arc<[Mat4]>>,
    poses: &[EvaluatedPose],
) -> Vec3 {
    let mut result = Vec3::ZERO;
    for (indices, weights) in [
        (Mesh::ATTRIBUTE_JOINT_INDEX, Mesh::ATTRIBUTE_JOINT_WEIGHT),
        (EXTRA_JOINT_INDEX, EXTRA_JOINT_WEIGHT),
    ] {
        if let Some((indices, weights)) = skin_row(mesh, vertex, indices, weights) {
            for (&joint, &weight) in indices.iter().zip(weights) {
                if weight > 0. {
                    let joint = usize::from(joint);
                    result += (poses[skin.joints[joint]].matrix * skin.binds[joint])
                        .transform_point3(point)
                        * weight;
                }
            }
        }
    }
    result
}

pub(super) fn rest_bounds(
    model: &CompiledUniversalModel,
    authored: Option<Bounds>,
) -> Option<Aabb> {
    let basis = model.basis.to_matrix();
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    if let Some(bounds) = authored {
        let lo = Vec3::from_array(bounds.minimum);
        let hi = Vec3::from_array(bounds.maximum);
        if lo.is_finite() && hi.is_finite() && lo.cmple(hi).all() {
            for x in [lo.x, hi.x] {
                for y in [lo.y, hi.y] {
                    for z in [lo.z, hi.z] {
                        include(
                            basis.transform_point3(Vec3::new(x, y, z)),
                            &mut min,
                            &mut max,
                        );
                    }
                }
            }
        }
    }
    let mut poses = vec![EvaluatedPose::default(); model.nodes.len()];
    for &index in model.order.iter() {
        let node = &model.nodes[index];
        poses[index] = evaluate_pose(
            node,
            node.rest,
            node.parent
                .map_or_else(EvaluatedPose::default, |parent| poses[parent]),
            None,
        );
    }
    for draw in &model.draws {
        let Some(VertexAttributeValues::Float32x3(positions)) =
            draw.mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            continue;
        };
        for (vertex, &position) in positions.iter().enumerate() {
            let position = Vec3::from_array(position);
            let point = if let Some(skin) = draw.skin {
                skinned(&draw.mesh, vertex, position, &model.skins[skin], &poses)
            } else {
                poses[draw.node].matrix.transform_point3(position)
            };
            include(basis.transform_point3(point), &mut min, &mut max);
        }
    }
    for liquid in &model.liquids {
        let Some(VertexAttributeValues::Float32x3(positions)) =
            liquid.mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            continue;
        };
        let frame = basis * poses[liquid.node].matrix;
        for &position in positions {
            include(
                frame.transform_point3(Vec3::from_array(position)),
                &mut min,
                &mut max,
            );
        }
    }
    (min.is_finite() && max.is_finite()).then(|| Aabb::from_min_max(min, max))
}

pub(super) fn transform_bounds(bounds: Aabb, frame: Mat4) -> Aabb {
    let lo = Vec3::from(bounds.center - bounds.half_extents);
    let hi = Vec3::from(bounds.center + bounds.half_extents);
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for x in [lo.x, hi.x] {
        for y in [lo.y, hi.y] {
            for z in [lo.z, hi.z] {
                include(
                    frame.transform_point3(Vec3::new(x, y, z)),
                    &mut min,
                    &mut max,
                );
            }
        }
    }
    Aabb::from_min_max(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::PrimitiveTopology;
    #[test]
    fn rest_skin_bounds_evaluate_the_extra_four_influences() {
        let mut mesh = Mesh::new(PrimitiveTopology::PointList, RenderAssetUsages::default());
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_JOINT_INDEX,
            VertexAttributeValues::Uint16x4(vec![[0; 4]]),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, vec![[0.5, 0., 0., 0.]]);
        mesh.insert_attribute(
            EXTRA_JOINT_INDEX,
            VertexAttributeValues::Uint16x4(vec![[1; 4]]),
        );
        mesh.insert_attribute(EXTRA_JOINT_WEIGHT, vec![[0.5, 0., 0., 0.]]);
        let mut poses = [EvaluatedPose::default(); 2];
        poses[1].matrix = Mat4::from_translation(Vec3::new(100., 0., 0.));
        let skin = RuntimeSkin {
            joints: vec![0, 1],
            binds: Arc::from([Mat4::IDENTITY; 2]),
        };
        assert_eq!(
            skinned(&mesh, 0, Vec3::X, &skin, &poses),
            Vec3::new(51., 0., 0.)
        );
    }
}
