//! Checked curve shapes retain native borrowed arrays.
mod value;
pub use value::*;
mod keyframe;
use crate::model::semantic::{Error, Interpolation, Link, Time, Values};
pub use keyframe::*;

#[derive(Clone, Debug)]
enum CurveShape<'a, T: CurveValue> {
    Step,
    Linear,
    Slerp,
    Nlerp,
    Hermite(HermiteDerivatives<'a, T::Derivative>),
    Bezier(BezierControls<'a, T::ControlPoint>),
    Tcb(Values<'a, TcbParameters>),
}
/// Checked, ordered typed keys with exactly the controls required by interpolation.
///
/// Separate arrays retain borrowed native storage. Keyframe constructors provide
/// an interleaved authoring interface without requiring an interleaved allocation.
/// Coincident keys preserve an authored discontinuity: the first supplies the
/// incoming limit and the last supplies the boundary value and outgoing segment.
///
/// Discrete properties accept only stepped curves:
/// ```compile_fail
/// use wc3::model::semantic::{Curve, Time};
/// let curve = Curve::linear([Time::default()], [true]);
/// ```
/// Spline interpolation also requires a continuous value domain:
/// ```compile_fail
/// use wc3::model::semantic::{Curve, Time};
/// let curve = Curve::hermite([Time::default()], [true], [false], [false]);
/// ```
/// Texture selections cannot be interpolated:
/// ```compile_fail
/// use wc3::model::semantic::{Curve, TextureId, Time};
/// fn invalid(texture: TextureId) {
///     let curve = Curve::linear([Time::default()], [texture]);
/// }
/// ```
/// Spherical interpolation requires rotations:
/// ```compile_fail
/// use wc3::model::semantic::{Curve, Time};
/// let curve = Curve::slerp([Time::default()], [1.0f32]);
/// ```
#[derive(Clone, Debug)]
pub struct Curve<'a, T: CurveValue> {
    times: Values<'a, Time>,
    values: Values<'a, T>,
    shape: CurveShape<'a, T>,
}
impl<'a, T: CurveValue> Curve<'a, T> {
    fn checked(
        times: Values<'a, Time>,
        values: Values<'a, T>,
        shape: CurveShape<'a, T>,
    ) -> Result<Self, Error> {
        let curve = Self {
            times,
            values,
            shape,
        };
        curve.validate()?;
        Ok(curve)
    }
    /// Constructs stepped values, including discrete resource selections.
    pub fn step(
        times: impl Into<Values<'a, Time>>,
        values: impl Into<Values<'a, T>>,
    ) -> Result<Self, Error> {
        Self::checked(times.into(), values.into(), CurveShape::Step)
    }
    /// Constructs stepped values from typed keyframes, preserving input order.
    pub fn step_keys(keys: impl Into<Values<'a, ValueKeyframe<T>>>) -> Result<Self, Error> {
        let keys = keys.into();
        Self::step(keys.clone().map(|k| k.time), keys.map(|k| k.value))
    }
    /// Ordered key times.
    pub fn times(&self) -> &Values<'a, Time> {
        &self.times
    }
    /// Values matching key times.
    pub fn values(&self) -> &Values<'a, T> {
        &self.values
    }
    /// Interpolation selected by the curve's shape.
    pub fn interpolation(&self) -> Interpolation {
        match self.shape {
            CurveShape::Step => Interpolation::Step,
            CurveShape::Linear => Interpolation::Linear,
            CurveShape::Slerp => Interpolation::Slerp,
            CurveShape::Nlerp => Interpolation::Nlerp,
            CurveShape::Hermite(_) => Interpolation::Hermite,
            CurveShape::Bezier(_) => Interpolation::Bezier,
            CurveShape::Tcb(_) => Interpolation::Tcb,
        }
    }
    /// Derivatives only when this is a Hermite curve.
    pub fn hermite_derivatives(&self) -> Option<&HermiteDerivatives<'a, T::Derivative>> {
        if let CurveShape::Hermite(v) = &self.shape {
            Some(v)
        } else {
            None
        }
    }
    /// Control points only when this is a Bezier curve.
    pub fn bezier_controls(&self) -> Option<&BezierControls<'a, T::ControlPoint>> {
        if let CurveShape::Bezier(v) = &self.shape {
            Some(v)
        } else {
            None
        }
    }
    /// Authored parameters only when this is a TCB curve.
    pub fn tcb_parameters(&self) -> Option<&Values<'a, TcbParameters>> {
        if let CurveShape::Tcb(v) = &self.shape {
            Some(v)
        } else {
            None
        }
    }
    pub(crate) fn incoming_value(&self, index: usize) -> Option<T> {
        match &self.shape {
            CurveShape::Hermite(v) => v.incoming.get(index).map(T::derivative_value),
            CurveShape::Bezier(v) => v.incoming.get(index).map(T::control_point_value),
            _ => None,
        }
    }
    pub(crate) fn outgoing_value(&self, index: usize) -> Option<T> {
        match &self.shape {
            CurveShape::Hermite(v) => v.outgoing.get(index).map(T::derivative_value),
            CurveShape::Bezier(v) => v.outgoing.get(index).map(T::control_point_value),
            _ => None,
        }
    }
    /// Decodes a native sampling directive; public authoring uses capability-bound constructors.
    pub(crate) fn from_samples(
        times: impl Into<Values<'a, Time>>,
        values: impl Into<Values<'a, T>>,
        mode: Interpolation,
    ) -> Result<Self, Error> {
        let shape =
            match mode {
                Interpolation::Step => CurveShape::Step,
                Interpolation::Linear if T::Domain::SUPPORTS_LINEAR => CurveShape::Linear,
                Interpolation::Slerp if T::Domain::SUPPORTS_ROTATION => CurveShape::Slerp,
                Interpolation::Nlerp if T::Domain::SUPPORTS_ROTATION => CurveShape::Nlerp,
                _ => return Err(Error::new(
                    "curve.interpolation",
                    "sampling directive is unsupported by this value domain or requires controls",
                )),
            };
        Self::checked(times.into(), values.into(), shape)
    }
    pub(crate) fn from_spline_samples(
        times: impl Into<Values<'a, Time>>,
        values: impl Into<Values<'a, T>>,
        incoming: impl Into<Values<'a, T>>,
        outgoing: impl Into<Values<'a, T>>,
        mode: Interpolation,
    ) -> Result<Self, Error> {
        if !T::Domain::SUPPORTS_SPLINE {
            return Err(Error::new(
                "curve.interpolation",
                "this value domain does not support splines",
            ));
        }
        let incoming = incoming.into();
        let outgoing = outgoing.into();
        let shape = match mode {
            Interpolation::Hermite => CurveShape::Hermite(HermiteDerivatives {
                incoming: incoming.map(T::into_derivative),
                outgoing: outgoing.map(T::into_derivative),
            }),
            Interpolation::Bezier => CurveShape::Bezier(BezierControls {
                incoming: incoming.map(T::into_control_point),
                outgoing: outgoing.map(T::into_control_point),
            }),
            _ => {
                return Err(Error::new(
                    "curve.interpolation",
                    "spline controls require Hermite or Bezier interpolation",
                ))
            }
        };
        Self::checked(times.into(), values.into(), shape)
    }
    fn validate(&self) -> Result<(), Error> {
        if self.times.len() != self.values.len() {
            return Err(Error::new("curve", "key and value cardinalities differ"));
        }
        let mut previous = None;
        for time in self.times.iter() {
            if previous.is_some_and(|p| time < p) {
                return Err(Error::new("curve.times", "keys must be nondecreasing"));
            }
            previous = Some(time);
        }
        if self.values.iter().any(|v| !v.is_valid()) {
            return Err(Error::new("curve.values", "invalid key value"));
        }
        match &self.shape {
            CurveShape::Hermite(v) => {
                self.validate_controls(&v.incoming)?;
                self.validate_controls(&v.outgoing)?;
            }
            CurveShape::Bezier(v) => {
                self.validate_controls(&v.incoming)?;
                self.validate_controls(&v.outgoing)?;
            }
            CurveShape::Tcb(v)
                if v.len() != self.times.len()
                    || v.iter().any(|p| {
                        ![p.tension, p.continuity, p.bias]
                            .iter()
                            .all(|v| v.is_finite())
                    }) =>
            {
                return Err(Error::new(
                    "curve.tcb",
                    "expected finite TCB parameters for every key",
                ));
            }
            _ => {}
        }
        Ok(())
    }
    fn validate_controls<U: CurveValue>(&self, values: &Values<'_, U>) -> Result<(), Error> {
        if values.len() != self.times.len() || values.iter().any(|v| !v.is_valid()) {
            return Err(Error::new(
                "curve.controls",
                "invalid spline control or cardinality",
            ));
        }
        Ok(())
    }
    pub(crate) fn visit_control_links(&self, visitor: &mut dyn FnMut(Link)) {
        match &self.shape {
            CurveShape::Hermite(v) => {
                for value in v.incoming.iter().chain(v.outgoing.iter()) {
                    value.visit_value_links(visitor);
                }
            }
            CurveShape::Bezier(v) => {
                for value in v.incoming.iter().chain(v.outgoing.iter()) {
                    value.visit_value_links(visitor);
                }
            }
            _ => {}
        }
    }
}
impl<'a, T: LinearValue> Curve<'a, T> {
    /// Constructs linear interpolation; quaternion values retain angular interpolation.
    pub fn linear(
        times: impl Into<Values<'a, Time>>,
        values: impl Into<Values<'a, T>>,
    ) -> Result<Self, Error> {
        Self::checked(times.into(), values.into(), CurveShape::Linear)
    }
    /// Constructs linear interpolation from typed keyframes.
    pub fn linear_keys(keys: impl Into<Values<'a, ValueKeyframe<T>>>) -> Result<Self, Error> {
        let keys = keys.into();
        Self::linear(keys.clone().map(|k| k.time), keys.map(|k| k.value))
    }
}
impl<'a, T: RotationValue> Curve<'a, T> {
    /// Constructs explicit spherical interpolation.
    pub fn slerp(
        times: impl Into<Values<'a, Time>>,
        values: impl Into<Values<'a, T>>,
    ) -> Result<Self, Error> {
        Self::checked(times.into(), values.into(), CurveShape::Slerp)
    }
    /// Constructs explicit normalized linear interpolation.
    pub fn nlerp(
        times: impl Into<Values<'a, Time>>,
        values: impl Into<Values<'a, T>>,
    ) -> Result<Self, Error> {
        Self::checked(times.into(), values.into(), CurveShape::Nlerp)
    }
    /// Constructs spherical interpolation from typed keyframes.
    pub fn slerp_keys(keys: impl Into<Values<'a, ValueKeyframe<T>>>) -> Result<Self, Error> {
        let keys = keys.into();
        Self::slerp(keys.clone().map(|k| k.time), keys.map(|k| k.value))
    }
    /// Constructs normalized linear interpolation from typed keyframes.
    pub fn nlerp_keys(keys: impl Into<Values<'a, ValueKeyframe<T>>>) -> Result<Self, Error> {
        let keys = keys.into();
        Self::nlerp(keys.clone().map(|k| k.time), keys.map(|k| k.value))
    }
}
impl<'a, T: SplineValue> Curve<'a, T> {
    /// Constructs a Hermite curve with derivatives in their mathematical domain.
    pub fn hermite(
        times: impl Into<Values<'a, Time>>,
        values: impl Into<Values<'a, T>>,
        incoming: impl Into<Values<'a, T::Derivative>>,
        outgoing: impl Into<Values<'a, T::Derivative>>,
    ) -> Result<Self, Error> {
        Self::checked(
            times.into(),
            values.into(),
            CurveShape::Hermite(HermiteDerivatives {
                incoming: incoming.into(),
                outgoing: outgoing.into(),
            }),
        )
    }
    /// Constructs a Bezier curve with component control points.
    pub fn bezier(
        times: impl Into<Values<'a, Time>>,
        values: impl Into<Values<'a, T>>,
        incoming: impl Into<Values<'a, T::ControlPoint>>,
        outgoing: impl Into<Values<'a, T::ControlPoint>>,
    ) -> Result<Self, Error> {
        Self::checked(
            times.into(),
            values.into(),
            CurveShape::Bezier(BezierControls {
                incoming: incoming.into(),
                outgoing: outgoing.into(),
            }),
        )
    }
    /// Constructs a TCB curve retaining its authored parameters.
    pub fn tcb(
        times: impl Into<Values<'a, Time>>,
        values: impl Into<Values<'a, T>>,
        parameters: impl Into<Values<'a, TcbParameters>>,
    ) -> Result<Self, Error> {
        Self::checked(
            times.into(),
            values.into(),
            CurveShape::Tcb(parameters.into()),
        )
    }
    /// Constructs Hermite interpolation from typed keyframes.
    pub fn hermite_keys(keys: impl Into<Values<'a, HermiteKeyframe<T>>>) -> Result<Self, Error> {
        let keys = keys.into();
        Self::hermite(
            keys.clone().map(|k| k.time),
            keys.clone().map(|k| k.value),
            keys.clone().map(|k| k.incoming),
            keys.map(|k| k.outgoing),
        )
    }
    /// Constructs Bezier interpolation from typed keyframes.
    pub fn bezier_keys(keys: impl Into<Values<'a, BezierKeyframe<T>>>) -> Result<Self, Error> {
        let keys = keys.into();
        Self::bezier(
            keys.clone().map(|k| k.time),
            keys.clone().map(|k| k.value),
            keys.clone().map(|k| k.incoming),
            keys.map(|k| k.outgoing),
        )
    }
    /// Constructs TCB interpolation from typed keyframes.
    pub fn tcb_keys(keys: impl Into<Values<'a, TcbKeyframe<T>>>) -> Result<Self, Error> {
        let keys = keys.into();
        Self::tcb(
            keys.clone().map(|k| k.time),
            keys.clone().map(|k| k.value),
            keys.map(|k| k.parameters),
        )
    }
}

#[cfg(test)]
mod tests;
