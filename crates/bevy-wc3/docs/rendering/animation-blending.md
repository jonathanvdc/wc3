# Animation blending

[Documentation index](../README.md) · [Verification notes](../verification.md)

## Playback controls

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
backward seeks retain their existing cleanup behavior. PRE2 particles
survive sequence changes. Ribbon sections also survive, but sequence changes
break ribbon connectivity and reset the emission phase. Attachment child models
retain their independent clocks and existing visibility/restart policy.

## Scheduled playback in captures

The existing `capture` example accepts repeated `--play SECONDS=INDEX` options
in strictly increasing time order. It splits simulation steps at switch times;
`--blend-ms MILLISECONDS` overrides the duration of scheduled switches. Initial
`--sequence` selection is immediate. Capture times are elapsed simulation seconds
from the initial start, including across sequence changes.

```sh
cargo run -p bevy-wc3 --example capture -- path/to/model.mdx /tmp/wc3-transitions \
  --sequence 0 --play 0.5=1 --times 0.5,0.75,1,1.5 --fps 60
```

Select indices available in the model. Add `--blend-ms 0` to switch immediately,
or `--play 1=0` to inspect an interrupted transition.
