# Node transforms and flags

Node transform flags control how model parts inherit their parents' motion,
face a camera, or follow the camera's position. The renderer evaluates these
rules on the CPU before Bevy propagates transforms. This guide explains pose
evaluation, driving-camera selection, and the limits of camera-dependent geometry.

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

Inheritance flags modify skeletal motion while retaining model-instance
placement. Billboard and camera-anchor flags then use the selected driving
camera. The table describes each flag and how combinations are resolved:

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

## Camera and transform constraints

One driving camera is used per model instance. Rendering the same instance from
multiple cameras uses that same resolved skeleton in every view. Per-view skeletal
poses are not implemented. Separate portrait/preview instances can select different
cameras. Shadows also consume the driving-camera pose. Camera-dependent geometry
can exceed authored mesh/model bounds; conservative bounds remain the application's
responsibility when culling would otherwise hide it.
