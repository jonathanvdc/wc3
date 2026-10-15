//! Model-owned texture definitions with scoped identities.
use crate::model::semantic::ids::Scope;
use crate::model::semantic::profiles::{Extends, Narrows, Profile, Universal};
use crate::model::semantic::{Error, Texture, TextureId};
use std::collections::BTreeMap;

/// Texture definitions owned by a model.
#[derive(Clone, Debug)]
pub struct Resources<'data, P: Profile = Universal> {
    scope: Scope,
    textures: BTreeMap<TextureId, Texture<'data, P>>,
}
impl<P: Profile> Default for Resources<'_, P> {
    fn default() -> Self {
        Self {
            scope: Scope::default(),
            textures: BTreeMap::new(),
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
            textures,
        })
    }

    /// Starts an unpublished library with independent scoped identities.
    pub fn builder() -> ResourcesBuilder<'data, P> {
        ResourcesBuilder {
            resources: Self::default(),
        }
    }
    /// Number of texture definitions.
    pub fn len(&self) -> usize {
        self.textures.len()
    }
    /// Whether the library has no definitions.
    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
    }
    /// Inspects definitions in stable identity order.
    pub fn textures(&self) -> impl ExactSizeIterator<Item = (TextureId, &Texture<'data, P>)> {
        self.textures.iter().map(|(id, texture)| (*id, texture))
    }
    /// Resolves a texture; foreign library identities return `None`.
    pub fn texture(&self, id: TextureId) -> Option<&Texture<'data, P>> {
        self.textures.get(&id)
    }
    /// Starts a draft with the same texture identities and borrowed payloads.
    pub fn draft(&self) -> ResourcesBuilder<'data, P> {
        ResourcesBuilder {
            resources: self.clone(),
        }
    }
    /// Adds one texture definition.
    pub(crate) fn add_texture(&mut self, texture: Texture<'data, P>) -> TextureId {
        let id = self.scope.allocate();
        self.textures.insert(id, texture);
        id
    }
    /// Replaces a definition, rejecting foreign library identities.
    pub(crate) fn set_texture(
        &mut self,
        id: TextureId,
        texture: Texture<'data, P>,
    ) -> Result<(), Error> {
        let slot = self
            .textures
            .get_mut(&id)
            .ok_or_else(|| Error::new("resources", "unknown texture identity"))?;
        *slot = texture;
        Ok(())
    }
    /// Removes a definition; the owning model must be checked before publication.
    pub(crate) fn remove_texture(&mut self, id: TextureId) -> Option<Texture<'data, P>> {
        self.textures.remove(&id)
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
    /// Borrows a definition from the draft dependency arena.
    pub fn texture(&self, id: TextureId) -> Option<&Texture<'data, P>> {
        self.resources.texture(id)
    }
    /// Iterates definitions before publishing edits.
    pub fn textures(&self) -> impl ExactSizeIterator<Item = (TextureId, &Texture<'data, P>)> {
        self.resources.textures()
    }
    /// Adds a texture definition to the draft.
    pub fn add_texture(&mut self, texture: Texture<'data, P>) -> TextureId {
        self.resources.add_texture(texture)
    }
    /// Replaces a definition, rejecting foreign identities.
    pub fn set_texture(&mut self, id: TextureId, texture: Texture<'data, P>) -> Result<(), Error> {
        self.resources.set_texture(id, texture)
    }
    /// Removes a texture definition from the draft.
    pub fn remove_texture(&mut self, id: TextureId) -> Option<Texture<'data, P>> {
        self.resources.remove_texture(id)
    }
    /// Publishes the immutable library. The owning model validates its graph relationships.
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
    fn widening_moves_owned_texture_payloads() {
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
    }
}
