//! Complete geometry vocabularies with borrowed inspection of their logical streams.
use crate::model::semantic::profiles::Either;
use crate::model::semantic::{
    Attribute, AttributeDomain, AttributeValues, Error, Influence, Link, M2Geometry, M3Geometry,
    MdlxGeometry, NodeInfluence, Topology, ValidateLinks, Values, VertexInfluences, WmoGeometry,
};
use std::fmt::{Debug, Formatter, Result as FmtResult};

/// An owner exposing attribute definitions without requiring contiguous storage.
pub trait AttributeSource<'data> {
    /// Number of authored layers in this source.
    fn len(&self) -> usize;
    /// Whether this source has no authored layers.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Resolves a layer; indices from zero up to `len()` must all be present.
    fn attribute(&self, index: usize) -> &Attribute<'data>;
}
/// Borrowed inspection of attribute layers, including profile-owned identified layers.
#[derive(Clone, Copy)]
pub enum Attributes<'view, 'data> {
    /// Contiguous common attribute definitions.
    Slice(&'view [Attribute<'data>]),
    /// Definitions owned by a profile's layer objects.
    Source(&'view dyn AttributeSource<'data>),
}
impl<'view, 'data> Attributes<'view, 'data> {
    /// Number of authored attribute definitions.
    pub fn len(self) -> usize {
        match self {
            Self::Slice(v) => v.len(),
            Self::Source(v) => v.len(),
        }
    }
    /// Whether there are no attribute definitions.
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
    /// Inspects an attribute without copying its definition or values.
    pub fn get(self, index: usize) -> Option<&'view Attribute<'data>> {
        match self {
            Self::Slice(v) => v.get(index),
            Self::Source(v) => (index < v.len()).then(|| v.attribute(index)),
        }
    }
    /// Visits definitions in their authored layer order.
    pub fn iter(self) -> AttributeIter<'view, 'data> {
        AttributeIter {
            attributes: self,
            index: 0,
        }
    }
}
impl Debug for Attributes<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_list().entries(self.iter()).finish()
    }
}
/// Ordered borrowed traversal of authored attributes.
pub struct AttributeIter<'view, 'data> {
    attributes: Attributes<'view, 'data>,
    index: usize,
}
impl<'view, 'data> Iterator for AttributeIter<'view, 'data> {
    type Item = &'view Attribute<'data>;
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.attributes.get(self.index)?;
        self.index += 1;
        Some(value)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.attributes.len() - self.index;
        (remaining, Some(remaining))
    }
}
impl ExactSizeIterator for AttributeIter<'_, '_> {}
impl<'view, 'data> IntoIterator for Attributes<'view, 'data> {
    type Item = &'view Attribute<'data>;
    type IntoIter = AttributeIter<'view, 'data>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Authored purpose required by relationships that consume a particular geometry vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeometryKind {
    /// Geometry used by shaded model placements.
    Display,
    /// Geometry defining a collision surface independently of shading.
    Collision,
}
/// Borrowed access to a profile's geometry, independent of its native packing.
///
/// Implementations own the streams they expose. Inspection does not construct a
/// second geometry object or materialize encoded arrays. Validation checks
/// authoring structure; native index widths and quantization belong to export.
pub trait GeometrySchema<'data>: Clone + Debug + Send + Sync + ValidateLinks {
    /// Returns the authored geometry category without inspecting its packed representation.
    fn kind(&self) -> GeometryKind {
        GeometryKind::Display
    }
    /// Model-space vertex positions.
    fn positions(&self) -> &Values<'data, [f32; 3]>;
    /// Vertex normals, when authored.
    fn normals(&self) -> Option<&Values<'data, [f32; 3]>>;
    /// Vertex tangents with handedness, when authored.
    fn tangents(&self) -> Option<&Values<'data, [f32; 4]>>;
    /// Ordered texture-coordinate streams.
    fn texture_coordinates(&self) -> &[Values<'data, [f32; 2]>];
    /// Vertex colors, when authored.
    fn colors(&self) -> Option<&Values<'data, [f32; 4]>>;
    /// Additional authoring attributes supported by this vocabulary.
    fn attributes(&self) -> Attributes<'_, 'data>;
    /// Logical primitive connectivity.
    fn topology(&self) -> &Topology<'data>;
    /// Vertex joint distributions, before resolving a placement's skin binding.
    fn influences(&self) -> InfluenceRows<'_, 'data>;
    /// Checks stream cardinalities, finite values, indices, and authoring invariants.
    fn validate(&self) -> Result<(), Error>;
}

/// Borrowed joint distributions with their authored structure preserved.
#[derive(Clone, Copy, Debug)]
pub enum InfluenceRows<'view, 'data> {
    /// Rigid geometry with no authored distributions.
    None,
    /// Variable-length normalized distributions.
    Variable(&'view Values<'data, Vec<Influence>>),
    /// Four ordered influence slots per vertex; zero-weight slots are retained.
    Four(&'view Values<'data, [Influence; 4]>),
    /// Direct hierarchy-node contributions, without an invented joint palette.
    Nodes(&'view Values<'data, Vec<NodeInfluence>>),
    /// Warcraft III matrix groups, whose members receive equal weights.
    MatrixGroups(&'view Values<'data, Vec<u32>>),
}
impl InfluenceRows<'_, '_> {
    /// Number of authored vertex distributions.
    pub fn len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Variable(values) => values.len(),
            Self::Four(values) => values.len(),
            Self::MatrixGroups(values) => values.len(),
            Self::Nodes(values) => values.len(),
        }
    }
    /// Whether the geometry is rigid or has no vertices.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Iterates decoded joint distributions without materializing a second stream.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = VertexInfluences> + '_ {
        (0..self.len()).map(|index| self.get(index).expect("checked influence index"))
    }
    /// Reads binding-slot contributions; direct-node geometry has no binding-slot row.
    pub fn joint_weights(&self, index: usize) -> Option<Vec<Influence>> {
        self.get(index)?.into_joint_weights()
    }
    /// Reads one distribution in logical joint order.
    pub fn get(&self, index: usize) -> Option<VertexInfluences> {
        match self {
            Self::None => None,
            Self::Nodes(values) => values.get(index).map(VertexInfluences::Nodes),
            Self::Variable(values) => values.get(index).map(VertexInfluences::Joints),
            Self::Four(values) => values
                .get(index)
                .map(|v| VertexInfluences::Joints(Vec::from(v))),
            Self::MatrixGroups(values) => values.get(index).map(|joints| {
                let weight = if joints.is_empty() {
                    0.
                } else {
                    1. / joints.len() as f32
                };
                VertexInfluences::Joints(
                    joints
                        .into_iter()
                        .map(|joint| Influence { joint, weight })
                        .collect(),
                )
            }),
        }
    }
}

