//! Ordered program keys remain editable when their destination is externally defined.
use super::playback::BlockFlags;
use crate::model::common::AsULE;
use crate::model::{
    Bounds, CurveValue, Error, EventId, Link, Quaternion, Time, ValidateLinks, ZeroVec,
};

/// Typed keys without a sampling convention inferred from a property declaration.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderedKeys<'a, T: AsULE> {
    /// Absolute model-timeline key times, including authored pre-roll.
    pub times: ZeroVec<'a, Time>,
    /// PropertyValues in the same authored order.
    pub values: ZeroVec<'a, T>,
}
impl<T: CurveValue> OrderedKeys<'_, T> {
    /// Checks ordering, cardinality, and the value domain without inventing interpolation.
    pub fn validate(&self) -> Result<(), Error> {
        self.validate_order()?;
        if self.values.iter().any(|value| !value.is_valid()) {
            return Err(Error::new("m3.unbound.keys", "invalid ordered key data"));
        }
        Ok(())
    }
}
impl<T: AsULE> OrderedKeys<'_, T> {
    fn validate_order(&self) -> Result<(), Error> {
        if self.times.len() != self.values.len()
            || self
                .times
                .iter()
                .zip(self.times.iter().skip(1))
                .any(|(a, b)| a > b)
        {
            Err(Error::new(
                "m3.unbound.keys",
                "timestamp/value order or cardinality differs",
            ))
        } else {
            Ok(())
        }
    }
}
/// Actual mathematical domains of keys whose property header is unavailable.
#[derive(Clone, Debug, PartialEq)]
pub enum UnboundKeys<'a> {
    /// Actual controller-owned command occurrences in authored key order.
    Events(Vec<EventId>),
    /// Scalar floating-point values.
    Scalar(OrderedKeys<'a, f32>),
    /// Two-component floating-point vectors.
    Vector2(OrderedKeys<'a, [f32; 2]>),
    /// Three-component floating-point vectors.
    Vector3(OrderedKeys<'a, [f32; 3]>),
    /// Quaternion values without an inferred sampling rule.
    Rotation(OrderedKeys<'a, Quaternion>),
    /// Four exact color components in red, green, blue, alpha order.
    Color(OrderedKeys<'a, [u8; 4]>),
    /// Unsigned eight-bit values.
    Byte(OrderedKeys<'a, u8>),
    /// Signed sixteen-bit values.
    Signed16(OrderedKeys<'a, i16>),
    /// Unsigned sixteen-bit values.
    Unsigned16(OrderedKeys<'a, u16>),
    /// Signed thirty-two-bit values.
    Signed32(OrderedKeys<'a, i32>),
    /// Unsigned thirty-two-bit values.
    Unsigned32(OrderedKeys<'a, u32>),
    /// Thirty-two-bit behavior masks.
    Flags(OrderedKeys<'a, u32>),
    /// Coupled bounding volumes.
    Extent(OrderedKeys<'a, Bounds>),
}
impl UnboundKeys<'_> {
    /// Checks each key without declaring an unavailable sampling convention.
    pub fn validate(&self) -> Result<(), Error> {
        match self {
            Self::Events(_) => Ok(()),
            Self::Scalar(keys) => keys.validate(),
            Self::Vector2(keys) => keys.validate(),
            Self::Vector3(keys) => keys.validate(),
            Self::Rotation(keys) => keys.validate(),
            Self::Color(keys) => keys.validate(),
            Self::Byte(keys) => keys.validate(),
            Self::Signed16(keys) => keys.validate_order(),
            Self::Unsigned16(keys) => keys.validate(),
            Self::Signed32(keys) => keys.validate(),
            Self::Unsigned32(keys) => keys.validate(),
            Self::Flags(keys) => keys.validate(),
            Self::Extent(keys) => {
                if keys.times.len() != keys.values.len()
                    || keys
                        .times
                        .iter()
                        .zip(keys.times.iter().skip(1))
                        .any(|(a, b)| a > b)
                    || keys.values.iter().any(|value| value.validate().is_err())
                {
                    Err(Error::new("m3.unbound.extents", "invalid ordered bounds"))
                } else {
                    Ok(())
                }
            }
        }
    }
}
/// A program block whose property is external or which has no active property selection.
#[derive(Clone, Debug, PartialEq)]
pub struct UnboundBlock<'a> {
    /// External animation identity, or no identity for an unselected pool entry.
    pub identity: Option<u32>,
    /// Retained block behavior, including unnamed bits.
    pub flags: BlockFlags,
    /// Authored end of the block on the model timeline.
    pub end: Time,
    /// Known typed keys without a fabricated destination or default value.
    pub keys: UnboundKeys<'a>,
}
impl ValidateLinks for UnboundBlock<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let UnboundKeys::Events(events) = &self.keys {
            events.visit_links(visitor);
        }
        if self.end.as_seconds() < 0.0 || self.keys.validate().is_err() {
            visitor(Link::InvalidValue);
        }
    }
}
