# Completed immutable-effect frames

This original workload compares SIMD CPU appearance with immutable shader
appearance through the maintained `capture` example. It uses 1,000 live child
particles and one controller per feature, with either one or 32 independent
features sharing one native programme and texture. The source has no external
asset dependency. Generate into a fresh temporary directory from the workspace:

```sh
python3 crates/bevy-wc3/benches/support/immutable_frames/generate.py \
  --repo "$PWD" --output /tmp/wc3-immutable-frame-assets
cargo build --release -p bevy-wc3 --features popcorn-jit --example capture
python3 crates/bevy-wc3/benches/support/immutable_frames/compare.py \
  target/release/examples/capture /tmp/wc3-immutable-frame-assets \
  /tmp/wc3-immutable-frame-results --build-label release-feature-enabled --execute
```

Omit `--execute` to inspect the plan and hashes. Keep builds and other GPU work
idle during measurement. The driver runs five alternating pairs per case, twenty
processes in total. The binary, authoring inputs, assets and driver remain hash
guarded around every process, and failed jobs retain their output. Use a fresh
asset/result directory for another series; do not overwrite prior evidence.

The original author reuses the checked native fixture writer. Each complete MDL
has consistent object IDs, origin pivots, sequence and bounds. Both models share
`effect.pkb` and an original white RGBA PNG; every feature has independent core
state. The default 1,024-slot pool includes its controller and 1,000 children.
Particles coincide, so both routes retain identical geometry and overdraw; this
is an effect-cost workload rather than a spatial crowd.

Pass `--motion` to the generator for moving Float3 positions, changing authored
axes, rotation, size, color and atlas frames. `--facing 0|2|3|4|5` selects a
supported billboard geometry in that moving control. Use a fresh directory for
each case, then run the same comparison driver and admissions. Moving centers
require CPU position projection and per-view sorting on the immutable route;
the resulting measurements include those costs. These overlapping populations
are performance controls. Use single-particle captures from
`crates/wc3/tests/fixtures/pkb/generate_immutable_motion.py` to inspect alpha,
atlas orientation and individual trajectories without coincident overdraw.

Capture reaches 0.5 seconds naturally, saves its image, warms residency with zero
delta, then measures 64 completed frames at 64 FPS through 1.5 seconds. No birth
or retirement occurs inside that interval. The camera, 160×120 viewport, MSAA1,
lighting and queue-completion policy are fixed by the author. Internal simulation
substeps can exceed one per external frame; actual route counters, rather than
64×population estimates, establish executed work.

Admission requires equal observable core/Bevy clocks, per-feature populations and
budget statistics; every shader sprite must have immutable publication and actual
authorizations without fallback. The CPU route must execute SIMD lanes without
substituting Rust packets. Core clocks are the tool's nine-decimal observations,
not raw-f64 equality. Source entity identity stays stable inside each process;
cross-process comparison retains source multiplicity without matching local IDs.
Inspect the companion images before interpreting rendering equivalence.

Raw frame wall/update/wait CSVs and actual programme/upload receipts are retained.
Queue bytes are distinct from initialized mapped bytes. Premeasurement receipts
include residency, warmup and the requested capture; they are separate from
advancing upload deltas and are not isolated cold-time measurements. Stable birth
rows must not upload during the interval. Unchanged source/order/uniform values
are charged only when an actual write occurs. Component quantiles remain
independent and must not be added.

The driver forms each pair's immutable/CPU quantile ratio, then takes the median
of five paired ratios. Ratios above one remain regressions in the record; the
workload does not assume an improvement. Report both one-feature and 32-feature
cases so amplification does not hide fixed renderer costs. These synchronized
wall samples include CPU submission and busy queue completion, not GPU timestamps
or pipelined throughput. See the [performance guide](../../../docs/performance.md)
for the measurement boundary. Keep hardware, quality, binary/source hashes and
raw-file hashes beside per-run summaries and all paired ratios.

## Moving visual controls

Generate single-particle and population controls with the original motion author,
then capture all five billboard geometries through the maintained `capture`
example:

```sh
python3 crates/wc3/tests/fixtures/pkb/generate_immutable_motion.py \
  --repo "$PWD" --output /tmp/wc3-motion-pkb \
  --capture-output /tmp/wc3-motion-assets --counts 1,128 --facing 0,2,3,4,5
python3 crates/bevy-wc3/benches/support/immutable_frames/check_motion.py \
  target/release/examples/capture /tmp/wc3-motion-assets \
  --output /tmp/wc3-motion-captures --execute
```

Omit `--execute` to inspect the commands and hashes. `--counts` and `--facing`
can select a complete smaller set for preflight; the default requires both
populations and all geometries. Every CPU/immutable pair uses the same requested
times, camera and simulation FPS. The driver retains logs and PNG hashes and
checks published populations, immutable admission, faults and omissions at each
capture. It does not establish pixel equivalence: compare the resulting images
and inspect motion, axes, atlas orientation, color and alpha separately.
