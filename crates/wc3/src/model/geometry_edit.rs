//! Checked common edits preserve the concrete geometry vocabulary.
use super::{
    Attribute, AttributeDomain, AttributeValues, Error, GeometrySchema, PackedArray, Topology,
};
use crate::model::common::PackedValue;
use crate::model::error::{GeometryError, GeometryOperation, Location, TopologyKind};
use glam::{Vec2, Vec3, Vec4};

/// Availability of a stream in a geometry vocabulary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StreamSupport {
    /// The stream cannot be authored.
    #[default]
    Unsupported,
    /// The stream may be absent.
    Optional,
    /// Every vertex must carry a value.
    Required,
}
/// Constraints for common geometry editing; native encoding restrictions remain export decisions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GeometryEditCapabilities {
    /// Position replacement is supported.
    pub positions: bool,
    /// Vertex normal availability.
    pub normals: StreamSupport,
    /// Vertex tangent availability.
    pub tangents: StreamSupport,
    /// Primary vertex color availability.
    pub colors: StreamSupport,
    /// Minimum number of texture-coordinate streams.
    pub minimum_uv_sets: usize,
    /// Maximum number of texture-coordinate streams, or no fixed limit.
    pub maximum_uv_sets: Option<usize>,
    /// Vertex permutation, deletion of unused vertices, and duplication are supported.
    pub vertex_remap: bool,
}
/// Atomic replacements of common vertex streams. `None` retains the existing stream;
/// an empty replacement removes an optional stream. Influence representations are retained.
///
/// ```
/// use wc3::model::common::IntoPackedArray;
/// use wc3::model::{GeometryData, GeometryEdit, GeometrySchema, Vec3};
///
/// let geometry = GeometryData::triangles(
///     vec![Vec3::ZERO, Vec3::X, Vec3::Y],
///     vec![[0, 1, 2]],
/// )?;
/// let edited = geometry.edited(GeometryEdit {
///     positions: Some(vec![Vec3::Z, Vec3::X + Vec3::Z, Vec3::Y + Vec3::Z].into_packed_array()),
///     ..GeometryEdit::default()
/// })?;
/// assert_eq!(edited.positions.get(0), Some(Vec3::Z));
/// assert_eq!(geometry.positions.get(0), Some(Vec3::ZERO));
/// # Ok::<(), wc3::model::Error>(())
/// ```
#[derive(Clone, Debug, Default)]
pub struct GeometryEdit<'a> {
    /// Replacement positions.
    pub positions: Option<PackedArray<'a, Vec3>>,
    /// Replacement vertex normals.
    pub normals: Option<PackedArray<'a, Vec3>>,
    /// Replacement vertex tangents and handedness.
    pub tangents: Option<PackedArray<'a, Vec4>>,
    /// Replacement primary vertex colors.
    pub colors: Option<PackedArray<'a, Vec4>>,
    /// Replacement ordered UV streams.
    pub texture_coordinates: Option<Vec<PackedArray<'a, Vec2>>>,
    /// Structural correspondence applied before stream replacements.
    pub remap: Option<VertexRemap<'a>>,
}
/// New vertex indices select old vertices. ConnectivitySets must preserve every primitive
/// and corner in order, selecting an equivalent old vertex at each corner. This permits
/// splitting shared vertices without changing face assignments or corner attributes.
#[derive(Clone, Debug)]
pub struct VertexRemap<'a> {
    /// One old vertex index per new vertex; duplicates are permitted.
    pub new_to_old: Vec<u32>,
    /// Replacement connectivity with unchanged primitive grouping and corner order.
    pub topology: Topology<'a>,
}
impl VertexRemap<'_> {
    pub(crate) fn check<'a>(&self, geometry: &impl GeometrySchema<'a>) -> Result<(), Error> {
        if self.new_to_old.len() > u32::MAX as usize {
            return Err(Error::at(
                Location::new().field("new_to_old"),
                GeometryError::CountOverflow {
                    actual: self.new_to_old.len(),
                    maximum: u32::MAX as usize,
                },
            ));
        }
        if let Some((index, value)) = self
            .new_to_old
            .iter()
            .enumerate()
            .find(|(_, value)| **value as usize >= geometry.positions().len())
        {
            return Err(Error::at(
                Location::new().field("new_to_old").index(index),
                GeometryError::IndexOutOfBounds {
                    index: *value as usize,
                    len: geometry.positions().len(),
                },
            ));
        }
        equivalent(geometry.topology(), &self.topology, &self.new_to_old)
            .map_err(|error| error.with_context(Location::new().field("topology")))
    }
}
fn equivalent(old: &Topology<'_>, new: &Topology<'_>, map: &[u32]) -> Result<(), Error> {
    fn count(expected: usize, actual: usize) -> Result<(), Error> {
        if expected != actual {
            return Err(GeometryError::CountMismatch { expected, actual }.into());
        }
        Ok(())
    }
    fn corners(
        a: impl Iterator<Item = u32>,
        b: impl Iterator<Item = u32>,
        map: &[u32],
    ) -> Result<(), Error> {
        for (index, (expected, new)) in a.zip(b).enumerate() {
            let actual = map.get(new as usize).copied();
            if actual != Some(expected) {
                return Err(Error::at(
                    Location::new().field("corners").index(index),
                    GeometryError::RemapCorrespondence { expected, actual },
                ));
            }
        }
        Ok(())
    }
    match (old, new) {
        (Topology::Mixed(a), Topology::Mixed(b)) => {
            count(a.len(), b.len())?;
            for (index, (a, b)) in a.iter().zip(b).enumerate() {
                equivalent(a, b, map).map_err(|error| {
                    error.with_context(Location::new().field("groups").index(index))
                })?;
            }
            Ok(())
        }
        (Topology::Triangles(a), Topology::Triangles(b)) => {
            count(a.len(), b.len())?;
            corners(a.iter().flatten(), b.iter().flatten(), map)
        }
        (Topology::Lines(a), Topology::Lines(b)) => {
            count(a.len(), b.len())?;
            corners(a.iter().flatten(), b.iter().flatten(), map)
        }
        (Topology::Points(a), Topology::Points(b)) => {
            count(a.len(), b.len())?;
            corners(a.iter(), b.iter(), map)
        }
        (
            Topology::Polygons {
                corners: a,
                offsets: ao,
            },
            Topology::Polygons {
                corners: b,
                offsets: bo,
            },
        ) => {
            count(ao.len(), bo.len())
                .map_err(|error| error.with_context(Location::new().field("offsets")))?;
            for (index, (expected, actual)) in ao.iter().zip(bo.iter()).enumerate() {
                if expected != actual {
                    return Err(Error::at(
                        Location::new().field("offsets").index(index),
                        GeometryError::RemapBoundary { expected, actual },
                    ));
                }
            }
            count(a.len(), b.len())?;
            corners(a.iter(), b.iter(), map)
        }
        (Topology::TriangleStrip(_), Topology::TriangleStrip(_)) => {
            Err(GeometryError::UnsupportedRemapTopology {
                topology: TopologyKind::TriangleStrip,
            }
            .into())
        }
        _ => Err(GeometryError::RemapTopology {
            expected: TopologyKind::from(old),
            actual: TopologyKind::from(new),
        }
        .into()),
    }
}
pub(crate) fn remap_values<'a, T: PackedValue + 'a>(values: &mut PackedArray<'a, T>, map: &[u32]) {
    if !values.is_empty() {
        *values = PackedArray::alloc_from_slice(
            &map.iter()
                .map(|i| values.get(*i as usize).expect("validated vertex stream"))
                .collect::<Vec<_>>(),
        );
    }
}
pub(crate) fn remap_attribute(attribute: &mut Attribute<'_>, map: &[u32]) {
    if attribute.domain != AttributeDomain::Vertex {
        return;
    }
    macro_rules! remap { ($($variant:ident),*) => { match &mut attribute.values { $(AttributeValues::$variant(v) => remap_values(v,map),)* } }; }
    remap!(
        Boolean,
        Byte3,
        Byte4,
        U16,
        Unsigned4,
        Quaternion,
        Scalar,
        Vector2,
        Vector3,
        Vector4,
        Unsigned,
        Signed,
        TriangleProperties
    );
}
pub(crate) fn check_edit(
    edit: &GeometryEdit<'_>,
    caps: GeometryEditCapabilities,
) -> Result<(), Error> {
    for (requested, operation) in [
        (
            edit.positions.is_some() && !caps.positions,
            GeometryOperation::ReplacePositions,
        ),
        (
            edit.normals.is_some() && caps.normals == StreamSupport::Unsupported,
            GeometryOperation::ReplaceNormals,
        ),
        (
            edit.tangents.is_some() && caps.tangents == StreamSupport::Unsupported,
            GeometryOperation::ReplaceTangents,
        ),
        (
            edit.colors.is_some() && caps.colors == StreamSupport::Unsupported,
            GeometryOperation::ReplaceColors,
        ),
        (
            edit.remap.is_some() && !caps.vertex_remap,
            GeometryOperation::VertexRemap,
        ),
    ] {
        if requested {
            return Err(Error::at(
                Location::new().field("edit"),
                GeometryError::UnsupportedOperation { operation },
            ));
        }
    }
    if let Some(values) = &edit.texture_coordinates {
        if values.len() < caps.minimum_uv_sets
            || caps
                .maximum_uv_sets
                .is_some_and(|maximum| values.len() > maximum)
        {
            return Err(Error::at(
                Location::new().field("texture_coordinates"),
                GeometryError::TextureCoordinateCount {
                    minimum: caps.minimum_uv_sets,
                    maximum: caps.maximum_uv_sets,
                    actual: values.len(),
                },
            ));
        }
    }
    Ok(())
}
