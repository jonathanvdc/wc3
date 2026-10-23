# Compare immutable field queries

Use this driver with one frozen, feature-enabled `particle_immutable_bench`
binary to compare repeated individual fields, bulk sampling and a prepared
projection. Each method returns the same fourteen fields of the original
checked-in immutable fixture. The four execution routes are Interpreter,
ScalarJit, Automatic SIMD and ImmutableAge, all under the client's Graphics
policy. This is a CPU query comparison; it measures no Bevy rendering or GPU
work and does not establish native-client fidelity.

Build the maintained example separately and keep its build receipt. Then inspect
the planned hashes and process order before explicitly executing:

```sh
cargo build -p wc3 --release --features popcorn-jit --example particle_immutable_bench
python3 crates/wc3/benches/support/immutable_queries/compare.py \
  target/release/examples/particle_immutable_bench /tmp/wc3-query-comparison
python3 crates/wc3/benches/support/immutable_queries/compare.py \
  target/release/examples/particle_immutable_bench /tmp/wc3-query-comparison \
  --samples 5 --execute
```

The first invocation only prints the plan and creates no result directory.
Execution requires a fresh output path; failed evidence is never overwritten.
`--samples` accepts five or seven samples per client process. There are five
rounds for each of two layouts: N128/C128/I1 and N1024/C1024/I8. N counts visible
children, C is child capacity, and the client reserves C+1 total slots for its
controller. Every round runs all twelve route/method configurations. Routes and
methods rotate, and odd rounds reverse their order, yielding 120 serial
processes. Keep other builds, GPU captures and timings idle during the study.

## Timing and preparation boundaries

The client separately measures cold graph compilation, execution preparation,
instantiation, birth/warmup, advance-only frames and fixed-time query sweeps.
A full query sample performs eight sweeps of fourteen fields over every child
in every instance, with no simulation advance or oracle assertions in that
query clock. Full-field query rankings compare only these identical requests;
advance medians/p95 remain a separate ledger. Durations include the client's
public API calls and value consumption. They are not isolated arithmetic-kernel
or cache-lookup timings.

Projection mode prepares virtual-field dependencies, reusable scratch and
raw/output mappings before steady clocks. Raw fields are combined without
running the full immutable DAG again. The client reports preparation cost and
[node,capture,output] counts separately. CPU routes have no immutable projection,
so their requested projection method takes the documented canonical bulk
fallback and reports zero preparation counts. Their labels do not imply GPU
or immutable-plan admission.

For ImmutableAge/projection, the client also reports a Position-only projection
sweep. That is one field, so this driver preserves it in each job's separate
costs but never ranks it against fourteen-field queries. Cold preparation is
likewise retained separately and is not folded into the steady query ratio.

## Admission and evidence

The driver reuses `compare_particle_immutable.py`'s maintained result parser,
route-counter checks and numeric comparison. Each client first validates every
field and frame against its independent authored oracle and compares its chosen
query method with individual reads. The driver requires matching public
structure across every method, route and repeat. Field bits must match exactly
within an execution route across methods and repeats. Across CPU routes all bits
match; when ImmutableAge participates, only the authored floating appearance
fields 9, 10 and 12 may differ within the existing predeclared bound of
128*f32::EPSILON*max(1,abs(a),abs(b)). Life, identity, Boolean fields, inactive
words and other outputs remain exact. The driver's checksum covers little-endian
u32 public field words; it is not a checksum of private VM temporaries or latent
random-generator state.

Preparation reports alone cannot pass dispatch admission. The client and parser
require actual per-frame scalar, SIMD and Rust packet counters for the requested
route, plus immutable invocations/fallback counts. Both hot advancement and
full-field query intervals must report zero allocation/reallocation/deallocation.
An unavailable backend, failed assertion, malformed result, stderr output,
timeout or changed guarded file stops the run.

`manifest.json` records every command, raw timing/counter/cost ledger, query
median and p95 ratios paired by round, and ratios to the strongest observed
full-query median. The raw stdout is preserved losslessly as `.stdout.gz` to
avoid repeating large numeric arrays in the manifest; stderr remains a separate
file. Both decompressed and compressed stdout SHA256 values are recorded.
Failed runs keep their logs and manifest rather than becoming partial successes.

The binary, example, shared fixture oracle, reused protocol, Cargo inputs,
fixture descriptors/generators and relevant core source inventories are hashed
before and after every process and at completion. Those guards establish that
the observed inputs stayed unchanged. They do not prove the executable was
built from those sources: retain the compiler/feature/source/binary build receipt
alongside the results. No benchmark results or speedup claims are checked in by
this driver.
