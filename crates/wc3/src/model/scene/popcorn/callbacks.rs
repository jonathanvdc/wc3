//! Checked numeric callback dispatch with invocation-owned generation capabilities.
use super::curve::sample;
use super::graph::*;
use super::payload::{Element as PayloadElement, Storage as PayloadStorage};
use super::runtime::{EffectLimits, EffectStats, FieldPlanes, RandomState, Spawn};
use super::spatial::{Search, Store as SpatialStore};
use super::turbulence::TemporalCache;
use super::{color, orientation, random, transforms};
use super::{Host, Value, ValueType};
use glam::{Mat4, Quat, Vec3};
use std::array::from_fn;
use std::collections::BinaryHeap;
use std::mem::size_of;

type Result<T> = std::result::Result<T, &'static str>;
#[derive(Clone, Copy)]
struct Occurrence {
    time: f64,
    fraction: f32,
    ordinal: u32,
}
#[derive(Clone, Copy)]
struct BoundElement {
    key: u32,
    batch: usize,
    element: PayloadElement,
}
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
    control: u32,
    omitted: bool,
}
struct SpatialDraft {
    key: u32,
    resource: usize,
    payload: Option<Vec3>,
    committed: bool,
}
pub(crate) struct Scratch {
    times: Vec<Occurrence>,
    elements: Vec<BoundElement>,
    payload_fields: Vec<Option<PayloadElement>>,
    payload_width: usize,
    batches: Vec<Batch>,
    drafts: Vec<Draft>,
    next: u32,
    spatial: Vec<SpatialDraft>,
    turbulence: TemporalCache,
}
impl Scratch {
    pub fn new(limits: EffectLimits, payload_width: usize, spatial: bool) -> Self {
        Self {
            times: Vec::with_capacity(limits.occurrences),
            elements: Vec::with_capacity(limits.handles),
            payload_fields: vec![None; limits.handles * payload_width],
            payload_width,
            batches: Vec::with_capacity(limits.handles),
            drafts: Vec::with_capacity(limits.handles),
            next: 0,
            spatial: Vec::with_capacity(if spatial { limits.handles } else { 0 }),
            turbulence: TemporalCache::default(),
        }
    }
    pub fn storage_bytes(
        limits: EffectLimits,
        payload_width: usize,
        spatial: bool,
    ) -> Option<usize> {
        limits
            .occurrences
            .checked_mul(size_of::<Occurrence>())?
            .checked_add(
                limits
                    .handles
                    .checked_mul(payload_width)?
                    .checked_mul(size_of::<Option<PayloadElement>>())?,
            )?
            .checked_add(limits.handles.checked_mul(
                size_of::<Batch>()
                    + size_of::<Draft>()
                    + size_of::<BoundElement>()
                    + if spatial {
                        size_of::<SpatialDraft>()
                    } else {
                        0
                    },
            )?)?
            .checked_add(size_of::<TemporalCache>())
    }
    pub fn clear(&mut self) {
        self.times.clear();
        self.elements.clear();
        self.batches.clear();
        self.drafts.clear();
        self.spatial.clear();
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
    pub spatial: &'a mut [SpatialStore],
    pub spatial_checks: &'a mut usize,
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
    pub inputs: &'a SceneInputs,
    pub payload_frame: Option<&'a Mat4>,
    pub payload: &'a mut PayloadStorage,
    pub attributes: &'a [Value],
    pub random: &'a mut RandomState,
    pub scratch: &'a mut Scratch,
    pub pending: &'a mut BinaryHeap<Spawn>,
    pub limits: &'a EffectLimits,
    pub stats: &'a mut EffectStats,
    pub order: &'a mut u64,
}
fn random_unit(policy: EffectRandom, random: &mut RandomState) -> f32 {
    match policy {
        EffectRandom::Midpoint => 0.5,
        EffectRandom::Counter => (random.next() >> 40) as f32 / 16_777_216.,
    }
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
    fn scene_context(&self, value: Value) -> Result<SceneContext> {
        if value.kind() != ValueType::Context {
            return Err("scene callback requires an opaque scene capability");
        }
        let slot = value.words()[0]
            .checked_sub(1)
            .and_then(|slot| u16::try_from(slot).ok())
            .ok_or("invalid scene context capability")?;
        if !matches!(self.external(slot)?, External::SceneContext) {
            return Err("scene context capability has the wrong ownership kind");
        }
        Ok(self.inputs.scene)
    }
    fn spatial_receiver(&self, receiver: Option<u16>) -> Result<usize> {
        let Some(receiver) = receiver else {
            return Err("spatial callback requires a bound resource");
        };
        let External::Spatial(resource) = self.external(receiver)? else {
            return Err("spatial callback receiver has the wrong resource kind");
        };
        if resource >= self.spatial.len() {
            return Err("spatial resource is outside the instance");
        }
        Ok(resource)
    }
    fn spatial_context(&self, value: Value) -> Result<()> {
        let slot = value.words()[0]
            .checked_sub(1)
            .and_then(|slot| u16::try_from(slot).ok())
            .ok_or("invalid spatial particle context capability")?;
        if value.kind() != ValueType::Context
            || !matches!(self.external(slot)?, External::ParticleContext)
        {
            return Err("spatial callback requires the current particle context");
        }
        Ok(())
    }
    fn spatial_draft(&mut self, resource: usize, key: u32) -> Result<&mut SpatialDraft> {
        let draft = self
            .scratch
            .spatial
            .iter_mut()
            .find(|draft| draft.key == key)
            .ok_or("spatial draft is not owned by this invocation")?;
        if draft.resource != resource || draft.committed {
            return Err("spatial draft belongs to another resource or was already inserted");
        }
        Ok(draft)
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
            for (ordinal, time) in times[end - kept..end].iter().enumerate() {
                let time = self.birth + f64::from(*time);
                let fraction = if self.delta > 0. {
                    ((time - (self.time - self.delta)) / self.delta).clamp(0., 1.) as f32
                } else {
                    0.
                };
                self.scratch.times.push(Occurrence {
                    time,
                    fraction,
                    ordinal: total + (count - kept + ordinal) as u32,
                });
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
                self.scratch.times.push(Occurrence {
                    time: self.time - self.delta + self.delta * f64::from(fraction),
                    fraction,
                    ordinal: total + index,
                });
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
            | External::SceneContext
            | External::Spatial(_)
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
            Callback::SpatialAllocate => {
                let resource = self.spatial_receiver(receiver)?;
                self.spatial_context(a[0])?;
                let key = if self.scratch.spatial.len() >= self.limits.handles {
                    0
                } else {
                    let key = self.scratch.key()?;
                    self.scratch.spatial.push(SpatialDraft {
                        key,
                        resource,
                        payload: None,
                        committed: false,
                    });
                    key
                };
                Some(Value::from(key as i32))
            }
            Callback::SpatialAppend => {
                let resource = self.spatial_receiver(receiver)?;
                self.spatial_context(a[3])?;
                if a[2].words()[0] != 0 {
                    return Err("unsupported spatial payload slot");
                }
                let key = a[0].words()[0];
                if key != 0 {
                    self.spatial_draft(resource, key)?.payload = Some(vector(a[1]));
                }
                Some(Value::from(key as i32))
            }
            Callback::SpatialInsert => {
                let resource = self.spatial_receiver(receiver)?;
                self.spatial_context(a[2])?;
                let key = a[0].words()[0];
                let inserted = if key == 0 {
                    false
                } else {
                    let draft = self.spatial_draft(resource, key)?;
                    let payload = draft
                        .payload
                        .ok_or("spatial draft has no named payload value")?;
                    draft.committed = true;
                    self.spatial[resource].insert(vector(a[1]), payload)
                };
                if !inserted {
                    self.stats.omitted_spatial_writes =
                        self.stats.omitted_spatial_writes.saturating_add(1);
                }
                None
            }
            Callback::SpatialClosest => {
                let resource = self.spatial_receiver(receiver)?;
                self.scene_context(a[6])?;
                if a[2..6].iter().any(|value| value.words()[0] != 0) {
                    return Err("spatial queries support only rank zero, zero options and payload slot zero");
                }
                let found = self.spatial[resource].closest(
                    vector(a[0]),
                    number(a[1]),
                    self.spatial_checks,
                )?;
                let value = match found {
                    Search::Found(payload) => payload,
                    Search::Missing => Vec3::splat(f32::INFINITY),
                    Search::Omitted => {
                        self.stats.omitted_spatial_queries =
                            self.stats.omitted_spatial_queries.saturating_add(1);
                        Vec3::splat(f32::INFINITY)
                    }
                };
                Some(Value::from(value))
            }
            Callback::ViewPosition => {
                let scene = self.scene_context(a[1])?;
                let position = scene
                    .camera_position(a[0].words()[0] as i32)
                    .unwrap_or(Vec3::splat(f32::INFINITY));
                Some(Value::from(position))
            }
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
                    let fraction = fraction.clamp(0., 1.);
                    self.scratch.times.push(Occurrence {
                        time: self.time - self.delta + self.delta * f64::from(fraction),
                        fraction,
                        ordinal: 0,
                    });
                    0
                } else {
                    u64::from(count)
                };
                Some(self.batch(count, 0., start, omitted)?)
            }
            Callback::InitPayload
            | Callback::BuildElement
            | Callback::Append
            | Callback::Extract(_)
            | Callback::Kick => self.payload_call(callback, receiver, a)?,
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
            Callback::VRand => {
                let radii = if a.len() == 1 {
                    self.context(a[0], true)?;
                    None
                } else {
                    self.context(a[2], true)?;
                    Some((number(a[0]), number(a[1])))
                };
                let first = random_unit(self.graph.random, self.random);
                let second = random_unit(self.graph.random, self.random);
                let third = if radii.is_some() {
                    random_unit(self.graph.random, self.random)
                } else {
                    0.
                };
                Some(Value::from(random::vector([first, second, third], radii)?))
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
                    let u = random_unit(self.graph.random, self.random);
                    let value = a + (b - a) * u;
                    if !value.is_finite() {
                        return Err("random interval result is not finite");
                    }
                    words[index] = value.to_bits();
                }
                Some(Value::from_words(kind, words))
            }
            Callback::Transform
            | Callback::TransformDirection
            | Callback::InverseTransform
            | Callback::InverseDirection => {
                self.context(a[2], false)?;
                let direction = matches!(
                    callback,
                    Callback::TransformDirection | Callback::InverseDirection
                );
                let point = if matches!(
                    callback,
                    Callback::InverseTransform | Callback::InverseDirection
                ) {
                    transforms::inverse(
                        self.inputs.world_from_effect,
                        self.payload_frame.copied(),
                        vector(a[0]),
                        a[1].words()[0],
                        direction,
                    )?
                } else {
                    transforms::forward(
                        self.inputs.world_from_effect,
                        self.payload_frame.copied(),
                        vector(a[0]),
                        a[1].words()[0],
                        direction,
                    )?
                };
                Some(Value::from(point))
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
                    self.inputs.world_from_effect,
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
                let draws = from_fn(|_| random_unit(self.graph.random, self.random));
                let position = shape.sample(draws)?;
                Some(Value::from(position))
            }
            Callback::TurbulenceSample => {
                self.scene_context(a[1])?;
                let External::Sampler(index) =
                    self.external(receiver.ok_or("turbulence sample requires a bound receiver")?)?
                else {
                    return Err("turbulence receiver is not a sampler");
                };
                let EffectSampler::Turbulence(resource) = &self.graph.samplers[index] else {
                    return Err("turbulence receiver has the wrong resource family");
                };
                let phase = self.scratch.turbulence.phase(index, resource, self.time)?;
                Some(Value::from(resource.sample(vector(a[0]), phase)?))
            }
            Callback::Sample => {
                let External::Sampler(index) =
                    self.external(receiver.ok_or("curve sample requires a bound receiver")?)?
                else {
                    return Err("curve receiver is not a sampler");
                };
                Some(sample(&self.graph.samplers[index], number(a[0]))?)
            }
        })
    }
}

#[path = "callbacks/payload.rs"]
mod payload_dispatch;
