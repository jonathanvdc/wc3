# Rendering verification notes

[Documentation index](README.md)

These notes preserve earlier test and capture reports. They describe the scope
of those checks and do not report new runs. Temporary `/tmp/` artifacts may no
longer exist. The [rendering guides](rendering/renderer.md) describe the current
implementation; controlled fixture captures establish renderer behavior, while
exact Warcraft III appearance remains unverified.

Use `examples/capture.rs` for rendering checks with the same simulation FPS,
capture times, and camera. Inspect the images for placement, motion, color,
alpha, texture frames, and blending. Compilation or image creation alone does
not establish visual correctness.

## Animation blending

[Behavior guide](rendering/animation-blending.md)

### Verification and game comparison

The renderer implements frozen-source transitions into an advancing destination.
Exact Warcraft III transition semantics remain unverified; game captures are
needed to distinguish this policy from moving-source crossfades or runtime lead-in.

An audit of all 85 model files under `data/` found 77 BlendTime values of 150 ms
and eight of 500 ms. Among 511 adjacent sequence interval pairs, 364 nonoverlapping
pairs had gaps smaller than BlendTime and five pairs overlapped; none had gaps
equal to BlendTime. No nonglobal transform keys occupied unused gaps within
BlendTime before the next interval. The only negative keys were five PRE2
visibility keys in a Classic Far Seer model, not skeletal transform keys. These
file counts are not deduplicated and do not analyze pre-roll before the first
sequence. They argue against required authored per-sequence lead-in buffers,
without proving runtime transition semantics. Negative keys retain their current
codec and sampler behavior.

Unit checks cover endpoints, shortest-path rotation, duration overrides, pauses,
speed, interruption, immediate switching, seek/restart cancellation, hierarchy
placement, birth sampling, event ownership, and particle retention.

A complete temporary MDL derived from `tests/fixtures/geoset_capture.mdl` uses two
colored skinned quads and a 1000 ms BlendTime. The source translation is zero;
the destination advances from x=2 to x=4 in one second. Captures at 60 FPS and
320x240, with eye `(2, 0, 12)` and target `(2, 0, 0)`, were inspected for:

- Switching at 0.5 seconds: source pose at the switch, blended translation x=1.5
  halfway through, and destination translation x=4 at completion.
- The same schedule with zero duration: immediate destination translation x=2
  at the switch, followed by the advancing destination.
- Interruption back to the source at 1 second: continuity at x=1.5, a return
  toward zero at 1.25 seconds, and source placement at 2 seconds.

GPU captures verified placement, quad shape, color, and continuity in the
controlled case. This fixture does not visually verify rotational blending or effect trails;
those are covered by CPU tests only. Artifacts are under
`/tmp/wc3-animation-blending/{default,immediate,interrupted}`.

The textured `data/hive-workshop-models/Murloc Xalatath/murkatath.mdx` was also
captured at 60 FPS and 320x240 with eye `(220, -440, 220)` and target `(0, 0, 100)`.
Stand (sequence 0) switches to Walk (6) at 0.5 seconds, then Attack (8) at
1 second, using the model's 150 ms BlendTime. Start, midpoint, and completion
captures showed coherent intermediate limb poses and intact skinning/textures.
They are visual smoke checks rather than a numerical or Warcraft-game reference
comparison. Artifacts are under `/tmp/wc3-animation-blending/murloc-framed`.

## Node flags

[Behavior guide](rendering/node-flags.md)

### Verification

Private unit tests cover inheritance with animated parents and transformed model
roots; pivots and ordinary descendants; seeking and pause stability; full/axis-locked
billboards; mixed inheritance/lock flags; camera anchoring; selection priorities;
inactive/removed/ambiguous cameras; attached overrides; subframe birth sampling;
mirrored/zero scale; hierarchy cycles; and MDL-to-rig flag preservation.

Temporary complete MDL fixtures under `/tmp/wc3-node-flags` use asymmetric
four-color textures. The existing `capture` example was used at 60 FPS, with
explicit cameras:

- Five panels (ordinary, full billboard, X/Y/Z locks), captured at 0 and 1 seconds
  from `(10,-7,5)` and `(-10,-7,5)`, targeting `(0,0,1)`. Inspected images show the
  full billboard remains camera-aligned; constrained panels retain their axes and
  change texture orientation with the camera. Paused/static images are stable.
- Two skinned panels under a translating/rotating/scaling helper, captured at
  0, 0.5, and 1 seconds. The ordinary panel changes position, orientation, and
  size; the flagged panel suppresses translation, orientation, and size inheritance
  while its pivot offset still follows the parent's rotation/scale. The endpoint
  camera causes partial overlap, so unit tests establish the separate positions.
