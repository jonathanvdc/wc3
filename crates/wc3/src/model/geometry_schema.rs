//! Complete geometry vocabularies with borrowed inspection of their logical streams.
use crate::model::common::IntoOwned;
use crate::model::common::IntoPackedArray;
use crate::model::common::PackedValue;
use crate::model::error::{
    AttributeProjectionRule, AttributeValueKind, ProfileError, ProfileKind, ProfileViolation,
};
use crate::model::error::{
    GeometryError, GeometryOperation, Location, PolygonBoundaryRule, TopologyKind,
};
use crate::model::geometry_edit::{check_edit, remap_values};
use crate::model::profiles::Either;
use crate::model::ValueGroups;
use crate::model::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, ConnectivityId,
    ConnectivitySets, DrawSubsetCapabilities, Error, GeometryAttributes, GeometryInfluences,
    Influence, JointWeights, Link, NodeInfluence, PackedArray, TextureCoordinates, Topology,
    TriangleProperties, ValidateLinks, VertexInfluences,
};
use crate::model::{GeometryEdit, GeometryEditCapabilities, StreamSupport};
use glam::{Vec2, Vec3, Vec4};
use std::fmt::Debug;

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
        Err(Error::at(
            Location::new().field("edit"),
            GeometryError::UnsupportedOperation {
                operation: GeometryOperation::Edit,
            },
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
        Err(Error::at(
            Location::new().field("connectivity"),
            GeometryError::UnsupportedOperation {
                operation: GeometryOperation::AddConnectivity,
            },
        ))
    }
    /// Typed surface behavior in primary triangle order, when supported.
    fn triangle_properties(&self) -> Option<&PackedArray<'data, TriangleProperties>> {
        None
    }
    /// Independently indexed draw pools, when supported.
    fn strip_indices(&self) -> Option<&PackedArray<'data, u32>> {
        None
    }
    /// Independently indexed tangent attributes selected by draw bindings.
    fn tangent_pool(&self) -> Option<&PackedArray<'data, Vec4>> {
        None
    }
    /// Model-space vertex positions.
    fn positions(&self) -> &PackedArray<'data, Vec3>;
    /// Vertex normals, when authored.
    fn normals(&self) -> Option<&PackedArray<'data, Vec3>>;
    /// Vertex tangents with handedness, when authored.
    fn tangents(&self) -> Option<&PackedArray<'data, Vec4>>;
    /// Ordered texture-coordinate streams.
    fn texture_coordinates(&self) -> TextureCoordinates<'_, 'data>;
    /// Vertex colors, when authored.
    fn colors(&self) -> Option<&PackedArray<'data, Vec4>>;
    /// Additional authoring attributes supported by this vocabulary.
    fn attributes(&self) -> &[Attribute<'data>];
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
    Variable(&'view ValueGroups<'data, Influence>),
    /// Three ordered influence slots per vertex; zero-weight slots are retained.
    Three(&'view PackedArray<'data, [Influence; 3]>),
    /// Four ordered influence slots per vertex; zero-weight slots are retained.
    Four(&'view PackedArray<'data, [Influence; 4]>),
    /// Direct hierarchy-node contributions, without an invented joint palette.
    Nodes(&'view ValueGroups<'data, NodeInfluence>),
    /// Warcraft III matrix groups, whose members receive equal weights.
    MatrixGroups(&'view ValueGroups<'data, u32>),
}
impl<'view> InfluenceRows<'view, '_> {
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
    /// Borrows rows and decodes contributions lazily without allocating.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = VertexInfluences<'view>> + '_ {
        (0..self.len()).map(|index| self.get(index).expect("checked influence index"))
    }
    /// Borrows binding-slot contributions; direct-node geometry has no binding-slot row.
    pub fn joint_weights(&self, index: usize) -> Option<JointWeights<'view>> {
        self.get(index)?.into_joint_weights()
    }
    /// Borrows one distribution in logical joint order without allocating.
    /// Fixed-width rows decode their three or four slots inline.
    pub fn get(&self, index: usize) -> Option<VertexInfluences<'view>> {
        match self {
            Self::None => None,
            Self::Nodes(values) => values.get(index).map(VertexInfluences::Nodes),
            Self::Variable(values) => values
                .get(index)
                .map(|row| VertexInfluences::Joints(JointWeights::packed(row))),
            Self::Three(values) => values
                .get(index)
                .map(|row| VertexInfluences::Joints(JointWeights::three(row))),
            Self::Four(values) => values
                .get(index)
                .map(|row| VertexInfluences::Joints(JointWeights::four(row))),
            Self::MatrixGroups(values) => values
                .get(index)
                .map(|row| VertexInfluences::Joints(JointWeights::matrix_groups(row))),
        }
    }
}

