# Measure renderer performance

Use these examples to compare a fixed workload across renderer revisions or
quality settings. They measure different things: `capture` waits for each completed
frame, while `crowd` retains CPU/render/GPU overlap by default. Run commands from
the workspace root and use [captures](tools.md) separately to check visual quality.

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

## Measure model and character crowds

The `crowd` example measures repeated instances of a model or resolved equipment
recipe with normal pipelined rendering. Recipes use the same character loader as
`capture`; ordinary models use the asset-backed `ModelInstance` loading path.
Both share prepared geometry and static materials while retaining independent
instance animation. Run an optimized build; unoptimized timings do not establish
a crowd frame budget:

```sh
cargo run --release -p bevy-wc3 --example crowd -- \
  /path/to/assets/filedata/BODY.m2 /path/to/recipe.json /path/to/assets \
  /tmp/crowd-100.json --count 100 --frames 300 --warmup 120
```

Use `-` in place of the recipe for an ordinary model, including models with
sprite, ribbon, model-particle or external-programme effects:

```sh
cargo run --release -p bevy-wc3 --example crowd -- \
  /path/to/assets/model.mdl - /path/to/assets /tmp/model-100.json \
  --count 100 --frames 300 --warmup 120
```

Add `--screenshot /tmp/model-100.png` to save the final crowd view. The tool
finishes timing and writes the report before requesting screenshot readback;
readback uses zero simulation delta. Inspect this image to check grid visibility
and material or effect coverage alongside the reported populations.

The ordinary model must be inside the asset root. Resource lookup uses the same
provider as `capture`, including `--file-id ID=PATH` aliases and explicit
`--wmo-profile PROFILE` selection when needed. Missing image bindings and
loading or programme execution failures terminate the run. Models with no clips
use the rest pose when sequence zero is selected.

Native import is strict by default; add `--allow-loss` to explicitly accept
conversion loss, as with `capture`. Compiled rendering diagnostics are printed
and retained in the JSON report. The recipe resolves the same appearance as the
capture recipe branch. The benchmark accepts counts from zero through one
hundred. Keep the model, recipe, sequence, viewport and quality settings fixed
when comparing zero, one, ten and one hundred instances. The camera and 1280×720
viewport remain fixed; instances occupy a centered grid with 2.5-unit spacing.
Instances start the selected sequence at staggered phases and advance at a fixed
60 Hz simulation rate, independently of wall-clock throughput. `--sequence INDEX`
chooses the clip. `--frozen` pauses its clocks, retaining the rig, skinning and
rendering path. It also prevents time-based effect births during
warmup; compare the reported populations before treating a frozen scene as the
same effect workload.

The default uses 4× MSAA (`--msaa 1|2|4|8`) and a directional light without
shadows or optional prepasses. Add `--shadows` and `--prepasses` independently
to measure their costs; the latter enables depth, normal and motion-vector
prepasses. Geometry defaults to finest authored level zero. `--lod auto` selects
the shared medium automatic policy, while `--lod LEVEL` selects a fixed authored
level. Compare LOD policies separately from renderer optimizations because
coarser geometry changes quality. There is no animation throttling. Compare
these settings at a fixed count rather than attributing every extra pass to
animation.

Use `--serial-completed-frames` when measuring advancing simulation with GPU
completion after every frame, for example when another renderer exposes only a
serial submission boundary:

```sh
cargo run --release -p bevy-wc3 --example crowd -- \
  /path/to/assets/model.mdl - /path/to/assets /tmp/model-serial.json \
  --count 100 --msaa 1 --serial-completed-frames --completion-wait spin
```

This mode disables Bevy's pipelined renderer and defaults to nonblocking device
polling with a spin loop, as `capture` does. `--completion-wait blocking`
instead uses the backend's blocking device poll. That API does not promise a
native event-based wait; backend timer granularity can affect the result. The
option is accepted only with serial completion. Keep the submission and wait
policies fixed between revisions, and report separate policy series rather than
attributing a wait-policy difference to renderer performance. The JSON records
both policies, update/submission latency and the separate serial completion
wait. Its `update_wall` field retains the complete per-iteration wall time,
including that wait in serial mode; `update_and_backpressure_wall` excludes the
explicit serial wait. Component percentile values need not sum to the wall
percentile.

Each invocation writes JSON with steady-state throughput, update latency
percentiles, main/render scheduling windows, extraction CPU time and available
render-pass diagnostics. Extraction measures entity synchronization and copying
render inputs; it excludes waiting for the render thread. Offscreen rendering
has no swapchain to bound queued GPU work. Submission fences limit the default
mode to three outstanding frames while preserving CPU/render/GPU overlap; serial
mode completes one frame before starting the next. Warmup work is fully drained
before measurement. A terminal GPU drain includes remaining measured work. Main
and render work overlap in the default mode, so their times must not be added.
Scheduling windows may include other concurrent systems and are not exclusive
profiler samples. GPU pass timings require supported device timestamp features;
absent samples mean unavailable, not zero cost. On macOS, encoder-wide GPU spans
are omitted because Bevy suppresses their timestamp writes. Nested GPU spans
must not be added as independent costs. Diagnostic sample counts can be lower
than measured frames because two initial asynchronous reports per diagnostic are
discarded to exclude delayed warmup samples, and the diagnostic transport
retains the most recent completed report.

Add `--render-stages` for `*_window_ms` spans bracketing adjacent Bevy phases,
including asset preparation, resource uploads and bind-group preparation. Nested queue and
preparation windows overlap their parent windows; do not add them together.
These are scheduling windows rather than exclusive function timings. The markers
also add scheduling and locking overhead and are disabled by default. Compare an
instrumented zero-instance control before attributing a small difference to model work.

The report's `effects` object records published sprite/model-emitter
populations, ribbon sections and segments, programme particles and extracted
geometry, model actor-tree residency, and omission counters. These remain
separate typed counts: programme particles can include controllers, M2 sprite
counters count live birth records, and staged head/tail sprites can count separate records.
Model-tree actors include pending and ready descendants and overlap model-emitter
ownership. None of these counts alone establishes GPU quad count or
visible coverage. Verify the expected populations and omissions for each
workload, then use matched `capture` images to check rendering equivalence. The
`simulate_effects_cpu_ms` span brackets the public simulation system set; like
the other scheduling windows, it can contain unrelated concurrent work.

The report also records the adapter, build mode, visible draw entities, unique
mesh and material handles, model nodes, skinned draw entities, visible triangle
references, and total draw-palette joint references. These describe resource
sharing and scaling; draw entities are not hardware draw calls, and palette
references are not unique skeleton bones. Use the zero-instance scene to expose
fixed scene/renderer cost. Use animated versus frozen instances to distinguish
changing-pose work from repeated render preparation and GPU drawing. The
benchmark reports measurements rather than asserting a production FPS target or
native rendering fidelity; use `capture` for matched image checks.