- One fully billboarded, camera-anchored panel, captured at 0 and 1 seconds with
  depth/normal/motion prepasses. Translating camera and target together by 10 on
  world X produces pixel-identical PNGs, which were also visually inspected. The
  panel remains centered and keeps the same orientation/size. This checks the
  renderer's chosen anchoring behavior, not Warcraft's semantics.

The local `data/` corpus includes GeneralAuraTarget and ShadowStrike with Z-locked
bones, ReviveNightElf with seven full-billboard bones, and RoarTarget with a
full-billboard bone. These identify real asset cases for comparison; presence of
flags and successful loading alone do not establish game fidelity.

GeneralAuraTarget's exact MDX was also copied into the temporary asset root and
captured at 0.25 and 1 seconds, with `(180,-240,140)` targeting the origin. The
original glow texture is absent from this corpus, so bitmap 0 was overridden with
the asymmetric diagnostic texture. Inspected images show the Z-locked skinned
panel and animated scale without pipeline/readback failures. This verifies real
flagged-model integration, not the original aura appearance or Warcraft fidelity.

## Geoset animation

[Behavior guide](rendering/geoset-animation.md)

### Verification

Missing-key sampling uses the explicit neutral fallback described above.

The checked-in `tests/fixtures/geoset_capture.mdl` has two quads sharing one
material: a static red geoset and a blue-to-green geoset that fades to zero.
Run from the repository root:

```sh
cargo run -p bevy-wc3 --example capture -- \
  crates/bevy-wc3/tests/fixtures/geoset_capture.mdl /tmp/wc3-geoset-captures \
  --times 0,0.5,0.75,1 --fps 60 --eye 0,0,8 --target 0,0,0
```

GPU captures inspected at these times verify channel order, cyan midpoint,
partial fade, disappearance, and stable red geometry. Temporary variants reuse
this fixture with Transparent, Blend, Additive, and an extra AddAlpha material
layer, at the same camera and 60 FPS. They verify color/opacity propagation,
including additive fades with preserved source alpha.

Scarlet_footman.mdx under `data/hive-workshop-models/Definitive edition Scarlet
Footman AoW/Definitive edition Scarlet Footman AoW/` was captured at 0.25 and
1 second, sequence 0, 60 FPS. The visible textured mesh remains intact; several
external textures are unavailable, so this is a loading/rendering smoke check,
not a game-color fidelity check. Exact Warcraft appearance remains unverified.

Unit tests cover interpolation, sequence switches, global clocks, missing-key
fallback, disabled color, shared-material isolation, independent instances,
multiple layers, and alpha multiplication. Codec tests check the fixed BGR wire
bytes independently of round trips. Optional corpus round-trip checks encounter
a pre-existing byte mismatch on VashjHighborn.mdx.

## Reforged materials

[Behavior guide](rendering/reforged-materials.md)

### Verification

Unit tests exercise UV-set selection and mirrored tangent handedness, authored
tangent preservation, missing-coordinate errors, slot animation/seeking, global
emissive animation, Fresnel sampling, independent instances, overrides, and
private linear image reuse. The existing geoset and instance tests remain in
place.

The checked-in `tests/fixtures/hd_capture.mdl` has two tinted, skinned quads with
asymmetric normal/ORM textures, animated emission/UV translation, and geoset
alpha. Capture with:

```sh
cargo run -p bevy-wc3 --example capture -- \
  crates/bevy-wc3/tests/fixtures/hd_capture.mdl /tmp/wc3-hd-captures \
  --times 0,0.5,1 --fps 60 --eye 0,-3,5 --target 0,0,0 \
  --prepasses --bevy-reference
```

GPU captures at those times were inspected: normal/material boundaries move
with UV translation, emission changes, the second geoset disappears at its
alpha endpoint, and an ordinary Bevy StandardMaterial sphere remains intact.
Depth/normal/motion prepasses and shadow pipelines compile with HD materials.
This does not independently establish the numerical accuracy of every prepass
attachment or shadow silhouette. The same camera/FPS with the Classic geoset
fixture checks the SD path. Footman captures at 0.25 and 1 second verify real
HD mesh loading and rendering, with several missing external texture warnings;
they are not game-fidelity evidence.

## Model lights

[Behavior guide](rendering/lights.md)

### Verification

Unit tests cover animated color/range, global intensity clocks, visibility and
backward seeking, independent instances, parent position/rotation, inherited
visibility, owner cleanup, runtime settings, shadow gates, invalid values,
preservation of consumer radius, and unchanged scene ambient light.

The complete `tests/fixtures/light_capture.mdl` isolates a moving, colored point
light above an HD surface. Its other lights exercise directional/ambient import.
Use the existing capture example:

```sh
cargo run -p bevy-wc3 --example capture -- \
  crates/bevy-wc3/tests/fixtures/light_capture.mdl /tmp/wc3-light-captures \
  --times 0,0.5,1 --fps 60 --eye 0,-7,7 --target 0,1,0 \
  --no-default-light --bevy-reference --prepasses
```

