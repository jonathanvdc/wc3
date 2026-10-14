//! Diablo III vertex meanings retain their independent color and tangent streams.
use crate::model::semantic::geometry_schema::validate_geometry;
use crate::model::semantic::{
    Attributes, Error, GeometrySchema, Influence, InfluenceRows, Link, Topology, ValidateLinks,
    Values,
};

/// Complete logical streams carried by an uncompressed Diablo III appearance vertex.
#[derive(Clone, Debug, Default)]
pub struct VertexStreams<'a> {
    /// Model-space vertex positions.
    pub positions: Values<'a, [f32; 3]>,
    /// Authored normals, without implicit renormalization.
    pub normals: Values<'a, [f32; 3]>,
    /// Primary RGBA colors.
    pub colors: Values<'a, [f32; 4]>,
    /// Independently authored auxiliary RGBA colors.
    pub auxiliary_colors: Values<'a, [f32; 4]>,
    /// Both authored texture-coordinate streams.
    pub texture_coordinates: [Values<'a, [f32; 2]>; 2],
    /// Tangent vectors, independent of binormal vectors.
    pub tangents: Values<'a, [f32; 3]>,
    /// Binormal vectors; no inferred handedness replaces this stream.
    pub binormals: Values<'a, [f32; 3]>,
    /// Authored mapping to the subobject's cloth vertices.
    pub cloth_vertices: Values<'a, u32>,
}
/// Diablo III triangle geometry with zero or three ordered skin contributions per vertex.
#[derive(Clone, Debug)]
pub struct Geometry<'a> {
    streams: VertexStreams<'a>,
    topology: Topology<'a>,
    influences: Values<'a, Vec<Influence>>,
}
impl<'a> Geometry<'a> {
    /// Checks stream cardinalities, triangle indices, and ordered influence rows.
    pub fn new(
        streams: VertexStreams<'a>,
        triangles: impl Into<Values<'a, [u32; 3]>>,
        influences: impl Into<Values<'a, Vec<Influence>>>,
    ) -> Result<Self, Error> {
        let value = Self {
            streams,
            topology: Topology::Triangles(triangles.into()),
            influences: influences.into(),
        };
        value.validate()?;
        Ok(value)
    }
    /// Inspects all independent logical vertex streams without decoding a second copy.
    pub fn streams(&self) -> &VertexStreams<'a> {
        &self.streams
    }
    /// Replaces complete vertex streams and checks their cardinalities.
    pub fn with_streams(mut self, streams: VertexStreams<'a>) -> Result<Self, Error> {
        self.streams = streams;
        self.validate()?;
        Ok(self)
    }
    /// Reads authored influence slots, preserving zero-weight contributions and their order.
    pub fn influence_rows(&self) -> &Values<'a, Vec<Influence>> {
        &self.influences
    }
}
impl Default for Geometry<'_> {
    fn default() -> Self {
        Self {
            streams: VertexStreams::default(),
            topology: Topology::Triangles(Values::default()),
            influences: Values::default(),
        }
    }
}
impl ValidateLinks for Geometry<'_> {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
impl<'a> GeometrySchema<'a> for Geometry<'a> {
    fn positions(&self) -> &Values<'a, [f32; 3]> {
        &self.streams.positions
    }
    fn normals(&self) -> Option<&Values<'a, [f32; 3]>> {
        Some(&self.streams.normals)
    }
    fn tangents(&self) -> Option<&Values<'a, [f32; 4]>> {
        None
    }
    fn texture_coordinates(&self) -> &[Values<'a, [f32; 2]>] {
        &self.streams.texture_coordinates
    }
    fn colors(&self) -> Option<&Values<'a, [f32; 4]>> {
        Some(&self.streams.colors)
    }
    fn attributes(&self) -> Attributes<'_, 'a> {
        Attributes::Slice(&[])
    }
    fn topology(&self) -> &Topology<'a> {
        &self.topology
    }
    fn influences(&self) -> InfluenceRows<'_, 'a> {
        if self.influences.is_empty() {
            InfluenceRows::None
        } else {
            InfluenceRows::Variable(&self.influences)
        }
    }
    fn validate(&self) -> Result<(), Error> {
        let count = self.streams.positions.len();
        let streams = &self.streams;
        if [
            streams.normals.len(),
            streams.colors.len(),
            streams.auxiliary_colors.len(),
            streams.texture_coordinates[0].len(),
            streams.texture_coordinates[1].len(),
            streams.tangents.len(),
            streams.binormals.len(),
            streams.cloth_vertices.len(),
        ]
        .iter()
        .any(|len| *len != count)
            || (!self.influences.is_empty()
                && (self.influences.len() != count
                    || self.influences.iter().any(|row| row.len() != 3)))
        {
            return Err(Error::new(
                "d3.geometry.streams",
                "expected complete vertex streams and zero or three influence slots per vertex",
            ));
        }
        if streams
            .auxiliary_colors
            .iter()
            .flatten()
            .chain(streams.tangents.iter().flatten())
            .chain(streams.binormals.iter().flatten())
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
