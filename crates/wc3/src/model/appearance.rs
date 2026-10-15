//! Appearance vocabularies preserve the assignments supported by their model family.
/// Diablo III named appearance variants.
pub mod diablo3;
/// Diablo IV indexed appearance variants.
pub mod diablo4;

use crate::model::profiles::Either;
use crate::model::{Error, Link, Look, LookId, MaterialSlotId, Scope, ValidateLinks};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::iter::once;

/// A complete appearance vocabulary, including its own material assignment domains.
pub trait AppearanceSchema<'data>: Clone + Debug + Default + Send + Sync + ValidateLinks {
    /// Complete draw assignments exposed by this vocabulary, when it uses draw-based appearances.
    fn looks(&self) -> Box<dyn ExactSizeIterator<Item = (LookId, &Look<'data>)> + '_>;
    /// Looks up a draw-based appearance by identity.
    fn look(&self, id: LookId) -> Option<&Look<'data>>;
    /// Default draw-based appearance, when one is authored.
    fn default_look(&self) -> Option<LookId>;
    /// Whether this vocabulary owns a named material slot.
    fn contains_slot(&self, _id: MaterialSlotId) -> bool {
        false
    }
    /// Checks local authoring invariants before publication.
    fn validate(&self) -> Result<(), Error>;
}
/// An appearance vocabulary that supports complete direct draw-material assignments.
pub trait DrawAppearanceSchema<'data>: AppearanceSchema<'data> {
    /// Edits an existing complete assignment; publication checks its relationships.
    fn look_mut(&mut self, id: LookId) -> Option<&mut Look<'data>>;
    /// Adds a draw-based appearance with an identity supplied by its owning graph.
    fn add_look(&mut self, id: LookId, look: Look<'data>) -> Result<(), Error>;
    /// Chooses the default appearance; publication checks that it exists.
    fn set_default_look(&mut self, id: Option<LookId>) -> Result<(), Error>;
    /// Removes an appearance; publication checks remaining relationships.
    fn remove_look(&mut self, id: LookId) -> Option<Look<'data>>;
}

/// Exactly one authored material assignment, as supported by MDLX, M3, and WMO.
/// The look always exists; an empty binding vector represents an empty assignment.
#[derive(Clone, Debug)]
pub struct SingleLook<'data> {
    id: LookId,
    look: Look<'data>,
}
impl Default for SingleLook<'_> {
    fn default() -> Self {
        Self {
            id: Scope::default().allocate(),
            look: Look::default(),
        }
    }
}
impl ValidateLinks for SingleLook<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visit_look(&self.look, visitor);
    }
}
impl<'data> AppearanceSchema<'data> for SingleLook<'data> {
    fn looks(&self) -> Box<dyn ExactSizeIterator<Item = (LookId, &Look<'data>)> + '_> {
        Box::new(once((self.id, &self.look)))
    }
    fn look(&self, id: LookId) -> Option<&Look<'data>> {
        (id == self.id).then_some(&self.look)
    }
    fn default_look(&self) -> Option<LookId> {
        Some(self.id)
    }
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}
impl<'data> DrawAppearanceSchema<'data> for SingleLook<'data> {
    fn look_mut(&mut self, id: LookId) -> Option<&mut Look<'data>> {
        (id == self.id).then_some(&mut self.look)
    }
    fn add_look(&mut self, _id: LookId, _look: Look<'data>) -> Result<(), Error> {
        Err(Error::new(
            "appearance",
            "this profile always has exactly one material assignment",
        ))
    }
    fn set_default_look(&mut self, id: Option<LookId>) -> Result<(), Error> {
        if id != Some(self.id) {
            return Err(Error::new(
                "appearance.default",
                "the sole look is always the default",
            ));
        }
        Ok(())
    }
    fn remove_look(&mut self, _id: LookId) -> Option<Look<'data>> {
        None
    }
}

/// Named alternatives assigning materials directly to placement draws.
#[derive(Clone, Debug, Default)]
pub struct DrawLooks<'data> {
    looks: BTreeMap<LookId, Look<'data>>,
    default: Option<LookId>,
}
impl ValidateLinks for DrawLooks<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for look in self.looks.values() {
            visit_look(look, visitor);
        }
    }
}
impl<'data> AppearanceSchema<'data> for DrawLooks<'data> {
    fn looks(&self) -> Box<dyn ExactSizeIterator<Item = (LookId, &Look<'data>)> + '_> {
        Box::new(self.looks.iter().map(|(id, look)| (*id, look)))
    }
    fn look(&self, id: LookId) -> Option<&Look<'data>> {
        self.looks.get(&id)
    }
    fn default_look(&self) -> Option<LookId> {
        self.default
    }
    fn validate(&self) -> Result<(), Error> {
        if self.default.is_some_and(|id| self.look(id).is_none()) {
            Err(Error::new(
                "appearance.default",
                "default appearance is absent",
            ))
        } else {
            Ok(())
        }
    }
}
impl<'data> DrawAppearanceSchema<'data> for DrawLooks<'data> {
    fn look_mut(&mut self, id: LookId) -> Option<&mut Look<'data>> {
        self.looks.get_mut(&id)
    }
    fn add_look(&mut self, id: LookId, look: Look<'data>) -> Result<(), Error> {
        self.looks.insert(id, look);
        Ok(())
    }
    fn set_default_look(&mut self, id: Option<LookId>) -> Result<(), Error> {
        self.default = id;
        Ok(())
    }
    fn remove_look(&mut self, id: LookId) -> Option<Look<'data>> {
        self.looks.remove(&id)
    }
}

