//! Native named sampler bindings and borrowed curve/event-stream payloads.
use super::{Document, ReadError};
use crate::model::common::PackedArray;

/// A named sampler declaration retaining its independently typed object link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedSampler<'a> {
    /// Native symbol used by script binding resolution.
    pub name: &'a [u8],
    /// One-based native sampler object link, including unsupported families.
    pub sampler: u32,
    /// Lossless native family selector; this is not a spatial-layer index.
    pub sampler_type: i32,
    /// Native usage controls, with sparse zero default.
    pub usage: u32,
}
/// Borrowed native curve arrays; component and interpolation policy belongs to lowering.
#[derive(Clone, Debug, PartialEq)]
pub struct CurveSampler<'a> {
    /// Localized editor names as native locale identifiers and string bytes.
    pub names: Vec<(u32, &'a [u8])>,
    /// Optional editor workspace coordinates, without simulation meaning.
    pub workspace: Option<[i32; 2]>,
    /// Authored component/type selector; omission remains distinct from an explicit value.
    pub value_type: Option<u32>,
    /// Authored interpolation selector; omission is preserved.
    pub interpolator: Option<u32>,
    /// Authored sampling quality selector; omission is preserved.
    pub quality: Option<u32>,
    /// Native key times, borrowing their encoded source.
    pub times: PackedArray<'a, f32>,
    /// Key-major native component values, without an inferred component width.
    pub values: PackedArray<'a, f32>,
    /// Native tangent components, empty when omitted.
    pub tangents: PackedArray<'a, f32>,
}
/// Borrowed native event occurrence times, without interval-boundary policy.
#[derive(Clone, Debug, PartialEq)]
pub struct EventStreamSampler<'a> {
    /// Native authored occurrence times.
    pub times: PackedArray<'a, f32>,
}
impl<'a> Document<'a> {
    /// Read a named sampler while retaining unknown payload families as raw object links.
    pub fn named_sampler(&self, link: u32) -> Result<NamedSampler<'a>, ReadError> {
        let object = self.declaration(link, b"CLayerCompileCacheSampler")?;
        let (mut name, mut sampler, mut sampler_type) = (None, None, None);
        let mut usage = 0;
        self.fields(object, |field, input| {
            match field {
                0 => name = Some(self.string(input)?),
                1 => sampler = Some(self.required_link(input)?),
                3 => sampler_type = Some(input.word()? as i32),
                4 => usage = input.word()?,
                _ => return Err(input.error("unsupported named-sampler field")),
            }
            Ok(())
        })?;
        Ok(NamedSampler {
            name: self.required(object, name)?,
            sampler: self.required(object, sampler)?,
            sampler_type: self.required(object, sampler_type)?,
            usage,
        })
    }
    /// Read a curve without replacing sparse type declarations with inferred dimensions.
    pub fn curve_sampler(&self, link: u32) -> Result<CurveSampler<'a>, ReadError> {
        let object = self.declaration(link, b"CParticleNodeSamplerData_Curve")?;
        let mut names = Vec::new();
        let (mut workspace, mut value_type, mut interpolator, mut quality) =
            (None, None, None, None);
        let (mut times, mut values, mut tangents) = (None, None, PackedArray::new());
        self.fields(object, |field, input| {
            match field {
                0 => {
                    let count = input.count(8)?;
                    names.reserve(count);
                    for _ in 0..count {
                        names.push((input.word()?, self.string(input)?));
                    }
                }
                7 => workspace = Some([input.word()? as i32, input.word()? as i32]),
                9 => value_type = Some(input.word()?),
                10 => interpolator = Some(input.word()?),
                14 => quality = Some(input.word()?),
                16 => times = Some(self.packed(input, 4)?),
                17 => values = Some(self.packed(input, 4)?),
                18 => tangents = self.packed(input, 4)?,
                _ => return Err(input.error("unsupported curve-sampler field")),
            }
            Ok(())
        })?;
        Ok(CurveSampler {
            names,
            workspace,
            value_type,
            interpolator,
            quality,
            times: self.required(object, times)?,
            values: self.required(object, values)?,
            tangents,
        })
    }
    /// Read an event stream independently of graph execution and scheduling policy.
    pub fn event_stream_sampler(&self, link: u32) -> Result<EventStreamSampler<'a>, ReadError> {
        let object = self.declaration(link, b"CParticleNodeSamplerData_EventStream")?;
        let mut times = None;
        self.fields(object, |field, input| {
            match field {
                10 => times = Some(self.packed(input, 4)?),
                _ => return Err(input.error("unsupported event-stream sampler field")),
            }
            Ok(())
        })?;
        Ok(EventStreamSampler {
            times: self.required(object, times)?,
        })
    }
}
