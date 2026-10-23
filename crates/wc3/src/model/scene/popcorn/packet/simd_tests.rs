use super::*;
use crate::model::scene::popcorn::graph::EffectProgram;
use crate::model::scene::popcorn::packet::tests::{fixture, persistent_fixture};
use crate::model::scene::popcorn::packet::{Lane, Scratch};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage};
use std::mem::transmute;
use std::sync::Arc;

type Entry<const BATCH: usize = PARTICLE_BATCH_SIZE> =
    unsafe extern "C" fn(*mut BatchFrame<BATCH>) -> u32;
type Code = BatchCode<PARTICLE_BATCH_SIZE>;
struct BatchCode<const BATCH: usize> {
    module: Option<JITModule>,
    entry: Entry<BATCH>,
}
impl<const BATCH: usize> Drop for BatchCode<BATCH> {
    fn drop(&mut self) {
        if let Some(module) = self.module.take() {
            unsafe {
                module.free_memory();
            }
        }
    }
}
impl<const BATCH: usize> BatchCode<BATCH> {
    fn configured<const LANES: usize>(plan: &PureEvolution, samplers: &[EffectSampler]) -> Self {
        let builder =
            JITBuilder::with_flags(&[("opt_level", "speed")], default_libcall_names()).unwrap();
        let mut module = JITModule::new(builder);
        let mut context = module.make_context();
        SimdCompiler::<BATCH, LANES>::lower(&mut module, plan, samplers, &mut context).unwrap();
        let id = module
            .declare_function(
                "original_simd_tile",
                Linkage::Local,
                &context.func.signature,
            )
            .unwrap();
        module.define_function(id, &mut context).unwrap();
        module.finalize_definitions().unwrap();
        let entry =
            unsafe { transmute::<*const u8, Entry<BATCH>>(module.get_finalized_function(id)) };
        Self {
            module: Some(module),
            entry,
        }
    }
    fn run(
        &self,
        scratch: &mut BatchScratch<BATCH>,
        plan: &PureEvolution,
        time: f64,
    ) -> Result<(), ()> {
        BatchFrame::<BATCH>::with(scratch, plan, time, |frame| unsafe { (self.entry)(frame) })
    }
}
impl BatchCode<PARTICLE_BATCH_SIZE> {
    fn new(plan: &PureEvolution, samplers: &[EffectSampler]) -> Self {
        Self::configured::<SIMD_LANES>(plan, samplers)
    }
}
fn staged(program: &EffectProgram) -> Box<Scratch> {
    staged_batch::<PARTICLE_BATCH_SIZE>(program)
}
fn staged_batch<const BATCH: usize>(program: &EffectProgram) -> Box<BatchScratch<BATCH>> {
    let layer = &program.inner.layers[0];
    let plan = layer.pure_evolution.as_ref().unwrap();
    let mut scratch = BatchScratch::<BATCH>::new((plan.registers, plan.fields.len()));
    scratch.poison();
    scratch.count = BATCH;
    scratch.registers[..plan.locals].fill([[0; BATCH]; VALUE_COMPONENTS]);
    for lane in 0..BATCH {
        scratch.lanes[lane] = Lane {
            slot: lane,
            life: 0.25,
            delta: (lane as f64) / 64.,
        };
        for (field, binding) in plan.fields.iter().enumerate() {
            let mut words = layer.fields[binding.canonical].initial;
            if binding.canonical == 2 {
                words[0] =
                    ([-1f32, -0.125, 0., 0.125, 0.25, 0.875, 1., 2.][lane % 8] - 0.125).to_bits();
            }
            for (component, value) in words.into_iter().enumerate() {
                scratch.fields[field][component][lane] = value;
            }
        }
    }
    scratch
}
#[test]
fn four_vector_groups_match_rust_packet_on_curve_keys_clamps_and_both_persistent_banks() {
    for program in [
        fixture(PARTICLE_BATCH_SIZE),
        persistent_fixture(PARTICLE_BATCH_SIZE),
    ] {
        let plan = program.inner.layers[0].pure_evolution.as_ref().unwrap();
        let code = Code::new(plan, &program.inner.samplers);
        for time in [0., 1.25, 7.5] {
            let mut scalar = staged(&program);
            let mut vector = staged(&program);
            scalar.execute(plan, &program.inner.samplers, time).unwrap();
            code.run(&mut vector, plan, time).unwrap();
            assert_eq!(
                scalar.fields, vector.fields,
                "all staged field words, including inactive normalized components"
            );
        }
    }
}
#[test]
fn linear_curve_and_signed_zero_key_values_match_shared_scalar_sampling() {
    let mut program = fixture(PARTICLE_BATCH_SIZE);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let EffectSampler::Curve {
        values, tangents, ..
    } = &mut graph.samplers[0]
    else {
        unreachable!()
    };
    *tangents = Box::new([]);
    values[0][0] = -0.;
    values[2][1] = -0.;
    let graph = &program.inner;
    let plan = graph.layers[0].pure_evolution.as_ref().unwrap();
    let code = Code::new(plan, &graph.samplers);
    let mut scalar = staged(&program);
    let mut vector = staged(&program);
    scalar.execute(plan, &graph.samplers, 1.25).unwrap();
    code.run(&mut vector, plan, 1.25).unwrap();
    assert_eq!(scalar.fields, vector.fields);
}
#[test]
fn later_group_failure_discards_speculation_and_fresh_staging_recovers() {
    let program = fixture(PARTICLE_BATCH_SIZE);
    let plan = program.inner.layers[0].pure_evolution.as_ref().unwrap();
    let code = Code::new(plan, &program.inner.samplers);
    for bad_lane in [
        0,
        SIMD_LANES - 1,
        SIMD_LANES,
        PARTICLE_BATCH_SIZE / 2 + 1,
        PARTICLE_BATCH_SIZE - 1,
    ] {
        let mut scalar = staged(&program);
        let mut vector = staged(&program);
        let field = plan
            .fields
            .iter()
            .position(|field| field.canonical == 2)
            .unwrap();
        scalar.fields[field][0][bad_lane] = f32::NAN.to_bits();
        vector.fields[field][0][bad_lane] = f32::NAN.to_bits();
        assert!(scalar.execute(plan, &program.inner.samplers, 1.25).is_err());
        assert!(code.run(&mut vector, plan, 1.25).is_err());
        // No call to Scratch::commit occurs on failure. Dirty speculative
        // overlay contents need not agree; the runtime's existing scalar replay
        // remains responsible for exact canonical failure prefixes.
        scalar = staged(&program);
        vector = staged(&program);
        scalar.execute(plan, &program.inner.samplers, 1.25).unwrap();
        code.run(&mut vector, plan, 1.25).unwrap();
        assert_eq!(scalar.fields, vector.fields);
    }
}
#[test]
fn nonfinite_time_delta_and_short_tiles_fail_before_any_canonical_commit() {
    let program = fixture(PARTICLE_BATCH_SIZE);
    let plan = program.inner.layers[0].pure_evolution.as_ref().unwrap();
    let code = Code::new(plan, &program.inner.samplers);
    for time in [f64::NAN, f64::INFINITY, f64::MAX] {
        let mut tile = staged(&program);
        assert!(code.run(&mut tile, plan, time).is_err());
    }
    let mut tile = staged(&program);
    tile.lanes[7].delta = f64::MAX;
    assert!(code.run(&mut tile, plan, 1.25).is_err());
    let mut tile = staged(&program);
    tile.count = PARTICLE_BATCH_SIZE - 1;
    assert!(code.run(&mut tile, plan, 1.25).is_err());
    assert_eq!(tile.attempts, 0);
}
#[test]
fn large_valid_curve_keeps_rust_lane_sampling_without_a_simd_entry() {
    let mut program = fixture(PARTICLE_BATCH_SIZE);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    graph.samplers[0] = EffectSampler::Curve {
        kind: ValueType::Float4,
        times: (0..=MAX_CURVE_KEYS).map(|i| i as f32 / 16.).collect(),
        values: (0..=MAX_CURVE_KEYS).map(|i| [i as f32 / 16.; 4]).collect(),
        tangents: Box::new([]),
    };
    let graph = &program.inner;
    let plan = graph.layers[0].pure_evolution.as_ref().unwrap();
    assert!(!eligible(plan, &graph.samplers));
    let mut scalar = staged(&program);
    scalar.execute(plan, &graph.samplers, 1.25).unwrap();
}