/// Appearance capabilities shared by the built-in profiles.
#[derive(Clone, Debug)]
pub enum UniversalAppearance<'data> {
    /// Diablo III material look alternatives.
    D3(diablo3::Appearance<'data>),
    /// Diablo IV hashed material look alternatives.
    D4(diablo4::Appearance),
    /// Draw-based material alternatives.
    Draw(DrawLooks<'data>),
    /// A single material assignment.
    Single(SingleLook<'data>),
}
impl Default for UniversalAppearance<'_> {
    fn default() -> Self {
        Self::Draw(DrawLooks::default())
    }
}
macro_rules! with_appearance {
    ($self:expr, $value:ident => $body:expr) => {
        match $self {
            UniversalAppearance::Draw($value) => $body,
            UniversalAppearance::Single($value) => $body,
            UniversalAppearance::D4($value) => $body,
            UniversalAppearance::D3($value) => $body,
        }
    };
}
impl ValidateLinks for UniversalAppearance<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        with_appearance!(self, value => value.visit_links(visitor));
    }
}
impl<'data> AppearanceSchema<'data> for UniversalAppearance<'data> {
    fn looks(&self) -> Box<dyn ExactSizeIterator<Item = (LookId, &Look<'data>)> + '_> {
        with_appearance!(self, value => value.looks())
    }
    fn look(&self, id: LookId) -> Option<&Look<'data>> {
        with_appearance!(self, value => value.look(id))
    }
    fn default_look(&self) -> Option<LookId> {
        with_appearance!(self, value => value.default_look())
    }
    fn contains_slot(&self, id: MaterialSlotId) -> bool {
        with_appearance!(self, value => value.contains_slot(id))
    }
    fn validate(&self) -> Result<(), Error> {
        with_appearance!(self, value => value.validate())
    }
}
impl<'data> DrawAppearanceSchema<'data> for UniversalAppearance<'data> {
    fn look_mut(&mut self, id: LookId) -> Option<&mut Look<'data>> {
        match self {
            Self::Draw(value) => value.look_mut(id),
            Self::Single(value) => value.look_mut(id),
            Self::D4(_) | Self::D3(_) => None,
        }
    }
    fn add_look(&mut self, id: LookId, look: Look<'data>) -> Result<(), Error> {
        match self {
            Self::Draw(value) => value.add_look(id, look),
            Self::Single(value) => value.add_look(id, look),
            Self::D4(_) | Self::D3(_) => Err(Error::new(
                "appearance.draw",
                "Diablo appearances assign material slots",
            )),
        }
    }
    fn set_default_look(&mut self, id: Option<LookId>) -> Result<(), Error> {
        match self {
            Self::Draw(value) => value.set_default_look(id),
            Self::Single(value) => value.set_default_look(id),
            Self::D4(_) | Self::D3(_) => Err(Error::new(
                "appearance.draw",
                "Diablo appearances select slot alternatives",
            )),
        }
    }
    fn remove_look(&mut self, id: LookId) -> Option<Look<'data>> {
        match self {
            Self::Draw(value) => value.remove_look(id),
            Self::Single(value) => value.remove_look(id),
            Self::D4(_) | Self::D3(_) => None,
        }
    }
}
impl<A: Default, B> Default for Either<A, B> {
    fn default() -> Self {
        Self::Left(A::default())
    }
}
impl<'data, A: AppearanceSchema<'data>, B: AppearanceSchema<'data>> AppearanceSchema<'data>
    for Either<A, B>
{
    fn looks(&self) -> Box<dyn ExactSizeIterator<Item = (LookId, &Look<'data>)> + '_> {
        match self {
            Self::Left(value) => value.looks(),
            Self::Right(value) => value.looks(),
        }
    }
    fn look(&self, id: LookId) -> Option<&Look<'data>> {
        match self {
            Self::Left(value) => value.look(id),
            Self::Right(value) => value.look(id),
        }
    }
    fn default_look(&self) -> Option<LookId> {
        match self {
            Self::Left(value) => value.default_look(),
            Self::Right(value) => value.default_look(),
        }
    }
    fn contains_slot(&self, id: MaterialSlotId) -> bool {
        match self {
            Self::Left(value) => value.contains_slot(id),
            Self::Right(value) => value.contains_slot(id),
        }
    }
    fn validate(&self) -> Result<(), Error> {
        match self {
            Self::Left(value) => value.validate(),
            Self::Right(value) => value.validate(),
        }
    }
}
impl<'data, A: DrawAppearanceSchema<'data>, B: DrawAppearanceSchema<'data>>
    DrawAppearanceSchema<'data> for Either<A, B>
{
    fn look_mut(&mut self, id: LookId) -> Option<&mut Look<'data>> {
        match self {
            Self::Left(value) => value.look_mut(id),
            Self::Right(value) => value.look_mut(id),
        }
    }
    fn add_look(&mut self, id: LookId, look: Look<'data>) -> Result<(), Error> {
        match self {
            Self::Left(value) => value.add_look(id, look),
            Self::Right(value) => value.add_look(id, look),
        }
    }
    fn set_default_look(&mut self, id: Option<LookId>) -> Result<(), Error> {
        match self {
            Self::Left(value) => value.set_default_look(id),
            Self::Right(value) => value.set_default_look(id),
        }
    }
    fn remove_look(&mut self, id: LookId) -> Option<Look<'data>> {
        match self {
            Self::Left(value) => value.remove_look(id),
            Self::Right(value) => value.remove_look(id),
        }
    }
}
fn visit_look(look: &Look<'_>, visitor: &mut dyn FnMut(Link)) {
    for binding in &look.bindings {
        binding.instance.visit_links(visitor);
        binding.material.visit_links(visitor);
    }
}
