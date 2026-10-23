//! Original Delta workload admission and public-output oracle; no timed work.
use std::array::from_fn;
use wc3::model::scene::popcorn::{
    EffectFieldId, EffectInstance, EffectLayerId, EffectLimits, EffectProgram, ExecutionStats,
    SceneInputs, Value, ValueType,
};

pub(super) const DT: f64 = 1. / 64.;
pub(super) const WARM: usize = 32;
pub(super) const STEPS: usize = 64;
// Measured production default, deliberately asserted by the counter ledger.
// This is a benchmark admission contract, not a second configurable runtime.
pub(super) const ADMITTED_PACKET_LANES: u64 = 16;
// Lifetime / DT, independent of the packet width despite equal numeric values.
pub(super) const ROUTED_LIVE_CHILDREN: usize = 16;
pub(super) const FIELDS: [EffectFieldId; 14] = [
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
// Fixed before timing: Graphics allows reassociation but not structural changes.
pub(super) const ABS: f32 = 0.000_002;
pub(super) const REL: f32 = 0.000_002;

pub(super) fn fixture(routed: bool, children: usize) -> &'static [u8] {
    if routed {
        assert_eq!(
            children, 1,
            "routed argument is one occurrence per root tick"
        );
        return include_bytes!("../../tests/fixtures/pkb/delta-affine.pkb");
    }
    match children {
        1 => include_bytes!("../fixtures/particle_delta/density-1.pkb"),
        128 => include_bytes!("../fixtures/particle_delta/density-128.pkb"),
        1024 => include_bytes!("../fixtures/particle_delta/density-1024.pkb"),
        8192 => include_bytes!("../fixtures/particle_delta/density-8192.pkb"),
        _ => panic!("unsupported density"),
    }
}

pub(super) fn limits(children: usize, routed: bool) -> EffectLimits {
    let capacity = if routed { 32 } else { children + 1 };
    EffectLimits {
        particles: capacity,
        pending_events: capacity,
        occurrences: capacity,
        ..Default::default()
    }
}

pub(super) fn advance(instance: &mut EffectInstance, program: &EffectProgram) {
    instance
        .advance(program, DT, SceneInputs::default())
        .unwrap();
}

pub(super) fn start(program: &EffectProgram, children: usize, routed: bool) -> EffectInstance {
    let mut instance = program.instantiate(limits(children, routed), 42).unwrap();
    instance
        .start(program, program.default_entry(), SceneInputs::default())
        .unwrap();
    instance
}

pub(super) fn counters(stats: ExecutionStats) -> [u64; 6] {
    [
        stats.scalar_jit_invocations,
        stats.simd_attempted_batches,
        stats.simd_committed_lanes,
        stats.rust_packet_committed_lanes,
        stats.immutable_age_invocations,
        stats.immutable_age_fallbacks,
    ]
}

pub(super) fn delta(current: [u64; 6], previous: [u64; 6]) -> [u64; 6] {
    from_fn(|index| {
        current[index]
            .checked_sub(previous[index])
            .expect("counter regressed")
    })
}

pub(super) fn no_omissions(instance: &EffectInstance) {
    let s = instance.stats();
    assert_eq!(
        (s.omitted_births, s.omitted_executions, s.suppressed_births),
        (0, 0, 0)
    );
    assert_eq!(
        (s.omitted_spatial_writes, s.omitted_spatial_queries),
        (0, 0)
    );
    assert_eq!(s.skipped_seconds, 0.);
}

fn close(a: f32, b: f32) {
    assert!(a.is_finite() && b.is_finite());
    assert!(
        (a - b).abs() <= ABS + REL * a.abs().max(b.abs()),
        "{a} != {b}"
    );
}

