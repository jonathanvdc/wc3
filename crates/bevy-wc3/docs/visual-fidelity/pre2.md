# Quad particles (PRE2)

[Documentation index](../README.md)

PRE2 emitters render textured, GPU-instanced head and tail quads for each model
instance. CPU simulation schedules births and stores immutable spawn records;
the vertex shader evaluates motion, lifetime color/alpha/size curves, atlas UVs,
and geometry independently for each view.

## Emission and motion

Continuous emission distributes births within each update. Spawn parameters and
node transforms are sampled at those subframe times, while emission rate and
visibility use the update endpoint. Squirt emission observes the current emission
key. Playback pause and speed control the emitter simulation clock.

World-space velocity uses the emitter’s full affine transform, with multiplicative
speed variation. Gravity is scaled by the emitter’s Z-axis length at birth.
ModelSpace transforms live local motion through the current node transform.

## Geometry, textures, and lighting

Billboard heads face the current view; tails follow particle velocity. Head sizes
and tail widths retain XYZ scale sampled at birth and apply it componentwise in
world space after orientation. Tail velocity already contains emitter scale.
ModelSpace retains birth scale for quad dimensions while moving centers and tails
with the current node. XYQuad heads stay in world XY, retaining an initial
XY-velocity facing angle; stationary and vertical particles still form full quads.

Unshaded particles use texture and segment color directly. Shaded particles use
Bevy scene lighting with a matte, zero-reflectance material. Billboard heads and
tails use a camera-facing normal; XYQuad heads use world +Z. Lighting preserves
alpha. Bitmap and replaceable textures use per-instance bindings; a zero
replaceable ID resolves TextureID through the bitmap slot.

PriorityPlane biases Bevy transparent sorting. SortPrimsFarZ orders particles
by analytic center depth along each camera’s forward axis.

## Record storage and GPU uploads

Spawn records use a growing ring, with chronological retirement and no free-slot
list. Capacity starts at 16 records and doubles with the live population, up to
8,192 records. Render extraction shares immutable blocks of up to 64 records,
so births copy only touched blocks. GPU preparation tracks the last uploaded
birth cursor and writes at most two contiguous ranges across wraparound. Growth
or a full ring lap between rendered frames uploads the entire current buffer.
Retirement alone does not upload spawn records. Per-view depth sorting still
uses a separate draw-order index buffer.

## Implementation limits

The shader applies no fog. Replaceable IDs require application-supplied textures.
Endpoint emission-rate/visibility sampling can miss changes within an update;
squirt updates crossing multiple keys or loops can miss bursts. A dedicated WC3
ordering scheme across model materials and effects is not implemented.
Quad size uses transformed axis lengths, so mirrored scale signs are discarded.

## Capture checks and game comparison

Capture the checked-in fixture from the repository root:

```sh
cargo run -p bevy-wc3 --example capture -- \
  crates/bevy-wc3/tests/fixtures/particle_capture.mdl /tmp/wc3-particle-captures \
  --times 0,0.5,1,2 --fps 60 --size 640x480 \
  --eye 0,-18,8 --target 0,0,2
```

Inspect placement, motion, color/alpha, atlas frames, head/tail geometry, and
blending at the same camera and simulation FPS. This command is a reproducible
check, not a report of new captures. Warcraft comparisons are still needed for
lighting, ordering, speed variation, rotation/scale order, ModelSpace gravity,
XYQuad facing offsets, and mirrored emitters.
