//! Geometry streams and connectivity available to this model family.
use crate::model::common::IntoZeroVec;
use crate::model::geometry_edit::{check_edit, remap_values};
use crate::model::geometry_schema::{present, validate_geometry};
use crate::model::vertex_geometry::RemapStreams;
use crate::model::vertex_geometry::{GeometryStreams, VertexStreams};
use crate::model::TextureCoordinates;
use crate::model::{
    Attribute, Error, GeometrySchema, InfluenceRows, JointInfluences, Link, Topology,
    ValidateLinks, ZeroVec,
};
use crate::model::{GeometryData, GeometryKind, GeometryParts, VertexData};
use crate::model::{GeometryEdit, GeometryEditCapabilities, StreamSupport};
type MdlxStreams<'data> = VertexStreams<
    'data,
    JointInfluences<'data>,
    Vec<ZeroVec<'data, [f32; 2]>>,
    ZeroVec<'data, [f32; 4]>,
    (),
>;

/// Warcraft III geometry with its native stream vocabulary.
///
/// Triangle groups retain their logical identities. Primitive kinds whose
/// authoring meanings are unknown remain the responsibility of the native
/// codec. General polygons and arbitrary attributes belong to
/// `GeometryData`. Tangent and weighted-skin availability is a native
/// dialect decision rather than a restriction on this family vocabulary.
#[derive(Clone, Debug)]
pub struct MdlxGeometry<'data> {
    pub(crate) streams: MdlxStreams<'data>,
}
impl<'data> MdlxGeometry<'data> {
    /// Creates an empty geometry definition.
    pub fn empty() -> Self {
        Self::default()
    }
    /// Creates checked triangle geometry without materializing borrowed arrays.
    pub fn triangles(
        positions: impl IntoZeroVec<'data, [f32; 3]>,
        triangles: impl IntoZeroVec<'data, [u32; 3]>,
    ) -> Result<Self, Error> {
        let geometry = Self {
            streams: VertexStreams {
                vertices: VertexData {
                    colors: (),
                    positions: positions.into_zero_vec(),
                    normals: ZeroVec::default(),
                    tangents: ZeroVec::default(),
                    texture_coordinates: Vec::new(),
                    extras: (),
                },
                topology: Topology::Triangles(triangles.into_zero_vec()),
                influences: JointInfluences::None,
            },
        };
        geometry.validate()?;
        Ok(geometry)
    }
    /// Creates ordered triangle groups without flattening their borrowed indices.
    pub fn triangle_groups(
        positions: impl IntoZeroVec<'data, [f32; 3]>,
        groups: Vec<ZeroVec<'data, [u32; 3]>>,
    ) -> Result<Self, Error> {
        let geometry = Self {
            streams: VertexStreams {
                vertices: VertexData {
                    positions: positions.into_zero_vec(),
                    extras: (),
                    ..Self::default().streams.vertices
                },
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
        positions: impl IntoZeroVec<'data, [f32; 3]>,
    ) -> Result<Self, Error> {
        self.streams.positions = positions.into_zero_vec();
        self.validate()?;
        Ok(self)
    }
    /// Supplies a vertex-normal stream, preserving its borrowed storage.
    pub fn with_normals(
        mut self,
        normals: impl IntoZeroVec<'data, [f32; 3]>,
    ) -> Result<Self, Error> {
        self.streams.normals = normals.into_zero_vec();
        self.validate()?;
        Ok(self)
    }
    /// Supplies tangents; the chosen native version must support their encoding.
    pub fn with_tangents(
        mut self,
        tangents: impl IntoZeroVec<'data, [f32; 4]>,
    ) -> Result<Self, Error> {
        self.streams.tangents = tangents.into_zero_vec();
        self.validate()?;
        Ok(self)
    }
    /// Supplies ordered texture-coordinate sets.
    pub fn with_texture_coordinates(
        mut self,
        coordinates: Vec<ZeroVec<'data, [f32; 2]>>,
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
            result.streams.vertices.remap(&map.new_to_old);
            match &mut result.streams.influences {
                JointInfluences::None => {}
                JointInfluences::MatrixGroups(v) => v.remap_streams(&map.new_to_old),
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

    fn positions(&self) -> &ZeroVec<'data, [f32; 3]> {
        &self.streams.positions
    }
    fn normals(&self) -> Option<&ZeroVec<'data, [f32; 3]>> {
        present(&self.streams.normals)
    }
    fn tangents(&self) -> Option<&ZeroVec<'data, [f32; 4]>> {
        present(&self.streams.tangents)
    }
    fn texture_coordinates(&self) -> TextureCoordinates<'_, 'data> {
        TextureCoordinates::Slice(&self.streams.texture_coordinates)
    }
    fn colors(&self) -> Option<&ZeroVec<'data, [f32; 4]>> {
        None
    }
    fn attributes(&self) -> &[Attribute<'data>] {
        &[]
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
                vertices: VertexData {
                    colors: (),
                    positions: ZeroVec::default(),
                    normals: ZeroVec::default(),
                    tangents: ZeroVec::default(),
                    texture_coordinates: Vec::new(),
                    extras: (),
                },
                topology: Topology::Triangles(ZeroVec::default()),
                influences: JointInfluences::None,
            },
        }
    }
}

impl ValidateLinks for MdlxGeometry<'_> {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}

impl<'data> From<MdlxGeometry<'data>> for GeometryData<'data> {
    fn from(value: MdlxGeometry<'data>) -> Self {
        let GeometryStreams {
            vertices,
            topology,
            influences,
        } = value.streams;
        GeometryParts {
            strip_set: false,
            surface_triangles: ZeroVec::default(),
            pools: Default::default(),
            kind: GeometryKind::Display,
            positions: vertices.positions,
            normals: vertices.normals,
            tangents: vertices.tangents,
            colors: ZeroVec::default(),
            texture_coordinates: vertices.texture_coordinates,
            attributes: Vec::new(),
            topology,
            influences: influences.into(),
        }
        .into_data()
        .expect("constrained geometry has unique typed attributes")
    }
}
impl<'data> TryFrom<GeometryData<'data>> for MdlxGeometry<'data> {
    type Error = Error;
    fn try_from(value: GeometryData<'data>) -> Result<Self, Error> {
        value.validate_vertex_streams()?;
        let vertices = value.into_parts();
        let topology = vertices.topology;
        let influences = vertices.influences;
        if !vertices.colors.is_empty() {
            return Err(Error::new(
                "geometry.colors",
                "MDLX geometry has no vertex color stream",
            ));
        }
        let influences = influences.try_into()?;
        let result = Self {
            streams: VertexStreams {
                vertices: VertexData {
                    positions: vertices.positions,
                    normals: vertices.normals,
                    tangents: vertices.tangents,
                    colors: (),
                    texture_coordinates: vertices.texture_coordinates,
                    extras: (),
                },
                topology,
                influences,
            },
        };
        result.validate()?;
        Ok(result)
    }
}
