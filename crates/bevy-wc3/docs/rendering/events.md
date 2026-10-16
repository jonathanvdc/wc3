# Semantic model events

`UniversalModelEvent` messages expose authored animation occurrences for gameplay.
The renderer does not execute commands, resolve sound tables, or spawn gameplay
effects. `UniversalEventSource` retains named actions, four-byte identifiers with
data IDs, and typed engine commands as distinct payloads.

## Consume occurrences

Read messages in `PostUpdate` after `Wc3Systems::DispatchEvents`, or in the next
`Update`. Each notification identifies its root, optional source feature/node, name,
payload bytes, owning clip/controller/global programme, clock, exact key, unwrapped
occurrence time, and occurrence-time world transform.

```rust
use bevy::prelude::*;
use bevy_wc3::UniversalModelEvent;
fn handle(mut events: MessageReader<UniversalModelEvent>) {
    for event in events.read() {
        // Interpret event.source and event.payload in application code.
    }
}
```

Definitions are shared; dispatch cursors are private to instances. Ordering within
one instance follows occurrence time and deterministic definition/key order.
Ordering between instances is unspecified. Consume buffered messages regularly;
entity IDs can become stale after despawn.

## Traversal and budgets

Forward advancement emits crossed keys in `(previous, current]`, including loops and
duplicate keys. Inclusive loop-end and next-loop-start keys remain distinct and
sample their own endpoint poses. Initial playback, `play`, and `restart` include
start keys once. `seek(seconds)` skips its interval and destination keys; subsequent
advancement still dispatches normally. Direct clock edits retain ordinary traversal.
Paused or reversed intervals emit no forward occurrences.

Clip, authoring-timeline, and shared clocks retain their distinct semantics.
Controller selection changes begin newly enabled programmes at current traversal
state rather than replaying unrelated global history. Shared clocks can continue
while a non-looping clip is clamped at its end.

Each update emits at most 4,096 earliest occurrences per instance. A bounded
chronological merge seeds one occurrence per active stream and only advances streams
that are consumed. `UniversalEventDispatchStats` reports emitted and omitted counts
without enumerating every key in a long jump.

## Occurrence poses

The shared pose evaluator samples hierarchy, pivots, inheritance, billboards,
anchors, overrides, and transition progress at the occurrence. Application root and
camera transforms use current update values; their historical motion is not
reconstructed. Commands without a hierarchy anchor retain no node identity.