// Append only to the private packet/simd_tests.rs test module.
#[test]
#[ignore = "cold diagnostic only: set WC3_SIMD_CODEGEN_OUT to a fresh output directory"]
fn dump_actual_simd_native_code_and_execute_the_same_entry() {
    use std::env::var_os;
    use std::fs::{create_dir, write};
    use std::path::PathBuf;
    use std::slice::from_raw_parts;

    let output = PathBuf::from(
        var_os("WC3_SIMD_CODEGEN_OUT").expect("fresh diagnostic output path is required"),
    );
    create_dir(&output).expect("diagnostic directory must not already exist");
    let program = fixture(PARTICLE_BATCH_SIZE);
    let plan = program.inner.layers[0].pure_evolution.as_ref().unwrap();
    let builder =
        JITBuilder::with_flags(&[("opt_level", "speed")], default_libcall_names()).unwrap();
    let mut module = JITModule::new(builder);
    let mut context = module.make_context();
    context.set_disasm(true);
    lower(&mut module, plan, &program.inner.samplers, &mut context).unwrap();
    let clif = context.func.display().to_string();
    assert!(
        clif.contains("f32x4"),
        "the diagnostic must lower actual vectors"
    );
    let id = module
        .declare_function(
            "diagnostic_simd_tile",
            Linkage::Local,
            &context.func.signature,
        )
        .unwrap();
    module.define_function(id, &mut context).unwrap();
    let compiled = context
        .compiled_code()
        .expect("definition must have machine code");
    let vcode = compiled
        .vcode
        .as_ref()
        .expect("set_disasm must retain final native vcode")
        .clone();
    let code_length = compiled.buffer.data().len();
    let isa = module.isa().name().to_owned();
    let triple = module.isa().triple().to_string();
    module.finalize_definitions().unwrap();
    let address = module.get_finalized_function(id);
    // The length belongs to this exact compiled definition, and its same module
    // owner retains the finalized executable allocation until copying/execution
    // below completes. Copy installed bytes rather than pre-relocation buffer.
    let installed = unsafe { from_raw_parts(address, code_length) }.to_vec();
    write(output.join("lowered.clif"), &clif).unwrap();
    write(output.join("vcode.txt"), &vcode).unwrap();
    write(output.join("finalized-code.bin"), &installed).unwrap();
    let packed = |operation: &str| {
        vcode
            .lines()
            .filter(|line| {
                if cfg!(target_arch = "aarch64") {
                    line.contains(operation) && line.contains(".4s")
                } else {
                    let mnemonic = match operation {
                        "fadd" => "addps",
                        "fsub" => "subps",
                        "fmul" => "mulps",
                        _ => unreachable!(),
                    };
                    line.contains(mnemonic)
                }
            })
            .count()
    };
    let counts = [packed("fadd"), packed("fsub"), packed("fmul")];
    assert!(
        counts.iter().all(|count| *count > 0),
        "native code must contain packed add/sub/mul: {counts:?}"
    );
    let entry = unsafe { transmute::<*const u8, Entry>(address) };
    let code = Code {
        module: Some(module),
        entry,
    };
    let mut scalar = staged(&program);
    let mut vector = staged(&program);
    scalar.execute(plan, &program.inner.samplers, 1.25).unwrap();
    code.run(&mut vector, plan, 1.25).unwrap();
    assert_eq!(
        scalar.fields, vector.fields,
        "dumped finalized entry must execute and match its Rust tile"
    );
    assert_eq!(vector.attempts, 1);
    write(output.join("metadata.json"), format!(
        "{{\"status\":\"executed_field_parity_pass\",\"measurement\":false,\"isa\":\"{isa}\",\"triple\":\"{triple}\",\"code_bytes\":{code_length},\"lanes\":{PARTICLE_BATCH_SIZE},\"actual_entry_calls\":1,\"packed_add_sub_mul\":{counts:?}}}\n"
    )).unwrap();
}