/// Complete built-in geometry vocabulary, preserving each authored schema on widening.
#[derive(Clone, Debug)]
pub enum UniversalGeometry<'data> {
    /// Diablo III geometry with independent tangent and binormal streams.
    D3(crate::model::semantic::profiles::diablo::d3::Geometry<'data>),
    /// Diablo IV declared logical vertex streams.
    D4(crate::model::semantic::profiles::diablo::d4::Geometry<'data>),
    /// General authoring geometry, including polygons and custom attributes.
    Flexible(GeometryData<'data>),
    /// Warcraft III geometry and joint distribution semantics.
    Mdlx(MdlxGeometry<'data>),
    /// World of Warcraft geometry with its two coordinate streams and four joint slots.
    M2(M2Geometry<'data>),
    /// StarCraft II geometry, including sparse coordinate slots.
    M3(M3Geometry<'data>),
    /// Static-world geometry, including face collision and rendering controls.
    Wmo(WmoGeometry<'data>),
    /// Editable-model polygon groups, typed layers, and direct node influences.
    Wem(crate::model::semantic::profiles::wem::Geometry<'data>),
}
impl Default for UniversalGeometry<'_> {
    fn default() -> Self {
        Self::Flexible(GeometryData::default())
    }
}
impl<'data> From<MdlxGeometry<'data>> for UniversalGeometry<'data> {
    fn from(value: MdlxGeometry<'data>) -> Self {
        Self::Mdlx(value)
    }
}
impl<'data> From<M2Geometry<'data>> for UniversalGeometry<'data> {
    fn from(value: M2Geometry<'data>) -> Self {
        Self::M2(value)
    }
}
impl<'data> From<M3Geometry<'data>> for UniversalGeometry<'data> {
    fn from(value: M3Geometry<'data>) -> Self {
        Self::M3(value)
    }
}
impl<'data> From<GeometryData<'data>> for UniversalGeometry<'data> {
    fn from(value: GeometryData<'data>) -> Self {
        Self::Flexible(value)
    }
}
impl<'data> GeometrySchema<'data> for UniversalGeometry<'data> {
    fn kind(&self) -> GeometryKind {
        match self {
            Self::Flexible(v) => v.kind(),
            Self::Mdlx(v) => v.kind(),
            Self::M2(v) => v.kind(),
            Self::M3(v) => v.kind(),
            Self::D4(v) => v.kind(),
            Self::D3(v) => v.kind(),
            Self::Wmo(v) => v.kind(),
            Self::Wem(v) => v.kind(),
        }
    }
    fn positions(&self) -> &Values<'data, [f32; 3]> {
        match self {
            Self::Flexible(value) => value.positions(),
            Self::Mdlx(value) => value.positions(),
            Self::M2(value) => value.positions(),
            Self::M3(value) => value.positions(),
            Self::D4(value) => value.positions(),
            Self::D3(value) => value.positions(),
            Self::Wmo(value) => value.positions(),
            Self::Wem(value) => value.positions(),
        }
    }
    fn normals(&self) -> Option<&Values<'data, [f32; 3]>> {
        match self {
            Self::Flexible(value) => value.normals(),
            Self::Mdlx(value) => value.normals(),
            Self::M2(value) => value.normals(),
            Self::M3(value) => value.normals(),
            Self::D4(value) => value.normals(),
            Self::D3(value) => value.normals(),
            Self::Wmo(value) => value.normals(),
            Self::Wem(value) => value.normals(),
        }
    }
    fn tangents(&self) -> Option<&Values<'data, [f32; 4]>> {
        match self {
            Self::Flexible(value) => value.tangents(),
            Self::Mdlx(value) => value.tangents(),
            Self::M2(value) => value.tangents(),
            Self::M3(value) => value.tangents(),
            Self::D4(value) => value.tangents(),
            Self::D3(value) => value.tangents(),
            Self::Wmo(value) => value.tangents(),
            Self::Wem(value) => value.tangents(),
        }
    }
    fn texture_coordinates(&self) -> &[Values<'data, [f32; 2]>] {
        match self {
            Self::Flexible(value) => value.texture_coordinates(),
            Self::Mdlx(value) => value.texture_coordinates(),
            Self::M2(value) => value.texture_coordinates(),
            Self::M3(value) => value.texture_coordinates(),
            Self::D4(value) => value.texture_coordinates(),
            Self::D3(value) => value.texture_coordinates(),
            Self::Wmo(value) => value.texture_coordinates(),
            Self::Wem(value) => value.texture_coordinates(),
        }
    }
    fn colors(&self) -> Option<&Values<'data, [f32; 4]>> {
        match self {
            Self::Flexible(value) => value.colors(),
            Self::Mdlx(value) => value.colors(),
            Self::M2(value) => value.colors(),
            Self::M3(value) => value.colors(),
            Self::D4(value) => value.colors(),
            Self::D3(value) => value.colors(),
            Self::Wmo(value) => value.colors(),
            Self::Wem(value) => value.colors(),
        }
    }
    fn attributes(&self) -> Attributes<'_, 'data> {
        match self {
            Self::Flexible(value) => value.attributes(),
            Self::Mdlx(value) => value.attributes(),
            Self::M2(value) => value.attributes(),
            Self::M3(value) => value.attributes(),
            Self::D4(value) => value.attributes(),
            Self::D3(value) => value.attributes(),
            Self::Wmo(value) => value.attributes(),
            Self::Wem(value) => value.attributes(),
        }
    }
    fn topology(&self) -> &Topology<'data> {
        match self {
            Self::Flexible(value) => value.topology(),
            Self::Mdlx(value) => value.topology(),
            Self::M2(value) => value.topology(),
            Self::M3(value) => value.topology(),
            Self::D4(value) => value.topology(),
            Self::D3(value) => value.topology(),
            Self::Wmo(value) => value.topology(),
            Self::Wem(value) => value.topology(),
        }
    }
    fn influences(&self) -> InfluenceRows<'_, 'data> {
        match self {
            Self::Flexible(value) => value.influences(),
            Self::Mdlx(value) => value.influences(),
            Self::M2(value) => value.influences(),
            Self::M3(value) => value.influences(),
            Self::D4(value) => value.influences(),
            Self::D3(value) => value.influences(),
            Self::Wmo(value) => value.influences(),
            Self::Wem(value) => value.influences(),
        }
    }
    fn validate(&self) -> Result<(), Error> {
        match self {
            Self::Flexible(value) => value.validate(),
            Self::Mdlx(value) => value.validate(),
            Self::M2(value) => value.validate(),
            Self::M3(value) => value.validate(),
            Self::D4(value) => value.validate(),
            Self::D3(value) => value.validate(),
            Self::Wmo(value) => value.validate(),
            Self::Wem(value) => value.validate(),
        }
    }
}
impl ValidateLinks for UniversalGeometry<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Flexible(value) => value.visit_links(visitor),
            Self::Mdlx(value) => value.visit_links(visitor),
            Self::M2(value) => value.visit_links(visitor),
            Self::M3(value) => value.visit_links(visitor),
            Self::D4(value) => value.visit_links(visitor),
            Self::D3(value) => value.visit_links(visitor),
            Self::Wmo(value) => value.visit_links(visitor),
            Self::Wem(value) => value.visit_links(visitor),
        }
    }
}
/// General authoring geometry, including polygon domains and custom attributes.
#[derive(Clone, Debug)]
pub struct GeometryData<'data> {
    /// Vertex positions.
    pub positions: Values<'data, [f32; 3]>,
    /// Vertex normals; empty when absent.
    pub normals: Values<'data, [f32; 3]>,
    /// Vertex tangents; empty when absent.
    pub tangents: Values<'data, [f32; 4]>,
    /// Ordered texture-coordinate sets.
    pub texture_coordinates: Vec<Values<'data, [f32; 2]>>,
    /// Vertex colors; empty when absent.
    pub colors: Values<'data, [f32; 4]>,
    /// Additional attributes, including corner and face domains.
    pub attributes: Vec<Attribute<'data>>,
    /// Authored connectivity.
    pub topology: Topology<'data>,
    /// Arbitrary normalized joint distributions; empty when rigid.
    pub influences: Values<'data, Vec<Influence>>,
}
impl<'data> GeometryData<'data> {
    /// Creates an empty geometry definition.
    pub fn empty() -> Self {
        Self::default()
    }
    /// Creates checked triangle geometry without copying its arrays.
    pub fn triangles(
        positions: impl Into<Values<'data, [f32; 3]>>,
        triangles: impl Into<Values<'data, [u32; 3]>>,
    ) -> Result<Self, Error> {
        let geometry = Self {
            positions: positions.into(),
            normals: Values::default(),
            tangents: Values::default(),
            texture_coordinates: Vec::new(),
            colors: Values::default(),
            attributes: Vec::new(),
            topology: Topology::Triangles(triangles.into()),
            influences: Values::default(),
        };
        geometry.validate()?;
        Ok(geometry)
    }
}
impl<'data> GeometrySchema<'data> for GeometryData<'data> {
    fn positions(&self) -> &Values<'data, [f32; 3]> {
        &self.positions
    }
    fn normals(&self) -> Option<&Values<'data, [f32; 3]>> {
        present(&self.normals)
    }
    fn tangents(&self) -> Option<&Values<'data, [f32; 4]>> {
        present(&self.tangents)
    }
    fn texture_coordinates(&self) -> &[Values<'data, [f32; 2]>] {
        &self.texture_coordinates
    }
    fn colors(&self) -> Option<&Values<'data, [f32; 4]>> {
        present(&self.colors)
    }
    fn attributes(&self) -> Attributes<'_, 'data> {
        Attributes::Slice(&self.attributes)
    }
    fn topology(&self) -> &Topology<'data> {
        &self.topology
    }
    fn influences(&self) -> InfluenceRows<'_, 'data> {
        InfluenceRows::Variable(&self.influences)
    }
    fn validate(&self) -> Result<(), Error> {
        validate_geometry(self)
    }
}

