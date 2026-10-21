//! Checked sparse defaults for the original procedural turbulence interpreter.
use super::{error, Document, EffectCompileError};
use crate::model::scene::popcorn::turbulence::{Parameters, Turbulence};

pub(super) fn prepare(
    document: &Document<'_>,
    link: u32,
) -> Result<Turbulence, EffectCompileError> {
    let native = document.turbulence_sampler(link)?;
    if native.data_source.unwrap_or(0) != 0 {
        return Err(error(
            "external PKVF or unknown turbulence data source is unsupported",
        ));
    }
    if native.flow_factor.is_some_and(|value| value != 0.)
        || native.divergence_factor.is_some_and(|value| value != 0.)
        || native.fast_fake_flow == Some(true)
    {
        return Err(error(
            "authored turbulence flow/divergence/fake-flow controls are unsupported",
        ));
    }
    let defaults = Parameters::default();
    Turbulence::new(Parameters {
        scale: native.global_scale.unwrap_or(defaults.scale),
        strength: native.strength.unwrap_or(defaults.strength),
        wavelength: native.wavelength.unwrap_or(defaults.wavelength),
        octaves: native.octaves.unwrap_or(defaults.octaves),
        lacunarity: native.lacunarity.unwrap_or(defaults.lacunarity),
        gain: native.gain.unwrap_or(defaults.gain),
        gain_multiplier: native.gain_multiplier.unwrap_or(defaults.gain_multiplier),
        interpolator: native.interpolator.unwrap_or(defaults.interpolator),
        time_scale: native.time_scale.unwrap_or(defaults.time_scale),
        time_base: native.time_base.unwrap_or(defaults.time_base),
        time_variation: native.time_variation.unwrap_or(defaults.time_variation),
        seed: native.initial_seed.unwrap_or(defaults.seed),
    })
    .map_err(error)
}
#[cfg(test)]
#[path = "effect_turbulence_tests.rs"]
mod tests;
