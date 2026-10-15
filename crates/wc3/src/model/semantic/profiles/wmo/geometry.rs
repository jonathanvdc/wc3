//! Static world geometry with the authored world-model stream vocabulary.
use super::TriangleFlags;
use crate::model::semantic::geometry_edit::{check_edit, remap_values};
use crate::model::semantic::geometry_schema::validate_geometry;
use crate::model::semantic::{
    Attributes, Error, GeometrySchema, InfluenceRows, Link, MaterialId, Topology,
    UniversalGeometry, ValidateLinks, Values,
};
use crate::model::semantic::{GeometryEdit, GeometryEditCapabilities, StreamSupport};

/// Static triangle geometry for a world model.
///
/// Coordinate and color streams retain their authored order. Native dialects
/// decide which stream counts can be encoded; geometry authoring itself does
/// not impose the limits of one particular file revision.
#[derive(Clone, Debug)]
pub struct WmoGeometry<'data> {
    positions: Values<'data, [f32; 3]>,
    normals: Values<'data, [f32; 3]>,
    coordinates: Vec<Values<'data, [f32; 2]>>,
    colors: Vec<Values<'data, [f32; 4]>>,
    strip_indices: Values<'data, u32>,
    tangent_vectors: Values<'data, [f32; 4]>,
    lightmap_coordinates: Values<'data, [f32; 2]>,
    triangle_controls: Values<'data, WmoTriangleControls>,
    topology: Topology<'data>,
}
impl<'data> WmoGeometry<'data> {
    /// Replaces positions while checking all geometry stream relationships.
    pub fn with_positions(mut self, positions: Values<'data, [f32; 3]>) -> Result<Self, Error> {
        self.positions = positions;
        self.validate()?;
        Ok(self)
    }
    /// Creates checked triangle geometry while retaining borrowed arrays.
    pub fn triangles(
        positions: impl Into<Values<'data, [f32; 3]>>,
        triangles: impl Into<Values<'data, [u32; 3]>>,
    ) -> Result<Self, Error> {
        let triangles = triangles.into();
        let triangle_controls = Values::view(triangles.len(), |_| WmoTriangleControls::default());
        let geometry = Self {
            positions: positions.into(),
            normals: Values::default(),
            coordinates: Vec::new(),
            colors: Vec::new(),
            strip_indices: Values::default(),
            tangent_vectors: Values::default(),
            lightmap_coordinates: Values::default(),
            triangle_controls,
            topology: Topology::Triangles(triangles),
        };
        geometry.validate()?;
        Ok(geometry)
    }
    /// Sets per-triangle rendering, collision, and query behavior.
    pub fn with_triangle_controls(
        mut self,
        controls: Values<'data, WmoTriangleControls>,
    ) -> Result<Self, Error> {
        self.triangle_controls = controls;
        self.validate()?;
        Ok(self)
    }
    /// Sets shared strip connectivity selected by individual draw windows.
    pub fn with_strip_indices(mut self, indices: Values<'data, u32>) -> Result<Self, Error> {
        self.strip_indices = indices;
        self.validate()?;
        Ok(self)
    }
    /// Reads shared strip connectivity selected by individual draw windows.
    pub fn strip_indices(&self) -> &Values<'data, u32> {
        &self.strip_indices
    }
    /// Sets the shared tangent-vector pool selected by individual draws.
    pub fn with_tangent_vectors(mut self, values: Values<'data, [f32; 4]>) -> Result<Self, Error> {
        self.tangent_vectors = values;
        self.validate()?;
        Ok(self)
    }
    /// Reads the shared tangent-vector pool selected by individual draws.
    pub fn tangent_vectors(&self) -> &Values<'data, [f32; 4]> {
        &self.tangent_vectors
    }
    /// Sets the independently authored baked-light coordinate stream.
    pub fn with_lightmap_coordinates(
        mut self,
        coordinates: Values<'data, [f32; 2]>,
    ) -> Result<Self, Error> {
        self.lightmap_coordinates = coordinates;
        self.validate()?;
        Ok(self)
    }
    /// Reads the independently authored baked-light coordinate stream.
    pub fn lightmap_coordinates(&self) -> &Values<'data, [f32; 2]> {
        &self.lightmap_coordinates
    }
    /// Reads per-triangle controls in connectivity order.
    pub fn triangle_controls(&self) -> &Values<'data, WmoTriangleControls> {
        &self.triangle_controls
    }
    /// Sets authored vertex normals.
    pub fn with_normals(
        mut self,
        normals: impl Into<Values<'data, [f32; 3]>>,
    ) -> Result<Self, Error> {
        self.normals = normals.into();
        self.validate()?;
        Ok(self)
    }
    /// Sets ordered texture-coordinate streams.
    pub fn with_texture_coordinates(
        mut self,
        coordinates: Vec<Values<'data, [f32; 2]>>,
    ) -> Result<Self, Error> {
        self.coordinates = coordinates;
        self.validate()?;
        Ok(self)
    }
    /// Sets ordered vertex-color streams, including independent lighting sets.
    pub fn with_color_sets(mut self, colors: Vec<Values<'data, [f32; 4]>>) -> Result<Self, Error> {
        self.colors = colors;
        self.validate()?;
        Ok(self)
    }
    /// Reads all independently authored vertex-color sets.
    pub fn color_sets(&self) -> &[Values<'data, [f32; 4]>] {
        &self.colors
    }
}
impl<'data> GeometrySchema<'data> for WmoGeometry<'data> {
    fn edit_capabilities(&self) -> GeometryEditCapabilities {
        GeometryEditCapabilities {
            positions: true,
            normals: StreamSupport::Optional,
            tangents: StreamSupport::Unsupported,
            colors: StreamSupport::Optional,
            minimum_uv_sets: 0,
            maximum_uv_sets: None,
            vertex_remap: self.strip_indices.is_empty(),
        }
    }
    fn edited(&self, edit: GeometryEdit<'data>) -> Result<Self, Error> {
        self.validate()?;
        check_edit(&edit, self.edit_capabilities())?;
        let mut result = self.clone();
        if let Some(map) = &edit.remap {
            map.check(self)?;
            remap_values(&mut result.positions, &map.new_to_old);
            remap_values(&mut result.normals, &map.new_to_old);
            for stream in &mut result.coordinates {
                remap_values(stream, &map.new_to_old);
            }
            for stream in &mut result.colors {
                remap_values(stream, &map.new_to_old);
            }
            remap_values(&mut result.lightmap_coordinates, &map.new_to_old);
            result.topology = map.topology.clone();
        }
        if let Some(values) = edit.positions {
            result.positions = values;
        }
        if let Some(values) = edit.normals {
            result.normals = values;
        }
        if let Some(values) = edit.texture_coordinates {
            result.coordinates = values;
        }
        if let Some(values) = edit.colors {
            if values.is_empty() {
                if !result.colors.is_empty() {
                    result.colors.remove(0);
                }
            } else if result.colors.is_empty() {
                result.colors.push(values);
            } else {
                result.colors[0] = values;
            }
        }
        result.validate()?;
        Ok(result)
    }

    fn positions(&self) -> &Values<'data, [f32; 3]> {
        &self.positions
    }
    fn normals(&self) -> Option<&Values<'data, [f32; 3]>> {
        (!self.normals.is_empty()).then_some(&self.normals)
    }
    fn tangents(&self) -> Option<&Values<'data, [f32; 4]>> {
        None
    }
    fn texture_coordinates(&self) -> &[Values<'data, [f32; 2]>] {
        &self.coordinates
    }
    fn colors(&self) -> Option<&Values<'data, [f32; 4]>> {
        self.colors.first()
    }
    fn attributes(&self) -> Attributes<'_, 'data> {
        Attributes::Slice(&[])
    }
    fn topology(&self) -> &Topology<'data> {
        &self.topology
    }
    fn influences(&self) -> InfluenceRows<'_, 'data> {
        InfluenceRows::None
    }
    fn validate(&self) -> Result<(), Error> {
        validate_geometry(self)?;
        if self
            .strip_indices
            .iter()
            .any(|index| index as usize >= self.positions.len())
            || !self.tangent_vectors.iter().flatten().all(f32::is_finite)
        {
            return Err(Error::new(
                "geometry.draw_streams",
                "strip indices must address vertices and tangent vectors must be finite",
            ));
        }
        if !self.triangle_controls.is_empty()
            && self.triangle_controls.len() != self.topology.primitive_count()
        {
            return Err(Error::new(
                "geometry.triangle_controls",
                "controls must match triangle count",
            ));
        }
        if !self.lightmap_coordinates.is_empty()
            && (self.lightmap_coordinates.len() != self.positions.len()
                || !self
                    .lightmap_coordinates
                    .iter()
                    .flatten()
                    .all(f32::is_finite))
        {
            return Err(Error::new(
                "geometry.lightmap_coordinates",
                "baked-light coordinates must contain one finite coordinate per vertex",
            ));
        }
        for colors in &self.colors {
            if colors.len() != self.positions.len() || !colors.iter().flatten().all(f32::is_finite)
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
impl<'data> From<WmoGeometry<'data>> for UniversalGeometry<'data> {
    fn from(value: WmoGeometry<'data>) -> Self {
        Self::Wmo(value)
    }
}

impl Default for WmoGeometry<'_> {
    fn default() -> Self {
        Self {
            positions: Values::default(),
            normals: Values::default(),
            coordinates: Vec::new(),
            colors: Vec::new(),
            strip_indices: Values::default(),
            tangent_vectors: Values::default(),
            lightmap_coordinates: Values::default(),
            triangle_controls: Values::default(),
            topology: Topology::Triangles(Values::default()),
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
/// Authored world-triangle rendering and query behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WmoTriangleControls {
    /// Include this face in the static spatial-query structure.
    pub spatial_queries: bool,
    /// Surface material; absent for collision-only faces.
    pub material: Option<MaterialId>,
    /// Unresolved ground-type selector when the extended triangle marks a ground entry.
    pub ground_type_selector: Option<u16>,
    /// Optional authored terrain query classification.
    pub ground_type: Option<u32>,
    /// Historical baked-light selector whose table interpretation remains unresolved.
    pub lightmap_selector: Option<u8>,
    /// Historical alignment control whose meaning remains unresolved.
    pub alignment: u8,
    /// Interior/exterior lighting transition face.
    pub transition: bool,
    /// Disable camera collisions.
    pub no_camera_collision: bool,
    /// Detail geometry classification.
    pub detail: bool,
    /// Include the triangle in collision queries.
    pub collision: bool,
    /// Geometry hint classification.
    pub hint: bool,
    /// Include the triangle in rendering queries.
    pub render: bool,
    /// Cull nearby objects against this triangle.
    pub cull_objects: bool,
    /// Participate in hit queries.
    pub collide_hit: bool,
    /// Behavior bits with unresolved meanings.
    pub unclassified: TriangleFlags,
}