`--no-default-light` omits the example's directional light, leaving scene ambient
unchanged. The offscreen camera supplies a Bevy shadow LOD origin.
GPU captures at 0, 0.5, and 1 seconds were inspected: the point light changes
from blue to red, moves across the surface, changes the ordinary Bevy sphere's
highlight, and stops contributing at the hidden endpoint. These verify Bevy
scene integration, not exact Warcraft appearance.

A temporary variant of the same complete fixture disables the point light and
rotates a white directional light from +Z through +X to -Z. At the same camera,
FPS, and times, inspected GPU images show an illuminated surface at the start,
a side-lit sphere at the midpoint, and ambient-only top-surface illumination at
the endpoint. Point and directional shadow/prepass pipelines complete without
render errors. Exact shadow silhouettes, biases, and game orientation remain
unverified.

Scarlet Footman's HD portrait was also captured at 0.25 and 1 seconds in sequence
0, with the default capture light disabled, an explicit camera framing the head,
and available diffuse/body normal/ORM DDS maps supplied through bitmap overrides.
The model renders with its imported light records; the result is relatively dim
at the default conversion scales. Missing helmet/background/game texture maps
prevent a complete appearance comparison. These captures exercise real portrait
asset integration and do not calibrate power or establish Reforged fidelity.

## Classic PREM particles

[Behavior guide](rendering/prem.md)

### Capture checks

Capture the fixture with the existing offscreen example:

```sh
cargo run -p bevy-wc3 --example capture -- \
  crates/bevy-wc3/tests/fixtures/prem_capture.mdl /tmp/wc3-prem-captures \
  --times 0,0.5,1,1.5,2 --fps 60 --size 640x480 \
  --eye 0,-18,9 --target 0,0,2
```

The fixture reuses `attachment_capture_child.mdl` and `capture_white.png`. It
isolates a moving emitter with visibility keys and a nonuniformly scaled cone.
GPU captures at the listed times were visually inspected for detached birth
positions, motion, model scale, and independently animated geometry. The current
`data/` corpus has no populated PREM chunks, so it cannot establish game-model
PREM fidelity.

## Ribbon emitters

[Behavior guide](rendering/ribbons.md)

### Capture checks

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

## Quad particles (PRE2)

[Behavior guide](rendering/pre2.md)

### Capture checks and game comparison

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

## Model cameras

[Behavior guide](rendering/cameras.md)

Unit and public integration tests cover authored discovery, independent instance
clocks, sequence/global sampling, roll across view directions, transformed roots,
camera-parent compensation, invalid input recovery, binding removal, and
same-frame billboard/projection scheduling.

The shared capture example supports `--model-camera INDEX` and
`--camera-fov-multiplier NUMBER`. Model-camera selection excludes `--eye` and
`--target`; invalid indices/lenses fail before GPU initialization and invalid
sampled views fail before writing a capture. Exact Warcraft III framing, roll
sign, and Reforged lens fidelity remain unverified.

GPU captures at 640x480 and 60 FPS verified a controlled two-panel MDL with static
geometry/color: additive eye/target translation and 0/45/90-degree roll at
0/0.5/1 seconds, with depth/normal/motion prepasses enabled. A second authored
camera with FOV multiplier 0.75 increased panel size at the same pose. A near
distance of 11 clipped panels 10 units from the eye. These images were inspected
for placement, orientation, size, and clipping, rather than just file creation.

Scarlet Footman's authored portrait camera also rendered at 0 and 0.5 seconds
with multiplier 0.75, producing a stable face close-up. Several normal/ORM,
environment, and background textures were unavailable, so this check establishes
camera integration, not complete material fidelity or game-equivalent framing.

## Model event objects

[Behavior guide](rendering/events.md)

Tests cover sorted MDX/MDL decoding and replacement, duplicates, interval
partitioning, multiple loops, sequence/global boundaries, missing and
zero-duration clocks, sequence switching, same-sequence restart, seeking,
pause/reverse playback, instance isolation, scheduling, and sampled node poses.
These establish the dispatch contract, not exact Warcraft III trigger semantics
or visual equivalence. No visual effect is rendered by dispatch, so no new
rendering capture is claimed.

## Texture extension fallback

[Texture selection and lookup](usage.md#choose-textures)

Asset unit tests cover literal-first extension ordering, model-directory precedence
over the asset root, duplicate removal, uppercase reference extensions, empty and
unsafe paths, and unchanged child-model candidate generation. A file-backed Bevy
loader test decodes a one-pixel TGA for missing `.tif` references at both search
locations and verifies that an exact local filename beats an alternate extension.
These checks establish asset selection and decoding; no GPU capture or Warcraft III
visual fidelity comparison is claimed.
