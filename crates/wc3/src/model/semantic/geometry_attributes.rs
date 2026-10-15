//! Canonical typed attributes, keyed by their meaning and correspondence domain.
use super::geometry_edit::remap_attribute;
use super::{Attribute, AttributeDomain, AttributeSemantic, AttributeValues, Error, Values};
use std::ops::Index;
use std::slice::Iter;
use std::sync::LazyLock;
use std::vec::IntoIter;

/// Ordered attributes with one definition per semantic and domain.
#[derive(Clone, Debug, Default)]
pub struct GeometryAttributes<'data> {
    entries: Vec<Attribute<'data>>,
}
impl<'data> GeometryAttributes<'data> {
    /// Inserts a definition without replacing an already authored key.
    pub fn insert(&mut self, attribute: Attribute<'data>) -> Result<(), Error> {
        if self.get(&attribute.semantic, attribute.domain).is_some() {
            return Err(Error::new(
                "geometry.attributes",
                "duplicate semantic and domain",
            ));
        }
        self.check_type(&attribute)?;
        self.entries.push(attribute);
        Ok(())
    }
    /// Replaces one definition, retaining its position in authored order.
    pub fn set(&mut self, attribute: Attribute<'data>) -> Result<(), Error> {
        self.check_type(&attribute)?;
        if let Some(index) = self.entries.iter().position(|value| {
            value.semantic == attribute.semantic && value.domain == attribute.domain
        }) {
            self.entries[index] = attribute;
        } else {
            self.entries.push(attribute);
        }
        Ok(())
    }
    fn check_type(&self, attribute: &Attribute<'data>) -> Result<(), Error> {
        use AttributeSemantic as S;
        use AttributeValues as V;
        let valid = match (&attribute.semantic, &attribute.values) {
            (S::Normal | S::TangentVector | S::Binormal, V::Vector3(_)) => true,
            (S::Tangent | S::Color(_), V::Vector4(_)) => true,
            (S::TexCoord(_) | S::LightmapCoordinates, V::Vector2(_)) => true,
            (S::ClothVertex, V::Unsigned(_)) => true,
            (S::CollisionTriangleControl, V::Scalar(_)) => true,
            (S::SurfaceControls, V::SurfaceControls(_)) => {
                attribute.domain == AttributeDomain::Face
            }
            (S::Material, V::Materials(_)) => true,
            (S::Named(_), _) => true,
            _ => false,
        };
        if valid {
            Ok(())
        } else {
            Err(Error::new(
                "geometry.attributes",
                "attribute value type does not match its semantic",
            ))
        }
    }
    /// Inspects a definition by meaning and correspondence domain.
    pub fn get(
        &self,
        semantic: &AttributeSemantic<'_>,
        domain: AttributeDomain,
    ) -> Option<&Attribute<'data>> {
        self.entries
            .iter()
            .find(|entry| &entry.semantic == semantic && entry.domain == domain)
    }
    /// Edits one definition atomically, checking its type and key uniqueness.
    pub fn update(
        &mut self,
        index: usize,
        edit: impl FnOnce(&mut Attribute<'data>),
    ) -> Result<(), Error> {
        let mut attribute = self
            .entries
            .get(index)
            .ok_or_else(|| Error::new("geometry.attributes", "attribute is absent"))?
            .clone();
        edit(&mut attribute);
        self.check_type(&attribute)?;
        if self.entries.iter().enumerate().any(|(other, value)| {
            other != index
                && value.semantic == attribute.semantic
                && value.domain == attribute.domain
        }) {
            return Err(Error::new(
                "geometry.attributes",
                "duplicate semantic and domain",
            ));
        }
        self.entries[index] = attribute;
        Ok(())
    }
    /// Removes a definition by its key.
    pub fn take(
        &mut self,
        semantic: &AttributeSemantic<'_>,
        domain: AttributeDomain,
    ) -> Option<Attribute<'data>> {
        self.entries
            .iter()
            .position(|entry| &entry.semantic == semantic && entry.domain == domain)
            .map(|index| self.entries.remove(index))
    }
    /// Borrows definitions in authored order.
    pub fn as_slice(&self) -> &[Attribute<'data>] {
        &self.entries
    }
    /// Visits definitions without copying their arrays.
    pub fn iter(&self) -> Iter<'_, Attribute<'data>> {
        self.entries.iter()
    }
    /// Number of authored definitions.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    /// Whether no definitions are authored.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    /// Removes all definitions.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    /// Removes a definition in authored order.
    pub fn remove(&mut self, index: usize) -> Attribute<'data> {
        self.entries.remove(index)
    }
    /// Retains definitions selected by their meaning and domain.
    pub fn retain(&mut self, mut keep: impl FnMut(&Attribute<'data>) -> bool) {
        self.entries.retain(|entry| keep(entry));
    }
    pub(crate) fn remap(&mut self, indices: &[u32]) {
        for attribute in &mut self.entries {
            remap_attribute(attribute, indices);
        }
    }
    /// Inspects a three-vector attribute without relabeling its meaning.
    pub fn vector3(
        &self,
        semantic: &AttributeSemantic<'_>,
        domain: AttributeDomain,
    ) -> Option<&Values<'data, [f32; 3]>> {
        match &self.get(semantic, domain)?.values {
            AttributeValues::Vector3(values) => Some(values),
            _ => None,
        }
    }
    /// Inspects a four-vector attribute without relabeling its meaning.
    pub fn vector4(
        &self,
        semantic: &AttributeSemantic<'_>,
        domain: AttributeDomain,
    ) -> Option<&Values<'data, [f32; 4]>> {
        match &self.get(semantic, domain)?.values {
            AttributeValues::Vector4(values) => Some(values),
            _ => None,
        }
    }
    /// Inspects ordered vertex texture-coordinate slots.
    pub fn texture_coordinates(&self) -> TextureCoordinates<'_, 'data> {
        TextureCoordinates::Attributes(self)
    }
}
impl<'data> TryFrom<Vec<Attribute<'data>>> for GeometryAttributes<'data> {
    type Error = Error;
    fn try_from(entries: Vec<Attribute<'data>>) -> Result<Self, Error> {
        let mut result = Self::default();
        for entry in entries {
            result.insert(entry)?;
        }
        Ok(result)
    }
}
impl<'data> IntoIterator for GeometryAttributes<'data> {
    type Item = Attribute<'data>;
    type IntoIter = IntoIter<Attribute<'data>>;
    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}
