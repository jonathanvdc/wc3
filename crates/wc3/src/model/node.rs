//! Hierarchy and authored transform behavior.
use crate::model::profiles::{Profile, Universal};
use crate::model::{Error, NodeId, Quaternion};
use std::borrow::Cow;
/// Transform inheritance along a parent relationship.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Inheritance {
    /// Inherit translation.
    pub translation: bool,
    /// Inherit rotation.
    pub rotation: bool,
    /// Inherit scale.
    pub scale: bool,
}
impl Default for Inheritance {
    fn default() -> Self {
        Self {
            translation: true,
            rotation: true,
            scale: true,
        }
    }
}
/// Authored billboard behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Billboard {
    /// No camera-facing correction.
    #[default]
    None,
    /// Faces the driving camera.
    Spherical,
    /// Faces the camera while retaining selected authored axes.
    Locked {
        /// Whether camera-facing correction is active; axis choices remain authored when disabled.
        enabled: bool,
        /// Retain the X axis.
        x: bool,
        /// Retain the Y axis.
        y: bool,
        /// Retain the Z axis.
        z: bool,
    },
}
/// Authored rest transform with a pivot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    /// Translation.
    pub translation: [f32; 3],
    /// Rotation in x/y/z/w order.
    pub rotation: Quaternion,
    /// Scale.
    pub scale: [f32; 3],
    /// Pivot relative to the parent frame.
    pub pivot: [f32; 3],
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: [0.; 3],
            rotation: Quaternion::IDENTITY,
            scale: [1.; 3],
            pivot: [0.; 3],
        }
    }
}
impl Transform {
    pub(crate) fn validate(&self) -> Result<(), Error> {
        if self
            .translation
            .iter()
            .chain(&self.rotation.0)
            .chain(&self.scale)
            .chain(&self.pivot)
            .all(|x| x.is_finite())
            && self.rotation.is_rotation()
        {
            Ok(())
        } else {
            Err(Error::new(
                "transform",
                "transform must be finite and have a nonzero rotation",
            ))
        }
    }
}
/// A node definition. Insertion checks its relationship to the graph.
#[derive(Clone, Debug)]
pub struct Node<'a, P: Profile = Universal> {
    /// Additional owner-local behavior permitted by the vocabulary.
    pub controls: P::Node<'a>,
    /// Node name.
    pub name: Cow<'a, str>,
    /// Optional parent.
    pub parent: Option<NodeId>,
    /// Authored transform.
    pub transform: Transform,
    /// Parent transform inheritance.
    pub inheritance: Inheritance,
    /// Camera-facing behavior.
    pub billboard: Billboard,
    /// Whether this node is initially visible.
    pub visible: bool,
    /// Whether its origin follows the driving camera.
    pub camera_anchored: bool,
}
impl<'a, P: Profile> Node<'a, P> {
    /// Creates a visible root node with an identity transform.
    pub fn named(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: name.into(),
            controls: Default::default(),
            parent: None,
            transform: Transform::default(),
            inheritance: Inheritance::default(),
            billboard: Billboard::None,
            visible: true,
            camera_anchored: false,
        }
    }
    /// Supplies hierarchy behavior from this graph's vocabulary.
    pub fn with_controls(mut self, controls: impl Into<P::Node<'a>>) -> Self {
        self.controls = controls.into();
        self
    }
    /// Attaches the definition to a parent; graph insertion checks ownership.
    pub fn with_parent(mut self, parent: NodeId) -> Self {
        self.parent = Some(parent);
        self
    }
    /// Applies a checked authored transform.
    pub fn with_transform(mut self, transform: Transform) -> Result<Self, Error> {
        transform.validate()?;
        self.transform = transform;
        Ok(self)
    }
}
