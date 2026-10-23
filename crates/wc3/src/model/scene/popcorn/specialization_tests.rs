use super::*;
use crate::model::scene::popcorn::age::{AgePlan, MAX_NODES, MAX_OUTPUTS};
use crate::model::scene::popcorn::graph::{EffectProgram, External};
use crate::model::scene::popcorn::packet::{prepare, tests::fixture, PureEvolution};
use crate::model::scene::popcorn::{
    Address, BinaryOperation, Destination, ExecutionOptions, ExecutionPreference, NumericPolicy,
    Operation, Source,
};
use std::sync::Arc;

fn options(numeric_policy: NumericPolicy) -> ExecutionOptions {
    ExecutionOptions {
        preference: ExecutionPreference::ImmutableAge,
        numeric_policy,
    }
}
fn closed_fixture() -> EffectProgram {
    let mut program = fixture(1);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let layer = &mut graph.layers[0];
    let bound = layer.evolve.as_mut().unwrap();
    bound.externals[0] = External::Field(layer.life);
    bound.program.operations.truncate(10);
    // Leave the source curve coordinate directly Life, rather than Life+.125.
    drop(bound.program.operations.drain(2..4));
    prepare(&mut graph.layers, &graph.samplers);
    program
}
fn rejected(program: &EffectProgram) -> SpecializationRejection {
    AgePlan::analyze(&program.inner.layers[0], &program.inner.samplers).unwrap_err()
}
fn packet_rejected(program: &EffectProgram) -> SpecializationRejection {
    let layer = &program.inner.layers[0];
    PureEvolution::analyze(
        layer,
        layer.evolve.as_ref().unwrap(),
        &program.inner.samplers,
    )
    .unwrap_err()
}
fn prefix(program: &mut EffectProgram, instruction: Instruction) {
    Arc::get_mut(&mut program.inner).unwrap().layers[0]
        .evolve
        .as_mut()
        .unwrap()
        .program
        .operations
        .insert(
            0,
            Operation {
                offset: 987,
                instruction,
            },
        );
}
fn scalar() -> Source {
    Source {
        address: Address::Constant([1f32.to_bits(); 4]),
        kind: ValueType::Float,
    }
}

#[test]
fn absence_layer_role_and_source_barriers_have_distinct_provenance() {
    let mut program = closed_fixture();
    let layer = &mut Arc::get_mut(&mut program.inner).unwrap().layers[0];
    layer.evolve = None;
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    prepare(&mut graph.layers, &graph.samplers);
    let program = program.with_execution(options(NumericPolicy::Graphics));
    let report = program.specialization_report(EffectLayerId(0)).unwrap();
    assert_eq!(report.pure_packet, SpecializationStatus::NotApplicable);
    assert_eq!(
        report.immutable_expressions,
        SpecializationStatus::NotApplicable
    );

    let mut program = closed_fixture();
    Arc::get_mut(&mut program.inner).unwrap().layers[0].spawner = true;
    assert_eq!(rejected(&program).reason, SpecializationReason::Spawner);
    assert_eq!(
        packet_rejected(&program).reason,
        SpecializationReason::Spawner
    );

    let mut program = closed_fixture();
    let field = program.inner.layers[0].life;
    let layer = &mut Arc::get_mut(&mut program.inner).unwrap().layers[0];
    layer.evolve.as_mut().unwrap().externals[0] = External::Field(field);
    prefix(
        &mut program,
        Instruction::Store {
            source: scalar(),
            external: 0,
        },
    );
    let expected = SpecializationRejection::new(SpecializationReason::LifecycleWrite)
        .at(987)
        .field(field);
    assert_eq!(rejected(&program), expected);
    assert_eq!(packet_rejected(&program), expected);

    let mut program = closed_fixture();
    Arc::get_mut(&mut program.inner).unwrap().layers[0]
        .evolve
        .as_mut()
        .unwrap()
        .externals[0] = External::Time;
    assert_eq!(
        rejected(&program).reason,
        SpecializationReason::UncertifiedTime
    );
    assert_eq!(rejected(&program).offset, Some(0));
}

#[test]
fn invalid_bindings_persistent_inputs_layouts_and_operations_are_not_conflated() {
    let mut program = closed_fixture();
    prefix(
        &mut program,
        Instruction::Load {
            destination: Destination {
                scope: 1,
                slot: 0,
                kind: ValueType::Float,
            },
            external: u16::MAX,
        },
    );
    assert_eq!(
        rejected(&program),
        SpecializationRejection::new(SpecializationReason::InvalidBinding).at(987)
    );
    assert_eq!(packet_rejected(&program), rejected(&program));

    let mut program = closed_fixture();
    Arc::get_mut(&mut program.inner).unwrap().layers[0]
        .evolve
        .as_mut()
        .unwrap()
        .program
        .counts[2] = 1;
    prefix(
        &mut program,
        Instruction::Store {
            source: Source {
                address: Address::Slot { scope: 2, slot: 0 },
                kind: ValueType::Float,
            },
            external: 1,
        },
    );
    let expected =
        SpecializationRejection::new(SpecializationReason::PersistentInput { scope: 2, slot: 0 })
            .at(987);
    assert_eq!(rejected(&program), expected);
    assert_eq!(packet_rejected(&program), expected);

    let mut program = closed_fixture();
    prefix(
        &mut program,
        Instruction::Binary {
            destination: Destination {
                scope: 1,
                slot: 0,
                kind: ValueType::Int,
            },
            left: Source {
                kind: ValueType::Int,
                ..scalar()
            },
            right: scalar(),
            operation: BinaryOperation::Add,
        },
    );
    assert_eq!(
        rejected(&program).reason,
        SpecializationReason::Layout(ValueType::Int)
    );
    assert_eq!(packet_rejected(&program), rejected(&program));

    let mut program = closed_fixture();
    prefix(
        &mut program,
        Instruction::Binary {
            destination: Destination {
                scope: 1,
                slot: 0,
                kind: ValueType::Float,
            },
            left: scalar(),
            right: scalar(),
            operation: BinaryOperation::Divide,
        },
    );
    assert_eq!(
        rejected(&program).reason,
        SpecializationReason::Instruction(SpecializationInstruction::Arithmetic)
    );
    assert_eq!(packet_rejected(&program), rejected(&program));
}

