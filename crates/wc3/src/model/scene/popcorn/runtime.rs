//! Bounded reusable lifecycle storage and deterministic event-time scheduling.
use super::callbacks::{ExecutionHost, Scratch};
use super::graph::*;
use super::lifetime::Lifetime;
use super::payload::{Handle as PayloadHandle, Storage as PayloadStorage};
use super::{Failure, State, Value, ValueType};
use glam::Mat4;
use std::array::from_fn;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::iter::once;
use std::mem::size_of;
use std::sync::Arc;

/// Hard capacities allocated before an effect starts; omitted work never becomes a backlog.
#[derive(Clone, Copy, Debug)]
pub struct EffectLimits {
    /// Total reusable particle slots, including renderer-free controllers.
    pub particles: usize,
    /// Maximum pending routed births.
    pub pending_events: usize,
    /// Maximum generated occurrence times retained by one host invocation.
    pub occurrences: usize,
    /// Maximum generation batches and payload drafts per invocation.
    pub handles: usize,
    /// Maximum particle executions and births per update.
    pub work_per_update: usize,
    /// Maximum ordinary simulation step in seconds.
    pub step_seconds: f64,
    /// Maximum simulation steps examined per update.
    pub steps_per_update: usize,
    /// Maximum predicted allocated storage in bytes.
    pub storage_bytes: usize,
    /// Maximum payload component words copied or interpolated per update.
    pub payload_words_per_update: usize,
}
impl Default for EffectLimits {
    fn default() -> Self {
        Self {
            particles: 1024,
            pending_events: 8192,
            occurrences: 8192,
            handles: 64,
            work_per_update: 262_144,
            step_seconds: 1. / 60.,
            steps_per_update: 4096,
            storage_bytes: 128 * 1024 * 1024,
            payload_words_per_update: 1_048_576,
        }
    }
}
/// Observable cumulative bounded-work accounting.
#[derive(Clone, Copy, Debug, Default)]
pub struct EffectStats {
    /// Successfully initialized particles, including controllers.
    pub spawned: u64,
    /// Retired particles, including explicit stop-boundary spawners.
    pub retired: u64,
    /// Births omitted by pool, queue, generation or execution budgets.
    pub omitted_births: u64,
    /// Particle executions omitted by the per-update work budget.
    pub omitted_executions: u64,
    /// Renderer births suppressed by the reversible emission gate.
    pub suppressed_births: u64,
    /// Simulation time omitted by the step budget, without deferred catch-up.
    pub skipped_seconds: f64,
}
/// Explicit runtime or callback failure; unsupported behavior is never replaced by zero.
#[derive(Clone, Debug, PartialEq)]
pub struct EffectRuntimeError {
    /// Owning layer when failure occurred during execution.
    pub layer: Option<EffectLayerId>,
    /// Instruction byte position for scalar interpreter failures.
    pub offset: Option<usize>,
    /// Constraint or unsupported operation that stopped the instance.
    pub message: &'static str,
}
impl Display for EffectRuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "particle effect {:?} at {:?}: {}",
            self.layer, self.offset, self.message
        )
    }
}
impl Error for EffectRuntimeError {}
pub(crate) fn failure(message: &'static str) -> EffectRuntimeError {
    EffectRuntimeError {
        layer: None,
        offset: None,
        message,
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct RandomState {
    pub seed: u64,
    pub counter: u64,
}
impl RandomState {
    pub fn next(&mut self) -> u64 {
        let value = mix(self.seed.wrapping_add(self.counter));
        self.counter = self.counter.wrapping_add(1);
        value
    }
}
pub(crate) fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}
pub(crate) struct FieldPlanes {
    planes: Vec<[Vec<u32>; 4]>,
}
impl FieldPlanes {
    fn new(widths: &[usize], count: usize) -> Self {
        Self {
            planes: widths
                .iter()
                .map(|width| {
                    from_fn(|component| {
                        if component < *width {
                            vec![0; count]
                        } else {
                            Vec::new()
                        }
                    })
                })
                .collect(),
        }
    }
    pub fn read(&self, field: usize, slot: usize) -> [u32; 4] {
        from_fn(|component| {
            self.planes[field][component]
                .get(slot)
                .copied()
                .unwrap_or(0)
        })
    }
    pub fn write(&mut self, field: usize, slot: usize, words: [u32; 4]) {
        for (plane, word) in self.planes[field].iter_mut().zip(words) {
            if let Some(value) = plane.get_mut(slot) {
                *value = word;
            }
        }
    }
}
struct Particle {
    state: State,
    layer: EffectLayerId,
    generation: u32,
    birth: f64,
    cursor: f64,
    random: RandomState,
    parent: Option<EffectParticleId>,
    payload_frame: Option<Mat4>,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Spawn {
    pub time: f64,
    pub layer: EffectLayerId,
    pub parent: Option<EffectParticleId>,
    pub seed: u64,
    pub order: u64,
    pub payload: Option<PayloadHandle>,
    pub payload_event: usize,
    pub payload_target: usize,
    pub payload_fraction: f32,
    pub payload_ordinal: u32,
}
impl PartialEq for Spawn {
    fn eq(&self, other: &Self) -> bool {
        self.order == other.order && self.time.to_bits() == other.time.to_bits()
    }
}
impl Eq for Spawn {}
impl PartialOrd for Spawn {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Spawn {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .time
            .total_cmp(&self.time)
            .then_with(|| other.order.cmp(&self.order))
    }
}
/// Borrowed particle values; iteration does not construct transient extraction arrays.
pub struct ParticleView<'a> {
    /// Slot and incarnation of the living particle.
    pub id: EffectParticleId,
    /// Owning effect layer.
    pub layer: EffectLayerId,
    /// Absolute effect-local birth time in seconds.
    pub birth_time: f64,
    /// Parent identity retained independently of slot reuse.
    pub parent: Option<EffectParticleId>,
    fields: &'a FieldPlanes,
    types: &'a [Option<ValueType>],
}
impl ParticleView<'_> {
    /// Read a canonical field with its checked type; opaque metadata returns None.
    pub fn field(&self, field: EffectFieldId) -> Option<Value> {
        let kind = *self.types.get(field.0 as usize)?;
        Some(Value::from_words(
            kind?,
            self.fields.read(field.0 as usize, self.id.slot as usize),
        ))
    }
}
/// Reusable pooled state; callers supply the matching shared programme explicitly.
pub struct EffectInstance {
    identity: Arc<()>,
    layout: Vec<Box<[Option<ValueType>]>>,
    particles: Vec<Particle>,
    active: Vec<usize>,
    free: Vec<usize>,
    fields: FieldPlanes,
    pending: BinaryHeap<Spawn>,
    scratch: Scratch,
    payload: PayloadStorage,
    vm_scratch: State,
    limits: EffectLimits,
    stats: EffectStats,
    attributes: Vec<Value>,
    time: f64,
    running: bool,
    emitting: bool,
    faulted: bool,
    seed: u64,
    order: u64,
}
impl EffectProgram {
    /// Allocate all reusable particle, field, event and callback storage before simulation.
    pub fn instantiate(
        &self,
        limits: EffectLimits,
        seed: u64,
    ) -> Result<EffectInstance, EffectRuntimeError> {
        if limits.particles == 0
            || limits.particles > u32::MAX as usize
            || limits.pending_events == 0
            || limits.occurrences == 0
            || limits.handles == 0
            || limits.work_per_update == 0
            || limits.payload_words_per_update == 0
            || !limits.step_seconds.is_finite()
            || limits.step_seconds <= 0.
            || limits.steps_per_update == 0
        {
            return Err(failure(
                "effect capacities and step controls must be finite and positive",
            ));
        }
        let programs: Vec<_> = self
            .inner
            .layers
            .iter()
            .flat_map(|layer| {
                once(&layer.init.program).chain(layer.evolve.iter().map(|bound| &bound.program))
            })
            .collect();
        let mut counts = [0usize; 4];
        let mut arguments = 0;
        for program in &programs {
            for (count, required) in counts.iter_mut().zip(program.counts) {
                *count = (*count).max(required);
            }
            arguments = arguments.max(program.max_arguments);
        }
        let max_fields = self
            .inner
            .layers
            .iter()
            .map(|layer| layer.fields.len())
            .max()
            .unwrap_or(0);
        let mut widths = vec![0usize; max_fields];
        for layer in &self.inner.layers {
            for (width, field) in widths.iter_mut().zip(&layer.fields) {
                *width = (*width).max(field.value_type.map_or(1, ValueType::components));
            }
        }
        let payload_width = if self.inner.layers.iter().any(|layer| {
            Some(&layer.init)
                .into_iter()
                .chain(layer.evolve.as_ref())
                .any(|bound| {
                    bound.callbacks.iter().any(|callback| {
                        matches!(
                            callback,
                            Callback::BuildElement | Callback::Append | Callback::Extract(_)
                        )
                    })
                })
        }) {
            self.inner
                .layers
                .iter()
                .map(|layer| layer.payload.fields.len())
                .chain(
                    self.inner
                        .events
                        .iter()
                        .map(|event| event.payload.fields.len()),
                )
                .max()
                .unwrap_or(0)
        } else {
            0
        };
        let words = counts[2..]
            .iter()
            .sum::<usize>()
            .checked_mul(16)
            .and_then(|bytes| {
                bytes.checked_add(widths.iter().sum::<usize>() * 4 + size_of::<Particle>())
            });
        let bytes = words
            .and_then(|bytes| bytes.checked_mul(limits.particles))
            .and_then(|bytes| {
                bytes.checked_add((counts[0] + counts[1]) * 16 + arguments * size_of::<Value>())
            })
            .and_then(|bytes| {
                bytes.checked_add(limits.pending_events.checked_mul(size_of::<Spawn>())?)
            })
            .and_then(|bytes| bytes.checked_add(Scratch::storage_bytes(limits, payload_width)?))
            .and_then(|bytes| {
                bytes.checked_add(limits.particles.checked_mul(2 * size_of::<usize>())?)
            });
        let bytes = bytes.and_then(|bytes| {
            bytes.checked_add(PayloadStorage::bytes(
                limits.particles,
                limits.pending_events,
                payload_width,
            )?)
        });
        if bytes.is_none_or(|bytes| bytes > limits.storage_bytes) {
            return Err(failure("effect pool exceeds its configured storage budget"));
        }
        let particles = (0..limits.particles)
            .map(|_| Particle {
                state: State::persistent_for_programs(programs.iter().copied()),
                layer: EffectLayerId(0),
                generation: 0,
                birth: 0.,
                cursor: 0.,
                random: RandomState {
                    seed: 0,
                    counter: 0,
                },
                parent: None,
                payload_frame: None,
            })
            .collect();
        Ok(EffectInstance {
            identity: self.inner.identity.clone(),
            layout: self
                .inner
                .layers
                .iter()
                .map(|layer| layer.fields.iter().map(|field| field.value_type).collect())
                .collect(),
            particles,
            active: Vec::with_capacity(limits.particles),
            free: (0..limits.particles).rev().collect(),
            fields: FieldPlanes::new(&widths, limits.particles),
            pending: BinaryHeap::with_capacity(limits.pending_events),
            scratch: Scratch::new(limits, payload_width),
            payload: PayloadStorage::new(limits.particles, limits.pending_events, payload_width)
                .map_err(failure)?,
            vm_scratch: State::scratch_for_programs(programs.iter().copied()),
            limits,
            stats: EffectStats::default(),
            attributes: self
                .inner
                .attributes
                .iter()
                .map(|attribute| attribute.default)
                .collect(),
            time: 0.,
            running: false,
            emitting: true,
            faulted: false,
            seed,
            order: 0,
        })
    }
}
impl EffectInstance {
    fn compatible(&self, program: &EffectProgram) -> Result<(), EffectRuntimeError> {
        if !Arc::ptr_eq(&self.identity, &program.inner.identity) {
            Err(failure("instance and effect programme identities differ"))
        } else if self.faulted {
            Err(failure(
                "effect instance stopped after an execution failure; restart it explicitly",
            ))
        } else {
            Ok(())
        }
    }
    /// Restart at effect-local time zero, retaining all allocated capacities.
    /// Selected entry targets and authored no-parent controllers initialize once.
    pub fn start(
        &mut self,
        program: &EffectProgram,
        entry: u32,
        inputs: SceneInputs,
    ) -> Result<(), EffectRuntimeError> {
        if !Arc::ptr_eq(&self.identity, &program.inner.identity) {
            return Err(failure("instance and effect programme identities differ"));
        }
        let entry = program
            .inner
            .entries
            .get(entry as usize)
            .ok_or_else(|| failure("effect entry index is outside the programme"))?;
        if !inputs.world_from_effect.is_finite() {
            return Err(failure("effect transform must be finite"));
        }
        self.active.clear();
        self.free.clear();
        self.free.extend((0..self.particles.len()).rev());
        self.pending.clear();
        self.payload.clear();
        self.time = 0.;
        self.running = true;
        self.emitting = true;
        self.faulted = false;
        self.order = 0;
        self.stats = EffectStats::default();
        for target in &program.inner.events[entry.event].targets {
            self.enqueue(Spawn {
                time: 0.,
                layer: *target,
                parent: None,
                seed: mix(self.seed ^ u64::from(target.0)),
                order: self.order,
                payload: None,
                payload_event: 0,
                payload_target: 0,
                payload_fraction: 0.,
                payload_ordinal: 0,
            });
        }
        for (index, layer) in program.inner.layers.iter().enumerate() {
            if layer.standalone
                && !program.inner.events[entry.event]
                    .targets
                    .contains(&EffectLayerId(index as u32))
            {
                self.enqueue(Spawn {
                    time: 0.,
                    layer: EffectLayerId(index as u32),
                    parent: None,
                    seed: mix(self.seed ^ index as u64),
                    order: self.order,
                    payload: None,
                    payload_event: 0,
                    payload_target: 0,
                    payload_fraction: 0.,
                    payload_ordinal: 0,
                });
            }
        }
        let mut work = 0;
        while let Some(spawn) = self.pending.pop() {
            if let Err(error) = self.birth(program, spawn, 0., inputs, &mut work) {
                self.faulted = true;
                self.running = false;
                return Err(error);
            }
        }
        Ok(())
    }
    fn enqueue(&mut self, mut spawn: Spawn) {
        if self.pending.len() == self.limits.pending_events {
            self.stats.omitted_births = self.stats.omitted_births.saturating_add(1);
            return;
        }
        spawn.order = self.order;
        self.order = self.order.wrapping_add(1);
        self.pending.push(spawn);
    }
    /// Disable future generation while living visible particles and authored controllers finish.
    pub fn stop(&mut self) {
        self.running = false;
    }
    /// Reversibly permit renderer-bearing births. Structural controllers keep
    /// advancing their authored accumulators, so hidden intervals cannot accumulate backlog.
    pub fn set_emitting(&mut self, emitting: bool) {
        self.emitting = emitting;
    }
    /// Whether renderer-bearing births are currently permitted by both lifecycle gates.
    pub fn is_emitting(&self) -> bool {
        self.running && self.emitting
    }

