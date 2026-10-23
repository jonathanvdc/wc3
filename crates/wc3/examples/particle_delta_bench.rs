//! Delta-affine steady population and routed birth/death benchmarks; no rendering.
use std::array::from_fn;
use std::env;
use std::hint::black_box;
use std::time::Instant;
use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::scene::popcorn::{
    EffectFieldId, EffectInstance, EffectLayerId, EffectProgram, ExecutionBackend,
    ExecutionOptions, ExecutionPreference, ExecutionStage, NumericPolicy, SceneInputs, Value,
};

#[path = "../benches/support/benchmark_alloc.rs"]
mod benchmark_alloc;
#[path = "../benches/support/particle_delta.rs"]
mod support;
use benchmark_alloc::allocations;
use support::{
    advance, compare, counters, delta, fixture, limits, oracle, start, ABS, ADMITTED_PACKET_LANES,
    DT, FIELDS, REL, ROUTED_LIVE_CHILDREN, STEPS, WARM,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Route {
    Interpreter,
    Scalar,
    Simd,
    Immutable,
}
impl Route {
    fn parse(value: &str) -> Self {
        match value {
            "interpreter" => Self::Interpreter,
            "scalar-jit" => Self::Scalar,
            "simd-jit" => Self::Simd,
            "immutable-age" => Self::Immutable,
            _ => panic!("unknown route"),
        }
    }
    fn preference(self) -> ExecutionPreference {
        match self {
            Self::Interpreter => ExecutionPreference::Interpreter,
            Self::Scalar => ExecutionPreference::ScalarJit,
            Self::Simd => ExecutionPreference::Automatic,
            Self::Immutable => ExecutionPreference::ImmutableAge,
        }
    }
    fn verify_preparation(self, program: &EffectProgram) {
        assert!(!program.execution_reports().is_empty());
        for report in program.execution_reports() {
            let child_evolution =
                report.layer == EffectLayerId(1) && report.stage == ExecutionStage::Evolution;
            let expected = match self {
                Self::Interpreter => ExecutionBackend::Interpreter,
                Self::Scalar => ExecutionBackend::ScalarJit,
                Self::Simd if child_evolution => ExecutionBackend::SimdJit,
                Self::Immutable if child_evolution => ExecutionBackend::ImmutableAge,
                Self::Simd | Self::Immutable => ExecutionBackend::ScalarJit,
            };
            assert_eq!(
                report.backend, expected,
                "requested backend was not prepared"
            );
            assert!(
                report.fallback.is_none(),
                "native platform/compiler fallback"
            );
        }
    }
    // Stable measured interval, after the separately observed birth/death prefix.
    // First controller + child init are scalar JIT invocations where applicable.
    fn verify_work(self, children: usize, routed: bool, work: [u64; 6]) {
        let n = if routed {
            ROUTED_LIVE_CHILDREN as u64
        } else {
            children as u64
        };
        let scalar_prefix = if routed { 2 } else { 1 };
        let expected = match self {
            Self::Interpreter => [
                0,
                0,
                0,
                if !routed && n >= ADMITTED_PACKET_LANES {
                    n
                } else {
                    0
                },
                0,
                0,
            ],
            Self::Scalar => [n + scalar_prefix, 0, 0, 0, 0, 0],
            Self::Simd if !routed && n >= ADMITTED_PACKET_LANES => {
                [1, n / ADMITTED_PACKET_LANES, n, 0, 0, 0]
            }
            Self::Simd => [n + scalar_prefix, 0, 0, 0, 0, 0],
            Self::Immutable => [scalar_prefix, 0, 0, 0, n, 0],
        };
        assert_eq!(work, expected, "actual work differs from admitted route");
    }
}

fn percentile(values: &[u128], percentile: usize) -> u128 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() - 1) * percentile / 100]
}

fn hash_fields(instance: &EffectInstance) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for particle in instance.particles(EffectLayerId(1)) {
        for field in FIELDS {
            for word in particle.field(field).unwrap().words() {
                for byte in word.to_le_bytes() {
                    hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
                }
            }
        }
    }
    hash
}

