//! Geometry streams and connectivity available to this model family.
use crate::model::semantic::geometry_edit::{check_edit, remap_values};
use crate::model::semantic::geometry_schema::validate_geometry;
use crate::model::semantic::{
    Attributes, Error, GeometrySchema, Influence, InfluenceRows, Link, Topology, ValidateLinks,
    Values,
};
use crate::model::semantic::{GeometryEdit, GeometryEditCapabilities, StreamSupport};
/// World of Warcraft model geometry with two UV streams and four joint slots.
#[derive(Clone, Debug)]
pub struct M2Geometry<'data> {
    positions: Values<'data, [f32; 3]>,
    normals: Values<'data, [f32; 3]>,
    texture_coordinates: [Values<'data, [f32; 2]>; 2],
    topology: Topology<'data>,
    influences: Values<'data, [Influence; 4]>,
}
impl<'data> M2Geometry<'data> {
    /// Creates an empty geometry definition.
    pub fn empty() -> Self {
        Self::default()
    }
    /// Creates checked native-family geometry from borrowed or owned streams.
    pub fn new(
        positions: impl Into<Values<'data, [f32; 3]>>,
        normals: impl Into<Values<'data, [f32; 3]>>,
        texture_coordinates: [Values<'data, [f32; 2]>; 2],
        triangles: impl Into<Values<'data, [u32; 3]>>,
        influences: impl Into<Values<'data, [Influence; 4]>>,
    ) -> Result<Self, Error> {
        let geometry = Self {
            positions: positions.into(),
            normals: normals.into(),
            texture_coordinates,
            topology: Topology::Triangles(triangles.into()),
            influences: influences.into(),
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
            remap_values(&mut result.positions, &map.new_to_old);
            remap_values(&mut result.normals, &map.new_to_old);
            for stream in &mut result.texture_coordinates {
                remap_values(stream, &map.new_to_old);
            }
            remap_values(&mut result.influences, &map.new_to_old);
            result.topology = map.topology.clone();
        }
        if let Some(values) = edit.positions {
            result.positions = values;
        }
        if let Some(values) = edit.normals {
            result.normals = values;
        }
        if let Some(values) = edit.texture_coordinates {
            result.texture_coordinates = values
                .try_into()
                .map_err(|_| Error::new("geometry.edit.uv", "expected two UV streams"))?;
        }

        result.validate()?;
        Ok(result)
    }

    fn positions(&self) -> &Values<'data, [f32; 3]> {
        &self.positions
    }
    fn normals(&self) -> Option<&Values<'data, [f32; 3]>> {
        Some(&self.normals)
    }
    fn tangents(&self) -> Option<&Values<'data, [f32; 4]>> {
        None
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
        InfluenceRows::Four(&self.influences)
    }
    fn validate(&self) -> Result<(), Error> {
        if self
            .texture_coordinates
            .iter()
            .any(|stream| stream.len() != self.positions.len())
        {
            return Err(Error::new(
                "geometry.texture_coordinates",
                "M2 vertices require both texture-coordinate slots",
            ));
        }
        let count = self.positions.len();
        if self.normals.len() != count
            || self
                .texture_coordinates
                .iter()
                .any(|stream| stream.len() != count)
            || self.influences.len() != count
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
            positions: Values::default(),
            normals: Values::default(),
            texture_coordinates: [Values::default(), Values::default()],
            topology: Topology::Triangles(Values::default()),
            influences: Values::default(),
        }
    }
}

impl ValidateLinks for M2Geometry<'_> {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
