# Popcorn execution performance

Use the same compiled graph and simulation workload when comparing execution
backends. Native preparation, pool allocation, warmup and rendering are different
costs; keep them outside a warmed simulation clock and report them separately.
The [fixture guide](../benches/fixtures/particle_runtime/README.md) describes the
original inputs and independent numerical oracle shared by both benchmark tools.

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
exactly; the independent field oracle remains mandatory for both routes.

The Rust route retains its existing pure-packet optimization. The native route
requires successful scalar JIT preparation and one real native call for every
controller and child in each observation frame. Runtime tests separately verify
positive dense Rust packet commits and zero substituted Rust packet commits on
the scalar JIT route. Keep builds and GPU captures idle during a comparison.

## Scalar native measurements, 2026-10-22

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