fn main() {
    let args: Vec<_> = env::args().collect();
    assert_eq!(args.len(), 5,
        "usage: particle_delta_bench interpreter|scalar-jit|simd-jit|immutable-age density|routed N SAMPLES");
    let route = Route::parse(&args[1]);
    let routed = match args[2].as_str() {
        "density" => false,
        "routed" => true,
        _ => panic!("unknown workload"),
    };
    let children: usize = args[3].parse().unwrap();
    let samples: usize = args[4].parse().unwrap();
    assert!(samples >= 5);
    let bytes = fixture(routed, children);
    let begin = Instant::now();
    let document = Document::read(bytes).unwrap();
    let read_ns = begin.elapsed().as_nanos();
    let (root, _) = document.effect_roots().next().unwrap().unwrap();
    let begin = Instant::now();
    let source = document
        .compile_effect(root, &EffectCompileOptions::default())
        .unwrap();
    let compile_ns = begin.elapsed().as_nanos();
    let begin = Instant::now();
    let program = source.clone().with_execution(ExecutionOptions {
        preference: route.preference(),
        numeric_policy: NumericPolicy::Graphics,
    });
    let prepare_ns = begin.elapsed().as_nanos();
    route.verify_preparation(&program);

    // Full original source trajectory, including init/newborn prefixes, precedes
    // every clock. No midpoint override or artificially homogeneous birth state.
    let mut ordinary = start(&source, children, routed);
    let mut probe = start(&program, children, routed);
    compare(&probe, &ordinary);
    let mut ledger = Vec::with_capacity(STEPS);
    let mut expected_work = [0u64; 6];
    let mut previous = counters(probe.execution_stats());
    for frame in 0..WARM + STEPS {
        advance(&mut ordinary, &source);
        advance(&mut probe, &program);
        compare(&probe, &ordinary);
        oracle(&probe, children, routed);
        let current = counters(probe.execution_stats());
        let work = delta(current, previous);
        previous = current;
        if frame >= WARM {
            route.verify_work(children, routed, work);
            expected_work = from_fn(|i| expected_work[i] + work[i]);
            let stats = probe.stats();
            ledger.push((
                probe.time(),
                probe.active_count(),
                stats.spawned,
                stats.retired,
                work,
            ));
        }
    }
    let expected_hash = hash_fields(&probe);
    let fields: &[EffectFieldId] = if routed {
        &[EffectFieldId(10)]
    } else {
        &[EffectFieldId(8), EffectFieldId(10)]
    };
    let begin = Instant::now();
    let projection = program
        .immutable_recipe(EffectLayerId(1))
        .map(|recipe| recipe.project(fields).unwrap());
    let projection_ns = begin.elapsed().as_nanos();
    let mut projection_scratch = projection.as_ref().map(|p| p.new_scratch());
    let mut full_values = [Value::from(0f32); 14];
    let mut selected_values = vec![Value::from(0f32); fields.len()];
    // Query implementations must preserve the same entire value, including bits.
    for particle in probe.particles(EffectLayerId(1)) {
        particle.sample_fields(&FIELDS, &mut full_values).unwrap();
        for (field, value) in FIELDS.into_iter().zip(full_values) {
            assert_eq!(Some(value), particle.field(field));
        }
        if let (Some(projection), Some(scratch)) = (&projection, &mut projection_scratch) {
            projection
                .evaluate(
                    particle.immutable_sample().unwrap(),
                    scratch,
                    &mut selected_values,
                )
                .unwrap();
            for (&field, &value) in fields.iter().zip(&selected_values) {
                assert_eq!(Some(value), particle.field(field));
            }
        }
    }
    let mut frames = Vec::with_capacity(samples * STEPS);
    let mut setup_ns = Vec::with_capacity(samples);
    let mut instantiate_ns = Vec::with_capacity(samples);
    let mut start_birth_ns = Vec::with_capacity(samples);
    let mut full_query_ns = Vec::with_capacity(samples);
    let mut selected_query_ns = Vec::with_capacity(samples);
    let mut measured_work = Vec::with_capacity(samples);
    let mut hot_allocations = Vec::with_capacity(samples);
    let mut query_allocations = Vec::with_capacity(samples);
    for _ in 0..samples {
        let begin = Instant::now();
        let allocation_begin = Instant::now();
        let mut instance = program.instantiate(limits(children, routed), 42).unwrap();
        instantiate_ns.push(allocation_begin.elapsed().as_nanos());
        let birth_begin = Instant::now();
        instance
            .start(&program, program.default_entry(), SceneInputs::default())
            .unwrap();
        advance(&mut instance, &program);
        start_birth_ns.push(birth_begin.elapsed().as_nanos());
        for _ in 1..WARM {
            advance(&mut instance, &program);
        }
        setup_ns.push(begin.elapsed().as_nanos());
        oracle(&instance, children, routed);
        let before = counters(instance.execution_stats());
        let allocations_before = allocations();
        for _ in 0..STEPS {
            let begin = Instant::now();
            advance(&mut instance, &program);
            frames.push(begin.elapsed().as_nanos());
        }
        let allocation_delta: [u64; 3] = from_fn(|i| allocations()[i] - allocations_before[i]);
        assert_eq!(
            allocation_delta, [0; 3],
            "hot advancement allocated/reallocated/freed"
        );
        hot_allocations.push(allocation_delta);
        let work = delta(counters(instance.execution_stats()), before);
        assert_eq!(work, expected_work);
        measured_work.push(work);
        oracle(&instance, children, routed);
        compare(&instance, &ordinary);
        assert_eq!(hash_fields(&instance), expected_hash);

        let allocations_before = allocations();
        let begin = Instant::now();
        for _ in 0..8 {
            for particle in instance.particles(EffectLayerId(1)) {
                particle.sample_fields(&FIELDS, &mut full_values).unwrap();
                black_box(&full_values);
            }
        }
        full_query_ns.push(begin.elapsed().as_nanos());
        let begin = Instant::now();
        for _ in 0..8 {
            for particle in instance.particles(EffectLayerId(1)) {
                if let (Some(projection), Some(scratch)) = (&projection, &mut projection_scratch) {
                    projection
                        .evaluate(
                            particle.immutable_sample().unwrap(),
                            scratch,
                            &mut selected_values,
                        )
                        .unwrap();
                } else {
                    particle
                        .sample_fields(fields, &mut selected_values)
                        .unwrap();
                }
                black_box(&selected_values);
            }
        }
        selected_query_ns.push(begin.elapsed().as_nanos());
        let allocation_delta: [u64; 3] = from_fn(|i| allocations()[i] - allocations_before[i]);
        assert_eq!(
            allocation_delta, [0; 3],
            "hot queries allocated/reallocated/freed"
        );
        query_allocations.push(allocation_delta);
    }
    let ledger_json = format!(
        "[{}]",
        ledger
            .iter()
            .map(|(time, active, spawned, retired, work)| format!(
                "[{time},{active},{spawned},{retired},{work:?}]"
            ))
            .collect::<Vec<_>>()
            .join(",")
    );
    println!("route={:?}", program.execution_reports());
    println!("{{\"kind\":\"contract\",\"debug_assertions\":{},\"route\":\"{}\",\"workload\":\"{}\",\"requested_children_or_occurrences\":{children},\"capacity_total\":{},\"dt\":{DT},\"warmup\":{WARM},\"steps\":{STEPS},\"samples\":{samples},\"abs\":{},\"rel\":{},\"ledger_columns\":[\"time\",\"active\",\"spawned\",\"retired\",\"work\"],\"work_columns\":[\"scalar_jit\",\"simd_batches\",\"simd_lanes\",\"rust_lanes\",\"immutable_age\",\"immutable_fallbacks\"],\"ledger\":{ledger_json},\"measured_work\":{measured_work:?},\"final_candidate_hash\":{expected_hash},\"source_hash\":{}}}",
        cfg!(debug_assertions), args[1], args[2], limits(children, routed).particles, ABS, REL, hash_fields(&ordinary));
    println!("{{\"kind\":\"result\",\"read_ns\":{read_ns},\"compile_ns\":{compile_ns},\"prepare_ns\":{prepare_ns},\"projection_prepare_ns\":{projection_ns},\"instantiate_ns\":{instantiate_ns:?},\"start_and_first_birth_tick_ns\":{start_birth_ns:?},\"setup_ns\":{setup_ns:?},\"frames_ns\":{frames:?},\"frame_median_ns\":{},\"frame_p95_ns\":{},\"eight_full_query_sweeps_ns\":{full_query_ns:?},\"eight_selected_query_sweeps_ns\":{selected_query_ns:?},\"selected_query_projected\":{},\"hot_allocation_deltas\":{hot_allocations:?},\"query_allocation_deltas\":{query_allocations:?}}}",
        percentile(&frames, 50), percentile(&frames, 95), projection.is_some());
}
