//! Complete geometry vocabularies with borrowed inspection of their logical streams.
use crate::model::semantic::geometry_edit::{check_edit, remap_values};
use crate::model::semantic::profiles::Either;
use crate::model::semantic::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, ConnectivityId,
    ConnectivitySets, DrawSubsetCapabilities, Error, GeometryAttributes, GeometryInfluences,
    Influence, Link, NodeInfluence, SurfaceTriangleControls, TextureCoordinates, Topology,
    ValidateLinks, Values, VertexInfluences,
};
use crate::model::semantic::{GeometryEdit, GeometryEditCapabilities, StreamSupport};
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
    /// Describes common editing operations supported by this representation.
    fn edit_capabilities(&self) -> GeometryEditCapabilities {
        GeometryEditCapabilities::default()
    }
    /// Applies an atomic edit, returning a checked replacement and preserving this value.
    /// Custom schemas may opt in by overriding this method and the capabilities query.
    fn edited(&self, _edit: GeometryEdit<'data>) -> Result<Self, Error> {
        Err(Error::new(
            "geometry.edit",
            "this geometry schema does not support common edits",
        ))
    }
    /// Returns the authored geometry category without inspecting its packed representation.
    fn kind(&self) -> GeometryKind {
        GeometryKind::Display
    }
    /// Resolves explicit primary-domain aliases during checked mesh projection.
    fn normalize_primary_domains(&mut self, _primary: ConnectivityId) -> Result<(), Error> {
        Ok(())
    }
    /// Describes the mesh-owned selection bindings supported by this vocabulary.
    fn subset_capabilities(&self) -> DrawSubsetCapabilities {
        DrawSubsetCapabilities::default()
    }
    /// Resolves one topology set without copying its backing arrays.
    fn connectivity(&self, index: usize) -> Option<Topology<'data>> {
        match index {
            0 => Some(self.topology().clone()),
            1 => self
                .strip_indices()
                .map(|values| Topology::TriangleStrip(values.clone())),
            _ => None,
        }
    }
    /// Adds independently selectable topology when the schema permits it.
    fn add_connectivity(&mut self, _topology: Topology<'data>) -> Result<usize, Error> {
        Err(Error::new(
            "geometry.connectivity",
            "schema cannot add connectivity sets",
        ))
    }
    /// Typed surface behavior in primary triangle order, when supported.
    fn surface_triangles(&self) -> Option<&Values<'data, SurfaceTriangleControls>> {
        None
    }
    /// Independently indexed draw pools, when supported.
    fn strip_indices(&self) -> Option<&Values<'data, u32>> {
        None
    }
    /// Independently indexed tangent attributes selected by draw bindings.
    fn tangent_pool(&self) -> Option<&Values<'data, [f32; 4]>> {
        None
    }
    /// Model-space vertex positions.
    fn positions(&self) -> &Values<'data, [f32; 3]>;
    /// Vertex normals, when authored.
    fn normals(&self) -> Option<&Values<'data, [f32; 3]>>;
    /// Vertex tangents with handedness, when authored.
    fn tangents(&self) -> Option<&Values<'data, [f32; 4]>>;
    /// Ordered texture-coordinate streams.
    fn texture_coordinates(&self) -> TextureCoordinates<'_, 'data>;
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
    /// Three ordered influence slots per vertex; zero-weight slots are retained.
    Three(&'view Values<'data, [Influence; 3]>),
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
            Self::Three(values) => values.len(),
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
            Self::Three(values) => values
                .get(index)
                .map(|v| VertexInfluences::Joints(Vec::from(v))),
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

