# Measure renderer performance

Use these examples to compare a fixed workload across renderer revisions or
quality settings. They measure different things: `capture` waits for each completed
frame, while `crowd` retains CPU/render/GPU overlap. Run commands from the workspace
root and use [captures](tools.md) separately to check visual quality.

## Measure completed frames

Pass `--benchmark-frames N` to measure completed frames at each requested capture
pose. The tool warms the pose with eight synchronized frames, then measures `N`
frames while keeping simulation time fixed. It writes a timing CSV beside the
PNG and prints median and p95 wall times. CSV columns separate the complete
`wall_ms` sample, CPU/update/submission `step_ms`, and completion `wait_ms`;
component medians are independent and need not add to the wall median.

Use `--msaa 1`, `2`, `4` or `8` to select the sample count; the default is four.
Keep this count, tone mapping, viewport, camera and scene lighting fixed when
comparing renderers. Match simulation steps as well as the final time: floating
accumulators can produce different populations at an emission boundary when
given different step sequences.

Each sample includes CPU update, render submission, and waiting for the submitted
GPU work to finish. Loading, screenshot readback, and file writes are excluded.
Completion uses nonblocking device polling until the queue is empty, with a
timeout. This busy wait avoids platform blocking-wait timer granularity but
consumes CPU while waiting. The result includes this wait policy and does not
isolate GPU execution. `--single-threaded` runs both example schedules serially
to assess scheduling overhead in small scenes. This changes only the example's
schedule executors; applications using the plugin retain their configured
executors. Record this choice when comparing runs.

Use the [benchmark guide](../../../benchmarks/README.md#host-gpu-measurements)
for synthetic asset export and instructions for comparing renderer revisions.

## Measure effect preparation

Two opt-in CPU microbenchmarks isolate particle preparation and paused view
ordering. Use a release build, including optimized dependencies, and run the
benchmarks without other compilation or rendering work:

```sh
cargo test --release -p bevy-wc3 --lib \
  benchmark_compact_samples_against_cpu_corner_transport -- \
  --ignored --nocapture --test-threads=1
cargo test --release -p bevy-wc3 --lib benchmark_paused_view_order_cache -- \
  --ignored --nocapture --test-threads=1
```

The compact-sample benchmark compares 8,192 head-and-tail particles with the
indexed CPU corner implementation retained as a test oracle. It reports CPU
corner-plus-sort time, compact sampling alone, compact sampling with the actual
view sort, and record payload sizes. GPU submission and upload are excluded. The
view-order benchmark compares repeated ordering with cache hits over 120 paused
frames and two cameras, including the index payload that would require upload.
It excludes natural-order and source-record uploads.

These measurements isolate CPU kernels. They exclude GPU execution, frame
scheduling, asset loading and birth simulation, so use completed-frame captures
or pipelined workloads to assess an application's frame budget. Run matched
captures before interpreting an optimization as equivalent visual work.

## Measure character crowds

The `crowd` example measures repeated instances of one resolved equipment recipe
with normal pipelined rendering. It uses the same character loader as `capture`,
sharing prepared geometry and static materials while retaining independent instance
animation. Run an optimized build; unoptimized timings do not establish a crowd
frame budget:

```sh
cargo run --release -p bevy-wc3 --example crowd -- \
  /path/to/assets/filedata/BODY.m2 /path/to/recipe.json /path/to/assets \
  /tmp/crowd-100.json --count 100 --frames 300 --warmup 120
```

Native import is strict by default; add `--allow-loss` to explicitly accept
conversion loss, as with `capture`. Compiled rendering diagnostics are printed
and retained in the JSON report. The recipe resolves the same appearance as
the capture recipe branch. The benchmark
accepts counts from zero through one hundred. Keep the model, recipe, sequence,
viewport and quality settings fixed when comparing zero, one, ten and one hundred
characters. The camera and 1280×720 viewport remain fixed; characters occupy a
centered grid with 2.5-unit spacing. Instances start the selected sequence at
staggered phases and advance at a fixed 60 Hz simulation rate, independently of
wall-clock throughput. `--sequence INDEX` chooses the clip. `--frozen` pauses its
clocks, retaining the rig, skinning and rendering path.

The default uses 4× MSAA and a directional light without shadows or optional
prepasses. Add `--shadows` and `--prepasses` independently to measure their costs;
the latter enables depth, normal and motion-vector prepasses. Geometry defaults
to finest authored level zero. `--lod auto` selects the shared medium automatic
policy, while `--lod LEVEL` selects a fixed authored level. Compare LOD policies
separately from renderer optimizations because coarser geometry changes quality.
There is no animation throttling. Compare these settings at a fixed count rather
than attributing every extra pass to animation.

Each invocation writes JSON with steady-state throughput, update latency
percentiles, main/render scheduling windows, extraction CPU time and available
render-pass diagnostics. Extraction measures entity synchronization and copying
render inputs; it excludes waiting for the render thread.
Offscreen rendering has no swapchain to bound queued GPU work. Submission fences
limit this example to three outstanding frames while preserving CPU/render/GPU
overlap. Warmup work is fully drained before measurement. A terminal GPU drain
includes remaining measured work without serializing every frame. Main and render work overlap, so their times must not be
added. Scheduling windows may include other concurrent systems and are not
exclusive profiler samples. GPU pass timings require supported device timestamp
features; absent samples mean unavailable, not zero cost. On macOS, encoder-wide
GPU spans are omitted because Bevy suppresses their timestamp writes. Nested GPU
spans must not be added as independent costs. Diagnostic sample counts can be
lower than measured frames because two initial asynchronous reports per diagnostic
are discarded to exclude delayed warmup samples, and the diagnostic
transport retains the most recent completed report.

The report also records the adapter, build mode, visible draw entities, unique mesh
and material handles, model nodes, skinned draw entities, visible triangle
references, and total draw-palette joint references. These describe resource
sharing and scaling; draw entities are
not hardware draw calls, and palette references are not unique skeleton bones.
Use the zero-character scene to expose fixed scene/renderer cost. Use animated
versus frozen instances to distinguish changing-pose work from repeated render
preparation and GPU drawing. The benchmark reports measurements rather than
asserting a production FPS target or native rendering fidelity; use `capture` for
matched image checks.
