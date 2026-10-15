//! Mathematical value domains and interpolation capabilities.
use crate::model::semantic::{Link, Quaternion, TextureId};
use std::fmt::Debug;
use zerovec::ule::AsULE;

mod private {
    pub trait Domain {}
    pub trait LinearDomain: Domain {}
    pub trait SplineDomain: LinearDomain {}
    pub trait RotationDomain: SplineDomain {}
    pub trait LinearValue {}
    pub trait SplineValue {}
    pub trait RotationValue {}
}

/// Interpolation capabilities selected once by a value domain.
///
/// This sealed vocabulary keeps native decoding and constructor bounds consistent.
/// Custom [`CurveValue`] implementations choose one of the four built-in domains.
pub trait CurveDomain: private::Domain + Clone + Send + Sync + Debug + 'static {
    /// Whether neighboring values can be interpolated linearly.
    const SUPPORTS_LINEAR: bool;
    /// Whether Hermite, Bezier, and TCB controls are meaningful.
    const SUPPORTS_SPLINE: bool;
    /// Whether explicit spherical interpolation is meaningful.
    const SUPPORTS_ROTATION: bool;
}
/// Held values, including boolean gates and resource selections.
#[derive(Clone, Copy, Debug, Default)]
pub struct DiscreteDomain;
/// Numeric values with linear interpolation and exact stored components.
#[derive(Clone, Copy, Debug, Default)]
pub struct NumericDomain;
/// Continuous scalar or vector values supporting linear and spline interpolation.
#[derive(Clone, Copy, Debug, Default)]
pub struct ContinuousDomain;
/// Rotations supporting angular linear interpolation, splines, slerp, and nlerp.
#[derive(Clone, Copy, Debug, Default)]
pub struct RotationDomain;
macro_rules! domain {
    ($ty:ty, $linear:expr, $spline:expr, $rotation:expr) => {
        impl private::Domain for $ty {}
        impl CurveDomain for $ty {
            const SUPPORTS_LINEAR: bool = $linear;
            const SUPPORTS_SPLINE: bool = $spline;
            const SUPPORTS_ROTATION: bool = $rotation;
        }
    };
}
domain!(DiscreteDomain, false, false, false);
domain!(NumericDomain, true, false, false);
domain!(ContinuousDomain, true, true, false);
domain!(RotationDomain, true, true, true);
impl private::LinearDomain for NumericDomain {}
impl private::LinearDomain for ContinuousDomain {}
impl private::LinearDomain for RotationDomain {}
impl private::SplineDomain for ContinuousDomain {}
impl private::SplineDomain for RotationDomain {}
impl private::RotationDomain for RotationDomain {}

/// A value domain supported by semantic curves.
///
/// The associated types distinguish derivatives and control points from key values.
/// Native adapters use the conversion methods when decoding format-specific controls.
pub trait CurveValue: AsULE + Clone + Send + Sync + Debug + 'static {
    /// Interpolation domain, which determines all constructor capabilities.
    type Domain: CurveDomain;
    /// Hermite derivative domain. Quaternion derivatives are component vectors.
    type Derivative: CurveValue;
    /// Bezier control-point domain, which need not be a valid key value.
    type ControlPoint: CurveValue;
    /// Visits identities carried by this value.
    fn visit_value_links(&self, _visitor: &mut dyn FnMut(Link)) {}
    /// Checks the authored value domain.
    fn is_valid(&self) -> bool;
    /// Converts a native control represented in the key value's component layout.
    fn into_derivative(self) -> Self::Derivative;
    /// Converts a native control represented in the key value's component layout.
    fn into_control_point(self) -> Self::ControlPoint;
    /// Restores the component layout required by a native adapter.
    fn derivative_value(value: Self::Derivative) -> Self;
    /// Restores the component layout required by a native adapter.
    fn control_point_value(value: Self::ControlPoint) -> Self;
}
/// Value domains supporting linear interpolation, derived from [`CurveValue::Domain`].
pub trait LinearValue: CurveValue + private::LinearValue {}
/// Value domains supporting Hermite, Bezier, and TCB splines.
pub trait SplineValue: LinearValue + private::SplineValue {}
/// Rotation domains supporting spherical and normalized linear interpolation.
pub trait RotationValue: LinearValue + private::RotationValue {}
impl<T: CurveValue> private::LinearValue for T where T::Domain: private::LinearDomain {}
impl<T: CurveValue> LinearValue for T where T::Domain: private::LinearDomain {}
impl<T: CurveValue> private::SplineValue for T where T::Domain: private::SplineDomain {}
impl<T: CurveValue> SplineValue for T where T::Domain: private::SplineDomain {}
impl<T: CurveValue> private::RotationValue for T where T::Domain: private::RotationDomain {}
impl<T: CurveValue> RotationValue for T where T::Domain: private::RotationDomain {}