/// Shared geometry with canonical attributes and explicit connectivity.
#[derive(Clone, Debug)]
pub struct GeometryData<'data> {
    /// Authored purpose, independent of representation.
    pub kind: GeometryKind,
    /// Vertex positions addressed by connectivity and vertex attributes.
    pub positions: Values<'data, [f32; 3]>,
    /// Primary topology and additional draw-selected connectivity.
    pub connectivity: ConnectivitySets<'data>,
    /// Canonical typed attributes, including independently indexed pools.
    pub attributes: GeometryAttributes<'data>,
    /// Exact authored joint distributions.
    pub influences: GeometryInfluences<'data>,
}
impl<'data> GeometryData<'data> {
    /// Creates empty display geometry.
    pub fn empty() -> Self {
        Self::default()
    }
    /// Creates display triangles without copying their arrays.
    pub fn triangles(
        positions: impl Into<Values<'data, [f32; 3]>>,
        triangles: impl Into<Values<'data, [u32; 3]>>,
    ) -> Result<Self, Error> {
        let value = Self {
            positions: positions.into(),
            connectivity: ConnectivitySets::new(Topology::Triangles(triangles.into()), Vec::new()),
            ..Default::default()
        };
        value.validate()?;
        Ok(value)
    }
    /// Replaces exact authored joint distributions.
    pub fn with_influences(
        mut self,
        influences: impl Into<GeometryInfluences<'data>>,
    ) -> Result<Self, Error> {
        self.influences = influences.into();
        self.validate()?;
        Ok(self)
    }
    /// Replaces the vertex normal stream; an empty stream removes it.
    pub fn set_normals(&mut self, values: Values<'data, [f32; 3]>) {
        self.set_stream(AttributeSemantic::Normal, AttributeValues::Vector3(values));
    }
    /// Replaces the vertex tangent-frame stream; an empty stream removes it.
    pub fn set_tangents(&mut self, values: Values<'data, [f32; 4]>) {
        self.set_stream(AttributeSemantic::Tangent, AttributeValues::Vector4(values));
    }
    /// Replaces the primary vertex color stream; an empty stream removes it.
    pub fn set_colors(&mut self, values: Values<'data, [f32; 4]>) {
        self.set_stream(
            AttributeSemantic::Color(0),
            AttributeValues::Vector4(values),
        );
    }
    fn set_stream(&mut self, semantic: AttributeSemantic<'data>, values: AttributeValues<'data>) {
        if values.is_empty() {
            self.attributes.take(&semantic, AttributeDomain::Vertex);
        } else {
            self.attributes
                .set(Attribute {
                    semantic,
                    domain: AttributeDomain::Vertex,
                    values,
                })
                .expect("typed stream");
        }
    }
    /// Replaces ordered UV slots, including authored empty slots.
    pub fn set_texture_coordinates(&mut self, values: Vec<Values<'data, [f32; 2]>>) {
        self.attributes.retain(|attribute| {
            !matches!(
                (&attribute.semantic, attribute.domain),
                (AttributeSemantic::TexCoord(_), AttributeDomain::Vertex)
            )
        });
        for (slot, values) in values.into_iter().enumerate() {
            self.attributes
                .insert(Attribute {
                    semantic: AttributeSemantic::TexCoord(slot as u32),
                    domain: AttributeDomain::Vertex,
                    values: AttributeValues::Vector2(values),
                })
                .expect("unique typed UV slot");
        }
    }
    pub(crate) fn validate_vertex_streams(&self) -> Result<(), Error> {
        self.validate()?;
        if self.kind != GeometryKind::Display {
            return Err(Error::new(
                "geometry.kind",
                "target requires display geometry",
            ));
        }
        self.validate_no_additional_domains()?;
        if self.attributes.iter().any(|attribute| {
            attribute.domain != AttributeDomain::Vertex
                || !matches!(
                    attribute.semantic,
                    AttributeSemantic::Normal
                        | AttributeSemantic::Tangent
                        | AttributeSemantic::Color(0)
                        | AttributeSemantic::TexCoord(_)
                )
        }) {
            return Err(Error::new(
                "geometry.attributes",
                "target has no additional attribute domain",
            ));
        }
        Ok(())
    }
    pub(crate) fn validate_no_additional_domains(&self) -> Result<(), Error> {
        if !self.connectivity.additional().is_empty()
            || self.attributes.iter().any(|attribute| {
                attribute.domain == AttributeDomain::Pool
                    || attribute.semantic == AttributeSemantic::SurfaceControls
            })
        {
            return Err(Error::new(
                "geometry.domains",
                "target cannot retain surface controls or indexed pools",
            ));
        }
        Ok(())
    }
}
impl<'data> GeometrySchema<'data> for GeometryData<'data> {
    fn normalize_primary_domains(&mut self, primary: ConnectivityId) -> Result<(), Error> {
        for index in 0..self.attributes.len() {
            let domain = match self.attributes[index].domain {
                AttributeDomain::ConnectivityFace(id) if id == primary => {
                    Some(AttributeDomain::Face)
                }
                AttributeDomain::ConnectivityCorner(id) if id == primary => {
                    Some(AttributeDomain::Corner)
                }
                _ => None,
            };
            if let Some(domain) = domain {
                self.attributes
                    .update(index, |attribute| attribute.domain = domain)?;
            }
        }
        Ok(())
    }
    fn subset_capabilities(&self) -> DrawSubsetCapabilities {
        DrawSubsetCapabilities {
            alternate_selection: true,
            supplemental_strips: true,
            vertex_windows: true,
            tangent_bindings: true,
        }
    }
    fn add_connectivity(&mut self, topology: Topology<'data>) -> Result<usize, Error> {
        validate_topology(&topology, self.positions.len())?;
        Ok(self.connectivity.add(topology))
    }
    fn connectivity(&self, index: usize) -> Option<Topology<'data>> {
        if index == 0 {
            Some(self.connectivity.primary.clone())
        } else {
            self.connectivity.additional().get(index - 1).cloned()
        }
    }
    fn surface_triangles(&self) -> Option<&Values<'data, SurfaceTriangleControls>> {
        match &self
            .attributes
            .get(&AttributeSemantic::SurfaceControls, AttributeDomain::Face)?
            .values
        {
            AttributeValues::SurfaceControls(values) => Some(values),
            _ => None,
        }
    }
    fn strip_indices(&self) -> Option<&Values<'data, u32>> {
        match self.connectivity.additional().first()? {
            Topology::TriangleStrip(indices) => Some(indices),
            _ => None,
        }
    }
    fn tangent_pool(&self) -> Option<&Values<'data, [f32; 4]>> {
        self.attributes
            .vector4(&AttributeSemantic::Tangent, AttributeDomain::Pool)
    }
    fn kind(&self) -> GeometryKind {
        self.kind
    }
    fn edit_capabilities(&self) -> GeometryEditCapabilities {
        GeometryEditCapabilities {
            positions: true,
            normals: StreamSupport::Optional,
            tangents: StreamSupport::Optional,
            colors: StreamSupport::Optional,
            minimum_uv_sets: 0,
            maximum_uv_sets: None,
            vertex_remap: self.connectivity.additional().is_empty()
                && !self
                    .attributes
                    .iter()
                    .any(|attribute| attribute.domain == AttributeDomain::Pool),
        }
    }
    fn edited(&self, edit: GeometryEdit<'data>) -> Result<Self, Error> {
        self.validate()?;
        check_edit(&edit, self.edit_capabilities())?;
        let mut result = self.clone();
        if let Some(map) = &edit.remap {
            map.check(self)?;
            remap_values(&mut result.positions, &map.new_to_old);
            result.influences.remap(&map.new_to_old);
            result.attributes.remap(&map.new_to_old);
            result.connectivity.primary = map.topology.clone();
        }
        if let Some(values) = edit.positions {
            result.positions = values;
        }
        if let Some(values) = edit.normals {
            result.set_normals(values);
        }
        if let Some(values) = edit.tangents {
            result.set_tangents(values);
        }
        if let Some(values) = edit.colors {
            result.set_colors(values);
        }
        if let Some(values) = edit.texture_coordinates {
            result.set_texture_coordinates(values);
        }
        result.validate()?;
        Ok(result)
    }
    fn positions(&self) -> &Values<'data, [f32; 3]> {
        &self.positions
    }
    fn normals(&self) -> Option<&Values<'data, [f32; 3]>> {
        self.attributes
            .vector3(&AttributeSemantic::Normal, AttributeDomain::Vertex)
    }
    fn tangents(&self) -> Option<&Values<'data, [f32; 4]>> {
        self.attributes
            .vector4(&AttributeSemantic::Tangent, AttributeDomain::Vertex)
    }
    fn colors(&self) -> Option<&Values<'data, [f32; 4]>> {
        self.attributes
            .vector4(&AttributeSemantic::Color(0), AttributeDomain::Vertex)
    }
    fn texture_coordinates(&self) -> TextureCoordinates<'_, 'data> {
        self.attributes.texture_coordinates()
    }
    fn attributes(&self) -> Attributes<'_, 'data> {
        Attributes::Slice(self.attributes.as_slice())
    }
    fn topology(&self) -> &Topology<'data> {
        &self.connectivity.primary
    }
    fn influences(&self) -> InfluenceRows<'_, 'data> {
        self.influences.rows()
    }
    fn validate(&self) -> Result<(), Error> {
        validate_geometry(self)?;
        for topology in self.connectivity.additional() {
            validate_topology(topology, self.positions.len())?;
        }
        if self.surface_triangles().is_some_and(|values| {
            !matches!(self.connectivity.primary, Topology::Triangles(_))
                || values.len() != self.connectivity.primary.primitive_count()
        }) {
            return Err(Error::new("geometry.connectivity", "surface controls must cover primary triangles and strip indices must address vertices"));
        }
        Ok(())
    }
}

