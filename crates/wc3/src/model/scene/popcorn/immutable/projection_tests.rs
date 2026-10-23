use super::*;
use crate::model::formats::pkb::{Document, EffectCompileOptions};
use crate::model::scene::popcorn::{
    EffectInstance, EffectLayerId, EffectLimits, EffectProgram, ExecutionOptions,
    ExecutionPreference, ImmutableExpression, NumericPolicy, SceneInputs, ValueType,
};

fn program() -> EffectProgram {
    let document = Document::read(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/pkb/immutable-8.pkb"
    )))
    .unwrap();
    let (root, _) = document.effect_roots().next().unwrap().unwrap();
    document
        .compile_effect(root, &EffectCompileOptions::default())
        .unwrap()
        .with_execution(ExecutionOptions {
            preference: ExecutionPreference::ImmutableAge,
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
    assert!(instance.immutable_active());
    instance
}

#[test]
fn selected_dependency_closure_preserves_requested_order_duplicates_and_all_values() {
    let program = program();
    let recipe = program.immutable_recipe(EffectLayerId(1)).unwrap();
    let position = recipe.project(&[EffectFieldId(8)]).unwrap();
    assert_eq!(position.node_count(), 1);
    assert_eq!(position.capture_count(), 1);
    assert!(position.node_count() < recipe.node_count());
    let requested = [
        EffectFieldId(12),
        EffectFieldId(9),
        EffectFieldId(12),
        EffectFieldId(11),
    ];
    let projection = recipe.project(&requested).unwrap();
    assert_eq!(projection.fields().collect::<Vec<_>>(), requested);
    assert_eq!(projection.output_count(), requested.len());
    assert!(projection.node_count() < recipe.node_count());
    assert!(projection.capture_count() < recipe.capture_count());
    let mut scratch = projection.new_scratch();
    let instance = instance(&program);
    for particle in instance.particles(EffectLayerId(1)) {
        let mut values = [Value::from(-777f32); 4];
        projection
            .evaluate(
                particle.immutable_sample().unwrap(),
                &mut scratch,
                &mut values,
            )
            .unwrap();
        for (&field, &value) in requested.iter().zip(&values) {
            assert_eq!(Some(value), particle.field(field));
        }
    }
    // Requested duplicates are not bounded by the source's distinct output cap.
    let repeated = vec![EffectFieldId(9); 70];
    let duplicate_projection = recipe.project(&repeated).unwrap();
    let mut scratch = duplicate_projection.new_scratch();
    let mut values = vec![Value::from(0f32); 70];
    let particle = instance.particles(EffectLayerId(1)).next().unwrap();
    duplicate_projection
        .evaluate(
            particle.immutable_sample().unwrap(),
            &mut scratch,
            &mut values,
        )
        .unwrap();
    assert!(values
        .iter()
        .all(|value| Some(*value) == particle.field(EffectFieldId(9))));
}

#[test]
fn projection_identity_errors_and_buffer_validation_never_publish_partial_outputs() {
    let program = program();
    let recipe = program.immutable_recipe(EffectLayerId(1)).unwrap();
    let projection = recipe.project(&[EffectFieldId(12)]).unwrap();
    let empty = recipe.project(&[]).unwrap();
    assert_eq!(
        (
            empty.node_count(),
            empty.capture_count(),
            empty.output_count()
        ),
        (0, 0, 0)
    );
    assert!(matches!(
        recipe.project(&[EffectFieldId(0)]),
        Err(ImmutableProjectionError::UnknownOutput(EffectFieldId(0)))
    ));
    let source_weak = Arc::downgrade(recipe.plan);
    let first = instance(&program);
    let other_program = self::program();
    assert!(!other_program.same_program(&program));
    let other = instance(&other_program);
    let view = first
        .particles(EffectLayerId(1))
        .next()
        .unwrap()
        .immutable_sample()
        .unwrap();
    let foreign = other
        .particles(EffectLayerId(1))
        .next()
        .unwrap()
        .immutable_sample()
        .unwrap();
    let sentinel = Value::from_words(ValueType::Int, [0x12345678, 0, 0, 0]);
    let mut outputs = [sentinel];
    let mut scratch = projection.new_scratch();
    assert_eq!(
        projection.evaluate(foreign, &mut scratch, &mut outputs),
        Err(ImmutableProjectionError::RecipeMismatch)
    );
    assert_eq!(outputs, [sentinel]);
    assert_eq!(
        empty.evaluate(foreign, &mut scratch, &mut []),
        Err(ImmutableProjectionError::RecipeMismatch)
    );
    assert!(matches!(
        projection.evaluate(view, &mut scratch, &mut []),
        Err(ImmutableProjectionError::OutputLength { .. })
    ));
    let mut too_small = empty.new_scratch();
    assert!(matches!(
        projection.evaluate(view, &mut too_small, &mut outputs),
        Err(ImmutableProjectionError::ScratchTooSmall { .. })
    ));
    assert_eq!(outputs, [sentinel]);
    empty.evaluate(view, &mut scratch, &mut []).unwrap();
    let mut invalid = view;
    invalid.sample.life = f32::NAN;
    assert!(matches!(
        projection.evaluate(invalid, &mut scratch, &mut outputs),
        Err(ImmutableProjectionError::Evaluation { .. })
    ));
    assert_eq!(outputs, [sentinel]);
    projection
        .evaluate(view, &mut scratch, &mut outputs)
        .unwrap();
    assert_eq!(
        Some(outputs[0]),
        first
            .particles(EffectLayerId(1))
            .next()
            .unwrap()
            .field(EffectFieldId(12))
    );
    drop(program);
    drop(first);
    assert!(
        source_weak.upgrade().is_some(),
        "owned projection retains source allocation and prevents identity ABA"
    );
}

#[test]
fn compact_views_preserve_capture_mapping_samples_and_borrowed_curve_resources() {
    let owner = program();
    let first = instance(&owner);
    let source_recipe = first.immutable_recipe(EffectLayerId(1)).unwrap();
    let requested = [EffectFieldId(12), EffectFieldId(9), EffectFieldId(12)];
    let projection = source_recipe.project(&requested).unwrap();
    let position = source_recipe.project(&[EffectFieldId(8)]).unwrap();
    // Instances retain the immutable graph's samplers independently of wrappers.
    drop(owner);
    let compact = projection.recipe(source_recipe).unwrap();
    assert_eq!(compact.node_count(), projection.node_count());
    assert_eq!(compact.capture_count(), projection.capture_count());
    assert_eq!(compact.output_count(), requested.len());
    assert!(!compact.same_recipe(&source_recipe));
    let mut saw_curve = false;
    for index in 0..compact.node_count() {
        if let ImmutableExpression::Curve { sampler, .. } = compact.node(index).unwrap().expression
        {
            let original = source_recipe.curve(sampler).unwrap();
            let retained = compact.curve(sampler).unwrap();
            assert_eq!(retained.kind, original.kind);
            assert_eq!(retained.times.as_ptr(), original.times.as_ptr());
            assert_eq!(retained.values.as_ptr(), original.values.as_ptr());
            assert_eq!(retained.tangents.as_ptr(), original.tangents.as_ptr());
            saw_curve = true;
        }
    }
    assert!(saw_curve, "this control must retain a real immutable curve");
    let nested = compact.project(&[EffectFieldId(12)]).unwrap();
    let mut nested_scratch = nested.new_scratch();
    let mut scratch = projection.new_scratch();
    for particle in first.particles(EffectLayerId(1)) {
        let source = particle.immutable_sample().unwrap();
        let derived = projection.view(source).unwrap();
        assert!(derived.recipe().same_recipe(&compact));
        assert_eq!(derived.slot, source.slot);
        assert_eq!(derived.birth.time.to_bits(), source.birth.time.to_bits());
        assert_eq!(derived.birth.life.to_bits(), source.birth.life.to_bits());
        assert_eq!(
            derived.birth.inverse_life.to_bits(),
            source.birth.inverse_life.to_bits()
        );
        assert_eq!(
            derived.sample.effect_time.to_bits(),
            source.sample.effect_time.to_bits()
        );
        assert_eq!(derived.sample.life.to_bits(), source.sample.life.to_bits());
        for index in 0..compact.capture_count() {
            let capture = compact.capture(index).unwrap();
            let original_index = (0..source_recipe.capture_count())
                .find(|&source_index| {
                    let original = source_recipe.capture(source_index).unwrap();
                    original.field == capture.field && original.kind == capture.kind
                })
                .unwrap();
            assert_eq!(derived.capture(index), source.capture(original_index));
        }
        assert_eq!(derived.capture(compact.capture_count()), None);
        let mut values = [Value::from(-1f32); 3];
        projection
            .evaluate(derived, &mut scratch, &mut values)
            .unwrap();
        assert_eq!(
            values,
            requested.map(|field| particle.field(field).unwrap())
        );
        let mut nested_value = [Value::from(-1f32)];
        nested
            .evaluate(
                nested.view(derived).unwrap(),
                &mut nested_scratch,
                &mut nested_value,
            )
            .unwrap();
        assert_eq!(Some(nested_value[0]), particle.field(EffectFieldId(12)));
        assert!(matches!(
            nested.view(source),
            Err(ImmutableProjectionError::RecipeMismatch)
        ));
        assert!(matches!(
            projection.view(position.view(source).unwrap()),
            Err(ImmutableProjectionError::RecipeMismatch)
        ));
        assert!(matches!(
            projection.recipe(position.recipe(source_recipe).unwrap()),
            Err(ImmutableProjectionError::RecipeMismatch)
        ));
        assert_eq!(
            projection.evaluate(position.view(source).unwrap(), &mut scratch, &mut values),
            Err(ImmutableProjectionError::RecipeMismatch)
        );
    }
    let foreign_program = program();
    let foreign = instance(&foreign_program);
    let foreign_recipe = foreign.immutable_recipe(EffectLayerId(1)).unwrap();
    assert!(matches!(
        projection.recipe(foreign_recipe),
        Err(ImmutableProjectionError::RecipeMismatch)
    ));
    let foreign_view = foreign
        .particles(EffectLayerId(1))
        .next()
        .unwrap()
        .immutable_sample()
        .unwrap();
    assert!(matches!(
        projection.view(foreign_view),
        Err(ImmutableProjectionError::RecipeMismatch)
    ));
}