/// Shared geometry with canonical attributes and explicit connectivity.
#[derive(Clone, Debug, IntoOwned)]
pub struct GeometryData<'data> {
    /// Authored purpose, independent of representation.
    pub kind: GeometryKind,
    /// Vertex positions addressed by connectivity and vertex attributes.
    pub positions: PackedArray<'data, Vec3>,
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
        positions: impl IntoPackedArray<'data, Vec3>,
        triangles: impl IntoPackedArray<'data, [u32; 3]>,
    ) -> Result<Self, Error> {
        let value = Self {
            positions: positions.into_packed_array(),
            connectivity: ConnectivitySets::new(
                Topology::Triangles(triangles.into_packed_array()),
                Vec::new(),
            ),
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
    pub fn set_normals(&mut self, values: PackedArray<'data, Vec3>) {
        self.set_stream(AttributeSemantic::Normal, AttributeValues::Vector3(values));
    }
    /// Replaces the vertex tangent-frame stream; an empty stream removes it.
    pub fn set_tangents(&mut self, values: PackedArray<'data, Vec4>) {
        self.set_stream(AttributeSemantic::Tangent, AttributeValues::Vector4(values));
    }
    /// Replaces the primary vertex color stream; an empty stream removes it.
    pub fn set_colors(&mut self, values: PackedArray<'data, Vec4>) {
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
    pub fn set_texture_coordinates(&mut self, values: Vec<PackedArray<'data, Vec2>>) {
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
    pub(crate) fn validate_vertex_streams(&self, target: ProfileKind) -> Result<(), Error> {
        self.validate()?;
        if self.kind != GeometryKind::Display {
            return Err(Error::at(
                Location::new().field("kind"),
                ProfileError::new(target, ProfileViolation::GeometryKind { actual: self.kind }),
            ));
        }
        self.validate_no_additional_domains(target)?;
        if let Some(attribute) = self.attributes.iter().find(|attribute| {
            attribute.domain != AttributeDomain::Vertex
                || !matches!(
                    attribute.semantic,
                    AttributeSemantic::Normal
                        | AttributeSemantic::Tangent
                        | AttributeSemantic::Color(0)
                        | AttributeSemantic::TexCoord(_)
                )
        }) {
            return Err(Error::at(
                Location::new().attribute(attribute.semantic.clone(), attribute.domain),
                ProfileError::new(
                    target,
                    ProfileViolation::GeometryAttribute {
                        rule: if attribute.domain == AttributeDomain::Vertex {
                            AttributeProjectionRule::Semantic
                        } else {
                            AttributeProjectionRule::Domain
                        },
                        semantic: attribute.semantic.clone().into_owned(),
                        domain: attribute.domain,
                        values: AttributeValueKind::from(&attribute.values),
                    },
                ),
            ));
        }
        Ok(())
    }
    pub(crate) fn validate_no_additional_domains(&self, target: ProfileKind) -> Result<(), Error> {
        if !self.connectivity.additional().is_empty() {
            return Err(Error::at(
                Location::new().field("connectivity").field("additional"),
                ProfileError::new(
                    target,
                    ProfileViolation::AdditionalConnectivity {
                        maximum: 0,
                        actual: self.connectivity.additional().len(),
                    },
                ),
            ));
        }
        if let Some(attribute) = self.attributes.iter().find(|attribute| {
            attribute.domain == AttributeDomain::Pool
                || attribute.semantic == AttributeSemantic::TriangleProperties
        }) {
            return Err(Error::at(
                Location::new().attribute(attribute.semantic.clone(), attribute.domain),
                ProfileError::new(
                    target,
                    ProfileViolation::GeometryAttribute {
                        rule: if attribute.domain == AttributeDomain::Pool {
                            AttributeProjectionRule::Domain
                        } else {
                            AttributeProjectionRule::Semantic
                        },
                        semantic: attribute.semantic.clone().into_owned(),
                        domain: attribute.domain,
                        values: AttributeValueKind::from(&attribute.values),
                    },
                ),
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
    fn triangle_properties(&self) -> Option<&PackedArray<'data, TriangleProperties>> {
        match &self
            .attributes
            .get(
                &AttributeSemantic::TriangleProperties,
                AttributeDomain::Face,
            )?
            .values
        {
            AttributeValues::TriangleProperties(values) => Some(values),
            _ => None,
        }
    }
    fn strip_indices(&self) -> Option<&PackedArray<'data, u32>> {
        match self.connectivity.additional().first()? {
            Topology::TriangleStrip(indices) => Some(indices),
            _ => None,
        }
    }
    fn tangent_pool(&self) -> Option<&PackedArray<'data, Vec4>> {
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
    fn positions(&self) -> &PackedArray<'data, Vec3> {
        &self.positions
    }
    fn normals(&self) -> Option<&PackedArray<'data, Vec3>> {
        self.attributes
            .vector3(&AttributeSemantic::Normal, AttributeDomain::Vertex)
    }
    fn tangents(&self) -> Option<&PackedArray<'data, Vec4>> {
        self.attributes
            .vector4(&AttributeSemantic::Tangent, AttributeDomain::Vertex)
    }
    fn colors(&self) -> Option<&PackedArray<'data, Vec4>> {
        self.attributes
            .vector4(&AttributeSemantic::Color(0), AttributeDomain::Vertex)
    }
    fn texture_coordinates(&self) -> TextureCoordinates<'_, 'data> {
        self.attributes.texture_coordinates()
    }
    fn attributes(&self) -> &[Attribute<'data>] {
        self.attributes.as_slice()
    }
    fn topology(&self) -> &Topology<'data> {
        &self.connectivity.primary
    }
    fn influences(&self) -> InfluenceRows<'_, 'data> {
        self.influences.rows()
    }
    fn validate(&self) -> Result<(), Error> {
        validate_geometry(self)?;
        for (index, topology) in self.connectivity.additional().iter().enumerate() {
            validate_topology(topology, self.positions.len()).map_err(|error| {
                error.with_context(
                    Location::new()
                        .field("connectivity")
                        .field("additional")
                        .index(index),
                )
            })?;
        }
        if let Some(values) = self.triangle_properties() {
            if !matches!(self.connectivity.primary, Topology::Triangles(_)) {
                return Err(Error::at(
                    Location::new().field("triangle_properties"),
                    GeometryError::TrianglePropertiesTopology {
                        actual: TopologyKind::from(&self.connectivity.primary),
                    },
                ));
            }
            let expected = self.connectivity.primary.primitive_count();
            if values.len() != expected {
                return Err(Error::at(
                    Location::new().field("triangle_properties"),
                    GeometryError::CountMismatch {
                        expected,
                        actual: values.len(),
                    },
                ));
            }
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
    fn triangle_properties(&self) -> Option<&PackedArray<'data, TriangleProperties>> {
        match self {
            Self::Left(value) => value.triangle_properties(),
            Self::Right(value) => value.triangle_properties(),
        }
    }
    fn strip_indices(&self) -> Option<&PackedArray<'data, u32>> {
        match self {
            Self::Left(value) => value.strip_indices(),
            Self::Right(value) => value.strip_indices(),
        }
    }
    fn tangent_pool(&self) -> Option<&PackedArray<'data, Vec4>> {
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
    fn positions(&self) -> &PackedArray<'data, Vec3> {
        match self {
            Self::Left(v) => v.positions(),
            Self::Right(v) => v.positions(),
        }
    }
    fn normals(&self) -> Option<&PackedArray<'data, Vec3>> {
        match self {
            Self::Left(v) => v.normals(),
            Self::Right(v) => v.normals(),
        }
    }
    fn tangents(&self) -> Option<&PackedArray<'data, Vec4>> {
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
    fn colors(&self) -> Option<&PackedArray<'data, Vec4>> {
        match self {
            Self::Left(v) => v.colors(),
            Self::Right(v) => v.colors(),
        }
    }
    fn attributes(&self) -> &[Attribute<'data>] {
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

pub(crate) fn present<'view, 'data, T: PackedValue>(
    values: &'view PackedArray<'data, T>,
) -> Option<&'view PackedArray<'data, T>> {
    (!values.is_empty()).then_some(values)
}

pub(crate) fn validate_geometry<'data>(geometry: &impl GeometrySchema<'data>) -> Result<(), Error> {
    let count = geometry.positions().len();
    for (semantic, actual) in geometry
        .normals()
        .map(|v| (AttributeSemantic::Normal, v.len()))
        .into_iter()
        .chain(
            geometry
                .tangents()
                .map(|v| (AttributeSemantic::Tangent, v.len())),
        )
        .chain(
            geometry
                .colors()
                .map(|v| (AttributeSemantic::Color(0), v.len())),
        )
        .chain(
            geometry
                .texture_coordinates()
                .iter()
                .enumerate()
                .filter(|(_, v)| !v.is_empty())
                .map(|(slot, v)| (AttributeSemantic::TexCoord(slot as u32), v.len())),
        )
    {
        if actual != count {
            return Err(Error::at(
                Location::new().attribute(semantic, AttributeDomain::Vertex),
                GeometryError::CountMismatch {
                    expected: count,
                    actual,
                },
            ));
        }
    }
    let influences = geometry.influences().len();
    if influences != 0 && influences != count {
        return Err(Error::at(
            Location::new().field("influences"),
            GeometryError::CountMismatch {
                expected: count,
                actual: influences,
            },
        ));
    }
    check_finite(geometry.positions().iter().map(|v| v.to_array()), || {
        Location::new().field("positions")
    })?;
    if let Some(values) = geometry.normals() {
        check_finite(values.iter().map(|v| v.to_array()), || {
            Location::new().attribute(AttributeSemantic::Normal, AttributeDomain::Vertex)
        })?;
    }
    if let Some(values) = geometry.tangents() {
        check_finite(values.iter().map(|v| v.to_array()), || {
            Location::new().attribute(AttributeSemantic::Tangent, AttributeDomain::Vertex)
        })?;
    }
    if let Some(values) = geometry.colors() {
        check_finite(values.iter().map(|v| v.to_array()), || {
            Location::new().attribute(AttributeSemantic::Color(0), AttributeDomain::Vertex)
        })?;
    }
    for (slot, values) in geometry.texture_coordinates().iter().enumerate() {
        check_finite(values.iter().map(|v| v.to_array()), || {
            Location::new().attribute(
                AttributeSemantic::TexCoord(slot as u32),
                AttributeDomain::Vertex,
            )
        })?;
    }
    validate_topology(geometry.topology(), count)?;
    let faces = geometry.topology().primitive_count();
    let corners = geometry.topology().corner_count();
    for attribute in geometry.attributes() {
        let expected = match attribute.domain {
            AttributeDomain::Vertex => count,
            AttributeDomain::Corner => corners,
            AttributeDomain::Face => faces,
            AttributeDomain::ConnectivityFace(_)
            | AttributeDomain::ConnectivityCorner(_)
            | AttributeDomain::Pool => attribute.values.len(),
            AttributeDomain::Mesh => 1,
        };
        let empty_uv_slot = attribute.domain == AttributeDomain::Vertex
            && matches!(attribute.semantic, AttributeSemantic::TexCoord(_))
            && attribute.values.is_empty();
        if attribute.values.len() != expected && !empty_uv_slot {
            return Err(Error::at(
                Location::new().attribute(attribute.semantic.clone(), attribute.domain),
                GeometryError::CountMismatch {
                    expected,
                    actual: attribute.values.len(),
                },
            ));
        }
        macro_rules! finite {
            ($values:expr) => {
                check_finite($values.iter().map(|v| v.to_array()), || {
                    Location::new().attribute(attribute.semantic.clone(), attribute.domain)
                })?
            };
        }
        match &attribute.values {
            AttributeValues::Scalar(v) => check_finite_values(v.iter(), || {
                Location::new().attribute(attribute.semantic.clone(), attribute.domain)
            })?,
            AttributeValues::Vector2(v) => finite!(v),
            AttributeValues::Vector3(v) => finite!(v),
            AttributeValues::Vector4(v) => finite!(v),
            AttributeValues::Quaternion(v) => finite!(v),
            _ => {}
        }
    }
    let rows = geometry.influences();
    for index in 0..rows.len() {
        let row = rows.get(index).expect("checked row index");
        let sum = row.weights().sum::<f32>();
        for (influence, weight) in row.weights().enumerate() {
            if !weight.is_finite() || weight < 0. {
                return Err(Error::at(
                    Location::new()
                        .field("influences")
                        .index(index)
                        .index(influence)
                        .field("weight"),
                    GeometryError::InvalidWeight { weight },
                ));
            }
        }
        if sum != 0. && (sum - 1.).abs() > 0.0001 {
            return Err(Error::at(
                Location::new().field("influences").index(index),
                GeometryError::UnnormalizedWeights {
                    sum,
                    tolerance: 0.0001,
                },
            ));
        }
    }
    Ok(())
}

