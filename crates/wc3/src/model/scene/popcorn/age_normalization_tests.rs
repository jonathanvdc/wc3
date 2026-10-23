use super::*;
use crate::model::scene::popcorn::packet::tests::fixture;

fn new_builder(layer: &EffectLayer) -> Builder<'_> {
    Builder {
        symbols: &[],
        layer,
        written: &[],
        invariant: &[],
        phases: &[],
        captures: Vec::new(),
        nodes: Vec::new(),
        node_requests: 0,
        memo: Vec::new(),
    }
}
fn constant(builder: &mut Builder<'_>, value: Value, offset: usize) -> NodeId {
    builder
        .node(Expression::Constant(value), value.kind(), offset)
        .unwrap()
}
fn finish(builder: Builder<'_>, output: NodeId) -> AgePlan {
    AgePlan {
        captures: builder.captures.into_boxed_slice(),
        nodes: builder.nodes.into_boxed_slice(),
        outputs: vec![Output {
            field: 3,
            value: output,
        }]
        .into_boxed_slice(),
        phases: 0,
        integrated_delta: false,
    }
}
fn value(plan: &AgePlan, captures: &[Value], life: f32) -> Result<Value, AgeError> {
    let mut scratch = [Value::from(0f32); MAX_NODES];
    let mut outputs = [Value::from(0f32)];
    plan.evaluate(
        captures,
        &[],
        AgeSample {
            integrated_delta: 0.,
            effect_time: 0.,
            life,
        },
        &mut scratch[..plan.nodes.len()],
        &mut outputs,
    )?;
    Ok(outputs[0])
}

#[test]
fn exact_value_cse_keeps_layout_bits_operand_order_and_earliest_source() {
    let program = fixture(1);
    let mut builder = new_builder(&program.inner.layers[0]);
    let plus = constant(&mut builder, Value::from(0f32), 90);
    assert_eq!(constant(&mut builder, Value::from(0f32), 20), plus);
    assert_eq!(builder.nodes[plus.0].offset, 20);
    assert_ne!(constant(&mut builder, Value::from(-0f32), 30), plus);
    assert_ne!(constant(&mut builder, Value::from(0i32), 40), plus);
    let nan0 = Value::from_words(ValueType::Float, [0x7fc0_0001, 0, 0, 0]);
    let nan1 = Value::from_words(ValueType::Float, [0x7fc0_0002, 0, 0, 0]);
    assert_ne!(
        constant(&mut builder, nan0, 50),
        constant(&mut builder, nan1, 60)
    );
    let a = builder
        .node(Expression::Capture(0), ValueType::Float, 70)
        .unwrap();
    let b = builder
        .node(Expression::Capture(1), ValueType::Float, 80)
        .unwrap();
    let first = builder
        .node(
            Expression::Binary {
                operation: BinaryOperation::Add,
                left: a,
                right: b,
            },
            ValueType::Float,
            100,
        )
        .unwrap();
    assert_eq!(
        builder
            .node(
                Expression::Binary {
                    operation: BinaryOperation::Add,
                    left: a,
                    right: b,
                },
                ValueType::Float,
                101
            )
            .unwrap(),
        first
    );
    assert_ne!(
        builder
            .node(
                Expression::Binary {
                    operation: BinaryOperation::Add,
                    left: b,
                    right: a,
                },
                ValueType::Float,
                102
            )
            .unwrap(),
        first,
        "no commutative reassociation changes the exact ordered expression"
    );
}

#[test]
fn constant_folding_uses_shared_component_arithmetic_and_words_normalization() {
    let program = fixture(1);
    for kind in [ValueType::Float, ValueType::Float3, ValueType::Float4] {
        for operation in [
            BinaryOperation::Add,
            BinaryOperation::Subtract,
            BinaryOperation::Multiply,
        ] {
            let mut builder = new_builder(&program.inner.layers[0]);
            let left = Value::from_words(
                kind,
                [
                    (-0.0f32).to_bits(),
                    2.0f32.to_bits(),
                    (-3.0f32).to_bits(),
                    0.5f32.to_bits(),
                ],
            );
            let right = Value::from(0.5f32);
            let a = constant(&mut builder, left, 0);
            let b = constant(&mut builder, right, 1);
            let output = builder
                .node(
                    Expression::Binary {
                        operation,
                        left: a,
                        right: b,
                    },
                    kind,
                    2,
                )
                .unwrap();
            let Expression::Constant(folded) = builder.nodes[output.0].expression else {
                panic!("finite literal arithmetic must fold")
            };
            let mut expected_words = [0; 4];
            for (component, word) in expected_words
                .iter_mut()
                .enumerate()
                .take(kind.components())
            {
                let left = f32::from_bits(left.words()[component]);
                *word = match operation {
                    BinaryOperation::Add => left + 0.5,
                    BinaryOperation::Subtract => left - 0.5,
                    BinaryOperation::Multiply => left * 0.5,
                    _ => unreachable!(),
                }
                .to_bits();
            }
            let expected = Value::from_words(kind, expected_words);
            assert_eq!(folded, expected);
            assert_eq!(value(&finish(builder, output), &[], 0.).unwrap(), expected);
        }
    }
    let mut builder = new_builder(&program.inner.layers[0]);
    let words = constant(
        &mut builder,
        Value::from_words(ValueType::Float4, [1, 2, 3, 4]),
        0,
    );
    let output = builder
        .node(
            Expression::Words {
                input: words,
                kind: ValueType::Bool3,
            },
            ValueType::Bool3,
            1,
        )
        .unwrap();
    assert_eq!(
        value(&finish(builder, output), &[], 0.).unwrap().words(),
        [1, 2, 3, 0]
    );
}

