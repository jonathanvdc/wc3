# Authoring animation

Animation changes typed values in a model. Start with a checked [model graph](semantic.md),
then allocate values, attach curves, and select those curves through clips or shared
clocks. For pose sampling and geometry deformation, continue with [CPU evaluation](evaluation.md).

## Values and tracks

A `Property<T>` is a typed identity into the model's `PropertyValues` table. Every
potentially animated field owns such a reference, including node translation,
rotation, scale, and visibility. The table stores authored constants. Copying a
reference shares both its constant and its animation; allocating a fresh slot
creates an independent value. Animation never needs to traverse an owner object
to find a field.

`Track<T>` connects a slot to a checked `Curve<T>` and an explicit clock.
`DynamicTrack` wraps typed tracks in an enum whose variants select the value type,
so tracks of different types can share one collection. Both types are independent
of model profiles. A native exporter finds the fields consuming each slot and
writes the corresponding native tracks, duplicating a curve when several fields
share its slot. Exact projection rejects animation that the target
format cannot attach to a supported field. Slot identities carry their value type
and ownership domain, so bindings need no separate owner or field selector.

## Select animations and clocks

`Animation` owns ordered tracks in `Animation::tracks` and identified event
occurrences. Add it with `ModelBuilder::add_animation`, then reference its
`AnimationId` from `Clip::animations` or the model's ordered `global_animations`
selection. Reusing an identity shares its curves and events;
clips retain their own duration and loop/clamp policy. `add_global_track` and
`add_track` append to the selection's first animation, creating one when the selection is empty. Editing that animation affects
every selection of it. Use `add_animation_track` for explicit ownership.

`TimeSource::Animation` evaluates ordinary tracks at the selected clip's
`source_start` plus wrapped or clamped local playback time. Every clip selects an
inclusive source interval whose length is its duration; `source_start` defaults to
zero. Only keys inside that interval influence sampling. Animations keep their
authored timestamps, so overlapping clips can select the same curves without
copying or shifting keys. The MDLX vocabulary requires all clips to select the
same ordered ordinary animations, with shared-clock animations selected at model
level, matching its native timeline. Without a clip, ordinary model-level
tracks use `AnimationSample::clip_time` directly without interval bounds.
`TimeSource::Shared(ClockId)` instead wraps independent shared time by that clock's
period. This distinction expresses independent playback, not a choice between
local and absolute coordinates for ordinary tracks.

`AnimationSample` supplies the selected clip, elapsed clip time, and shared time.
Direct model sampling and compiled `AnimationProgram` sampling use the same curve
evaluator and clip-time mapping. Model-level animations run first, followed by the
clip's animations. Within each selection, animation and track order are
authoritative. Later applicable tracks override earlier ones; empty curves fall
through to earlier tracks and ultimately authored defaults. Value sampling does
not advance clocks or dispatch events; event traversal separately examines the
interval crossed between playback states.

## Construct values and curves

Allocate constants with `PropertyValues::add` or `ModelBuilder::add_value`. Construct
slot-bearing payloads with `Construct::construct(&mut values)` or
`builder.construct()`, which recursively allocates their defaults. `builder.new_node`
and `add_named_node` allocate the node's transform and visibility slots. Changing a
table entry preserves its identity. Model publication validates slot references,
value domains, and resource dependencies. `import_values` preserves identities
when combining independently constructed payloads; conflicting constants are an
error. Widening and narrowing retain existing slot identities and may allocate
additional constants required by the destination payload.

```rust
use wc3::model::{DynamicTrack, TimeSource, Curve, Model, ModelMetadata, Time, Track, Vec3};
use wc3::model::profiles::Universal;
let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
let node = builder.new_node("Root");
let translation = node.transform.translation;
builder.add_node(node)?;
builder.add_global_track(DynamicTrack::Vector3(Track {
    target: translation,
    clock: TimeSource::Animation,
    curve: Curve::linear(
        [Time::seconds(0.)?, Time::seconds(1.)?],
        [Vec3::ZERO, Vec3::X],
    )?,
}))?;
let model = builder.finish()?;
assert_eq!(model.values()[translation], Vec3::ZERO);
# Ok::<(), wc3::model::Error>(())
```

