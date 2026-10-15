//! Static world geometry with the authored world-model stream vocabulary.
use crate::model::semantic::TextureCoordinates;
use std::collections::BTreeMap;

use crate::model::semantic::geometry_edit::check_edit;
use crate::model::semantic::geometry_schema::validate_geometry;
use crate::model::semantic::vertex_geometry::VertexStreams;
use crate::model::semantic::VertexData;
use crate::model::semantic::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, Attributes,
    DrawSubsetCapabilities, Error, GeometryData, GeometryInfluences, GeometryKind, GeometryParts,
    GeometryPools, GeometrySchema, InfluenceRows, Link, SurfaceTriangleControls, Topology,
    ValidateLinks, Values,
};
use crate::model::semantic::{GeometryEdit, GeometryEditCapabilities, StreamSupport};

/// Static triangle geometry for a world model.
///
/// Coordinate and color streams retain their authored order. Native dialects
/// decide which stream counts can be encoded; geometry authoring itself does
/// not impose the limits of one particular file revision.
#[derive(Clone, Debug)]
pub struct WmoGeometry<'data> {
    streams: VertexStreams<
        'data,
        (),
        Vec<Values<'data, [f32; 2]>>,
        (),
        Vec<Values<'data, [f32; 4]>>,
        Values<'data, [f32; 3]>,
        Values<'data, [f32; 2]>,
    >,
    pools: GeometryPools<'data>,
    triangle_controls: Values<'data, SurfaceTriangleControls>,
}
impl<'data> WmoGeometry<'data> {
    /// Replaces positions while checking all geometry stream relationships.
    pub fn with_positions(mut self, positions: Values<'data, [f32; 3]>) -> Result<Self, Error> {
        self.streams.positions = positions;
        self.validate()?;
        Ok(self)
    }
    /// Creates checked triangle geometry while retaining borrowed arrays.
    pub fn triangles(
        positions: impl Into<Values<'data, [f32; 3]>>,
        triangles: impl Into<Values<'data, [u32; 3]>>,
    ) -> Result<Self, Error> {
        let triangles = triangles.into();
        let triangle_controls =
            Values::view(triangles.len(), |_| SurfaceTriangleControls::default());
        let geometry = Self {
            streams: VertexStreams {
                vertices: VertexData {
                    positions: positions.into(),
                    normals: Values::default(),
                    texture_coordinates: Vec::new(),
                    colors: Vec::new(),
                    tangents: (),
                    extras: Values::default(),
                },
                topology: Topology::Triangles(triangles),
                influences: (),
            },
            pools: GeometryPools::default(),
            triangle_controls,
        };
        geometry.validate()?;
        Ok(geometry)
    }
    /// Sets per-triangle rendering, collision, and query behavior.
    pub fn with_triangle_controls(
        mut self,
        controls: Values<'data, SurfaceTriangleControls>,
    ) -> Result<Self, Error> {
        self.triangle_controls = controls;
        self.validate()?;
        Ok(self)
    }
    /// Sets shared strip connectivity selected by individual draw windows.
    pub fn with_strip_indices(mut self, indices: Values<'data, u32>) -> Result<Self, Error> {
        self.pools.triangle_strip_indices = indices;
        self.validate()?;
        Ok(self)
    }
    /// Reads shared strip connectivity selected by individual draw windows.
    pub fn strip_indices(&self) -> &Values<'data, u32> {
        &self.pools.triangle_strip_indices
    }
    /// Sets the shared tangent-vector pool selected by individual draws.
    pub fn with_tangent_vectors(mut self, values: Values<'data, [f32; 4]>) -> Result<Self, Error> {
        self.pools.tangent_vectors = values;
        self.validate()?;
        Ok(self)
    }
    /// Reads the shared tangent-vector pool selected by individual draws.
    pub fn tangent_vectors(&self) -> &Values<'data, [f32; 4]> {
        &self.pools.tangent_vectors
    }
    /// Sets the independently authored baked-light coordinate stream.
    pub fn with_lightmap_coordinates(
        mut self,
        coordinates: Values<'data, [f32; 2]>,
    ) -> Result<Self, Error> {
        self.streams.extras = coordinates;
        self.validate()?;
        Ok(self)
    }
    /// Reads the independently authored baked-light coordinate stream.
    pub fn lightmap_coordinates(&self) -> &Values<'data, [f32; 2]> {
        &self.streams.extras
    }
    /// Reads per-triangle controls in connectivity order.
    pub fn triangle_controls(&self) -> &Values<'data, SurfaceTriangleControls> {
        &self.triangle_controls
    }
    /// Sets authored vertex normals.
    pub fn with_normals(
        mut self,
        normals: impl Into<Values<'data, [f32; 3]>>,
    ) -> Result<Self, Error> {
        self.streams.normals = normals.into();
        self.validate()?;
        Ok(self)
    }
    /// Sets ordered texture-coordinate streams.
    pub fn with_texture_coordinates(
        mut self,
        coordinates: Vec<Values<'data, [f32; 2]>>,
    ) -> Result<Self, Error> {
        self.streams.texture_coordinates = coordinates;
        self.validate()?;
        Ok(self)
    }
    /// Sets ordered vertex-color streams, including independent lighting sets.
    pub fn with_color_sets(mut self, colors: Vec<Values<'data, [f32; 4]>>) -> Result<Self, Error> {
        self.streams.colors = colors;
        self.validate()?;
        Ok(self)
    }
    /// Reads all independently authored vertex-color sets.
    pub fn color_sets(&self) -> &[Values<'data, [f32; 4]>] {
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
    fn surface_triangles(&self) -> Option<&Values<'data, SurfaceTriangleControls>> {
        Some(&self.triangle_controls)
    }
    fn strip_indices(&self) -> Option<&Values<'data, u32>> {
        Some(&self.pools.triangle_strip_indices)
    }
    fn tangent_pool(&self) -> Option<&Values<'data, [f32; 4]>> {
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

    fn positions(&self) -> &Values<'data, [f32; 3]> {
        &self.streams.positions
    }
    fn normals(&self) -> Option<&Values<'data, [f32; 3]>> {
        (!self.streams.normals.is_empty()).then_some(&self.streams.normals)
    }
    fn tangents(&self) -> Option<&Values<'data, [f32; 4]>> {
        None
    }
    fn texture_coordinates(&self) -> TextureCoordinates<'_, 'data> {
        TextureCoordinates::Slice(&self.streams.texture_coordinates)
    }
    fn colors(&self) -> Option<&Values<'data, [f32; 4]>> {
        self.streams.colors.first()
    }
    fn attributes(&self) -> Attributes<'_, 'data> {
        Attributes::Slice(&[])
    }
    fn topology(&self) -> &Topology<'data> {
        &self.streams.topology
    }
    fn influences(&self) -> InfluenceRows<'_, 'data> {
        InfluenceRows::None
    }
    fn validate(&self) -> Result<(), Error> {
        validate_geometry(self)?;
        if self
            .pools
            .triangle_strip_indices
            .iter()
            .any(|index| index as usize >= self.streams.positions.len())
            || !self
                .pools
                .tangent_vectors
                .iter()
                .flatten()
                .all(f32::is_finite)
        {
            return Err(Error::new(
                "geometry.draw_streams",
                "strip indices must address vertices and tangent vectors must be finite",
            ));
        }
        if !self.triangle_controls.is_empty()
            && self.triangle_controls.len() != self.streams.topology.primitive_count()
        {
            return Err(Error::new(
                "geometry.triangle_controls",
                "controls must match triangle count",
            ));
        }
        if !self.streams.extras.is_empty()
            && (self.streams.extras.len() != self.streams.positions.len()
                || !self.streams.extras.iter().flatten().all(f32::is_finite))
        {
            return Err(Error::new(
                "geometry.lightmap_coordinates",
                "baked-light coordinates must contain one finite coordinate per vertex",
            ));
        }
        for colors in &self.streams.colors {
            if colors.len() != self.streams.positions.len()
                || !colors.iter().flatten().all(f32::is_finite)
            {
                return Err(Error::new(
                    "geometry.colors",
                    "color streams must contain one finite color per vertex",
                ));
            }
        }
        Ok(())
    }
}