impl<'data, A: GeometrySchema<'data>, B: GeometrySchema<'data>> GeometrySchema<'data>
    for Either<A, B>
{
    fn kind(&self) -> GeometryKind {
        match self {
            Self::Left(v) => v.kind(),
            Self::Right(v) => v.kind(),
        }
    }
    fn positions(&self) -> &Values<'data, [f32; 3]> {
        match self {
            Self::Left(v) => v.positions(),
            Self::Right(v) => v.positions(),
        }
    }
    fn normals(&self) -> Option<&Values<'data, [f32; 3]>> {
        match self {
            Self::Left(v) => v.normals(),
            Self::Right(v) => v.normals(),
        }
    }
    fn tangents(&self) -> Option<&Values<'data, [f32; 4]>> {
        match self {
            Self::Left(v) => v.tangents(),
            Self::Right(v) => v.tangents(),
        }
    }
    fn texture_coordinates(&self) -> &[Values<'data, [f32; 2]>] {
        match self {
            Self::Left(v) => v.texture_coordinates(),
            Self::Right(v) => v.texture_coordinates(),
        }
    }
    fn colors(&self) -> Option<&Values<'data, [f32; 4]>> {
        match self {
            Self::Left(v) => v.colors(),
            Self::Right(v) => v.colors(),
        }
    }
    fn attributes(&self) -> Attributes<'_, 'data> {
        match self {
            Self::Left(v) => v.attributes(),
            Self::Right(v) => v.attributes(),
        }
    }
    fn topology(&self) -> &Topology<'data> {
        match self {
            Self::Left(v) => v.topology(),
            Self::Right(v) => v.topology(),
        }
    }
    fn influences(&self) -> InfluenceRows<'_, 'data> {
        match self {
            Self::Left(v) => v.influences(),
            Self::Right(v) => v.influences(),
        }
    }
    fn validate(&self) -> Result<(), Error> {
        match self {
            Self::Left(v) => v.validate(),
            Self::Right(v) => v.validate(),
        }
    }
}

