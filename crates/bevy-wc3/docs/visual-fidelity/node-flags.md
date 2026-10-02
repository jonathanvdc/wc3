# Node flags

[Documentation index](../README.md)

## CPU pose evaluation and camera selection

All node types retain their transform flags when the rig is spawned. The CPU
pose evaluator resolves parents before children, samples authored animation,
and writes compensated local `Transform`s before Bevy transform propagation.
The ordinary hierarchy remains responsible for visibility, skinning, attachments,
lights, and cleanup. Node flags do not require a special material shader.

`Wc3NodeCamera(camera_entity)` on a model animation root explicitly selects an
active `Camera3d`. Attached model roots inherit the nearest ancestor selection
unless they supply their own component. An invalid or inactive explicit camera
skips camera-dependent adjustments rather than silently selecting another camera.

Without an explicit selection, the evaluator chooses:

1. The sole active `Camera3d` marked `Wc3DefaultNodeCamera`.
2. Otherwise, the sole active window-target `Camera3d`.
3. If no active window cameras exist, the sole active `Camera3d`, including an
   offscreen capture camera.

Multiple cameras at the selected priority are ambiguous. Camera2d/UI cameras are
excluded. Ambiguous or missing selections warn once per affected model/selection;
ordinary animation and inheritance flags continue to work. Selection is refreshed
at runtime, including camera removal, activation changes, and component overrides.
A separate portrait model instance should explicitly select its portrait camera
when a viewport camera is also present.

```rust
use bevy::prelude::*;
use bevy_wc3::{Wc3DefaultNodeCamera, Wc3NodeCamera};

// Mark the application's main viewport camera when automatic selection is ambiguous.
commands.entity(viewport_camera).insert(Wc3DefaultNodeCamera);
// A preview instance can select its own offscreen camera.
commands.entity(preview_model).insert(Wc3NodeCamera(preview_camera));
```

Evaluation runs in `PostUpdate`, in `Wc3Systems::EvaluateNodePoses`, before Bevy's
`TransformSystems::Propagate` and PREM simulation. Camera/root movement performed
in `Update` is visible in the same frame. Applications moving these entities in
`PostUpdate` should order those systems before `Wc3Systems::EvaluateNodePoses`.

## Flag evaluation

| Flag | Current behavior |
| --- | --- |
| DontInheritTranslation | Removes accumulated skeletal translation-track displacement from the node's position. Rest-pivot offsets, parent rotation/scale acting on those offsets, the node's own translation, and model-instance placement remain. Evaluation is independent of the previously rendered pose. |
| DontInheritRotation | Compensates inherited skeletal rotation while retaining instance rotation and the node's authored rotation. Axis locks evaluate against this effective orientation. Full billboarding already cancels parent rotation. |
| DontInheritScaling | Compensates accumulated skeletal scale with signed componentwise ratios while retaining instance scale and authored local scale. Compensation operates on TRS components and does not generally remove affine shear. |
| Billboarded | Cancels parent rotation, applies camera world rotation and the MDX-facing basis conversion, then authored rotation. The node's pivot location remains unchanged. |
| BillboardedLockX/Y/Z | Projects the camera-facing direction into the authored/effective node frame and rotates about the selected local axis. X lock also flips local Z scale. Full billboard takes precedence over locks; otherwise X, Y, Z is the precedence. A direction parallel to the locked axis leaves the authored orientation. |
| CameraAnchored | Adds camera world translation to the hierarchy while retaining authored placement. Descendants inherit the offset; a second anchored node applies only the difference from the inherited offset, avoiding double translation. It does not itself rotate the node with the camera. |

PREM, PRE2, and ribbons use the same evaluator at subframe birth times. Authored
node tracks are sampled at birth; the selected camera pose is held at the current
update's pose, rather than reconstructed historically. Existing world-space
particles/trails retain their birth positions. Model-space effects follow the
current resolved node. Attached roots retain their ancestor camera offset during
birth evaluation.

Pose composition retains affine matrices and separately tracks signed scale and
rotation. Zero parent scale cannot be inverted exactly: compensated collapsed
axes remain collapsed, and finite fallbacks avoid infinities. Nonuniform scale
combined with rotated descendants can still produce shear.

## Camera constraints and game comparison

The renderer's node-flag semantics are explicit above; equivalence to Warcraft III
remains unverified. Full billboarding retains authored rotation after camera
alignment, and scaling compensation retains model-instance scale. Camera anchoring
and translation suppression are deterministic renderer behavior with unverified
game semantics.

Game comparisons should include combinations of flags, authored rotation, camera
roll, off-origin pivots, mirrored/nonuniform scaling, and camera motion during
emission.

One driving camera is used per model instance. Rendering the same instance from
multiple cameras uses that same resolved skeleton in every view. Per-view skeletal
poses are not implemented. Separate portrait/preview instances can select different
cameras. Shadows also consume the driving-camera pose. Camera-dependent geometry
can exceed authored mesh/model bounds; conservative bounds remain the application's
responsibility when culling would otherwise hide it.

## Verification

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
