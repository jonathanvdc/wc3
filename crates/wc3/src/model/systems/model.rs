//! Model collision, customization matrices, and physics systems.
use super::model_physics::Physics;
use crate::model::BoneOverrideResourceId;
use crate::model::{Bounds, Link, NodeId, ValidateLinks, ZeroVec};
/// Sparse per-bone transform override supplied by a companion asset.
#[derive(Clone, Debug)]
pub struct BoneOverride<'a> {
    /// External companion resource identity.
    pub resource: Option<BoneOverrideResourceId>,
    /// Bones receiving the matrices, in matching order.
    pub bones: Vec<NodeId>,
    /// Model-space override transforms.
    pub transforms: ZeroVec<'a, [[f32; 4]; 4]>,
}
/// Independent authoring systems that can coexist in one logical model.
#[derive(Clone, Debug)]
pub enum System<'a> {
    /// Static collision geometry.
    Collision(Collision<'a>),
    /// Sparse customization matrices and their external resources.
    BoneOverrides(Vec<BoneOverride<'a>>),
    /// Physics simulation and constraints.
    Physics(Physics<'a>),
}
/// Collision triangles without native offsets or vertex encodings.
#[derive(Clone, Debug)]
pub struct Collision<'a> {
    /// Collision vertices in model coordinates.
    pub positions: ZeroVec<'a, [f32; 3]>,
    /// Collision face connectivity.
    pub triangles: ZeroVec<'a, [u32; 3]>,
    /// Per-face collision normals.
    pub normals: ZeroVec<'a, [f32; 3]>,
    /// Conservative collision bounds.
    pub bounds: Bounds,
}
impl ValidateLinks for BoneOverride<'_> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.resource.visit_links(v);
        self.bones.visit_links(v)
    }
}
impl ValidateLinks for System<'_> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        match self {
            Self::Collision(value) => {
                if value.bounds.validate().is_err()
                    || value
                        .triangles
                        .iter()
                        .flatten()
                        .any(|i| i as usize >= value.positions.len())
                    || (!value.normals.is_empty() && value.normals.len() != value.triangles.len())
                    || value
                        .positions
                        .iter()
                        .chain(value.normals.iter())
                        .flatten()
                        .any(|x| !x.is_finite())
                {
                    v(Link::InvalidValue)
                }
            }
            Self::BoneOverrides(values) => {
                for value in values {
                    value.visit_links(v);
                    if value.bones.len() != value.transforms.len()
                        || value
                            .transforms
                            .iter()
                            .flatten()
                            .flatten()
                            .any(|x| !x.is_finite())
                    {
                        v(Link::InvalidValue)
                    }
                }
            }
            Self::Physics(value) => {
                value.visit_links(v);
                if value.validate().is_err() {
                    v(Link::InvalidValue)
                }
            }
        }
    }
}