pub(crate) fn present<'view, 'data, T>(
    values: &'view Values<'data, T>,
) -> Option<&'view Values<'data, T>> {
    (!values.is_empty()).then_some(values)
}

pub(crate) fn validate_geometry<'data>(geometry: &impl GeometrySchema<'data>) -> Result<(), Error> {
    let count = geometry.positions().len();
    for stream in geometry
        .normals()
        .map(Values::len)
        .into_iter()
        .chain(geometry.tangents().map(Values::len))
        .chain(geometry.colors().map(Values::len))
        .chain(
            geometry
                .texture_coordinates()
                .iter()
                .map(Values::len)
                .filter(|len| *len != 0),
        )
        .chain(Some(geometry.influences().len()).filter(|len| *len != 0))
    {
        if stream != count {
            return Err(Error::new(
                "geometry.streams",
                "stream cardinality differs from vertex count",
            ));
        }
    }
    if !geometry
        .positions()
        .iter()
        .flatten()
        .chain(
            geometry
                .normals()
                .into_iter()
                .flat_map(|v| v.iter().flatten()),
        )
        .chain(
            geometry
                .tangents()
                .into_iter()
                .flat_map(|v| v.iter().flatten()),
        )
        .chain(
            geometry
                .colors()
                .into_iter()
                .flat_map(|v| v.iter().flatten()),
        )
        .chain(
            geometry
                .texture_coordinates()
                .iter()
                .flat_map(|v| v.iter().flatten()),
        )
        .all(f32::is_finite)
    {
        return Err(Error::new(
            "geometry.streams",
            "vertex values must be finite",
        ));
    }
    validate_topology(geometry.topology(), count)?;
    let faces = geometry.topology().primitive_count();
    let corners = geometry.topology().corner_count();
    for attribute in geometry.attributes() {
        let expected = match attribute.domain {
            AttributeDomain::Vertex => count,
            AttributeDomain::Corner => corners,
            AttributeDomain::Face => faces,
            AttributeDomain::Mesh => 1,
        };
        let finite = match &attribute.values {
            AttributeValues::Scalar(v) => v.iter().all(f32::is_finite),
            AttributeValues::Vector2(v) => v.iter().flatten().all(f32::is_finite),
            AttributeValues::Vector3(v) => v.iter().flatten().all(f32::is_finite),
            AttributeValues::Vector4(v) | AttributeValues::Quaternion(v) => {
                v.iter().flatten().all(f32::is_finite)
            }
            _ => true,
        };
        if attribute.values.len() != expected || !finite {
            return Err(Error::new(
                "geometry.attributes",
                "attribute cardinality or values are invalid for its domain",
            ));
        }
    }
    let rows = geometry.influences();
    for index in 0..rows.len() {
        let row = rows.get(index).expect("checked row index");
        let sum = row.weights().sum::<f32>();
        if row
            .weights()
            .any(|weight| !weight.is_finite() || weight < 0.)
            || (sum != 0. && (sum - 1.).abs() > 0.0001)
        {
            return Err(Error::new(
                "geometry.influences",
                "weights must be finite, nonnegative, and normalized when nonzero",
            ));
        }
    }
    Ok(())
}

