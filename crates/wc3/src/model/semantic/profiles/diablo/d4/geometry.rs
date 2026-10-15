//! Diablo IV declared vertex streams are logical values independent of buffer packing.
use crate::model::semantic::TextureCoordinates;
use crate::model::semantic::{
    AttributeDomain, Attributes, Error, GeometryData, GeometryEdit, GeometryEditCapabilities,
    GeometryInfluences, GeometryKind, GeometrySchema, InfluenceRows, Link, Topology,
    UniversalGeometry, ValidateLinks, Values,
};

/// Triangle geometry whose independently declared streams retain their logical values.
#[derive(Clone, Debug, Default)]
pub struct Geometry<'a> {
    data: GeometryData<'a>,
}
impl<'a> Geometry<'a> {
    /// Checks connectivity and vertex stream declarations without imposing native packing.
    pub fn new(data: GeometryData<'a>) -> Result<Self, Error> {
        let value = Self { data };
        value.validate()?;
        Ok(value)
    }
    /// Inspects complete streams, including independently declared additional attributes.
    pub fn data(&self) -> &GeometryData<'a> {
        &self.data
    }
    /// Consumes the family declaration while retaining its array storage.
    pub fn into_data(self) -> GeometryData<'a> {
        self.data
    }
}
impl ValidateLinks for Geometry<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.data.visit_links(visitor);
    }
}
impl<'a> GeometrySchema<'a> for Geometry<'a> {
    fn edit_capabilities(&self) -> GeometryEditCapabilities {
        self.data.edit_capabilities()
    }
    fn edited(&self, edit: GeometryEdit<'a>) -> Result<Self, Error> {
        Self::new(self.data.edited(edit)?)
    }

    fn positions(&self) -> &Values<'a, [f32; 3]> {
        self.data.positions()
    }
    fn normals(&self) -> Option<&Values<'a, [f32; 3]>> {
        self.data.normals()
    }
    fn tangents(&self) -> Option<&Values<'a, [f32; 4]>> {
        self.data.tangents()
    }
    fn texture_coordinates(&self) -> TextureCoordinates<'_, 'a> {
        self.data.texture_coordinates()
    }
    fn colors(&self) -> Option<&Values<'a, [f32; 4]>> {
        self.data.colors()
    }
    fn attributes(&self) -> Attributes<'_, 'a> {
        self.data.attributes()
    }
    fn topology(&self) -> &Topology<'a> {
        self.data.topology()
    }
    fn influences(&self) -> InfluenceRows<'_, 'a> {
        self.data.influences()
    }
    fn validate(&self) -> Result<(), Error> {
        self.data.validate_no_additional_domains()?;
        if self.data.kind != GeometryKind::Display {
            return Err(Error::new("d4.geometry.kind", "expected display geometry"));
        }
        if matches!(self.data.influences, GeometryInfluences::MatrixGroups(_)) {
            return Err(Error::new(
                "d4.geometry.influences",
                "declared geometry requires weighted joint contributions",
            ));
        }
        if !matches!(self.data.connectivity.primary, Topology::Triangles(_))
            || self
                .data
                .attributes
                .iter()
                .any(|a| a.domain != AttributeDomain::Vertex)
        {
            return Err(Error::new(
                "d4.geometry",
                "Diablo IV declarations describe triangle connectivity and vertex-domain streams",
            ));
        }
        self.data.validate()
    }
}

impl<'a> From<Geometry<'a>> for UniversalGeometry<'a> {
    fn from(value: Geometry<'a>) -> Self {
        Self::Data(value.into_data())
    }
}
impl<'a> TryFrom<UniversalGeometry<'a>> for Geometry<'a> {
    type Error = Error;
    fn try_from(value: UniversalGeometry<'a>) -> Result<Self, Error> {
        match value {
            UniversalGeometry::Data(data) => Self::new(data),
            _ => Err(Error::new(
                "d4.narrow.geometry",
                "geometry carries a different authored domain",
            )),
        }
    }
}