pub(crate) fn check_finite<const N: usize>(
    values: impl IntoIterator<Item = [f32; N]>,
    location: impl Fn() -> Location,
) -> Result<(), Error> {
    for (index, value) in values.into_iter().enumerate() {
        check_finite_values(value, || location().index(index))?;
    }
    Ok(())
}

/// Checks a scalar sequence with one actual index level, such as vector components.
pub(crate) fn check_finite_values(
    values: impl IntoIterator<Item = f32>,
    location: impl FnOnce() -> Location,
) -> Result<(), Error> {
    if let Some((index, value)) = values
        .into_iter()
        .enumerate()
        .find(|(_, value)| !value.is_finite())
    {
        return Err(Error::at(
            location().index(index),
            GeometryError::NonFinite { value },
        ));
    }
    Ok(())
}

fn validate_topology(topology: &Topology<'_>, vertices: usize) -> Result<(), Error> {
    fn indices(values: impl IntoIterator<Item = u32>, vertices: usize) -> Result<(), Error> {
        for (corner, index) in values.into_iter().enumerate() {
            if index as usize >= vertices {
                return Err(Error::at(
                    Location::new().field("indices").index(corner),
                    GeometryError::IndexOutOfBounds {
                        index: index as usize,
                        len: vertices,
                    },
                ));
            }
        }
        Ok(())
    }
    match topology {
        Topology::Points(values) | Topology::TriangleStrip(values) => {
            indices(values.iter(), vertices)
        }
        Topology::Lines(values) => indices(values.iter().flatten(), vertices),
        Topology::Triangles(values) => indices(values.iter().flatten(), vertices),
        Topology::Mixed(groups) => {
            for (index, group) in groups.iter().enumerate() {
                validate_topology(group, vertices).map_err(|error| {
                    error.with_context(Location::new().field("groups").index(index))
                })?;
            }
            Ok(())
        }
        Topology::Polygons { corners, offsets } => {
            if offsets.get(0) != Some(0) {
                return Err(Error::at(
                    Location::new().field("offsets").index(0),
                    GeometryError::PolygonBoundary {
                        rule: PolygonBoundaryRule::StartsAtZero,
                        previous: None,
                        actual: offsets.get(0),
                        corners: corners.len(),
                    },
                ));
            }
            let last = offsets.len() - 1;
            if offsets.get(last).map(|value| value as usize) != Some(corners.len()) {
                return Err(Error::at(
                    Location::new().field("offsets").index(last),
                    GeometryError::PolygonBoundary {
                        rule: PolygonBoundaryRule::EndsAtCornerCount,
                        previous: None,
                        actual: offsets.get(last),
                        corners: corners.len(),
                    },
                ));
            }
            let mut previous = 0;
            for (index, actual) in offsets.iter().enumerate().skip(1) {
                if actual < previous || actual - previous < 3 {
                    return Err(Error::at(
                        Location::new().field("offsets").index(index),
                        GeometryError::PolygonBoundary {
                            rule: PolygonBoundaryRule::OrderedWithThreeCorners,
                            previous: Some(previous),
                            actual: Some(actual),
                            corners: corners.len(),
                        },
                    ));
                }
                previous = actual;
            }
            indices(corners.iter(), vertices)
        }
    }
    .map_err(|error| error.with_context(Location::new().field("topology")))
}

