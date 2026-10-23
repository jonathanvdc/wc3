# Popcorn execution performance

Use the same compiled graph and simulation workload when comparing execution
backends. Native preparation, pool allocation, warmup and rendering are different
costs; keep them outside a warmed simulation clock and report them separately.
The [fixture guide](../benches/fixtures/particle_runtime/README.md) describes the
original inputs and independent numerical oracle used by the time-driven runtime
benchmarks. The immutable workload below uses a separate direct-Life variant.

## Run a paired comparison

Build the optional native backend and its comparison client, then pass the
resulting executable to the guarded driver:

```sh
cargo build -p wc3 --release --features popcorn-jit --example particle_jit_bench
python3 crates/wc3/benches/compare_particle_jit.py \
  target/release/examples/particle_jit_bench /tmp/particle-jit-results --execute
```

Without `--execute`, the driver only lists its plan and input hashes. A run needs
a fresh output directory. It preserves raw output, preparation times, actual
native invocation counts, block and frame samples, input hashes and failed jobs.
The client checks the full authored trajectory outside its clocks. The driver
requires identical structural results and reports whether numeric words match
exactly; the independent field oracle remains mandatory for all routes.

The default comparison rotates and reverses runs of `interpreter`
(`Interpreter`), `scalar-jit` (`ScalarJit`) and `simd-jit` (`Automatic`). Use
`--routes interpreter,scalar-jit` for a two-route comparison; historical results
below describe their frozen implementation artifacts.

The Rust route retains its existing pure-packet optimization. The scalar native route requires successful JIT preparation and one real native
call for every controller and child in each observation frame. The SIMD route
requires real native batch entries and committed lanes for dense populations,
while short populations retain scalar native calls. Runtime tests separately verify
positive dense Rust packet commits and zero substituted Rust packet commits on
the scalar JIT route. The maintained comparison additionally requires these
actual counters from its own workload. Keep builds and GPU captures idle during a comparison.

## Compare immutable appearance

The separate immutable workload compares the same original checked graph through
Rust packets, scalar native, SIMD and immutable-age execution:

```sh
cargo build -p wc3 --release --features popcorn-jit --example particle_immutable_bench
python3 crates/wc3/benches/compare_particle_immutable.py \
  target/release/examples/particle_immutable_bench /tmp/particle-immutable-results --execute
```

The client embeds four original native fixtures; no runtime fixture directory is
required. The guarded driver rotates and reverses the four routes over the same
population, capacity and instance cases as the native comparison. It checks
structural identity, actual route counters and every declared child field outside
clocks. Constant and lifecycle words remain exact. Size, rotation and color must
pass an independent polynomial oracle with a declared f32 error bound; comparisons
involving analytic appearance allow that bound, while ordinary CPU routes must
remain bit-exact. This is a synthetic workload, not a native asset fidelity claim.

Hot clocks contain 64 natural updates at 1/64 second, with preallocated block and
frame samples and zero allocator-call admission. Compilation, backend preparation,
instantiation and start/first-birth costs are reported separately. A separate
fixed-time measurement reads all fourteen public child fields eight times,
exposing CPU work deferred by immutable appearance. Report that query cost and
preparation/birth costs with advance timings instead of treating them as absent.
This CPU-only client does not measure shader execution or renderer uploads; use
advancing completed-frame captures for those costs. No performance result is
implied merely by adding a route or passing its preparation checks.

To compare extraction methods, the same client accepts
`--query-method individual|bulk|projection`, defaulting to individual reads.
The [query driver](../benches/support/immutable_queries/README.md) compares the
same fourteen-field request across all four routes and three methods:

```sh
python3 crates/wc3/benches/support/immutable_queries/compare.py \
  target/release/examples/particle_immutable_bench /tmp/particle-query-results --execute
```

It uses five rotating/reversing rounds at two dense layouts and retains raw logs,
actual dispatch, cold projection costs and zero-allocation checks. Position-only
projection queries are a separately named workload and never replace full-field
queries in its rankings.

## Scalar native measurements, 2026-10-22

