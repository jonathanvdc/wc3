# Model event objects

Model event objects mark animation times at which an application may play
sounds, spawn effects, or perform other game actions. The renderer publishes
these occurrences as `Wc3ModelEvent` messages with sampled poses. This guide
explains how to consume them and how playback, loops, and seeking affect dispatch.

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

Dispatch follows forward clock traversal rather than sampling only the current
frame. Every key crossed in `(previous, current]` is emitted, including multiple
keys or loops in one update. Model keys use the selected sequence's inclusive
interval; global keys use `0..=duration`. Constructors, MDX/MDL readers, and
`set_frames` sort timestamps, and `frames()` exposes a read-only slice. Duplicate
keys remain separate events and serialization emits them in sorted order.

Initial playback and `Wc3Animation::play` include start-time events once.
Playing the current sequence explicitly restarts it; invalid indices leave
playback unchanged. `restart()` restarts the current clock, including global
tracks in models without animation sequences. A zero-duration clock fires its
zero-offset keys once at playback start. Invalid global IDs and reversed sequence
intervals dispatch nothing.

Use `sequence()` and `elapsed_ms()` to inspect playback state. `seek(ms)` moves
the clock without emitting skipped keys or keys at the destination. Later forward
advancement still emits crossed keys, including advancement before the next
dispatch. Nonfinite seeks are rejected. Paused and reverse playback emit no events;
resuming forward playback traverses from the most recently observed position.
Start events remain pending while paused at their initial position.

Non-looping model keys fire once per playback. Global tracks share elapsed time
with the instance, pause and reverse with it, and reset when `play` restarts it.
They continue looping after a non-looping model sequence ends and work without
a model sequence. A loop-end key and the next loop-start key are distinct
notifications at the same elapsed time. Their own clocks sample their respective
authored end/start frames, preserving distinct poses at the boundary; other clocks
sample the occurrence's elapsed time normally.

## Occurrence poses

Occurrence poses reuse CPU node evaluation, including hierarchy, pivots,
inheritance flags, and the currently selected driving camera. Model animation is
sampled at each key rather than at the update endpoint. Application-owned root
and ancestor transforms and driving-camera inputs use current update values;
their movement history is not reconstructed.