#[test]
fn boolean_raw_zero_and_all_ones_copy_tiles_match_rust_without_float_canonicalization() {
    use crate::model::scene::popcorn::packet::tests::boolean_copy_fixture;
    for word in [0, u32::MAX, 1, 0x7f80_0001, 0x7fc1_2345] {
        let program = boolean_copy_fixture(word);
        let plan = program.inner.layers[0].pure_evolution.as_ref().unwrap();
        let code = Code::new(plan, &program.inner.samplers);
        let mut scalar = staged(&program);
        let mut vector = staged(&program);
        scalar.execute(plan, &program.inner.samplers, 0.).unwrap();
        code.run(&mut vector, plan, 0.).unwrap();
        assert_eq!(scalar.fields, vector.fields);
    }
}

#[test]
fn actual_native_benchmark_child_lowers_and_defines_a_simd_entry() {
    use crate::model::formats::pkb::{Document, EffectCompileOptions};
    let document = Document::read(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/benches/fixtures/particle_runtime/burst-1.pkb"
    )))
    .unwrap();
    let (root, _) = document.effect_roots().next().unwrap().unwrap();
    let program = document
        .compile_effect(root, &EffectCompileOptions::default())
        .unwrap();
    let plan = program.inner.layers[1]
        .pure_evolution
        .as_ref()
        .expect("native benchmark pure plan");
    assert!(eligible(plan, &program.inner.samplers));
    assert!(cost(plan, &program.inner.samplers).is_some());
    let _code = Code::new(plan, &program.inner.samplers);
}

