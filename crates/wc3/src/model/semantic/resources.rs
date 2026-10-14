//! Shared texture definitions retain one edit authority across several models.
use crate::model::semantic::ids::Scope;
use crate::model::semantic::profiles::{Extends, Narrows, Profile, Universal};
use crate::model::semantic::{Error, Texture, TextureId};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT_AUTHORITY: AtomicU64 = AtomicU64::new(1);
fn authority() -> u64 {
    NEXT_AUTHORITY.fetch_add(1, Ordering::Relaxed)
}

/// Texture definitions owned by a model collection and shared by all its views.
#[derive(Clone, Debug)]
pub struct Resources<'data, P: Profile = Universal> {
    scope: Scope,
    textures: BTreeMap<TextureId, Texture<'data, P>>,
    authority: u64,
}
impl<P: Profile> Default for Resources<'_, P> {
    fn default() -> Self {
        Self {
            scope: Scope::default(),
            textures: BTreeMap::new(),
            authority: authority(),
        }
    }
}
impl<'data> Resources<'data, Universal> {
    /// Creates an empty library using the complete built-in vocabulary.
    pub fn new() -> Self {
        Self::default()
    }
}
impl<'data, P: Profile> Resources<'data, P> {
    /// Projects resource controls exactly, moving paths and retaining texture identities.
    pub fn try_narrow<Q: Narrows<P>>(self) -> Result<Resources<'data, Q>, Error> {
        let textures = self
            .textures
            .into_iter()
            .map(|(id, texture)| {
                Ok((
                    id,
                    Texture {
                        resource: texture.resource,
                        color_space: texture.color_space,
                        sampler: texture.sampler,
                        controls: Q::texture(texture.controls)?,
                    },
                ))
            })
            .collect::<Result<_, Error>>()?;
        Ok(Resources {
            scope: self.scope,
            authority: self.authority,
            textures,
        })
    }

    /// Starts an unpublished library with independent scoped identities.
    pub fn builder() -> ResourcesBuilder<'data, P> {
        ResourcesBuilder {
            resources: Self::default(),
        }
    }
    /// Number of shared texture definitions.
    pub fn len(&self) -> usize {
        self.textures.len()
    }
    /// Whether the library has no definitions.
    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
    }
    /// Inspects shared definitions in stable identity order.
    pub fn textures(&self) -> impl ExactSizeIterator<Item = (TextureId, &Texture<'data, P>)> {
        self.textures.iter().map(|(id, texture)| (*id, texture))
    }
    /// Resolves a shared texture; foreign library identities return `None`.
    pub fn texture(&self, id: TextureId) -> Option<&Texture<'data, P>> {
        self.textures.get(&id)
    }
    /// Starts a draft with the same texture identities and borrowed payloads.
    pub fn draft(&self) -> ResourcesBuilder<'data, P> {
        ResourcesBuilder {
            resources: self.clone(),
        }
    }
    /// Identity of the most recent published edit authority.
    pub(crate) fn authority_id(&self) -> u64 {
        self.authority
    }
    pub(crate) fn merge(mut self, other: Self) -> Result<Self, Error> {
        if self.is_empty() {
            return Ok(other);
        }
        if other.is_empty() || self.authority == other.authority {
            return Ok(self);
        }
        if other
            .textures
            .keys()
            .any(|id| self.textures.contains_key(id))
        {
            return Err(Error::new(
                "resources",
                "conflicting edits to the same texture definition",
            ));
        }
        self.textures.extend(other.textures);
        self.authority = authority();
        Ok(self)
    }
    /// Widens texture controls while preserving identities and borrowed resources.
    pub fn widen<Q: Extends<P>>(self) -> Resources<'data, Q> {
        self.map_controls(Q::texture)
    }
    pub(crate) fn map_controls<Q: Profile>(
        self,
        map: impl Fn(P::Texture<'data>) -> Q::Texture<'data>,
    ) -> Resources<'data, Q> {
        Resources {
            scope: self.scope,
            authority: authority(),
            textures: self
                .textures
                .into_iter()
                .map(|(id, texture)| {
                    (
                        id,
                        Texture {
                            resource: texture.resource,
                            color_space: texture.color_space,
                            sampler: texture.sampler,
                            controls: map(texture.controls),
                        },
                    )
                })
                .collect(),
        }
    }
}
/// An unpublished library draft; model publication checks inherited relationships.
#[derive(Clone, Debug)]
pub struct ResourcesBuilder<'data, P: Profile = Universal> {
    resources: Resources<'data, P>,
}
impl<'data, P: Profile> ResourcesBuilder<'data, P> {
    pub(crate) fn from_resources(resources: Resources<'data, P>) -> Self {
        Self { resources }
    }
    pub(crate) fn resources(&self) -> &Resources<'data, P> {
        &self.resources
    }
    /// Borrows a definition from the draft dependency arena.
    pub fn texture(&self, id: TextureId) -> Option<&Texture<'data, P>> {
        self.resources.texture(id)
    }
    /// Iterates shared definitions before publishing edits.
    pub fn textures(&self) -> impl ExactSizeIterator<Item = (TextureId, &Texture<'data, P>)> {
        self.resources.textures()
    }
    /// Adds one shared texture definition.
    pub fn add_texture(&mut self, texture: Texture<'data, P>) -> TextureId {
        let id = self.resources.scope.allocate();
        self.resources.textures.insert(id, texture);
        self.resources.authority = authority();
        id
    }
    /// Replaces a shared definition, rejecting foreign library identities.
    pub fn set_texture(&mut self, id: TextureId, texture: Texture<'data, P>) -> Result<(), Error> {
        let slot = self
            .resources
            .textures
            .get_mut(&id)
            .ok_or_else(|| Error::new("resources", "unknown texture identity"))?;
        *slot = texture;
        self.resources.authority = authority();
        Ok(())
    }
    /// Removes a definition; inheriting models must be checked before publication.
    pub fn remove_texture(&mut self, id: TextureId) -> Option<Texture<'data, P>> {
        let removed = self.resources.textures.remove(&id);
        if removed.is_some() {
            self.resources.authority = authority();
        }
        removed
    }
    /// Publishes the immutable library. Inheriting models validate its graph relationships.
    pub fn finish(self) -> Resources<'data, P> {
        self.resources
    }
}

#[cfg(test)]
mod tests {
    use super::Resources;
    use crate::model::semantic::profiles::{Mdlx, Universal};
    use crate::model::semantic::{Resource, Texture};
    use std::borrow::Cow;

    #[test]
    fn widening_and_merging_move_owned_texture_payloads() {
        let bytes = b"owned-texture-path.blp".to_vec();
        let pointer = bytes.as_ptr();
        let mut source = Resources::<Mdlx>::builder();
        let mut texture = Texture::file("");
        texture.resource = Resource::Path(Cow::Owned(bytes));
        let id = source.add_texture(texture);
        let graph = source.finish().widen::<Universal>();
        let Resource::Path(value) = &graph.texture(id).unwrap().resource else {
            unreachable!()
        };
        assert_eq!(value.as_ptr(), pointer);
        let mut second = Resources::<Universal>::builder();
        second.add_texture(Texture::file("other"));
        let graph = second.finish().merge(graph).unwrap();
        let Resource::Path(value) = &graph.texture(id).unwrap().resource else {
            unreachable!()
        };
        assert_eq!(value.as_ptr(), pointer);
    }
}
