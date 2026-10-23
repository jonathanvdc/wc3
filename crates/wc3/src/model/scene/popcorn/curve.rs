//! Shared checked immutable curve evaluation.
use super::graph::EffectSampler;
use super::Value;
#[inline(always)]
pub(super) fn sample(sampler: &EffectSampler, time: f32) -> Result<Value, &'static str> {
    let EffectSampler::Curve {
        kind,
        times,
        values,
        tangents,
    } = sampler
    else {
        return Err("sample receiver is not a curve");
    };
    if !time.is_finite() {
        return Err("curve sample coordinate must be finite");
    }
    let index = times.partition_point(|key| *key <= time).saturating_sub(1);
    let mut result = values[index];
    if time <= times[0] {
        result = values[0];
    } else if index + 1 < times.len() {
        let t = (time - times[index]) / (times[index + 1] - times[index]);
        for component in 0..kind.components() {
            result[component] = if tangents.is_empty() {
                values[index][component]
                    + (values[index + 1][component] - values[index][component]) * t
            } else {
                let t2 = t * t;
                let t3 = t2 * t;
                (2. * t3 - 3. * t2 + 1.) * values[index][component]
                    + (t3 - 2. * t2 + t) * tangents[index * 2 + 1][component]
                    + (-2. * t3 + 3. * t2) * values[index + 1][component]
                    + (t3 - t2) * tangents[(index + 1) * 2][component]
            };
        }
    }
    Ok(Value::from_words(*kind, result.map(f32::to_bits)))
}
