//! Appearance vocabularies preserve the assignments supported by their model family.
use crate::model::semantic::profiles::Either;
use crate::model::semantic::{Error, Link, Look, LookId, MaterialSlotId, ValidateLinks};
use std::collections::BTreeMap;
use std::fmt::Debug;

/// A complete appearance vocabulary, including its own material assignment domains.
pub trait AppearanceSchema<'data>: Clone + Debug + Default + Send + Sync + ValidateLinks {
    /// Draw overrides exposed by this vocabulary, when it uses draw-based appearances.
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
/// An appearance vocabulary that supports direct draw-material overrides.
pub trait DrawAppearanceSchema<'data>: AppearanceSchema<'data> {
    /// Adds a draw-based appearance with an identity supplied by its owning graph.
    fn add_look(&mut self, id: LookId, look: Look<'data>) -> Result<(), Error>;
    /// Chooses the default appearance; publication checks that it exists.
    fn set_default_look(&mut self, id: Option<LookId>) -> Result<(), Error>;
    /// Removes an appearance; publication checks remaining relationships.
    fn remove_look(&mut self, id: LookId) -> Option<Look<'data>>;
}

/// One authored material assignment, as supported by MDLX, M3, and WMO.
#[derive(Clone, Debug, Default)]
pub struct SingleLook<'data> {
    look: Option<(LookId, Look<'data>)>,
    default: Option<LookId>,
}
impl ValidateLinks for SingleLook<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Some((_, look)) = &self.look {
            visit_look(look, visitor);
        }
    }
}
impl<'data> AppearanceSchema<'data> for SingleLook<'data> {
    fn looks(&self) -> Box<dyn ExactSizeIterator<Item = (LookId, &Look<'data>)> + '_> {
        Box::new(self.look.iter().map(|(id, look)| (*id, look)))
    }
    fn look(&self, id: LookId) -> Option<&Look<'data>> {
        self.look
            .as_ref()
            .filter(|(key, _)| *key == id)
            .map(|(_, look)| look)
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
impl<'data> DrawAppearanceSchema<'data> for SingleLook<'data> {
    fn add_look(&mut self, id: LookId, look: Look<'data>) -> Result<(), Error> {
        if self.look.is_some() {
            return Err(Error::new(
                "appearance",
                "this profile permits one material assignment",
            ));
        }
        self.look = Some((id, look));
        Ok(())
    }
    fn set_default_look(&mut self, id: Option<LookId>) -> Result<(), Error> {
        self.default = id;
        Ok(())
    }
    fn remove_look(&mut self, id: LookId) -> Option<Look<'data>> {
        if self.look(id).is_some() {
            self.look.take().map(|(_, look)| look)
        } else {
            None
        }
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

/// Built-in appearance families, retaining their original authoring domains on widening.
#[derive(Clone, Debug)]
pub enum UniversalAppearance<'data> {
    /// General draw-based material alternatives.
    Flexible(DrawLooks<'data>),
    /// Warcraft III's single material assignment.
    Mdlx(SingleLook<'data>),
    /// World of Warcraft draw alternatives.
    M2(DrawLooks<'data>),
    /// StarCraft II's single material assignment.
    M3(SingleLook<'data>),
    /// Static-world material assignment.
    Wmo(SingleLook<'data>),
    /// Editable-model named slots and family-local alternatives.
    Wem(crate::model::semantic::profiles::wem::Appearance<'data>),
}
impl Default for UniversalAppearance<'_> {
    fn default() -> Self {
        Self::Flexible(DrawLooks::default())
    }
}
macro_rules! with_appearance {
    ($self:expr, $value:ident => $body:expr) => {
        match $self {
            UniversalAppearance::Flexible($value) | UniversalAppearance::M2($value) => $body,
            UniversalAppearance::Mdlx($value)
            | UniversalAppearance::M3($value)
            | UniversalAppearance::Wmo($value) => $body,
            UniversalAppearance::Wem($value) => $body,
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
    fn add_look(&mut self, id: LookId, look: Look<'data>) -> Result<(), Error> {
        match self {
            Self::Flexible(value) | Self::M2(value) => value.add_look(id, look),
            Self::Mdlx(value) | Self::M3(value) | Self::Wmo(value) => value.add_look(id, look),
            Self::Wem(_) => Err(Error::new(
                "appearance.draw",
                "editable-model appearances assign named polygon-group slots",
            )),
        }
    }
    fn set_default_look(&mut self, id: Option<LookId>) -> Result<(), Error> {
        match self {
            Self::Flexible(value) | Self::M2(value) => value.set_default_look(id),
            Self::Mdlx(value) | Self::M3(value) | Self::Wmo(value) => value.set_default_look(id),
            Self::Wem(_) => Err(Error::new(
                "appearance.draw",
                "editable-model appearances select family-local slot alternatives",
            )),
        }
    }
    fn remove_look(&mut self, id: LookId) -> Option<Look<'data>> {
        match self {
            Self::Flexible(value) | Self::M2(value) => value.remove_look(id),
            Self::Mdlx(value) | Self::M3(value) | Self::Wmo(value) => value.remove_look(id),
            Self::Wem(_) => None,
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
