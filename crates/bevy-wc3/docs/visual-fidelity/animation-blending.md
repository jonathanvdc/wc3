# Animation blending

[Documentation index](../README.md)

## Evaluation and rendering

`Wc3Animation::play(sequence)` starts or restarts the selected sequence and blends
from the latest evaluated authored local pose using the model's `BlendTime` in
milliseconds. `play_with_blend(sequence, Duration)` overrides the duration;
`play_immediately(sequence)` and a zero duration switch immediately. A missing
model-info record or zero BlendTime disables the default fade. Before the first
pose evaluation, playback switches immediately rather than inventing a source pose.
Invalid sequence indices leave the player unchanged.

## Pose transitions

Each instance caches its evaluated local node transforms before inheritance,
billboarding, or camera anchoring. A transition freezes that pose while the
destination sequence advances from its beginning. Translation and signed scale
use linear interpolation; rotation uses normalized shortest-path quaternion
slerp. The blended local pose then passes through the existing hierarchy and
node-flag evaluator. Missing destination tracks use the usual neutral defaults.

Interrupted transitions start from the latest evaluated blended local pose.
Multiple play requests between evaluations share that source; the last valid
request wins. Calling play on the current sequence restarts it, so applications
should call play when their desired animation changes, not every frame.

Transition progress pauses with `playing = false` and advances with positive
playback speed. Negative speed reverses the destination clock without reversing
the fade. `seek` and `restart` cancel the transition and invalidate the cached
pose until the next evaluation. Nonfinite seeks leave all state unchanged.
Calls intended for an entire simulation step should precede
`Wc3Systems::AdvanceAnimation`; source poses reflect the latest node evaluation,
not unevaluated changes to clocks or external transforms.

## Events and effects

Only the destination sequence dispatches events, including its start-time keys
once. Global tracks retain the existing shared destination clock and restart
on play; there is no independent persistent global clock. Node tracks on that
clock participate in the local-pose fade. Material, geoset color/alpha, visibility,
emission controls, lights, and model-camera properties sample the destination
directly; they do not crossfade.

Visible nodes, PREM/PRE2 particle births, ribbon births, and event occurrence
poses share the same transition-aware node sampler. Within-update samples rewind
both destination time and fade progress to the birth/occurrence time. The source
pose is retained after fade completion so earlier occurrences in a large update
still sample correctly; memory remains bounded to one frozen source and one
current pose, shared by sampling snapshots.

Blended sequence changes retain live PREM model particles; immediate changes and
backward seeks retain their existing cleanup behavior. PRE2 particles already
survive sequence changes. Ribbon sections also survive, but sequence changes
break ribbon connectivity and reset the emission phase. Attachment child models
retain their independent clocks and existing visibility/restart policy.

## Verification and game comparison

The renderer implements frozen-source transitions into an advancing destination.
Exact Warcraft III transition semantics remain unverified; game captures are
needed to distinguish this policy from moving-source crossfades or runtime lead-in.

An audit of all 85 model files under `data/` found 77 BlendTime values of 150 ms
and eight of 500 ms. Among 511 adjacent sequence interval pairs, 364 nonoverlapping
pairs had gaps smaller than BlendTime and five pairs overlapped; none had gaps
equal to BlendTime. No nonglobal transform keys occupied unused gaps within
BlendTime before the next interval. The only negative keys were five PRE2
visibility keys in a Classic Far Seer model, not skeletal transform keys. These
file counts are not deduplicated and do not analyze pre-roll before the first
sequence. They argue against required authored per-sequence lead-in buffers,
without proving runtime transition semantics. Negative keys retain their current
codec and sampler behavior.

Unit checks cover endpoints, shortest-path rotation, duration overrides, pauses,
speed, interruption, immediate switching, seek/restart cancellation, hierarchy
placement, birth sampling, event ownership, and particle retention.

A complete temporary MDL derived from `tests/fixtures/geoset_capture.mdl` uses two
colored skinned quads and a 1000 ms BlendTime. The source translation is zero;
the destination advances from x=2 to x=4 in one second. Captures at 60 FPS and
320x240, with eye `(2, 0, 12)` and target `(2, 0, 0)`, were inspected for:

- Switching at 0.5 seconds: source pose at the switch, blended translation x=1.5
  halfway through, and destination translation x=4 at completion.
- The same schedule with zero duration: immediate destination translation x=2
  at the switch, followed by the advancing destination.
- Interruption back to the source at 1 second: continuity at x=1.5, a return
  toward zero at 1.25 seconds, and source placement at 2 seconds.

GPU captures verified placement, quad shape, color, and continuity in the
controlled case. This fixture does not visually verify rotational blending or effect trails;
those are covered by CPU tests only. Artifacts are under
`/tmp/wc3-animation-blending/{default,immediate,interrupted}`.

The textured `data/hive-workshop-models/Murloc Xalatath/murkatath.mdx` was also
captured at 60 FPS and 320x240 with eye `(220, -440, 220)` and target `(0, 0, 100)`.
Stand (sequence 0) switches to Walk (6) at 0.5 seconds, then Attack (8) at
1 second, using the model's 150 ms BlendTime. Start, midpoint, and completion
captures showed coherent intermediate limb poses and intact skinning/textures.
They are visual smoke checks rather than a numerical or Warcraft-game reference
comparison. Artifacts are under `/tmp/wc3-animation-blending/murloc-framed`.

## Capturing transitions

The existing `capture` example accepts repeated `--play SECONDS=INDEX` options
in strictly increasing time order. It splits simulation steps at switch times;
`--blend-ms MILLISECONDS` optionally overrides every scheduled switch's duration.
Initial `--sequence` selection is immediate. Capture times remain elapsed
simulation seconds from the initial sequence start, even after switches.

```sh
cargo run -p bevy-wc3 --example capture -- \
  /tmp/wc3-animation-blending/blend.mdl /tmp/wc3-animation-blending/default \
  --play 0.5=1 --times 0.5,0.75,1,1.5 --fps 60 --size 320x240 \
  --eye 2,0,12 --target 2,0,0
```

Use the same command with `--blend-ms 0` for an immediate-switch comparison, or
add `--play 1=0` to inspect interruption continuity. GPU captures verify this
renderer's controlled behavior; matching Warcraft III remains unverified.
