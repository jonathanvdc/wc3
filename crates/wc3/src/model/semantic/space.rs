//! Explicit coordinate conventions for logical spatial values.
use crate::model::semantic::Error;
/// Direction along a coordinate axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    /// Direction along the positive X axis.
    PositiveX,
    /// Direction along the negative X axis.
    NegativeX,
    /// Direction along the positive Y axis.
    PositiveY,
    /// Direction along the negative Y axis.
    NegativeY,
    /// Direction along the positive Z axis.
    PositiveZ,
    /// Direction along the negative Z axis.
    NegativeZ,
}
impl Axis {
    fn dimension(self) -> u8 {
        match self {
            Self::PositiveX | Self::NegativeX => 0,
            Self::PositiveY | Self::NegativeY => 1,
            Self::PositiveZ | Self::NegativeZ => 2,
        }
    }
}
/// Orientation of the coordinate basis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handedness {
    /// Left-handed coordinate orientation.
    Left,
    /// Right-handed coordinate orientation.
    Right,
}
/// Coordinate basis. Up and forward are distinct axes by construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Basis {
    up: Axis,
    forward: Axis,
    handedness: Handedness,
}
impl Basis {
    /// Constructs a basis with independent up and forward axes.
    pub fn new(up: Axis, forward: Axis, handedness: Handedness) -> Result<Self, Error> {
        if up.dimension() == forward.dimension() {
            return Err(Error::new(
                "basis",
                "up and forward must be independent axes",
            ));
        }
        Ok(Self {
            up,
            forward,
            handedness,
        })
    }
    /// Direction considered upward.
    pub fn up(self) -> Axis {
        self.up
    }
    /// Direction considered forward.
    pub fn forward(self) -> Axis {
        self.forward
    }
    /// Orientation used to interpret cross products.
    pub fn handedness(self) -> Handedness {
        self.handedness
    }
}
impl Default for Basis {
    fn default() -> Self {
        Self {
            up: Axis::PositiveZ,
            forward: Axis::PositiveY,
            handedness: Handedness::Right,
        }
    }
}
/// Physical interpretation of a logical distance unit, when declared by its author.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum LengthUnit {
    /// No physical distance scale has been declared.
    #[default]
    Unspecified,
    /// Number of meters represented by one coordinate unit.
    MetersPerUnit(f64),
}
impl LengthUnit {
    pub(crate) fn validate(self) -> Result<(), Error> {
        match self {
            Self::MetersPerUnit(v) if !v.is_finite() || v <= 0. => Err(Error::new(
                "units",
                "meters per unit must be finite and positive",
            )),
            _ => Ok(()),
        }
    }
}
