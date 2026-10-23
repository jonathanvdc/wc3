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

To include changing animation, effect execution and uploads, pass
`--benchmark-advance-frames N`. After every requested PNG and frozen-pose benchmark
has finished, the tool warms residency with eight zero-delta frames and then
advances the existing live scene for `N` completed frames at `--fps`. It never
seeks, restarts or resets the instance. This option excludes `--bake-pose`;
scheduled sequence changes must finish by the final requested capture.

`advance-timings.csv` records each rounded input delta, requested accumulated time,
actual Bevy clock, wall/update/wait times and published effect populations.
`advance-programmes.csv` records each programme's actual core clock, populations,
omissions and cumulative execution-route counters, outside the timed interval.
Its frame `-1` row captures the initial instance ledger after warmup, before any
measured advancement. Subtract successive counters to identify actual work per frame;
controllers are included in programme populations. Programme restart retains
execution counters, while replacing the underlying instance begins a new ledger.
An absent instance has an empty clock and zero route counters. These execution
diagnostics are distinct from the simulation birth and omission statistics.
`advance-uploads.csv` contains cumulative render-world transfer snapshots just
before and after the advancing interval. Subtract matching categories to measure
actual birth/sample/resident records, source/view indices and uniform writes.
The initial snapshot follows residency warmup; the final snapshot follows the
last completed frame. Initialized buffer bytes remain separate from queue writes.
These receipts cover shared effect buffers, not textures, meshes or all Bevy work.
Zero birth bytes alone does not establish GPU evolution: require positive actual
immutable route counters and published immutable sprites as well.
PNG readback, observations and CSV writes are excluded from the frame clocks.
Keep the initial captures, FPS, execution selection and advancing frame count
identical in paired runs; frozen-pose timings cannot establish CPU programme
execution speed or changing-record upload cost.

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

## Compare immutable effect frames

The original [immutable-frame workload](../benches/support/immutable_frames/README.md)
compares the same 1,000-particle programme through SIMD CPU appearance and immutable
shader appearance, with either one or 32 independent features. Generate its complete
MDLs, native programme and white sprite into a fresh directory, then run the
paired driver with an optimized, native-feature-enabled capture binary:

```sh
python3 crates/bevy-wc3/benches/support/immutable_frames/generate.py \
  --repo "$PWD" --output /tmp/wc3-immutable-frame-assets
cargo build --release -p bevy-wc3 --features popcorn-jit --example capture
python3 crates/bevy-wc3/benches/support/immutable_frames/compare.py \
  target/release/examples/capture /tmp/wc3-immutable-frame-assets \
  /tmp/wc3-immutable-frame-results --build-label release-feature-enabled --execute
```

Five alternating pairs per case preserve raw completed-frame samples and actual
clock, population, route and transfer receipts. The driver requires real SIMD
work on the CPU route and positive immutable publication/authorizations without
fallback on the shader route. Internal simulation substeps are observed rather
than inferred from FPS. Stable birth uploads are separate from premeasurement
residency transfers and changing CPU sample uploads.

Read both one-feature and 32-feature results: amplification isolates effect cost
but does not remove fixed renderer overhead. The driver reports median paired
ratios, retaining regressions; no speedup is presumed. Inspect images and the
admission ledger before interpreting those ratios. Record hardware, quality,
artifact and raw-file hashes beside per-run route/traffic and paired timing
summaries.

### Initial stationary WGSL snapshot, 2026-10-22

On an Apple M4 Pro with a 20-core GPU and 24 GiB memory, five alternating pairs
per case compared optimized Rust 1.98.1 builds with `popcorn-jit`. The workload
above used 64 advancing completed frames at 64 Hz, a 160×120 viewport, 1× MSAA
and identical coincident billboards. Each emitter retained 1,000 live children
plus its controller. All twenty runs matched published clocks, populations and
budgets, with actual SIMD entries or immutable authorizations and no fallback.
Matched release captures were pixel-identical and inspected.

| Emitters | Particles | SIMD frame median | WGSL frame median | Paired median ratio | Paired p95 ratio |
| --- | --- | --- | --- | --- | --- |
| 1 | 1,000 | 3.289 ms | 3.215 ms | 0.974 | 0.976 |
| 32 | 32,000 | 13.902 ms | 8.895 ms | 0.640 | 0.570 |

Frame medians are medians of five run medians; ratios are medians of five paired
WGSL/SIMD quantile ratios. The one-emitter difference is small, and individual
pairs include regressions. With 32 emitters, median completed-frame time fell
36% and p95 fell 43%; the paired update/submission median ratio was 0.551.
These are completed wall frames, including queue completion, rather than GPU
timestamp or pipelined-throughput measurements.

