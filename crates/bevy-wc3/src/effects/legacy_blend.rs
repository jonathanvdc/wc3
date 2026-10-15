//! Adapters for legacy record-driven callers of the shared effect transports.
use super::blend::EffectBlend;
use wc3::model::formats::mdlx::emitters::Particle2FilterMode;
use wc3::model::formats::mdlx::materials::LayerFilterMode;
impl From<Particle2FilterMode> for EffectBlend {
    fn from(value: Particle2FilterMode) -> Self {
        match value {
            Particle2FilterMode::Additive => Self::AdditiveAlpha,
            Particle2FilterMode::Modulate => Self::Modulate,
            Particle2FilterMode::Modulate2x => Self::Modulate2x,
            Particle2FilterMode::AlphaKey => Self::Cutout,
            _ => Self::Alpha,
        }
    }
}
impl From<LayerFilterMode> for EffectBlend {
    fn from(value: LayerFilterMode) -> Self {
        match value {
            LayerFilterMode::None => Self::Opaque,
            LayerFilterMode::Transparent => Self::Cutout,
            LayerFilterMode::Blend => Self::Alpha,
            LayerFilterMode::Additive | LayerFilterMode::AddAlpha => Self::AdditiveAlpha,
            LayerFilterMode::Modulate => Self::Modulate,
            LayerFilterMode::Modulate2x => Self::Modulate2x,
            LayerFilterMode::Unknown(_) => Self::Alpha,
        }
    }
}