The historical synthetic fixture below did not qualify for Rust packets because
its `Enabled` boolean copy was outside that admission subset. Its Rust timings
therefore measure scalar execution. The scalar speedups remain valid for those
artifacts, but do not establish a win over Rust packets. The explicit SIMD stage
adds boolean-copy admission and compares all three routes with actual counters.

These CPU measurements used an Apple M4 Pro (Mac16,8, 14 logical CPUs), macOS
27.0.1, Rust 1.99.0 release optimization and Cranelift 0.137.0 with `opt_level=speed`.
The native implementation lowers checked arithmetic, accesses canonical field
planes directly and retains checked Rust services for curves and other callbacks.
It performs an exact field-layout comparison for each invocation. No explicit
SIMD or analytic shader execution contributes to these results.

Each synthetic case used five alternating pairs, 31 samples per run, two warmup
ticks and 64 measured ticks at 1/64 second. Child capacity excludes its one
controller. Speedup is the reciprocal of the median paired block-time ratio;
the p95 column is native frame p95 divided by Rust frame p95, with smaller values
indicating less time. Every synthetic result was bit-exact across routes and
passed the independent field oracle.

| Children | Child capacity | Instances | Speedup | Frame p95 ratio |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 128 | 1 | 1.92× | 0.539 |
| 1 | 128 | 8 | 2.09× | 0.475 |
| 8 | 128 | 1 | 3.03× | 0.339 |
| 8 | 128 | 8 | 3.12× | 0.308 |
| 1 | 1024 | 1 | 2.01× | 0.500 |
| 1 | 1024 | 8 | 2.09× | 0.485 |
| 8 | 1024 | 1 | 3.09× | 0.328 |
| 8 | 1024 | 8 | 3.13× | 0.303 |
| 128 | 128 | 1 | 3.56× | 0.292 |
| 128 | 128 | 8 | 3.54× | 0.281 |
| 1024 | 1024 | 1 | 3.59× | 0.276 |
| 1024 | 1024 | 8 | 3.54× | 0.289 |

The acquired HD LichAttack effect supplied a separate live workload. Its five
alternating pairs used emission multipliers 1 and 64, total pool capacities 128
and 1,024, and one or eight instances. Each trajectory warmed for 120 ticks and
measured 600 ticks at 1/64 second. All structural and full-field trajectory hashes
matched exactly, and steady advancement made no allocator calls. These are
simulation measurements; they include controllers, callbacks and scheduling.
Lich speedups use paired frame medians; synthetic speedups above use paired
64-tick block medians.

| Total capacity | Instances | Emission multiplier | Speedup | Frame p95 ratio |
| ---: | ---: | ---: | ---: | ---: |
| 1024 | 1 | 1 | 3.00× | 0.364 |
| 1024 | 1 | 64 | 1.66× | 0.597 |
| 1024 | 8 | 1 | 2.90× | 0.377 |
| 1024 | 8 | 64 | 1.60× | 0.728 |
| 128 | 1 | 1 | 2.92× | 0.383 |
| 128 | 1 | 64 | 1.75× | 0.481 |
| 128 | 8 | 1 | 2.98× | 0.361 |
| 128 | 8 | 64 | 1.73× | 0.587 |

Synthetic scalar preparation took approximately 1.7 ms on this host, outside hot
clocks. Preparation is shared by instances; it is not paid for each update. The
[recorded results](../benches/results/popcorn-scalar-jit-2026-10-22.json) retain
paired quantiles, preparation observations, allocation totals and artifact/source
hashes. The optimized native corpus check additionally matched 58 effects across
330 cases and 83,064 observation rows exactly. These observations establish the
measured workloads on this host; they make no cross-engine or completed-frame
rendering claim.

## Inline curves, 2026-10-22

The next scalar optimization embeds eligible immutable curves directly into
machine code. Scalar branches and SIMD lane masks share their interpolation
algebra, while the unchanged Rust sampler remains the numerical oracle. Curves
outside the bounded knot/code budget retain checked services.