impl<'data, A: GeometrySchema<'data>, B: GeometrySchema<'data>> GeometrySchema<'data>
    for Either<A, B>
{
    fn normalize_primary_domains(&mut self, primary: ConnectivityId) -> Result<(), Error> {
        match self {
            Self::Left(value) => value.normalize_primary_domains(primary),
            Self::Right(value) => value.normalize_primary_domains(primary),
        }
    }
    fn subset_capabilities(&self) -> DrawSubsetCapabilities {
        match self {
            Self::Left(value) => value.subset_capabilities(),
            Self::Right(value) => value.subset_capabilities(),
        }
    }
    fn add_connectivity(&mut self, topology: Topology<'data>) -> Result<usize, Error> {
        match self {
            Self::Left(value) => value.add_connectivity(topology),
            Self::Right(value) => value.add_connectivity(topology),
        }
    }
    fn connectivity(&self, index: usize) -> Option<Topology<'data>> {
        match self {
            Self::Left(value) => value.connectivity(index),
            Self::Right(value) => value.connectivity(index),
        }
    }
    fn surface_triangles(&self) -> Option<&Values<'data, SurfaceTriangleControls>> {
        match self {
            Self::Left(value) => value.surface_triangles(),
            Self::Right(value) => value.surface_triangles(),
        }
    }
    fn strip_indices(&self) -> Option<&Values<'data, u32>> {
        match self {
            Self::Left(value) => value.strip_indices(),
            Self::Right(value) => value.strip_indices(),
        }
    }
    fn tangent_pool(&self) -> Option<&Values<'data, [f32; 4]>> {
        match self {
            Self::Left(value) => value.tangent_pool(),
            Self::Right(value) => value.tangent_pool(),
        }
    }
    fn edit_capabilities(&self) -> GeometryEditCapabilities {
        match self {
            Self::Left(value) => value.edit_capabilities(),
            Self::Right(value) => value.edit_capabilities(),
        }
    }
    fn edited(&self, edit: GeometryEdit<'data>) -> Result<Self, Error> {
        match self {
            Self::Left(value) => Ok(Self::Left(value.edited(edit)?)),
            Self::Right(value) => Ok(Self::Right(value.edited(edit)?)),
        }
    }

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
    fn texture_coordinates(&self) -> TextureCoordinates<'_, 'data> {
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
            AttributeDomain::ConnectivityFace(_) | AttributeDomain::ConnectivityCorner(_) => {
                attribute.values.len()
            }
            AttributeDomain::Mesh => 1,
            AttributeDomain::Pool => attribute.values.len(),
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
        let empty_uv_slot = attribute.domain == AttributeDomain::Vertex
            && matches!(attribute.semantic, AttributeSemantic::TexCoord(_))
            && attribute.values.is_empty();
        if (attribute.values.len() != expected && !empty_uv_slot) || !finite {
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
        Topology::Points(values) | Topology::TriangleStrip(values) => values.iter().all(valid),
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
            kind: GeometryKind::Display,
            positions: Values::default(),
            connectivity: ConnectivitySets::default(),
            attributes: GeometryAttributes::default(),
            influences: GeometryInfluences::None,
        }
    }
}
impl ValidateLinks for GeometryData<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for attribute in &self.attributes {
            if let AttributeValues::SurfaceControls(values) = &attribute.values {
                for face in values.iter() {
                    face.material.visit_links(visitor);
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
            .with_influences(JointInfluences::MatrixGroups(Values::from_slice(&groups)))
            .unwrap();
        let widened = GeometryData::from(geometry);
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