Boolean-to-scalar reads are explicit: `PropertyValues::scalar_from_boolean` allocates a
scalar slot linked to a boolean source, sampled as zero or one. Both identities
remain available, so native conversions can recover the original boolean field.
A direct scalar track on that output takes precedence over its converted read.
This relation belongs to the value table rather than a field-specific binding.

Choose interpolation through `Curve::step`, `linear`, `hermite`, `bezier`, `tcb`,
`slerp`, or `nlerp`. The constructor's trait bounds express the permitted value
capabilities. Boolean and texture-selection values permit stepped curves;
numeric domains permit linear curves; floating-point scalars, vectors, and
quaternions also permit splines. Explicit spherical and normalized linear
interpolation require rotation values. Integer domains retain their exact value
types rather than passing through floating-point key storage. Constructors still
check finite values, ordered times, and matching array lengths. Times are finite
seconds and can precede playback. Coincident keys retain their input order: the
first key defines the incoming limit, while the last defines the boundary value
and outgoing segment.

Each curve stores exactly the controls required by its interpolation. Hermite
curves expose `hermite_derivatives`, Bezier curves expose `bezier_controls`, and
TCB curves expose `tcb_parameters`; the other accessors return `None`. Derivatives
and control points have distinct associated domains. For quaternion keys both
use finite four-component vectors, allowing zero derivatives and component
controls that are not themselves rotations. Hermite derivatives retain the
source curve's segment convention; constructing a semantic curve does not
rescale them to derivatives per second or infer a new quaternion spline convention.
Native adapters check decoded sampling directives against these capabilities
and translate component layouts without changing the interpolation mode.

The array constructors accept separate `PackedArray` arrays for times, values, and
controls through `IntoPackedArray`. Existing encoded arrays move unchanged; native
slices, arrays, and vectors are encoded into owned storage. Typed `ValueKeyframe`,
`HermiteKeyframe`, `BezierKeyframe`, and `TcbKeyframe` records provide a convenient
authoring interface through the corresponding `*_keys` constructors. Those
constructors materialize separate encoded arrays from the interleaved records.
For example, a quaternion Hermite curve can carry zero derivatives explicitly:

```rust
use wc3::model::{Curve, HermiteKeyframe, Quat, Time, Vec4};

let keys = [HermiteKeyframe {
    time: Time::seconds(0.)?,
    value: Quat::IDENTITY,
    incoming: Vec4::ZERO,
    outgoing: Vec4::ZERO,
}];
let curve = Curve::hermite_keys(keys)?;
assert!(curve.values().is_owned());
assert_eq!(curve.hermite_derivatives().unwrap().incoming.get(0), Some(Vec4::ZERO));
Ok::<(), wc3::model::Error>(())
```

## Events

Event sources are independent model-owned definitions. `Profile::EventSource` selects
the complete source type directly: `mdlx::EventSource` owns a name, node anchor,
and clock; `m2::EventSource` owns a four-byte identifier, game-data ID, node anchor,
and clock. Universal's `EventSource::Mdlx` and `EventSource::M2` preserve those
complete source definitions, while WMO uses `Unavailable`. Sources are separate from attached features and remain
present even when they have no occurrences. Declare one with
`ModelBuilder::add_event_source` and inspect it through `Model::event_sources`
or `Model::event_source`. Equal declarations remain independent; reusing an
`EventSourceId` explicitly shares one definition.

`EventOccurrence { source, time }` schedules that action on its source's clock.
Publication validates source links, including unused sources. Widening preserves
identities; narrowing rejects source kinds outside the destination vocabulary.
M2 requires ordinary event occurrences to be selected by
clips and shared-clock occurrences to be selected at model level; publication
rejects the opposite selections because native M2 cannot retain them.