impl<'view, 'data> IntoIterator for &'view GeometryAttributes<'data> {
    type Item = &'view Attribute<'data>;
    type IntoIter = Iter<'view, Attribute<'data>>;
    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}
impl<'data> Index<usize> for GeometryAttributes<'data> {
    type Output = Attribute<'data>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

/// Borrowed inspection of ordered UV slots from constrained or shared geometry.
#[derive(Clone, Copy, Debug)]
pub enum TextureCoordinates<'view, 'data> {
    /// Fixed or contiguous profile-owned slots.
    Slice(&'view [Values<'data, [f32; 2]>]),
    /// Slots stored as typed vertex attributes.
    Attributes(&'view GeometryAttributes<'data>),
}
impl<'view, 'data> TextureCoordinates<'view, 'data> {
    /// Number of authored slots, including empty slots.
    pub fn len(self) -> usize {
        match self {
            Self::Slice(values) => values.len(),
            Self::Attributes(values) => values
                .iter()
                .filter_map(|attribute| match (&attribute.semantic, attribute.domain) {
                    (AttributeSemantic::TexCoord(slot), AttributeDomain::Vertex) => {
                        Some(*slot as usize + 1)
                    }
                    _ => None,
                })
                .max()
                .unwrap_or(0),
        }
    }
    /// Whether no slots are authored.
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
    /// Inspects an authored slot.
    pub fn get(self, index: usize) -> Option<&'view Values<'data, [f32; 2]>> {
        match self {
            Self::Slice(values) => values.get(index),
            Self::Attributes(values) => {
                if index >= self.len() {
                    return None;
                }
                match values
                    .get(
                        &AttributeSemantic::TexCoord(u32::try_from(index).ok()?),
                        AttributeDomain::Vertex,
                    )
                    .map(|attribute| &attribute.values)
                {
                    Some(AttributeValues::Vector2(values)) => Some(values),
                    _ => Some(&EMPTY_UV),
                }
            }
        }
    }
    /// Visits slots in ascending order, exposing gaps as empty streams.
    pub fn iter(self) -> impl ExactSizeIterator<Item = &'view Values<'data, [f32; 2]>> {
        (0..self.len()).map(move |index| self.get(index).expect("validated UV slots"))
    }
    /// Clones slot handles while sharing their array storage.
    pub fn to_vec(self) -> Vec<Values<'data, [f32; 2]>> {
        self.iter().cloned().collect()
    }
}
impl<'data> Index<usize> for TextureCoordinates<'_, 'data> {
    type Output = Values<'data, [f32; 2]>;
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("UV slot")
    }
}
impl<'view, 'data> From<&'view [Values<'data, [f32; 2]>]> for TextureCoordinates<'view, 'data> {
    fn from(value: &'view [Values<'data, [f32; 2]>]) -> Self {
        Self::Slice(value)
    }
}

static EMPTY_UV: LazyLock<Values<'static, [f32; 2]>> = LazyLock::new(Values::default);

/// Ordered borrowed traversal of UV slots.
pub struct TextureCoordinateIter<'view, 'data> {
    streams: TextureCoordinates<'view, 'data>,
    index: usize,
}
impl<'view, 'data> Iterator for TextureCoordinateIter<'view, 'data> {
    type Item = &'view Values<'data, [f32; 2]>;
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.streams.get(self.index)?;
        self.index += 1;
        Some(value)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.streams.len() - self.index;
        (count, Some(count))
    }
}
impl ExactSizeIterator for TextureCoordinateIter<'_, '_> {}
impl<'view, 'data> IntoIterator for TextureCoordinates<'view, 'data> {
    type Item = &'view Values<'data, [f32; 2]>;
    type IntoIter = TextureCoordinateIter<'view, 'data>;
    fn into_iter(self) -> Self::IntoIter {
        TextureCoordinateIter {
            streams: self,
            index: 0,
        }
    }
}
impl<'view, 'data, const N: usize> From<&'view [Values<'data, [f32; 2]>; N]>
    for TextureCoordinates<'view, 'data>
{
    fn from(value: &'view [Values<'data, [f32; 2]>; N]) -> Self {
        Self::Slice(value)
    }
}
impl<'view, 'data> From<&'view Vec<Values<'data, [f32; 2]>>> for TextureCoordinates<'view, 'data> {
    fn from(value: &'view Vec<Values<'data, [f32; 2]>>) -> Self {
        Self::Slice(value)
    }
}
