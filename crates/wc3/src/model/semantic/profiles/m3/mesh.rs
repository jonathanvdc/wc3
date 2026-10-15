//! M3 display and collision geometry remain distinct authored shapes.
//! Display vertices carry the streams needed for skinning and shading. Collision
//! meshes contain positions and connectivity, without invented shading data.
use crate::model::semantic::geometry_edit::{check_edit, remap_values};
use crate::model::semantic::geometry_schema::validate_geometry;
use crate::model::semantic::vertex_geometry::{GeometryStreams, VertexStreams};
use crate::model::semantic::{
    Attributes, Error, GeometryKind, GeometrySchema, Influence, InfluenceRows, JointInfluences,
    Link, Topology, ValidateLinks, Values,
};
use crate::model::semantic::{GeometryEdit, GeometryEditCapabilities, StreamSupport};
use crate::model::semantic::{VertexData, VertexGeometry};
fn present<'view, 'data, T>(values: &'view Values<'data, T>) -> Option<&'view Values<'data, T>> {
    (!values.is_empty()).then_some(values)
}
/// StarCraft II geometry with native vertex streams and four joint slots.
#[derive(Clone, Debug)]
pub struct M3DisplayGeometry<'data> {
    pub(crate) streams: VertexStreams<'data, Values<'data, [Influence; 4]>>,
}
impl<'data> M3DisplayGeometry<'data> {
    /// Creates an empty geometry definition.
    pub fn empty() -> Self {
        Self::default()
    }
    /// Creates checked triangle geometry with its required vertex-normal stream.
    pub fn new(
        positions: impl Into<Values<'data, [f32; 3]>>,
        normals: impl Into<Values<'data, [f32; 3]>>,
        triangles: impl Into<Values<'data, [u32; 3]>>,
        influences: impl Into<Values<'data, [Influence; 4]>>,
    ) -> Result<Self, Error> {
        let geometry = Self {
            streams: VertexStreams {
                vertices: VertexData {
                    positions: positions.into(),
                    normals: normals.into(),
                    tangents: Values::default(),
                    texture_coordinates: Vec::new(),
                    colors: Values::default(),
                    extras: (),
                },
                topology: Topology::Triangles(triangles.into()),
                influences: influences.into(),
            },
        };
        geometry.validate()?;
        Ok(geometry)
    }
    /// Supplies a native tangent stream.
    pub fn with_tangents(
        mut self,
        tangents: impl Into<Values<'data, [f32; 4]>>,
    ) -> Result<Self, Error> {
        self.streams.tangents = tangents.into();
        self.validate()?;
        Ok(self)
    }
    /// Supplies up to five native texture-coordinate streams.
    pub fn with_texture_coordinates(
        mut self,
        coordinates: Vec<Values<'data, [f32; 2]>>,
    ) -> Result<Self, Error> {
        self.streams.texture_coordinates = coordinates;
        self.validate()?;
        Ok(self)
    }
    /// Supplies native vertex colors.
    pub fn with_colors(
        mut self,
        colors: impl Into<Values<'data, [f32; 4]>>,
    ) -> Result<Self, Error> {
        self.streams.colors = colors.into();
        self.validate()?;
        Ok(self)
    }
}
impl<'data> GeometrySchema<'data> for M3DisplayGeometry<'data> {
    fn edit_capabilities(&self) -> GeometryEditCapabilities {
        GeometryEditCapabilities {
            positions: true,
            normals: StreamSupport::Required,
            tangents: StreamSupport::Optional,
            colors: StreamSupport::Optional,
            minimum_uv_sets: 0,
            maximum_uv_sets: Some(5),
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
        if let Some(values) = edit.tangents {
            result.streams.tangents = values;
        }
        if let Some(values) = edit.colors {
            result.streams.colors = values;
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
        Some(&self.streams.normals)
    }
    fn tangents(&self) -> Option<&Values<'data, [f32; 4]>> {
        present(&self.streams.tangents)
    }
    fn texture_coordinates(&self) -> &[Values<'data, [f32; 2]>] {
        &self.streams.texture_coordinates
    }
    fn colors(&self) -> Option<&Values<'data, [f32; 4]>> {
        present(&self.streams.colors)
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
        if self.streams.texture_coordinates.len() > 5 {
            return Err(Error::new(
                "geometry.texture_coordinates",
                "M3 vertices have at most five UV streams",
            ));
        }
        if self.streams.normals.len() != self.streams.positions.len()
            || self.streams.influences.len() != self.streams.positions.len()
        {
            return Err(Error::new(
                "geometry.streams",
                "M3 vertices require normals and four joint slots",
            ));
        }
        validate_geometry(self)
    }
}

