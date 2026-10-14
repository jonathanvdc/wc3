//! Attribute layers retain identities independently of physical ordering.
use super::{EdgeAttribute, Geometry};
use crate::model::semantic::{Attribute, AttributeSource, Error, Id};
/// Identity marker for a mesh-owned WEM attribute layer.
pub enum LayerObject {}
/// Stable layer identity scoped to its actual geometry owner.
pub type LayerId = Id<LayerObject>;
/// A standard geometric domain or a declared ordinal edge domain.
#[derive(Clone, Debug)]
pub enum LayerData<'a> {
    /// Vertex, corner, face, or mesh values.
    Standard(Attribute<'a>),
    /// Edge values whose endpoint connectivity is not declared.
    Edge(EdgeAttribute<'a>),
}
/// An independently identified authored layer.
#[derive(Clone, Debug)]
pub struct Layer<'a> {
    id: LayerId,
    /// Authored domain, name, and typed values.
    pub data: LayerData<'a>,
}
impl<'a> Layer<'a> {
    pub(super) fn new(id: LayerId, data: LayerData<'a>) -> Self {
        Self { id, data }
    }
    /// Identity that remains stable when other layers are inserted or removed.
    pub fn id(&self) -> LayerId {
        self.id
    }
}
impl<'a> Geometry<'a> {
    /// Adds an authored layer without copying its values.
    pub fn add_layer(&mut self, data: LayerData<'a>) -> LayerId {
        let id = self.layer_scope.allocate();
        self.layers.push(Layer::new(id, data));
        id
    }
    /// Borrows a layer only when this geometry owns its identity.
    pub fn layer(&self, id: LayerId) -> Option<&Layer<'a>> {
        self.layers.iter().find(|layer| layer.id == id)
    }
    /// Iterates all authored domains in layer order, including ordinal edges.
    pub fn layers(&self) -> impl ExactSizeIterator<Item = &Layer<'a>> {
        self.layers.iter()
    }
    /// Replaces a layer's authored values while retaining its identity.
    pub fn set_layer(&mut self, id: LayerId, data: LayerData<'a>) -> Result<(), Error> {
        let layer = self
            .layers
            .iter_mut()
            .find(|layer| layer.id == id)
            .ok_or_else(|| {
                Error::new("wem.geometry.layers", "layer belongs to another geometry")
            })?;
        layer.data = data;
        Ok(())
    }
    /// Removes a layer by actual identity, leaving all other handles unchanged.
    pub fn remove_layer(&mut self, id: LayerId) -> Option<LayerData<'a>> {
        let index = self.layers.iter().position(|layer| layer.id == id)?;
        Some(self.layers.remove(index).data)
    }
}
impl<'a> AttributeSource<'a> for Geometry<'a> {
    fn len(&self) -> usize {
        self.layers
            .iter()
            .filter(|layer| matches!(layer.data, LayerData::Standard(_)))
            .count()
    }
    fn attribute(&self, index: usize) -> &Attribute<'a> {
        self.layers
            .iter()
            .filter_map(|layer| match &layer.data {
                LayerData::Standard(value) => Some(value),
                LayerData::Edge(_) => None,
            })
            .nth(index)
            .expect("checked attribute facade index")
    }
}
