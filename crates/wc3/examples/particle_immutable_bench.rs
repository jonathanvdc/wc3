//! Original four-route positive immutable-age workload; no rendering.
use std::alloc::{GlobalAlloc, Layout, System};
use std::array::from_fn;
use std::env;
use std::hint::black_box;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::time::Instant;
use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::scene::popcorn::{
    EffectFieldId, EffectInstance, EffectLayerId, EffectProgram, ExecutionBackend,
    ExecutionOptions, ExecutionPreference, ExecutionStage, ExecutionStats, ImmutableProjection,
    ImmutableProjectionScratch, NumericPolicy, ParticleView, SceneInputs, Value,
};

#[path = "../benches/support/particle_immutable.rs"]
mod support;
use support::{fixture, verify_frame, verify_trajectory, warm, DT, STEPS};

struct Tracking;
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static REALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static DEALLOCATIONS: AtomicU64 = AtomicU64::new(0);
#[global_allocator]
static ALLOCATOR: Tracking = Tracking;
unsafe impl GlobalAlloc for Tracking {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Relaxed);
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOCATIONS.fetch_add(1, Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        REALLOCATIONS.fetch_add(1, Relaxed);
        unsafe { System.realloc(ptr, layout, size) }
    }
}
fn allocations() -> [u64; 3] {
    [
        ALLOCATIONS.load(Relaxed),
        REALLOCATIONS.load(Relaxed),
        DEALLOCATIONS.load(Relaxed),
    ]
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Route {
    Interpreter,
    ScalarJit,
    SimdJit,
    ImmutableAge,
}
impl Route {
    fn preference(self) -> ExecutionPreference {
        match self {
            Self::Interpreter => ExecutionPreference::Interpreter,
            Self::ScalarJit => ExecutionPreference::ScalarJit,
            Self::SimdJit => ExecutionPreference::Automatic,
            Self::ImmutableAge => ExecutionPreference::ImmutableAge,
        }
    }
    fn work(self, children: usize) -> [u64; 6] {
        let n = children as u64;
        match self {
            Self::Interpreter => [0, 0, 0, if children >= 16 { n } else { 0 }, 0, 0],
            Self::ScalarJit => [n + 1, 0, 0, 0, 0, 0],
            Self::SimdJit if children >= 16 => [1, n / 16, n, 0, 0, 0],
            Self::SimdJit => [n + 1, 0, 0, 0, 0, 0],
            Self::ImmutableAge => [1, 0, 0, 0, n, 0],
        }
    }
}
fn counters(stats: ExecutionStats) -> [u64; 6] {
    [
        stats.scalar_jit_invocations,
        stats.simd_attempted_batches,
        stats.simd_committed_lanes,
        stats.rust_packet_committed_lanes,
        stats.immutable_age_invocations,
        stats.immutable_age_fallbacks,
    ]
}
fn difference(current: ExecutionStats, previous: ExecutionStats) -> [u64; 6] {
    let current = counters(current);
    let previous = counters(previous);
    from_fn(|index| {
        current[index]
            .checked_sub(previous[index])
            .expect("execution counter regressed")
    })
}

const FIELD_COUNT: usize = 14;
const FIELDS: [EffectFieldId; FIELD_COUNT] = [
    EffectFieldId(0),
    EffectFieldId(1),
    EffectFieldId(2),
    EffectFieldId(3),
    EffectFieldId(4),
    EffectFieldId(5),
    EffectFieldId(6),
    EffectFieldId(7),
    EffectFieldId(8),
    EffectFieldId(9),
    EffectFieldId(10),
    EffectFieldId(11),
    EffectFieldId(12),
    EffectFieldId(13),
];
#[derive(Clone, Copy, PartialEq, Eq)]
enum QueryMethod {
    Individual,
    Bulk,
    Projection,
}
impl QueryMethod {
    fn parse(value: &str) -> Self {
        match value {
            "individual" => Self::Individual,
            "bulk" => Self::Bulk,
            "projection" => Self::Projection,
            _ => panic!("unknown query method; expected individual|bulk|projection"),
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Individual => "individual",
            Self::Bulk => "bulk",
            Self::Projection => "projection",
        }
    }
}
struct ProjectionQuery {
    projection: ImmutableProjection,
    scratch: ImmutableProjectionScratch,
    values: Vec<Value>,
    positions: Vec<usize>,
    raw_positions: Vec<usize>,
}
impl ProjectionQuery {
    fn prepare(program: &EffectProgram, requested: &[EffectFieldId]) -> Option<Self> {
        let recipe = program.immutable_recipe(EffectLayerId(1))?;
        let mut positions = Vec::new();
        let mut raw_positions = Vec::new();
        let mut fields = Vec::new();
        for (position, &field) in requested.iter().enumerate() {
            if (0..recipe.output_count()).any(|i| recipe.output(i).unwrap().field == field) {
                positions.push(position);
                fields.push(field);
            } else {
                raw_positions.push(position);
            }
        }
        let projection = recipe.project(&fields).unwrap();
        let scratch = projection.new_scratch();
        Some(Self {
            values: vec![Value::from(0f32); fields.len()],
            projection,
            scratch,
            positions,
            raw_positions,
        })
    }
    fn counts(&self) -> [usize; 3] {
        [
            self.projection.node_count(),
            self.projection.capture_count(),
            self.projection.output_count(),
        ]
    }
    fn read(
        &mut self,
        particle: &ParticleView<'_>,
        fields: &[EffectFieldId],
        values: &mut [Value],
    ) {
        if let Some(sample) = particle.immutable_sample() {
            self.projection
                .evaluate(sample, &mut self.scratch, &mut self.values)
                .unwrap();
            for (&position, &value) in self.positions.iter().zip(&self.values) {
                values[position] = value;
            }
            for &position in &self.raw_positions {
                values[position] = particle.field(fields[position]).unwrap();
            }
        } else {
            particle.sample_fields(fields, values).unwrap();
        }
    }
}
struct Queries {
    method: QueryMethod,
    full: Option<ProjectionQuery>,
    position: Option<ProjectionQuery>,
    full_prepare_ns: u128,
    position_prepare_ns: u128,
}
impl Queries {
    fn prepare(program: &EffectProgram, method: QueryMethod) -> Self {
        let begin = Instant::now();
        let full = (method == QueryMethod::Projection)
            .then(|| ProjectionQuery::prepare(program, &FIELDS))
            .flatten();
        let full_prepare_ns = if full.is_some() {
            begin.elapsed().as_nanos()
        } else {
            0
        };
        let begin = Instant::now();
        let position = (method == QueryMethod::Projection)
            .then(|| ProjectionQuery::prepare(program, &[EffectFieldId(8)]))
            .flatten();
        let position_prepare_ns = if position.is_some() {
            begin.elapsed().as_nanos()
        } else {
            0
        };
        Self {
            method,
            full,
            position,
            full_prepare_ns,
            position_prepare_ns,
        }
    }
    fn read(&mut self, particle: &ParticleView<'_>, values: &mut [Value; FIELD_COUNT]) {
        match self.method {
            QueryMethod::Individual => {
                for (&field, value) in FIELDS.iter().zip(values) {
                    *value = particle.field(field).unwrap();
                }
            }
            QueryMethod::Bulk => particle.sample_fields(&FIELDS, values).unwrap(),
            QueryMethod::Projection => match &mut self.full {
                Some(query) => query.read(particle, &FIELDS, values),
                None => particle.sample_fields(&FIELDS, values).unwrap(),
            },
        }
    }
}

// Public structure hash deliberately excludes numeric field values, whose independent
// analytic oracle is checked separately. It cannot hide births, ownership or order.
fn structure(instance: &EffectInstance) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    let mut mix = |word: u64| {
        for byte in word.to_le_bytes() {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
    };
    mix(instance.time().to_bits());
    mix(instance.is_running() as u64);
    mix(instance.is_emitting() as u64);
    mix(instance.is_finished() as u64);
    for layer in 0..2 {
        for p in instance.particles(EffectLayerId(layer)) {
            mix(u64::from(layer));
            mix(u64::from(p.id.slot));
            mix(u64::from(p.id.generation));
            mix(p.birth_time.to_bits());
            mix(p.birth_serial);
            mix(p.parent.is_some() as u64);
            if let Some(parent) = p.parent {
                mix(u64::from(parent.slot));
                mix(u64::from(parent.generation));
            }
        }
    }
    hash
}
fn percentile(values: &[u128], numerator: usize) -> u128 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() - 1) * numerator / 100]
}
fn main() {
    let args: Vec<_> = env::args().collect();
    assert!(
        args.len() == 6 || (args.len() == 8 && args[6] == "--query-method"),
        "usage: particle_immutable_bench interpreter|scalar-jit|simd-jit|immutable-age N C INSTANCES SAMPLES [--query-method individual|bulk|projection]"
    );
    let query_method = if args.len() == 8 {
        QueryMethod::parse(&args[7])
    } else {
        QueryMethod::Individual
    };
    let route = match args[1].as_str() {
        "interpreter" => Route::Interpreter,
        "scalar-jit" => Route::ScalarJit,
        "simd-jit" => Route::SimdJit,
        "immutable-age" => Route::ImmutableAge,
        _ => panic!("unknown route"),
    };
    let children: usize = args[2].parse().unwrap();
    let capacity: usize = args[3].parse().unwrap();
    let count: usize = args[4].parse().unwrap();
    let samples: usize = args[5].parse().unwrap();
    assert!([1, 8, 128, 1024].contains(&children));
    assert!(capacity >= children && [128, 1024].contains(&capacity));
    assert!([1, 8].contains(&count) && samples >= 5);
    let document = Document::read(fixture(children)).unwrap();
    let (root, _) = document.effect_roots().next().unwrap().unwrap();
    let begin = Instant::now();
    let base = document
        .compile_effect(root, &EffectCompileOptions::default())
        .unwrap();
    let compile_ns = begin.elapsed().as_nanos();
    let begin = Instant::now();
    let program = base.with_execution(ExecutionOptions {
        preference: route.preference(),
        numeric_policy: NumericPolicy::Graphics,
    });
    let prepare_ns = begin.elapsed().as_nanos();
    assert!(!program.execution_reports().is_empty());
    for report in program.execution_reports() {
        let expected = match route {
            Route::Interpreter => ExecutionBackend::Interpreter,
            Route::ScalarJit => ExecutionBackend::ScalarJit,
            Route::SimdJit
                if report.layer == EffectLayerId(1)
                    && report.stage == ExecutionStage::Evolution =>
            {
                ExecutionBackend::SimdJit
            }
            Route::SimdJit => ExecutionBackend::ScalarJit,
            Route::ImmutableAge
                if report.layer == EffectLayerId(1)
                    && report.stage == ExecutionStage::Evolution =>
            {
                ExecutionBackend::ImmutableAge
            }
            Route::ImmutableAge => ExecutionBackend::ScalarJit,
        };
        assert_eq!(report.backend, expected, "requested route was not prepared");
        assert!(
            report.fallback.is_none(),
            "fallback is not an admitted native measurement"
        );
    }
    let mut queries = Queries::prepare(&program, query_method);
    verify_trajectory(&program, children, capacity);
    if route == Route::ImmutableAge {
        let probe = warm(&program, children, capacity);
        assert!(probe.immutable_active());
        assert_eq!(probe.immutable_time(), Some(probe.time()));
        assert!(probe.immutable_recipe(EffectLayerId(1)).is_some());
    }
    // Route proof is an independent untimed trajectory, not a timed statistics
    // read on every simulated frame. All six deltas must match exactly.
    let expected_work = route.work(children);
    let mut work_deltas = Vec::with_capacity(STEPS);
    let mut route_probe = warm(&program, children, capacity);
    let mut previous = route_probe.execution_stats();
    for _ in 0..STEPS {
        route_probe
            .advance(&program, DT, SceneInputs::default())
            .unwrap();
        let current = route_probe.execution_stats();
        let delta = difference(current, previous);
        assert_eq!(
            delta, expected_work,
            "actual work does not match the requested route"
        );
        work_deltas.push(delta);
        previous = current;
    }
    let mut structural = Vec::with_capacity(STEPS + 1);
    let mut numeric = Vec::with_capacity((STEPS + 1) * children * 14);
    let mut admission = warm(&program, children, capacity);
    for frame in 0..=STEPS {
        verify_frame(&admission, children);
        structural.push(structure(&admission));
        for p in admission.particles(EffectLayerId(1)) {
            let mut values = [Value::from(0f32); FIELD_COUNT];
            queries.read(&p, &mut values);
            for (&field, value) in FIELDS.iter().zip(values) {
                assert_eq!(
                    Some(value),
                    p.field(field),
                    "query methods must return identical full fields"
                );
                numeric.push(value.words());
            }
        }
        if frame < STEPS {
            admission
                .advance(&program, DT, SceneInputs::default())
                .unwrap();
        }
    }
    // Allocation/warmup/destruction are outside steady clocks; setup has its own clock.
    // Full admission above scans every field/frame; timed endpoints repeat it.
    let mut blocks = Vec::with_capacity(samples);
    let mut frames = Vec::with_capacity(samples * STEPS);
    let mut setup = Vec::with_capacity(samples);
    let mut instantiate_ns = Vec::with_capacity(samples);
    let mut birth_ns = Vec::with_capacity(samples);
    let mut query_ns = Vec::with_capacity(samples);
    let mut position_query_ns = Vec::with_capacity(samples);
    let mut query_allocation_deltas = Vec::with_capacity(samples);
    let mut allocation_deltas = Vec::with_capacity(samples);
    let mut measured_work_deltas = Vec::with_capacity(samples * count);
    for _ in 0..samples {
        let begin = Instant::now();
        let allocation_begin = Instant::now();
        let mut instances: Vec<_> = (0..count)
            .map(|_| program.instantiate(support::limits(capacity), 42).unwrap())
            .collect();
        instantiate_ns.push(allocation_begin.elapsed().as_nanos());
        let birth_begin = Instant::now();
        for instance in &mut instances {
            instance
                .start(&program, program.default_entry(), SceneInputs::default())
                .unwrap();
            instance
                .advance(&program, DT, SceneInputs::default())
                .unwrap();
        }
        birth_ns.push(birth_begin.elapsed().as_nanos());
        for instance in &mut instances {
            for _ in 1..support::WARMUP {
                instance
                    .advance(&program, DT, SceneInputs::default())
                    .unwrap();
            }
        }
        setup.push(begin.elapsed().as_nanos());
        let before: Vec<_> = instances
            .iter()
            .map(EffectInstance::execution_stats)
            .collect();
        let allocations_before = allocations();
        let begin = Instant::now();
        for _ in 0..STEPS {
            let frame = Instant::now();
            for instance in &mut instances {
                instance
                    .advance(&program, DT, SceneInputs::default())
                    .unwrap();
            }
            frames.push(frame.elapsed().as_nanos());
        }
        blocks.push(begin.elapsed().as_nanos());
        let allocation_delta = from_fn(|i| allocations()[i] - allocations_before[i]);
        assert_eq!(allocation_delta, [0; 3], "hot advance allocated");
        allocation_deltas.push(allocation_delta);
        for (instance, previous) in instances.iter().zip(before) {
            verify_frame(instance, children);
            let delta = difference(instance.execution_stats(), previous);
            assert_eq!(delta, expected_work.map(|value| value * STEPS as u64));
            measured_work_deltas.push(delta);
            black_box(instance);
        }
        // Separate fixed-time public query sweep; no advancement or proof in clock.
        let mut values = [Value::from(0f32); FIELD_COUNT];
        let allocations_before = allocations();
        let begin = Instant::now();
        for _ in 0..8 {
            for instance in &instances {
                for particle in instance.particles(EffectLayerId(1)) {
                    queries.read(&particle, &mut values);
                    for value in values {
                        black_box(value);
                    }
                }
            }
        }
        query_ns.push(begin.elapsed().as_nanos());
        let delta = from_fn(|i| allocations()[i] - allocations_before[i]);
        assert_eq!(delta, [0; 3], "hot queries allocated");
        query_allocation_deltas.push(delta);
        // This is deliberately a different workload: one Position field, not
        // a replacement for the full-field query cost or its numeric oracle.
        if let Some(position) = &mut queries.position {
            let mut value = [Value::from(0f32)];
            let allocations_before = allocations();
            let begin = Instant::now();
            for _ in 0..8 {
                for instance in &instances {
                    for particle in instance.particles(EffectLayerId(1)) {
                        position.read(&particle, &[EffectFieldId(8)], &mut value);
                        black_box(value[0]);
                    }
                }
            }
            position_query_ns.push(begin.elapsed().as_nanos());
            assert_eq!(
                allocations(),
                allocations_before,
                "position-only queries allocated"
            );
            for instance in &instances {
                for particle in instance.particles(EffectLayerId(1)) {
                    position.read(&particle, &[EffectFieldId(8)], &mut value);
                    assert_eq!(Some(value[0]), particle.field(EffectFieldId(8)));
                }
            }
        }
    }
    let projection_counts = queries
        .full
        .as_ref()
        .map_or([0; 3], ProjectionQuery::counts);
    let position_projection_counts = queries
        .position
        .as_ref()
        .map_or([0; 3], ProjectionQuery::counts);
    println!("route={:?}", program.execution_reports());
    println!("{{\"kind\":\"separate_costs\",\"query_method\":\"{}\",\"instantiate_ns\":{instantiate_ns:?},\"start_and_first_birth_tick_ns\":{birth_ns:?},\"fixed_time_8_full_query_sweeps_ns\":{query_ns:?},\"full_projection_prepare_ns\":{},\"full_projection_counts\":{projection_counts:?},\"position_projection_prepare_ns\":{},\"position_projection_counts\":{position_projection_counts:?},\"fixed_time_8_position_only_projection_sweeps_ns\":{position_query_ns:?},\"query_allocation_deltas\":{query_allocation_deltas:?},\"hot_allocation_deltas\":{allocation_deltas:?}}}", query_method.name(), queries.full_prepare_ns, queries.position_prepare_ns);
    println!("{{\"kind\":\"result\",\"route\":\"{}\",\"query_method\":\"{}\",\"children\":{},\"capacity_child_units\":{},\"capacity_total\":{},\"instances\":{},\"samples\":{},\"steps\":{},\"compile_ns\":{},\"prepare_ns\":{},\"block_median_ns\":{},\"block_p95_ns\":{},\"frame_median_ns\":{},\"frame_p95_ns\":{},\"setup_ns\":{:?},\"blocks_ns\":{:?},\"frames_ns\":{:?},\"structure\":{:?},\"numeric\":{:?},\"work_columns\":[\"scalar_jit\",\"simd_batches\",\"simd_lanes\",\"rust_lanes\",\"immutable_age\",\"immutable_fallbacks\"],\"work_deltas\":{:?},\"measured_work_deltas\":{:?}}}",
        args[1], query_method.name(), children, capacity, capacity + 1, count, samples, STEPS, compile_ns, prepare_ns,
        percentile(&blocks, 50), percentile(&blocks, 95), percentile(&frames, 50), percentile(&frames, 95),
        setup, blocks, frames, structural, numeric, work_deltas, measured_work_deltas);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_mappings_match_full_fields_across_initial_samples_and_denied_work() {
        for preference in [
            ExecutionPreference::Interpreter,
            ExecutionPreference::ImmutableAge,
        ] {
            let document = Document::read(fixture(8)).unwrap();
            let (root, _) = document.effect_roots().next().unwrap().unwrap();
            let program = document
                .compile_effect(root, &EffectCompileOptions::default())
                .unwrap()
                .with_execution(ExecutionOptions {
                    preference,
                    numeric_policy: NumericPolicy::Graphics,
                });
            let mut queries = [
                QueryMethod::Individual,
                QueryMethod::Bulk,
                QueryMethod::Projection,
            ]
            .map(|method| Queries::prepare(&program, method));
            let mut instance = program.instantiate(support::limits(128), 42).unwrap();
            instance
                .start(&program, program.default_entry(), SceneInputs::default())
                .unwrap();
            for _ in 0..3 {
                instance
                    .advance(&program, DT, SceneInputs::default())
                    .unwrap();
                let stats = instance.execution_stats();
                for particle in instance.particles(EffectLayerId(1)) {
                    let expected = FIELDS.map(|field| particle.field(field).unwrap());
                    for query in &mut queries {
                        let mut values = [Value::from(-1f32); FIELD_COUNT];
                        query.read(&particle, &mut values);
                        assert_eq!(values, expected);
                        if let Some(position) = &mut query.position {
                            let mut value = [Value::from(-1f32)];
                            position.read(&particle, &[EffectFieldId(8)], &mut value);
                            assert_eq!(value[0], expected[8]);
                        }
                    }
                }
                assert_eq!(instance.execution_stats(), stats);
            }
            instance
                .advance_with_step_budget(&program, DT, SceneInputs::default(), || false)
                .unwrap();
            for particle in instance.particles(EffectLayerId(1)) {
                let expected = FIELDS.map(|field| particle.field(field).unwrap());
                for query in &mut queries {
                    let mut values = [Value::from(-1f32); FIELD_COUNT];
                    query.read(&particle, &mut values);
                    assert_eq!(values, expected);
                }
            }
            if preference == ExecutionPreference::ImmutableAge {
                let full = queries[2].full.as_ref().unwrap();
                let position = queries[2].position.as_ref().unwrap();
                assert!(full.projection.output_count() > 1);
                assert_eq!(position.counts(), [1, 1, 1]);
                assert!(position.projection.node_count() < full.projection.node_count());
                assert!(!full.raw_positions.is_empty());
            } else {
                assert!(queries[2].full.is_none());
                assert!(queries[2].position.is_none());
            }
        }
    }
}
