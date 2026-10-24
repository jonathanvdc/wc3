//! Hierarchy and authored transform behavior.
use crate::model::error::{Location, ReferenceError, TransformError};
use crate::model::profiles::{Profile, Universal};
use crate::model::{
    Construct, CurveValue as _, Error, NodeId, Property, PropertyValues, Quat, ValueSample,
};
use glam::Vec3;
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
/// Composition and selective inheritance rules, independent of native properties.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TransformSemantics {
    /// Compose parent and pivot-adjusted local affine transforms. Selective inheritance
    /// rebuilds the inherited frame from its retained signed factors.
    #[default]
    Affine,
    /// Compose pivot-local bone frames. Selective inheritance compensates accumulated
    /// authored motion, rotation and scale inside the parent frame.
    PivotLocal,
}
/// Effective camera-facing behavior. Inactive native declarations live in profile properties.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Billboard {
    /// No camera-facing correction.
    #[default]
    None,
    /// Replace inherited rotation with the camera frame, then apply authored rotation.
    Spherical {
        /// Orientation of the authored geometry within the camera frame.
        frame: BillboardFrame,
        /// Scale retained when replacing rotation.
        scale: BillboardScale,
    },
    /// Rotate around one authored axis toward the camera viewing axis. X also reflects local Z;
    /// Y and Z do not reflect. Correction occurs in the node's authored rotation frame.
    Axial(BillboardAxis),
    /// Preserve one composed axis and reconstruct the other axes against the camera image plane.
    /// Degenerate camera frames or axes leave the composed transform unchanged.
    ScreenAxis(BillboardAxis),
    /// Retain selected XYZ Euler components of the composed rotation and take the
    /// remaining components from the camera. This is not an axial rotation constraint.
    Euler(EulerLocks),
}
/// Geometry orientation relative to the camera's image plane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BillboardFrame {
    /// Authored +X is camera right, +Y is camera up, and +Z faces the camera.
    Camera,
    /// Authored +Y is camera right, +Z is camera up, and +X faces the camera.
    Yz,
}
/// Scale treatment when a spherical billboard replaces rotation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BillboardScale {
    /// Retain accumulated signed authored scale factors.
    SignedFactors,
    /// Retain affine basis lengths measured before facing correction, with authored signs.
    BasisLengths,
    /// Retain composed affine basis lengths with signs from this node's local scale.
    LocalBasisLengths,
}
/// Selected authored axis for view-facing rotation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BillboardAxis {
    /// Authored X axis.
    X,
    /// Authored Y axis.
    Y,
    /// Authored Z axis.
    Z,
}
/// Nonempty, proper subsets of XYZ Euler components retained from authored rotation.
/// Retaining all is no facing correction and is represented by `Billboard::None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EulerLocks {
    /// Retain X.
    X,
    /// Retain Y.
    Y,
    /// Retain Z.
    Z,
    /// Retain X and Y.
    Xy,
    /// Retain X and Z.
    Xz,
    /// Retain Y and Z.
    Yz,
}
impl EulerLocks {
    /// Retained XYZ components.
    pub fn axes(self) -> [bool; 3] {
        match self {
            Self::X => [true, false, false],
            Self::Y => [false, true, false],
            Self::Z => [false, false, true],
            Self::Xy => [true, true, false],
            Self::Xz => [true, false, true],
            Self::Yz => [false, true, true],
        }
    }
}
/// Authored rest transform with a pivot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    /// Translation.
    pub translation: Vec3,
    /// Rotation in x/y/z/w order.
    pub rotation: Quat,
    /// Scale.
    pub scale: Vec3,
    /// Pivot relative to the parent frame.
    pub pivot: Vec3,
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::from_array([0.; 3]),
            rotation: Quat::IDENTITY,
            scale: Vec3::from_array([1.; 3]),
            pivot: Vec3::from_array([0.; 3]),
        }
    }
}
impl Transform {
    pub(crate) fn validate(&self) -> Result<(), Error> {
        if !self.translation.is_finite() {
            return Err(Error::at(
                Location::new().field("translation"),
                TransformError::NonFiniteVector {
                    vector: self.translation,
                },
            ));
        }
        if !self.rotation.is_valid() {
            return Err(Error::at(
                Location::new().field("rotation"),
                TransformError::InvalidRotation {
                    rotation: self.rotation,
                },
            ));
        }
        for (field, vector) in [("scale", self.scale), ("pivot", self.pivot)] {
            if !vector.is_finite() {
                return Err(Error::at(
                    Location::new().field(field),
                    TransformError::NonFiniteVector { vector },
                ));
            }
        }
        Ok(())
    }
}
/// Slot-backed authored transform. Pivot placement is static.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimatedTransform {
    /// Local translation slot.
    pub translation: Property<Vec3>,
    /// Local rotation slot.
    pub rotation: Property<Quat>,
    /// Local scale slot.
    pub scale: Property<Vec3>,
    /// Pivot relative to the parent frame.
    pub pivot: Vec3,
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
            translation: *values.get(self.translation).ok_or_else(|| {
                Error::at(
                    Location::new().field("translation"),
                    ReferenceError::Unknown {
                        target: self.translation.slot().into(),
                    },
                )
            })?,
            rotation: *values.get(self.rotation).ok_or_else(|| {
                Error::at(
                    Location::new().field("rotation"),
                    ReferenceError::Unknown {
                        target: self.rotation.slot().into(),
                    },
                )
            })?,
            scale: *values.get(self.scale).ok_or_else(|| {
                Error::at(
                    Location::new().field("scale"),
                    ReferenceError::Unknown {
                        target: self.scale.slot().into(),
                    },
                )
            })?,
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
    pub properties: P::NodeProperties<'a>,
    /// Node name.
    pub name: Cow<'a, str>,
    /// Optional parent.
    pub parent: Option<NodeId>,
    /// Authored transform.
    pub transform: AnimatedTransform,
    /// Parent transform inheritance.
    pub inheritance: Inheritance,
    /// Transform composition and selective inheritance rules.
    pub transform_semantics: TransformSemantics,
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
            properties: Default::default(),
            parent: None,
            transform: AnimatedTransform::new(Transform::default(), values),
            inheritance: Inheritance::default(),
            transform_semantics: TransformSemantics::Affine,
            billboard: Billboard::None,
            visible: values.add(true),
            camera_anchored: false,
        }
    }
    /// Supplies hierarchy behavior from this graph's vocabulary.
    pub fn with_properties(mut self, properties: impl Into<P::NodeProperties<'a>>) -> Self {
        self.properties = properties.into();
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
