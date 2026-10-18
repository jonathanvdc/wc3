//! Hierarchy and authored transform behavior.
use crate::model::profiles::{Profile, Universal};
use crate::model::{Construct, Error, NodeId, Property, PropertyValues, Quaternion, ValueSample};
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
/// Slot-backed authored transform. Pivot placement is static.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimatedTransform {
    /// Local translation slot.
    pub translation: Property<[f32; 3]>,
    /// Local rotation slot.
    pub rotation: Property<Quaternion>,
    /// Local scale slot.
    pub scale: Property<[f32; 3]>,
    /// Pivot relative to the parent frame.
    pub pivot: [f32; 3],
}
impl AnimatedTransform {
    /// Allocates the components of an authored transform.
    pub fn new(value: Transform, values: &mut PropertyValues) -> Self {
        Self {
            translation: values.add(value.translation),
            rotation: values.add(value.rotation),
            scale: values.add(value.scale),
            pivot: value.pivot,
        }
    }
    pub(crate) fn validate(&self, values: &PropertyValues) -> Result<(), Error> {
        let transform = Transform {
            translation: *values
                .get(self.translation)
                .ok_or_else(|| Error::new("transform.translation", "slot is absent"))?,
            rotation: *values
                .get(self.rotation)
                .ok_or_else(|| Error::new("transform.rotation", "slot is absent"))?,
            scale: *values
                .get(self.scale)
                .ok_or_else(|| Error::new("transform.scale", "slot is absent"))?,
            pivot: self.pivot,
        };
        transform.validate()
    }
    /// Resolves authored constants.
    pub fn defaults(&self, values: &PropertyValues) -> Transform {
        Transform {
            translation: *values.get(self.translation).expect("validated slot"),
            rotation: *values.get(self.rotation).expect("validated slot"),
            scale: *values.get(self.scale).expect("validated slot"),
            pivot: self.pivot,
        }
    }
    /// Resolves evaluated components.
    pub fn sample(&self, sample: &impl ValueSample) -> Transform {
        Transform {
            translation: sample.animated(&self.translation),
            rotation: sample.animated(&self.rotation),
            scale: sample.animated(&self.scale),
            pivot: self.pivot,
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
    pub transform: AnimatedTransform,
    /// Parent transform inheritance.
    pub inheritance: Inheritance,
    /// Camera-facing behavior.
    pub billboard: Billboard,
    /// Whether this node is initially visible.
    pub visible: Property<bool>,
    /// Whether its origin follows the driving camera.
    pub camera_anchored: bool,
}
impl<'a, P: Profile> Node<'a, P> {
    /// Creates a visible root node with an identity transform.
    pub fn named(name: impl Into<Cow<'a, str>>, values: &mut PropertyValues) -> Self {
        Self {
            name: name.into(),
            controls: Default::default(),
            parent: None,
            transform: AnimatedTransform::new(Transform::default(), values),
            inheritance: Inheritance::default(),
            billboard: Billboard::None,
            visible: values.add(true),
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
    pub fn with_transform(
        mut self,
        transform: Transform,
        values: &mut PropertyValues,
    ) -> Result<Self, Error> {
        transform.validate()?;
        values.set(self.transform.translation, transform.translation)?;
        values.set(self.transform.rotation, transform.rotation)?;
        values.set(self.transform.scale, transform.scale)?;
        self.transform.pivot = transform.pivot;
        Ok(self)
    }
}

impl Construct for AnimatedTransform {
    fn construct(values: &mut PropertyValues) -> Self {
        Self::new(Transform::default(), values)
    }
}
