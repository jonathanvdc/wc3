//! Static world geometry with the authored world-model stream vocabulary.
use crate::model::common::IntoOwned;
use crate::model::common::IntoPackedArray;
use crate::model::TextureCoordinates;
use glam::{Vec2, Vec3, Vec4};
use std::collections::BTreeMap;

use crate::model::error::{
    AttributeValueKind, GeometryError, InfluenceKind, Location, ProfileError, ProfileKind,
    ProfileViolation, TopologyKind,
};
use crate::model::geometry_edit::check_edit;
use crate::model::geometry_schema::{check_finite, validate_geometry};
use crate::model::vertex_geometry::VertexStreams;
use crate::model::VertexData;
use crate::model::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, DrawSubsetCapabilities, Error,
    GeometryData, GeometryInfluences, GeometryKind, GeometryParts, GeometryPools, GeometrySchema,
    InfluenceRows, Link, PackedArray, Topology, TriangleProperties, ValidateLinks,
};
use crate::model::{GeometryEdit, GeometryEditCapabilities, StreamSupport};

type WmoStreams<'data> = VertexStreams<
    'data,
    (),
    Vec<PackedArray<'data, Vec2>>,
    (),
    Vec<PackedArray<'data, Vec4>>,
    PackedArray<'data, Vec3>,
    PackedArray<'data, Vec2>,
>;