fn validate_topology(topology: &Topology<'_>, vertices: usize) -> Result<(), Error> {
    let valid = |index: u32| (index as usize) < vertices;
    let indices_valid = match topology {
        Topology::Points(values) => values.iter().all(valid),
        Topology::Lines(values) => values.iter().flatten().all(valid),
        Topology::Triangles(values) => values.iter().flatten().all(valid),
        Topology::Mixed(groups) => {
            for group in groups {
                validate_topology(group, vertices)?;
            }
            true
        }
        Topology::Polygons { corners, offsets } => {
            let offsets = offsets.to_vec();
            offsets.first() == Some(&0)
                && offsets.last().copied().map(|value| value as usize) == Some(corners.len())
                && offsets
                    .windows(2)
                    .all(|pair| pair[1] >= pair[0] && pair[1] - pair[0] >= 3)
                && corners.iter().all(valid)
        }
    };
    if indices_valid {
        Ok(())
    } else {
        Err(Error::new(
            "geometry.topology",
            "primitive indices or polygon boundaries are invalid",
        ))
    }
}

impl Default for GeometryData<'_> {
    fn default() -> Self {
        Self {
            positions: Values::default(),
            normals: Values::default(),
            tangents: Values::default(),
            texture_coordinates: Vec::new(),
            colors: Values::default(),
            attributes: Vec::new(),
            topology: Topology::Triangles(Values::default()),
            influences: Values::default(),
        }
    }
}

