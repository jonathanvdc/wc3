//! Original public-API corpus differential oracle. No timings, private VM access,
//! forced optimisation mode, pointers, or reference implementation code.
//! Run from the repository root, or pass its absolute path as the sole argument.
//! Float tolerance is fixed at 512*f32::EPSILON*max(1,abs(reference)), only
//! for actual per-particle immutable Float output components. Everything else
//! is compared bitwise, including nonfinite values, canonical life and errors.
//! Public Value normalizes inactive words; opaque metadata has no public value.
//! Each accepted programme is run at seeds 0 and 42, capacity 128, through:
//! start; zero; 120 advancing frames; zero; stop; zero; one advancing stopped
//! frame; zero; restart; zero; 120 advancing frames; zero. Failed calls retain
//! their exact error and canonical partial state, and do not suppress later calls.
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use wc3::model::formats::pkb::{Document, EffectCompileOptions, EffectEvolution};
use wc3::model::scene::popcorn::{
    EffectFieldId, EffectInstance, EffectLayerId, EffectLimits, EffectProgram, EffectRandom,
    EffectRuntimeError, EffectStats, EventStreamDuration, ExecutionOptions, ExecutionPreference,
    MissingPayloadPolicy, NumericPolicy, SceneInputs, ValueType,
};

const BASIS: u64 = 0xcbf29ce484222325;
const DT: f64 = 1.0 / 60.0;
const FILES: usize = 58;