/// Static triangle geometry for a world model.
///
/// Coordinate and color streams retain their authored order. Native dialects
/// decide which stream counts can be encoded; geometry authoring itself does
/// not impose the limits of one particular file revision.
#[derive(Clone, Debug, IntoOwned)]
pub struct WmoGeometry<'data> {
    streams: WmoStreams<'data>,
    pools: GeometryPools<'data>,
    triangle_properties: PackedArray<'data, TriangleProperties>,
}
impl<'data> WmoGeometry<'data> {
    /// Replaces positions while checking all geometry stream relationships.
    pub fn with_positions(mut self, positions: PackedArray<'data, Vec3>) -> Result<Self, Error> {
        self.streams.positions = positions;
        self.validate()?;
        Ok(self)
    }
    /// Creates checked triangle geometry while retaining borrowed arrays.
    pub fn triangles(
        positions: impl IntoPackedArray<'data, Vec3>,
        triangles: impl IntoPackedArray<'data, [u32; 3]>,
    ) -> Result<Self, Error> {
        let triangles = triangles.into_packed_array();
        let triangle_properties = (0..triangles.len())
            .map(|_| TriangleProperties::default())
            .collect();
        let geometry = Self {
            streams: VertexStreams {
                vertices: VertexData {
                    positions: positions.into_packed_array(),
                    normals: PackedArray::default(),
                    texture_coordinates: Vec::new(),
                    colors: Vec::new(),
                    tangents: (),
                    extras: PackedArray::default(),
                },
                topology: Topology::Triangles(triangles),
                influences: (),
            },
            pools: GeometryPools::default(),
            triangle_properties,
        };
        geometry.validate()?;
        Ok(geometry)
    }
    /// Sets per-triangle rendering, collision, and query behavior.
    pub fn with_triangle_properties(
        mut self,
        properties: PackedArray<'data, TriangleProperties>,
    ) -> Result<Self, Error> {
        self.triangle_properties = properties;
        self.validate()?;
        Ok(self)
    }
    /// Sets shared strip connectivity selected by individual draw windows.
    pub fn with_strip_indices(mut self, indices: PackedArray<'data, u32>) -> Result<Self, Error> {
        self.pools.triangle_strip_indices = indices;
        self.validate()?;
        Ok(self)
    }
    /// Reads shared strip connectivity selected by individual draw windows.
    pub fn strip_indices(&self) -> &PackedArray<'data, u32> {
        &self.pools.triangle_strip_indices
    }
    /// Sets the shared tangent-vector pool selected by individual draws.
    pub fn with_tangent_vectors(mut self, values: PackedArray<'data, Vec4>) -> Result<Self, Error> {
        self.pools.tangent_vectors = values;
        self.validate()?;
        Ok(self)
    }
    /// Reads the shared tangent-vector pool selected by individual draws.
    pub fn tangent_vectors(&self) -> &PackedArray<'data, Vec4> {
        &self.pools.tangent_vectors
    }
    /// Sets the independently authored baked-light coordinate stream.
    pub fn with_lightmap_coordinates(
        mut self,
        coordinates: PackedArray<'data, Vec2>,
    ) -> Result<Self, Error> {
        self.streams.extras = coordinates;
        self.validate()?;
        Ok(self)
    }
    /// Reads the independently authored baked-light coordinate stream.
    pub fn lightmap_coordinates(&self) -> &PackedArray<'data, Vec2> {
        &self.streams.extras
    }
    /// Reads per-triangle properties in connectivity order.
    pub fn triangle_properties(&self) -> &PackedArray<'data, TriangleProperties> {
        &self.triangle_properties
    }
    /// Sets authored vertex normals.
    pub fn with_normals(
        mut self,
        normals: impl IntoPackedArray<'data, Vec3>,
    ) -> Result<Self, Error> {
        self.streams.normals = normals.into_packed_array();
        self.validate()?;
        Ok(self)
    }
    /// Sets ordered texture-coordinate streams.
    pub fn with_texture_coordinates(
        mut self,
        coordinates: Vec<PackedArray<'data, Vec2>>,
    ) -> Result<Self, Error> {
        self.streams.texture_coordinates = coordinates;
        self.validate()?;
        Ok(self)
    }
    /// Sets ordered vertex-color streams, including independent lighting sets.
    pub fn with_color_sets(mut self, colors: Vec<PackedArray<'data, Vec4>>) -> Result<Self, Error> {
        self.streams.colors = colors;
        self.validate()?;
        Ok(self)
    }
    /// Reads all independently authored vertex-color sets.
    pub fn color_sets(&self) -> &[PackedArray<'data, Vec4>] {
        &self.streams.colors
    }
}
impl<'data> GeometrySchema<'data> for WmoGeometry<'data> {
    fn subset_capabilities(&self) -> DrawSubsetCapabilities {
        DrawSubsetCapabilities {
            alternate_selection: false,
            supplemental_strips: true,
            vertex_windows: true,
            tangent_bindings: true,
        }
    }
    fn triangle_properties(&self) -> Option<&PackedArray<'data, TriangleProperties>> {
        Some(&self.triangle_properties)
    }
    fn strip_indices(&self) -> Option<&PackedArray<'data, u32>> {
        Some(&self.pools.triangle_strip_indices)
    }
    fn tangent_pool(&self) -> Option<&PackedArray<'data, Vec4>> {
        Some(&self.pools.tangent_vectors)
    }
    fn edit_capabilities(&self) -> GeometryEditCapabilities {
        GeometryEditCapabilities {
            positions: true,
            normals: StreamSupport::Optional,
            tangents: StreamSupport::Unsupported,
            colors: StreamSupport::Optional,
            minimum_uv_sets: 0,
            maximum_uv_sets: None,
            vertex_remap: self.pools.triangle_strip_indices.is_empty(),
        }
    }
    fn edited(&self, edit: GeometryEdit<'data>) -> Result<Self, Error> {
        self.validate()?;
        check_edit(&edit, self.edit_capabilities())?;
        let mut result = self.clone();
        if let Some(map) = &edit.remap {
            map.check(self)?;
            result.streams.vertices.remap(&map.new_to_old);
            result.streams.topology = map.topology.clone();
        }
        if let Some(values) = edit.positions {
            result.streams.positions = values;
        }
        if let Some(values) = edit.normals {
            result.streams.normals = values;
        }
        if let Some(values) = edit.texture_coordinates {
            result.streams.texture_coordinates = values;
        }
        if let Some(values) = edit.colors {
            if values.is_empty() {
                if !result.streams.colors.is_empty() {
                    result.streams.colors.remove(0);
                }
            } else if result.streams.colors.is_empty() {
                result.streams.colors.push(values);
            } else {
                result.streams.colors[0] = values;
            }
        }
        result.validate()?;
        Ok(result)
    }

    fn positions(&self) -> &PackedArray<'data, Vec3> {
        &self.streams.positions
    }
    fn normals(&self) -> Option<&PackedArray<'data, Vec3>> {
        (!self.streams.normals.is_empty()).then_some(&self.streams.normals)
    }
    fn tangents(&self) -> Option<&PackedArray<'data, Vec4>> {
        None
    }
    fn texture_coordinates(&self) -> TextureCoordinates<'_, 'data> {
        TextureCoordinates::Slice(&self.streams.texture_coordinates)
    }
    fn colors(&self) -> Option<&PackedArray<'data, Vec4>> {
        self.streams.colors.first()
    }
    fn attributes(&self) -> &[Attribute<'data>] {
        &[]
    }
    fn topology(&self) -> &Topology<'data> {
        &self.streams.topology
    }
    fn influences(&self) -> InfluenceRows<'_, 'data> {
        InfluenceRows::None
    }
    fn validate(&self) -> Result<(), Error> {
        validate_geometry(self)?;
        if let Some((index, value)) = self
            .pools
            .triangle_strip_indices
            .iter()
            .enumerate()
            .find(|(_, value)| *value as usize >= self.streams.positions.len())
        {
            return Err(Error::at(
                Location::new()
                    .field("pools")
                    .field("triangle_strip_indices")
                    .index(index),
                GeometryError::IndexOutOfBounds {
                    index: value as usize,
                    len: self.streams.positions.len(),
                },
            ));
        }
        check_finite(
            self.pools.tangent_vectors.iter().map(|v| v.to_array()),
            || Location::new().field("pools").field("tangent_vectors"),
        )?;
        if !self.triangle_properties.is_empty()
            && self.triangle_properties.len() != self.streams.topology.primitive_count()
        {
            return Err(Error::at(
                Location::new().field("triangle_properties"),
                GeometryError::CountMismatch {
                    expected: self.streams.topology.primitive_count(),
                    actual: self.triangle_properties.len(),
                },
            ));
        }
        if !self.streams.extras.is_empty() {
            if self.streams.extras.len() != self.streams.positions.len() {
                return Err(Error::at(
                    Location::new().attribute(
                        AttributeSemantic::LightmapCoordinates,
                        AttributeDomain::Vertex,
                    ),
                    GeometryError::CountMismatch {
                        expected: self.streams.positions.len(),
                        actual: self.streams.extras.len(),
                    },
                ));
            }
            check_finite(self.streams.extras.iter().map(|v| v.to_array()), || {
                Location::new().attribute(
                    AttributeSemantic::LightmapCoordinates,
                    AttributeDomain::Vertex,
                )
            })?;
        }
        for (slot, colors) in self.streams.colors.iter().enumerate() {
            if colors.len() != self.streams.positions.len() {
                return Err(Error::at(
                    Location::new().attribute(
                        AttributeSemantic::Color(slot as u32),
                        AttributeDomain::Vertex,
                    ),
                    GeometryError::CountMismatch {
                        expected: self.streams.positions.len(),
                        actual: colors.len(),
                    },
                ));
            }
            check_finite(colors.iter().map(|v| v.to_array()), || {
                Location::new().attribute(
                    AttributeSemantic::Color(slot as u32),
                    AttributeDomain::Vertex,
                )
            })?;
        }
        Ok(())
    }
}

