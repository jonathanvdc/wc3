# Node transforms and camera controls

The renderer evaluates semantic local poses parent-first and retains full affine
matrices alongside signed rotation and scale factors. Ordinary hierarchy preserves
shear. Pivots, selective inheritance, billboarding, and camera anchoring share one
evaluator across live poses, effect births, event occurrences, and offline baking.

## Driving camera

Set `Playback::driving_camera` explicitly on each camera-sensitive model
instance. There is no automatic main-camera or window-camera selection. Missing
camera input leaves camera-dependent adjustments unavailable while ordinary
animation can still evaluate. One instance has one resolved skeleton for all views;
use separate instances for portrait and world billboards.

Pose evaluation runs in `PostUpdate` under `Wc3Systems::EvaluateNodePoses`, before
transform propagation. Move roots and driving cameras before that set. Authored
camera bindings run earlier in `AnimateCameras`; ordinary application hierarchy
motion is resolved before camera projection updates.

## Inheritance and native conventions

Translation, rotation, and scale inheritance remain independently selectable.
Compensation retains instance placement and the node's own authored motion. Warcraft
III native controls preserve their facing convention: full billboarding applies the
native facing conversion and authored rotation, and takes precedence over axis
locks. Otherwise X, then Y, then Z lock precedence applies; native X lock also
changes local Z scale sign.

M2 spherical billboards use their own native frame: geometry in the YZ plane faces
the camera along +X, while native +Z follows camera up. Camera orientation replaces
inherited rotation, then authored local rotation applies within that frame. The
transformed pivot stays fixed and each inherited basis length is retained, including
signed and collapsed scale factors. M2 axis locks still use generic rotation-lock
approximations and emit a diagnostic; their native mirroring and camera-frame
reconstruction are not implemented. Generic semantic billboards retain their +Z
facing convention.

Camera anchoring adds the driving camera position once along an inherited chain.
Repeated anchored descendants account for the already inherited offset. It does not
itself force a node to face the camera.

Signed factors retain mirrored scales. A collapsed parent axis cannot be inverted
exactly; finite fallbacks keep it collapsed. Exception nodes that need affine
compensation cannot preserve arbitrary shear in a single Bevy Transform. Exact
native billboard appearance beyond the established facing and pivot rules remains
unverified. M2 spherical orientation follows the driving camera frame.

## Occurrence-time evaluation

Emitter and event sampling evaluates authored hierarchy at each occurrence,
including nonlinear parent animation and transition progress. Camera pose is held at
current update input. World-space births retain their sampled frame; local particles
follow the live emitter. Application root motion for effects interpolates observed
update endpoints rather than reconstructing unobserved movement.
