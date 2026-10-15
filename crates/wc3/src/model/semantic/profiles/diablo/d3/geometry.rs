//! Diablo III vertex meanings retain their independent color and tangent streams.
use crate::model::semantic::geometry_edit::{check_edit, remap_values};
use crate::model::semantic::geometry_schema::validate_geometry;
use crate::model::semantic::vertex_geometry::{GeometryStreams, RemapStreams};
use crate::model::semantic::TextureCoordinates;
use crate::model::semantic::VertexData;
use crate::model::semantic::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, Attributes, Error,
    GeometryData, GeometryInfluences, GeometryKind, GeometryParts, GeometrySchema, Influence,
    InfluenceRows, Link, Topology, ValidateLinks, Values,
};
use crate::model::semantic::{GeometryEdit, GeometryEditCapabilities, StreamSupport};

/// Named vertex attribute used when widening the D3 auxiliary direction bytes.
/// Its values are `AttributeValues::Byte3`, ordered normal, tangent, binormal.
pub const DIRECTION_AUXILIARY_ATTRIBUTE: &str = "d3.direction_auxiliary";

/// Independent auxiliary streams authored alongside Diablo III's common vertices.
#[derive(Clone, Debug, Default)]
pub struct VertexExtras<'a> {
    /// Fourth packed bytes of the normal, tangent, and binormal, in that order.
    /// Their meaning is unresolved. An absent stream exports zero bytes.
    pub direction_auxiliary: Values<'a, [u8; 3]>,
    /// Independently authored auxiliary RGBA colors.
    pub auxiliary_colors: Values<'a, [f32; 4]>,
    /// Binormal vectors; no inferred handedness replaces this stream.
    pub binormals: Values<'a, [f32; 3]>,
    /// Authored mapping to the subobject's cloth vertices.
    pub cloth_vertices: Values<'a, u32>,
}
impl RemapStreams for VertexExtras<'_> {
    fn remap_streams(&mut self, indices: &[u32]) {
        self.direction_auxiliary.remap_streams(indices);
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
    fn texture_coordinates(&self) -> TextureCoordinates<'_, 'a> {
        TextureCoordinates::Slice(&self.data.vertices.texture_coordinates)
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
            || (!streams.extras.direction_auxiliary.is_empty()
                && streams.extras.direction_auxiliary.len() != count)
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

impl<'a> From<Geometry<'a>> for GeometryData<'a> {
    fn from(value: Geometry<'a>) -> Self {
        let GeometryStreams {
            vertices,
            topology,
            influences,
        } = value.data;
        let mut attributes = Vec::new();
        if !vertices.extras.direction_auxiliary.is_empty() {
            attributes.push(Attribute {
                semantic: AttributeSemantic::Named(DIRECTION_AUXILIARY_ATTRIBUTE.into()),
                domain: AttributeDomain::Vertex,
                values: AttributeValues::Byte3(vertices.extras.direction_auxiliary),
            });
        }
        attributes.extend([
            Attribute {
                semantic: AttributeSemantic::TangentVector,
                domain: AttributeDomain::Vertex,
                values: AttributeValues::Vector3(vertices.tangents),
            },
            Attribute {
                semantic: AttributeSemantic::Binormal,
                domain: AttributeDomain::Vertex,
                values: AttributeValues::Vector3(vertices.extras.binormals),
            },
            Attribute {
                semantic: AttributeSemantic::Color(1),
                domain: AttributeDomain::Vertex,
                values: AttributeValues::Vector4(vertices.extras.auxiliary_colors),
            },
            Attribute {
                semantic: AttributeSemantic::ClothVertex,
                domain: AttributeDomain::Vertex,
                values: AttributeValues::Unsigned(vertices.extras.cloth_vertices),
            },
        ]);
        GeometryParts {
            strip_set: false,
            surface_triangles: Values::default(),
            pools: Default::default(),
            kind: GeometryKind::Display,
            positions: vertices.positions,
            normals: vertices.normals,
            tangents: Values::default(),
            colors: vertices.colors,
            texture_coordinates: Vec::from(vertices.texture_coordinates),
            attributes,
            topology,
            influences: if influences.is_empty() {
                GeometryInfluences::None
            } else {
                GeometryInfluences::Three(influences)
            },
        }
        .into_data()
        .expect("constrained geometry has unique typed attributes")
    }
}

impl<'a> TryFrom<GeometryData<'a>> for Geometry<'a> {
    type Error = Error;
    fn try_from(value: GeometryData<'a>) -> Result<Self, Error> {
        value.validate()?;
        value.validate_no_additional_domains()?;
        let value = value.into_parts();
        if value.kind != GeometryKind::Display {
            return Err(Error::new("d3.geometry.kind", "expected display geometry"));
        }
        if !value.tangents.is_empty() {
            return Err(Error::new(
                "d3.geometry.tangents",
                "independent tangent vectors cannot retain a tangent frame",
            ));
        }
        let Topology::Triangles(triangles) = value.topology else {
            return Err(Error::new(
                "d3.geometry.topology",
                "expected triangle connectivity",
            ));
        };
        let influences = match value.influences {
            GeometryInfluences::None => Values::default(),
            GeometryInfluences::Three(rows) if !rows.is_empty() => rows,
            _ => {
                return Err(Error::new(
                    "d3.geometry.influences",
                    "expected absent or three-slot contributions",
                ))
            }
        };
        let mut direction_auxiliary = None;
        let mut tangents = None;
        let mut binormals = None;
        let mut auxiliary_colors = None;
        let mut cloth_vertices = None;
        for attribute in value.attributes {
            if attribute.domain != AttributeDomain::Vertex {
                return Err(Error::new(
                    "d3.geometry.attributes",
                    "expected vertex-domain streams",
                ));
            }
            match (attribute.semantic, attribute.values) {
                (AttributeSemantic::Named(name), AttributeValues::Byte3(value))
                    if name == DIRECTION_AUXILIARY_ATTRIBUTE && direction_auxiliary.is_none() =>
                {
                    direction_auxiliary = Some(value)
                }
                (AttributeSemantic::TangentVector, AttributeValues::Vector3(value))
                    if tangents.is_none() =>
                {
                    tangents = Some(value)
                }
                (AttributeSemantic::Binormal, AttributeValues::Vector3(value))
                    if binormals.is_none() =>
                {
                    binormals = Some(value)
                }
                (AttributeSemantic::Color(1), AttributeValues::Vector4(value))
                    if auxiliary_colors.is_none() =>
                {
                    auxiliary_colors = Some(value)
                }
                (AttributeSemantic::ClothVertex, AttributeValues::Unsigned(value))
                    if cloth_vertices.is_none() =>
                {
                    cloth_vertices = Some(value)
                }
                _ => {
                    return Err(Error::new(
                        "d3.geometry.attributes",
                        "unsupported, duplicate, or incorrectly typed attribute",
                    ))
                }
            }
        }
        let missing = || {
            Error::new(
                "d3.geometry.attributes",
                "missing independent tangent, binormal, auxiliary color, or cloth mapping",
            )
        };
        Self::new(
            VertexStreams {
                positions: value.positions,
                normals: value.normals,
                colors: value.colors,
                texture_coordinates: value
                    .texture_coordinates
                    .try_into()
                    .map_err(|_| Error::new("d3.geometry.uv", "expected exactly two UV streams"))?,
                tangents: tangents.ok_or_else(missing)?,
                extras: VertexExtras {
                    direction_auxiliary: direction_auxiliary.unwrap_or_default(),
                    binormals: binormals.ok_or_else(missing)?,
                    auxiliary_colors: auxiliary_colors.ok_or_else(missing)?,
                    cloth_vertices: cloth_vertices.ok_or_else(missing)?,
                },
            },
            triangles,
            influences,
        )
    }
}