    /// Effect-local elapsed time, including explicitly omitted simulation intervals.
    pub fn time(&self) -> f64 {
        self.time
    }
    /// Whether new authored generation is enabled.
    pub fn is_running(&self) -> bool {
        self.running
    }
    /// Whether no particles or pending events remain after stopping.
    pub fn is_finished(&self) -> bool {
        !self.running && self.active.is_empty() && self.pending.is_empty()
    }
    /// Living particles, including renderer-free controllers and generators.
    pub fn active_count(&self) -> usize {
        self.active.len()
    }
    /// Cumulative bounded-work counters.
    pub fn stats(&self) -> &EffectStats {
        &self.stats
    }
    /// Change one resolved exposed control without native name lookup during execution.
    pub fn set_attribute(
        &mut self,
        id: EffectAttributeId,
        value: Value,
    ) -> Result<(), EffectRuntimeError> {
        let target = self
            .attributes
            .get_mut(id.0 as usize)
            .ok_or_else(|| failure("attribute index is outside the effect"))?;
        if target.kind() != value.kind() {
            return Err(failure(
                "attribute value layout does not match its declaration",
            ));
        }
        *target = value;
        Ok(())
    }
    /// Borrow living canonical values for one layer, without transient extraction buffers.
    pub fn particles(&self, layer: EffectLayerId) -> impl Iterator<Item = ParticleView<'_>> {
        self.active
            .iter()
            .copied()
            .filter(move |slot| self.particles[*slot].layer == layer)
            .map(move |slot| {
                let particle = &self.particles[slot];
                ParticleView {
                    id: EffectParticleId {
                        slot: slot as u32,
                        generation: particle.generation,
                    },
                    layer,
                    birth_time: particle.birth,
                    parent: particle.parent,
                    fields: &self.fields,
                    types: &self.layout[layer.0 as usize],
                }
            })
    }
    /// Advance in bounded steps. Births retain event timestamps and evolve only their remaining interval.
    pub fn advance(
        &mut self,
        program: &EffectProgram,
        delta: f64,
        inputs: SceneInputs,
    ) -> Result<(), EffectRuntimeError> {
        self.compatible(program)?;
        if !delta.is_finite()
            || delta < 0.
            || !inputs.world_from_effect.is_finite()
            || !(self.time + delta).is_finite()
        {
            return Err(failure(
                "effect delta and transform must be finite with nonnegative elapsed time",
            ));
        }
        if delta == 0. {
            return Ok(());
        }
        self.payload.reset_work();
        let end = self.time + delta;
        let mut steps = 0;
        let mut work = 0;
        while self.time < end && steps < self.limits.steps_per_update {
            let next = (self.time + self.limits.step_seconds).min(end);
            if next <= self.time {
                break;
            }
            if let Err(error) = self.step(program, next, inputs, &mut work) {
                self.faulted = true;
                self.running = false;
                return Err(error);
            }
            self.time = next;
            steps += 1;
        }
        if self.time < end {
            self.stats.skipped_seconds += end - self.time;
            self.time = end;
            let mut index = 0;
            while index < self.active.len() {
                let slot = self.active[index];
                let particle = &mut self.particles[slot];
                let layer = &program.inner.layers[particle.layer.0 as usize];
                let lifetime = Lifetime::read(
                    f32::from_bits(self.fields.read(layer.life, slot)[0]),
                    f32::from_bits(self.fields.read(layer.inverse_life, slot)[0]),
                )
                .map_err(|message| EffectRuntimeError {
                    layer: Some(particle.layer),
                    offset: None,
                    message,
                })?;
                let life = lifetime.advance(end - particle.cursor);
                particle.cursor = end;
                self.fields
                    .write(layer.life, slot, [life.to_bits(), 0, 0, 0]);
                self.stats.omitted_executions = self.stats.omitted_executions.saturating_add(1);
                if life >= 1.0 {
                    self.retire_at(index);
                } else {
                    index += 1;
                }
            }
        }
        Ok(())
    }
    fn retire_at(&mut self, index: usize) {
        let slot = self.active.swap_remove(index);
        self.free.push(slot);
        self.stats.retired = self.stats.retired.saturating_add(1);
    }
    fn step(
        &mut self,
        program: &EffectProgram,
        end: f64,
        inputs: SceneInputs,
        work: &mut usize,
    ) -> Result<(), EffectRuntimeError> {
        let mut index = 0;
        while index < self.active.len() {
            let slot = self.active[index];
            let layer = &program.inner.layers[self.particles[slot].layer.0 as usize];
            if !self.running && layer.spawner {
                self.retire_at(index);
                continue;
            }
            let dead = self.evolve(program, slot, end, inputs, work)?;
            if dead {
                self.retire_at(index);
            } else {
                index += 1;
            }
        }
        while let Some(spawn) = self.pending.pop() {
            if *work >= self.limits.work_per_update {
                self.stats.omitted_births = self
                    .stats
                    .omitted_births
                    .saturating_add(1 + self.pending.len() as u64);
                if let Some(handle) = spawn.payload {
                    self.payload.arena.release(handle).map_err(failure)?;
                }
                while let Some(spawn) = self.pending.pop() {
                    if let Some(handle) = spawn.payload {
                        self.payload.arena.release(handle).map_err(failure)?;
                    }
                }
                break;
            }
            self.birth(program, spawn, end, inputs, work)?;
        }
        Ok(())
    }
    fn birth(
        &mut self,
        program: &EffectProgram,
        mut spawn: Spawn,
        end: f64,
        inputs: SceneInputs,
        work: &mut usize,
    ) -> Result<(), EffectRuntimeError> {
        let result = self.birth_inner(program, &mut spawn, end, inputs, work);
        if let Some(handle) = spawn.payload {
            self.payload.arena.release(handle).map_err(failure)?;
        }
        result
    }
    fn birth_inner(
        &mut self,
        program: &EffectProgram,
        spawn: &mut Spawn,
        end: f64,
        inputs: SceneInputs,
        work: &mut usize,
    ) -> Result<(), EffectRuntimeError> {
        if *work >= self.limits.work_per_update {
            self.stats.omitted_births += 1;
            return Ok(());
        }
        let payload_words = self
            .payload
            .birth_words(
                spawn.payload,
                program.inner.layers[spawn.layer.0 as usize]
                    .payload
                    .fields
                    .len(),
            )
            .map_err(failure)?;
        if !self
            .payload
            .charge(payload_words, self.limits.payload_words_per_update)
        {
            self.stats.omitted_births = self.stats.omitted_births.saturating_add(1);
            return Ok(());
        }
        let Some(slot) = self.free.pop() else {
            self.stats.omitted_births += 1;
            return Ok(());
        };
        let layer = &program.inner.layers[spawn.layer.0 as usize];
        let particle = &mut self.particles[slot];
        particle.generation = particle
            .generation
            .checked_add(1)
            .ok_or_else(|| failure("particle incarnation exhausted"))?;
        particle.layer = spawn.layer;
        particle.birth = spawn.time;
        particle.cursor = spawn.time;
        particle.random = RandomState {
            seed: spawn.seed,
            counter: 0,
        };
        particle.parent = spawn.parent;
        particle.state.clear();
        for (field, definition) in layer.fields.iter().enumerate() {
            self.fields.write(field, slot, definition.initial);
        }
        let mapping = if spawn.payload.is_some() {
            program.inner.events[spawn.payload_event]
                .maps
                .get(spawn.payload_target)
                .ok_or_else(|| failure("queued payload target map is absent"))?
                .as_ref()
        } else {
            &[]
        };
        self.payload
            .initialize(
                slot,
                layer.payload.fields.len(),
                spawn.payload,
                mapping,
                spawn.payload_fraction,
                spawn.payload_ordinal,
            )
            .map_err(failure)?;
        self.particles[slot].payload_frame =
            self.payload.frame(slot, &layer.payload).map_err(failure)?;
        // Canonical values and the frame now own everything the initializer needs.
        // Release queued ownership before it can emit another bounded payload.
        if let Some(handle) = spawn.payload.take() {
            self.payload.arena.release(handle).map_err(failure)?;
        }
        self.active.push(slot);
        self.execute(program, slot, &layer.init, spawn.time, 0., inputs)?;
        *work += 1;
        self.stats.spawned = self.stats.spawned.saturating_add(1);
        let life = f32::from_bits(self.fields.read(layer.life, slot)[0]);
        let inverse = f32::from_bits(self.fields.read(layer.inverse_life, slot)[0]);
        Lifetime::read(life, inverse).map_err(|message| EffectRuntimeError {
            layer: Some(spawn.layer),
            offset: None,
            message,
        })?;
        if life >= 1.0 {
            self.retire_at(self.active.len() - 1);
            return Ok(());
        }
        if end > spawn.time && self.evolve(program, slot, end, inputs, work)? {
            let index = self.active.len() - 1;
            self.retire_at(index);
        }
        Ok(())
    }
    fn evolve(
        &mut self,
        program: &EffectProgram,
        slot: usize,
        end: f64,
        inputs: SceneInputs,
        work: &mut usize,
    ) -> Result<bool, EffectRuntimeError> {
        let particle = &mut self.particles[slot];
        let layer_id = particle.layer;
        let layer = &program.inner.layers[layer_id.0 as usize];
        let delta = (end - particle.cursor).max(0.);
        particle.cursor = end;
        let inverse = f32::from_bits(self.fields.read(layer.inverse_life, slot)[0]);
        let mut life = f32::from_bits(self.fields.read(layer.life, slot)[0]);
        life = Lifetime::read(life, inverse)
            .map_err(|message| EffectRuntimeError {
                layer: Some(layer_id),
                offset: None,
                message,
            })?
            .advance(delta);
        self.fields
            .write(layer.life, slot, [life.to_bits(), 0, 0, 0]);
        if *work >= self.limits.work_per_update {
            self.stats.omitted_executions = self.stats.omitted_executions.saturating_add(1);
            return Ok(life >= 1.);
        }
        *work += 1;
        if life < 1. || layer.evolve_on_death {
            if let Some(bound) = &layer.evolve {
                self.execute(program, slot, bound, end, delta, inputs)?;
            }
        }
        let life = f32::from_bits(self.fields.read(layer.life, slot)[0]);
        Lifetime::read(
            life,
            f32::from_bits(self.fields.read(layer.inverse_life, slot)[0]),
        )
        .map_err(|message| EffectRuntimeError {
            layer: Some(layer_id),
            offset: None,
            message,
        })?;
        Ok(life >= 1.)
    }
    fn execute(
        &mut self,
        program: &EffectProgram,
        slot: usize,
        bound: &BoundProgram,
        time: f64,
        delta: f64,
        inputs: SceneInputs,
    ) -> Result<(), EffectRuntimeError> {
        self.scratch.clear();
        let particle = &mut self.particles[slot];
        let layer = particle.layer;
        let id = EffectParticleId {
            slot: slot as u32,
            generation: particle.generation,
        };
        let mut host = ExecutionHost {
            graph: &program.inner,
            bound,
            fields: &mut self.fields,
            slot,
            id,
            layer,
            birth: particle.birth,
            time,
            delta,
            running: self.running,
            emitting: self.emitting,
            world: inputs.world_from_effect,
            payload_frame: particle.payload_frame,
            payload: &mut self.payload,
            attributes: &self.attributes,
            random: &mut particle.random,
            scratch: &mut self.scratch,
            pending: &mut self.pending,
            limits: self.limits,
            stats: &mut self.stats,
            order: &mut self.order,
        };
        bound
            .program
            .execute_with_scratch(&mut particle.state, &mut self.vm_scratch, &mut host)
            .map_err(|error| EffectRuntimeError {
                layer: Some(layer),
                offset: Some(error.offset),
                message: match error.kind {
                    Failure::Host(message) => message,
                    Failure::HostType => "effect callback returned incompatible value",
                    Failure::IncompatibleState => "effect particle storage is incompatible",
                    Failure::IntegerDivisionByZero => "particle integer division by zero",
                },
            })
    }
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;