fn quoted(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c <= '\u{1f}' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
fn mix(hash: &mut u64, word: u64) {
    for byte in word.to_le_bytes() {
        *hash = (*hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
    }
}
fn bytes(hash: &mut u64, data: &[u8]) {
    mix(hash, data.len() as u64);
    for byte in data {
        mix(hash, u64::from(*byte));
    }
}
fn kind_tag(kind: ValueType) -> u64 {
    // Distinguish equal-width layouts (Float4/Quaternion, Bool/Int/Float/Context).
    match kind {
        ValueType::Bool => 1,
        ValueType::Bool3 => 2,
        ValueType::Int => 3,
        ValueType::Int2 => 4,
        ValueType::Int3 => 5,
        ValueType::Float => 6,
        ValueType::Float2 => 7,
        ValueType::Float3 => 8,
        ValueType::Float4 => 9,
        ValueType::Quaternion => 10,
        ValueType::Context => 11,
    }
}
fn stats_words(s: &EffectStats) -> [u64; 8] {
    [
        s.spawned,
        s.retired,
        s.omitted_births,
        s.omitted_executions,
        s.suppressed_births,
        s.omitted_spatial_writes,
        s.omitted_spatial_queries,
        s.skipped_seconds.to_bits(),
    ]
}
fn state_hash(program: &EffectProgram, instance: &EffectInstance) -> u64 {
    let mut hash = BASIS;
    mix(&mut hash, instance.time().to_bits());
    mix(&mut hash, instance.is_running() as u64);
    mix(&mut hash, instance.is_emitting() as u64);
    mix(&mut hash, instance.is_finished() as u64);
    mix(&mut hash, instance.active_count() as u64);
    for word in stats_words(instance.stats()) {
        mix(&mut hash, word);
    }
    mix(&mut hash, program.layers().len() as u64);
    for (index, layer) in program.layers().iter().enumerate() {
        mix(&mut hash, index as u64);
        mix(&mut hash, layer.fields.len() as u64);
        // Public iterator order is part of the oracle; do not sort away changes
        // to actual execution/publication order or recycled-slot identity.
        let mut count = 0;
        for particle in instance.particles(EffectLayerId(index as u32)) {
            count += 1;
            mix(&mut hash, particle.layer.0 as u64);
            mix(&mut hash, particle.id.slot as u64);
            mix(&mut hash, particle.id.generation as u64);
            mix(&mut hash, particle.birth_time.to_bits());
            mix(&mut hash, particle.birth_serial);
            mix(&mut hash, particle.parent.is_some() as u64);
            if let Some(parent) = particle.parent {
                mix(&mut hash, parent.slot as u64);
                mix(&mut hash, parent.generation as u64);
            }
            for field in 0..layer.fields.len() {
                mix(&mut hash, field as u64);
                match particle.field(EffectFieldId(field as u32)) {
                    Some(value) => {
                        mix(&mut hash, 1);
                        mix(&mut hash, kind_tag(value.kind()));
                        for word in value.words() {
                            mix(&mut hash, word as u64);
                        }
                    }
                    None => mix(&mut hash, 0),
                }
            }
        }
        mix(&mut hash, count);
    }
    hash
}
fn schema_hash(program: &EffectProgram) -> u64 {
    let mut hash = BASIS;
    mix(&mut hash, program.layers().len() as u64);
    mix(&mut hash, program.renderers().len() as u64);
    mix(&mut hash, program.default_entry() as u64);
    for layer in program.layers() {
        mix(&mut hash, layer.fields.len() as u64);
        for field in layer.fields.iter() {
            bytes(&mut hash, &field.name);
            mix(&mut hash, field.native_type as u64);
            mix(&mut hash, field.value_type.map(kind_tag).unwrap_or(0));
        }
    }
    mix(&mut hash, program.attributes().len() as u64);
    for attr in program.attributes() {
        bytes(&mut hash, &attr.name);
        mix(&mut hash, kind_tag(attr.default.kind()));
        for word in attr.default.words() {
            mix(&mut hash, word as u64);
        }
    }
    hash
}
// Fixed before observing corpus results. Only actual virtual Float outputs qualify.
const FLOAT_TOLERANCE: f64 = 512. * f32::EPSILON as f64;
struct Case<'a> {
    key: &'a str,
    program: &'a EffectProgram,
    candidate_program: &'a EffectProgram,
    instance: EffectInstance,
    candidate: EffectInstance,
    calls: usize,
    errors: usize,
    exact_words: u64,
    approximate_words: u64,
    changed_words: u64,
    worst_scaled_error: f64,
}
impl Case<'_> {
    fn record(
        &mut self,
        operation: &str,
        frame: usize,
        dt: f64,
        result: Result<(), EffectRuntimeError>,
        candidate_result: Result<(), EffectRuntimeError>,
    ) {
        let context = format!("{} operation={operation} frame={frame}", self.key);
        assert_eq!(result, candidate_result, "checked error prefix: {context}");
        assert_eq!(
            self.instance.time().to_bits(),
            self.candidate.time().to_bits(),
            "time: {context}"
        );
        assert_eq!(
            (
                self.instance.is_running(),
                self.instance.is_emitting(),
                self.instance.is_finished(),
                self.instance.active_count()
            ),
            (
                self.candidate.is_running(),
                self.candidate.is_emitting(),
                self.candidate.is_finished(),
                self.candidate.active_count()
            ),
            "lifecycle: {context}"
        );
        assert_eq!(
            stats_words(self.instance.stats()),
            stats_words(self.candidate.stats()),
            "work/budget: {context}"
        );
        for (index, layer) in self.program.layers().iter().enumerate() {
            let layer_id = EffectLayerId(index as u32);
            let mut expected = self.instance.particles(layer_id);
            let mut actual = self.candidate.particles(layer_id);
            loop {
                let (p, q) = match (expected.next(), actual.next()) {
                    (None, None) => break,
                    (Some(p), Some(q)) => (p, q),
                    _ => panic!("particle count/order: {context} layer={index}"),
                };
                assert_eq!(
                    (
                        p.id,
                        p.layer,
                        p.birth_time.to_bits(),
                        p.birth_serial,
                        p.parent
                    ),
                    (
                        q.id,
                        q.layer,
                        q.birth_time.to_bits(),
                        q.birth_serial,
                        q.parent
                    ),
                    "particle identity: {context}"
                );
                let recipe = q.immutable_sample().map(|v| v.recipe());
                for (field, declaration) in layer.fields.iter().enumerate() {
                    let id = EffectFieldId(field as u32);
                    let (p, q) = (p.field(id), q.field(id));
                    assert_eq!(
                        p.map(|v| v.kind()),
                        declaration.value_type,
                        "reference field layout"
                    );
                    assert_eq!(
                        q.map(|v| v.kind()),
                        declaration.value_type,
                        "candidate field layout"
                    );
                    let (Some(p), Some(q)) = (p, q) else {
                        assert_eq!(p, q);
                        continue;
                    };
                    let virtual_output = recipe.is_some_and(|r| {
                        (0..r.output_count()).any(|i| r.output(i).unwrap().field == id)
                    });
                    let float = matches!(
                        p.kind(),
                        ValueType::Float
                            | ValueType::Float2
                            | ValueType::Float3
                            | ValueType::Float4
                    );
                    // Life/inverseLife are not recipe outputs; their canonical words stay exact.
                    for (component, (a, b)) in p.words().into_iter().zip(q.words()).enumerate() {
                        let approximate =
                            virtual_output && float && component < p.kind().components();
                        if approximate {
                            let (a_float, b_float) =
                                (f64::from(f32::from_bits(a)), f64::from(f32::from_bits(b)));
                            if a_float.is_finite() && b_float.is_finite() {
                                self.approximate_words += 1;
                                let scaled = (a_float - b_float).abs() / a_float.abs().max(1.);
                                self.worst_scaled_error = self.worst_scaled_error.max(scaled);
                                assert!(scaled <= FLOAT_TOLERANCE, "virtual float mismatch: {context} layer={index} field={field} name={:?} component={component} expected={a_float} actual={b_float} scaled={scaled}", declaration.name);
                                self.changed_words += u64::from(a != b);
                                continue;
                            }
                        }
                        self.exact_words += 1;
                        assert_eq!(a, b, "exact/nonfinite/inactive word: {context} layer={index} field={field} component={component}");
                    }
                }
            }
        }
        self.calls += 1;
        let error = match result {
            Ok(()) => "null".into(),
            Err(e) => {
                self.errors += 1;
                quoted(&e.to_string())
            }
        };
        let a = self.instance.execution_stats();
        let b = self.candidate.execution_stats();
        println!("{{\"kind\":\"observation\",{},\"operation\":{},\"frame\":{frame},\"dt_bits\":{},\"reference_state\":\"{:016x}\",\"candidate_state\":\"{:016x}\",\"exact_words\":{},\"approximate_words\":{},\"changed_words\":{},\"worst_scaled_error\":{},\"reference_counters\":{:?},\"candidate_counters\":{:?},\"error\":{error}}}", self.key, quoted(operation), dt.to_bits(), state_hash(self.program, &self.instance), state_hash(self.candidate_program, &self.candidate), self.exact_words, self.approximate_words, self.changed_words, self.worst_scaled_error,
            [a.scalar_jit_invocations,a.immutable_age_invocations,a.immutable_age_fallbacks,a.rust_packet_committed_lanes,a.simd_attempted_batches,a.simd_committed_lanes],
            [b.scalar_jit_invocations,b.immutable_age_invocations,b.immutable_age_fallbacks,b.rust_packet_committed_lanes,b.simd_attempted_batches,b.simd_committed_lanes]);
    }
    fn start(&mut self, operation: &str) {
        let a = self.instance.start(
            self.program,
            self.program.default_entry(),
            SceneInputs::default(),
        );
        let b = self.candidate.start(
            self.candidate_program,
            self.candidate_program.default_entry(),
            SceneInputs::default(),
        );
        self.record(operation, 0, 0., a, b);
    }
    fn step(&mut self, operation: &str, frame: usize, dt: f64) {
        let a = self
            .instance
            .advance(self.program, dt, SceneInputs::default());
        let b = self
            .candidate
            .advance(self.candidate_program, dt, SceneInputs::default());
        self.record(operation, frame, dt, a, b);
    }
    fn run(&mut self) {
        self.start("start");
        self.step("zero_before", 0, 0.);
        for frame in 1..=120 {
            self.step("advance", frame, DT);
        }
        self.step("zero_after", 120, 0.);
        self.instance.stop();
        self.candidate.stop();
        self.record("stop", 120, 0., Ok(()), Ok(()));
        self.step("stopped_zero_before", 0, 0.);
        self.step("stopped_advance", 1, DT);
        self.step("stopped_zero_after", 1, 0.);
        self.start("restart");
        self.step("restart_zero_before", 0, 0.);
        for frame in 1..=120 {
            self.step("restart_advance", frame, DT);
        }
        self.step("restart_zero_after", 120, 0.);
        assert_eq!(self.calls, 250);
        println!("{{\"kind\":\"case_end\",{},\"observations\":{},\"runtime_errors\":{},\"exact_words\":{},\"approximate_words\":{},\"changed_words\":{},\"worst_scaled_error\":{}}}", self.key,self.calls,self.errors,self.exact_words,self.approximate_words,self.changed_words,self.worst_scaled_error);
    }
}
fn walk(path: &Path, output: &mut Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            return Err(format!("unexpected symlink: {}", entry.path().display()).into());
        }
        if kind.is_dir() {
            walk(&entry.path(), output)?;
        } else if entry.path().extension().is_some_and(|s| s == "pkb") {
            output.push(entry.path());
        }
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() > 1 {
        return Err("usage: particle_differential_audit [REPOSITORY_ROOT]".into());
    }
    let repo = args
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut paths = Vec::new();
    walk(&repo.join("data/dependencies/files"), &mut paths)?;
    paths.sort();
    if paths.len() != FILES {
        return Err(format!("expected {FILES} PKBs; found {}", paths.len()).into());
    }
    let limits = EffectLimits {
        particles: 128,
        ..Default::default()
    };
    println!("{{\"kind\":\"config\",\"version\":1,\"reference\":\"Interpreter\",\"candidate\":\"ImmutableAge\",\"numeric_policy\":\"Graphics\",\"float_tolerance_epsilons\":512,\"files\":{FILES},\"capacity\":128,\"seeds\":[0,42],\"evolution\":[\"Physics\",\"Fixed\",\"Varying\"],\"missing_payload\":\"Zero\",\"random\":\"Counter\",\"stream_duration\":\"LastOccurrence\",\"frames\":120,\"dt_bits\":{},\"limits\":{}}}", DT.to_bits(), quoted(&format!("{limits:?}")));
    let mut accepted = 0;
    let mut rejected = 0;
    let mut cases = 0;
    for path in paths {
        let name = path
            .strip_prefix(&repo)?
            .to_str()
            .ok_or("non-UTF8 corpus path")?;
        let file_key = format!("\"path\":{}", quoted(name));
        let data = match fs::read(&path) {
            Ok(data) => data,
            Err(error) => {
                println!(
                    "{{\"kind\":\"read_error\",{file_key},\"error\":{}}}",
                    quoted(&error.to_string())
                );
                rejected += 1;
                continue;
            }
        };
        let document = match Document::read(&data) {
            Ok(document) => document,
            Err(error) => {
                println!(
                    "{{\"kind\":\"document_error\",{file_key},\"error\":{}}}",
                    quoted(&error.to_string())
                );
                rejected += 1;
                continue;
            }
        };
        let mut roots = Vec::new();
        for (ordinal, root) in document.effect_roots().enumerate() {
            match root {
                Ok((link, _)) => roots.push(link),
                Err(error) => {
                    println!(
                        "{{\"kind\":\"root_error\",{file_key},\"ordinal\":{ordinal},\"error\":{}}}",
                        quoted(&error.to_string())
                    );
                    rejected += 1;
                }
            }
        }
        roots.sort_unstable();
        println!(
            "{{\"kind\":\"file\",{file_key},\"bytes\":{},\"roots\":{:?}}}",
            data.len(),
            roots
        );
        if roots.is_empty() {
            println!("{{\"kind\":\"no_effect_roots\",{file_key}}}");
            rejected += 1;
        }
        for root in roots {
            for (evolution, evolution_name) in [
                (EffectEvolution::Physics, "Physics"),
                (EffectEvolution::Fixed, "Fixed"),
                (EffectEvolution::Varying, "Varying"),
            ] {
                let key = format!(
                    "{file_key},\"root\":{root},\"evolution\":{}",
                    quoted(evolution_name)
                );
                let options = EffectCompileOptions {
                    evolution,
                    missing_payload: MissingPayloadPolicy::Zero,
                    random: EffectRandom::Counter,
                    stream_duration: EventStreamDuration::LastOccurrence,
                    ..Default::default()
                };
                let program = match document.compile_effect(root, &options) {
                    Ok(program) => program.with_execution(ExecutionOptions {
                        preference: ExecutionPreference::Interpreter,
                        numeric_policy: NumericPolicy::Strict,
                    }),
                    Err(error) => {
                        println!(
                            "{{\"kind\":\"compile_error\",{key},\"error\":{}}}",
                            quoted(&error.to_string())
                        );
                        rejected += 1;
                        continue;
                    }
                };
                accepted += 1;
                println!("{{\"kind\":\"compiled\",{key},\"schema\":\"{:016x}\",\"layers\":{},\"renderers\":{}}}", schema_hash(&program), program.layers().len(), program.renderers().len());
                let candidate_program = program.clone().with_execution(ExecutionOptions {
                    preference: ExecutionPreference::ImmutableAge,
                    numeric_policy: NumericPolicy::Graphics,
                });
                assert!(program.same_program(&candidate_program));
                for seed in [0, 42] {
                    let case_key = format!("{key},\"seed\":{seed}");
                    match (
                        program.instantiate(limits, seed),
                        candidate_program.instantiate(limits, seed),
                    ) {
                        (Ok(instance), Ok(candidate)) => {
                            cases += 1;
                            Case {
                                key: &case_key,
                                program: &program,
                                candidate_program: &candidate_program,
                                instance,
                                candidate,
                                calls: 0,
                                errors: 0,
                                exact_words: 0,
                                approximate_words: 0,
                                changed_words: 0,
                                worst_scaled_error: 0.,
                            }
                            .run();
                        }
                        (Err(a), Err(b)) => {
                            assert_eq!(a, b, "instantiate failure: {case_key}");
                            println!(
                                "{{\"kind\":\"instantiate_error\",{case_key},\"error\":{}}}",
                                quoted(&a.to_string())
                            );
                            rejected += 1;
                        }
                        _ => panic!("instantiate outcome mismatch: {case_key}"),
                    }
                }
            }
        }
    }
    println!("{{\"kind\":\"end\",\"files\":{FILES},\"compiled\":{accepted},\"rejected\":{rejected},\"cases\":{cases}}}");
    Ok(())
}
