//! Animation failures in checked semantic models.
use crate::model::{AnimationId, ClipId, ClockId, Interpolation, Time, ValueSlot};
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

/// A timeline, curve or animation-composition constraint.
///
/// Element and control positions are carried by the enclosing error location.
/// Custom curve values retain their invalid element's index without assuming a
/// numeric representation or replacing the value's domain with a string.
#[derive(Clone, Debug)]
pub enum AnimationError {
    /// A timeline input is not a finite number of seconds.
    NonFiniteTime {
        #[doc = "Rejected numeric input."]
        value: f64,
    },
    /// A clip duration is negative.
    NegativeDuration {
        #[doc = "Rejected numeric input."]
        value: f64,
    },
    /// A repeating clock period is not positive.
    NonPositivePeriod {
        #[doc = "Rejected numeric input."]
        value: f64,
    },
    /// The requested interpolation is unavailable for this value domain.
    UnsupportedInterpolation {
        #[doc = "Requested interpolation shape."]
        interpolation: Interpolation,
        #[doc = "Whether the value domain supports linear interpolation."]
        supports_linear: bool,
        #[doc = "Whether the value domain supports spline interpolation."]
        supports_spline: bool,
        #[doc = "Whether the value domain supports rotational interpolation."]
        supports_rotation: bool,
    },
    /// Spherical controls require rotational Hermite or Bezier interpolation.
    SphericalControlsRequireQuaternionSpline {
        #[doc = "Requested interpolation shape."]
        interpolation: Interpolation,
        #[doc = "Whether the value domain supports rotational interpolation."]
        supports_rotation: bool,
    },
    /// Separate spline controls were supplied to another interpolation shape.
    SplineControlsInterpolation {
        #[doc = "Requested interpolation shape."]
        interpolation: Interpolation,
    },
    /// Separate key arrays have different lengths.
    KeyValueCount {
        #[doc = "Number of authored key times."]
        keys: usize,
        #[doc = "Number of authored key values."]
        values: usize,
    },
    /// A key precedes the previous authored key.
    UnorderedKeys {
        #[doc = "Previous authored key time."]
        previous: Time,
        #[doc = "Out-of-order authored key time."]
        current: Time,
    },
    /// A key fails its value domain's validity check.
    InvalidKeyValue,
    /// One spline control array has a different length from the key array.
    ControlCount {
        #[doc = "Number of authored key times."]
        keys: usize,
        #[doc = "Number of authored controls."]
        controls: usize,
    },
    /// A spline control fails its control domain's validity check.
    InvalidControlValue,
    /// The TCB parameter array has a different length from the key array.
    TcbControlCount {
        #[doc = "Number of authored key times."]
        keys: usize,
        #[doc = "Number of authored controls."]
        controls: usize,
    },
    /// An authored TCB coefficient is not finite.
    NonFiniteTcbParameter {
        #[doc = "Rejected numeric input."]
        value: f32,
    },
    /// The requested clip is not owned by the sampled programme.
    UnknownClip {
        #[doc = "Actual requested clip identity."]
        clip: ClipId,
    },
    /// A composition maps one destination clip more than once.
    DuplicateDestinationClip {
        #[doc = "Actual requested clip identity."]
        clip: ClipId,
    },
    /// Mapped source intervals begin at different authored times.
    ClipStartMismatch {
        #[doc = "Destination clip identity."]
        destination: ClipId,
        #[doc = "Source clip identity."]
        source: ClipId,
        #[doc = "Authored start of the destination interval."]
        destination_start: Time,
        #[doc = "Authored start of the source interval."]
        source_start: Time,
    },
    /// A shared clock identity carries different periods in the two programmes.
    ClockPeriodConflict {
        #[doc = "Conflicting shared clock identity."]
        clock: ClockId,
        #[doc = "Period already owned by the destination programme."]
        existing: Time,
        #[doc = "Period supplied by the source programme."]
        incoming: Time,
    },
    /// The two composed programmes contain the same animation identity.
    AnimationIdentityConflict {
        #[doc = "Overlapping animation identity."]
        animation: AnimationId,
    },
    /// Both composed programmes own tracks writing the same property.
    PropertyOwnershipConflict {
        #[doc = "Property written by both programmes."]
        slot: ValueSlot,
    },
}
impl Display for AnimationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteTime { value } => write!(f, "seconds must be finite, found {value}"),
            Self::NegativeDuration { value } => write!(f, "clip duration must be nonnegative, found {value}"),
            Self::NonPositivePeriod { value } => write!(f, "repeating clock period must be positive, found {value}"),
            Self::UnsupportedInterpolation { interpolation, .. } => write!(f, "{interpolation:?} interpolation is unsupported by the value domain or requires controls"),
            Self::SphericalControlsRequireQuaternionSpline { interpolation, .. } => write!(f, "spherical controls require a quaternion Hermite or Bezier spline, found {interpolation:?}"),
            Self::SplineControlsInterpolation { interpolation } => write!(f, "spline controls require Hermite or Bezier interpolation, found {interpolation:?}"),
            Self::KeyValueCount { keys, values } => write!(f, "expected {keys} key values, found {values}"),
            Self::UnorderedKeys { previous, current } => write!(f, "key time {} precedes {}", current.as_seconds(), previous.as_seconds()),
            Self::InvalidKeyValue => f.write_str("key fails its value domain's validity check"),
            Self::ControlCount { keys, controls } => write!(f, "expected {keys} spline controls, found {controls}"),
            Self::InvalidControlValue => f.write_str("spline control fails its value domain's validity check"),
            Self::TcbControlCount { keys, controls } => write!(f, "expected {keys} TCB parameters, found {controls}"),
            Self::NonFiniteTcbParameter { value } => write!(f, "TCB coefficient must be finite, found {value}"),
            Self::UnknownClip { clip } => write!(f, "unknown clip {clip:?}"),
            Self::DuplicateDestinationClip { clip } => write!(f, "destination clip {clip:?} is mapped more than once"),
            Self::ClipStartMismatch { destination, source, destination_start, source_start } => write!(f, "mapped clips {destination:?} and {source:?} start at different authored times ({} and {})", destination_start.as_seconds(), source_start.as_seconds()),
            Self::ClockPeriodConflict { clock, existing, incoming } => write!(f, "clock {clock:?} has conflicting periods {} and {}", existing.as_seconds(), incoming.as_seconds()),
            Self::AnimationIdentityConflict { animation } => write!(f, "animation identity {animation:?} belongs to both programmes"),
            Self::PropertyOwnershipConflict { slot } => write!(f, "property {slot:?} is owned by both programmes"),
        }
    }
}
impl StdError for AnimationError {}
