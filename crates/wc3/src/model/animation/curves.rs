//! Checked curve shapes retain canonical packed arrays.
use zerovec::ZeroVec;
mod sampling;
pub use sampling::SampleValue;
mod value;
pub use value::*;
mod keyframe;
use crate::model::common::IntoZeroVec;
use crate::model::{Error, Interpolation, Link, Time};
pub use keyframe::*;

#[derive(Clone, Debug)]
enum CurveShape<'a, T: CurveValue> {
    Step,
    Linear,
    Slerp,
    Nlerp,
    Hermite(HermiteDerivatives<'a, T::Derivative>),
    Bezier(BezierControls<'a, T::ControlPoint>),
    Tcb(ZeroVec<'a, TcbParameters>),
}
/// Checked, ordered typed keys with exactly the controls required by interpolation.
///
/// Separate arrays retain compatible borrowed packed storage. Keyframe constructors
/// deinterleave authored keys into packed time, value, and control arrays.
/// Coincident keys preserve an authored discontinuity: the first supplies the
/// incoming limit and the last supplies the boundary value and outgoing segment.
///
/// Discrete properties accept only stepped curves:
/// ```compile_fail
/// use wc3::model::{Curve, Time};
/// let curve = Curve::linear([Time::default()], [true]);
/// ```
/// Spline interpolation also requires a continuous value domain:
/// ```compile_fail
/// use wc3::model::{Curve, Time};
/// let curve = Curve::hermite([Time::default()], [true], [false], [false]);
/// ```
/// Texture selections cannot be interpolated:
/// ```compile_fail
/// use wc3::model::{Curve, Texture, Time};
/// fn invalid(texture: Texture) {
///     let curve = Curve::linear([Time::default()], [texture]);
/// }
/// ```
/// Spherical interpolation requires rotations:
/// ```compile_fail
/// use wc3::model::{Curve, Time};
/// let curve = Curve::slerp([Time::default()], [1.0f32]);
/// ```
#[derive(Clone, Debug)]
pub struct Curve<'a, T: CurveValue> {
    times: ZeroVec<'a, Time>,
    values: ZeroVec<'a, T>,
    shape: CurveShape<'a, T>,
    spherical_controls: bool,
}
impl<'a, T: CurveValue> Curve<'a, T> {
    fn checked(
        times: ZeroVec<'a, Time>,
        values: ZeroVec<'a, T>,
        shape: CurveShape<'a, T>,
    ) -> Result<Self, Error> {
        let curve = Self {
            times,
            values,
            shape,
            spherical_controls: false,
        };
        curve.validate()?;
        Ok(curve)
    }
    /// Constructs stepped values, including discrete resource selections.
    pub fn step(
        times: impl IntoZeroVec<'a, Time>,
        values: impl IntoZeroVec<'a, T>,
    ) -> Result<Self, Error> {
        Self::checked(
            times.into_zero_vec(),
            values.into_zero_vec(),
            CurveShape::Step,
        )
    }
    /// Constructs stepped values from typed keyframes, preserving input order.
    pub fn step_keys(keys: impl IntoIterator<Item = ValueKeyframe<T>>) -> Result<Self, Error> {
        let keys: Vec<_> = keys.into_iter().collect();
        Self::step(
            keys.iter().map(|k| k.time).collect::<ZeroVec<_>>(),
            keys.iter().map(|k| k.value).collect::<ZeroVec<_>>(),
        )
    }
    /// Detaches packed key and control arrays from their backing input.
    /// Already owned arrays move without rebuilding their contents.
    pub fn into_owned(self) -> Curve<'static, T> {
        Curve {
            spherical_controls: self.spherical_controls,
            times: self.times.into_owned(),
            values: self.values.into_owned(),
            shape: match self.shape {
                CurveShape::Step => CurveShape::Step,
                CurveShape::Linear => CurveShape::Linear,
                CurveShape::Slerp => CurveShape::Slerp,
                CurveShape::Nlerp => CurveShape::Nlerp,
                CurveShape::Hermite(v) => CurveShape::Hermite(HermiteDerivatives {
                    incoming: v.incoming.into_owned(),
                    outgoing: v.outgoing.into_owned(),
                }),
                CurveShape::Bezier(v) => CurveShape::Bezier(BezierControls {
                    incoming: v.incoming.into_owned(),
                    outgoing: v.outgoing.into_owned(),
                }),
                CurveShape::Tcb(v) => CurveShape::Tcb(v.into_owned()),
            },
        }
    }
    /// Whether spline controls are rotations combined with nested spherical
    /// interpolation, rather than component derivatives or control points.
    pub fn uses_spherical_controls(&self) -> bool {
        self.spherical_controls
    }
    pub(crate) fn with_spherical_controls(mut self, spherical: bool) -> Result<Self, Error> {
        if spherical
            && (!T::Domain::SUPPORTS_ROTATION
                || !matches!(self.shape, CurveShape::Hermite(_) | CurveShape::Bezier(_)))
        {
            return Err(Error::new(
                "curve.controls",
                "spherical controls require a quaternion spline",
            ));
        }
        self.spherical_controls = spherical;
        Ok(self)
    }
    /// Ordered key times.
    pub fn times(&self) -> &ZeroVec<'a, Time> {
        &self.times
    }
    /// Values matching key times.
    pub fn values(&self) -> &ZeroVec<'a, T> {
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
    pub fn tcb_parameters(&self) -> Option<&ZeroVec<'a, TcbParameters>> {
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
        times: impl IntoZeroVec<'a, Time>,
        values: impl IntoZeroVec<'a, T>,
        mode: Interpolation,
    ) -> Result<Self, Error> {
        let shape = match mode {
            Interpolation::Step => CurveShape::Step,
            Interpolation::Linear if T::Domain::SUPPORTS_LINEAR => CurveShape::Linear,
            Interpolation::Slerp if T::Domain::SUPPORTS_ROTATION => CurveShape::Slerp,
            Interpolation::Nlerp if T::Domain::SUPPORTS_ROTATION => CurveShape::Nlerp,
            _ => {
                return Err(Error::new(
                    "curve.interpolation",
                    "sampling directive is unsupported by this value domain or requires controls",
                ));
            }
        };
        Self::checked(times.into_zero_vec(), values.into_zero_vec(), shape)
    }
    pub(crate) fn from_spline_samples(
        times: impl IntoZeroVec<'a, Time>,
        values: impl IntoZeroVec<'a, T>,
        incoming: impl IntoZeroVec<'a, T>,
        outgoing: impl IntoZeroVec<'a, T>,
        mode: Interpolation,
    ) -> Result<Self, Error> {
        if !T::Domain::SUPPORTS_SPLINE {
            return Err(Error::new(
                "curve.interpolation",
                "this value domain does not support splines",
            ));
        }
        let incoming = incoming.into_zero_vec();
        let outgoing = outgoing.into_zero_vec();
        let shape = match mode {
            Interpolation::Hermite => CurveShape::Hermite(HermiteDerivatives {
                incoming: incoming
                    .iter()
                    .map(T::into_derivative)
                    .collect::<ZeroVec<_>>(),
                outgoing: outgoing
                    .iter()
                    .map(T::into_derivative)
                    .collect::<ZeroVec<_>>(),
            }),
            Interpolation::Bezier => CurveShape::Bezier(BezierControls {
                incoming: incoming
                    .iter()
                    .map(T::into_control_point)
                    .collect::<ZeroVec<_>>(),
                outgoing: outgoing
                    .iter()
                    .map(T::into_control_point)
                    .collect::<ZeroVec<_>>(),
            }),
            _ => {
                return Err(Error::new(
                    "curve.interpolation",
                    "spline controls require Hermite or Bezier interpolation",
                ));
            }
        };
        Self::checked(times.into_zero_vec(), values.into_zero_vec(), shape)
    }
    fn validate(&self) -> Result<(), Error> {
        if self.times.len() != self.values.len() {
            return Err(Error::new("curve", "key and value cardinalities differ"));
        }
        let mut previous = None;
        for time in self.times.iter() {
            if !time.as_seconds().is_finite() || previous.is_some_and(|p| time < p) {
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
    fn validate_controls<U: CurveValue>(&self, values: &ZeroVec<'_, U>) -> Result<(), Error> {
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
        times: impl IntoZeroVec<'a, Time>,
        values: impl IntoZeroVec<'a, T>,
    ) -> Result<Self, Error> {
        Self::checked(
            times.into_zero_vec(),
            values.into_zero_vec(),
            CurveShape::Linear,
        )
    }
    /// Constructs linear interpolation from typed keyframes.
    pub fn linear_keys(keys: impl IntoIterator<Item = ValueKeyframe<T>>) -> Result<Self, Error> {
        let keys: Vec<_> = keys.into_iter().collect();
        Self::linear(
            keys.iter().map(|k| k.time).collect::<ZeroVec<_>>(),
            keys.iter().map(|k| k.value).collect::<ZeroVec<_>>(),
        )
    }
}
impl<'a, T: RotationValue> Curve<'a, T> {
    /// Constructs a rotation spline by blending the endpoint and control arcs
    /// with the weight `2 t (1 - t)`. Zero controls normalize to identity.
    pub fn spherical_hermite(
        times: impl IntoZeroVec<'a, Time>,
        values: impl IntoZeroVec<'a, T>,
        incoming: impl IntoZeroVec<'a, T::Derivative>,
        outgoing: impl IntoZeroVec<'a, T::Derivative>,
    ) -> Result<Self, Error>
    where
        T: SplineValue,
    {
        Self::hermite(times, values, incoming, outgoing)?.with_spherical_controls(true)
    }
    /// Constructs the spherical control interpretation used by native rotation
    /// Bezier tracks. Evaluation blends the endpoint and control arcs.
    pub fn spherical_bezier(
        times: impl IntoZeroVec<'a, Time>,
        values: impl IntoZeroVec<'a, T>,
        incoming: impl IntoZeroVec<'a, T::ControlPoint>,
        outgoing: impl IntoZeroVec<'a, T::ControlPoint>,
    ) -> Result<Self, Error>
    where
        T: SplineValue,
    {
        Self::bezier(times, values, incoming, outgoing)?.with_spherical_controls(true)
    }

    /// Constructs explicit spherical interpolation.
    pub fn slerp(
        times: impl IntoZeroVec<'a, Time>,
        values: impl IntoZeroVec<'a, T>,
    ) -> Result<Self, Error> {
        Self::checked(
            times.into_zero_vec(),
            values.into_zero_vec(),
            CurveShape::Slerp,
        )
    }
    /// Constructs explicit normalized linear interpolation.
    pub fn nlerp(
        times: impl IntoZeroVec<'a, Time>,
        values: impl IntoZeroVec<'a, T>,
    ) -> Result<Self, Error> {
        Self::checked(
            times.into_zero_vec(),
            values.into_zero_vec(),
            CurveShape::Nlerp,
        )
    }
    /// Constructs spherical interpolation from typed keyframes.
    pub fn slerp_keys(keys: impl IntoIterator<Item = ValueKeyframe<T>>) -> Result<Self, Error> {
        let keys: Vec<_> = keys.into_iter().collect();
        Self::slerp(
            keys.iter().map(|k| k.time).collect::<ZeroVec<_>>(),
            keys.iter().map(|k| k.value).collect::<ZeroVec<_>>(),
        )
    }
    /// Constructs normalized linear interpolation from typed keyframes.
    pub fn nlerp_keys(keys: impl IntoIterator<Item = ValueKeyframe<T>>) -> Result<Self, Error> {
        let keys: Vec<_> = keys.into_iter().collect();
        Self::nlerp(
            keys.iter().map(|k| k.time).collect::<ZeroVec<_>>(),
            keys.iter().map(|k| k.value).collect::<ZeroVec<_>>(),
        )
    }
}
impl<'a, T: SplineValue> Curve<'a, T> {
    /// Constructs a Hermite curve with derivatives in their mathematical domain.
    pub fn hermite(
        times: impl IntoZeroVec<'a, Time>,
        values: impl IntoZeroVec<'a, T>,
        incoming: impl IntoZeroVec<'a, T::Derivative>,
        outgoing: impl IntoZeroVec<'a, T::Derivative>,
    ) -> Result<Self, Error> {
        Self::checked(
            times.into_zero_vec(),
            values.into_zero_vec(),
            CurveShape::Hermite(HermiteDerivatives {
                incoming: incoming.into_zero_vec(),
                outgoing: outgoing.into_zero_vec(),
            }),
        )
    }
    /// Constructs a Bezier curve with component control points.
    pub fn bezier(
        times: impl IntoZeroVec<'a, Time>,
        values: impl IntoZeroVec<'a, T>,
        incoming: impl IntoZeroVec<'a, T::ControlPoint>,
        outgoing: impl IntoZeroVec<'a, T::ControlPoint>,
    ) -> Result<Self, Error> {
        Self::checked(
            times.into_zero_vec(),
            values.into_zero_vec(),
            CurveShape::Bezier(BezierControls {
                incoming: incoming.into_zero_vec(),
                outgoing: outgoing.into_zero_vec(),
            }),
        )
    }
    /// Constructs a TCB curve retaining its authored parameters.
    pub fn tcb(
        times: impl IntoZeroVec<'a, Time>,
        values: impl IntoZeroVec<'a, T>,
        parameters: impl IntoZeroVec<'a, TcbParameters>,
    ) -> Result<Self, Error> {
        Self::checked(
            times.into_zero_vec(),
            values.into_zero_vec(),
            CurveShape::Tcb(parameters.into_zero_vec()),
        )
    }
    /// Constructs Hermite interpolation from typed keyframes.
    pub fn hermite_keys(keys: impl IntoIterator<Item = HermiteKeyframe<T>>) -> Result<Self, Error> {
        let keys: Vec<_> = keys.into_iter().collect();
        Self::hermite(
            keys.iter().map(|k| k.time).collect::<ZeroVec<_>>(),
            keys.iter().map(|k| k.value).collect::<ZeroVec<_>>(),
            keys.iter().map(|k| k.incoming).collect::<ZeroVec<_>>(),
            keys.iter().map(|k| k.outgoing).collect::<ZeroVec<_>>(),
        )
    }
    /// Constructs Bezier interpolation from typed keyframes.
    pub fn bezier_keys(keys: impl IntoIterator<Item = BezierKeyframe<T>>) -> Result<Self, Error> {
        let keys: Vec<_> = keys.into_iter().collect();
        Self::bezier(
            keys.iter().map(|k| k.time).collect::<ZeroVec<_>>(),
            keys.iter().map(|k| k.value).collect::<ZeroVec<_>>(),
            keys.iter().map(|k| k.incoming).collect::<ZeroVec<_>>(),
            keys.iter().map(|k| k.outgoing).collect::<ZeroVec<_>>(),
        )
    }
    /// Constructs TCB interpolation from typed keyframes.
    pub fn tcb_keys(keys: impl IntoIterator<Item = TcbKeyframe<T>>) -> Result<Self, Error> {
        let keys: Vec<_> = keys.into_iter().collect();
        Self::tcb(
            keys.iter().map(|k| k.time).collect::<ZeroVec<_>>(),
            keys.iter().map(|k| k.value).collect::<ZeroVec<_>>(),
            keys.iter().map(|k| k.parameters).collect::<ZeroVec<_>>(),
        )
    }
}

#[cfg(test)]
mod tests;