#[test]
fn overflow_and_zero_identities_cannot_erase_cold_finite_guards() {
    let program = fixture(1);
    let mut builder = new_builder(&program.inner.layers[0]);
    let huge = constant(&mut builder, Value::from(f32::MAX), 0);
    let two = constant(&mut builder, Value::from(2f32), 1);
    let overflow = builder
        .node(
            Expression::Binary {
                operation: BinaryOperation::Multiply,
                left: huge,
                right: two,
            },
            ValueType::Float,
            2,
        )
        .unwrap();
    assert!(matches!(
        builder.nodes[overflow.0].expression,
        Expression::Binary { .. }
    ));
    let zero = constant(&mut builder, Value::from(0f32), 3);
    let output = builder
        .node(
            Expression::Binary {
                operation: BinaryOperation::Multiply,
                left: overflow,
                right: zero,
            },
            ValueType::Float,
            4,
        )
        .unwrap();
    let plan = finish(builder, output);
    assert!(!plan.captures_permitted(|_| unreachable!()));
    assert!(f32::from_bits(value(&plan, &[], 0.).unwrap().words()[0]).is_nan());

    let mut builder = new_builder(&program.inner.layers[0]);
    let capture = builder.capture(2, 0).unwrap();
    let zero = constant(&mut builder, Value::from(0f32), 1);
    let output = builder
        .node(
            Expression::Binary {
                operation: BinaryOperation::Multiply,
                left: capture,
                right: zero,
            },
            ValueType::Float,
            2,
        )
        .unwrap();
    assert!(matches!(
        builder.nodes[output.0].expression,
        Expression::Binary { .. }
    ));
    let plan = finish(builder, output);
    assert!(!plan.captures_permitted(|_| Value::from(f32::INFINITY)));
}

#[test]
fn repeated_checked_curves_remain_distinct_even_with_unused_results() {
    let program = fixture(1);
    let mut builder = new_builder(&program.inner.layers[0]);
    let life = builder.node(Expression::Life, ValueType::Float, 0).unwrap();
    let first = builder
        .node(
            Expression::Curve {
                sampler: 0,
                input: life,
            },
            ValueType::Float4,
            17,
        )
        .unwrap();
    let second = builder
        .node(
            Expression::Curve {
                sampler: 0,
                input: life,
            },
            ValueType::Float4,
            99,
        )
        .unwrap();
    assert_ne!(first, second);
    let plan = finish(builder, second);
    assert_eq!(
        plan.nodes
            .iter()
            .filter(|node| matches!(node.expression, Expression::Curve { .. }))
            .count(),
        2
    );
    let mut scratch = [Value::from(0f32); MAX_NODES];
    let sentinel = Value::from(-71f32);
    let mut output = [sentinel];
    let error = plan
        .evaluate(
            &[],
            &program.inner.samplers,
            AgeSample {
                integrated_delta: 0.,
                effect_time: 0.,
                life: f32::NAN,
            },
            &mut scratch[..plan.nodes.len()],
            &mut output,
        )
        .unwrap_err();
    assert_eq!(
        error.offset,
        Some(17),
        "the unused first guard still owns its fault"
    );
    assert_eq!(output, [sentinel]);
}

#[test]
fn cse_preserves_original_construction_budget_before_node_deduplication() {
    let program = fixture(1);
    let mut builder = new_builder(&program.inner.layers[0]);
    for offset in 0..MAX_NODES {
        constant(&mut builder, Value::from(1f32), offset);
    }
    assert_eq!(builder.nodes.len(), 1);
    let error = builder
        .node(
            Expression::Constant(Value::from(1f32)),
            ValueType::Float,
            1234,
        )
        .unwrap_err();
    assert_eq!(
        error.reason,
        Reason::Limit {
            resource: Resource::Nodes,
            limit: MAX_NODES
        }
    );
    assert_eq!(error.offset, Some(1234));
}
