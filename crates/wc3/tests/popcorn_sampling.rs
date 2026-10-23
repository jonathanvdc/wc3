//! Public multi-output preparation, fixed ownership and no-allocation sampling.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::scene::popcorn::{
    EffectFieldId, EffectInstance, EffectLayerId, EffectLimits, EffectProgram, ExecutionOptions,
    ExecutionPreference, FieldSamplingError, ImmutableProjectionError, NumericPolicy, SceneInputs,
    Value,
};

struct Meter;
thread_local! {
    static MEASURING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}
fn allocated() {
    let _ = MEASURING.try_with(|enabled| {
        if enabled.get() {
            let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
        }
    });
}
unsafe impl GlobalAlloc for Meter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        allocated();
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        allocated();
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, bytes: usize) -> *mut u8 {
        allocated();
        unsafe { System.realloc(ptr, layout, bytes) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}
#[global_allocator]
static ALLOCATOR: Meter = Meter;
struct Measurement;
impl Measurement {
    fn begin() -> Self {
        ALLOCATIONS.with(|count| count.set(0));
        MEASURING.with(|enabled| enabled.set(true));
        Self
    }
    fn finish(self) -> usize {
        MEASURING.with(|enabled| enabled.set(false));
        ALLOCATIONS.with(Cell::get)
    }
}
impl Drop for Measurement {
    fn drop(&mut self) {
        let _ = MEASURING.try_with(|enabled| enabled.set(false));
    }
}
fn program(preference: ExecutionPreference) -> EffectProgram {
    let document = Document::read(include_bytes!("fixtures/pkb/immutable-8.pkb")).unwrap();
    let (root, _) = document.effect_roots().next().unwrap().unwrap();
    document
        .compile_effect(root, &EffectCompileOptions::default())
        .unwrap()
        .with_execution(ExecutionOptions {
            preference,
            numeric_policy: NumericPolicy::Graphics,
        })
}
fn instance(program: &EffectProgram) -> EffectInstance {
    let mut instance = program
        .instantiate(
            EffectLimits {
                particles: 16,
                step_seconds: 1. / 64.,
                ..Default::default()
            },
            42,
        )
        .unwrap();
    instance
        .start(program, program.default_entry(), SceneInputs::default())
        .unwrap();
    instance
        .advance(program, 2. / 64., SceneInputs::default())
        .unwrap();
    instance
}

#[test]
fn mixed_field_sampling_matches_individual_reads_without_advancing_or_allocating() {
    let program = program(ExecutionPreference::ImmutableAge);
    let recipe = program.immutable_recipe(EffectLayerId(1)).unwrap();
    let selected = [EffectFieldId(12), EffectFieldId(9), EffectFieldId(12)];
    let projection = recipe.project(&selected).unwrap();
    let mut scratch = projection.new_scratch();
    let instance = instance(&program);
    let particle = instance.particles(EffectLayerId(1)).next().unwrap();
    let fields = [
        EffectFieldId(0),
        EffectFieldId(9),
        EffectFieldId(11),
        EffectFieldId(12),
        EffectFieldId(3),
        EffectFieldId(9),
    ];
    let expected = fields.map(|field| particle.field(field).unwrap());
    let expected_projection = selected.map(|field| particle.field(field).unwrap());
    let mut mixed = [Value::from(-1f32); 6];
    let mut projected = [Value::from(-1f32); 3];
    let before = instance.execution_stats();
    let observation = particle.immutable_sample().unwrap().sample().effect_time;
    let measurement = Measurement::begin();
    for _ in 0..128 {
        particle.sample_fields(&fields, &mut mixed).unwrap();
        projection
            .evaluate(
                particle.immutable_sample().unwrap(),
                &mut scratch,
                &mut projected,
            )
            .unwrap();
    }
    let allocations = measurement.finish();
    assert_eq!(allocations, 0);
    assert_eq!(mixed, expected);
    assert_eq!(projected, expected_projection);
    assert_eq!(instance.execution_stats(), before);
    assert_eq!(
        particle.immutable_sample().unwrap().sample().effect_time,
        observation
    );
    assert_eq!(instance.time(), observation);
    drop(program);
    projection
        .evaluate(
            particle.immutable_sample().unwrap(),
            &mut scratch,
            &mut projected,
        )
        .unwrap();
    assert_eq!(projected, expected_projection);
}

#[test]
fn invalid_requests_are_atomic_and_raw_fallback_includes_initialized_newborns() {
    for preference in [
        ExecutionPreference::Interpreter,
        ExecutionPreference::ImmutableAge,
    ] {
        let program = program(preference);
        let mut instance = program
            .instantiate(
                EffectLimits {
                    particles: 16,
                    step_seconds: 1. / 64.,
                    ..Default::default()
                },
                42,
            )
            .unwrap();
        instance
            .start(&program, program.default_entry(), SceneInputs::default())
            .unwrap();
        instance
            .advance(&program, 1. / 64., SceneInputs::default())
            .unwrap();
        let mut initial = 0;
        for particle in instance.particles(EffectLayerId(1)) {
            if particle.immutable_sample().is_none() {
                initial += 1;
            }
            let fields = [EffectFieldId(12), EffectFieldId(0), EffectFieldId(12)];
            let expected = fields.map(|field| particle.field(field).unwrap());
            let mut result = [Value::from(-1f32); 3];
            particle.sample_fields(&fields, &mut result).unwrap();
            assert_eq!(result, expected);
            let sentinel = [Value::from(-77f32); 2];
            let mut result = sentinel;
            assert_eq!(
                particle.sample_fields(&[EffectFieldId(12), EffectFieldId(u32::MAX)], &mut result),
                Err(FieldSamplingError::UnknownField(EffectFieldId(u32::MAX)))
            );
            assert_eq!(result, sentinel);
            assert!(matches!(
                particle.sample_fields(&[EffectFieldId(12)], &mut result),
                Err(FieldSamplingError::OutputLength { .. })
            ));
            assert_eq!(result, sentinel);
            particle.sample_fields(&[], &mut []).unwrap();
        }
        assert!(
            initial > 0,
            "endpoint birth must exercise raw initialized values"
        );
    }
}

#[test]
fn captured_recipe_identity_and_budget_fallback_keep_authorized_values() {
    let program = program(ExecutionPreference::ImmutableAge);
    let projection = program
        .immutable_recipe(EffectLayerId(1))
        .unwrap()
        .project(&[EffectFieldId(9), EffectFieldId(12)])
        .unwrap();
    let mut scratch = projection.new_scratch();
    let mut instance = instance(&program);
    let replacement = program.clone().with_execution(ExecutionOptions {
        preference: ExecutionPreference::ImmutableAge,
        numeric_policy: NumericPolicy::Graphics,
    });
    assert!(replacement.same_program(&program));
    let foreign = self::program(ExecutionPreference::ImmutableAge);
    let wrong = foreign
        .immutable_recipe(EffectLayerId(1))
        .unwrap()
        .project(&[EffectFieldId(9), EffectFieldId(12)])
        .unwrap();
    let mut wrong_scratch = wrong.new_scratch();
    let fields = [EffectFieldId(9), EffectFieldId(12)];
    let before: Vec<_> = instance
        .particles(EffectLayerId(1))
        .map(|particle| {
            let sample = particle.immutable_sample().unwrap();
            let mut values = [Value::from(0f32); 2];
            projection
                .evaluate(sample, &mut scratch, &mut values)
                .unwrap();
            assert_eq!(
                wrong.evaluate(sample, &mut wrong_scratch, &mut values),
                Err(ImmutableProjectionError::RecipeMismatch)
            );
            (particle.id, values)
        })
        .collect();
    // Explicitly denied work advances lifecycle but must not invent a newer
    // appearance sample. Core restores the last authorized values and latches.
    instance
        .advance_with_step_budget(&replacement, 1. / 64., SceneInputs::default(), || false)
        .unwrap();
    assert!(!instance.immutable_active());
    assert!(instance.stats().skipped_seconds > 0.);
    assert_eq!(instance.execution_stats().immutable_age_fallbacks, 1);
    assert_eq!(instance.particles(EffectLayerId(1)).count(), before.len());
    for (particle, (id, expected)) in instance.particles(EffectLayerId(1)).zip(&before) {
        assert_eq!(particle.id, *id);
        assert!(particle.immutable_sample().is_none());
        let mut values = [Value::from(0f32); 2];
        particle.sample_fields(&fields, &mut values).unwrap();
        assert_eq!(values, *expected);
    }
    drop(program);
    drop(replacement);
    // Restart ownership and all preparation handles may die while the instance
    // and retained projection still safely own their immutable allocations.
    assert!(projection.node_count() > 0);
}

#[test]
fn live_execution_plan_cache_hits_allocate_nothing() {
    let source = program(ExecutionPreference::Interpreter);
    let options = ExecutionOptions {
        preference: ExecutionPreference::ImmutableAge,
        numeric_policy: NumericPolicy::Graphics,
    };
    let prepared = source.clone().with_execution(options);
    let recipe = prepared.immutable_recipe(EffectLayerId(1)).unwrap();
    let measured = Measurement::begin();
    for _ in 0..64 {
        let hit = source.clone().with_execution(options);
        assert!(recipe.same_recipe(&hit.immutable_recipe(EffectLayerId(1)).unwrap()));
        let unchanged = hit.with_execution(options);
        assert!(unchanged.same_program(&source));
    }
    assert_eq!(measured.finish(), 0);
}
