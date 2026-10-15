//! Geometry streams and connectivity available to this model family.
use crate::model::common::IntoZeroVec;
use crate::model::geometry_edit::{check_edit, remap_values};
use crate::model::geometry_schema::validate_geometry;
use crate::model::vertex_geometry::{GeometryStreams, VertexStreams};
use crate::model::TextureCoordinates;
use crate::model::{
    Attributes, Error, GeometrySchema, Influence, InfluenceRows, Link, Topology, ValidateLinks,
    ZeroVec,
};
use crate::model::{GeometryData, GeometryInfluences, GeometryKind, GeometryParts, VertexData};
use crate::model::{GeometryEdit, GeometryEditCapabilities, StreamSupport};
type M2Streams<'data> =
    VertexStreams<'data, ZeroVec<'data, [Influence; 4]>, [ZeroVec<'data, [f32; 2]>; 2], (), ()>;

/// World of Warcraft model geometry with two UV streams and four joint slots.
/// Unsupported streams are unavailable through this type's constructors.
///
/// ```compile_fail
/// use wc3::model::M2Geometry;
/// let geometry = M2Geometry::empty();
/// let _ = geometry.with_tangents(vec![[1., 0., 0., 1.]]);
/// ```
///
/// ```compile_fail
/// use wc3::model::{Influence, M2Geometry, ZeroVec};
/// let _ = M2Geometry::new(
///     Vec::<[f32; 3]>::new(), Vec::<[f32; 3]>::new(),
///     [ZeroVec::default()], Vec::<[u32; 3]>::new(), Vec::<[Influence; 4]>::new(),
/// );
/// ```
#[derive(Clone, Debug)]
pub struct M2Geometry<'data> {
    pub(crate) streams: M2Streams<'data>,
}
impl<'data> M2Geometry<'data> {
    /// Creates an empty geometry definition.
    pub fn empty() -> Self {
        Self::default()
    }
    /// Creates checked native-family geometry from borrowed or owned streams.
    pub fn new(
        positions: impl IntoZeroVec<'data, [f32; 3]>,
        normals: impl IntoZeroVec<'data, [f32; 3]>,
        texture_coordinates: [ZeroVec<'data, [f32; 2]>; 2],
        triangles: impl IntoZeroVec<'data, [u32; 3]>,
        influences: impl IntoZeroVec<'data, [Influence; 4]>,
    ) -> Result<Self, Error> {
        let geometry = Self {
            streams: VertexStreams {
                vertices: VertexData {
                    tangents: (),
                    colors: (),
                    positions: positions.into_zero_vec(),
                    normals: normals.into_zero_vec(),
                    texture_coordinates,
                    extras: (),
                },
                topology: Topology::Triangles(triangles.into_zero_vec()),
                influences: influences.into_zero_vec(),
            },
        };
        geometry.validate()?;
        Ok(geometry)
    }
}
impl<'data> GeometrySchema<'data> for M2Geometry<'data> {
    fn edit_capabilities(&self) -> GeometryEditCapabilities {
        GeometryEditCapabilities {
            positions: true,
            normals: StreamSupport::Required,
            tangents: StreamSupport::Unsupported,
            colors: StreamSupport::Unsupported,
            minimum_uv_sets: 2,
            maximum_uv_sets: Some(2),
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
            remap_values(&mut result.streams.influences, &map.new_to_old);
            result.streams.topology = map.topology.clone();
        }
        if let Some(values) = edit.positions {
            result.streams.positions = values;
        }
        if let Some(values) = edit.normals {
            result.streams.normals = values;
        }
        if let Some(values) = edit.texture_coordinates {
            result.streams.texture_coordinates = values
                .try_into()
                .map_err(|_| Error::new("geometry.edit.uv", "expected two UV streams"))?;
        }

        result.validate()?;
        Ok(result)
    }

    fn positions(&self) -> &ZeroVec<'data, [f32; 3]> {
        &self.streams.positions
    }
    fn normals(&self) -> Option<&ZeroVec<'data, [f32; 3]>> {
        Some(&self.streams.normals)
    }
    fn tangents(&self) -> Option<&ZeroVec<'data, [f32; 4]>> {
        None
    }
    fn texture_coordinates(&self) -> TextureCoordinates<'_, 'data> {
        TextureCoordinates::Slice(&self.streams.texture_coordinates)
    }
    fn colors(&self) -> Option<&ZeroVec<'data, [f32; 4]>> {
        None
    }
    fn attributes(&self) -> Attributes<'_, 'data> {
        Attributes::Slice(&[])
    }
    fn topology(&self) -> &Topology<'data> {
        &self.streams.topology
    }
    fn influences(&self) -> InfluenceRows<'_, 'data> {
        InfluenceRows::Four(&self.streams.influences)
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
                "M2 vertices require both texture-coordinate slots",
            ));
        }
        let count = self.streams.positions.len();
        if self.streams.normals.len() != count
            || self
                .streams
                .texture_coordinates
                .iter()
                .any(|stream| stream.len() != count)
            || self.streams.influences.len() != count
        {
            return Err(Error::new(
                "geometry.streams",
                "M2 vertices require normals, two UV streams, and four joint slots",
            ));
        }
        validate_geometry(self)
    }
}

impl Default for M2Geometry<'_> {
    fn default() -> Self {
        Self {
            streams: VertexStreams {
                vertices: VertexData {
                    tangents: (),
                    colors: (),
                    positions: ZeroVec::default(),
                    normals: ZeroVec::default(),
                    texture_coordinates: [ZeroVec::default(), ZeroVec::default()],
                    extras: (),
                },
                topology: Topology::Triangles(ZeroVec::default()),
                influences: ZeroVec::default(),
            },
        }
    }
}

impl ValidateLinks for M2Geometry<'_> {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}

impl<'data> From<M2Geometry<'data>> for GeometryData<'data> {
    fn from(value: M2Geometry<'data>) -> Self {
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
            tangents: ZeroVec::default(),
            colors: ZeroVec::default(),
            texture_coordinates: Vec::from(vertices.texture_coordinates),
            attributes: Vec::new(),
            topology,
            influences: GeometryInfluences::Four(influences),
        }
        .into_data()
        .expect("constrained geometry has unique typed attributes")
    }
}
impl<'data> TryFrom<GeometryData<'data>> for M2Geometry<'data> {
    type Error = Error;
    fn try_from(value: GeometryData<'data>) -> Result<Self, Error> {
        value.validate_vertex_streams()?;
        let vertices = value.into_parts();
        let topology = vertices.topology;
        let influences = vertices.influences;
        if !vertices.tangents.is_empty() || !vertices.colors.is_empty() {
            return Err(Error::new(
                "geometry.streams",
                "M2 geometry has no tangent or vertex color streams",
            ));
        }
        if !matches!(topology, Topology::Triangles(_)) {
            return Err(Error::new(
                "geometry.topology",
                "M2 geometry requires triangle connectivity",
            ));
        }
        let GeometryInfluences::Four(influences) = influences else {
            return Err(Error::new(
                "geometry.influences",
                "M2 geometry requires four authored joint slots",
            ));
        };
        let texture_coordinates = vertices.texture_coordinates.try_into().map_err(|_| {
            Error::new(
                "geometry.texture_coordinates",
                "M2 geometry requires exactly two UV streams",
            )
        })?;
        let result = Self {
            streams: VertexStreams {
                vertices: VertexData {
                    positions: vertices.positions,
                    normals: vertices.normals,
                    tangents: (),
                    colors: (),
                    texture_coordinates,
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