impl Default for WmoGeometry<'_> {
    fn default() -> Self {
        Self {
            streams: VertexStreams {
                vertices: VertexData {
                    positions: PackedArray::default(),
                    normals: PackedArray::default(),
                    texture_coordinates: Vec::new(),
                    colors: Vec::new(),
                    tangents: (),
                    extras: PackedArray::default(),
                },
                topology: Topology::Triangles(PackedArray::default()),
                influences: (),
            },
            pools: GeometryPools::default(),
            triangle_properties: PackedArray::default(),
        }
    }
}

impl ValidateLinks for WmoGeometry<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for face in self.triangle_properties.iter() {
            if let Some(material) = face.material {
                visitor(Link::Material(material));
            }
        }
    }
}

impl<'data> From<WmoGeometry<'data>> for GeometryData<'data> {
    fn from(value: WmoGeometry<'data>) -> Self {
        let vertices = value.streams.vertices;
        let mut attributes = Vec::new();
        for (slot, colors) in vertices.colors.into_iter().enumerate() {
            attributes.push(Attribute {
                semantic: AttributeSemantic::Color(slot as u32),
                domain: AttributeDomain::Vertex,
                values: AttributeValues::Vector4(colors),
            });
        }
        if !vertices.extras.is_empty() {
            attributes.push(Attribute {
                semantic: AttributeSemantic::LightmapCoordinates,
                domain: AttributeDomain::Vertex,
                values: AttributeValues::Vector2(vertices.extras),
            });
        }
        GeometryParts {
            strip_set: true,
            positions: vertices.positions,
            normals: vertices.normals,
            texture_coordinates: vertices.texture_coordinates,
            topology: value.streams.topology,
            attributes,
            triangle_properties: value.triangle_properties,
            pools: GeometryPools {
                triangle_strip_indices: value.pools.triangle_strip_indices,
                tangent_vectors: value.pools.tangent_vectors,
            },
            ..GeometryParts::default()
        }
        .into_data()
        .expect("constrained geometry has unique typed attributes")
    }
}