impl Default for GeometryData<'_> {
    fn default() -> Self {
        Self {
            kind: GeometryKind::Display,
            positions: PackedArray::default(),
            connectivity: ConnectivitySets::default(),
            attributes: GeometryAttributes::default(),
            influences: GeometryInfluences::None,
        }
    }
}
impl ValidateLinks for GeometryData<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for attribute in &self.attributes {
            if let AttributeValues::TriangleProperties(values) = &attribute.values {
                for face in values.iter() {
                    face.material.visit_links(visitor);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::common::Borrowed as _;
    use crate::model::common::PackedArray;
    use crate::model::*;

    #[test]
    fn borrowed_geometry_rejects_invalid_streams_and_indices() {
        let positions = PackedArray::alloc_from_slice(&[Vec3::ZERO, Vec3::X, Vec3::Y]);
        let triangles = [[0, 1, 2]];
        let geometry = MdlxGeometry::triangles(positions.as_borrowed(), &triangles[..]).unwrap();
        assert!(!geometry.positions().is_owned());
        assert!(geometry
            .clone()
            .with_normals(
                (vec![[0., 0., 1.]])
                    .into_iter()
                    .map(Vec3::from_array)
                    .collect::<PackedArray<_>>()
            )
            .is_err());
        assert!(geometry
            .with_tangents(
                (vec![[0., 0., f32::NAN, 1.]; 3])
                    .into_iter()
                    .map(Vec4::from_array)
                    .collect::<PackedArray<_>>()
            )
            .is_err());
        assert!(MdlxGeometry::triangles(positions.as_borrowed(), vec![[0, 1, 3]]).is_err());
    }

    #[test]
    fn four_slots_and_matrix_groups_widen_without_losing_distribution() {
        let positions = PackedArray::alloc_from_slice(&[Vec3::ZERO]);
        let groups = [vec![3, 8, 12, 19, 24]];
        let geometry = MdlxGeometry::triangles(positions.as_borrowed(), Vec::<[u32; 3]>::new())
            .unwrap()
            .with_influences(JointInfluences::MatrixGroups(ValueGroups::from_slice(
                &groups,
            )))
            .unwrap();
        let widened = GeometryData::from(geometry);
        assert_eq!(
            widened.positions().as_bytes().as_ptr(),
            positions.as_bytes().as_ptr()
        );
        assert!(
            matches!(widened.influences(), InfluenceRows::MatrixGroups(values) if values.to_vec() == groups)
        );
        let row = widened.influences().joint_weights(0).unwrap();
        assert_eq!(row.len(), 5);
        assert_eq!(row.get(4).unwrap().joint, 24);
        assert!((row.get(4).unwrap().weight - 0.2).abs() < f32::EPSILON);
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
            (positions)
                .into_iter()
                .map(Vec3::from_array)
                .collect::<PackedArray<_>>(),
            (normals)
                .into_iter()
                .map(Vec3::from_array)
                .collect::<PackedArray<_>>(),
            [
                PackedArray::alloc_from_slice(&[Vec2::from_array([0., 0.])]),
                PackedArray::default()
            ],
            Vec::<[u32; 3]>::new(),
            weights
        )
        .is_err());
    }
}