The same host, compiler, fixtures and five-pair protocol produced the following
block-median speedups. The previous-scalar column comes from a direct paired
comparison of the two frozen native artifacts, rather than dividing separate
Rust-relative measurements. Every compared output remained bit-exact.

| Children | Child capacity | Instances | Versus Rust | Versus previous scalar |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 128 | 1 | 2.18× | 1.14× |
| 1 | 128 | 8 | 2.31× | 1.13× |
| 8 | 128 | 1 | 4.35× | 1.40× |
| 8 | 128 | 8 | 4.36× | 1.40× |
| 1 | 1024 | 1 | 2.17× | 1.15× |
| 1 | 1024 | 8 | 2.24× | 1.08× |
| 8 | 1024 | 1 | 4.25× | 1.44× |
| 8 | 1024 | 8 | 4.41× | 1.39× |
| 128 | 128 | 1 | 6.01× | 1.66× |
| 128 | 128 | 8 | 5.82× | 1.66× |
| 1024 | 1024 | 1 | 6.08× | 1.66× |
| 1024 | 1024 | 8 | 5.92× | 1.68× |

The eight Lich cases improved to 2.29–4.05× the Rust evaluator using paired frame
medians. Every trajectory still matched exactly and steady advancement made no
allocator calls. The [curve-stage record](../benches/results/popcorn-scalar-curves-2026-10-22.json)
retains paired ratios and workload/artifact provenance. The checked feature suite
passed 300 unit and seven integration tests, including distinct incoming/outgoing
tangents, nonuniform eight-knot curves, exceptional coordinates and failure prefixes.

## Explicit SIMD, 2026-10-22

Automatic native preparation now lowers the shared checked pure plan to fixed
SIMD groups. Particle batch size, native lane count and value component count
have separate named constants: this build uses 16 particles, four SIMD lanes
and four value words. Generated instructions specialize those dimensions before
execution. The Rust packet and native tile both gather only old field values
actually read by the programme, component by component, with a bulk-copy path
for contiguous slots. Write-only outputs and inactive components need no gather.

The first measured native tile was 2–10% slower than scalar JIT on dense synthetic
cases. Input-only gathering and one shared checked Time input reversed that
result. A direct five-pair comparison of the frozen initial and revised SIMD
artifacts measured a further 1.84–1.88× speedup, with exact outputs and actual
native entries in both builds. The failed preparation attempt and slower
measurements remain in the evidence record.

These final comparisons used the same host, compiler and synthetic protocol as
the scalar stages, with five rotated/reversed triples. Unlike the historical
synthetic results above, this fixture now qualifies for the Rust packet through
its boolean-copy support. Every dense Rust frame committed its child population
through that packet; native frames used exactly the required SIMD entries and
committed lanes. All 180 synthetic jobs passed the independent field oracle and
matched full trajectories exactly. The frame p95 column is SIMD divided by scalar
JIT; smaller is faster.

| Children | Child capacity | Instances | Versus Rust packet | Versus scalar JIT | Frame p95 ratio |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 128 | 128 | 1 | 2.16× | 1.63× | 0.626 |
| 128 | 128 | 8 | 2.12× | 1.63× | 0.602 |
| 1024 | 1024 | 1 | 2.19× | 1.65× | 0.600 |
| 1024 | 1024 | 8 | 2.17× | 1.63× | 0.631 |

The acquired Lich workload retained its continuous 120-tick warmup and 600-tick
measurement. Dense emission multiplier 64 produced real SIMD work, zero Rust
packet fallback and the following paired frame results. All 120 jobs retained
exact structural/full-field trajectories and made no steady allocator calls.

| Total capacity | Instances | Versus Rust evaluator | Versus scalar JIT | Frame p95 ratio |
| ---: | ---: | ---: | ---: | ---: |
| 128 | 1 | 2.91× | 1.21× | 0.885 |
| 128 | 8 | 2.93× | 1.23× | 0.806 |
| 1024 | 1 | 2.92× | 1.37× | 0.691 |
| 1024 | 8 | 2.62× | 1.31× | 0.661 |

