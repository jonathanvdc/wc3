# Animation and pose transitions

`Playback` selects typed clips and controller programmes with independent
clip, authoring-timeline, and shared clocks in seconds. Native adapters preserve
loop/clamp policy, aliases, and sequence intervals; the runtime samples properties
through the semantic `AnimationProgram`.

## Playback controls

Choose a `ClipId` from `PreparedModel::clips()`. `play` starts or restarts
it using the selected M3 sequence's authored blend duration or the model's default.
`play_with_blend` accepts a `Duration`, and `play_immediately` requests no fade.
Repeated play calls restart the clock, so
issue them when the desired animation changes. Invalid selections become
`PlaybackError` during evaluation.

`controllers` defaults to `ControllerSelection::Authored`: selecting an
M3 clip evaluates its authored controller selection in source order. Use
`ControllerSelection::Explicit(programmes)` to replace that list;
`Explicit(Vec::new())` deliberately selects no controllers. These choices apply to
node poses, materials, cameras, effect births, events, and offline pose baking.
After model replacement, authored selections come from the replacement clip;
explicit scoped controller identities are validated and invalid entries are removed.

M2 incoming/outgoing duration pairs are retained by the semantic clips, but native
arbitration between source and destination timing is not interpreted. Nonzero
pairs produce a diagnostic; use `play_with_blend` to supply an application-selected
transition duration.

Set `paused` to suspend clock advancement and `speed` to change its rate.
`seek(seconds)` validates the value, cancels the transition, and skips crossed
events. `restart` resets instance clocks and includes start events again. Direct
clock-field changes retain normal forward traversal; use `seek` when skipped
notifications are intended. Issue whole-step playback changes before
`Wc3Systems::AdvanceAnimation` in `Update`.

## Local-pose blending

A transition freezes the latest evaluated authored local poses before inheritance,
billboarding, and camera anchoring. Translation and signed scale interpolate
linearly; rotations use normalized shortest-path slerp. The advancing destination
uses authored rest values for absent tracks. Interrupted transitions start from the
latest evaluated blended pose, so a new request does not snap back to an older
source. Before the first evaluated pose there is no source to blend.

Fade progress pauses with playback. Negative speed reverses destination sampling
without reversing fade progress. Materials, visibility, light properties, emission
controls, and lenses sample destination values directly rather than crossfading.

## Events and effects

Visible nodes, effect births, and event occurrence poses use the same
transition-aware sampler. Within-update sampling rewinds destination time and fade
progress to the occurrence. Only destination programmes emit events. Repeating
shared clocks follow their own authored periods. Playback methods reset the shared clock along
with other clocks, while direct fields can sample them independently.

Frozen shared clocks retain their authored time-zero values and do not traverse
event windows. Native zero-duration M2 clocks use this explicit semantic state.

Effects retain their own lifetime policies across changes. Model particles may
survive blended transitions, atlas particles retain live births, and ribbons break
connectivity across programme changes. Their focused guides describe cleanup and
bounded simulation behavior. Offline pose baking samples explicit clock state
without live transition history.