During the stable 64-frame interval, each SIMD emitter uploaded 7,168,000 sampled
record bytes and 512,000 source/view index bytes. Each immutable emitter instead
uploaded 4,096 uniform bytes, with no birth, sample or index writes. Initial
residency transfers are retained separately. A sparse Lich replay also reduced
birth uploads from 133,120 to 2,080 bytes with the same ten writes and identical
images and simulation observations; that replay establishes transfer efficiency,
not a release frame-time gain for that asset.

The [measurement record](../benches/results/immutable-frames-2026-10-22.json)
retains binary/source/input hashes, all run quantiles, paired ratios, actual work
counters, transfer receipts and raw CSV/log paths. These original synthetic
controls establish a gain for the admitted stationary-billboard workload on this
host. That frozen artifact admitted stationary geometry; the specialization study
below adds moving outputs. Neither establishes cross-engine parity or exact game
fidelity.

### Compact and moving WGSL specialization, 2026-10-22

A later frozen Rust 1.98.1 build on the same M4 Pro compared compact renderer
projections for both stationary and moving geometry. Each study used the same
1,000 children plus one controller per feature, one or 32 features, 160×120
pixels, 1× MSAA and 64 advancing completed frames at 64 Hz after residency warmup.
Five alternating SIMD/WGSL pairs per layout produced twenty runs per study.
The moving workload includes CPU Position projection and per-view sorting in the
completed frame. Its moving particles remain tightly clustered because their
initial birth times differ; it does not establish worst-case spatial ordering cost.

| Geometry | Emitters | SIMD frame median | WGSL frame median | Paired median ratio | Paired p95 ratio |
| --- | ---: | ---: | ---: | ---: | ---: |
| Stationary | 1 | 3.293 ms | 3.231 ms | 0.983 | 0.984 |
| Stationary | 32 | 14.276 ms | 8.911 ms | 0.619 | 0.586 |
| Moving | 1 | 3.407 ms | 3.326 ms | 0.976 | 0.992 |
| Moving | 32 | 15.670 ms | 11.182 ms | 0.714 | 0.925 |

Frame medians summarize five run medians; ratios summarize the five paired
WGSL/SIMD quantile ratios. The one-feature median differences are only 1.7% and
2.4%, with individual regressions, so they do not establish a broad gain. With
32 features, stationary median completed-frame time fell 38.1%; moving median
fell 28.6%. Moving wall p95 improved only 7.46%, while its completion-wait p95
**regressed 14.43%**. Its paired update/submission median ratio was 0.631 and
wait median ratio was 0.810. These independent component quantiles must not be
summed or interpreted as GPU timestamps. Spin-poll completion and all CPU/render
work remain in the wall clock; this is not pipelined throughput.

All forty runs admitted the expected clocks, populations, zero omissions and
actual execution routes. SIMD frames included native batch entries plus scalar
remainders; immutable frames authorized and published every child without
fallback. The record retains the observed work rather than inferring invocation
counts from FPS. During each stable 64-frame interval, each SIMD feature uploaded
7,168,000 sampled-record bytes plus 256,000 natural-index and 256,000 view-order
bytes. Stationary WGSL uploaded only 7,168 uniform bytes. Moving WGSL additionally
uploaded 256,000 view-order bytes; natural indices remained resident. Both WGSL
studies had zero birth and sampled-record uploads during that interval. Startup
residency is recorded separately. These variants use 112-byte uniforms, so their
64 writes are larger than the earlier snapshot's 64-byte uniforms.

Separate original moving controls covered all five supported billboard facing
modes, changing axes, one and 128 children, and four matched times. Of forty CPU/WGSL
image pairs, 39 were exact; the remaining pair differed by one blue-channel unit
at one pixel. Both complete comparison sheets were inspected. An initial GPU
preflight caught the 64/112-byte uniform-layout mismatch; separate typed layouts
resolved it, and the repeated preflight and subsequent captures passed.

The [specialization record](../benches/results/popcorn-immutable-specialization-2026-10-22.json)
retains all paired wall/update/wait quantiles, raw CSV/log hashes, actual work and
transfer receipts, frozen build/host inputs and visual comparisons, including the
failed preflight receipt. These controls establish the measured application-frame
tradeoff on this host, not native-game fidelity or a universal renderer speedup.

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

The report also records the adapter, build mode, actual bindless model-material
selection, schedule executor choice, visible draw entities, unique mesh and
material handles, model nodes, skinned draw entities, visible triangle references,
and total draw-palette joint references. Keep material binding mode fixed when
comparing schedule choices. The resource counts describe sharing and scaling;
draw entities are not hardware draw calls, and palette references are not unique
skeleton bones. Use the zero-instance scene to expose
fixed scene/renderer cost. Use animated versus frozen instances to distinguish
changing-pose work from repeated render preparation and GPU drawing. The
benchmark reports measurements rather than asserting a production FPS target or
native rendering fidelity; use `capture` for matched image checks.
