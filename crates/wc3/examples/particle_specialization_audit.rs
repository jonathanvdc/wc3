//! Original public corpus coverage observation, not a benchmark or GPU proof.
use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};
use wc3::model::formats::pkb::{Document, EffectCompileOptions, EffectEvolution};
use wc3::model::scene::popcorn::{
    EffectInstance, EffectLimits, EffectProgram, EffectRandom, EffectRuntimeError,
    EventStreamDuration, ExecutionOptions, ExecutionPreference, MissingPayloadPolicy,
    NumericPolicy, SceneInputs, SpecializationStatus,
};
const FILES: usize = 58;
const DT: f64 = 1. / 60.;
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

fn status(value: SpecializationStatus) -> String {
    match value {
        SpecializationStatus::Accepted => "{\"status\":\"Accepted\"}".into(),
        SpecializationStatus::NotExamined => "{\"status\":\"NotExamined\"}".into(),
        SpecializationStatus::NotApplicable => "{\"status\":\"NotApplicable\"}".into(),
        SpecializationStatus::Rejected(why) => format!(
            "{{\"status\":\"Rejected\",\"reason\":{},\"offset\":{},\"field\":{}}}",
            quoted(&format!("{:?}", why.reason)),
            why.offset.map_or("null".into(), |x| x.to_string()),
            why.field.map_or("null".into(), |x| x.0.to_string())
        ),
    }
}
struct Case<'a> {
    key: &'a str,
    program: &'a EffectProgram,
    instance: EffectInstance,
    observations: usize,
    errors: usize,
}
impl Case<'_> {
    fn record(
        &mut self,
        operation: &str,
        frame: usize,
        dt: f64,
        result: Result<(), EffectRuntimeError>,
    ) {
        self.observations += 1;
        let error = match result {
            Ok(()) => "null".into(),
            Err(error) => {
                self.errors += 1;
                quoted(&error.to_string())
            }
        };
        let layers: Vec<_> = self.program.specialization_reports().map(|report| {
            let mut live = 0usize;
            let mut samples = 0usize;
            let mut current = 0usize;
            for particle in self.instance.particles(report.layer) {
                live += 1;
                if let Some(view) = particle.immutable_sample() {
                    samples += 1;
                    current += usize::from(view.sample().effect_time == self.instance.time());
                }
            }
            format!("{{\"layer\":{},\"live\":{live},\"immutable_samples\":{samples},\"current_immutable_samples\":{current}}}", report.layer.0)
        }).collect();
        let work = self.instance.execution_stats();
        let stats = self.instance.stats();
        println!(
            "{{\"kind\":\"observation\",{},\"operation\":{},\"frame\":{frame},\"advancing\":{},\"time_bits\":{},\"active\":{},\"immutable_active\":{},\"layers\":[{}],\"instance_counters\":{{\"scalar_jit_invocations\":{},\"immutable_age_invocations\":{},\"immutable_age_fallbacks\":{},\"rust_packet_committed_lanes\":{},\"simd_attempted_batches\":{},\"simd_committed_lanes\":{}}},\"omissions\":{{\"births\":{},\"executions\":{},\"spatial_writes\":{},\"spatial_queries\":{},\"skipped_seconds_bits\":{}}},\"error\":{error}}}",
            self.key,
            quoted(operation),
            dt > 0.,
            self.instance.time().to_bits(),
            self.instance.active_count(),
            self.instance.immutable_active(),
            layers.join(","),
            work.scalar_jit_invocations,
            work.immutable_age_invocations,
            work.immutable_age_fallbacks,
            work.rust_packet_committed_lanes,
            work.simd_attempted_batches,
            work.simd_committed_lanes,
            stats.omitted_births,
            stats.omitted_executions,
            stats.omitted_spatial_writes,
            stats.omitted_spatial_queries,
            stats.skipped_seconds.to_bits()
        );
    }
    fn start(&mut self, operation: &str) {
        let result = self.instance.start(
            self.program,
            self.program.default_entry(),
            SceneInputs::default(),
        );
        self.record(operation, 0, 0., result);
    }
    fn step(&mut self, operation: &str, frame: usize, dt: f64) {
        let result = self
            .instance
            .advance(self.program, dt, SceneInputs::default());
        self.record(operation, frame, dt, result);
    }
    fn run(mut self) {
        self.start("start");
        self.step("zero_before", 0, 0.);
        for frame in 1..=120 {
            self.step("advance", frame, DT);
        }
        self.step("zero_after", 120, 0.);
        self.instance.stop();
        self.record("stop", 120, 0., Ok(()));
        self.step("stopped_zero_before", 0, 0.);
        self.step("stopped_advance", 1, DT);
        self.step("stopped_zero_after", 1, 0.);
        self.start("restart");
        self.step("restart_zero_before", 0, 0.);
        for frame in 1..=120 {
            self.step("restart_advance", frame, DT);
        }
        self.step("restart_zero_after", 120, 0.);
        println!(
            "{{\"kind\":\"case_end\",{},\"observations\":{},\"runtime_errors\":{}}}",
            self.key, self.observations, self.errors
        );
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() > 1 {
        return Err("usage: eligibility-coverage [REPOSITORY_ROOT]".into());
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
    println!(
        "{{\"kind\":\"config\",\"version\":1,\"files\":58,\"capacity\":128,\"seeds\":[0,42],\"frames_per_epoch\":120,\"dt_bits\":{},\"execution\":\"ImmutableAge\",\"numeric_policy\":\"Graphics\",\"missing_payload\":\"Zero\"}}",
        DT.to_bits()
    );
    let mut compiled = 0;
    let mut errors = 0;
    let mut cases = 0;
    for path in paths {
        let name = path
            .strip_prefix(&repo)?
            .to_str()
            .ok_or("non-UTF8 corpus path")?;
        let file_key = format!("\"path\":{}", quoted(name));
        println!("{{\"kind\":\"file\",{file_key}}}");
        let data = match fs::read(&path) {
            Ok(data) => data,
            Err(error) => {
                println!(
                    "{{\"kind\":\"read_error\",{file_key},\"error\":{}}}",
                    quoted(&error.to_string())
                );
                errors += 1;
                continue;
            }
        };
        let document = match Document::read(&data) {
            Ok(doc) => doc,
            Err(error) => {
                println!(
                    "{{\"kind\":\"document_error\",{file_key},\"error\":{}}}",
                    quoted(&error.to_string())
                );
                errors += 1;
                continue;
            }
        };
        let mut roots = Vec::new();
        for (ordinal, root) in document.effect_roots().enumerate() {
            match root {
                Ok((link, _)) => roots.push(link),
                Err(error) => {
                    errors += 1;
                    println!(
                        "{{\"kind\":\"root_error\",{file_key},\"ordinal\":{ordinal},\"error\":{}}}",
                        quoted(&error.to_string())
                    );
                }
            }
        }
        roots.sort_unstable();
        if roots.is_empty() {
            errors += 1;
            println!("{{\"kind\":\"no_effect_roots\",{file_key}}}");
        }
        for root in roots {
            for (evolution, label) in [
                (EffectEvolution::Physics, "Physics"),
                (EffectEvolution::Fixed, "Fixed"),
                (EffectEvolution::Varying, "Varying"),
            ] {
                let key = format!("{file_key},\"root\":{root},\"evolution\":{}", quoted(label));
                let options = EffectCompileOptions {
                    evolution,
                    missing_payload: MissingPayloadPolicy::Zero,
                    random: EffectRandom::Counter,
                    stream_duration: EventStreamDuration::LastOccurrence,
                    ..Default::default()
                };
                let program = match document.compile_effect(root, &options) {
                    Ok(program) => program.with_execution(ExecutionOptions {
                        preference: ExecutionPreference::ImmutableAge,
                        numeric_policy: NumericPolicy::Graphics,
                    }),
                    Err(error) => {
                        errors += 1;
                        println!(
                            "{{\"kind\":\"compile_error\",{key},\"error\":{}}}",
                            quoted(&error.to_string())
                        );
                        continue;
                    }
                };
                compiled += 1;
                println!(
                    "{{\"kind\":\"compiled\",{key},\"layers\":{},\"renderers\":{}}}",
                    program.layers().len(),
                    program.renderers().len()
                );
                for report in program.specialization_reports() {
                    let renderers = program
                        .renderers()
                        .iter()
                        .filter(|r| r.layer == report.layer)
                        .count();
                    println!(
                        "{{\"kind\":\"layer\",{key},\"layer\":{},\"renderers\":{renderers},\"pure_packet\":{},\"immutable_expressions\":{}}}",
                        report.layer.0,
                        status(report.pure_packet),
                        status(report.immutable_expressions)
                    );
                }
                for report in program.execution_reports() {
                    println!(
                        "{{\"kind\":\"stage\",{key},\"layer\":{},\"stage\":{},\"prepared_route\":{},\"fallback\":{}}}",
                        report.layer.0,
                        quoted(&format!("{:?}", report.stage)),
                        quoted(&format!("{:?}", report.backend)),
                        report
                            .fallback
                            .as_ref()
                            .map_or("null".into(), |x| quoted(&format!("{x:?}")))
                    );
                }
                for seed in [0, 42] {
                    let key = format!("{key},\"seed\":{seed}");
                    match program.instantiate(
                        EffectLimits {
                            particles: 128,
                            ..Default::default()
                        },
                        seed,
                    ) {
                        Ok(instance) => {
                            cases += 1;
                            Case {
                                key: &key,
                                program: &program,
                                instance,
                                observations: 0,
                                errors: 0,
                            }
                            .run();
                        }
                        Err(error) => {
                            errors += 1;
                            println!(
                                "{{\"kind\":\"instantiate_error\",{key},\"error\":{}}}",
                                quoted(&error.to_string())
                            );
                        }
                    }
                }
            }
        }
    }
    println!(
        "{{\"kind\":\"end\",\"files\":58,\"compiled\":{compiled},\"cases\":{cases},\"preparation_errors\":{errors}}}"
    );
    Ok(())
}
