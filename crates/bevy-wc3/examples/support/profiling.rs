//! Asynchronous profiling for the pipelined crowd benchmark.
use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;
use bevy::render::diagnostic::DiagnosticsRecorder;
use bevy::render::pipelined_rendering::RenderAppChannels;
use bevy::render::render_resource::{CachedPipelineState, PipelineCache, PollType, WgpuFeatures};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy::tasks::block_on;
use bevy::transform::TransformSystems;
use bevy_wc3::Wc3Systems;
use serde_json::{json, Value};
use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Default)]
struct Measurements {
    active: bool,
    completed_frames: usize,
    fences: VecDeque<PollType>,
    samples: BTreeMap<String, Vec<f64>>,
    seen: BTreeMap<String, Instant>,
    skipped: BTreeMap<String, usize>,
    starts: BTreeMap<String, Instant>,
    ready: bool,
    gpu_pass_timestamps: bool,
    error: Option<String>,
}
#[derive(Resource, Clone, Default)]
struct Shared(Arc<Mutex<Measurements>>, Arc<Condvar>);
#[derive(Resource, Default)]
struct TimeSource(Option<Instant>);

pub fn install(app: &mut App) {
    let shared = Shared::default();
    let mut extract = app.sub_app_mut(RenderApp).take_extract().unwrap();
    let extraction = shared.clone();
    app.sub_app_mut(RenderApp).set_extract(move |main, render| {
        let clock = TimeSource(Some(Instant::now()));
        extract(main, render);
        finish(&clock, &extraction, "extract_cpu_ms");
    });
    app.insert_resource(shared.clone())
        .init_resource::<TimeSource>()
        .add_systems(First, start)
        .add_systems(Last, (finish_main, collect).chain());
    app.add_systems(
        Update,
        (
            (|shared: Res<Shared>| stage_start(&shared, "animate_instances_cpu_ms"))
                .before(Wc3Systems::AnimateInstances),
            (|shared: Res<Shared>| stage_end(&shared, "animate_instances_cpu_ms"))
                .after(Wc3Systems::AnimateInstances),
        ),
    )
    .add_systems(
        PostUpdate,
        (
            (|shared: Res<Shared>| stage_start(&shared, "evaluate_node_poses_cpu_ms"))
                .before(Wc3Systems::EvaluateNodePoses),
            (|shared: Res<Shared>| stage_end(&shared, "evaluate_node_poses_cpu_ms"))
                .after(Wc3Systems::EvaluateNodePoses),
            (|shared: Res<Shared>| stage_start(&shared, "transform_propagation_cpu_ms"))
                .before(TransformSystems::Propagate),
            (|shared: Res<Shared>| stage_end(&shared, "transform_propagation_cpu_ms"))
                .after(TransformSystems::Propagate),
        ),
    );
    app.sub_app_mut(RenderApp)
        .insert_resource(shared)
        .init_resource::<TimeSource>()
        .add_systems(Render, start.before(RenderSystems::ExtractCommands))
        .add_systems(Render, finish_render.after(RenderSystems::PostCleanup));
}
fn stage_start(shared: &Shared, name: &str) {
    let mut data = shared.0.lock().unwrap();
    if data.active {
        data.starts.insert(name.into(), Instant::now());
    }
}
fn stage_end(shared: &Shared, name: &str) {
    let mut data = shared.0.lock().unwrap();
    if let Some(start) = data.starts.remove(name) {
        if data.active {
            data.samples
                .entry(name.into())
                .or_default()
                .push(start.elapsed().as_secs_f64() * 1000.);
        }
    }
}
fn start(mut clock: ResMut<TimeSource>) {
    clock.0 = Some(Instant::now());
}
fn finish(clock: &TimeSource, shared: &Shared, name: &str) {
    if let Some(start) = clock.0 {
        let mut data = shared.0.lock().unwrap();
        if data.active {
            data.samples
                .entry(name.into())
                .or_default()
                .push(start.elapsed().as_secs_f64() * 1000.);
        }
    }
}
fn finish_main(clock: Res<TimeSource>, shared: Res<Shared>) {
    finish(&clock, &shared, "main_schedule_cpu_ms");
}
fn finish_render(
    clock: Res<TimeSource>,
    shared: Res<Shared>,
    cache: Res<PipelineCache>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    finish(&clock, &shared, "render_schedule_cpu_ms");
    let mut data = shared.0.lock().unwrap();
    data.gpu_pass_timestamps = device.features().contains(
        WgpuFeatures::TIMESTAMP_QUERY
            | WgpuFeatures::TIMESTAMP_QUERY_INSIDE_PASSES
            | WgpuFeatures::TIMESTAMP_QUERY_INSIDE_ENCODERS,
    );
    data.ready = cache.waiting_pipelines().next().is_none();
    data.error = cache
        .pipelines()
        .find_map(|pipeline| match &pipeline.state {
            CachedPipelineState::Err(error) => Some(error.to_string()),
            _ => None,
        });
    // Empty submission provides a fence after this frame's actual command
    // buffers without copying or retaining them.
    data.fences.push_back(PollType::Wait {
        submission_index: Some(queue.submit([])),
        timeout: None,
    });
    data.completed_frames += 1;
    shared.1.notify_all();
}
fn collect(store: Res<DiagnosticsStore>, shared: Res<Shared>) {
    collect_store(&store, &shared);
}
fn collect_store(store: &DiagnosticsStore, shared: &Shared) {
    let mut data = shared.0.lock().unwrap();
    for diagnostic in store.iter() {
        let path = diagnostic.path().as_str();
        if !path.starts_with("render/") {
            continue;
        }
        if path.ends_with("/elapsed_gpu") {
            if !data.gpu_pass_timestamps {
                continue;
            }
            // Bevy suppresses encoder timestamps on macOS. Retain only spans
            // known to write inside a render pass rather than an encoder.
            if cfg!(target_os = "macos")
                && ![
                    "main_opaque_pass_3d",
                    "main_transparent_pass_3d",
                    "early prepass",
                    "late prepass",
                ]
                .iter()
                .any(|name| path.split('/').any(|part| part == *name))
            {
                continue;
            }
        }
        let Some(measurement) = diagnostic.measurement() else {
            continue;
        };
        if data.seen.get(path) == Some(&measurement.time) {
            continue;
        }
        data.seen.insert(path.into(), measurement.time);
        if data.active {
            // Diagnostics arrive asynchronously; exclude delayed warmup reports.
            let skipped = data.skipped.entry(path.into()).or_default();
            if *skipped < 2 {
                *skipped += 1;
                continue;
            }
            data.samples
                .entry(path.into())
                .or_default()
                .push(measurement.value);
        }
    }
}
pub fn ready(app: &App) -> Result<bool> {
    let shared = app.world().resource::<Shared>();
    let data = shared.0.lock().unwrap();
    if let Some(error) = &data.error {
        return Err(error.clone().into());
    }
    Ok(data.ready)
}
/// Bound headless GPU queue depth to three frames, matching an ordinary
/// presentation queue while preserving simulation/render overlap.
pub fn pace(app: &App) -> Result<()> {
    let shared = app.world().resource::<Shared>();
    let fence = {
        let mut data = shared.0.lock().unwrap();
        if data.fences.len() >= 2 {
            data.fences.pop_front()
        } else {
            None
        }
    };
    if let Some(fence) = fence {
        app.world().resource::<RenderDevice>().poll(fence)?;
    }
    Ok(())
}
/// Wait for all submitted warmup frames and their GPU work without advancing
/// either world. `submitted_frames` counts every preceding `app.update()`.
pub fn synchronize(app: &mut App, submitted_frames: usize) -> Result<()> {
    let shared = app.world().resource::<Shared>();
    let deadline = Instant::now() + Duration::from_secs(120);
    let mut data = shared.0.lock().unwrap();
    while data.completed_frames < submitted_frames {
        if let Some(error) = &data.error {
            return Err(error.clone().into());
        }
        let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
            return Err("render warmup synchronization timed out".into());
        };
        let (next, timeout) = shared.1.wait_timeout(data, remaining).unwrap();
        data = next;
        if timeout.timed_out() && data.completed_frames < submitted_frames {
            return Err("render warmup synchronization timed out".into());
        }
    }
    data.fences.clear();
    drop(data);
    app.world()
        .resource::<RenderDevice>()
        .poll(PollType::Wait {
            submission_index: None,
            timeout: None,
        })?;
    Ok(())
}
pub fn reset(app: &mut App) {
    let shared = app.world().resource::<Shared>();
    let mut data = shared.0.lock().unwrap();
    data.samples.clear();
    data.skipped.clear();
    data.starts.clear();
    data.active = true;
}
/// Terminal drain only: recover the render world after its last submission, then
/// wait once for outstanding GPU work. Do not call app.update() after this.
pub fn drain(app: &mut App) -> Result<()> {
    if app.world().contains_resource::<RenderAppChannels>() {
        let render_app = {
            let mut channels = app.world_mut().resource_mut::<RenderAppChannels>();
            block_on(channels.recv()).ok_or("render thread terminated")?
        };
        app.insert_sub_app(RenderApp, render_app);
    }
    app.world()
        .resource::<RenderDevice>()
        .poll(PollType::Wait {
            submission_index: None,
            timeout: None,
        })?;
    app.sub_app_mut(RenderApp)
        .world_mut()
        .resource_mut::<DiagnosticsRecorder>()
        .begin_frame();
    // The recorder's async completion is published through its normal main-world
    // sync system. Running PreUpdate alone avoids another simulation/render frame.
    app.world_mut().run_schedule(PreUpdate);
    let shared = app.world().resource::<Shared>();
    collect_store(app.world().resource::<DiagnosticsStore>(), shared);
    shared.0.lock().unwrap().active = false;
    Ok(())
}
pub fn report(app: &App) -> Value {
    let shared = app.world().resource::<Shared>();
    let data = shared.0.lock().unwrap();
    let mut result = serde_json::Map::new();
    for (name, values) in &data.samples {
        if values.is_empty() {
            continue;
        }
        let mut sorted = values.clone();
        sorted.sort_by(f64::total_cmp);
        result.insert(name.clone(), json!({"samples":sorted.len(), "median": sorted[sorted.len()/2], "p95":sorted[(sorted.len()*95/100).min(sorted.len()-1)]}));
    }
    json!({"maximum_queued_frames":3, "gpu_pass_timestamps_available":data.gpu_pass_timestamps, "spans":result, "note":"Main and render schedules overlap on separate threads. The first two fresh asynchronous reports per diagnostic are discarded after warmup. Set spans are scheduling windows and can include unrelated concurrent systems; they are not exclusive function CPU time. Render-pass CPU spans measure command recording, not GPU completion. GPU spans require supported timestamp features; macOS encoder timestamps are suppressed by Bevy, so only pass spans can be valid GPU timings."})
}
