# Performance benchmarks

The workspace uses Criterion to measure model codecs, texture codecs, archive
operations, animation, and Bevy integration. Every workload is generated
deterministically or uses a checked-in synthetic fixture. Running the suites
requires neither downloaded game assets nor network access after Cargo dependencies
are available. Benchmark IDs identify the operation and its input parameters, so
individual workloads can be selected when investigating a change.

## Run the CI workload

For the short set used on every CI push and pull request, run:

```sh
bash benchmarks/run-quick.sh
```

This selects representative workloads from all eight targets, reduces fixture
construction, and uses ten samples with 50 ms warmup and 100 ms measurement
windows, 1,000 bootstrap resamples, and no plot generation. It retains optimized
builds and the same workload IDs as the broader suite. Quick results live under
`target/criterion/quick`, keeping their shorter sampling windows separate from
normal local measurements.

Use the short measurements to check timing loops and spot large changes. Repeat
small differences locally before treating them as regressions on shared runners.
Cold compilation adds to measurement time, especially for Bevy. GPU measurements
remain a local check.

Use `bash benchmarks/run-quick.sh --test` for the representative smoke set,
or `bash benchmarks/run-quick.sh --save-baseline NAME` to retain its measurements.
`WC3_BENCH_QUICK` reduces input combinations; the script also supplies the operation
filter and timing settings. Quick mode takes precedence over `WC3_BENCH_LARGE`.

## Run targeted measurements

The suites separate operations with different costs so a result can be tied to
the path affected by a change. The following overview identifies each target's
scope; its benchmark IDs distinguish the individual operations and input sizes.

| Target | Workloads |
| --- | --- |
| `wc3/model` | MDX read/write and typed traversal, MDL parse/write, version conversion, geometry access, and resource discovery in 800 and 1800 layouts. |
| `wc3/animation` | Scalar, vector, and quaternion sampling across interpolation modes, key counts, and access patterns; sequence/global clock boundaries and event traversal. |
| `wc3/evaluation` | Prepared pose evaluation, node transforms, and projection of 1,024 triangles. |
| `wc3/blp` | Container read/write, base-level and full-chain pixel decoding, caller-provided output buffers, and encoding with generated or supplied mipmaps. Indexed alpha, JPEG, BGRA, and DXT1/3/5 are covered. |
| `wc3/mpq` | Archive creation/opening, lookup, serial/shared extraction, compression, encryption, sector framing, and all four header versions. Synthetic streams cover decode-only codecs and chains. |
| `bevy-mpq/assets` | Direct extraction versus async reads, concurrent batches at different semaphore limits, and overlay hits/misses at different depths. |
| `bevy-wc3/preparation` | Universal import and compilation, and shared mesh and bind-pose asset preparation. |
| `bevy-wc3/runtime` | Instance spawn/despawn and headless updates, including animation, materials, transforms, PREM model particles, PRE2 quad particles, and ribbons. |

Choose the target that matches the changed code. These commands run optimized
measurements with Criterion's normal sampling settings:

```sh
cargo bench -p wc3 --bench model
cargo bench -p wc3 --bench animation
cargo bench -p wc3 --bench evaluation
cargo bench -p wc3 --features blp-decode,blp-encode --bench blp
cargo bench -p wc3 --features mpq-decode,mpq-encode --bench mpq
cargo bench -p bevy-mpq --bench assets
cargo bench -p bevy-wc3 --bench preparation
cargo bench -p bevy-wc3 --bench runtime
```

Pass a filter after `--` to measure a particular workload, for example
`cargo bench -p wc3 --bench model -- 'model/unit/mdx/read/1800'`.
Use `-- --test` to execute each selected workload once and check fixture
construction and output validation without collecting statistical measurements.
For a complete CPU smoke run, use
`cargo bench --workspace --all-features -- --test`.

