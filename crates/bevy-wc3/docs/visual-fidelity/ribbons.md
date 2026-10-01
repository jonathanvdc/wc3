# Ribbon emitters

[Documentation index](../README.md)

RIBB emitters render for both MDX and MDL model instances. The CPU schedules
births at the fixed emission rate, samples the animated node hierarchy and
HeightAbove/HeightBelow at each birth, and retains world-space cross-sections.
Height extends along local Y; the Bevy node transform already incorporates its
pivot. Moving or rotating the emitter later does not move existing sections.

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

Color, Alpha, TextureSlot, layer alpha, and layer texture selection update across
the live trail using the current sequence/global-sequence time. Layer texture
translation, rotation, and scaling are applied to atlas UVs, around UV (0.5, 0.5).
Bitmap and replaceable bindings use the existing per-instance texture API.

Every referenced material layer receives a pass in source order. Blend modes,
alpha testing, TwoSided, Unshaded/Unlit, NoDepthTest, and NoDepthSet are respected.
Material PriorityPlane is a signed bias in Bevy's transparent phase. Material
SortPrimsFarZ/SortPrimsNearZ sorts segment centers per view without changing their
endpoint pairs or UV ranks; NearZ takes precedence if both flags are set.

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
emitter-scale multiplier. Exact Warcraft III ribbon gravity remains unverified.
Births are distributed within each update, avoiding coincident sections after a
slow update. Each connected chain occupies exactly one atlas cell using
floating-point UVs.

## Captures and remaining verification

Capture the checked-in fixture:

```sh
cargo run -p bevy-wc3 --example capture -- \
  crates/bevy-wc3/tests/fixtures/ribbon_capture.mdl /tmp/wc3-ribbon-captures \
  --times 0,0.5,1,2 --fps 60 --size 640x480 \
  --eye 0,-12,12 --target 0,0,1
```

The fixture reuses `capture_white.png` and covers the empty initial frame,
detached birth positions, continuous geometry, and gravity bend.
Additional capture checks cover animated heights, parent rotation, a four-cell
asymmetric atlas, texture-slot changes, and separate chains across visibility
gaps. Captures at 0.5, 1, 1.5, and 2 seconds produce byte-identical PNGs at
30 FPS and 60 FPS. Two-layer checks cover matching geometry across an unshaded
blend pass and a shaded additive pass sharing the GPU section buffer.

Game-model geometry checks cover Winter Cenarius (sequence 8) and Thrall
Shadowlands (sequence 1) from `data/` at 0.15, 0.3, 0.45, and 0.6 seconds.
White bitmap substitutions are required because the shared game textures are
absent. These checks exercise animated parent rigs, three Cenarius emitters,
and Thrall's weapon trail; texture fidelity remains unverified. Those emitters
all have zero gravity, so they do not exercise falling motion.

A non-looping variant ending at 1 second with a 1-second ribbon lifespan covers
expiry at 0.5, 1, 1.5, and 2.25 seconds: the trail shrinks at 1.5 seconds and
is absent at 2.25 seconds. Ribbon and PRE2 draws use transient phase items,
so empty or hidden emitters do not retain their last GPU draw.

Remaining verification and renderer limits:

- Compare gravity, emission density, UV direction, visibility thresholds, and
  material-layer semantics against actual Warcraft III captures.
- Visibility is sampled at births and update endpoints; a hidden interval entirely
  between those samples can go undetected.
- All ribbon passes use Bevy's transparent phase, including opaque/alpha-tested
  layers with depth writes. Dedicated WC3 ordering across meshes and effects is
  still absent.
- Shaded ribbons use Bevy scene lighting, rather than Classic lighting. Fog,
  environment mapping, and Reforged normal/ORM material slots are not implemented.
- Animated node billboard flags retain the general renderer's limitations.