#[test]
fn unused_curve_result_still_requires_a_coordinate_certificate() {
    let mut program = closed_fixture();
    let bound = Arc::get_mut(&mut program.inner).unwrap().layers[0]
        .evolve
        .as_mut()
        .unwrap();
    let curve = bound
        .program
        .operations
        .iter()
        .find(|op| matches!(op.instruction, Instruction::Call { .. }))
        .unwrap()
        .clone();
    let mut unused = curve;
    unused.offset = 1234;
    let Instruction::Call { arguments, .. } = &mut unused.instruction else {
        unreachable!()
    };
    arguments[0] = Source {
        address: Address::Constant([f32::NAN.to_bits(); 4]),
        kind: ValueType::Float,
    };
    bound.program.operations.push(unused);
    let error = rejected(&program);
    assert_eq!(
        error.reason,
        SpecializationReason::CurveCoordinate { sampler: 0 }
    );
    assert_eq!(error.offset, Some(1234));
    let layer = &program.inner.layers[0];
    assert!(
        PureEvolution::analyze(
            layer,
            layer.evolve.as_ref().unwrap(),
            &program.inner.samplers
        )
        .is_ok(),
        "packet runtime checks curve coordinates per lane instead"
    );
}

#[test]
fn preparation_budgets_report_resource_and_original_offset() {
    let mut program = closed_fixture();
    let bound = Arc::get_mut(&mut program.inner).unwrap().layers[0]
        .evolve
        .as_mut()
        .unwrap();
    bound.program.operations = (0..MAX_NODES)
        .map(|index| Operation {
            offset: 1000 + index,
            instruction: Instruction::Binary {
                destination: Destination {
                    scope: 1,
                    slot: 0,
                    kind: ValueType::Float,
                },
                left: scalar(),
                right: scalar(),
                operation: BinaryOperation::Add,
            },
        })
        .collect();
    let error = rejected(&program);
    assert_eq!(
        error.reason,
        SpecializationReason::Limit {
            resource: SpecializationResource::Nodes,
            limit: MAX_NODES
        }
    );
    assert!(error.offset.unwrap() >= 1000);

    let mut program = closed_fixture();
    let layer = &mut Arc::get_mut(&mut program.inner).unwrap().layers[0];
    let mut fields = layer.fields.to_vec();
    let scalar_field = fields[3].clone();
    fields.resize(MAX_OUTPUTS + 3, scalar_field.clone());
    for field in &mut fields[2..] {
        *field = scalar_field.clone();
    }
    layer.fields = fields.into_boxed_slice();
    let bound = layer.evolve.as_mut().unwrap();
    bound.externals = (2..MAX_OUTPUTS + 3).map(External::Field).collect();
    bound.program.operations = (0..MAX_OUTPUTS + 1)
        .map(|index| Operation {
            offset: index,
            instruction: Instruction::Store {
                source: scalar(),
                external: index as u16,
            },
        })
        .collect();
    assert_eq!(
        rejected(&program).reason,
        SpecializationReason::Limit {
            resource: SpecializationResource::Outputs,
            limit: MAX_OUTPUTS
        }
    );
}

#[test]
fn written_feedback_keeps_field_identity_and_empty_outputs_are_distinct() {
    let mut program = closed_fixture();
    let layer = &mut Arc::get_mut(&mut program.inner).unwrap().layers[0];
    let bound = layer.evolve.as_mut().unwrap();
    bound.externals[0] = External::Field(3);
    bound.program.operations.insert(
        1,
        Operation {
            offset: 55,
            instruction: Instruction::Binary {
                destination: Destination {
                    scope: 1,
                    slot: 0,
                    kind: ValueType::Float,
                },
                left: Source {
                    address: Address::Slot { scope: 1, slot: 0 },
                    kind: ValueType::Float,
                },
                right: scalar(),
                operation: BinaryOperation::Add,
            },
        },
    );
    let error = rejected(&program);
    assert_eq!(error.reason, SpecializationReason::Feedback);
    assert_eq!(error.field, Some(EffectFieldId(3)));
    assert_eq!(error.offset, Some(0));

    let mut program = closed_fixture();
    Arc::get_mut(&mut program.inner).unwrap().layers[0]
        .evolve
        .as_mut()
        .unwrap()
        .program
        .operations
        .clear();
    assert_eq!(rejected(&program).reason, SpecializationReason::NoOutputs);
}
