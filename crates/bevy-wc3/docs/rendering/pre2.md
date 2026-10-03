# Quad particles (PRE2)

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

## Implementation limits

Each emitter is limited to 8,192 live spawn records. Storage grows with the
population; [architecture](../architecture.md#effect-record-storage) describes
record extraction and GPU uploads for contributors.

The shader applies no fog. Replaceable IDs require application-supplied textures.
Endpoint emission-rate/visibility sampling can miss changes within an update;
squirt updates crossing multiple keys or loops can miss bursts. A dedicated WC3
ordering scheme across model materials and effects is not implemented.
Quad size uses transformed axis lengths, so mirrored scale signs are discarded.
