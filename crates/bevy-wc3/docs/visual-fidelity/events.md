# Model event objects

[Documentation index](../README.md)

## Dispatch and application integration

Event objects dispatch `Wc3ModelEvent` messages during forward playback. The
renderer preserves their names without interpreting prefixes, resolving game
tables, loading sounds, spawning effects, or projecting footprints onto terrain.
Those behaviors belong to the application.

Messages identify the instance root, rig node, object ID, event-record index,
sorted key index, selected sequence, optional global-sequence ID, authored frame,
unwrapped occurrence time, and sampled world transform. Definitions are shared
by prepared-model instances; each instance owns its dispatch cursor.

Read messages in `PostUpdate` after `Wc3Systems::DispatchEvents`, or during the
next `Update`. Ordering within each instance is by occurrence time, event-record
index, then sorted key index. Ordering between instances is unspecified. As with
other buffered Bevy messages, consumers must read regularly before messages age
out; entity IDs can become invalid if their instance is despawned.

```rust
use bevy::prelude::*;
use bevy_wc3::{Wc3ModelEvent, Wc3Systems};

fn handle_model_events(mut events: MessageReader<Wc3ModelEvent>) {
    for event in events.read() {
        // Interpret event.name and use event.transform in application code.
    }
}

// Add this alongside Wc3BevyPlugin:
// app.add_systems(PostUpdate,
//     handle_model_events.after(Wc3Systems::DispatchEvents));
```

## Timing contract

- `EventObject` constructors, MDX/MDL readers, and `set_frames` sort timestamps.
  `frames()` exposes a read-only slice. Duplicate keys remain separate events;
  serialization emits sorted timestamps.
- Traversal emits every crossed key in `(previous, current]`, including multiple
  keys or loops crossed in one update. Model keys use the selected sequence's
  inclusive interval; global keys use `0..=duration`.
- Initial playback and `Wc3Animation::play` include start-time events once.
  Restarting the same sequence is an explicit restart. Invalid sequence indices
  leave playback unchanged. `restart()` restarts the current clock, including
  global tracks in models without animation sequences.
- `sequence()` and `elapsed_ms()` expose read-only playback state.
  `seek(ms)` moves the clock without emitting skipped keys or keys exactly at the
  destination. Forward advancement after the seek still emits crossed keys,
  including advancement before the next dispatch. Nonfinite seeks are rejected.
- Paused and reverse playback emit no events. Resuming forward playback traverses
  from the most recently observed position. Start events remain pending while
  paused at their initial position.
- Non-looping model keys fire once per playback. Global tracks retain the existing
  animation-clock behavior: they share elapsed time, pause and reverse with the
  instance, and reset when `play` restarts it. They continue looping after a
  non-looping model sequence ends and work without a model sequence.
- A zero-duration clock fires its zero-offset keys once at playback start.
  Invalid global IDs and reversed sequence intervals dispatch nothing.
- A loop-end key and the next loop-start key are distinct notifications at the
  same elapsed time. Their own clock's animated tracks sample their respective
  authored end/start frames, preserving distinct poses at the boundary. Other
  clocks sample the occurrence's elapsed time normally.

## Pose sampling and verification limits

Occurrence poses reuse CPU node evaluation, including hierarchy, pivots,
inheritance flags, and the currently selected driving camera. Model animation is
sampled at each key rather than at the update endpoint. Application-owned root
and ancestor transforms and driving-camera inputs use current update values;
their movement history is not reconstructed.

Tests cover sorted MDX/MDL decoding and replacement, duplicates, interval
partitioning, multiple loops, sequence/global boundaries, missing and
zero-duration clocks, sequence switching, same-sequence restart, seeking,
pause/reverse playback, instance isolation, scheduling, and sampled node poses.
These establish the dispatch contract, not exact Warcraft III trigger semantics
or visual equivalence. No visual effect is rendered by dispatch, so no new
rendering capture is claimed.
