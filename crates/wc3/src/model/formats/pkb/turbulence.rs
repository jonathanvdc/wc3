//! Borrowed procedural/external turbulence declarations without implicit defaults.
use super::{Cursor, Document, ReadError};

/// Sparse native turbulence parameters; omissions remain distinct from defaults.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TurbulenceSampler<'a> {
    /// Localized editor names.
    pub names: Vec<(u32, &'a [u8])>,
    /// Localized editor descriptions.
    pub descriptions: Vec<(u32, &'a [u8])>,
    /// Authored editor activation, without an inferred runtime gate.
    pub active: Option<bool>,
    /// Compiler execution-stage metadata.
    pub execution_stage: Option<u32>,
    /// Compiler execution-frequency metadata.
    pub execution_frequency: Option<u32>,
    /// Editor input links.
    pub input_pins: Vec<u32>,
    /// Editor output links.
    pub output_pins: Vec<u32>,
    /// Editor position.
    pub workspace: Option<[i32; 2]>,
    /// Editor rollup state.
    pub rollup: Option<i32>,
    /// Zero procedural, one external field.
    pub data_source: Option<u32>,
    /// External vector-field resource name.
    pub vector_field: Option<&'a [u8]>,
    /// Spatial and velocity amplitude multiplier.
    pub global_scale: Option<f32>,
    /// Velocity strength.
    pub strength: Option<f32>,
    /// External-field spatial/time addressing controls.
    pub wrapping: [Option<bool>; 4],
    /// External-field filtering selector.
    pub filtering: Option<i32>,
    /// Initial spatial wavelength.
    pub wavelength: Option<f32>,
    /// Authored octave count.
    pub octaves: Option<u32>,
    /// Wavelength ratio between octaves.
    pub lacunarity: Option<f32>,
    /// Amplitude ratio between octaves.
    pub gain: Option<f32>,
    /// Procedural interpolation selector.
    pub interpolator: Option<i32>,
    /// Effect-global time multiplier.
    pub time_scale: Option<f32>,
    /// Temporal phase offset.
    pub time_base: Option<f32>,
    /// Per-corner temporal-rate variation.
    pub time_variation: Option<f32>,
    /// Retained unsupported flow control.
    pub flow_factor: Option<f32>,
    /// Retained unsupported divergence control.
    pub divergence_factor: Option<f32>,
    /// Numeric integer seed, never a float reinterpretation.
    pub initial_seed: Option<u32>,
    /// Retained unsupported flow approximation control.
    pub fast_fake_flow: Option<bool>,
    /// Additional octave amplitude ratio.
    pub gain_multiplier: Option<f32>,
}
impl<'a> Document<'a> {
    /// Decode turbulence fields without preparing a field or assigning sparse defaults.
    pub fn turbulence_sampler(&self, link: u32) -> Result<TurbulenceSampler<'a>, ReadError> {
        let object = self.declaration(link, b"CParticleNodeSamplerData_Turbulence")?;
        let mut result = TurbulenceSampler::default();
        self.fields(object, |field, input| {
            let boolean = |input: &mut Cursor<'_>| match input.byte()? {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(input.error("native turbulence boolean is neither zero nor one")),
            };
            match field {
                0 | 1 => {
                    let target = if field == 0 {
                        &mut result.names
                    } else {
                        &mut result.descriptions
                    };
                    let count = input.count(8)?;
                    target.reserve(count);
                    for _ in 0..count {
                        target.push((input.word()?, self.string(input)?));
                    }
                }
                2 => result.active = Some(boolean(input)?),
                3 => result.execution_stage = Some(input.word()?),
                4 => result.execution_frequency = Some(input.word()?),
                5 => result.input_pins = self.links(input)?,
                6 => result.output_pins = self.links(input)?,
                7 => result.workspace = Some([input.word()? as i32, input.word()? as i32]),
                8 => result.rollup = Some(input.word()? as i32),
                9 => result.data_source = Some(input.word()?),
                10 => result.vector_field = Some(self.string(input)?),
                11 => result.global_scale = Some(f32::from_bits(input.word()?)),
                12 => result.strength = Some(f32::from_bits(input.word()?)),
                13..=16 => result.wrapping[(field - 13) as usize] = Some(boolean(input)?),
                17 => result.filtering = Some(input.word()? as i32),
                18 => result.wavelength = Some(f32::from_bits(input.word()?)),
                19 => result.octaves = Some(input.word()?),
                20 => result.lacunarity = Some(f32::from_bits(input.word()?)),
                21 => result.gain = Some(f32::from_bits(input.word()?)),
                22 => result.interpolator = Some(input.word()? as i32),
                23 => result.time_scale = Some(f32::from_bits(input.word()?)),
                24 => result.time_base = Some(f32::from_bits(input.word()?)),
                25 => result.time_variation = Some(f32::from_bits(input.word()?)),
                26 => result.flow_factor = Some(f32::from_bits(input.word()?)),
                27 => result.divergence_factor = Some(f32::from_bits(input.word()?)),
                28 => result.initial_seed = Some(input.word()?),
                29 => result.fast_fake_flow = Some(boolean(input)?),
                30 => result.gain_multiplier = Some(f32::from_bits(input.word()?)),
                _ => return Err(input.error("unsupported native turbulence field")),
            }
            Ok(())
        })?;
        Ok(result)
    }
}