Sparse synthetic and multiplier-1 Lich observations made no SIMD calls. They
retain scalar native execution; their raw observations remain in the record
without a SIMD speed claim. Synthetic native preparation took approximately
1.8–2.0 ms for scalar-only execution and 2.9–3.3 ms with SIMD on this host.

The [SIMD record](../benches/results/popcorn-simd-jit-2026-10-22.json) retains all
paired summaries, cold/allocation observations, actual route evidence and frozen
artifact/source hashes. Validation passed 322 feature-enabled unit tests and
seven integration tests, 295 feature-disabled unit tests and six integration
tests, plus strict Clippy. A focused native-code check executed the dumped
2,836-byte AArch64 kernel and confirmed packed add/subtract/multiply instructions.
The optimized corpus again matched 58 effects, 330 cases and 83,064 rows exactly.

These are CPU simulation results. x86-64 execution, larger supported curves,
unmeasured effects and completed rendered frames have no performance claim here.

## Immutable CPU appearance, 2026-10-22

This stage keeps controller execution, births, lifecycle retirement and bounded
work on the CPU, while eligible appearance fields use a closed age expression.
Public field queries evaluate that expression on demand. The following results
measure CPU advancement and its deferred query cost; WGSL and completed rendered
frames have not been measured in this record.

The frozen release build used the same M4 Pro host, Rust 1.99.0 and Cranelift
0.137.0; its build took 5m44s. The synthetic comparison completed 240 guarded jobs:
five rotated/reversed four-route runs over twelve population/capacity/instance
cases, each with 31 samples of 64 ticks at 1/64 second after two warmup ticks.
All four routes passed the independent fourteen-field oracle and matched numeric
words exactly, although the analytic appearance oracle permits a declared bounded
rounding difference. Dense Rust runs actually committed packets, Automatic runs
committed SIMD lanes, scalar-native runs used scalar entries, and immutable runs
authorized appearance without substituting
packet work. Every hot block made zero allocator calls.

Speedups below are reciprocals of the median of five paired block-median ratios
against the strongest current dense alternative, SIMD. The frame p95 column is
immutable divided by SIMD, again paired per repeat; smaller is faster.

| Children | Child capacity | Instances | Versus SIMD | Frame p95 ratio |
| ---: | ---: | ---: | ---: | ---: |
| 128 | 128 | 1 | 3.71× | 0.265 |
| 128 | 128 | 8 | 3.53× | 0.301 |
| 1024 | 1024 | 1 | 3.91× | 0.271 |
| 1024 | 1024 | 8 | 3.89× | 0.256 |

The acquired HD LichAttack programme completed 160 guarded jobs, with 120 warmup
and 600 measured ticks at 1/64 second, total capacities 128/1024, one/eight instances
and emission multipliers 1 and 64. Birth identities, ordering, canonical lifetime and
budget ledgers matched the ordinary evaluator. Only authorized virtual float
outputs used the declared 512-epsilon relative-or-absolute bound; other words and
inactive components remained exact. The largest absolute appearance difference
was below 0.000947 and consumed less than 64% of that bound. Capacity-limited birth
omissions remain in the matched ledgers; the comparison does not hide them by
changing capacities or spawning policy. Steady advancement made no allocator
calls. All observed immutable particles had coherent authorized sample times.

Lich speedups use paired frame medians against SIMD, with the same paired p95
ratio definition. Sparse cases made no SIMD entries and compare against its
ordinary scalar fallback; dense cases had actual SIMD entries.

| Total capacity | Instances | Emission multiplier | Versus Automatic | Frame p95 ratio |
| ---: | ---: | ---: | ---: | ---: |
| 128 | 1 | 1 | 1.90× | 0.700 |
| 128 | 1 | 64 | 2.43× | 0.574 |
| 128 | 8 | 1 | 1.92× | 0.779 |
| 128 | 8 | 64 | 2.16× | 0.501 |
| 1024 | 1 | 1 | 1.90× | 0.821 |
| 1024 | 1 | 64 | 2.50× | 0.470 |
| 1024 | 8 | 1 | 1.96× | 0.787 |
| 1024 | 8 | 64 | 2.59× | 0.429 |