impl Default for WmoGeometry<'_> {
    fn default() -> Self {
        Self {
            streams: VertexStreams {
                vertices: VertexData {
                    positions: Values::default(),
                    normals: Values::default(),
                    texture_coordinates: Vec::new(),
                    colors: Vec::new(),
                    tangents: (),
                    extras: Values::default(),
                },
                topology: Topology::Triangles(Values::default()),
                influences: (),
            },
            pools: GeometryPools::default(),
            triangle_controls: Values::default(),
        }
    }
}

impl ValidateLinks for WmoGeometry<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for face in self.triangle_controls.iter() {
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
            surface_triangles: value.triangle_controls,
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
        if value.connectivity.additional().len() > 1
            || value
                .connectivity
                .additional()
                .iter()
                .any(|topology| !matches!(topology, Topology::TriangleStrip(_)))
        {
            return Err(Error::new(
                "world.geometry.connectivity",
                "world geometry retains primary triangles and one strip set",
            ));
        }
        let value = value.into_parts();
        if value.kind != GeometryKind::Display
            || !matches!(value.topology, Topology::Triangles(_))
            || !value.tangents.is_empty()
            || !matches!(value.influences, GeometryInfluences::None)
        {
            return Err(Error::new(
                "world.geometry",
                "expected unskinned display triangles without vertex tangent frames",
            ));
        }
        let mut colors = BTreeMap::new();
        if !value.colors.is_empty() {
            colors.insert(0, value.colors);
        }
        let mut lightmap = None;
        for attribute in value.attributes {
            if attribute.domain != AttributeDomain::Vertex {
                return Err(Error::new(
                    "world.geometry.attributes",
                    "expected vertex-domain attributes",
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
                _ => {
                    return Err(Error::new(
                        "world.geometry.attributes",
                        "unsupported, duplicate, empty, or incorrectly typed attribute",
                    ))
                }
            }
        }
        if colors
            .keys()
            .copied()
            .enumerate()
            .any(|(index, slot)| index as u32 != slot)
        {
            return Err(Error::new(
                "world.geometry.colors",
                "color slots must be contiguous from zero",
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
            triangle_controls: value.surface_triangles,
            pools: value.pools,
        };
        result.validate()?;
        Ok(result)
    }
}
