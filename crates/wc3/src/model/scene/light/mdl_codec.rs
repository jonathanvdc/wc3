//! MDL accessors for version-selected light storage.
use super::{
    FalloffField, Light, LightFalloff, ShadowCastingField, ShadowIntensityField, ShadowRangeField,
};
use crate::model::mdl;
use crate::model::mdl::{ReadErrorKind, Span};
use crate::model::scene::{set_node_kind, validate_node_kind};
use crate::model::{Color, ModelVersion};
pub(super) fn zero() -> f32 {
    0.0
}
pub(super) fn white() -> Color {
    [1.0; 3]
}
pub(super) fn quadratic() -> f32 {
    LightFalloff::default().quadratic
}
pub(super) fn damping() -> f32 {
    LightFalloff::default().damping
}
pub(super) fn is_zero(value: &f32) -> bool {
    value.to_bits() == 0
}
fn require_supported(present: bool, supported: bool, span: Span) -> Result<(), mdl::ReadError> {
    if present && !supported {
        return Err(mdl::ReadError::new(span, ReadErrorKind::UnsupportedField));
    }
    Ok(())
}
impl<V: ModelVersion> Light<V> {
    pub(super) fn mdl_kind(&self) -> u32 {
        1u32.checked_shl(self.light_type).unwrap_or(0)
    }
    pub(super) fn set_mdl_kind(
        &mut self,
        value: u32,
        _: bool,
        span: Span,
    ) -> Result<(), mdl::ReadError> {
        if !value.is_power_of_two() {
            return Err(mdl::ReadError::new(
                span,
                ReadErrorKind::Expected("one light-type flag"),
            ));
        }
        self.light_type = value.trailing_zeros();
        Ok(())
    }
    pub(super) fn mdl_casting(&self) -> bool {
        self.shadow_casting.shadow_casting().unwrap_or(0) != 0
    }
    pub(super) fn set_mdl_casting(
        &mut self,
        value: bool,
        present: bool,
        span: Span,
    ) -> Result<(), mdl::ReadError> {
        require_supported(
            present,
            self.shadow_casting.shadow_casting().is_some(),
            span,
        )?;
        if let Some(target) = self.shadow_casting.shadow_casting_mut() {
            *target = u32::from(value);
        }
        Ok(())
    }
    pub(super) fn finish_mdl(&mut self, _: Span) -> Result<(), mdl::ReadError> {
        set_node_kind(&mut self.node, 0x200);
        Ok(())
    }
    pub(super) fn validate_mdl(&self) -> Result<(), mdl::WriteError> {
        validate_node_kind(&self.node, 0x200)?;
        if self.light_type > 2 {
            return Err(mdl::WriteError::Unsupported("light type"));
        }
        if self.shadow_casting.shadow_casting().unwrap_or(0) > 1 {
            return Err(mdl::WriteError::Unsupported("nonboolean shadow casting"));
        }
        Ok(())
    }
    pub(super) fn mdl_shadow_intensity(&self) -> Option<f32> {
        self.shadow_intensity.shadow_intensity()
    }
    pub(super) fn mdl_shadow_intensity_mut(&mut self) -> Option<&mut f32> {
        self.shadow_intensity.shadow_intensity_mut()
    }
    pub(super) fn mdl_shadow_start(&self) -> Option<f32> {
        self.shadow_range
            .shadow_casting_range()
            .map(|value| value.start)
    }
    pub(super) fn mdl_shadow_start_mut(&mut self) -> Option<&mut f32> {
        self.shadow_range
            .shadow_casting_range_mut()
            .map(|value| &mut value.start)
    }
    pub(super) fn mdl_shadow_end(&self) -> Option<f32> {
        self.shadow_range
            .shadow_casting_range()
            .map(|value| value.end)
    }
    pub(super) fn mdl_shadow_end_mut(&mut self) -> Option<&mut f32> {
        self.shadow_range
            .shadow_casting_range_mut()
            .map(|value| &mut value.end)
    }
    pub(super) fn mdl_quadratic(&self) -> Option<f32> {
        self.falloff.falloff().map(|value| value.quadratic)
    }
    pub(super) fn mdl_quadratic_mut(&mut self) -> Option<&mut f32> {
        self.falloff.falloff_mut().map(|value| &mut value.quadratic)
    }
    pub(super) fn mdl_linear(&self) -> Option<f32> {
        self.falloff.falloff().map(|value| value.linear)
    }
    pub(super) fn mdl_linear_mut(&mut self) -> Option<&mut f32> {
        self.falloff.falloff_mut().map(|value| &mut value.linear)
    }
    pub(super) fn mdl_damping(&self) -> Option<f32> {
        self.falloff.falloff().map(|value| value.damping)
    }
    pub(super) fn mdl_damping_mut(&mut self) -> Option<&mut f32> {
        self.falloff.falloff_mut().map(|value| &mut value.damping)
    }
}
