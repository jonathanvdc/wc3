//! Checked common edits preserve the concrete geometry vocabulary.
use super::{
    Attribute, AttributeDomain, AttributeValues, Error, GeometrySchema, PackedArray, Topology,
};
use crate::model::common::PackedValue;
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
        if self.new_to_old.len() > u32::MAX as usize
            || self
                .new_to_old
                .iter()
                .any(|i| *i as usize >= geometry.positions().len())
            || !equivalent(geometry.topology(), &self.topology, &self.new_to_old)
        {
            return Err(Error::new(
                "geometry.edit.remap",
                "invalid vertex correspondence or changed primitive/corner ordering",
            ));
        }
        Ok(())
    }
}
fn equivalent(old: &Topology<'_>, new: &Topology<'_>, map: &[u32]) -> bool {
    let same = |a: u32, b: u32| map.get(b as usize) == Some(&a);
    match (old, new) {
        (Topology::Mixed(a), Topology::Mixed(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equivalent(a, b, map))
        }
        (Topology::Triangles(a), Topology::Triangles(b)) => {
            a.len() == b.len()
                && a.iter()
                    .flatten()
                    .zip(b.iter().flatten())
                    .all(|(a, b)| same(a, b))
        }
        (Topology::Lines(a), Topology::Lines(b)) => {
            a.len() == b.len()
                && a.iter()
                    .flatten()
                    .zip(b.iter().flatten())
                    .all(|(a, b)| same(a, b))
        }
        (Topology::Points(a), Topology::Points(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| same(a, b))
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
            ao.iter().eq(bo.iter())
                && a.len() == b.len()
                && a.iter().zip(b.iter()).all(|(a, b)| same(a, b))
        }
        _ => false,
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
        SurfaceControls
    );
}
pub(crate) fn check_edit(
    edit: &GeometryEdit<'_>,
    caps: GeometryEditCapabilities,
) -> Result<(), Error> {
    if (edit.positions.is_some() && !caps.positions)
        || (edit.normals.is_some() && caps.normals == StreamSupport::Unsupported)
        || (edit.tangents.is_some() && caps.tangents == StreamSupport::Unsupported)
        || (edit.colors.is_some() && caps.colors == StreamSupport::Unsupported)
        || (edit.remap.is_some() && !caps.vertex_remap)
        || edit.texture_coordinates.as_ref().is_some_and(|v| {
            v.len() < caps.minimum_uv_sets || caps.maximum_uv_sets.is_some_and(|n| v.len() > n)
        })
    {
        return Err(Error::new(
            "geometry.edit",
            "operation is unsupported by this geometry representation",
        ));
    }
    Ok(())
}
