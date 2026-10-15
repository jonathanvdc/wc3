//! Shared vertex storage with constrained profile owners and a profile-independent view.
use super::geometry_edit::{check_edit, remap_values};
use super::geometry_schema::{present, validate_geometry};
use super::{
    Attributes, Error, GeometryData, GeometryEdit, GeometryEditCapabilities, GeometrySchema,
    Influence, InfluenceRows, Link, StreamSupport, Topology, ValidateLinks, Values,
};

/// Joint distributions preserve their authored slot structure and blending interpretation.
#[derive(Clone, Debug, Default)]
pub enum JointInfluences<'data> {
    /// No authored skinning distribution.
    #[default]
    None,
    /// Equal blending among the ordered joints of each matrix group.
    MatrixGroups(Values<'data, Vec<u32>>),
    /// Four ordered weighted joint slots, including zero-weight slots.
    Four(Values<'data, [Influence; 4]>),
}
impl<'data> JointInfluences<'data> {
    pub(crate) fn rows(&self) -> InfluenceRows<'_, 'data> {
        match self {
            Self::None => InfluenceRows::None,
            Self::MatrixGroups(value) => InfluenceRows::MatrixGroups(value),
            Self::Four(value) => InfluenceRows::Four(value),
        }
    }
    fn remap(&mut self, indices: &[u32]) {
        match self {
            Self::None => {}
            Self::MatrixGroups(value) => remap_values(value, indices),
            Self::Four(value) => remap_values(value, indices),
        }
    }
}

// Profile owners choose concrete field types: absent streams use (), M2 UVs use
// an array, and weighted-only profiles store four-slot rows without an enum.
#[derive(Clone, Debug)]
pub(crate) struct VertexStreams<
    'data,
    I,
    U = Vec<Values<'data, [f32; 2]>>,
    T = Values<'data, [f32; 4]>,
    C = Values<'data, [f32; 4]>,
> {
    pub positions: Values<'data, [f32; 3]>,
    pub normals: Values<'data, [f32; 3]>,
    pub tangents: T,
    pub colors: C,
    pub texture_coordinates: U,
    pub topology: Topology<'data>,
    pub influences: I,
}

/// Vertex streams without source-format identity. Profile owners retain their
/// constrained constructors; checked conversion back never synthesizes streams.
#[derive(Clone, Debug)]
pub struct VertexGeometry<'data> {
    pub(crate) streams: VertexStreams<'data, JointInfluences<'data>>,
}
impl<'data> VertexGeometry<'data> {
    /// Creates unskinned triangle geometry with optional shading streams.
    pub fn triangles(
        positions: impl Into<Values<'data, [f32; 3]>>,
        triangles: impl Into<Values<'data, [u32; 3]>>,
    ) -> Result<Self, Error> {
        let result = Self {
            streams: VertexStreams {
                positions: positions.into(),
                topology: Topology::Triangles(triangles.into()),
                ..Self::default().streams
            },
        };
        result.validate()?;
        Ok(result)
    }
    /// Replaces joint distributions without changing their interpretation.
    pub fn with_influences(mut self, influences: JointInfluences<'data>) -> Result<Self, Error> {
        self.streams.influences = influences;
        self.validate()?;
        Ok(self)
    }
    /// Reads the exact authored distribution representation.
    pub fn vertex_influences(&self) -> &JointInfluences<'data> {
        &self.streams.influences
    }
}
impl Default for VertexGeometry<'_> {
    fn default() -> Self {
        Self {
            streams: VertexStreams {
                positions: Values::default(),
                normals: Values::default(),
                tangents: Values::default(),
                colors: Values::default(),
                texture_coordinates: Vec::new(),
                topology: Topology::Triangles(Values::default()),
                influences: JointInfluences::None,
            },
        }
    }
}
impl ValidateLinks for VertexGeometry<'_> {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
impl<'data> GeometrySchema<'data> for VertexGeometry<'data> {
    fn edit_capabilities(&self) -> GeometryEditCapabilities {
        GeometryEditCapabilities {
            positions: true,
            normals: StreamSupport::Optional,
            tangents: StreamSupport::Optional,
            colors: StreamSupport::Optional,
            minimum_uv_sets: 0,
            maximum_uv_sets: None,
            vertex_remap: true,
        }
    }
    fn edited(&self, edit: GeometryEdit<'data>) -> Result<Self, Error> {
        self.validate()?;
        check_edit(&edit, self.edit_capabilities())?;
        let mut result = self.clone();
        let streams = &mut result.streams;
        if let Some(map) = &edit.remap {
            map.check(self)?;
            remap_values(&mut streams.positions, &map.new_to_old);
            remap_values(&mut streams.normals, &map.new_to_old);
            remap_values(&mut streams.tangents, &map.new_to_old);
            remap_values(&mut streams.colors, &map.new_to_old);
            for stream in &mut streams.texture_coordinates {
                remap_values(stream, &map.new_to_old);
            }
            streams.influences.remap(&map.new_to_old);
            streams.topology = map.topology.clone();
        }
        if let Some(values) = edit.positions {
            streams.positions = values;
        }
        if let Some(values) = edit.normals {
            streams.normals = values;
        }
        if let Some(values) = edit.tangents {
            streams.tangents = values;
        }
        if let Some(values) = edit.colors {
            streams.colors = values;
        }
        if let Some(values) = edit.texture_coordinates {
            streams.texture_coordinates = values;
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
    fn colors(&self) -> Option<&Values<'data, [f32; 4]>> {
        present(&self.streams.colors)
    }
    fn texture_coordinates(&self) -> &[Values<'data, [f32; 2]>] {
        &self.streams.texture_coordinates
    }
    fn attributes(&self) -> Attributes<'_, 'data> {
        Attributes::Slice(&[])
    }
    fn topology(&self) -> &Topology<'data> {
        &self.streams.topology
    }
    fn influences(&self) -> InfluenceRows<'_, 'data> {
        self.streams.influences.rows()
    }
    fn validate(&self) -> Result<(), Error> {
        validate_geometry(self)
    }
}
impl<'data> TryFrom<GeometryData<'data>> for VertexGeometry<'data> {
    type Error = Error;
    fn try_from(value: GeometryData<'data>) -> Result<Self, Error> {
        value.validate()?;
        if !value.attributes.is_empty() || !value.influences.is_empty() {
            return Err(Error::new(
                "geometry.streams",
                "custom attributes and variable joint distributions require general geometry",
            ));
        }
        let result = Self {
            streams: VertexStreams {
                positions: value.positions,
                normals: value.normals,
                tangents: value.tangents,
                colors: value.colors,
                texture_coordinates: value.texture_coordinates,
                topology: value.topology,
                influences: JointInfluences::None,
            },
        };
        result.validate()?;
        Ok(result)
    }
}
