//! Renderer-independent native graph and pooled lifecycle integration checks.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::path::PathBuf;
use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::scene::popcorn::{
    EffectFieldId, EffectLayerId, EffectLimits, EffectProgram, EffectRandom, SceneInputs, ValueType,
};

thread_local! {
    static TRACKING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}
struct CountingAllocator;
fn allocated() {
    if TRACKING.try_with(Cell::get).unwrap_or(false) {
        let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
    }
}
// Counting is thread-local so parallel test-harness allocations do not contaminate a simulation.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        allocated();
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        allocated();
        unsafe { System.realloc(pointer, layout, size) }
    }
}
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;
fn without_allocations(run: impl FnOnce()) {
    ALLOCATIONS.set(0);
    TRACKING.set(true);
    run();
    TRACKING.set(false);
    assert_eq!(
        ALLOCATIONS.get(),
        0,
        "simulation allocated after pool creation"
    );
}
fn sprite() -> EffectProgram {
    Document::read(include_bytes!("fixtures/pkb/sprite.pkb"))
        .unwrap()
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap()
}
#[test]
fn original_native_fixture_exposes_checked_renderer_and_canonical_values() {
    let bytes = include_bytes!("fixtures/pkb/sprite.pkb");
    let document = Document::read(bytes).unwrap();
    assert_eq!(document.effect_roots().count(), 1);
    let program = sprite();
    assert_eq!(program.default_entry(), 0);
    assert_eq!(program.layers().len(), 1);
    assert_eq!(
        program.renderers()[0].primary_image(),
        Some(b"sprite.png".as_slice())
    );
    let mut instance = program.instantiate(EffectLimits::default(), 42).unwrap();
    instance.start(&program, 0, SceneInputs::default()).unwrap();
    let particle = instance.particles(EffectLayerId(0)).next().unwrap();
    assert_eq!(
        particle.field(EffectFieldId(2)).unwrap().kind(),
        ValueType::Float3
    );
    assert_eq!(
        f32::from_bits(particle.field(EffectFieldId(3)).unwrap().words()[0]),
        2.
    );
    assert!(particle.field(EffectFieldId(99)).is_none());
}
#[test]
fn pooled_start_advance_retire_and_restart_allocate_nothing() {
    let program = sprite();
    let mut instance = program
        .instantiate(
            EffectLimits {
                particles: 8,
                ..Default::default()
            },
            42,
        )
        .unwrap();
    without_allocations(|| {
        for _ in 0..3 {
            instance.start(&program, 0, SceneInputs::default()).unwrap();
            for _ in 0..150 {
                instance
                    .advance(&program, 1. / 60., SceneInputs::default())
                    .unwrap();
            }
            instance.stop();
            assert!(instance.is_finished());
        }
    });
}
#[test]
#[ignore = "requires the separately acquired HD LichAttack dependency"]
fn actual_lich_graph_initializes_generates_stops_and_retires_without_allocation() {
    let path=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/dependencies/files/war3.w3mod/_hd.w3mod/sharedfx/lich/lichattack--f9ecf5d6868f.pkb");
    let bytes = std::fs::read(path).expect("acquire the HD LichAttack PKB dependency");
    let document = Document::read(&bytes).unwrap();
    let program = document
        .compile_effect(
            1,
            &EffectCompileOptions {
                random: EffectRandom::Midpoint,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        (
            program.layers().len(),
            program.renderers().len(),
            program.attributes().len()
        ),
        (4, 1, 7)
    );
    let mut instance = program
        .instantiate(
            EffectLimits {
                particles: 128,
                ..Default::default()
            },
            42,
        )
        .unwrap();
    without_allocations(|| {
        instance
            .start(&program, program.default_entry(), SceneInputs::default())
            .unwrap();
        assert_eq!(instance.active_count(), 2);
        for _ in 0..120 {
            instance
                .advance(&program, 1. / 60., SceneInputs::default())
                .unwrap();
        }
        assert_eq!(instance.stats().spawned, 23);
        assert_eq!(instance.stats().retired, 14);
        assert_eq!(instance.particles(EffectLayerId(3)).count(), 7);
        instance.stop();
        for _ in 0..90 {
            instance
                .advance(&program, 1. / 60., SceneInputs::default())
                .unwrap();
        }
        assert!(instance.is_finished());
        assert_eq!(instance.stats().spawned, instance.stats().retired);
        assert_eq!(instance.stats().omitted_births, 0);
        assert_eq!(instance.stats().omitted_executions, 0);
    });
}

#[test]
fn ergonomic_values_preserve_scalar_vector_quaternion_and_boolean_layouts() {
    use glam::{Quat, Vec2, Vec3, Vec4};
    use wc3::model::scene::popcorn::Value;
    assert_eq!(Value::from(true).words(), [u32::MAX, 0, 0, 0]);
    assert_eq!(Value::from(false).words(), [0; 4]);
    assert_eq!(Value::from(-7i32).words()[0] as i32, -7);
    assert_eq!(Value::from(-0f32).words()[0], (-0f32).to_bits());
    for (value, kind, expected) in [
        (
            Value::from(Vec2::new(1., 2.)),
            ValueType::Float2,
            [1., 2., 0., 0.],
        ),
        (
            Value::from(Vec3::new(1., 2., 3.)),
            ValueType::Float3,
            [1., 2., 3., 0.],
        ),
        (
            Value::from(Vec4::new(1., 2., 3., 4.)),
            ValueType::Float4,
            [1., 2., 3., 4.],
        ),
        (
            Value::from(Quat::IDENTITY),
            ValueType::Quaternion,
            [0., 0., 0., 1.],
        ),
    ] {
        assert_eq!(value.kind(), kind);
        assert_eq!(value.words().map(f32::from_bits), expected);
    }
}