#[test]
fn repeated_time_loads_share_tile_input_and_unused_nonfinite_time_is_not_checked() {
    use crate::model::scene::popcorn::packet::tests::boolean_copy_fixture;
    // The original checked fixture loads Time; duplicate that operation without
    // changing the source result and compare every staged field word.
    let program = fixture(PARTICLE_BATCH_SIZE);
    let graph = &program.inner;
    let mut plan = graph.layers[0].pure_evolution.as_deref().unwrap().clone();
    let repeated = plan
        .operations
        .iter()
        .find(|operation| {
            matches!(
                operation.instruction,
                InstructionPlan::Load {
                    input: Input::Time,
                    ..
                }
            )
        })
        .unwrap()
        .clone();
    let mut operations = plan.operations.to_vec();
    operations.push(repeated);
    plan.operations = operations.into_boxed_slice();
    let code = Code::new(&plan, &graph.samplers);
    for time in [-0., 0., 1.25, 7.5] {
        let mut scalar = staged(&program);
        let mut vector = staged(&program);
        scalar
            .execute(&plan, &program.inner.samplers, time)
            .unwrap();
        code.run(&mut vector, &plan, time).unwrap();
        assert_eq!(scalar.fields, vector.fields);
    }
    for time in [f64::NAN, f64::INFINITY, f64::MAX] {
        let mut vector = staged(&program);
        assert!(code.run(&mut vector, &plan, time).is_err());
    }
    let program = boolean_copy_fixture(u32::MAX);
    let plan = program.inner.layers[0].pure_evolution.as_ref().unwrap();
    assert!(!plan.operations.iter().any(|operation| matches!(
        operation.instruction,
        InstructionPlan::Load {
            input: Input::Time,
            ..
        }
    )));
    let code = Code::new(plan, &program.inner.samplers);
    for time in [f64::NAN, f64::INFINITY, f64::MAX] {
        let mut scalar = staged(&program);
        let mut vector = staged(&program);
        scalar.execute(plan, &program.inner.samplers, time).unwrap();
        code.run(&mut vector, plan, time).unwrap();
        assert_eq!(scalar.fields, vector.fields);
    }
}