Immutable advancement defers CPU field evaluation rather than removing its cost.
The separate fixed-time eight-sweep query measurement was approximately 17–18×
slower than SIMD for dense synthetic instances. Compilation, native preparation,
instantiation, start/first-birth and query observations are recorded separately;
none is included in the hot advance clock. For one 1,024-child instance, median
pool creation was 34 µs and start/first-birth 169 µs. Eight complete query sweeps
cost 10.63 ms versus 0.595 ms for SIMD; the median of paired query ratios was 17.68×.
These are separate phase medians, not components of an advance-time ratio.
Applications extracting every field on the CPU must include that query cost in
their comparison. These measurements do not establish renderer upload savings or
a total-frame improvement.

The [immutable record](../benches/results/popcorn-immutable-age-2026-10-22.json)
retains actual route deltas, all paired ratios, cold/birth/query costs, numerical
contracts and hashes/paths for preserved raw logs and frozen artifacts. Its
source snapshot precedes subsequent packet specialization; no later unmeasured
change contributes to these results.

The subsequent const-generic packet refactor retains the private default of 16
particles and four native SIMD lanes, independently of the VM's four-word values.
The [default-layout receipt](../benches/results/popcorn-simd-default-layout-2026-10-22.json)
records byte-identical 2,836-byte machine code and executed parity for the same
diagnostic kernel before and after this refactor. This establishes code identity
for that kernel, not all programmes or rendered-frame performance.

## Bulk and projected queries, 2026-10-22

The next study compares individual reads, one-pass `sample_fields`, and a
prepared `ImmutableProjection` within the same frozen binary. Each method returns
all fourteen public child fields, combining projected virtual outputs with unchanged
canonical fields. This measures deferred CPU extraction, separately from
advancement; it does not measure WGSL, renderer uploads or completed frames.

The release artifact used an Apple M4 Pro with 14 logical CPUs, macOS 27.0.1,
Rust 1.98.1 and Cranelift 0.137.0. This is a different compiler snapshot from the
historical measurements above. The driver completed 120 guarded processes: five
rotating/reversing rounds of four routes and three query methods, at 128 children
in one instance and 1,024 children in eight instances. Child capacity equals the
population; each pool also reserves one controller slot. Each process uses five
samples, two warmup ticks and 64 measured ticks per sample at 1/64 second. Its
separate query clock performs eight fixed-time sweeps of the complete field set.

Every route passed the independent authored oracle, public structure and actual
dispatch checks. All observed full-field numeric hashes were identical, including
ImmutableAge, although its declared Graphics comparison still permits bounded
appearance differences. Every measured advance and full-field query interval
made zero allocation, reallocation and deallocation calls.

Within ImmutableAge, bulk and prepared queries reduced the cost of repeated
individual reads. The speedups below are reciprocals of the median of five
paired query-median ratios. The projected cost is the median of the five process
query medians; the slowdown column uses paired ratios against the strongest
observed materialized-field query configuration. All columns concern the same
fourteen-field request.

| Children | Instances | Bulk vs individual | Projection vs individual | Projected eight-sweep cost | Projection vs materialized query |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 128 | 1 | 4.37× faster | 6.39× faster | 0.205 ms | 2.63× slower |
| 1024 | 8 | 4.39× faster | 6.38× faster | 13.437 ms | 2.64× slower |

Prepared full-field extraction therefore remains more expensive than reading
ordinary, already materialized fields. The strongest observed configuration was
Interpreter with the requested projection method, which uses canonical bulk
sampling when no immutable recipe exists. That label does not mean an immutable
projection was prepared for the ordinary route, or establish a difference in VM
execution speed. Applications should compare their actual extraction workload,
not infer a total-frame gain from reduced advancement alone.

