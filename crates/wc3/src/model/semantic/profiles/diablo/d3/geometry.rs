//! Diablo III vertex meanings retain their independent color and tangent streams.
use crate::model::semantic::geometry_edit::{check_edit, remap_values};
use crate::model::semantic::geometry_schema::validate_geometry;
use crate::model::semantic::vertex_geometry::{GeometryStreams, RemapStreams};
use crate::model::semantic::VertexData;
use crate::model::semantic::{
    Attributes, Error, GeometrySchema, Influence, InfluenceRows, Link, Topology, ValidateLinks,
    Values,
};
use crate::model::semantic::{GeometryEdit, GeometryEditCapabilities, StreamSupport};

/// Independent auxiliary streams authored alongside Diablo III's common vertices.
#[derive(Clone, Debug, Default)]
pub struct VertexExtras<'a> {
    /// Independently authored auxiliary RGBA colors.
    pub auxiliary_colors: Values<'a, [f32; 4]>,
    /// Binormal vectors; no inferred handedness replaces this stream.
    pub binormals: Values<'a, [f32; 3]>,
    /// Authored mapping to the subobject's cloth vertices.
    pub cloth_vertices: Values<'a, u32>,
}
impl RemapStreams for VertexExtras<'_> {
    fn remap_streams(&mut self, indices: &[u32]) {
        self.auxiliary_colors.remap_streams(indices);
        self.binormals.remap_streams(indices);
        self.cloth_vertices.remap_streams(indices);
    }
}
/// Complete logical streams carried by an uncompressed Diablo III appearance vertex.
/// Tangents are three-vectors independent of the binormals in `extras`.
///
/// ```compile_fail
/// use wc3::model::semantic::profiles::diablo::d3::VertexStreams;
/// let mut streams = VertexStreams::default();
/// streams.tangents = vec![[1., 0., 0., 1.]].into();
/// ```
pub type VertexStreams<'a> = VertexData<
    'a,
    [Values<'a, [f32; 2]>; 2],
    Values<'a, [f32; 3]>,
    Values<'a, [f32; 4]>,
    Values<'a, [f32; 3]>,
    VertexExtras<'a>,
>;
/// Diablo III triangle geometry with zero or three ordered skin contributions per vertex.
///
/// ```compile_fail
/// use wc3::model::semantic::profiles::diablo::d3::{Geometry, VertexStreams};
/// use wc3::model::semantic::Influence;
/// let _ = Geometry::new(VertexStreams::default(), Vec::<[u32; 3]>::new(),
///     vec![[Influence { joint: 0, weight: 0. }; 4]]);
/// ```
#[derive(Clone, Debug)]
pub struct Geometry<'a> {
    data: GeometryStreams<'a, VertexStreams<'a>, Values<'a, [Influence; 3]>>,
}
impl<'a> Geometry<'a> {
    /// Checks stream cardinalities, triangle indices, and ordered influence rows.
    pub fn new(
        streams: VertexStreams<'a>,
        triangles: impl Into<Values<'a, [u32; 3]>>,
        influences: impl Into<Values<'a, [Influence; 3]>>,
    ) -> Result<Self, Error> {
        let value = Self {
            data: GeometryStreams {
                vertices: streams,
                topology: Topology::Triangles(triangles.into()),
                influences: influences.into(),
            },
        };
        value.validate()?;
        Ok(value)
    }
    /// Inspects all independent logical vertex streams without decoding a second copy.
    pub fn streams(&self) -> &VertexStreams<'a> {
        &self.data.vertices
    }
    /// Replaces complete vertex streams and checks their cardinalities.
    pub fn with_streams(mut self, streams: VertexStreams<'a>) -> Result<Self, Error> {
        self.data.vertices = streams;
        self.validate()?;
        Ok(self)
    }
    /// Reads authored influence slots, preserving zero-weight contributions and their order.
    pub fn influence_rows(&self) -> &Values<'a, [Influence; 3]> {
        &self.data.influences
    }
}
impl Default for Geometry<'_> {
    fn default() -> Self {
        Self {
            data: GeometryStreams::default(),
        }
    }
}
impl ValidateLinks for Geometry<'_> {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
impl<'a> GeometrySchema<'a> for Geometry<'a> {
    fn edit_capabilities(&self) -> GeometryEditCapabilities {
        GeometryEditCapabilities {
            positions: true,
            normals: StreamSupport::Required,
            tangents: StreamSupport::Unsupported,
            colors: StreamSupport::Required,
            minimum_uv_sets: 2,
            maximum_uv_sets: Some(2),
            vertex_remap: true,
        }
    }
    fn edited(&self, edit: GeometryEdit<'a>) -> Result<Self, Error> {
        self.validate()?;
        check_edit(&edit, self.edit_capabilities())?;
        let mut result = self.clone();
        if let Some(map) = &edit.remap {
            map.check(self)?;
            result.data.vertices.remap(&map.new_to_old);
            remap_values(&mut result.data.influences, &map.new_to_old);
            result.data.topology = map.topology.clone();
        }
        if let Some(values) = edit.positions {
            result.data.vertices.positions = values;
        }
        if let Some(values) = edit.normals {
            result.data.vertices.normals = values;
        }
        if let Some(values) = edit.colors {
            result.data.vertices.colors = values;
        }
        if let Some(values) = edit.texture_coordinates {
            result.data.vertices.texture_coordinates = values
                .try_into()
                .map_err(|_| Error::new("geometry.edit.uv", "expected two UV streams"))?;
        }

        result.validate()?;
        Ok(result)
    }

    fn positions(&self) -> &Values<'a, [f32; 3]> {
        &self.data.vertices.positions
    }
    fn normals(&self) -> Option<&Values<'a, [f32; 3]>> {
        Some(&self.data.vertices.normals)
    }
    fn tangents(&self) -> Option<&Values<'a, [f32; 4]>> {
        None
    }
    fn texture_coordinates(&self) -> &[Values<'a, [f32; 2]>] {
        &self.data.vertices.texture_coordinates
    }
    fn colors(&self) -> Option<&Values<'a, [f32; 4]>> {
        Some(&self.data.vertices.colors)
    }
    fn attributes(&self) -> Attributes<'_, 'a> {
        Attributes::Slice(&[])
    }
    fn topology(&self) -> &Topology<'a> {
        &self.data.topology
    }
    fn influences(&self) -> InfluenceRows<'_, 'a> {
        if self.data.influences.is_empty() {
            InfluenceRows::None
        } else {
            InfluenceRows::Three(&self.data.influences)
        }
    }
    fn validate(&self) -> Result<(), Error> {
        let count = self.data.vertices.positions.len();
        let streams = &self.data.vertices;
        if [
            streams.normals.len(),
            streams.colors.len(),
            streams.extras.auxiliary_colors.len(),
            streams.texture_coordinates[0].len(),
            streams.texture_coordinates[1].len(),
            streams.tangents.len(),
            streams.extras.binormals.len(),
            streams.extras.cloth_vertices.len(),
        ]
        .iter()
        .any(|len| *len != count)
            || (!self.data.influences.is_empty() && self.data.influences.len() != count)
        {
            return Err(Error::new(
                "d3.geometry.streams",
                "expected complete vertex streams and zero or three influence slots per vertex",
            ));
        }
        if streams
            .extras
            .auxiliary_colors
            .iter()
            .flatten()
            .chain(streams.tangents.iter().flatten())
            .chain(streams.extras.binormals.iter().flatten())
            .any(|v| !v.is_finite())
        {
            return Err(Error::new(
                "d3.geometry.streams",
                "nonfinite auxiliary color or tangent component",
            ));
        }
        validate_geometry(self)
    }
}