impl Default for M3DisplayGeometry<'_> {
    fn default() -> Self {
        Self {
            streams: VertexStreams {
                vertices: VertexData {
                    positions: Values::default(),
                    normals: Values::default(),
                    tangents: Values::default(),
                    texture_coordinates: Vec::new(),
                    colors: Values::default(),
                    extras: (),
                },
                topology: Topology::Triangles(Values::default()),
                influences: Values::default(),
            },
        }
    }
}

impl ValidateLinks for M3DisplayGeometry<'_> {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
/// Collision triangles without shading or skinning streams.
#[derive(Clone, Debug)]
pub struct M3CollisionGeometry<'data> {
    streams: VertexStreams<'data, (), (), (), (), ()>,
    face_normals: Values<'data, [f32; 3]>,
    triangle_controls: Values<'data, f32>,
}
impl<'data> M3CollisionGeometry<'data> {
    /// Constructs checked triangle connectivity over borrowed or owned positions.
    pub fn new(
        positions: impl Into<Values<'data, [f32; 3]>>,
        triangles: impl Into<Values<'data, [u32; 3]>>,
    ) -> Result<Self, Error> {
        let result = Self {
            streams: VertexStreams {
                vertices: VertexData {
                    positions: positions.into(),
                    normals: (),
                    tangents: (),
                    colors: (),
                    texture_coordinates: (),
                    extras: (),
                },
                topology: Topology::Triangles(triangles.into()),
                influences: (),
            },
            face_normals: Values::default(),
            triangle_controls: Values::default(),
        };
        result.validate()?;
        Ok(result)
    }
    /// Defines collision points from which a convex boundary can be constructed.
    pub fn points(positions: impl Into<Values<'data, [f32; 3]>>) -> Result<Self, Error> {
        let positions = positions.into();
        u32::try_from(positions.len()).map_err(|_| {
            Error::new(
                "m3.collision.points",
                "point count exceeds logical index capacity",
            )
        })?;
        let indices = Values::view(positions.len(), |index| index as u32);
        let result = Self {
            streams: VertexStreams {
                vertices: VertexData {
                    positions,
                    normals: (),
                    tangents: (),
                    colors: (),
                    texture_coordinates: (),
                    extras: (),
                },
                topology: Topology::Points(indices),
                influences: (),
            },
            face_normals: Values::default(),
            triangle_controls: Values::default(),
        };
        result.validate()?;
        Ok(result)
    }
    /// Defines polygon boundaries for authored convex-hull geometry.
    pub fn polygons(
        positions: impl Into<Values<'data, [f32; 3]>>,
        corners: impl Into<Values<'data, u32>>,
        offsets: impl Into<Values<'data, u32>>,
    ) -> Result<Self, Error> {
        let result = Self {
            streams: VertexStreams {
                vertices: VertexData {
                    positions: positions.into(),
                    normals: (),
                    tangents: (),
                    colors: (),
                    texture_coordinates: (),
                    extras: (),
                },
                topology: Topology::Polygons {
                    corners: corners.into(),
                    offsets: offsets.into(),
                },
                influences: (),
            },
            face_normals: Values::default(),
            triangle_controls: Values::default(),
        };
        result.validate()?;
        Ok(result)
    }
    /// Supplies an optional normal for each collision triangle.
    pub fn with_face_normals(
        mut self,
        normals: impl Into<Values<'data, [f32; 3]>>,
    ) -> Result<Self, Error> {
        self.face_normals = normals.into();
        self.validate()?;
        Ok(self)
    }
    /// Supplies scalar triangle controls whose game interpretation is unresolved.
    pub fn with_triangle_controls(
        mut self,
        controls: impl Into<Values<'data, f32>>,
    ) -> Result<Self, Error> {
        self.triangle_controls = controls.into();
        self.validate()?;
        Ok(self)
    }
    /// Returns the declared control for each triangle, when authored.
    pub fn triangle_controls(&self) -> Option<&Values<'data, f32>> {
        present(&self.triangle_controls)
    }
    /// Returns authored per-triangle normals, independent of vertex shading streams.
    pub fn face_normals(&self) -> Option<&Values<'data, [f32; 3]>> {
        present(&self.face_normals)
    }
}
impl ValidateLinks for M3CollisionGeometry<'_> {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
impl<'data> GeometrySchema<'data> for M3CollisionGeometry<'data> {
    fn edit_capabilities(&self) -> GeometryEditCapabilities {
        GeometryEditCapabilities {
            positions: true,
            normals: StreamSupport::Unsupported,
            tangents: StreamSupport::Unsupported,
            colors: StreamSupport::Unsupported,
            minimum_uv_sets: 0,
            maximum_uv_sets: Some(0),
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
            result.streams.topology = map.topology.clone();
        }
        if let Some(values) = edit.positions {
            result.streams.positions = values;
        }

        result.validate()?;
        Ok(result)
    }

    fn kind(&self) -> GeometryKind {
        GeometryKind::Collision
    }
    fn positions(&self) -> &Values<'data, [f32; 3]> {
        &self.streams.positions
    }
    fn normals(&self) -> Option<&Values<'data, [f32; 3]>> {
        None
    }
    fn tangents(&self) -> Option<&Values<'data, [f32; 4]>> {
        None
    }
    fn texture_coordinates(&self) -> &[Values<'data, [f32; 2]>] {
        &[]
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
        InfluenceRows::None
    }
    fn validate(&self) -> Result<(), Error> {
        validate_geometry(self)?;
        let face_count = match &self.streams.topology {
            Topology::Triangles(triangles) => triangles.len(),
            Topology::Points(_) | Topology::Polygons { .. }
                if self.face_normals.is_empty() && self.triangle_controls.is_empty() =>
            {
                0
            }
            _ => {
                return Err(Error::new(
                    "m3.collision.topology",
                    "collision geometry requires points, polygons, or triangles; per-triangle normals require triangles",
                ));
            }
        };
        if !self.triangle_controls.is_empty()
            && (self.triangle_controls.len() != face_count
                || self
                    .triangle_controls
                    .iter()
                    .any(|value| !value.is_finite()))
        {
            return Err(Error::new(
                "m3.collision.triangle_controls",
                "finite controls must cover each triangle",
            ));
        }
        if !self.face_normals.is_empty() && self.face_normals.len() != face_count {
            return Err(Error::new(
                "m3.collision.face_normals",
                "normal count must match collision triangle count",
            ));
        }
        if self
            .face_normals
            .iter()
            .any(|value| value.iter().any(|lane| !lane.is_finite()))
        {
            return Err(Error::new(
                "m3.collision.face_normals",
                "collision normals must be finite",
            ));
        }
        Ok(())
    }
}
/// Complete geometry shapes understood by native M3 models.
#[derive(Clone, Debug)]
pub enum M3Geometry<'data> {
    /// Packed, shaded, potentially deforming display geometry.
    Display(M3DisplayGeometry<'data>),
    /// Collision positions and connectivity independent of shading.
    Collision(M3CollisionGeometry<'data>),
}
impl<'data> M3Geometry<'data> {
    /// Creates empty display geometry.
    pub fn empty() -> Self {
        Self::default()
    }
    /// Creates checked display geometry with four ordered influence slots.
    pub fn new(
        positions: impl Into<Values<'data, [f32; 3]>>,
        normals: impl Into<Values<'data, [f32; 3]>>,
        triangles: impl Into<Values<'data, [u32; 3]>>,
        influences: impl Into<Values<'data, [Influence; 4]>>,
    ) -> Result<Self, Error> {
        Ok(Self::Display(M3DisplayGeometry::new(
            positions, normals, triangles, influences,
        )?))
    }
    /// Creates collision geometry without synthesizing absent vertex streams.
    pub fn collision(
        positions: impl Into<Values<'data, [f32; 3]>>,
        triangles: impl Into<Values<'data, [u32; 3]>>,
    ) -> Result<Self, Error> {
        Ok(Self::Collision(M3CollisionGeometry::new(
            positions, triangles,
        )?))
    }
    /// Adds tangents to display geometry; collision geometry has no tangent stream.
    pub fn with_tangents(self, values: impl Into<Values<'data, [f32; 4]>>) -> Result<Self, Error> {
        match self {
            Self::Display(value) => Ok(Self::Display(value.with_tangents(values)?)),
            Self::Collision(_) => Err(Error::new(
                "m3.geometry.kind",
                "collision geometry has no tangents",
            )),
        }
    }
    /// Adds ordered display UV slots; an empty slot denotes an absent native channel.
    pub fn with_texture_coordinates(
        self,
        values: Vec<Values<'data, [f32; 2]>>,
    ) -> Result<Self, Error> {
        match self {
            Self::Display(value) => Ok(Self::Display(value.with_texture_coordinates(values)?)),
            Self::Collision(_) => Err(Error::new(
                "m3.geometry.kind",
                "collision geometry has no texture-coordinate streams",
            )),
        }
    }
    /// Adds vertex colors to display geometry.
    pub fn with_colors(self, values: impl Into<Values<'data, [f32; 4]>>) -> Result<Self, Error> {
        match self {
            Self::Display(value) => Ok(Self::Display(value.with_colors(values)?)),
            Self::Collision(_) => Err(Error::new(
                "m3.geometry.kind",
                "collision geometry has no vertex colors",
            )),
        }
    }
}
impl Default for M3Geometry<'_> {
    fn default() -> Self {
        Self::Display(M3DisplayGeometry::default())
    }
}
impl ValidateLinks for M3Geometry<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Display(v) => v.visit_links(visitor),
            Self::Collision(v) => v.visit_links(visitor),
        }
    }
}
macro_rules! delegate {
    ($method:ident,$output:ty) => {
        fn $method(&self) -> $output {
            match self {
                Self::Display(v) => v.$method(),
                Self::Collision(v) => v.$method(),
            }
        }
    };
}
impl<'data> GeometrySchema<'data> for M3Geometry<'data> {
    fn edit_capabilities(&self) -> GeometryEditCapabilities {
        match self {
            Self::Display(value) => value.edit_capabilities(),
            Self::Collision(value) => value.edit_capabilities(),
        }
    }
    fn edited(&self, edit: GeometryEdit<'data>) -> Result<Self, Error> {
        match self {
            Self::Display(value) => Ok(Self::Display(value.edited(edit)?)),
            Self::Collision(value) => Ok(Self::Collision(value.edited(edit)?)),
        }
    }

    delegate!(kind, GeometryKind);
    delegate!(positions, &Values<'data, [f32; 3]>);
    delegate!(normals, Option<&Values<'data, [f32; 3]>>);
    delegate!(tangents, Option<&Values<'data, [f32; 4]>>);
    delegate!(texture_coordinates, &[Values<'data, [f32; 2]>]);
    delegate!(colors, Option<&Values<'data, [f32; 4]>>);
    delegate!(attributes, Attributes<'_, 'data>);
    delegate!(topology, &Topology<'data>);
    delegate!(influences, InfluenceRows<'_, 'data>);
    delegate!(validate,Result<(),Error>);
}

impl<'data> From<M3DisplayGeometry<'data>> for VertexGeometry<'data> {
    fn from(value: M3DisplayGeometry<'data>) -> Self {
        let GeometryStreams {
            vertices,
            topology,
            influences,
        } = value.streams;
        Self {
            streams: VertexStreams {
                vertices: VertexData {
                    positions: vertices.positions,
                    normals: vertices.normals,
                    tangents: vertices.tangents,
                    colors: vertices.colors,
                    texture_coordinates: vertices.texture_coordinates,
                    extras: (),
                },
                topology,
                influences: JointInfluences::Four(influences),
            },
        }
    }
}
impl<'data> TryFrom<VertexGeometry<'data>> for M3DisplayGeometry<'data> {
    type Error = Error;
    fn try_from(value: VertexGeometry<'data>) -> Result<Self, Error> {
        value.validate()?;
        let GeometryStreams {
            vertices,
            topology,
            influences,
        } = value.streams;
        if !matches!(topology, Topology::Triangles(_)) {
            return Err(Error::new(
                "geometry.topology",
                "M3 display geometry requires triangle connectivity",
            ));
        }
        let JointInfluences::Four(influences) = influences else {
            return Err(Error::new(
                "geometry.influences",
                "M3 display geometry requires four authored joint slots",
            ));
        };
        let result = Self {
            streams: VertexStreams {
                vertices: VertexData {
                    positions: vertices.positions,
                    normals: vertices.normals,
                    tangents: vertices.tangents,
                    colors: vertices.colors,
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