Advancement remained separate from those query clocks. Comparing only the five
paired runs using the same individual query method gives the results below;
the other method runs are not additional independent advance pairs. Speedup uses
paired frame medians against SIMD, and p95 is ImmutableAge divided by SIMD.

| Children | Instances | Advance vs SIMD | Frame p95 ratio |
| ---: | ---: | ---: | ---: |
| 128 | 1 | 3.79× faster | 0.223 |
| 1024 | 8 | 3.76× faster | 0.252 |

The full query projection contains 22 nodes, six captures and six virtual outputs;
the remaining requested fields are canonical reads. A separate Position-only
projection contains one node, one capture and one output. Its raw timings and
both projections' cold preparation costs remain in the record, outside hot
advancement and full-field rankings. A one-field result is not an equivalent
fourteen-field query.

The [query record](../benches/results/popcorn-immutable-queries-2026-10-22.json)
retains the complete summaries, paired ratios, compact exact dispatch/zero-
allocation ledgers, per-job configurations and raw-log hashes. It also identifies
the full preserved manifest and frozen build/host receipts by path and SHA256.
Its 94 guarded compiled inputs and measured binary match the build receipt.
These are public workload results on this host, with no private VM-state,
native-game fidelity, cross-engine or rendering-performance claim.

## Plan preparation and live reuse, 2026-10-22

Execution wrappers for one immutable graph can reuse a live plan prepared for
exactly the same `ExecutionOptions`. A same-wrapper request returns its existing
plan; a sibling wrapper can find a live exact-policy plan through the graph's
weak cache. The cache does not keep executable code alive after its last owner
drops. These are preparation paths, separate from particle advancement and field
sampling.

Reproduce the focused preparation workload with the maintained example:

```sh
cargo run -p wc3 --release --features popcorn-jit --example particle_plan_bench -- 15
```

The recorded run used the same original 128-child fixture, M4 Pro host, Rust
1.98.1 and Graphics policy as the query study. Parsing and compiling its source
graph occur before measurement. Each case has 15 samples. Cold requests prepare
one plan per sample and drop it before the next request, leaving no strong cache
owner. Their median preparation times were:

| Preference | Cold single-call median |
| --- | ---: |
| ScalarJit | 1.664 ms |
| Automatic | 2.645 ms |
| ImmutableAge | 2.272 ms |

Live cases hold a prepared owner throughout measurement. Their requests and
output capacity are prepared before timing; options and inputs pass through
`black_box`. Batches of 1,024 or 4,096 calls avoid treating a single-call timer
quantum as an exact cost. Reported per-call values below divide the batch median
by its call count. They include loop, Vec draining/push/movement and actual API
work, including Arc operations. They are not isolated cache-lookup timings or
timer-overhead subtractions.

| Preference | Calls per sample | Same-wrapper estimate | Live sibling estimate |
| --- | ---: | ---: | ---: |
| ScalarJit | 1024 | 2.40 ns/call | 7.32 ns/call |
| ScalarJit | 4096 | 3.01 ns/call | 7.50 ns/call |
| Automatic | 1024 | 2.77 ns/call | 7.36 ns/call |
| Automatic | 4096 | 2.93 ns/call | 7.36 ns/call |
| ImmutableAge | 1024 | 2.65 ns/call | 7.24 ns/call |
| ImmutableAge | 4096 | 2.87 ns/call | 7.71 ns/call |

All 180 hot batch samples made zero calling-thread allocation, reallocation and
deallocation calls, covering 460,800 requests. Each of the 45 cold samples made
real allocations. Wrapper cloning, report checks and outer wrapper/Vec destruction
are excluded equally; cold preparation can include cleaning an expired weak
entry. Requested backend reports and exact options were checked after each call.
No particle programme was executed in these clocks.

The [preparation record](../benches/results/popcorn-plan-preparation-2026-10-22.json)
preserves raw totals, medians, batch sizes, per-call estimates, allocator rows and
frozen source/fixture/binary/host receipts. This was one fixed-order run, not a
paired arithmetic-throughput or rendered-frame comparison. Earlier coarse
single-call hot timings were timer-quantized and are excluded from this record.
