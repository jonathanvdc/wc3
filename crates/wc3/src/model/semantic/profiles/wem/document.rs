//! Document-wide authoring declarations and actual animation selection programs.
use super::Family;
use crate::model::semantic::Scope;
use crate::model::semantic::{Bounds, ClipId, Error, Id, Link, ValidateLinks};
use std::borrow::Cow;
/// Marker of a document-owned animation selection program.
pub enum AnimationSetObject {}
/// Checked identity within an owning document's animation selection vocabulary.
pub type AnimationSetId = Id<AnimationSetObject>;
/// Declared coordinate convention; axes remain explicit on each model.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CoordinateConvention {
    /// General Blizzard model convention.
    #[default]
    Blizzard,
    /// StarCraft II authoring convention.
    StarCraft,
    /// Unrecognized authoring convention.
    Unknown(u8),
}
/// A named external tag-to-playback selection program.
#[derive(Clone, Debug)]
pub struct AnimationSet<'a> {
    /// Display name.
    pub name: Cow<'a, str>,
    /// Tag-to-clip assignments in authored order; clips retain their actual model owner.
    pub tags: Vec<(u32, ClipId)>,
    /// Selection program consulted when this program has no matching tag.
    pub fallback: Option<AnimationSetId>,
}
/// Actual document-wide values, independent of physical file layout.
#[derive(Clone, Debug)]
pub struct DocumentControls<'a> {
    /// Document display name.
    pub name: Cow<'a, str>,
    /// Declared shader and actor families in authoring order.
    pub families: Vec<Family>,
    /// Selected family when no explicit choice is made.
    pub default_family: Family,
    /// Declared convention independent of the numeric coordinate streams.
    pub convention: CoordinateConvention,
    /// Authored unit multiplier; no physical meter declaration is implied.
    pub unit_multiplier: f32,
    /// Conservative bounds of all document models.
    pub bounds: Bounds,
    scope: Scope,
    animation_sets: Vec<(AnimationSetId, AnimationSet<'a>)>,
}
impl Default for DocumentControls<'_> {
    fn default() -> Self {
        Self {
            name: Cow::Borrowed(""),
            families: vec![Family::Common],
            default_family: Family::Common,
            convention: CoordinateConvention::default(),
            unit_multiplier: 1.,
            bounds: Bounds::default(),
            scope: Scope::default(),
            animation_sets: Vec::new(),
        }
    }
}
impl<'a> DocumentControls<'a> {
    /// Allocates a selection identity before filling its fallback relationships.
    pub fn reserve_animation_set(&mut self) -> AnimationSetId {
        let id = self.scope.allocate();
        self.animation_sets.push((
            id,
            AnimationSet {
                name: Cow::Borrowed(""),
                tags: Vec::new(),
                fallback: None,
            },
        ));
        id
    }
    /// Replaces a document-owned selection program without changing its identity.
    pub fn set_animation_set(
        &mut self,
        id: AnimationSetId,
        value: AnimationSet<'a>,
    ) -> Result<(), Error> {
        let entry = self
            .animation_sets
            .iter_mut()
            .find(|(key, _)| *key == id)
            .ok_or_else(|| {
                Error::new(
                    "wem.animation_set",
                    "selection program belongs to another document",
                )
            })?;
        entry.1 = value;
        Ok(())
    }
    /// Borrows one selection program by its checked identity.
    pub fn animation_set(&self, id: AnimationSetId) -> Option<&AnimationSet<'a>> {
        self.animation_sets
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Iterates all independently authored selection programs.
    pub fn animation_sets(
        &self,
    ) -> impl ExactSizeIterator<Item = (AnimationSetId, &AnimationSet<'a>)> {
        self.animation_sets.iter().map(|(id, value)| (*id, value))
    }
    /// Checks declarations and selection fallback relationships independently of model clip references.
    pub fn validate(&self) -> Result<(), Error> {
        if !self.unit_multiplier.is_finite()
            || self.bounds.validate().is_err()
            || !self.families.contains(&self.default_family)
        {
            return Err(Error::new(
                "wem.document",
                "invalid declared authoring values",
            ));
        }
        for (id, _) in &self.animation_sets {
            let mut visited = Vec::new();
            let mut current = Some(*id);
            while let Some(id) = current {
                if visited.contains(&id) {
                    return Err(Error::new("wem.animation_set", "cyclic fallback selection"));
                }
                visited.push(id);
                current = self
                    .animation_set(id)
                    .ok_or_else(|| {
                        Error::new("wem.animation_set", "fallback belongs to another document")
                    })?
                    .fallback;
            }
        }
        Ok(())
    }
}
impl ValidateLinks for DocumentControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for (_, set) in &self.animation_sets {
            for (_, clip) in &set.tags {
                clip.visit_links(visitor)
            }
        }
        if self.validate().is_err() {
            visitor(Link::InvalidValue)
        }
    }
}
