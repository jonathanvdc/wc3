//! Model collision, customization matrices, and independent texture programs.
use super::model_physics::Physics;
use crate::model::{
    Animated, Bounds, Link, NodeId, Resource, TextureTransform, ValidateLinks, ZeroVec,
};
/// Sparse per-bone transform override supplied by a companion asset.
#[derive(Clone, Debug)]
pub struct BoneOverride<'a> {
    /// External companion resource identity.
    pub resource: Option<Resource<'a>>,
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
    /// Animated texture controls not assigned to an active surface.
    TextureAnimations(TextureAnimations),
    /// Physics simulation and constraints.
    Physics(Physics<'a>),
}
/// Independently authored texture animation controls.
#[derive(Clone, Debug, Default)]
pub struct TextureAnimations {
    /// Texture-space transformations.
    pub transforms: Vec<TextureTransform>,
    /// Animated texture opacity weights.
    pub weights: Vec<Animated<f32>>,
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
        self.bones.visit_links(v)
    }
}
impl ValidateLinks for TextureAnimations {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.transforms.visit_links(v);
        self.weights.visit_links(v)
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
            Self::TextureAnimations(value) => value.visit_links(v),
            Self::Physics(value) => {
                value.visit_links(v);
                if value.validate().is_err() {
                    v(Link::InvalidValue)
                }
            }
        }
    }
}
