//! Checked numeric callback dispatch with invocation-owned generation capabilities.
use super::graph::*;
use super::runtime::{EffectLimits, EffectStats, FieldPlanes, RandomState, Spawn};
use super::{color, orientation};
use super::{Host, Value, ValueType};
use glam::{Mat4, Quat, Vec3};
use std::array::from_fn;
use std::collections::BinaryHeap;
use std::mem::size_of;

type Result<T> = std::result::Result<T, &'static str>;
#[derive(Clone, Copy)]
struct Batch {
    key: u32,
    start: usize,
    end: usize,
    omitted: u64,
}
#[derive(Clone, Copy)]
struct Draft {
    key: u32,
    batch: usize,
    event: usize,
    committed: bool,
}
pub(crate) struct Scratch {
    times: Vec<f64>,
    batches: Vec<Batch>,
    drafts: Vec<Draft>,
    next: u32,
}
impl Scratch {
    pub fn new(limits: EffectLimits) -> Self {
        Self {
            times: Vec::with_capacity(limits.occurrences),
            batches: Vec::with_capacity(limits.handles),
            drafts: Vec::with_capacity(limits.handles),
            next: 0,
        }
    }
    pub fn storage_bytes(limits: EffectLimits) -> Option<usize> {
        limits.occurrences.checked_mul(8)?.checked_add(
            limits
                .handles
                .checked_mul(size_of::<Batch>() + size_of::<Draft>())?,
        )
    }
    pub fn clear(&mut self) {
        self.times.clear();
        self.batches.clear();
        self.drafts.clear();
    }
    fn key(&mut self) -> Result<u32> {
        self.next = self
            .next
            .checked_add(1)
            .ok_or("effect invocation capability space exhausted")?;
        Ok(self.next)
    }
}
pub(crate) struct ExecutionHost<'a> {
    pub graph: &'a EffectGraph,
    pub bound: &'a BoundProgram,
    pub fields: &'a mut FieldPlanes,
    pub slot: usize,
    pub id: EffectParticleId,
    pub layer: EffectLayerId,
    pub birth: f64,
    pub time: f64,
    pub delta: f64,
    pub running: bool,
    pub emitting: bool,
    pub world: Mat4,
    pub attributes: &'a [Value],
    pub random: &'a mut RandomState,
    pub scratch: &'a mut Scratch,
    pub pending: &'a mut BinaryHeap<Spawn>,
    pub limits: EffectLimits,
    pub stats: &'a mut EffectStats,
    pub order: &'a mut u64,
}
fn scalar(value: f32) -> Result<Value> {
    if !value.is_finite() {
        return Err("effect callback produced a nonfinite scalar");
    }
    Ok(Value::from_words(
        ValueType::Float,
        [value.to_bits(), 0, 0, 0],
    ))
}
fn number(value: Value) -> f32 {
    f32::from_bits(value.words()[0])
}
fn vector(value: Value) -> Vec3 {
    let [x, y, z, _] = value.words().map(f32::from_bits);
    Vec3::new(x, y, z)
}
fn quaternion(value: Value) -> Quat {
    Quat::from_array(value.words().map(f32::from_bits))
}
impl ExecutionHost<'_> {
    fn external(&self, slot: u16) -> Result<External> {
        self.bound
            .externals
            .get(usize::from(slot))
            .copied()
            .ok_or("effect external binding is outside the compiled table")
    }
    fn context(&self, value: Value, random: bool) -> Result<()> {
        let slot = value.words()[0]
            .checked_sub(1)
            .and_then(|slot| u16::try_from(slot).ok())
            .ok_or("invalid effect context capability")?;
        match (self.external(slot)?, random) {
            (External::RandomContext, true)
            | (External::ParticleContext | External::EffectContext, false) => Ok(()),
            _ => Err("effect context capability has the wrong ownership kind"),
        }
    }
    fn batch(&mut self, total: u32, residual: f32, start: usize, omitted: u64) -> Result<Value> {
        if self.scratch.batches.len() >= self.limits.handles {
            return Err("effect generation batch capacity exhausted");
        }
        let key = self.scratch.key()?;
        self.scratch.batches.push(Batch {
            key,
            start,
            end: self.scratch.times.len(),
            omitted,
        });
        Ok(Value::from_words(
            ValueType::Int3,
            [residual.to_bits(), total, key, 0],
        ))
    }
    fn accumulator(&self, value: Value) -> Result<(f32, u32)> {
        let words = value.words();
        let residual = f32::from_bits(words[0]);
        if !residual.is_finite() || !(0.0..1.0).contains(&residual) || words[1] > i32::MAX as u32 {
            return Err("generation accumulator is outside its supported finite domain");
        }
        Ok((residual, words[1]))
    }
    fn generate(&mut self, receiver: Option<u16>, a: &[Value]) -> Result<Value> {
        self.context(a[5], false)?;
        let (residual, total) = self.accumulator(a[0])?;
        let start = self.scratch.times.len();
        if !self.running {
            return self.batch(total, residual, start, 0);
        }
        if let Some(receiver) = receiver {
            let External::Sampler(index) = self.external(receiver)? else {
                return Err("bound generation requires an event-stream sampler");
            };
            let EffectSampler::Events(times) = &self.graph.samplers[index] else {
                return Err("bound generation sampler is not an event stream");
            };
            let previous = number(a[1]);
            let current = number(a[2]);
            if !previous.is_finite() || !current.is_finite() || previous < 0. || current < previous
            {
                return Err("event-stream interval must be finite, nonnegative and ordered");
            }
            let first = times.partition_point(|time| *time < previous);
            let end = times.partition_point(|time| *time < current);
            let count = end - first;
            let next = total
                .checked_add(
                    u32::try_from(count).map_err(|_| "event count exceeds native accumulator")?,
                )
                .filter(|next| *next <= i32::MAX as u32)
                .ok_or("event cumulative count exceeds native accumulator")?;
            let kept = count.min(self.limits.occurrences - self.scratch.times.len());
            for time in &times[end - kept..end] {
                self.scratch.times.push(self.birth + f64::from(*time));
            }
            self.batch(next, 0., start, (count - kept) as u64)
        } else {
            if number(a[3]) != 0. || number(a[4]) != 1. {
                return Err(
                    "free generation supports only the observed zero/one interpolation controls",
                );
            }
            let numerator = number(a[1]);
            let denominator = number(a[2]);
            if !numerator.is_finite()
                || !denominator.is_finite()
                || numerator < 0.
                || denominator < 0.
            {
                return Err("free generation advancement must be finite and nonnegative");
            }
            let advancement = if denominator > 1e-12 {
                numerator / denominator
            } else {
                0.
            };
            let sum = residual + advancement;
            if !sum.is_finite() || sum >= i32::MAX as f32 {
                return Err("free generation count exceeds native accumulator");
            }
            let count = sum.trunc() as u32;
            let next = total
                .checked_add(count)
                .filter(|next| *next <= i32::MAX as u32)
                .ok_or("free generation cumulative count exceeds native accumulator")?;
            let kept = (count as usize).min(self.limits.occurrences - self.scratch.times.len());
            for index in count - kept as u32..count {
                let fraction = if advancement > 0. {
                    ((index as f32 + 1. - residual) / advancement).clamp(0., 1.)
                } else {
                    1.
                };
                self.scratch
                    .times
                    .push(self.time - self.delta + self.delta * f64::from(fraction));
            }
            self.batch(
                next,
                sum - count as f32,
                start,
                u64::from(count - kept as u32),
            )
        }
    }
}
impl Host for ExecutionHost<'_> {
    type Error = &'static str;
    fn load(&mut self, external: u16) -> Result<Value> {
        Ok(match self.external(external)? {
            External::Field(field) => {
                let kind = self.graph.layers[self.layer.0 as usize].fields[field]
                    .value_type
                    .ok_or("opaque metadata cannot become a VM value")?;
                Value::from_words(kind, self.fields.read(field, self.slot))
            }
            External::Time => return scalar(self.time as f32),
            External::Delta => return scalar(self.delta as f32),
            External::Attribute(index) => self.attributes[index],
            External::ParticleContext
            | External::EffectContext
            | External::RandomContext
            | External::Sampler(_)
            | External::Event(_) => {
                Value::from_words(ValueType::Context, [u32::from(external) + 1, 0, 0, 0])
            }
        })
    }
    fn store(&mut self, external: u16, value: Value) -> Result<()> {
        let External::Field(field) = self.external(external)? else {
            return Err("effect programme writes to a read-only external capability");
        };
        if self.graph.layers[self.layer.0 as usize].fields[field].value_type != Some(value.kind()) {
            return Err("canonical particle field type mismatch");
        }
        self.fields.write(field, self.slot, value.words());
        Ok(())
    }
    fn call(
        &mut self,
        function: u16,
        receiver: Option<u16>,
        a: &[Value],
        _: &[u8],
        _: u8,
    ) -> Result<Option<Value>> {
        let callback = *self
            .bound
            .callbacks
            .get(function as usize)
            .ok_or("effect callback binding is outside its table")?;
        Ok(match callback {
            Callback::HsvToRgb | Callback::RgbToHsv => Some(Value::from_words(
                ValueType::Float4,
                color::convert(
                    a[0].words().map(f32::from_bits),
                    callback == Callback::HsvToRgb,
                )?
                .map(f32::to_bits),
            )),
            Callback::Age => {
                self.context(a[0], false)?;
                Some(scalar((self.time - self.birth).max(0.) as f32)?)
            }
            Callback::Running => {
                self.context(a[0], false)?;
                Some(Value::from_words(
                    ValueType::Bool,
                    [if self.running { u32::MAX } else { 0 }, 0, 0, 0],
                ))
            }
            Callback::StreamDuration => {
                self.context(a[0], false)?;
                let External::Sampler(index) =
                    self.external(receiver.ok_or("stream duration requires a bound receiver")?)?
                else {
                    return Err("duration receiver is not a sampler");
                };
                let EffectSampler::Events(times) = &self.graph.samplers[index] else {
                    return Err("duration receiver is not an event stream");
                };
                Some(scalar(match self.graph.duration {
                    EventStreamDuration::LastOccurrence => times.last().copied().unwrap_or(0.),
                    EventStreamDuration::Explicit(duration) => duration,
                })?)
            }
            Callback::Generate => Some(self.generate(receiver, a)?),
            Callback::Trigger => {
                self.context(a[2], false)?;
                let fraction = number(a[1]);
                if !fraction.is_finite() {
                    return Err("trigger fraction must be finite");
                }
                let count = u32::from(self.running && a[0].words()[0] != 0);
                let start = self.scratch.times.len();
                let omitted = if count != 0 && start < self.limits.occurrences {
                    self.scratch.times.push(
                        self.time - self.delta + self.delta * f64::from(fraction.clamp(0., 1.)),
                    );
                    0
                } else {
                    u64::from(count)
                };
                Some(self.batch(count, 0., start, omitted)?)
            }
            Callback::InitPayload => {
                self.context(a[2], false)?;
                if a[0].words()[0] != 0 {
                    return Err("unsupported native payload initialization control");
                }
                let External::Event(event) = self.external(
                    receiver.ok_or("payload initialization requires an event receiver")?,
                )?
                else {
                    return Err("payload initialization receiver is not an event");
                };
                let batch = self
                    .scratch
                    .batches
                    .iter()
                    .position(|batch| batch.key == a[1].words()[2])
                    .ok_or(
                        "generation batch capability is stale or belongs to another invocation",
                    )?;
                if self.scratch.drafts.len() >= self.limits.handles {
                    return Err("effect payload draft capacity exhausted");
                }
                let key = self.scratch.key()?;
                self.scratch.drafts.push(Draft {
                    key,
                    batch,
                    event,
                    committed: false,
                });
                Some(Value::from_words(ValueType::Int, [key, 0, 0, 0]))
            }
            Callback::Kick => {
                self.context(a[1], true)?;
                self.context(a[2], false)?;
                let External::Event(event) =
                    self.external(receiver.ok_or("event kick requires a bound receiver")?)?
                else {
                    return Err("event kick receiver is not an event");
                };
                let draft = self
                    .scratch
                    .drafts
                    .iter_mut()
                    .find(|draft| draft.key == a[0].words()[0])
                    .ok_or("payload draft capability is stale or belongs to another invocation")?;
                if draft.event != event || draft.committed {
                    return Err("payload draft belongs to another event or was already committed");
                }
                draft.committed = true;
                let batch = self.scratch.batches[draft.batch];
                if self.running {
                    let targets = &self.graph.events[event].targets;
                    self.stats.omitted_births = self
                        .stats
                        .omitted_births
                        .saturating_add(batch.omitted.saturating_mul(targets.len() as u64));
                    self.random.counter = self.random.counter.wrapping_add(batch.omitted);
                    for time in &self.scratch.times[batch.start..batch.end] {
                        let seed = self.random.next();
                        for layer in targets {
                            if !self.emitting && self.graph.layers[layer.0 as usize].rendered {
                                self.stats.suppressed_births =
                                    self.stats.suppressed_births.saturating_add(1);
                                continue;
                            }
                            if self.pending.len() >= self.limits.pending_events {
                                self.stats.omitted_births =
                                    self.stats.omitted_births.saturating_add(1);
                            } else {
                                self.pending.push(Spawn {
                                    time: *time,
                                    layer: *layer,
                                    parent: Some(self.id),
                                    seed,
                                    order: *self.order,
                                });
                                *self.order = self.order.wrapping_add(1);
                            }
                        }
                    }
                }
                None
            }
            Callback::Kill => {
                let condition = if a.len() == 1 {
                    self.context(a[0], false)?;
                    true
                } else {
                    self.context(a[1], false)?;
                    a[0].words()[0] != 0
                };
                if condition {
                    let life = self.graph.layers[self.layer.0 as usize].life;
                    self.fields
                        .write(life, self.slot, [1.0f32.to_bits(), 0, 0, 0]);
                }
                None
            }
            Callback::Rand => {
                self.context(a[2], true)?;
                let kind = a[0].kind();
                let left = a[0].words();
                let right = a[1].words();
                let mut words = [0; 4];
                for index in 0..kind.components() {
                    let a = f32::from_bits(left[index]);
                    let b = f32::from_bits(right[index]);
                    if !a.is_finite() || !b.is_finite() {
                        return Err("random interval endpoints must be finite");
                    }
                    let u = match self.graph.random {
                        EffectRandom::Midpoint => 0.5,
                        EffectRandom::Counter => (self.random.next() >> 40) as f32 / 16_777_216.,
                    };
                    let value = a + (b - a) * u;
                    if !value.is_finite() {
                        return Err("random interval result is not finite");
                    }
                    words[index] = value.to_bits();
                }
                Some(Value::from_words(kind, words))
            }
            Callback::Transform | Callback::TransformDirection => {
                self.context(a[2], false)?;
                let words = a[0].words();
                let point = transform(
                    self.world,
                    Vec3::new(
                        f32::from_bits(words[0]),
                        f32::from_bits(words[1]),
                        f32::from_bits(words[2]),
                    ),
                    a[1].words()[0],
                    callback == Callback::TransformDirection,
                )?;
                Some(Value::from_words(
                    ValueType::Float3,
                    [point.x.to_bits(), point.y.to_bits(), point.z.to_bits(), 0],
                ))
            }
            Callback::Orientation => Some(Value::from(orientation::from_forward(vector(a[0]))?)),
            Callback::OrientationSide => {
                Some(Value::from(orientation::rotate(Vec3::X, quaternion(a[0]))?))
            }
            Callback::Rotate => Some(Value::from(orientation::rotate(
                vector(a[0]),
                quaternion(a[1]),
            )?)),
            Callback::RotateAbout => Some(Value::from(orientation::rotate_about(
                vector(a[0]),
                vector(a[1]),
                number(a[2]),
            )?)),
            Callback::EffectSide | Callback::EffectUp => {
                self.context(a[0], false)?;
                Some(Value::from(orientation::emitter_axis(
                    self.world,
                    callback == Callback::EffectUp,
                )?))
            }
            Callback::SamplePosition => {
                self.context(a[0], true)?;
                let External::Sampler(index) =
                    self.external(receiver.ok_or("shape sample requires a bound receiver")?)?
                else {
                    return Err("shape receiver is not a sampler");
                };
                let EffectSampler::Shape(shape) = &self.graph.samplers[index] else {
                    return Err("samplePosition receiver is not a shape");
                };
                let draws = from_fn(|_| match self.graph.random {
                    EffectRandom::Midpoint => 0.5,
                    EffectRandom::Counter => (self.random.next() >> 40) as f32 / 16_777_216.,
                });
                let position = shape.sample(draws)?;
                Some(Value::from(position))
            }
            Callback::Sample => {
                let External::Sampler(index) =
                    self.external(receiver.ok_or("curve sample requires a bound receiver")?)?
                else {
                    return Err("curve receiver is not a sampler");
                };
                let EffectSampler::Curve {
                    kind,
                    times,
                    values,
                    tangents,
                } = &self.graph.samplers[index]
                else {
                    return Err("sample receiver is not a curve");
                };
                let time = number(a[0]);
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
                Some(Value::from_words(*kind, result.map(f32::to_bits)))
            }
        })
    }
}

fn transform(world: Mat4, value: Vec3, mask: u32, direction: bool) -> Result<Vec3> {
    let transformed = match (direction, mask) {
        (false, 17) => value + world.w_axis.truncate(),
        (false, 19) => world.transform_point3(value),
        (true, 18) => world.transform_vector3(value),
        _ => return Err("unsupported native local-to-world transform mask"),
    };
    if !transformed.is_finite() {
        return Err("effect transform produced nonfinite components");
    }
    Ok(transformed)
}

#[cfg(test)]
#[path = "callback_tests.rs"]
mod tests;
