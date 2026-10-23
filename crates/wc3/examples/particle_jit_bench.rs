//! Compare explicit scalar native execution with the strongest Rust evaluator.
use std::env;
use std::hint::black_box;
use std::time::Instant;
use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::scene::popcorn::{
    EffectFieldId, EffectInstance, EffectLayerId, ExecutionBackend, ExecutionOptions,
    ExecutionPreference, NumericPolicy, SceneInputs,
};

#[path = "../benches/support/particle_runtime.rs"]
mod support;
use support::{fixture, verify_frame, verify_trajectory, warm, DT, STEPS};

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
    assert_eq!(
        args.len(),
        6,
        "usage: jit-stage-bench interpreter|scalar-jit N C INSTANCES SAMPLES"
    );
    let jit = match args[1].as_str() {
        "interpreter" => false,
        "scalar-jit" => true,
        _ => panic!("unknown route"),
    };
    let children: usize = args[2].parse().unwrap();
    let capacity: usize = args[3].parse().unwrap();
    let count: usize = args[4].parse().unwrap();
    let samples: usize = args[5].parse().unwrap();
    assert!([1, 8, 128, 1024].contains(&children));
    assert!(capacity >= children && [128, 1024].contains(&capacity));
    assert!([1, 8].contains(&count) && samples >= 5);
    let bytes = fixture(children);
    let document = Document::read(bytes).unwrap();
    let (root, _) = document.effect_roots().next().unwrap().unwrap();
    let begin = Instant::now();
    let base = document
        .compile_effect(root, &EffectCompileOptions::default())
        .unwrap();
    let compile_ns = begin.elapsed().as_nanos();
    let begin = Instant::now();
    let program = base.with_execution(ExecutionOptions {
        preference: if jit {
            ExecutionPreference::Automatic
        } else {
            ExecutionPreference::Interpreter
        },
        numeric_policy: NumericPolicy::Graphics,
    });
    let prepare_ns = begin.elapsed().as_nanos();
    let expected = if jit {
        ExecutionBackend::ScalarJit
    } else {
        ExecutionBackend::Interpreter
    };
    assert!(!program.execution_reports().is_empty());
    for report in program.execution_reports() {
        assert_eq!(report.backend, expected, "requested route was not selected");
        assert!(
            report.fallback.is_none(),
            "fallback is not an admitted JIT measurement"
        );
    }
    verify_trajectory(&program, children, capacity);
    let mut invocation_deltas = Vec::with_capacity(STEPS);
    let mut route_probe = warm(&program, children, capacity);
    let mut previous = route_probe.execution_stats().scalar_jit_invocations;
    if jit {
        assert!(
            previous > 0,
            "selected JIT never entered native code during start/warmup"
        );
    } else {
        assert_eq!(previous, 0);
    }
    for _ in 0..STEPS {
        route_probe
            .advance(&program, DT, SceneInputs::default())
            .unwrap();
        let current = route_probe.execution_stats().scalar_jit_invocations;
        let delta = current
            .checked_sub(previous)
            .expect("invocation counter regressed");
        if jit {
            assert!(delta > 0, "no actual JIT invocation in advancing frame");
        } else {
            assert_eq!(delta, 0, "Interpreter unexpectedly entered JIT");
        }
        invocation_deltas.push(delta);
        previous = current;
    }
    println!("invocation_deltas={invocation_deltas:?}");
    let mut structural = Vec::with_capacity(STEPS + 1);
    let mut numeric = Vec::with_capacity((STEPS + 1) * children * 14);
    let mut admission = warm(&program, children, capacity);
    for frame in 0..=STEPS {
        verify_frame(&admission, children);
        structural.push(structure(&admission));
        for p in admission.particles(EffectLayerId(1)) {
            for index in 0..14 {
                numeric.push(p.field(EffectFieldId(index)).unwrap().words());
            }
        }
        if frame < STEPS {
            admission
                .advance(&program, DT, SceneInputs::default())
                .unwrap();
        }
    }
    // Allocation and destruction of each fixed instance set are outside clocks.
    // Full admission above scans every field/frame; timed endpoints repeat it.
    let mut blocks = Vec::with_capacity(samples);
    let mut frames = Vec::with_capacity(samples * STEPS);
    let mut setup = Vec::with_capacity(samples);
    for _ in 0..samples {
        let begin = Instant::now();
        let mut instances: Vec<_> = (0..count)
            .map(|_| warm(&program, children, capacity))
            .collect();
        setup.push(begin.elapsed().as_nanos());
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
        for instance in &instances {
            verify_frame(instance, children);
            let calls = instance.execution_stats().scalar_jit_invocations;
            if jit {
                assert!(calls > 0);
            } else {
                assert_eq!(calls, 0);
            }
            black_box(instance);
        }
    }
    println!("route={:?}", program.execution_reports());
    println!("{{\"kind\":\"result\",\"route\":\"{}\",\"children\":{},\"capacity_child_units\":{},\"capacity_total\":{},\"instances\":{},\"samples\":{},\"steps\":{},\"compile_ns\":{},\"prepare_ns\":{},\"block_median_ns\":{},\"block_p95_ns\":{},\"frame_median_ns\":{},\"frame_p95_ns\":{},\"setup_ns\":{:?},\"blocks_ns\":{:?},\"frames_ns\":{:?},\"structure\":{:?},\"numeric\":{:?}}}",
        args[1], children, capacity, capacity + 1, count, samples, STEPS, compile_ns, prepare_ns,
        percentile(&blocks, 50), percentile(&blocks, 95), percentile(&frames, 50), percentile(&frames, 95),
        setup, blocks, frames, structural, numeric);
}
