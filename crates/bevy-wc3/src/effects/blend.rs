//! Render operations shared by immutable sprite and ribbon transports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum EffectBlend {
    Opaque,
    Cutout,
    Alpha,
    Additive,
    AdditiveAlpha,
    Modulate,
    Modulate2x,
}
