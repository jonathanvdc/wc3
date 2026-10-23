# Delta-affine specialization benchmark

These are original CPU simulation controls. They do not establish native game
fidelity or GPU rendering performance. Generate only missing density fixtures
from the repository root; the author refuses to overwrite existing files:

```sh
python3 crates/wc3/benches/fixtures/particle_delta/generate.py
cargo build --release -p wc3 --example particle_delta_bench --features popcorn-jit
```

The author reuses the maintained original PKB graph/container author. Its four
new files contain a one-shot prefix controller and 1, 128, 1,024 or 8,192 children.
Children live for 8s and update only two fields: Position += velocity × Delta and
Rotation += rate × Delta. Position starts at (0.125,0.25,-0.5), velocity is
(1,-0.5,0.25), rotation starts at 3 and angular rate is 2. Births span the first
1/64s interval; they are not artificially synchronized. Total particle capacity
is N+1, including the ordinary controller, and the pending-event and occurrence
capacities match that total. Other capacities retain production defaults.

The separate `routed 1` control reuses the checked-in original
`tests/fixtures/pkb/delta-affine.pkb`. Its root emits a child at fraction
f32(0.4) of every actual step. Lifetime is 0.25s; rotation starts at 3 and
integrates rate 2. After warmup, 16 children remain resident. Each next tick
retires one, evolves 15 survivors and initializes/evolves one partial newborn.
It measures ongoing births, deaths and slot reuse, not the density control's
stable no-birth interval. Core execution checks need no image files; renderer
metadata in these fixtures is not a GPU acceptance claim.

Each process first compares the selected route against ordinary interpreted
source for all 96 warmup/measured steps. This checks all 14 child fields and
all root fields, identity, generation, parent, serial, birth time, lifetime,
population, clock and omission counters. Canonical lifetime, root values,
Boolean bits and inactive words remain exact. Ordinary floating fields use the
fixed bound `abs(a-b) <= 2e-6 + 2e-6*max(abs(a),abs(b))`. A separate numerical
oracle checks the actual rounded first partial Delta and subsequent intervals.
Immutable particles must expose exactly that integrated Delta sample. No
numerical bound changes automatically after a failure.

Prepared backend reports and actual per-frame execution counters are separate.
Dense SIMD must commit real tiles; dense Interpreter must commit Rust packet
lanes. Density1 and routed SIMD requests legitimately use scalar tails, because
there is no full admitted tile. Scalar interpreter calls have no public
invocation counter and are not assigned fabricated counts. Every measured
interval must retain zero omissions and zero immutable fallbacks.

Each sample uses a fresh instance, 32 warm frames and 64 measured advances of
1/64s, covering 0.5–1.5s. Five samples are required. Reading/compiling/preparing,
instance allocation, start plus first birth tick, and warmup are outside advance
clocks and reported separately. Per-frame duration storage is preallocated.
The client compares the measured endpoint and aggregate route counts against
its admitted trajectory. It separately measures eight full-field bulk sweeps
and eight selected Position/Rotation sweeps (Rotation only for routed).
Immutable selected queries use a prepared projection; other routes read the
same fields directly. Full-query and selected-query costs must not be treated
as equivalent work or folded into an advance-only ratio.

Rust global-allocator allocation, reallocation and deallocation deltas must all
be zero for both hot advancement and hot queries. These counters do not observe
foreign C or driver allocators. Observation, checking and logging happen outside
the per-frame clocks.

Run the guarded comparison with an immutable release binary and a fresh output
directory:

```sh
python3 crates/wc3/benches/fixtures/particle_delta/compare.py \
  --repo "$PWD" --binary /absolute/path/to/particle_delta_bench \
  --output /tmp/wc3-delta-results-01 --execute
```

Omit `--execute` to print the plan and input hashes without creating output. The driver requires disabled debug assertions and runs 100
processes: five rounds × five workloads × four requested routes. Route order
rotates by workload and reverses on alternate rounds. Each process performs
five samples. `--timeout` defaults to 600s per process. The runner verifies binary,
Rust sources, fixture authors, all five PKBs and Cargo manifests/lock hashes
before and after every process. Empty stderr, exit0 and exactly one prepared
route line plus the contract/result JSON records are mandatory. First failures,
partial timeout logs and the failed manifest remain available; nothing retries,
relaxes bounds or overwrites a previous output directory.

Across routes the driver requires identical source-output hashes and semantic
population/birth/death/clock ledgers. Selected-route numeric hashes may differ
within the client-checked Graphics bound; same-route repeated runs must remain
deterministic. This is not a blanket checksum tolerance. Raw stdout/stderr and
hashes retain the full per-frame ledger, actual route work, durations and query
costs. The summary includes five paired median and p95 advance ratios and
separate query ratios. It reports every individual regression. The fastest CPU
comparator is chosen explicitly from admitted Interpreter/scalar/SIMD requests
using the median of five per-process medians, then held fixed for all five pairs;
its observed route is recorded, including scalar tails. That post-hoc comparator
selection is disclosed. No total-frame or cross-engine claim follows from this
standalone benchmark.
