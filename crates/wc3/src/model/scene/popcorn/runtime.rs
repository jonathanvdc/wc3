//! Bounded reusable lifecycle storage and deterministic event-time scheduling.
use super::callbacks::{ExecutionHost, Scratch};
use super::execution_plan::ExecutionPlan;
use super::graph::*;
use super::lifetime::Lifetime;
use super::packet::{Lane, Scratch as PacketScratch, MINIMUM, WIDTH};
use super::payload::{Handle as PayloadHandle, Storage as PayloadStorage};
use super::spatial::Store as SpatialStore;
use super::{ExecutionReport, ExecutionStage, ExecutionStats, Failure, State, Value, ValueType};
use glam::Mat4;
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
    /// Entries retained per shared spatial resource in each of two snapshots.
    pub spatial_entries: usize,
    /// Spatial candidate rows examined across one whole advance, not per step.
    pub spatial_checks_per_update: usize,
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
            spatial_entries: 1024,
            spatial_checks_per_update: 1_048_576,
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
    /// Spatial commits omitted by entry/draft capacity or unavailable row data.
    pub omitted_spatial_writes: u64,
    /// Exact spatial queries omitted by the shared per-advance row-check budget.
    pub omitted_spatial_queries: u64,
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
/// Programme-wide physical planes shared by layer-local field indices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct FieldLayout {
    ordinals: Vec<[usize; 4]>,
    plane_count: usize,
}
impl FieldLayout {
    pub(super) const ABSENT: usize = usize::MAX;

