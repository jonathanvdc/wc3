//! Geometry streams and connectivity available to this model family.
use crate::model::semantic::geometry_schema::{present, validate_geometry};
use crate::model::semantic::{
    Attributes, Error, GeometrySchema, Influence, InfluenceRows, Link, Topology, ValidateLinks,
    Values,
};
/// Warcraft III joint distributions preserve matrix-group or weighted semantics.
#[derive(Clone, Debug, Default)]
pub enum MdlxVertexInfluences<'data> {
    /// No authored skinning distribution.
    #[default]
    None,
    /// Equal blending among the ordered joints of each matrix group.
    MatrixGroups(Values<'data, Vec<u32>>),
    /// Four ordered weighted joint slots per vertex.
    Weighted(Values<'data, [Influence; 4]>),
}

/// Warcraft III geometry with its native stream vocabulary.
///
/// Triangle groups retain their logical identities. Primitive kinds whose
/// authoring meanings are unknown remain the responsibility of the native
/// codec. General polygons and arbitrary attributes belong to
/// `GeometryData`. Tangent and weighted-skin availability is a native
/// dialect decision rather than a restriction on this family vocabulary.
#[derive(Clone, Debug)]
pub struct MdlxGeometry<'data> {
    positions: Values<'data, [f32; 3]>,
    normals: Values<'data, [f32; 3]>,
    tangents: Values<'data, [f32; 4]>,
    texture_coordinates: Vec<Values<'data, [f32; 2]>>,
    topology: Topology<'data>,
    influences: MdlxVertexInfluences<'data>,
}
impl<'data> MdlxGeometry<'data> {
    /// Creates an empty geometry definition.
    pub fn empty() -> Self {
        Self::default()
    }
    /// Creates checked triangle geometry without materializing borrowed arrays.
    pub fn triangles(
        positions: impl Into<Values<'data, [f32; 3]>>,
        triangles: impl Into<Values<'data, [u32; 3]>>,
    ) -> Result<Self, Error> {
        let geometry = Self {
            positions: positions.into(),
            normals: Values::default(),
            tangents: Values::default(),
            texture_coordinates: Vec::new(),
            topology: Topology::Triangles(triangles.into()),
            influences: MdlxVertexInfluences::None,
        };
        geometry.validate()?;
        Ok(geometry)
    }
    /// Creates ordered triangle groups without flattening their borrowed indices.
    pub fn triangle_groups(
        positions: impl Into<Values<'data, [f32; 3]>>,
        groups: Vec<Values<'data, [u32; 3]>>,
    ) -> Result<Self, Error> {
        let geometry = Self {
            positions: positions.into(),
            topology: Topology::Mixed(groups.into_iter().map(Topology::Triangles).collect()),
            ..Self::default()
        };
        geometry.validate()?;
        Ok(geometry)
    }
    /// Replaces vertex positions while preserving the remaining authored streams.
    pub fn with_positions(
        mut self,
        positions: impl Into<Values<'data, [f32; 3]>>,
    ) -> Result<Self, Error> {
        self.positions = positions.into();
        self.validate()?;
        Ok(self)
    }
    /// Supplies a vertex-normal stream, preserving its borrowed storage.
    pub fn with_normals(
        mut self,
        normals: impl Into<Values<'data, [f32; 3]>>,
    ) -> Result<Self, Error> {
        self.normals = normals.into();
        self.validate()?;
        Ok(self)
    }
    /// Supplies tangents; the chosen native version must support their encoding.
    pub fn with_tangents(
        mut self,
        tangents: impl Into<Values<'data, [f32; 4]>>,
    ) -> Result<Self, Error> {
        self.tangents = tangents.into();
        self.validate()?;
        Ok(self)
    }
    /// Supplies ordered texture-coordinate sets.
    pub fn with_texture_coordinates(
        mut self,
        coordinates: Vec<Values<'data, [f32; 2]>>,
    ) -> Result<Self, Error> {
        self.texture_coordinates = coordinates;
        self.validate()?;
        Ok(self)
    }
    /// Supplies an authored matrix-group or four-slot joint distribution.
    pub fn with_influences(
        mut self,
        influences: MdlxVertexInfluences<'data>,
    ) -> Result<Self, Error> {
        self.influences = influences;
        self.validate()?;
        Ok(self)
    }
    /// Reads the authored skinning form without expanding matrix groups.
    pub fn vertex_influences(&self) -> &MdlxVertexInfluences<'data> {
        &self.influences
    }
}
impl<'data> GeometrySchema<'data> for MdlxGeometry<'data> {
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
        None
    }
    fn attributes(&self) -> Attributes<'_, 'data> {
        Attributes::Slice(&[])
    }
    fn topology(&self) -> &Topology<'data> {
        &self.topology
    }
    fn influences(&self) -> InfluenceRows<'_, 'data> {
        match &self.influences {
            MdlxVertexInfluences::None => InfluenceRows::None,
            MdlxVertexInfluences::MatrixGroups(values) => InfluenceRows::MatrixGroups(values),
            MdlxVertexInfluences::Weighted(values) => InfluenceRows::Four(values),
        }
    }
    fn validate(&self) -> Result<(), Error> {
        if self
            .texture_coordinates
            .iter()
            .any(|stream| stream.len() != self.positions.len())
        {
            return Err(Error::new(
                "geometry.texture_coordinates",
                "MDLX texture-coordinate sets must cover every vertex",
            ));
        }
        fn native_topology(topology: &Topology<'_>) -> bool {
            match topology {
                Topology::Triangles(_) => true,
                Topology::Mixed(groups) => groups.iter().all(native_topology),
                _ => false,
            }
        }
        if !native_topology(&self.topology) {
            return Err(Error::new(
                "geometry.topology",
                "Warcraft III geometry requires known triangle primitive groups",
            ));
        }
        validate_geometry(self)
    }
}

impl Default for MdlxGeometry<'_> {
    fn default() -> Self {
        Self {
            positions: Values::default(),
            normals: Values::default(),
            tangents: Values::default(),
            texture_coordinates: Vec::new(),
            topology: Topology::Triangles(Values::default()),
            influences: MdlxVertexInfluences::None,
        }
    }
}

impl ValidateLinks for MdlxGeometry<'_> {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
