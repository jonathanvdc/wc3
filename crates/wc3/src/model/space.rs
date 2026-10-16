//! Explicit coordinate conventions for logical spatial values.
use crate::model::{Error, Model};
use glam::{Mat3, Mat4, Vec3};
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

impl Axis {
    /// Unit vector along this signed axis.
    pub fn vector(self) -> Vec3 {
        use Vec3;
        match self {
            Self::PositiveX => Vec3::X,
            Self::NegativeX => Vec3::NEG_X,
            Self::PositiveY => Vec3::Y,
            Self::NegativeY => Vec3::NEG_Y,
            Self::PositiveZ => Vec3::Z,
            Self::NegativeZ => Vec3::NEG_Z,
        }
    }
}
impl Basis {
    /// Coordinate conversion retaining handedness, including reflections.
    pub fn conversion_to(self, target: Self) -> Mat4 {
        let frame = |basis: Self| {
            let up = basis.up.vector();
            let forward = basis.forward.vector();
            let right = match basis.handedness {
                Handedness::Right => forward.cross(up),
                Handedness::Left => up.cross(forward),
            };
            Mat3::from_cols(right, up, -forward).transpose()
        };
        Mat4::from_mat3(frame(target).transpose() * frame(self))
    }
}
impl Model<'_> {
    /// Converts authored coordinates to an explicit basis and physical unit.
    /// Unspecified units use a numerical scale of one meter per unit.
    pub fn coordinate_transform(&self, target: Basis, units: LengthUnit) -> Result<Mat4, Error> {
        let meters = |unit| match unit {
            LengthUnit::Unspecified => 1.,
            LengthUnit::MetersPerUnit(value) => value,
        };
        units.validate()?;
        let scale = (meters(self.units()) / meters(units)) as f32;
        if !scale.is_finite() || scale <= 0. {
            return Err(Error::new(
                "units",
                "physical scale cannot be represented by f32 transforms",
            ));
        }
        Ok(self.basis().conversion_to(target) * Mat4::from_scale(Vec3::splat(scale)))
    }
}