impl ValidateLinks for GeometryData<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for attribute in &self.attributes {
            if let AttributeValues::Materials(values) = &attribute.values {
                for id in values.iter() {
                    id.visit_links(visitor);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::semantic::*;

    #[test]
    fn borrowed_geometry_rejects_invalid_streams_and_indices() {
        let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
        let triangles = [[0, 1, 2]];
        let geometry = MdlxGeometry::triangles(&positions[..], &triangles[..]).unwrap();
        assert!(geometry.positions().is_borrowed());
        assert!(geometry.clone().with_normals(vec![[0., 0., 1.]]).is_err());
        assert!(geometry
            .with_tangents(vec![[0., 0., f32::NAN, 1.]; 3])
            .is_err());
        assert!(MdlxGeometry::triangles(&positions[..], vec![[0, 1, 3]]).is_err());
    }

    #[test]
    fn four_slots_and_matrix_groups_widen_without_losing_distribution() {
        let positions = [[0., 0., 0.]];
        let groups = [vec![3, 8, 12, 19, 24]];
        let geometry = MdlxGeometry::triangles(&positions[..], Vec::<[u32; 3]>::new())
            .unwrap()
            .with_influences(MdlxVertexInfluences::MatrixGroups(Values::from_slice(
                &groups,
            )))
            .unwrap();
        let widened = UniversalGeometry::from(geometry);
        assert!(widened.positions().is_borrowed());
        assert!(
            matches!(widened.influences(), InfluenceRows::MatrixGroups(values) if values.is_borrowed())
        );
        let row = widened.influences().joint_weights(0).unwrap();
        assert_eq!(row.len(), 5);
        assert_eq!(row[4].joint, 24);
        assert!((row[4].weight - 0.2).abs() < f32::EPSILON);
        widened.validate().unwrap();
    }

    #[test]
    fn m2_requires_both_texture_coordinate_streams() {
        let positions = vec![[0., 0., 0.]];
        let normals = vec![[0., 0., 1.]];
        let weights = vec![
            [Influence {
                joint: 0,
                weight: 0.,
            }; 4],
        ];
        assert!(M2Geometry::new(
            positions,
            normals,
            [Values::owned(vec![[0., 0.]]), Values::default()],
            Vec::<[u32; 3]>::new(),
            weights
        )
        .is_err());
    }

    #[test]
    fn m3_retains_fifth_coordinate_stream_but_rejects_sixth() {
        let geometry = M3Geometry::new(
            vec![[0., 0., 0.]],
            vec![[0., 0., 1.]],
            Vec::<[u32; 3]>::new(),
            vec![
                [Influence {
                    joint: 0,
                    weight: 0.,
                }; 4],
            ],
        )
        .unwrap();
        let coordinates = vec![Values::owned(vec![[0., 0.]]); 5];
        assert_eq!(
            geometry
                .clone()
                .with_texture_coordinates(coordinates)
                .unwrap()
                .texture_coordinates()
                .len(),
            5
        );
        assert!(geometry
            .with_texture_coordinates(vec![Values::owned(vec![[0., 0.]]); 6])
            .is_err());
    }
}
