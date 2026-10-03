# Ribbon trails (RIBB)

RIBB emitters render for both MDX and MDL model instances. The CPU schedules
births at the fixed emission rate, samples the animated node hierarchy and
HeightAbove/HeightBelow at each birth, and retains world-space cross-sections.
Height extends along local Y; the Bevy node transform already incorporates its
pivot. Moving or rotating the emitter later does not move existing sections.

## GPU geometry and storage

The GPU stores immutable endpoints and split birth timestamps. Each instanced
quad references two adjacent live sections. The vertex shader constructs the
quad, evaluates ballistic motion independently at each endpoint, derives its
normal, and maps one texture-atlas cell across each connected live chain. Only
birth ranges are uploaded, using at most two contiguous writes across ring
wraparound. Storage starts at 16 records and doubles with the live population,
up to 8,192 records. Growth rebases live sections and uploads the resized buffer.
Immutable extraction snapshots share blocks of up to 64 records; births copy only
touched blocks, rather than the entire ring. GPU cursors account for skipped
frames, requesting a full upload when a whole ring has been overwritten. Material layers share CPU
records and the GPU section buffer; each layer has separate uniforms and texture
bindings. No compute simulation or per-frame CPU mesh rebuilding is required.

## Materials and atlas animation

Color, Alpha, TextureSlot, layer alpha, and layer texture selection update across
the live trail using the current sequence/global-sequence time. Layer texture
translation, rotation, and scaling are applied to atlas UVs, around UV (0.5, 0.5).
Bitmap and replaceable bindings use the existing per-instance texture API.

Every referenced material layer receives a pass in source order. Blend modes,
alpha testing, TwoSided, Unshaded/Unlit, NoDepthTest, and NoDepthSet are respected.
Material PriorityPlane is a signed bias in Bevy's transparent phase. Material
SortPrimsFarZ/SortPrimsNearZ sorts segment centers per view without changing their
endpoint pairs or UV ranks; NearZ takes precedence if both flags are set.

## Playback and lifetime

Pause and playback speed control the effect clock. Visibility stops new births
but lets existing sections age. Sequence changes, backward seeks, and sampled
visibility gaps break connectivity; old sections remain until their lifetime
expires. Non-looping sequences stop emission at their end but existing sections
continue aging. Root visibility hides the effect, and root despawn removes all
emitter state and render entities. As with PRE2, hiding the root does not pause
the simulation. Capacity is limited to 8,192 live sections per emitter; excessive
births or large time steps retain only the newest live capacity.

## Gravity

Gravity uses frame-rate-independent world -Z displacement
`0.5 * gravity * age²`, with zero initial falling velocity and no additional
emitter-scale multiplier.
Births are distributed within each update, avoiding coincident sections after a
slow update. Each connected chain occupies exactly one atlas cell using
floating-point UVs.

## Sampling and render-pass limits

- Visibility is sampled at births and update endpoints; a hidden interval entirely
  between those samples can go undetected.
- All ribbon passes use Bevy's transparent phase, including opaque/alpha-tested
  layers with depth writes. Dedicated WC3 ordering across meshes and effects is
  still absent.
- Shaded ribbons use Bevy scene lighting, rather than Classic lighting. Fog,
  environment mapping, and Reforged normal/ORM material slots are not implemented.
- Animated node flags use the shared CPU evaluator and one driving camera per
  instance. Subframe births sample authored tracks with the current camera pose;
  see [node flags](node-flags.md) for camera selection and transform rules.