pub(super) fn compare_value(actual: Value, expected: Value, exact: bool) {
    assert_eq!(actual.kind(), expected.kind());
    if exact || actual.kind() == ValueType::Bool {
        assert_eq!(actual.words(), expected.words());
    } else {
        for (a, b) in actual
            .words()
            .into_iter()
            .zip(expected.words())
            .take(actual.kind().components())
        {
            close(f32::from_bits(a), f32::from_bits(b));
        }
        assert!(actual.words()[actual.kind().components()..]
            .iter()
            .all(|word| *word == 0));
        assert!(expected.words()[expected.kind().components()..]
            .iter()
            .all(|word| *word == 0));
    }
}

// Includes every whole field, inactive word, identity, owner and time. Numeric
// lifetime fields stay bit-exact; only ordinary floating fields use Graphics bounds.
pub(super) fn compare(actual: &EffectInstance, source: &EffectInstance) {
    assert_eq!(actual.time().to_bits(), source.time().to_bits());
    assert_eq!(actual.active_count(), source.active_count());
    assert_eq!(
        (
            actual.is_running(),
            actual.is_emitting(),
            actual.is_finished()
        ),
        (
            source.is_running(),
            source.is_emitting(),
            source.is_finished()
        )
    );
    no_omissions(actual);
    no_omissions(source);
    let a = actual.stats();
    let b = source.stats();
    assert_eq!((a.spawned, a.retired), (b.spawned, b.retired));
    for layer in 0..2 {
        let layer = EffectLayerId(layer);
        let mut expected = source.particles(layer);
        for p in actual.particles(layer) {
            let q = expected.next().expect("extra particle");
            assert_eq!(
                (p.id, p.parent, p.birth_serial),
                (q.id, q.parent, q.birth_serial)
            );
            assert_eq!(p.birth_time.to_bits(), q.birth_time.to_bits());
            let fields = if layer == EffectLayerId(0) { 4 } else { 14 };
            for field in 0..fields {
                let id = EffectFieldId(field);
                compare_value(
                    p.field(id).unwrap(),
                    q.field(id).unwrap(),
                    field < 2 || layer == EffectLayerId(0),
                );
            }
        }
        assert!(expected.next().is_none(), "missing particle");
    }
}

pub(super) fn oracle(instance: &EffectInstance, children: usize, routed: bool) {
    no_omissions(instance);
    assert_eq!(instance.particles(EffectLayerId(0)).count(), 1);
    let mut count = 0;
    for p in instance.particles(EffectLayerId(1)) {
        count += 1;
        // One partial first invocation, followed by exact binary 1/64 intervals.
        let ticks = (instance.time() / DT).round() as u64;
        let birth_tick = (p.birth_time / DT).floor() as u64;
        let t = if routed {
            f64::from((DT * (1. - f64::from(0.4f32))) as f32) + (ticks - birth_tick - 1) as f64 * DT
        } else {
            f64::from((DT - p.birth_time) as f32) + (ticks - 1) as f64 * DT
        };
        if let Some(sample) = p.immutable_sample() {
            assert_eq!(sample.sample().integrated_delta, t);
        }
        let rotation = f32::from_bits(p.field(EffectFieldId(10)).unwrap().words()[0]);
        close(rotation, (3. + 2. * t) as f32);
        if !routed {
            let position = p.field(EffectFieldId(8)).unwrap().words();
            for (i, (base, velocity)) in [(0.125, 1.), (0.25, -0.5), (-0.5, 0.25)]
                .into_iter()
                .enumerate()
            {
                close(f32::from_bits(position[i]), (base + velocity * t) as f32);
            }
            assert_eq!(position[3], 0, "inactive Float3 word");
        }
    }
    let expected = if routed {
        ((instance.time() / DT).round() as usize).min(ROUTED_LIVE_CHILDREN)
    } else {
        children
    };
    assert_eq!(count, expected);
    assert_eq!(instance.active_count(), expected + 1);
    if !routed {
        assert_eq!(instance.stats().spawned, (children + 1) as u64);
        assert_eq!(instance.stats().retired, 0);
    }
}
