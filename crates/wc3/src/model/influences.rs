//! Owned geometry distributions preserve slot structure and blending interpretation.
use super::vertex_geometry::RemapStreams;
use super::{Error, Influence, InfluenceRows, JointInfluences, PackedArray};
use crate::model::common::IntoOwned;
use crate::model::error::{InfluenceKind, Location, ProfileError, ProfileKind, ProfileViolation};
use crate::model::ValueGroups;

/// Authored joint distributions in the shared geometry representation.
#[derive(Clone, Debug, Default, IntoOwned)]
pub enum GeometryInfluences<'data> {
    /// No authored skinning distribution.
    #[default]
    None,
    /// Variable-length normalized weighted rows.
    Variable(ValueGroups<'data, Influence>),
    /// Three ordered weighted slots, including zero-weight slots.
    Three(PackedArray<'data, [Influence; 3]>),
    /// Four ordered weighted slots, including zero-weight slots.
    Four(PackedArray<'data, [Influence; 4]>),
    /// Equal blending among ordered joints, without inferred explicit weights.
    MatrixGroups(ValueGroups<'data, u32>),
}
impl<'data> GeometryInfluences<'data> {
    /// Borrows the exact authored distribution representation.
    pub fn rows(&self) -> InfluenceRows<'_, 'data> {
        match self {
            Self::None => InfluenceRows::None,
            Self::Variable(value) => InfluenceRows::Variable(value),
            Self::Three(value) => InfluenceRows::Three(value),
            Self::Four(value) => InfluenceRows::Four(value),
            Self::MatrixGroups(value) => InfluenceRows::MatrixGroups(value),
        }
    }
    /// Number of authored vertex distributions.
    pub fn len(&self) -> usize {
        self.rows().len()
    }
    /// Whether no vertex distributions are authored.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub(crate) fn remap(&mut self, indices: &[u32]) {
        match self {
            Self::None => {}
            Self::Variable(value) => value.remap_streams(indices),
            Self::Three(value) => value.remap_streams(indices),
            Self::Four(value) => value.remap_streams(indices),
            Self::MatrixGroups(value) => value.remap_streams(indices),
        }
    }
}
impl<'data> From<JointInfluences<'data>> for GeometryInfluences<'data> {
    fn from(value: JointInfluences<'data>) -> Self {
        match value {
            JointInfluences::None => Self::None,
            JointInfluences::Four(value) => Self::Four(value),
            JointInfluences::MatrixGroups(value) => Self::MatrixGroups(value),
        }
    }
}
impl<'data> TryFrom<GeometryInfluences<'data>> for JointInfluences<'data> {
    type Error = Error;
    fn try_from(value: GeometryInfluences<'data>) -> Result<Self, Error> {
        match value {
            GeometryInfluences::None => Ok(Self::None),
            GeometryInfluences::Four(value) => Ok(Self::Four(value)),
            GeometryInfluences::MatrixGroups(value) => Ok(Self::MatrixGroups(value)),
            value => Err(Error::at(
                Location::new().field("influences"),
                ProfileError::new(
                    ProfileKind::Mdlx,
                    ProfileViolation::GeometryInfluences {
                        actual: InfluenceKind::from(&value),
                    },
                ),
            )),
        }
    }
}
