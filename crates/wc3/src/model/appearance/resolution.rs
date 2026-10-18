//! Resolve complete draw material assignments under an authored look.
use crate::model::profiles::Universal;
use crate::model::{DrawId, Error, InstanceId, LookId, MaterialId, Model};

/// An authored look used to resolve a draw's material.
#[derive(Clone, Copy, Debug, Default)]
pub enum MaterialSelection {
    /// Use the authored default look.
    #[default]
    Default,
    /// Select a complete draw-based look.
    Draw(LookId),
}

impl Model<'_, Universal> {
    /// Resolves a draw's material, rejecting absent or foreign identities.
    pub fn resolve_material(
        &self,
        instance: InstanceId,
        draw: DrawId,
        selection: MaterialSelection,
    ) -> Result<MaterialId, Error> {
        let look = match selection {
            MaterialSelection::Default => None,
            MaterialSelection::Draw(look) => Some(look),
        };
        self.material_for_draw(instance, draw, look)
    }
}
