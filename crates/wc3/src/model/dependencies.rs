//! External declarations grouped by the kind of dependency they describe.
use super::profiles::{Profile, Universal};
use super::{
    BoneOverrideResourceObject, EffectResourceObject, ImageResourceObject, ModelResourceObject,
    PhysicsResourceObject, Table,
};

/// Model-owned external dependencies, with a separate typed table for each kind.
///
/// Sharing a declaration is explicit: consumers reuse its ID. Equal declarations
/// inserted separately retain independent identities and can be edited separately.
#[derive(Clone, Debug)]
pub struct Dependencies<'a, P: Profile = Universal> {
    pub(super) images: Table<ImageResourceObject, P::ImageResource<'a>>,
    pub(super) models: Table<ModelResourceObject, P::ModelResource<'a>>,
    pub(super) effects: Table<EffectResourceObject, P::EffectResource<'a>>,
    pub(super) physics: Table<PhysicsResourceObject, P::PhysicsResource<'a>>,
    pub(super) bone_overrides: Table<BoneOverrideResourceObject, P::BoneOverrideResource<'a>>,
}
impl<'a, P: Profile> Default for Dependencies<'a, P> {
    fn default() -> Self {
        Self {
            images: Table::default(),
            models: Table::default(),
            effects: Table::default(),
            physics: Table::default(),
            bone_overrides: Table::default(),
        }
    }
}
impl<'a, P: Profile> Dependencies<'a, P> {
    /// Image declarations in stable identity order.
    pub fn images(&self) -> &Table<ImageResourceObject, P::ImageResource<'a>> {
        &self.images
    }
    /// Model declarations in stable identity order.
    pub fn models(&self) -> &Table<ModelResourceObject, P::ModelResource<'a>> {
        &self.models
    }
    /// Effect declarations in stable identity order.
    pub fn effects(&self) -> &Table<EffectResourceObject, P::EffectResource<'a>> {
        &self.effects
    }
    /// Physics declarations in stable identity order.
    pub fn physics(&self) -> &Table<PhysicsResourceObject, P::PhysicsResource<'a>> {
        &self.physics
    }
    /// Bone override declarations in stable identity order.
    pub fn bone_overrides(
        &self,
    ) -> &Table<BoneOverrideResourceObject, P::BoneOverrideResource<'a>> {
        &self.bone_overrides
    }
}
