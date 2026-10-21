//! M2 opacity selects Bevy's phase independently of authored depth operations.
use super::{Blend, ModelMaterialExtension};
use bevy::prelude::*;

const VISIBLE_OPACITY: f32 = 0.0001;
const OPAQUE_OPACITY: f32 = 0.99999;

/// Authored classification survives fades so returning to full opacity restores it.
#[derive(Clone, Copy, Debug)]
pub(super) struct M2Opacity {
    mode: AlphaMode,
    blend: Option<Blend>,
}

impl M2Opacity {
    pub(super) fn new(mode: AlphaMode, blend: Option<Blend>) -> Self {
        Self { mode, blend }
    }

    fn has_opacity(self, opacity: f32) -> bool {
        opacity.is_finite()
            && (self.blend == Some(Blend::Premultiplied) || opacity >= VISIBLE_OPACITY)
    }

    pub(super) fn apply(
        self,
        material: &mut StandardMaterial,
        state: &mut ModelMaterialExtension,
    ) -> bool {
        let opacity = material.base_color.alpha();
        let faded =
            opacity < OPAQUE_OPACITY && matches!(self.mode, AlphaMode::Opaque | AlphaMode::Mask(_));
        let cutoff = if !self.has_opacity(opacity) {
            f32::MAX
        } else if let AlphaMode::Mask(cutoff) = self.mode {
            cutoff * opacity
        } else if self.mode == AlphaMode::Opaque || self.blend == Some(Blend::Premultiplied) {
            0.
        } else {
            1. / 255.
        };
        let mode = if faded {
            AlphaMode::Blend
        } else if matches!(self.mode, AlphaMode::Mask(_)) {
            AlphaMode::Mask(cutoff)
        } else {
            self.mode
        };
        let blend = if faded {
            Some(Blend::Alpha)
        } else {
            self.blend
        };
        let changed =
            material.alpha_mode != mode || state.blend != blend || state.combiner.clip.x != cutoff;
        material.alpha_mode = mode;
        state.blend = blend;
        state.combiner.clip.x = cutoff;
        changed
    }
}

impl ModelMaterialExtension {
    pub(crate) fn apply_opacity(&mut self, material: &mut StandardMaterial) -> bool {
        self.m2_opacity
            .is_some_and(|policy| policy.apply(material, self))
    }

    pub(crate) fn has_visible_opacity(&self, opacity: f32) -> bool {
        self.m2_opacity
            .map_or(opacity > 0., |policy| policy.has_opacity(opacity))
    }
}

#[cfg(test)]
#[path = "opacity_tests.rs"]
mod tests;