macro_rules! value_domain {
    ($ty:ty, $domain:ty, $valid:expr) => {
        impl CurveValue for $ty {
            type Domain = $domain;
            type Derivative = Self;
            type ControlPoint = Self;
            fn is_valid(&self) -> bool {
                ($valid)(self)
            }
            fn into_derivative(self) -> Self {
                self
            }
            fn into_control_point(self) -> Self {
                self
            }
            fn derivative_value(value: Self) -> Self {
                value
            }
            fn control_point_value(value: Self) -> Self {
                value
            }
        }
    };
}
value_domain!(f32, ContinuousDomain, |v: &f32| v.is_finite());
value_domain!(f64, ContinuousDomain, |v: &f64| v.is_finite());
value_domain!(u32, NumericDomain, |_: &u32| true);
value_domain!(i32, NumericDomain, |_: &i32| true);
value_domain!(u16, NumericDomain, |_: &u16| true);
value_domain!(u8, NumericDomain, |_: &u8| true);
value_domain!(bool, DiscreteDomain, |_: &bool| true);
macro_rules! array_domain {
    ($ty:ty, $domain:ty, $valid:expr) => {
        impl<const N: usize> CurveValue for [$ty; N] {
            type Domain = $domain;
            type Derivative = Self;
            type ControlPoint = Self;
            fn is_valid(&self) -> bool {
                ($valid)(self)
            }
            fn into_derivative(self) -> Self {
                self
            }
            fn into_control_point(self) -> Self {
                self
            }
            fn derivative_value(value: Self) -> Self {
                value
            }
            fn control_point_value(value: Self) -> Self {
                value
            }
        }
    };
}
array_domain!(f32, ContinuousDomain, |v: &[f32; N]| v
    .iter()
    .all(|v| v.is_finite()));
array_domain!(u32, NumericDomain, |_: &[u32; N]| true);
array_domain!(u8, NumericDomain, |_: &[u8; N]| true);
impl CurveValue for Quaternion {
    type Domain = RotationDomain;
    type Derivative = [f32; 4];
    type ControlPoint = [f32; 4];
    fn is_valid(&self) -> bool {
        self.is_rotation()
    }
    fn into_derivative(self) -> Self::Derivative {
        self.0
    }
    fn into_control_point(self) -> Self::ControlPoint {
        self.0
    }
    fn derivative_value(value: Self::Derivative) -> Self {
        Self(value)
    }
    fn control_point_value(value: Self::ControlPoint) -> Self {
        Self(value)
    }
}
macro_rules! resource_domain {
    ($ty:ty, $visit:expr) => {
        impl CurveValue for $ty {
            type Domain = DiscreteDomain;
            type Derivative = Self;
            type ControlPoint = Self;
            fn is_valid(&self) -> bool {
                true
            }
            fn visit_value_links(&self, visitor: &mut dyn FnMut(Link)) {
                ($visit)(self, visitor);
            }
            fn into_derivative(self) -> Self {
                self
            }
            fn into_control_point(self) -> Self {
                self
            }
            fn derivative_value(value: Self) -> Self {
                value
            }
            fn control_point_value(value: Self) -> Self {
                value
            }
        }
    };
}
resource_domain!(
    TextureId,
    |id: &TextureId, visitor: &mut dyn FnMut(Link)| visitor(Link::Texture(*id))
);
resource_domain!(
    Option<TextureId>,
    |id: &Option<TextureId>, visitor: &mut dyn FnMut(Link)| {
        if let Some(id) = id {
            visitor(Link::Texture(*id));
        }
    }
);
