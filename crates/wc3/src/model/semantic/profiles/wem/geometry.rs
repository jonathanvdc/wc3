//! WEM retains polygon domains without projecting corners into duplicated vertices.
use super::{Layer, LayerData, RepairHistory, Section, SectionId};
use crate::model::semantic::geometry_edit::{check_edit, remap_attribute, remap_values};
use crate::model::semantic::TextureCoordinates;
use crate::model::semantic::{
    validate_geometry, AttributeValues, Attributes, Error, GeometryData, GeometrySchema,
    InfluenceRows, Link, NodeInfluence, Scope, Topology, ValidateLinks, Values,
};
use crate::model::semantic::{GeometryEdit, GeometryEditCapabilities, StreamSupport};
use std::borrow::Cow;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT_SECTION_SCOPE: AtomicU64 = AtomicU64::new(1);
/// One typed layer over the declared ordinal edge domain.
///
/// WEM supplies an edge count but does not define an endpoint table or ordering
/// convention. These identities are ordinal; they do not imply connectivity.
#[derive(Clone, Debug)]
pub struct EdgeAttribute<'a> {
    /// Authored attribute name.
    pub name: Cow<'a, str>,
    /// Values in the declared ordinal order.
    pub values: AttributeValues<'a>,
}
/// Complete WEM polygon geometry and its independently indexed authoring domains.
#[derive(Clone, Debug)]
pub struct Geometry<'a> {
    scope: u64,
    sections: Vec<Section<'a>>,
    face_sections: Values<'a, SectionId>,
    /// Geometric repair history, with references to this geometry’s draw groups.
    pub repair: RepairHistory<'a>,
    /// Geometric vertices, polygon loops, shading domains, and joint distributions.
    pub positions: Values<'a, [f32; 3]>,
    /// Vertex normals, when authored.
    pub normals: Values<'a, [f32; 3]>,
    /// Vertex tangent frames, when authored.
    pub tangents: Values<'a, [f32; 4]>,
    /// Ordered vertex texture coordinate sets.
    pub texture_coordinates: Vec<Values<'a, [f32; 2]>>,
    /// Vertex colors, when authored.
    pub colors: Values<'a, [f32; 4]>,
    /// Authored polygon or triangle connectivity.
    pub topology: Topology<'a>,
    /// Joint distributions for each geometric vertex.
    pub influences: Values<'a, Vec<NodeInfluence>>,
    pub(super) layer_scope: Scope,
    pub(super) layers: Vec<Layer<'a>>,
    /// Declared size of the edge domain, whose connectivity is unresolved.
    pub edge_count: usize,
    /// Optional triangle decompositions per polygon using geometric vertex identities.
    pub triangulation: Values<'a, Vec<[u32; 3]>>,
}
impl<'a> Geometry<'a> {
    /// Iterates actual authored polygon groups with their local identities.
    pub fn sections(&self) -> impl ExactSizeIterator<Item = (SectionId, &Section<'a>)> {
        self.sections.iter().enumerate().map(|(index, value)| {
            (
                SectionId {
                    scope: self.scope,
                    index: index as u32,
                },
                value,
            )
        })
    }
    /// Reads a polygon group only when the handle belongs to this geometry.
    pub fn section(&self, id: SectionId) -> Option<&Section<'a>> {
        (id.scope == self.scope)
            .then(|| self.sections.get(id.index as usize))
            .flatten()
    }
    /// Adds a group; the first group initially owns every polygon.
    pub fn add_section(&mut self, section: Section<'a>) -> Result<SectionId, Error> {
        let index = u32::try_from(self.sections.len())
            .map_err(|_| Error::new("wem.geometry.sections", "too many polygon groups"))?;
        let id = SectionId {
            scope: self.scope,
            index,
        };
        if self.sections.is_empty() {
            self.face_sections = Values::owned(vec![id; self.topology().primitive_count()]);
        }
        self.sections.push(section);
        Ok(id)
    }
    /// Borrows per-polygon authored group assignments.
    pub fn face_sections(&self) -> &Values<'a, SectionId> {
        &self.face_sections
    }
    /// Replaces group assignments after checking cardinality and geometry ownership.
    pub fn set_face_sections(&mut self, values: Values<'a, SectionId>) -> Result<(), Error> {
        if values.len() != self.topology().primitive_count()
            || values.iter().any(|id| self.section(id).is_none())
        {
            return Err(Error::new(
                "wem.geometry.sections",
                "invalid polygon-group assignment",
            ));
        }
        self.face_sections = values;
        Ok(())
    }

    /// Creates a geometry definition and checks domain relationships.
    pub fn new(data: GeometryData<'a>) -> Result<Self, Error> {
        if !data.influences.is_empty() {
            return Err(Error::new(
                "wem.geometry.influences",
                "WEM uses direct node identities, not joint-palette indices",
            ));
        }
        data.validate_no_additional_domains()?;
        let layer_scope = Scope::default();
        let mut data = data.into_parts();
        let layers = data
            .attributes
            .drain(..)
            .map(|attribute| Layer::new(layer_scope.allocate(), LayerData::Standard(attribute)))
            .collect();
        let mut scope = NEXT_SECTION_SCOPE.load(Ordering::Relaxed);
        loop {
            let next = scope
                .checked_add(1)
                .ok_or_else(|| Error::new("wem.geometry.sections", "section scope exhausted"))?;
            match NEXT_SECTION_SCOPE.compare_exchange_weak(
                scope,
                next,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(current) => scope = current,
            }
        }
        let value = Self {
            scope,
            sections: Vec::new(),
            face_sections: Values::default(),
            repair: RepairHistory::default(),
            positions: data.positions,
            normals: data.normals,
            tangents: data.tangents,
            texture_coordinates: data.texture_coordinates,
            colors: data.colors,
            topology: data.topology,
            influences: Values::default(),
            layer_scope,
            layers,
            edge_count: 0,
            triangulation: Values::default(),
        };
        value.validate()?;
        Ok(value)
    }
}
impl<'a> GeometrySchema<'a> for Geometry<'a> {
    fn edit_capabilities(&self) -> GeometryEditCapabilities {
        GeometryEditCapabilities {
            positions: true,
            normals: StreamSupport::Optional,
            tangents: StreamSupport::Optional,
            colors: StreamSupport::Optional,
            minimum_uv_sets: 0,
            maximum_uv_sets: None,
            vertex_remap: self.edge_count == 0
                && self.triangulation.is_empty()
                && self.repair.splits.is_empty()
                && self.repair.removed_faces.is_empty(),
        }
    }
    fn edited(&self, edit: GeometryEdit<'a>) -> Result<Self, Error> {
        self.validate()?;
        check_edit(&edit, self.edit_capabilities())?;
        let mut result = self.clone();
        if let Some(map) = &edit.remap {
            map.check(self)?;
            remap_values(&mut result.positions, &map.new_to_old);
            remap_values(&mut result.normals, &map.new_to_old);
            remap_values(&mut result.tangents, &map.new_to_old);
            remap_values(&mut result.colors, &map.new_to_old);
            for stream in &mut result.texture_coordinates {
                remap_values(stream, &map.new_to_old);
            }
            remap_values(&mut result.influences, &map.new_to_old);
            for layer in &mut result.layers {
                if let LayerData::Standard(attribute) = &mut layer.data {
                    remap_attribute(attribute, &map.new_to_old);
                }
            }
            result.topology = map.topology.clone();
        }
        if let Some(values) = edit.positions {
            result.positions = values;
        }
        if let Some(values) = edit.normals {
            result.normals = values;
        }
        if let Some(values) = edit.tangents {
            result.tangents = values;
        }
        if let Some(values) = edit.colors {
            result.colors = values;
        }
        if let Some(values) = edit.texture_coordinates {
            result.texture_coordinates = values;
        }

        result.validate()?;
        Ok(result)
    }

    fn positions(&self) -> &Values<'a, [f32; 3]> {
        &self.positions
    }
    fn normals(&self) -> Option<&Values<'a, [f32; 3]>> {
        (!self.normals.is_empty()).then_some(&self.normals)
    }
    fn tangents(&self) -> Option<&Values<'a, [f32; 4]>> {
        (!self.tangents.is_empty()).then_some(&self.tangents)
    }
    fn texture_coordinates(&self) -> TextureCoordinates<'_, 'a> {
        TextureCoordinates::Slice(&self.texture_coordinates)
    }
    fn colors(&self) -> Option<&Values<'a, [f32; 4]>> {
        (!self.colors.is_empty()).then_some(&self.colors)
    }
    fn attributes(&self) -> Attributes<'_, 'a> {
        Attributes::Source(self)
    }
    fn topology(&self) -> &Topology<'a> {
        &self.topology
    }
    fn influences(&self) -> InfluenceRows<'_, 'a> {
        if self.influences.is_empty() {
            InfluenceRows::None
        } else {
            InfluenceRows::Nodes(&self.influences)
        }
    }
    fn validate(&self) -> Result<(), Error> {
        validate_geometry(self)?;
        if (!self.sections.is_empty()
            && self.face_sections.len() != self.topology().primitive_count())
            || self
                .face_sections
                .iter()
                .any(|id| self.section(id).is_none())
            || self
                .repair
                .removed_faces
                .iter()
                .any(|face| face.section.is_some_and(|id| self.section(id).is_none()))
        {
            return Err(Error::new(
                "wem.geometry.sections",
                "polygon or repair group reference is invalid",
            ));
        }

        if self.layers.iter().any(|layer| matches!(&layer.data, LayerData::Edge(value) if value.values.len()!=self.edge_count))
        {
            return Err(Error::new(
                "wem.geometry.edges",
                "edge layer cardinality differs from declared domain",
            ));
        }
        if !self.triangulation.is_empty() {
            let faces = match self.topology() {
                Topology::Polygons { offsets, .. } => offsets.len().saturating_sub(1),
                Topology::Triangles(triangles) => triangles.len(),
                _ => {
                    return Err(Error::new(
                        "wem.geometry.triangulation",
                        "decomposition requires polygon or triangle topology",
                    ));
                }
            };
            if self.triangulation.len() != faces
                || self
                    .triangulation
                    .iter()
                    .flatten()
                    .flatten()
                    .any(|index| index as usize >= self.positions.len())
            {
                return Err(Error::new(
                    "wem.geometry.triangulation",
                    "invalid polygon decomposition",
                ));
            }
        }
        Ok(())
    }
}
impl ValidateLinks for Geometry<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for row in self.influences.iter() {
            for influence in row {
                influence.node.visit_links(visitor);
            }
        }
        for section in &self.sections {
            section.visit_links(visitor);
        }

        if self.validate().is_err() {
            visitor(Link::InvalidValue);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Metadata, SectionFlags};
    use super::*;
    use crate::model::semantic::Bounds;
    fn geometry() -> Geometry<'static> {
        Geometry::new(
            GeometryData::triangles(
                Values::owned(vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]),
                Values::owned(vec![[0, 1, 2]]),
            )
            .unwrap(),
        )
        .unwrap()
    }
    fn section() -> Section<'static> {
        Section {
            availability: Default::default(),
            name: Cow::Borrowed("body"),
            slot: None,
            rigid_node: None,
            selection_group: 0,
            flags: SectionFlags::default(),
            bounds: Bounds::default(),
            metadata: Metadata::default(),
        }
    }
    #[test]
    fn polygon_groups_cannot_reference_another_mesh() {
        let mut first = geometry();
        let id = first.add_section(section()).unwrap();
        let mut second = geometry();
        second.add_section(section()).unwrap();
        assert!(second.set_face_sections(Values::owned(vec![id])).is_err());
        assert!(first.set_face_sections(Values::owned(vec![id])).is_ok());
        assert_eq!(first.face_sections().get(0), Some(id));
    }
    #[test]
    fn edge_attributes_keep_ordinal_values_without_inventing_endpoints() {
        let mut geometry = geometry();
        geometry.edge_count = 2;
        geometry.add_layer(LayerData::Edge(EdgeAttribute {
            name: Cow::Borrowed("crease"),
            values: AttributeValues::Unsigned(Values::owned(vec![3, 7])),
        }));
        assert!(geometry.validate().is_ok());
        geometry.edge_count = 3;
        assert!(geometry.validate().is_err());
    }
}
