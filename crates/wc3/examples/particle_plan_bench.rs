//! Preparation-only comparison of exact-policy live reuse and weak-cache misses.
//! Run with `--features popcorn-jit --example particle_plan_bench -- [SAMPLES]`.
//! Fixture parsing/graph compilation, wrapper cloning, report checks and drops
//! are outside each prepare-only clock. No particle simulation is measured.
use md5::compute;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::env::{
    args,
    consts::{ARCH, OS},
};
use std::hint::black_box;
use std::time::Instant;
use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::scene::popcorn::{
    EffectLayerId, EffectProgram, ExecutionBackend, ExecutionOptions, ExecutionPreference,
    ExecutionStage, NumericPolicy,
};

const FIXTURE: &[u8] = include_bytes!("../tests/fixtures/pkb/immutable-128.pkb");

struct Tracking;
thread_local! {
    static TRACKING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<[u64; 3]> = const { Cell::new([0; 3]) };
}
fn record(index: usize) {
    let _ = TRACKING.try_with(|enabled| {
        if enabled.get() {
            let _ = ALLOCATIONS.try_with(|counts| {
                let mut values = counts.get();
                values[index] += 1;
                counts.set(values);
            });
        }
    });
}
unsafe impl GlobalAlloc for Tracking {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record(0);
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record(0);
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, bytes: usize) -> *mut u8 {
        record(1);
        unsafe { System.realloc(pointer, layout, bytes) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        record(2);
        unsafe { System.dealloc(pointer, layout) }
    }
}
#[global_allocator]
static ALLOCATOR: Tracking = Tracking;
struct AllocationScope;
impl AllocationScope {
    fn begin() -> Self {
        ALLOCATIONS.with(|counts| counts.set([0; 3]));
        TRACKING.with(|enabled| enabled.set(true));
        Self
    }
    fn finish(self) -> [u64; 3] {
        TRACKING.with(|enabled| enabled.set(false));
        ALLOCATIONS.with(Cell::get)
    }
}
impl Drop for AllocationScope {
    fn drop(&mut self) {
        let _ = TRACKING.try_with(|enabled| enabled.set(false));
    }
}

fn verify(program: &EffectProgram, source: &EffectProgram, options: ExecutionOptions) {
    assert!(program.same_program(source));
    assert_eq!(program.execution_options(), options);
    let mut selected_child = false;
    assert!(!program.execution_reports().is_empty());
    for report in program.execution_reports() {
        let child_evolution =
            report.layer == EffectLayerId(1) && report.stage == ExecutionStage::Evolution;
        selected_child |= child_evolution;
        let expected = match options.preference {
            ExecutionPreference::ImmutableAge if child_evolution => ExecutionBackend::ImmutableAge,
            ExecutionPreference::Automatic
                if child_evolution
                    && cfg!(any(target_arch = "x86_64", target_arch = "aarch64")) =>
            {
                ExecutionBackend::SimdJit
            }
            _ => ExecutionBackend::ScalarJit,
        };
        assert_eq!(
            report.backend, expected,
            "requested preparation was not admitted: {report:?}"
        );
        assert!(
            report.fallback.is_none(),
            "native fallback invalidates this comparison"
        );
    }
    assert!(selected_child);
    black_box(program.execution_reports());
}
struct Measurements {
    times: Vec<u128>,
    allocations: Vec<[u64; 3]>,
}
impl Measurements {
    fn median(&self) -> u128 {
        let mut sorted = self.times.clone();
        sorted.sort_unstable();
        sorted[(sorted.len() - 1) / 2]
    }
    fn report(&self, route: &str, case: &str) {
        println!("{{\"kind\":\"result\",\"route\":\"{route}\",\"case\":\"{case}\",\"median_ns\":{},\"prepare_ns\":{:?},\"calling_thread_alloc_realloc_dealloc\":{:?}}}", self.median(), self.times, self.allocations);
    }
}
fn measure(
    request_source: &EffectProgram,
    graph_source: &EffectProgram,
    options: ExecutionOptions,
    samples: usize,
    hot: bool,
) -> Measurements {
    let mut result = Measurements {
        times: Vec::with_capacity(samples),
        allocations: Vec::with_capacity(samples),
    };
    for _ in 0..samples {
        // A separate wrapper is consumed by every preparation, but its Arc
        // cloning is excluded equally for all three cases.
        let request = black_box(request_source.clone());
        let options = black_box(options);
        let allocation_scope = AllocationScope::begin();
        let begin = Instant::now();
        let prepared = request.with_execution(options);
        let elapsed = begin.elapsed().as_nanos();
        let allocations = allocation_scope.finish();
        verify(&prepared, graph_source, options);
        if hot {
            assert_eq!(allocations, [0; 3], "live plan reuse must allocate nothing");
        } else {
            assert!(
                allocations[0] > 0,
                "cold preparation unexpectedly reused a live plan"
            );
        }
        result.times.push(elapsed);
        result.allocations.push(allocations);
        // Destruction is deliberately outside the prepare-only clock. In the
        // cold case no prepared wrapper/instance remains: the cache is weak.
        drop(prepared);
    }
    result
}
fn main() {
    let arguments: Vec<_> = args().collect();
    assert!(arguments.len() <= 2, "usage: particle_plan_bench [SAMPLES]");
    let samples = arguments
        .get(1)
        .map_or(15, |value| value.parse::<usize>().unwrap());
    assert!(samples >= 5, "at least five samples are required");
    let document = Document::read(FIXTURE).unwrap();
    let (root, _) = document.effect_roots().next().unwrap().unwrap();
    let source = document
        .compile_effect(root, &EffectCompileOptions::default())
        .unwrap();
    assert_eq!(
        source.execution_options().preference,
        ExecutionPreference::Interpreter
    );
    println!("{{\"kind\":\"metadata\",\"crate_version\":\"{}\",\"os\":\"{OS}\",\"arch\":\"{ARCH}\",\"samples\":{samples},\"fixture\":\"tests/fixtures/pkb/immutable-128.pkb\",\"fixture_bytes\":{},\"fixture_md5\":\"{:x}\",\"source_md5\":\"{:x}\",\"allocation_scope\":\"calling thread, prepare only\",\"clock\":\"single prepare call; clone, observers and drop excluded\"}}", env!("CARGO_PKG_VERSION"), FIXTURE.len(), compute(FIXTURE), compute(include_bytes!("particle_plan_bench.rs")));
    for (route, preference) in [
        ("scalar-jit", ExecutionPreference::ScalarJit),
        ("simd-jit", ExecutionPreference::Automatic),
        ("immutable-age", ExecutionPreference::ImmutableAge),
    ] {
        let options = ExecutionOptions {
            preference,
            numeric_policy: NumericPolicy::Graphics,
        };
        // Prepare and release once to initialize compiler services and ensure
        // measured misses include the same expired-entry cleanup each time.
        let warm = source.clone().with_execution(options);
        verify(&warm, &source, options);
        drop(warm);
        measure(&source, &source, options, samples, false).report(route, "cold_prepare");
        let live = source.clone().with_execution(options);
        verify(&live, &source, options);
        measure(&live, &source, options, samples, true).report(route, "same_wrapper");
        measure(&source, &source, options, samples, true).report(route, "live_sibling_hit");
        drop(live);
    }
}
