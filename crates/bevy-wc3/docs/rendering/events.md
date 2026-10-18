# Semantic model events

`ModelEvent` messages expose authored animation occurrences for gameplay.
The renderer does not execute commands, resolve sound tables, or spawn gameplay
effects. `EventSource` retains named actions, four-byte identifiers with
data IDs as distinct payloads.

## Consume occurrences

Read messages in `PostUpdate` after `Wc3Systems::DispatchEvents`, or in the next
`Update`. Each notification identifies its root, optional source feature/node, name,
payload bytes, activating clip or model-level selection, time source, exact key, unwrapped
occurrence time, and occurrence-time world transform.

```rust
use bevy::prelude::*;
use bevy_wc3::ModelEvent;
fn handle(mut events: MessageReader<ModelEvent>) {
    for event in events.read() {
        // Interpret event.source and event.payload in application code.
    }
}
```

Occurrences belong to reusable `Animation` definitions. Clips and
model-level selections activate those definitions; value sampling itself never
dispatches an event. Definitions are shared; dispatch cursors are private to instances. Ordering within
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

Ordinary events use animation time within the selected clip source interval,
including intervals beginning at zero. Traversal subtracts the source start to
locate each occurrence in playback time; independent shared events follow their
own clock periods.
Shared clocks can continue
while a non-looping clip is clamped at its end.

Each update emits at most 4,096 earliest occurrences per instance. A bounded
chronological merge seeds one occurrence per active stream and only advances streams
that are consumed. `EventDispatchStats` reports emitted and omitted counts
without enumerating every key in a long jump.

## Occurrence poses

The shared pose evaluator samples hierarchy, pivots, inheritance, billboards,
anchors, overrides, and transition progress at the occurrence. Application root and
camera transforms use current update values; their historical motion is not
reconstructed. Occurrences without a hierarchy anchor retain no node identity.