    /// One authoritative width union for runtime storage and native preparation.
    pub(super) fn for_graph(graph: &EffectGraph) -> Option<Self> {
        let count = graph
            .layers
            .iter()
            .map(|layer| layer.fields.len())
            .max()
            .unwrap_or(0);
        let mut widths = vec![0; count];
        for layer in &graph.layers {
            for (width, field) in widths.iter_mut().zip(&layer.fields) {
                *width = (*width).max(field.value_type.map_or(1, ValueType::components));
            }
        }
        Self::from_widths(&widths)
    }
    fn from_widths(widths: &[usize]) -> Option<Self> {
        let mut plane_count = 0usize;
        let mut ordinals = Vec::with_capacity(widths.len());
        for &width in widths {
            if width > 4 {
                return None;
            }
            let mut field = [Self::ABSENT; 4];
            for ordinal in field.iter_mut().take(width) {
                *ordinal = plane_count;
                plane_count = plane_count.checked_add(1)?;
            }
            ordinals.push(field);
        }
        Some(Self {
            ordinals,
            plane_count,
        })
    }
    #[cfg(any(test, feature = "popcorn-jit"))]
    pub(super) fn component_ordinals(&self) -> &[[usize; 4]] {
        &self.ordinals
    }
    pub(super) fn storage_bytes(&self, capacity: usize) -> Option<usize> {
        self.plane_count
            .checked_mul(capacity)?
            .checked_mul(size_of::<u32>())?
            .checked_add(self.ordinals.len().checked_mul(size_of::<[usize; 4]>())?)?
            .checked_add(size_of::<FieldPlanes>())
    }
}
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(Clone))]
pub(crate) struct FieldPlanes {
    layout: FieldLayout,
    data: Vec<u32>,
    capacity: usize,
}
impl FieldPlanes {
    fn new(layout: FieldLayout, capacity: usize) -> Option<Self> {
        let words = layout.plane_count.checked_mul(capacity)?;
        // Reject impossible byte sizes before Vec's allocation/capacity arithmetic.
        words
            .checked_mul(size_of::<u32>())
            .filter(|&bytes| bytes <= isize::MAX as usize)?;
        Some(Self {
            layout,
            data: vec![0; words],
            capacity,
        })
    }
    /// Borrowed invocation ABI only: buffers never resize during pool simulation.
    /// The caller must not retain this pointer beyond the exclusive invocation.
    #[cfg(any(test, feature = "popcorn-jit"))]
    pub(super) fn data_mut_ptr(&mut self) -> *mut u32 {
        self.data.as_mut_ptr()
    }
    #[cfg(any(test, feature = "popcorn-jit"))]
    pub(super) fn plane_stride(&self) -> usize {
        self.capacity
    }
    #[cfg(any(test, feature = "popcorn-jit"))]
    pub(super) fn layout(&self) -> &FieldLayout {
        &self.layout
    }
    pub fn read(&self, field: usize, slot: usize) -> [u32; 4] {
        let ordinals = self.layout.ordinals[field];
        if slot >= self.capacity {
            return [0; 4];
        }
        ordinals.map(|ordinal| {
            if ordinal == FieldLayout::ABSENT {
                0
            } else {
                self.data[ordinal * self.capacity + slot]
            }
        })
    }
    pub fn write(&mut self, field: usize, slot: usize, words: [u32; 4]) {
        let ordinals = self.layout.ordinals[field];
        if slot >= self.capacity {
            return;
        }
        for (ordinal, word) in ordinals.into_iter().zip(words) {
            if ordinal != FieldLayout::ABSENT {
                self.data[ordinal * self.capacity + slot] = word;
            }
        }
    }
}
#[cfg(test)]
#[path = "field_storage_tests.rs"]
mod field_storage_tests;
struct Particle {
    state: State,
    layer: EffectLayerId,
    generation: u32,
    birth: f64,
    serial: u64,
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
    /// Monotonic allocation order within this instance restart, independent of slot reuse.
    pub birth_serial: u64,
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
    execution: Arc<ExecutionPlan>,
    execution_stats: ExecutionStats,
    spatial: Vec<SpatialStore>,
    spatial_checks: usize,
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
    packet: Option<Box<PacketScratch>>,
    limits: EffectLimits,
    stats: EffectStats,
    attributes: Vec<Value>,
    time: f64,
    running: bool,
    emitting: bool,
    faulted: bool,
    seed: u64,
    order: u64,
    next_serial: u64,
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
        let field_layout = FieldLayout::for_graph(&self.inner)
            .ok_or_else(|| failure("effect field layout exceeds its checked storage domain"))?;
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
            .and_then(|bytes| bytes.checked_add(size_of::<Particle>()));
        let bytes = words
            .and_then(|bytes| bytes.checked_mul(limits.particles))
            .and_then(|bytes| bytes.checked_add(field_layout.storage_bytes(limits.particles)?))
            .and_then(|bytes| {
                bytes.checked_add((counts[0] + counts[1]) * 16 + arguments * size_of::<Value>())
            })
            .and_then(|bytes| {
                bytes.checked_add(limits.pending_events.checked_mul(size_of::<Spawn>())?)
            })
            .and_then(|bytes| {
                bytes.checked_add(Scratch::storage_bytes(
                    limits,
                    payload_width,
                    !self.inner.spatial.is_empty(),
                )?)
            })
            .and_then(|bytes| {
                bytes.checked_add(limits.particles.checked_mul(2 * size_of::<usize>())?)
            });
        let bytes = bytes.and_then(|bytes| {
            if self.inner.spatial.is_empty() {
                Some(bytes)
            } else {
                bytes.checked_add(
                    SpatialStore::storage_bytes(limits.spatial_entries)?
                        .checked_mul(self.inner.spatial.len())?,
                )
            }
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
        // Optional acceleration cannot reject an otherwise valid scalar pool.
        // Include the boxed metadata and the exact fixed-plane allocations.
        let packet = PacketScratch::shape(&self.inner.layers).and_then(|shape| {
            let total = bytes?.checked_add(PacketScratch::bytes(shape)?)?;
            (total <= limits.storage_bytes).then(|| PacketScratch::new(shape))
        });
        let particles = (0..limits.particles)
            .map(|_| Particle {
                state: State::persistent_for_programs(programs.iter().copied()),
                layer: EffectLayerId(0),
                generation: 0,
                birth: 0.,
                serial: 0,
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
            execution: self.execution.clone(),
            execution_stats: ExecutionStats::default(),
            spatial: self
                .inner
                .spatial
                .iter()
                .map(|resource| {
                    SpatialStore::new(limits.spatial_entries, resource.cell_size).map_err(failure)
                })
                .collect::<Result<_, _>>()?,
            spatial_checks: limits.spatial_checks_per_update,
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
            fields: FieldPlanes::new(field_layout, limits.particles)
                .ok_or_else(|| failure("effect field storage exceeds its checked size"))?,
            pending: BinaryHeap::with_capacity(limits.pending_events),
            scratch: Scratch::new(limits, payload_width, !self.inner.spatial.is_empty()),
            payload: PayloadStorage::new(limits.particles, limits.pending_events, payload_width)
                .map_err(failure)?,
            vm_scratch: State::scratch_for_programs(programs.iter().copied()),
            packet,
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
            next_serial: 0,
        })
    }
}
impl EffectInstance {
    /// Prepared routes captured when this instance was allocated.
    pub fn execution_reports(&self) -> &[ExecutionReport] {
        self.execution.reports()
    }
    /// Cumulative native invocation attempts over this instance's lifetime.
    /// Restarting with `start` retains these diagnostics while resetting simulation statistics.
    pub fn execution_stats(&self) -> ExecutionStats {
        self.execution_stats
    }
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
        for store in &mut self.spatial {
            store.clear();
        }
        self.spatial_checks = self.limits.spatial_checks_per_update;
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
        self.next_serial = 0;
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
                    birth_serial: particle.serial,
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
        self.advance_with_step_budget(program, delta, inputs, || true)
    }
    /// Advance with an additional shared step allowance. The callback is queried
    /// immediately before each real simulation step, after the local limit. On
    /// denial the remaining time is omitted and particles age/retire normally,
    /// without deferred execution. Zero delta never queries the callback.
    pub fn advance_with_step_budget(
        &mut self,
        program: &EffectProgram,
        delta: f64,
        inputs: SceneInputs,
        mut permit_step: impl FnMut() -> bool,
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
        self.spatial_checks = self.limits.spatial_checks_per_update;
        let end = self.time + delta;
        let mut steps = 0;
        let mut work = 0;
        while self.time < end && steps < self.limits.steps_per_update {
            let next = (self.time + self.limits.step_seconds).min(end);
            if next <= self.time {
                break;
            }
            if !permit_step() {
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
            for store in &mut self.spatial {
                store.clear();
            }
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
        for store in &mut self.spatial {
            store.begin_tick();
        }
        // Existing-active processing can only retire; births are drained later.
        if !self.execution.is_native() && self.packet.is_some() && self.active.len() >= MINIMUM {
            self.evolve_packet_active(program, end, inputs, work)?;
        } else {
            self.evolve_active::<false>(program, end, inputs, work)?;
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
    // Keep the packet-enabled loop outside the common scalar step frame.
    #[inline(never)]
    fn evolve_packet_active(
        &mut self,
        program: &EffectProgram,
        end: f64,
        inputs: SceneInputs,
        work: &mut usize,
    ) -> Result<(), EffectRuntimeError> {
        self.evolve_active::<true>(program, end, inputs, work)
    }
    fn evolve_active<const PACKETS: bool>(
        &mut self,
        program: &EffectProgram,
        end: f64,
        inputs: SceneInputs,
        work: &mut usize,
    ) -> Result<(), EffectRuntimeError> {
        let mut index = 0;
        let mut scalar_until = 0;
        while index < self.active.len() {
            let slot = self.active[index];
            let layer = &program.inner.layers[self.particles[slot].layer.0 as usize];
            if !self.running && layer.spawner {
                self.retire_at(index);
                continue;
            }
            if PACKETS
                && self.active.len() - index >= MINIMUM
                && index >= scalar_until
                && layer.evolve.is_some()
                && layer.pure_evolution.is_some()
            {
                let (committed, examined) = self.try_packet(program, index, end, work);
                if committed > 0 {
                    index += committed;
                    continue;
                }
                // A short run or failed speculative tile is examined only once.
                // Scalar replay still owns its exact host-write/error prefix.
                scalar_until = index + examined.max(1);
            }
            let dead = self.evolve(program, slot, end, inputs, work)?;
            if dead {
                self.retire_at(index);
            } else {
                index += 1;
            }
        }
        Ok(())
    }
    // Keep packet execution storage and code out of the scalar step frame.
    #[inline(never)]
    fn try_packet(
        &mut self,
        program: &EffectProgram,
        index: usize,
        end: f64,
        work: &mut usize,
    ) -> (usize, usize) {
        let remaining = self.limits.work_per_update.saturating_sub(*work);
        if remaining < MINIMUM {
            return (0, remaining.max(1));
        }
        let layer_id = self.particles[self.active[index]].layer;
        let layer = &program.inner.layers[layer_id.0 as usize];
        let Some(plan) = layer.evolve.as_ref().and(layer.pure_evolution.as_deref()) else {
            return (0, 1);
        };
        let Some(scratch) = &mut self.packet else {
            return (0, 1);
        };
        scratch.count = 0;
        for &slot in self.active[index..].iter().take(WIDTH.min(remaining)) {
            let particle = &self.particles[slot];
            if particle.layer != layer_id {
                break;
            }
            let delta = (end - particle.cursor).max(0.);
            let Ok(lifetime) = Lifetime::read(
                f32::from_bits(self.fields.read(layer.life, slot)[0]),
                f32::from_bits(self.fields.read(layer.inverse_life, slot)[0]),
            ) else {
                break;
            };
            let life = lifetime.advance(delta);
            // Terminal, instantaneous and malformed lanes stay at their exact
            // scalar cursor/work/death boundary; no speculation escapes here.
            if !life.is_finite() || life >= 1. {
                break;
            }
            scratch.lanes[scratch.count] = Lane { slot, delta, life };
            scratch.count += 1;
        }
        let count = scratch.count;
        if count < MINIMUM {
            return (0, count);
        }
        scratch.stage(plan, &self.fields, layer.life);
        if scratch.execute(plan, &program.inner.samplers, end).is_err() {
            #[cfg(test)]
            {
                scratch.fallbacks += 1;
            }
            return (0, count);
        }
        for lane in 0..count {
            let value = scratch.lanes[lane];
            self.particles[value.slot].cursor = end;
            self.fields
                .write(layer.life, value.slot, [value.life.to_bits(), 0, 0, 0]);
            *work += 1;
            // Preserve the checked host invocation boundary. Pure operations
            // never allocate capabilities or touch its preserved temporal cache.
            self.scratch.clear();
            scratch.commit(plan, lane, &mut self.fields);
            #[cfg(test)]
            {
                scratch.committed += 1;
            }
            debug_assert!(Lifetime::read(
                f32::from_bits(self.fields.read(layer.life, value.slot)[0]),
                f32::from_bits(self.fields.read(layer.inverse_life, value.slot)[0]),
            )
            .is_ok());
        }
        (count, count)
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
        let serial = self.next_serial;
        let next_serial = serial
            .checked_add(1)
            .ok_or_else(|| failure("particle birth serial exhausted"))?;
        let Some(slot) = self.free.pop() else {
            self.stats.omitted_births += 1;
            return Ok(());
        };
        self.next_serial = next_serial;
        let layer = &program.inner.layers[spawn.layer.0 as usize];
        let particle = &mut self.particles[slot];
        particle.generation = particle
            .generation
            .checked_add(1)
            .ok_or_else(|| failure("particle incarnation exhausted"))?;
        particle.layer = spawn.layer;
        particle.birth = spawn.time;
        particle.serial = serial;
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
        self.execute(
            program,
            slot,
            &layer.init,
            ExecutionStage::Initialization,
            spawn.time,
            0.,
            inputs,
        )?;
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
                self.execute(
                    program,
                    slot,
                    bound,
                    ExecutionStage::Evolution,
                    end,
                    delta,
                    inputs,
                )?;
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
    #[allow(clippy::too_many_arguments)]
    fn execute(
        &mut self,
        program: &EffectProgram,
        slot: usize,
        bound: &BoundProgram,
        stage: ExecutionStage,
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
            spatial: &mut self.spatial,
            spatial_checks: &mut self.spatial_checks,
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
            inputs: &inputs,
            payload_frame: particle.payload_frame.as_ref(),
            payload: &mut self.payload,
            attributes: &self.attributes,
            random: &mut particle.random,
            scratch: &mut self.scratch,
            pending: &mut self.pending,
            limits: &self.limits,
            stats: &mut self.stats,
            order: &mut self.order,
        };
        if self.execution.is_native() {
            self.execution_stats.scalar_jit_invocations = self
                .execution_stats
                .scalar_jit_invocations
                .saturating_add(1);
        }
        self.execution
            .execute(
                layer,
                stage,
                &bound.program,
                &mut particle.state,
                &mut self.vm_scratch,
                &mut host,
            )
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

#[cfg(test)]
#[path = "packet_runtime_tests.rs"]
mod packet_tests;

#[cfg(test)]
#[path = "native_runtime_tests.rs"]
mod native_tests;