fn configured_execution_matches_rust<const BATCH: usize>() {
    for program in [fixture(BATCH), persistent_fixture(BATCH)] {
        let plan = program.inner.layers[0].pure_evolution.as_ref().unwrap();
        let code = BatchCode::<BATCH>::configured::<4>(plan, &program.inner.samplers);
        for time in [-0., 1.25, 7.5] {
            let mut scalar = staged_batch::<BATCH>(&program);
            let mut vector = staged_batch::<BATCH>(&program);
            scalar.execute(plan, &program.inner.samplers, time).unwrap();
            code.run(&mut vector, plan, time).unwrap();
            assert_eq!(scalar.fields, vector.fields);
            assert_eq!(vector.attempts, 1);
        }
        let field = plan
            .fields
            .iter()
            .position(|field| field.canonical == 2)
            .unwrap();
        let mut vector = staged_batch::<BATCH>(&program);
        vector.fields[field][0][BATCH - 1] = f32::NAN.to_bits();
        assert!(code.run(&mut vector, plan, 1.25).is_err());
        let mut short = staged_batch::<BATCH>(&program);
        short.count -= 1;
        assert!(code.run(&mut short, plan, 1.25).is_err());
        assert_eq!(short.attempts, 0);
    }
}
#[test]
fn alternative_batch_layouts_execute_complete_groups_and_keep_late_failures_transactional() {
    // Only batch size varies for execution: four-lane vectors are supported by
    // both production target ISAs. No eight-lane hardware assumption is made.
    configured_execution_matches_rust::<8>();
    configured_execution_matches_rust::<32>();
}
#[test]
fn compiler_lane_configuration_drives_ir_and_rejects_incomplete_groups() {
    let program = fixture(PARTICLE_BATCH_SIZE);
    let plan = program.inner.layers[0].pure_evolution.as_ref().unwrap();
    assert!(!SimdCompiler::<8, 0>::eligible(
        plan,
        &program.inner.samplers
    ));
    assert!(!SimdCompiler::<8, 1>::eligible(
        plan,
        &program.inner.samplers
    ));
    assert!(!SimdCompiler::<8, 3>::eligible(
        plan,
        &program.inner.samplers
    ));
    assert!(!SimdCompiler::<0, 4>::eligible(
        plan,
        &program.inner.samplers
    ));
    assert!(SimdCompiler::<8, 3>::cost(plan, &program.inner.samplers).is_none());
    let builder =
        JITBuilder::with_flags(&[("opt_level", "speed")], default_libcall_names()).unwrap();
    let mut module = JITModule::new(builder);
    let mut context = module.make_context();
    assert!(
        SimdCompiler::<8, 3>::lower(&mut module, plan, &program.inner.samplers, &mut context)
            .is_err()
    );
    assert!(context.func.signature.params.is_empty());
    // IR-only type selection is independent of target legalization/ISA support.
    // The production owner retains its normal per-kernel codegen fallback.
    SimdCompiler::<16, 8>::lower(&mut module, plan, &program.inner.samplers, &mut context).unwrap();
    assert!(context.func.display().to_string().contains("f32x8"));
}

#[test]
fn ordinary_float_widths_broadcast_in_both_directions_and_zero_inactive_words() {
    use crate::model::scene::popcorn::packet::tests::floating_fixture;
    for kind in [
        ValueType::Float,
        ValueType::Float2,
        ValueType::Float3,
        ValueType::Float4,
    ] {
        let program = floating_fixture(PARTICLE_BATCH_SIZE, kind);
        let plan = program.inner.layers[0].pure_evolution.as_ref().unwrap();
        let code = Code::new(plan, &program.inner.samplers);
        let field = plan
            .fields
            .iter()
            .position(|field| field.canonical == 4)
            .unwrap();
        let mut rust = staged(&program);
        let mut simd = staged(&program);
        rust.execute(plan, &program.inner.samplers, 1.25).unwrap();
        code.run(&mut simd, plan, 1.25).unwrap();
        assert_eq!(rust.fields, simd.fields);
        for component in 0..VALUE_COMPONENTS {
            let expected = if component < kind.components() {
                ((f32::from_bits(program.inner.layers[0].fields[4].initial[component]) + 0.125)
                    * 0.125
                    - [0.5, -0.25, 0.75, -1.][component])
                    .to_bits()
            } else {
                0
            };
            assert!(simd.fields[field][component]
                .iter()
                .all(|&word| word == expected));
        }
    }
}