impl<'data> TryFrom<GeometryData<'data>> for WmoGeometry<'data> {
    type Error = Error;
    fn try_from(value: GeometryData<'data>) -> Result<Self, Error> {
        value.validate()?;
        if value.connectivity.additional().len() > 1 {
            return Err(Error::at(
                Location::new().field("connectivity").field("additional"),
                ProfileError::new(
                    ProfileKind::Wmo,
                    ProfileViolation::AdditionalConnectivity {
                        maximum: 1,
                        actual: value.connectivity.additional().len(),
                    },
                ),
            ));
        }
        if let Some((index, topology)) = value
            .connectivity
            .additional()
            .iter()
            .enumerate()
            .find(|(_, topology)| !matches!(topology, Topology::TriangleStrip(_)))
        {
            return Err(Error::at(
                Location::new()
                    .field("connectivity")
                    .field("additional")
                    .index(index),
                ProfileError::new(
                    ProfileKind::Wmo,
                    ProfileViolation::GeometryTopology {
                        actual: TopologyKind::from(topology),
                    },
                ),
            ));
        }
        let value = value.into_parts();
        if value.kind != GeometryKind::Display {
            return Err(Error::at(
                Location::new().field("kind"),
                ProfileError::new(
                    ProfileKind::Wmo,
                    ProfileViolation::GeometryKind { actual: value.kind },
                ),
            ));
        }
        if !matches!(value.topology, Topology::Triangles(_)) {
            return Err(Error::at(
                Location::new().field("topology"),
                ProfileError::new(
                    ProfileKind::Wmo,
                    ProfileViolation::GeometryTopology {
                        actual: TopologyKind::from(&value.topology),
                    },
                ),
            ));
        }
        if !value.tangents.is_empty() {
            return Err(Error::at(
                Location::new().field("tangents"),
                ProfileError::new(
                    ProfileKind::Wmo,
                    ProfileViolation::GeometryStream {
                        semantic: AttributeSemantic::Tangent,
                    },
                ),
            ));
        }
        if !matches!(value.influences, GeometryInfluences::None) {
            return Err(Error::at(
                Location::new().field("influences"),
                ProfileError::new(
                    ProfileKind::Wmo,
                    ProfileViolation::GeometryInfluences {
                        actual: InfluenceKind::from(&value.influences),
                    },
                ),
            ));
        }
        let mut colors = BTreeMap::new();
        if !value.colors.is_empty() {
            colors.insert(0, value.colors);
        }
        let mut lightmap = None;
        for attribute in value.attributes {
            if attribute.domain != AttributeDomain::Vertex {
                return Err(Error::at(
                    Location::new().attribute(attribute.semantic.clone(), attribute.domain),
                    ProfileError::new(
                        ProfileKind::Wmo,
                        ProfileViolation::GeometryAttribute {
                            semantic: attribute.semantic.clone().into_owned(),
                            domain: attribute.domain,
                            values: AttributeValueKind::from(&attribute.values),
                        },
                    ),
                ));
            }
            let admissible = match (&attribute.semantic, &attribute.values) {
                (AttributeSemantic::Color(slot), AttributeValues::Vector4(_)) => {
                    !colors.contains_key(slot)
                }
                (AttributeSemantic::LightmapCoordinates, AttributeValues::Vector2(values)) => {
                    lightmap.is_none() && !values.is_empty()
                }
                _ => false,
            };
            if !admissible {
                return Err(Error::at(
                    Location::new().attribute(attribute.semantic.clone(), attribute.domain),
                    ProfileError::new(
                        ProfileKind::Wmo,
                        ProfileViolation::GeometryAttribute {
                            semantic: attribute.semantic.clone().into_owned(),
                            domain: attribute.domain,
                            values: AttributeValueKind::from(&attribute.values),
                        },
                    ),
                ));
            }
            match (attribute.semantic, attribute.values) {
                (AttributeSemantic::Color(slot), AttributeValues::Vector4(values))
                    if !colors.contains_key(&slot) =>
                {
                    colors.insert(slot, values);
                }
                (AttributeSemantic::LightmapCoordinates, AttributeValues::Vector2(values))
                    if lightmap.is_none() && !values.is_empty() =>
                {
                    lightmap = Some(values)
                }
                _ => unreachable!("checked attribute projection"),
            }
        }
        if let Some((expected, actual)) = colors
            .keys()
            .copied()
            .enumerate()
            .find(|(index, slot)| *index as u32 != *slot)
        {
            return Err(Error::at(
                Location::new().field("colors"),
                ProfileError::new(
                    ProfileKind::Wmo,
                    ProfileViolation::ColorSlot {
                        expected: expected as u32,
                        actual,
                    },
                ),
            ));
        }
        let result = Self {
            streams: VertexStreams {
                vertices: VertexData {
                    positions: value.positions,
                    normals: value.normals,
                    texture_coordinates: value.texture_coordinates,
                    colors: colors.into_values().collect(),
                    tangents: (),
                    extras: lightmap.unwrap_or_default(),
                },
                topology: value.topology,
                influences: (),
            },
            triangle_properties: value.triangle_properties,
            pools: value.pools,
        };
        result.validate()?;
        Ok(result)
    }
}