Set `WC3_BENCH_LARGE=1` to include the building model, 4,096-key tracks,
larger textures and archive entries, and 128 effect instances. These cases are
deliberately excluded from the default set. Criterion filters select measurements;
fixture construction and validation can still occur for unselected cases.
The BLP suite includes costly palette quantization and block compression, so a
complete statistical run takes substantially longer than its smoke run.

## Interpret results

Read latency together with the throughput unit. Codecs report bytes or pixels,
track sampling reports samples, pose evaluation reports nodes, and Bevy workloads
report vertices or instances.
Chain decoding counts all mip pixels; chain encoding counts source pixels.
Runtime instance throughput counts instances updated per second. It does not
measure rendered frames per second.

Fixtures, file creation, and correctness checks happen before measurement.
Prepared pose evaluation reuses its output storage and a previously sampled
animation; it does not time animation compilation or sampling.
Spawn/despawn setup and teardown are excluded, as is teardown of the asset stores
returned by preparation. Ordinary codec loops include destruction of their outputs. The geometry-access
workload borrows records and their encoded `ZeroVec` arrays without cloning
buffers. Despawning measures entity removal; later asset garbage collection is
excluded.

The MPQ file-backed case measures a warm filesystem cache. Tiny fixed codec
streams exercise decode paths but do not predict large-file throughput. Async
reads include consumption of the returned reader, and batch requests are polled
concurrently. Validation warms the blocking workers before timing.

Runtime updates advance a manual 1/60-second clock after warmup. Animation keeps
looping between iterations, and effects maintain bounded live populations. Paused
cases seek every instance to 0.5 seconds before freezing; the one-animated case
keeps the remaining instances at that pose. PREM
loads its checked-in child model and texture before measurement. Synthetic
geometry preparation resolves no images, and headless runtime updates have no
GPU renderer. Use the capture measurements below to investigate rendering costs.

The [fixture design guide](fixtures.md) describes how the synthetic inputs preserve
useful asset characteristics and how to revise them without introducing an
external corpus dependency.

## Compare revisions

Criterion stores results under `target/criterion`. Add
`-- --save-baseline NAME` to retain a reference, and later use
`-- --baseline NAME` to compare against it. Compare the same benchmark IDs,
features, compiler, build profile, and machine. Keep the machine otherwise idle
and repeat suspicious changes before treating them as regressions. These suites
do not impose performance thresholds on shared CI hosts.

## Host GPU measurements

The existing capture tool also measures synchronized native GPU frames. It
waits for loading and pipelines, simulates from time zero to each requested
pose, warms eight synchronized frames, measures repeated zero-delta frames,
and writes a CSV beside the normal PNG. Each sample includes the main-world
update, render extraction and submission, and waiting for the submitted GPU work.
PNG encoding, screenshot readback, file writes, and loading are outside the measured interval. This is wall time
for a completed frame, not GPU timestamp time or free-running FPS.

Export a synthetic model and texture, then capture and measure it:

```sh
cargo run -p wc3 --features blp-encode --example benchmark_fixtures -- /tmp/wc3-bench-assets
cargo run --release -p bevy-wc3 --example capture -- \
  /tmp/wc3-bench-assets/unit-1800.mdx /tmp/wc3-gpu-bench \
  --replaceable 1=synthetic/diffuse.blp \
  --times 0.5,1 --fps 60 --eye 0,-32,24 --target 0,-5,3 \
  --benchmark-frames 120
```

The CSV records each frame's wall time, and the tool prints median and p95.
Always inspect the companion images before drawing conclusions about rendering.
Keep model, resolution, camera, LOD, prepasses, lighting, simulation FPS, and
capture times fixed between comparisons. The timed pose is frozen, including its
live effect population; use the CPU runtime suite for advancing simulation costs.
For effect GPU cases, use the checked-in particle and ribbon capture fixtures
with the same flag. Native GPU access is required, and device/pipeline/readback
failures are errors rather than successful performance measurements.
