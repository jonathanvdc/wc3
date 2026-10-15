//! Geometry streams and connectivity available to this model family.
use crate::model::semantic::geometry_edit::{check_edit, remap_values};
use crate::model::semantic::geometry_schema::{present, validate_geometry};
use crate::model::semantic::vertex_geometry::VertexStreams;
use crate::model::semantic::VertexGeometry;
use crate::model::semantic::{
    Attributes, Error, GeometrySchema, InfluenceRows, JointInfluences, Link, Topology,
    ValidateLinks, Values,
};
use crate::model::semantic::{GeometryEdit, GeometryEditCapabilities, StreamSupport};
/// Warcraft III geometry with its native stream vocabulary.
///
/// Triangle groups retain their logical identities. Primitive kinds whose
/// authoring meanings are unknown remain the responsibility of the native
/// codec. General polygons and arbitrary attributes belong to
/// `GeometryData`. Tangent and weighted-skin availability is a native
/// dialect decision rather than a restriction on this family vocabulary.
#[derive(Clone, Debug)]
pub struct MdlxGeometry<'data> {
    pub(crate) streams: VertexStreams<
        'data,
        JointInfluences<'data>,
        Vec<Values<'data, [f32; 2]>>,
        Values<'data, [f32; 4]>,
        (),
    >,
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
            streams: VertexStreams {
                colors: (),
                positions: positions.into(),
                normals: Values::default(),
                tangents: Values::default(),
                texture_coordinates: Vec::new(),
                topology: Topology::Triangles(triangles.into()),
                influences: JointInfluences::None,
            },
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
            streams: VertexStreams {
                positions: positions.into(),
                topology: Topology::Mixed(groups.into_iter().map(Topology::Triangles).collect()),
                ..Self::default().streams
            },
        };
        geometry.validate()?;
        Ok(geometry)
    }
    /// Replaces vertex positions while preserving the remaining authored streams.
    pub fn with_positions(
        mut self,
        positions: impl Into<Values<'data, [f32; 3]>>,
    ) -> Result<Self, Error> {
        self.streams.positions = positions.into();
        self.validate()?;
        Ok(self)
    }
    /// Supplies a vertex-normal stream, preserving its borrowed storage.
    pub fn with_normals(
        mut self,
        normals: impl Into<Values<'data, [f32; 3]>>,
    ) -> Result<Self, Error> {
        self.streams.normals = normals.into();
        self.validate()?;
        Ok(self)
    }
    /// Supplies tangents; the chosen native version must support their encoding.
    pub fn with_tangents(
        mut self,
        tangents: impl Into<Values<'data, [f32; 4]>>,
    ) -> Result<Self, Error> {
        self.streams.tangents = tangents.into();
        self.validate()?;
        Ok(self)
    }
    /// Supplies ordered texture-coordinate sets.
    pub fn with_texture_coordinates(
        mut self,
        coordinates: Vec<Values<'data, [f32; 2]>>,
    ) -> Result<Self, Error> {
        self.streams.texture_coordinates = coordinates;
        self.validate()?;
        Ok(self)
    }
    /// Supplies an authored matrix-group or four-slot joint distribution.
    pub fn with_influences(mut self, influences: JointInfluences<'data>) -> Result<Self, Error> {
        self.streams.influences = influences;
        self.validate()?;
        Ok(self)
    }
    /// Reads the authored skinning form without expanding matrix groups.
    pub fn vertex_influences(&self) -> &JointInfluences<'data> {
        &self.streams.influences
    }
}
impl<'data> GeometrySchema<'data> for MdlxGeometry<'data> {
    fn edit_capabilities(&self) -> GeometryEditCapabilities {
        GeometryEditCapabilities {
            positions: true,
            normals: StreamSupport::Optional,
            tangents: StreamSupport::Optional,
            colors: StreamSupport::Unsupported,
            minimum_uv_sets: 0,
            maximum_uv_sets: None,
            vertex_remap: true,
        }
    }
    fn edited(&self, edit: GeometryEdit<'data>) -> Result<Self, Error> {
        self.validate()?;
        check_edit(&edit, self.edit_capabilities())?;
        let mut result = self.clone();
        if let Some(map) = &edit.remap {
            map.check(self)?;
            remap_values(&mut result.streams.positions, &map.new_to_old);
            remap_values(&mut result.streams.normals, &map.new_to_old);
            remap_values(&mut result.streams.tangents, &map.new_to_old);
            for stream in &mut result.streams.texture_coordinates {
                remap_values(stream, &map.new_to_old);
            }
            match &mut result.streams.influences {
                JointInfluences::None => {}
                JointInfluences::MatrixGroups(v) => remap_values(v, &map.new_to_old),
                JointInfluences::Four(v) => remap_values(v, &map.new_to_old),
            }
            result.streams.topology = map.topology.clone();
        }
        if let Some(values) = edit.positions {
            result.streams.positions = values;
        }
        if let Some(values) = edit.normals {
            result.streams.normals = values;
        }
        if let Some(values) = edit.tangents {
            result.streams.tangents = values;
        }
        if let Some(values) = edit.texture_coordinates {
            result.streams.texture_coordinates = values;
        }

        result.validate()?;
        Ok(result)
    }

    fn positions(&self) -> &Values<'data, [f32; 3]> {
        &self.streams.positions
    }
    fn normals(&self) -> Option<&Values<'data, [f32; 3]>> {
        present(&self.streams.normals)
    }
    fn tangents(&self) -> Option<&Values<'data, [f32; 4]>> {
        present(&self.streams.tangents)
    }
    fn texture_coordinates(&self) -> &[Values<'data, [f32; 2]>] {
        &self.streams.texture_coordinates
    }
    fn colors(&self) -> Option<&Values<'data, [f32; 4]>> {
        None
    }
    fn attributes(&self) -> Attributes<'_, 'data> {
        Attributes::Slice(&[])
    }
    fn topology(&self) -> &Topology<'data> {
        &self.streams.topology
    }
    fn influences(&self) -> InfluenceRows<'_, 'data> {
        match &self.streams.influences {
            JointInfluences::None => InfluenceRows::None,
            JointInfluences::MatrixGroups(values) => InfluenceRows::MatrixGroups(values),
            JointInfluences::Four(values) => InfluenceRows::Four(values),
        }
    }
    fn validate(&self) -> Result<(), Error> {
        if self
            .streams
            .texture_coordinates
            .iter()
            .any(|stream| stream.len() != self.streams.positions.len())
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
        if !native_topology(&self.streams.topology) {
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
            streams: VertexStreams {
                colors: (),
                positions: Values::default(),
                normals: Values::default(),
                tangents: Values::default(),
                texture_coordinates: Vec::new(),
                topology: Topology::Triangles(Values::default()),
                influences: JointInfluences::None,
            },
        }
    }
}

impl ValidateLinks for MdlxGeometry<'_> {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}

impl<'data> From<MdlxGeometry<'data>> for VertexGeometry<'data> {
    fn from(value: MdlxGeometry<'data>) -> Self {
        let s = value.streams;
        Self {
            streams: VertexStreams {
                positions: s.positions,
                normals: s.normals,
                tangents: s.tangents,
                colors: Values::default(),
                texture_coordinates: s.texture_coordinates,
                topology: s.topology,
                influences: s.influences,
            },
        }
    }
}
impl<'data> TryFrom<VertexGeometry<'data>> for MdlxGeometry<'data> {
    type Error = Error;
    fn try_from(value: VertexGeometry<'data>) -> Result<Self, Error> {
        value.validate()?;
        let s = value.streams;
        if !s.colors.is_empty() {
            return Err(Error::new(
                "geometry.colors",
                "MDLX geometry has no vertex color stream",
            ));
        }
        let result = Self {
            streams: VertexStreams {
                positions: s.positions,
                normals: s.normals,
                tangents: s.tangents,
                colors: (),
                texture_coordinates: s.texture_coordinates,
                topology: s.topology,
                influences: s.influences,
            },
        };
        result.validate()?;
        Ok(result)
    }
}
